#!/usr/bin/env python3
"""Copy SDK source bytes from a Docker tar only after authoritative manifest checks.

This is source acquisition, not execution evidence. The caller independently
verifies the entire copied source tree before running the installed SDK.
"""
import hashlib
import io
import json
import os
from pathlib import Path
import posixpath
import re
import stat
import sys
import tarfile


def require(condition, message):
    if not condition:
        raise ValueError(message)


def bounded_bytes(path, maximum):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        before = os.fstat(fd)
        require(stat.S_ISREG(before.st_mode) and before.st_size <= maximum,
                "bounded regular acquisition input required")
        with os.fdopen(fd, "rb", closefd=False) as source:
            data = source.read(maximum + 1)
        after = os.fstat(fd)
        require(len(data) == before.st_size and len(data) <= maximum and
                (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) ==
                (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns),
                "acquisition input changed")
        return data
    finally:
        os.close(fd)


def safe_path(value):
    require(isinstance(value, str) and value and not value.startswith("/") and
            re.fullmatch(r"[ -~]+", value) and "\\" not in value and
            all(part not in ("", ".", "..") for part in value.split("/")),
            "unsafe source member path")
    return value


def source_capture(manifest_path):
    manifest = json.loads(bounded_bytes(manifest_path, 64 * 1024 * 1024))
    require(set(manifest) == {"revision", "files"} and
            re.fullmatch(r"[0-9a-f]{40}", manifest["revision"]),
            "authoritative source capture required")
    require(isinstance(manifest["files"], list) and len(manifest["files"]) <= 100000,
            "bounded source capture required")
    expected = {}
    aliases = set()
    for row in manifest["files"]:
        name = row["path"][:-1] if row["kind"] == "directory" else row["path"]
        safe_path(name)
        require(name not in expected, "duplicate captured source path")
        expected[name] = row
        if row["kind"] == "symlink":
            aliases.add(name)
    return expected, aliases


def acquire(archive, destination, selected, capture, aliases):
    safe_path(selected)
    expected = {name: row for name, row in capture.items()
                if name == selected or name.startswith(selected + "/")}
    require(selected in expected, "selected source root absent from capture")
    parent = posixpath.dirname(selected)
    entries = {}
    data = bounded_bytes(archive, 256 * 1024 * 1024)
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:") as source:
        for member in source:
            require(len(entries) < 100000, "source archive entry bound exceeded")
            require(member.type in (tarfile.REGTYPE, tarfile.AREGTYPE, tarfile.DIRTYPE, tarfile.SYMTYPE)
                    and member.sparse is None, "special source archive member refused")
            name = member.name[:-1] if member.isdir() and member.name.endswith("/") else member.name
            safe_path(name)
            logical = posixpath.join(parent, name) if parent else name
            require(logical not in entries, "duplicate source archive member")
            require(logical in expected, "unexpected source archive member")
            for ancestor in Path(logical).parents:
                require(ancestor.as_posix() not in aliases, "source member beneath symlink ancestor")
            row = expected[logical]
            kind = "directory" if member.isdir() else "symlink" if member.issym() else "file"
            require(kind == row["kind"], "source archive member type differs")
            payload = None
            if kind == "file":
                require(member.size == row["size"] and 0 <= member.size <= 64 * 1024 * 1024,
                        "source archive member size differs")
                payload = source.extractfile(member).read(member.size + 1)
                require(len(payload) == member.size and hashlib.sha256(payload).hexdigest() == row["sha256"],
                        "source archive member digest differs")
            else:
                require(member.size == 0, "non-file source member has payload")
                if kind == "symlink":
                    require(member.linkname == row["target"] and not member.linkname.startswith("/"),
                            "source archive alias differs")
                    target = posixpath.normpath(posixpath.join(posixpath.dirname(logical), member.linkname))
                    safe_path(target)
            entries[logical] = (kind, payload, member.linkname)
    require(set(entries) == set(expected), "source archive members omitted")
    root = Path(destination)
    require(root.is_dir() and not root.is_symlink(), "source destination is not a private directory")
    # Check every destination and ancestor before creating any archive output.
    for logical, (kind, _, _) in entries.items():
        path = root / logical
        for ancestor in path.parents:
            if ancestor == root:
                break
            if ancestor.exists() or ancestor.is_symlink():
                require(ancestor.is_dir() and not ancestor.is_symlink(), "destination ancestor is not a directory")
        if path.exists() or path.is_symlink():
            require(kind == "directory" and path.is_dir() and not path.is_symlink(),
                    "source destination already exists")
    # Do not restore image ownership/modes or apply archive metadata. These
    # private comparison copies never execute; the real SDK stays immutable.
    for logical, (kind, payload, target) in sorted(entries.items()):
        path = root / logical
        path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
        if kind == "directory":
            path.mkdir(exist_ok=True, mode=0o700)
        elif kind == "symlink":
            path.symlink_to(target)
        else:
            fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
            with os.fdopen(fd, "wb") as output:
                output.write(payload)


if __name__ == "__main__":
    require(len(sys.argv) >= 5 and len(sys.argv) % 2 == 1,
            "usage: binary-sdk-source-extract.py DESTINATION SOURCE_CAPTURE ARCHIVE SELECTED_ROOT [ARCHIVE SELECTED_ROOT ...]")
    destination, manifest_path = sys.argv[1:3]
    capture, aliases = source_capture(manifest_path)
    pairs = list(zip(sys.argv[3::2], sys.argv[4::2]))
    require(len(pairs) <= 1000, "source root bound exceeded")
    selected_roots = [safe_path(selected) for _, selected in pairs]
    require(len(set(selected_roots)) == len(selected_roots), "duplicate selected source root")
    selected_members = [name for selected in selected_roots for name in capture
                        if name == selected or name.startswith(selected + "/")]
    require(len(set(selected_members)) == len(selected_members) and
            set(selected_members) == set(capture), "complete non-overlapping source roots required")
    for archive, selected in pairs:
        acquire(archive, destination, selected, capture, aliases)
