"""Linux SDK helper custody: sealed executable bytes and owned-child reaping."""
import ctypes
import fcntl
import hashlib
import json
import os
import selectors
import signal
import stat
import subprocess
import sys
import time


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sealed_executable(row, header):
    require(os.path.realpath(row["executable"]) == row["executable"], "aliased helper")
    source = os.open(row["executable"], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    sealed = os.memfd_create("prismpm-credential-helper", os.MFD_ALLOW_SEALING)
    try:
        before = os.fstat(source)
        require(stat.S_ISREG(before.st_mode) and 0 < before.st_size <= 128 * 1024 * 1024,
                "invalid helper file")
        require(before.st_mode & 0o111 and not before.st_mode & 0o022, "unsafe helper mode")
        digest = hashlib.sha256()
        total = 0
        while True:
            part = os.read(source, 65536)
            if not part:
                break
            total += len(part)
            require(total <= before.st_size, "helper grew")
            digest.update(part)
            view = memoryview(part)
            while view:
                view = view[os.write(sealed, view):]
        require(total == before.st_size and digest.hexdigest() == row["sha256"], "helper digest differs")
        after = os.fstat(source)
        require(all(getattr(before, key) == getattr(after, key) for key in
                    ("st_dev", "st_ino", "st_size", "st_mode", "st_mtime_ns", "st_ctime_ns")), "helper changed")
        os.fchmod(sealed, 0o500)
        fcntl.fcntl(sealed, fcntl.F_ADD_SEALS,
                    fcntl.F_SEAL_SEAL | fcntl.F_SEAL_SHRINK | fcntl.F_SEAL_GROW | fcntl.F_SEAL_WRITE)
        os.lseek(sealed, 0, os.SEEK_SET)
        observed = os.read(sealed, 256)
        if observed.startswith(b"#!"):
            require(observed.split(b"\n", 1)[0].decode("utf-8") == header, "helper interpreter changed")
        else:
            require(header is None, "helper executable kind changed")
        os.lseek(sealed, 0, os.SEEK_SET)
        return sealed
    except BaseException:
        os.close(sealed)
        raise
    finally:
        os.close(source)


def children():
    with open(f"/proc/self/task/{os.getpid()}/children", encoding="ascii") as stream:
        return [int(value) for value in stream.read(65536).split()]


def cleanup(process):
    # Subreaping retains ownership even when descendants create new sessions.
    deadline = time.monotonic() + 2
    while True:
        process.poll()
        for pid in children():
            try:
                os.kill(pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        while True:
            try:
                pid, _ = os.waitpid(-1, os.WNOHANG)
                if not pid:
                    break
            except ChildProcessError:
                break
        if not children():
            return
        require(time.monotonic() < deadline, "credential helper cleanup not established")
        time.sleep(0.005)


def main():
    libc = ctypes.CDLL(None, use_errno=True)
    require(libc.prctl(36, 1, 0, 0, 0) == 0, "SDK child subreaper unavailable")
    interrupted = False

    def stop(_signum, _frame):
        nonlocal interrupted
        interrupted = True

    for name in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(name, stop)
    raw = sys.stdin.buffer.read(65537)
    require(len(raw) <= 65536, "helper request exceeds bound")
    request = json.loads(raw)
    server = request["server"]
    require(isinstance(server, str) and 0 < len(server) <= 4096 and not any(c in server for c in "\r\n\0"), "invalid server")
    fd = sealed_executable(request["command"], request["header"])
    process = None
    selector = selectors.DefaultSelector()
    try:
        require(not interrupted, "credential helper canceled")
        arguments = ["/proc/self/fd/" + str(fd), "get"]
        interpreter = request["interpreter"]
        if interpreter is not None:
            arguments = [interpreter["executable"], *interpreter["args"], *arguments]
        process = subprocess.Popen(arguments, pass_fds=(fd,),
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   start_new_session=True, env=request["environment"])
        process.stdin.write((server + "\n").encode())
        process.stdin.close()
        for stream in (process.stdout, process.stderr):
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, selectors.EVENT_READ)
        output = bytearray()
        total = 0
        deadline = time.monotonic() + 12
        while selector.get_map() or process.poll() is None:
            require(not interrupted and time.monotonic() < deadline, "credential helper deadline exceeded")
            for event, _ in selector.select(0.02):
                part = os.read(event.fileobj.fileno(), 65536)
                if not part:
                    selector.unregister(event.fileobj)
                    continue
                total += len(part)
                require(total <= 65536, "credential helper output exceeds bound")
                if event.fileobj is process.stdout:
                    output.extend(part)
        code = process.wait()
        if code == 1 and output == b"credentials not found in native keychain\n":
            result = None
        else:
            require(code == 0, "credential helper failed")
            # JS performs strict duplicate-key/schema validation on these bytes.
            result = output.decode("utf-8", errors="strict")
    finally:
        selector.close()
        os.close(fd)
        if process is not None:
            cleanup(process)
            for stream in (process.stdout, process.stderr):
                stream.close()
    require(not interrupted, "credential helper deadline exceeded")
    sys.stdout.write(json.dumps({"output": result}, separators=(",", ":")))


if __name__ == "__main__":
    try:
        main()
    except BaseException as error:
        message = str(error) if type(error) is RuntimeError else "credential helper execution failed"
        sys.stderr.write(message + "\n")
        sys.exit(1)
