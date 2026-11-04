"""Read-only native experiment observations; unavailable measurements fail closed.

The allocated walk is an interval observation of a live filesystem, not an
instantaneous/absolute peak. Backing allocation and inner filesystem usage are
reported separately by device. Only the driver terminates experiment processes.
"""
import argparse
import hashlib
import json
import os
import re
import signal
import stat
import subprocess
import sys
import tempfile
import time
import uuid

GIB = 1024 ** 3
STORE_BYTES = 40 * GIB
BACKING_RESERVE = 12 * GIB
OUTSIDE_LIMIT = 2 * GIB
INNER_STOP = GIB
HOST_MEMORY_STOP = GIB
SCAN_SECONDS = 30
SCAN_ENTRIES = 2_000_000
SAMPLE_LIMIT = 64 * 1024 ** 2
JSON_LIMIT = 1024 ** 2
BUILDKIT = "moby/buildkit@sha256:de10faf919fc71ba4eb1dd7bd6449566d012b0c9436b1c61bfee21d621b009aa"
PLAN_KEYS = {"schema", "phase", "arch", "source_revision", "run_id", "run_attempt",
             "work_end_ms", "whole_end_ms", "output_root", "store_root", "backing",
             "loop_device", "docker_root", "participating_paths", "owner", "finish_file"}
PLAN_KEYS.add("outside_baseline")
OWNER_KEYS = {"name", "cid", "label_key", "label_value", "image_id", "memory_bytes",
              "state", "run", "config_root"}


class Unavailable(Exception):
    def __init__(self, reason, **details):
        super().__init__(reason)
        self.reason = reason
        self.details = details


def need(condition, reason, **details):
    if not condition:
        raise Unavailable(reason, **details)


def integer(value, minimum=0):
    return type(value) is int and value >= minimum


def closed(value, fields, reason):
    need(isinstance(value, dict) and set(value) == fields, reason)


def absolute(path):
    need(isinstance(path, str) and 0 < len(path) <= 4096 and "\0" not in path
         and path.startswith("/") and os.path.normpath(path) == path,
         "noncanonical_plan_path")
    return path


def beneath(path, root):
    return path == root or path.startswith(root.rstrip("/") + "/")


def validate_plan(plan):
    closed(plan, PLAN_KEYS, "plan_fields")
    need(plan["schema"] == "prismpm/native-resource-plan/1", "plan_schema")
    need(plan["arch"] in ("amd64", "arm64"), "plan_architecture")
    need(isinstance(plan["phase"], str) and re.fullmatch(r"[A-Za-z0-9_-]{1,40}", plan["phase"]), "plan_phase")
    need(isinstance(plan["source_revision"], str) and re.fullmatch(r"[0-9a-f]{40}", plan["source_revision"]), "plan_revision")
    for key in ("run_id", "run_attempt"):
        need(isinstance(plan[key], str) and re.fullmatch(r"[1-9][0-9]{0,19}", plan[key]), "plan_run_identity")
    need(integer(plan["work_end_ms"], 1) and integer(plan["whole_end_ms"], 1)
         and plan["work_end_ms"] <= plan["whole_end_ms"], "plan_deadlines")
    for key in ("output_root", "store_root", "loop_device", "docker_root", "finish_file"):
        absolute(plan[key])
    need(plan["store_root"] != "/" and plan["docker_root"] != "/" and plan["output_root"] != "/", "unsafe_root")
    need(not beneath(plan["output_root"], plan["store_root"]), "output_must_be_outside_store")
    need(beneath(plan["finish_file"], plan["output_root"]), "finish_outside_output_root")
    need(plan["finish_file"] != plan["output_root"], "finish_is_output_directory")
    reference = plan["outside_baseline"]
    if reference is not None:
        closed(reference, {"path", "sha256"}, "outside_baseline_fields")
        absolute(reference["path"])
        need(beneath(reference["path"], plan["output_root"]) and reference["path"] != plan["output_root"]
             and isinstance(reference["sha256"], str) and re.fullmatch(r"[0-9a-f]{64}", reference["sha256"]), "outside_baseline_reference")
    closed(plan["backing"], {"path", "dev", "ino", "size", "blocks_min"}, "backing_fields")
    absolute(plan["backing"]["path"])
    for key in ("dev", "ino", "size", "blocks_min"):
        need(integer(plan["backing"][key], 1 if key == "ino" else 0), "backing_identity")
    need(plan["backing"]["size"] == STORE_BYTES and plan["backing"]["blocks_min"] == STORE_BYTES,
         "backing_must_be_fully_allocated_40gib")
    need(not beneath(plan["backing"]["path"], plan["store_root"]), "backing_inside_store")
    paths = plan["participating_paths"]
    need(isinstance(paths, list) and 0 < len(paths) <= 128 and all(isinstance(p, str) for p in paths)
         and len(set(paths)) == len(paths), "participating_paths")
    for path in paths:
        absolute(path)
        need(beneath(path, plan["store_root"]), "participant_outside_hard_store")
    owner = plan["owner"]
    if owner is not None:
        closed(owner, OWNER_KEYS, "owner_fields")
        need(isinstance(owner["name"], str) and re.fullmatch(r"buildx_buildkit_[A-Za-z0-9_.-]{1,160}0", owner["name"]), "owner_name")
        need(isinstance(owner["cid"], str) and re.fullmatch(r"[0-9a-f]{64}", owner["cid"]), "owner_cid")
        need(isinstance(owner["image_id"], str) and re.fullmatch(r"sha256:[0-9a-f]{64}", owner["image_id"]), "owner_image")
        need(owner["label_key"] == "org.prismpm.native-resource-owner"
             and isinstance(owner["label_value"], str)
             and re.fullmatch(r"[A-Za-z0-9_-]{1,100}", owner["label_value"]), "owner_label")
        need(integer(owner["memory_bytes"], 1), "owner_memory")
        for key in ("state", "run", "config_root"):
            absolute(owner[key])
            need(beneath(owner[key], plan["store_root"]) and owner[key] != plan["store_root"], "owner_path_outside_store")
        need(len({owner[k] for k in ("state", "run", "config_root")}) == 3, "owner_paths_overlap")
        own_paths = [owner[k] for k in ("state", "run", "config_root")]
        need(not any(a != b and beneath(a, b) for a in own_paths for b in own_paths), "owner_paths_overlap")
    return plan


def identity(value):
    return (value.st_dev, value.st_ino, stat.S_IFMT(value.st_mode))


def allocation(value):
    need(hasattr(value, "st_blocks") and integer(value.st_blocks), "allocated_blocks_unavailable")
    return value.st_blocks * 512


def receipt(plan):
    return {key: plan[key] for key in ("source_revision", "run_id", "run_attempt", "phase", "arch")}


def bounded_json(path, limit=JSON_LIMIT, with_bytes=False):
    before = os.lstat(path)
    need(stat.S_ISREG(before.st_mode) and before.st_nlink == 1 and before.st_size <= limit,
         "json_not_bounded_independent_file")
    need(os.path.realpath(path) == path, "json_path_aliased")
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        need(identity(os.fstat(fd)) == identity(before), "json_identity_changed")
        with os.fdopen(fd, "rb", closefd=False) as stream:
            raw = stream.read(limit + 1)
        after = os.fstat(fd)
        need(len(raw) <= limit and len(raw) == before.st_size and before.st_mtime_ns == after.st_mtime_ns
             and before.st_ctime_ns == after.st_ctime_ns and identity(os.lstat(path)) == identity(before),
             "json_changed_during_read")
        value = json.loads(raw)
        return (value, raw) if with_bytes else value
    finally:
        os.close(fd)


class System:
    def __init__(self, temporary_root=None):
        self.temporary_root = temporary_root

    lstat = staticmethod(os.lstat)
    realpath = staticmethod(os.path.realpath)
    statvfs = staticmethod(os.statvfs)
    monotonic = staticmethod(time.monotonic)
    now_ms = staticmethod(lambda: time.time_ns() // 1_000_000)
    sleep = staticmethod(time.sleep)

    @staticmethod
    def baseline(reference):
        value, raw = bounded_json(reference["path"], with_bytes=True)
        need(hashlib.sha256(raw).hexdigest() == reference["sha256"], "outside_baseline_digest_changed")
        return value

    @staticmethod
    def store_empty(path):
        def fail(error):
            raise error
        count = 0
        for directory, names, files in os.walk(path, followlinks=False, onerror=fail):
            count += len(names) + len(files) + 1
            need(count <= SCAN_ENTRIES and not files
                 and all(not os.path.islink(os.path.join(directory, name)) for name in names), "baseline_store_not_empty")
        need(count > 0, "baseline_store_unavailable")
        return True

    @staticmethod
    def text(path, maximum=65536):
        # Kernel files only. Files from DockerRootDir are never opened/read.
        with open(path, "rb") as stream:
            raw = stream.read(maximum + 1)
        need(len(raw) <= maximum, "kernel_observation_exceeds_bound")
        return raw.decode("ascii")

    def docker(self, args):
        environment = os.environ.copy()
        for key in ("DOCKER_HOST", "DOCKER_CONTEXT", "DOCKER_TLS", "DOCKER_TLS_VERIFY", "DOCKER_CERT_PATH"):
            environment.pop(key, None)
        # No registry calls, endpoint discovery, environment/config dumps or shell.
        with tempfile.TemporaryFile(dir=self.temporary_root) as output, tempfile.TemporaryFile(dir=self.temporary_root) as errors:
            child = subprocess.run(["docker", "--host=unix:///var/run/docker.sock", *args],
                                   env=environment, stdin=subprocess.DEVNULL,
                                   stdout=output, stderr=errors, timeout=5, check=False)
            need(child.returncode == 0, "docker_observation_failed", status=child.returncode)
            need(output.tell() <= JSON_LIMIT and errors.tell() <= JSON_LIMIT, "docker_observation_exceeds_bound")
            output.seek(0)
            return json.load(output)


def canonical_directory(path, system):
    need(system.realpath(path) == path, "directory_path_aliased")
    value = system.lstat(path)
    need(stat.S_ISDIR(value.st_mode), "directory_required")
    return value


def parse_mountinfo(text):
    result = []
    def decode(value):
        return re.sub(r"\\([0-7]{3})", lambda m: chr(int(m[1], 8)), value)
    for line in text.splitlines():
        row = line.split()
        need("-" in row and len(row) >= 10, "mountinfo_unavailable")
        split = row.index("-")
        need(split >= 6 and len(row) >= split + 4 and re.fullmatch(r"[0-9]+:[0-9]+", row[2]), "mountinfo_unavailable")
        result.append({"id": int(row[0]), "device": row[2], "root": decode(row[3]),
                       "path": decode(row[4]), "options": set(row[5].split(",")),
                       "type": row[split + 1], "source": decode(row[split + 2]),
                       "super_options": set(row[split + 3].split(","))})
    need(result, "mountinfo_unavailable")
    return result


def backing_and_mount(plan, system, mounts):
    backing = plan["backing"]
    need(system.realpath(backing["path"]) == backing["path"], "backing_path_aliased")
    value = system.lstat(backing["path"])
    need(stat.S_ISREG(value.st_mode) and value.st_nlink == 1, "backing_not_independent_file")
    need(value.st_dev == backing["dev"] and value.st_ino == backing["ino"] and value.st_size == backing["size"],
         "backing_identity_changed")
    blocks = allocation(value)
    need(blocks >= backing["blocks_min"], "backing_hole", allocated_bytes=blocks, required_bytes=backing["blocks_min"])
    loop = system.lstat(plan["loop_device"])
    need(stat.S_ISBLK(loop.st_mode), "loop_device_not_block")
    device = f"{os.major(loop.st_rdev)}:{os.minor(loop.st_rdev)}"
    candidates = [m for m in mounts if m["path"] == plan["store_root"]]
    need(len(candidates) == 1, "store_mount_missing_or_ambiguous")
    mount = candidates[0]
    need(mount["type"] == "ext4" and mount["device"] == device and mount["root"] == "/"
         and mount["source"] == plan["loop_device"] and "rw" in mount["options"], "store_mount_changed")
    need(not any(option.startswith("discard") for option in mount["options"] | mount["super_options"]), "discard_not_permitted")
    need(not any(m["path"] != plan["store_root"] and beneath(m["path"], plan["store_root"]) for m in mounts), "nested_mount_in_store")
    store = canonical_directory(plan["store_root"], system)
    need(f"{os.major(store.st_dev)}:{os.minor(store.st_dev)}" == device, "store_device_changed")
    actual = system.text(f"/sys/dev/block/{device}/loop/backing_file", 8192).strip()
    actual = "/" + actual.lstrip("/")
    need(actual == backing["path"], "loop_backing_changed")
    need(identity(system.lstat(actual)) == identity(value), "loop_backing_identity_changed")
    return {"dev": value.st_dev, "ino": value.st_ino, "size_bytes": value.st_size,
            "allocated_bytes": blocks, "loop_device": device, "mount_id": mount["id"]}


def filesystems(paths, system):
    records, observations = {}, []
    for path in dict.fromkeys(paths):
        need(system.realpath(path) == path, "filesystem_path_aliased")
        before = system.lstat(path)
        value = system.statvfs(path)
        unit = value.f_frsize or value.f_bsize
        need(integer(unit, 1) and all(integer(x) for x in (value.f_blocks, value.f_bfree, value.f_bavail)), "filesystem_capacity_unavailable")
        after = system.lstat(path)
        need(identity(before) == identity(after), "filesystem_path_changed")
        key = str(before.st_dev)
        row = {"dev": before.st_dev, "device": f"{os.major(before.st_dev)}:{os.minor(before.st_dev)}",
               "capacity_bytes": value.f_blocks * unit, "free_bytes": value.f_bfree * unit,
               "available_bytes": value.f_bavail * unit, "paths": []}
        need(row["available_bytes"] <= row["free_bytes"] <= row["capacity_bytes"], "filesystem_capacity_inconsistent")
        if key in records:
            # Capacity may change during the sample; keep the lowest available.
            prior = records[key]
            need(prior["capacity_bytes"] == row["capacity_bytes"], "filesystem_capacity_changed")
            prior["available_bytes"] = min(prior["available_bytes"], row["available_bytes"])
            prior["free_bytes"] = min(prior["free_bytes"], row["free_bytes"])
        else:
            records[key] = row
        records[key]["paths"].append(path)
        observations.append({"path": path, "dev": before.st_dev, "ino": before.st_ino})
    return {"devices": list(records.values()), "paths": observations}


def numeric_fields(text):
    result = {}
    for line in text.splitlines():
        row = line.split()
        need(len(row) == 2 and re.fullmatch(r"[a-z][a-z0-9_.]*", row[0]) and row[1].isdigit() and row[0] not in result,
             "kernel_counter_unavailable")
        result[row[0]] = int(row[1])
    need(0 < len(result) <= 128, "kernel_counter_unavailable")
    return result


def memory_info(text):
    result = {}
    for line in text.splitlines():
        row = line.split()
        if row and row[0] in ("MemTotal:", "MemAvailable:"):
            need(len(row) == 3 and row[1].isdigit() and row[2] == "kB", "host_memory_unavailable")
            need(row[0] not in result, "host_memory_unavailable")
            result[row[0]] = int(row[1]) * 1024
    need(set(result) == {"MemTotal:", "MemAvailable:"}, "host_memory_unavailable")
    need(result["MemTotal:"] >= result["MemAvailable:"] >= HOST_MEMORY_STOP,
         "host_memory_reserve", available_bytes=result["MemAvailable:"])
    return {"total_bytes": result["MemTotal:"], "available_bytes": result["MemAvailable:"]}


def graph_hash(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def configuration_graph(value):
    return {"Config": value["Config"], "HostConfig": value["HostConfig"], "Image": value["Image"],
            "Mounts": sorted(value["Mounts"], key=lambda row: row["Destination"])}


def load_baseline(plan, system):
    need(plan["outside_baseline"] is not None, "outside_baseline_missing")
    value = system.baseline(plan["outside_baseline"])
    need(value.get("schema") == "prismpm/native-resource-baseline/1" and value.get("complete") is True
         and all(value.get(k) == v for k, v in receipt(plan).items()), "outside_baseline_identity")
    for sample in (value["fast"], value["final_fast"]):
        need(sample["owner"] is None and sample["docker_root"] == plan["docker_root"], "outside_baseline_owner_or_docker_root")
        need(all(sample["backing"][k] == plan["backing"][p] for k, p in (("dev", "dev"), ("ino", "ino"), ("size_bytes", "size"))), "outside_baseline_backing_identity")
    allocation_row = value["allocation"]
    need(allocation_row.get("complete") is True, "outside_baseline_incomplete")
    devices = allocation_row["outside_allocated_bytes_by_dev"]
    need(isinstance(devices, dict) and 0 < len(devices) <= 128
         and all(isinstance(k, str) and k.isdigit() and str(int(k)) == k and integer(v) for k, v in devices.items()), "outside_baseline_allocation")
    need(sum(devices.values()) == allocation_row["outside_allocated_bytes"], "outside_baseline_total")
    return devices


def inspect_owner(plan, system, memory, mounts):
    owner = plan["owner"]
    if owner is None:
        return None
    value = system.docker(["container", "inspect", "--format", "{{json .}}", owner["cid"]])
    image = system.docker(["image", "inspect", "--format", "{{json .}}", owner["image_id"]])
    need(value["Id"] == owner["cid"] and value["Name"] == "/" + owner["name"]
         and value["Image"] == owner["image_id"] and image["Id"] == owner["image_id"], "owner_identity_changed")
    need(image["Architecture"] == plan["arch"] and image["Os"] == "linux", "owner_image_platform")
    need(isinstance(image.get("RepoDigests"), list) and any(d.split("@")[-1] == BUILDKIT.split("@")[1] for d in image["RepoDigests"]), "owner_image_digest_unbound")
    config, expected, host = value["Config"], image["Config"], value["HostConfig"]
    state = value["State"]
    need(state["Running"] is True and integer(state["Pid"], 2) and state.get("OOMKilled") is False
         and state.get("Restarting") is False and state.get("Dead") is False, "owner_not_running")
    pid = state["Pid"]
    need(config["Image"] in (owner["image_id"], BUILDKIT, "docker.io/" + BUILDKIT), "owner_config_image")
    for key in ("Entrypoint", "Cmd", "User", "WorkingDir", "Healthcheck", "Env", "Volumes", "ExposedPorts", "StopSignal", "StopTimeout"):
        need(config.get(key) == expected.get(key), "owner_image_defaults_changed")
    labels = dict(expected.get("Labels") or {})
    labels[owner["label_key"]] = owner["label_value"]
    need(config.get("Labels") == labels, "owner_labels_changed")
    cap = min(12 * GIB, memory["total_bytes"] - 2 * GIB)
    need(cap > 0 and owner["memory_bytes"] == cap, "owner_memory_plan")
    required = {"Privileged": True, "ReadonlyRootfs": True, "Init": True,
                "NanoCpus": 2_000_000_000, "Memory": cap, "MemorySwap": cap, "PidsLimit": 4096,
                "NetworkMode": "bridge"}
    for key, expected_value in required.items():
        need(type(host.get(key)) is type(expected_value) and host[key] == expected_value,
             "owner_limit_or_config_changed", field=key)
    need(host.get("LogConfig") == {"Type": "local", "Config": {"max-size": "16m", "max-file": "2"}}, "owner_logs_unbounded")
    need(not host.get("Binds") and not host.get("VolumesFrom") and not host.get("Devices")
         and not host.get("DeviceRequests") and not host.get("CapAdd") and not host.get("CapDrop")
         and not host.get("PidMode") and host.get("IpcMode") in (None, "", "private")
         and not host.get("CgroupParent"), "owner_extra_configuration")
    tmpfs = host.get("Tmpfs")
    need(isinstance(tmpfs, dict) and set(tmpfs) == {"/tmp"}, "owner_tmpfs_changed")
    need(set(tmpfs["/tmp"].split(",")) == {"rw", "noexec", "nosuid", "nodev", "size=268435456"}, "owner_tmpfs_changed")
    observed_mounts = value.get("Mounts")
    need(isinstance(observed_mounts, list) and len(observed_mounts) == 2, "owner_mount_count")
    intended = {"/var/lib/buildkit": owner["state"], "/run": owner["run"]}
    bound = []
    namespace = parse_mountinfo(system.text(f"/proc/{pid}/mountinfo", 4 * JSON_LIMIT))
    root_mount = [row for row in namespace if row["path"] == "/"]
    tmp_mount = [row for row in namespace if row["path"] == "/tmp"]
    need(len(root_mount) == 1 and "ro" in root_mount[0]["options"], "owner_root_not_readonly")
    need(len(tmp_mount) == 1 and tmp_mount[0]["type"] == "tmpfs"
         and {"rw", "noexec", "nosuid", "nodev"} <= tmp_mount[0]["options"], "owner_kernel_tmpfs_changed")
    for mount in observed_mounts:
        target = mount["Destination"]
        need(target in intended and mount["Type"] == "bind" and mount["Source"] == intended.pop(target)
             and mount["RW"] is True and mount.get("Propagation", "rprivate") == "rprivate", "owner_mount_changed")
        source = canonical_directory(mount["Source"], system)
        need(system.lstat(mount["Source"]).st_dev == system.lstat(plan["store_root"]).st_dev, "owner_state_outside_store_filesystem")
        target_inode = system.lstat(f"/proc/{pid}/root{target}")
        need(identity(target_inode) == identity(source), "owner_bind_source_identity_changed")
        kernel_mount = [row for row in namespace if row["path"] == target]
        need(len(kernel_mount) == 1 and kernel_mount[0]["device"] == f"{os.major(source.st_dev)}:{os.minor(source.st_dev)}"
             and kernel_mount[0]["root"] == "/" + os.path.relpath(mount["Source"], plan["store_root"])
             and "rw" in kernel_mount[0]["options"], "owner_kernel_bind_changed")
        bound.append({"source": mount["Source"], "target": target, "dev": source.st_dev,
                      "ino": source.st_ino, "mount_id": kernel_mount[0]["id"]})
    need(not intended, "owner_mount_missing")
    for key in ("state", "run", "config_root"):
        canonical_directory(owner[key], system)
    tmp_path = f"/proc/{pid}/root/tmp"
    tmp_inode = system.lstat(tmp_path)
    tmp_capacity = system.statvfs(tmp_path)
    unit = tmp_capacity.f_frsize or tmp_capacity.f_bsize
    need(stat.S_ISDIR(tmp_inode.st_mode) and integer(unit, 1)
         and tmp_capacity.f_blocks * unit == 268435456, "owner_kernel_tmpfs_size_changed")
    tmpfs = {"dev": tmp_inode.st_dev, "ino": tmp_inode.st_ino, "mount_id": tmp_mount[0]["id"],
             "capacity_bytes": tmp_capacity.f_blocks * unit, "available_bytes": tmp_capacity.f_bavail * unit,
             "free_bytes": tmp_capacity.f_bfree * unit}
    need(all(integer(tmpfs[k]) for k in ("capacity_bytes", "free_bytes", "available_bytes"))
         and tmpfs["available_bytes"] <= tmpfs["free_bytes"] <= tmpfs["capacity_bytes"], "owner_kernel_tmpfs_capacity_unavailable")
    process_stat = system.text(f"/proc/{pid}/stat", 65536)
    tail = process_stat[process_stat.rfind(")") + 2:].split()
    need(len(tail) >= 20 and tail[19].isdigit(), "owner_pid_identity_unavailable")
    start = int(tail[19])
    rows = system.text(f"/proc/{pid}/cgroup").splitlines()
    need(len(rows) == 1 and rows[0].startswith("0::/"), "cgroup_v2_required")
    membership = rows[0][3:]
    absolute(membership)
    controllers = [m for m in mounts if m["type"] == "cgroup2" and m["path"] == "/sys/fs/cgroup"]
    need(len(controllers) == 1 and beneath(membership, controllers[0]["root"]), "cgroup_mount_unavailable")
    relative = os.path.relpath(membership, controllers[0]["root"])
    cgroup = os.path.normpath(os.path.join("/sys/fs/cgroup", relative))
    need(beneath(cgroup, "/sys/fs/cgroup") and cgroup != "/sys/fs/cgroup", "owner_cgroup_not_private")
    group = canonical_directory(cgroup, system)
    group_parent_path = os.path.dirname(cgroup)
    group_parent = canonical_directory(group_parent_path, system)
    def number(name):
        text = system.text(cgroup + "/" + name, 65536).strip()
        need(text.isdigit(), "cgroup_limit_unavailable", field=name)
        return int(text)
    counters = {name: number(name) for name in ("memory.current", "memory.peak", "memory.max", "memory.swap.max", "pids.current", "pids.max")}
    need(counters["memory.max"] == cap and counters["memory.swap.max"] == 0 and counters["pids.max"] == 4096,
         "kernel_limits_changed")
    need(counters["memory.current"] <= cap and counters["pids.current"] <= 4096, "kernel_limit_exceeded")
    need(counters["memory.peak"] >= counters["memory.current"], "kernel_memory_peak_inconsistent")
    cpu = system.text(cgroup + "/cpu.max").split()
    need(len(cpu) == 2 and all(v.isdigit() for v in cpu) and int(cpu[1]) > 0 and int(cpu[0]) == 2 * int(cpu[1]), "kernel_cpu_limit_changed")
    events = numeric_fields(system.text(cgroup + "/memory.events"))
    pids = numeric_fields(system.text(cgroup + "/pids.events"))
    cpu_stat = numeric_fields(system.text(cgroup + "/cpu.stat"))
    need({"low", "high", "max", "oom", "oom_kill", "oom_group_kill"} <= set(events) and "max" in pids
         and {"usage_usec", "user_usec", "system_usec"} <= set(cpu_stat), "kernel_events_unavailable")
    need(not any(events[key] for key in ("oom", "oom_kill", "oom_group_kill")), "owner_oom_event")
    need(pids["max"] == 0, "owner_pid_limit_event", events=pids["max"])
    after = system.text(f"/proc/{pid}/stat", 65536)
    need(after[after.rfind(")") + 2:].split()[19] == str(start), "owner_pid_replaced")
    need(system.text(f"/proc/{pid}/cgroup").splitlines() == rows and identity(system.lstat(cgroup)) == identity(group)
         and system.realpath(cgroup) == cgroup, "owner_cgroup_changed")
    # Digest the complete actual config/mount graph without exposing Env values.
    stable_configuration = configuration_graph(value)
    fingerprint = graph_hash({**stable_configuration, "NetworkSettings": value.get("NetworkSettings")})
    namespace_hash = graph_hash({"binds": bound, "root_mount_id": root_mount[0]["id"],
                                 "tmpfs": {k: v for k, v in tmpfs.items() if k not in ("available_bytes", "free_bytes")}})
    return {"cid": owner["cid"], "pid": pid, "pid_start_ticks": start, "cgroup_path": cgroup,
            "cgroup_dev": group.st_dev, "cgroup_ino": group.st_ino, "config_sha256": fingerprint,
            "retirement_config_sha256": graph_hash(stable_configuration),
            "cgroup_parent_path": group_parent_path, "cgroup_parent_dev": group_parent.st_dev,
            "cgroup_parent_ino": group_parent.st_ino, "cgroup_mount_id": controllers[0]["id"],
            "namespace_sha256": namespace_hash, "bind_identities": bound, "tmpfs": tmpfs,
            "memory": counters, "memory_events": events, "pids_events": pids, "cpu_max": list(map(int, cpu)), "cpu_stat": cpu_stat}


class Observer:
    def __init__(self, plan, system=None, establish_baseline=False):
        self.plan = validate_plan(plan)
        self.system = system or System(plan["store_root"])
        self.initial = None
        self.minimum_available = {}
        self.maximum_owner_memory = 0
        self.previous_owner_peak = None
        self.establish_baseline = establish_baseline

    def fast(self):
        plan, system = self.plan, self.system
        started = system.monotonic()
        if self.establish_baseline:
            need(plan["outside_baseline"] is None and plan["owner"] is None, "baseline_must_precede_owners")
        else:
            load_baseline(plan, system)
        now = system.now_ms()
        need(now < plan["whole_end_ms"], "whole_deadline", now_ms=now, deadline_ms=plan["whole_end_ms"])
        need(now < plan["work_end_ms"], "work_deadline", now_ms=now, deadline_ms=plan["work_end_ms"])
        mounts = parse_mountinfo(system.text("/proc/self/mountinfo", 4 * JSON_LIMIT))
        need(any(m["type"] == "cgroup2" and m["path"] == "/sys/fs/cgroup" for m in mounts), "cgroup_v2_required")
        backing = backing_and_mount(plan, system, mounts)
        actual_docker_root = system.docker(["info", "--format", "{{json .DockerRootDir}}"])
        need(actual_docker_root == plan["docker_root"], "docker_root_changed")
        canonical_directory(actual_docker_root, system)
        canonical_directory(plan["output_root"], system)
        paths = [actual_docker_root, os.path.dirname(plan["backing"]["path"]), plan["store_root"], plan["output_root"], *plan["participating_paths"]]
        if plan["owner"]:
            paths += [plan["owner"][key] for key in ("state", "run", "config_root")]
        for path in plan["participating_paths"]:
            canonical_directory(path, system)
            need(system.lstat(path).st_dev == system.lstat(plan["store_root"]).st_dev, "participant_filesystem_changed")
        fs = filesystems(paths, system)
        by_dev = {str(row["dev"]): row for row in fs["devices"]}
        backing_free = by_dev[str(plan["backing"]["dev"])]["available_bytes"]
        inner_free = by_dev[str(system.lstat(plan["store_root"]).st_dev)]["available_bytes"]
        need(backing_free >= BACKING_RESERVE, "backing_reserve", available_bytes=backing_free, required_bytes=BACKING_RESERVE)
        need(inner_free >= INNER_STOP, "inner_available_stop", available_bytes=inner_free, stop_bytes=INNER_STOP)
        inner_device = system.lstat(plan["store_root"]).st_dev
        for row in fs["devices"]:
            if row["dev"] != inner_device:
                need(row["available_bytes"] >= BACKING_RESERVE, "outer_filesystem_reserve", dev=row["dev"],
                     available_bytes=row["available_bytes"], required_bytes=BACKING_RESERVE)
        memory = memory_info(system.text("/proc/meminfo"))
        owner = inspect_owner(plan, system, memory, mounts)
        if owner is not None:
            peak = owner["memory"]["memory.peak"]
            need(self.previous_owner_peak is None or peak >= self.previous_owner_peak, "kernel_memory_peak_regressed")
            self.previous_owner_peak = peak
            self.maximum_owner_memory = max(self.maximum_owner_memory, owner["memory"]["memory.current"])
        custody = {"backing": {k: backing[k] for k in ("dev", "ino", "size_bytes", "loop_device", "mount_id")},
                   "paths": fs["paths"], "owner": None if owner is None else {k: owner[k] for k in ("cid", "pid", "pid_start_ticks", "cgroup_path", "cgroup_dev", "cgroup_ino", "config_sha256", "namespace_sha256")}}
        if self.initial is None:
            self.initial = custody
        need(custody == self.initial, "participating_identity_or_configuration_drift")
        for row in fs["devices"]:
            key = str(row["dev"])
            self.minimum_available[key] = min(self.minimum_available.get(key, row["available_bytes"]), row["available_bytes"])
        self.minimum_available["host_memory"] = min(self.minimum_available.get("host_memory", memory["available_bytes"]), memory["available_bytes"])
        elapsed = system.monotonic() - started
        need(elapsed <= 1, "fast_sample_time_budget", elapsed_seconds=elapsed)
        need(system.now_ms() < plan["work_end_ms"] and system.now_ms() < plan["whole_end_ms"], "deadline_during_sample")
        return {"kind": "fast", "observed_at_ms": now, **receipt(plan), "backing": backing,
                "docker_root": actual_docker_root, "filesystems": fs["devices"], "path_identities": fs["paths"],
                "host_memory": memory, "owner": owner,
                "sampled_maximum_owner_memory_current_bytes": self.maximum_owner_memory if owner is not None else None,
                "sample_elapsed_seconds": elapsed, "sampled_minimum_available_bytes": dict(self.minimum_available),
                "absolute_peak_claimed": False}


def scan_allocations(roots, exclusions=(), system=None, maximum=SCAN_ENTRIES, seconds=SCAN_SECONDS):
    """No file contents or symlink targets are read; aliases count once by inode."""
    system = system or System()
    need(integer(maximum, 1) and maximum <= SCAN_ENTRIES and 0 < seconds <= SCAN_SECONDS, "scan_limits")
    start = system.monotonic()
    seen, devices, excluded = set(), {}, []
    entries = directories = symlinks = 0
    roots = list(dict.fromkeys(absolute(root) for root in roots))
    need(0 < len(roots) <= 128, "scan_roots")
    effective = [p for p in roots if not any(p != other and beneath(p, other) and stat.S_ISDIR(system.lstat(other).st_mode) for other in roots)]
    def budget():
        need(system.monotonic() - start < seconds, "scan_time_budget")
    def count(value, allocated=True):
        nonlocal entries, symlinks
        budget()
        entries += 1
        need(entries <= maximum, "scan_entry_budget", observed_entries=entries, maximum_entries=maximum)
        key = (value.st_dev, value.st_ino)
        if allocated and key not in seen:
            seen.add(key)
            dev = str(value.st_dev)
            need(dev in devices or len(devices) < 128, "scan_device_budget")
            devices[dev] = devices.get(dev, 0) + allocation(value)
        if stat.S_ISLNK(value.st_mode):
            symlinks += 1
    def walk(path, parent=None, name=None, depth=0):
        nonlocal directories
        budget()
        need(depth <= 512, "scan_depth_budget")
        before = system.lstat(path) if parent is None else os.stat(name, dir_fd=parent, follow_symlinks=False)
        if path in exclusions:
            count(before, allocated=False)
            need(stat.S_ISDIR(before.st_mode), "overlay_view_changed")
            excluded.append(path)
            return
        count(before)
        if not stat.S_ISDIR(before.st_mode):
            return
        directories += 1
        fd = os.open(path if parent is None else name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
        try:
            need(identity(os.fstat(fd)) == identity(before), "scan_directory_replaced")
            with os.scandir(fd) as stream:
                for child in stream:
                    budget()
                    walk(os.path.join(path, child.name), fd, child.name, depth + 1)
            need(identity(os.fstat(fd)) == identity(before), "scan_directory_changed")
            after = system.lstat(path) if parent is None else os.stat(name, dir_fd=parent, follow_symlinks=False)
            need(identity(after) == identity(before), "scan_directory_custody_changed")
        finally:
            os.close(fd)
    for root in effective:
        budget()
        need(system.realpath(root) == root, "scan_root_aliased")
        walk(root)
        need(system.realpath(root) == root, "scan_root_changed")
    budget()
    return {"kind": "allocation", "complete": True, "observed_entries": entries, "unique_inodes": len(seen),
            "directories": directories, "symlinks": symlinks, "allocated_bytes_by_dev": devices,
            "requested_roots": roots, "walked_roots": effective, "omitted_merged_overlay_views": excluded,
            "elapsed_seconds": system.monotonic() - start, "instantaneous_snapshot_claimed": False,
            "absolute_peak_claimed": False}


def complete_scan(plan, system=None, establish_baseline=False):
    system = system or System(plan["store_root"])
    before = system.lstat(plan["backing"]["path"])
    mounts = parse_mountinfo(system.text("/proc/self/mountinfo", 4 * JSON_LIMIT))
    exclusions = [m["path"] for m in mounts if m["type"] == "overlay" and m["path"].endswith("/merged")
                  and beneath(m["path"], plan["docker_root"])]
    result = scan_allocations([plan["docker_root"], plan["store_root"], plan["output_root"], plan["backing"]["path"]], exclusions, system)
    backing = system.lstat(plan["backing"]["path"])
    need(identity(before) == identity(backing) and allocation(before) == allocation(backing), "backing_changed_during_scan")
    inner = system.lstat(plan["store_root"]).st_dev
    outside_devices = {dev: n for dev, n in result["allocated_bytes_by_dev"].items() if int(dev) != inner}
    key = str(backing.st_dev)
    need(key in outside_devices and outside_devices[key] >= allocation(backing), "backing_not_in_complete_scan")
    outside_devices[key] -= allocation(backing)
    if establish_baseline:
        need(plan["owner"] is None and plan["outside_baseline"] is None, "baseline_must_precede_owners")
        # Only empty source/client/state directories may exist at admission.
        need(system.store_empty(plan["store_root"]) is True, "baseline_store_not_empty")
        growth = {}
    else:
        baseline = load_baseline(plan, system)
        growth = {dev: max(0, n - baseline[dev]) if dev in baseline else n for dev, n in outside_devices.items()}
        need(sum(growth.values()) <= OUTSIDE_LIMIT, "outside_storage_growth_budget", observed_growth_bytes=sum(growth.values()), maximum_bytes=OUTSIDE_LIMIT)
        result["baseline_outside_allocated_bytes_by_dev"] = baseline
        result["introduced_outside_devices"] = sorted(set(outside_devices) - set(baseline))
    result.update(outside_allocated_bytes=sum(outside_devices.values()), outside_allocated_bytes_by_dev=outside_devices,
                  outside_positive_growth_bytes=sum(growth.values()), outside_positive_growth_bytes_by_dev=growth,
                  observed_at_ms=system.now_ms(), **receipt(plan))
    return result


def fault(error):
    if isinstance(error, Unavailable):
        return {"reason": error.reason, "details": error.details}
    if isinstance(error, OSError):
        return {"reason": "observation_os_error", "details": {"errno": error.errno}}
    if isinstance(error, subprocess.TimeoutExpired):
        return {"reason": "observation_timeout", "details": {"seconds": error.timeout}}
    return {"reason": "observation_unavailable", "details": {"error_type": type(error).__name__}}


def retire(plan, original_report, system=None):
    """Proof for cleanup after stop, not a resource-fit or capacity bypass."""
    system = system or System(plan["store_root"])
    report = {"schema": "prismpm/native-resource-retirement/1", **receipt(plan), "complete": False,
              "scope": "observed-stopped-cid-and-original-cgroup-exhaustion"}
    try:
        validate_plan(plan)
        load_baseline(plan, system)
        need(system.now_ms() < plan["whole_end_ms"], "whole_deadline")
        owner = plan["owner"]
        need(owner is not None and original_report.get("schema") == "prismpm/native-resource-snapshot/1"
             and original_report.get("complete") is True and original_report["allocation"].get("complete") is True
             and all(original_report.get(k) == v for k, v in receipt(plan).items()), "retirement_original_report")
        original = original_report["final_fast"]["owner"]
        need(original["cid"] == owner["cid"] and all(original_report["fast"]["owner"][k] == original[k]
             for k in ("cid", "pid", "pid_start_ticks", "cgroup_path", "cgroup_dev", "cgroup_ino",
                       "config_sha256", "namespace_sha256", "retirement_config_sha256",
                       "cgroup_parent_path", "cgroup_parent_dev", "cgroup_parent_ino", "cgroup_mount_id")), "retirement_original_owner")
        need(system.docker(["info", "--format", "{{json .DockerRootDir}}"] ) == plan["docker_root"], "docker_root_changed")
        def stopped():
            value = system.docker(["container", "inspect", "--format", "{{json .}}", owner["cid"]])
            need(value["Id"] == owner["cid"] and value["Name"] == "/" + owner["name"]
                 and value["Image"] == owner["image_id"] and value["Config"]["Labels"][owner["label_key"]] == owner["label_value"], "retirement_container_identity")
            need(value["State"]["Running"] is False and value["State"]["Pid"] == 0
                 and value["State"].get("Restarting") is False and value["State"].get("Dead") is False
                 and type(value["State"].get("OOMKilled")) is bool, "retirement_container_still_running")
            need(graph_hash(configuration_graph(value)) == original["retirement_config_sha256"], "retirement_configuration_changed")
            return value
        value = stopped()
        for binding in original["bind_identities"]:
            actual = canonical_directory(binding["source"], system)
            need((actual.st_dev, actual.st_ino) == (binding["dev"], binding["ino"]), "retirement_state_source_changed")
        group = absolute(original["cgroup_path"])
        parent = absolute(original["cgroup_parent_path"])
        need(beneath(group, "/sys/fs/cgroup") and group != "/sys/fs/cgroup" and os.path.dirname(group) == parent, "retirement_cgroup_path")
        mounts = parse_mountinfo(system.text("/proc/self/mountinfo", 4 * JSON_LIMIT))
        need(any(row["id"] == original["cgroup_mount_id"] and row["type"] == "cgroup2"
                 and row["path"] == "/sys/fs/cgroup" for row in mounts), "retirement_cgroup_mount_changed")
        parent_stat = canonical_directory(parent, system)
        need((parent_stat.st_dev, parent_stat.st_ino) == (original["cgroup_parent_dev"], original["cgroup_parent_ino"]), "retirement_cgroup_parent_changed")
        try:
            current = canonical_directory(group, system)
        except FileNotFoundError:
            proof = {"state": "absent", "parent_identity_confirmed": True}
        else:
            need((current.st_dev, current.st_ino) == (original["cgroup_dev"], original["cgroup_ino"]), "retirement_cgroup_replaced")
            for _ in range(2):
                events = numeric_fields(system.text(group + "/cgroup.events"))
                processes = system.text(group + "/cgroup.procs", JSON_LIMIT).split()
                need("populated" in events and events["populated"] == 0 and not processes,
                     "retirement_cgroup_populated", observed_processes=len(processes))
            need(identity(system.lstat(group)) == identity(current), "retirement_cgroup_changed")
            proof = {"state": "empty", "events": events, "observed_processes": 0}
        resource_complete, memory_events, pids_events, resource_error = False, None, None, None
        if proof["state"] == "absent":
            resource_error = {"reason": "terminal_cgroup_absent", "details": {}}
        else:
            try:
                memory_events = numeric_fields(system.text(group + "/memory.events"))
                pids_events = numeric_fields(system.text(group + "/pids.events"))
                need({"oom", "oom_kill", "oom_group_kill"} <= set(memory_events) and "max" in pids_events,
                     "retirement_resource_events_unavailable")
                resource_complete = True
            except Exception as error:
                resource_error = fault(error)
        need(identity(system.lstat(parent)) == identity(parent_stat) and system.realpath(parent) == parent, "retirement_cgroup_parent_changed")
        if proof["state"] == "absent":
            try:
                system.lstat(group)
            except FileNotFoundError:
                pass
            else:
                raise Unavailable("retirement_cgroup_reappeared")
        value = stopped()
        observed_oom = memory_events is not None and any(key in memory_events and memory_events[key] > 0
            for key in ("oom", "oom_kill", "oom_group_kill"))
        observed_pids = pids_events is not None and "max" in pids_events and pids_events["max"] > 0
        resource_failure = value["State"]["OOMKilled"] or observed_oom or observed_pids
        report.update(complete=True, observed_at_ms=system.now_ms(), cid=owner["cid"],
                      original_pid=original["pid"], original_pid_start_ticks=original["pid_start_ticks"],
                      cgroup_path=group, cgroup_dev=original["cgroup_dev"], cgroup_ino=original["cgroup_ino"],
                      original_cgroup=proof, owner_oom_killed=value["State"].get("OOMKilled"),
                      memory_events=memory_events, pids_events=pids_events,
                      resource_failure=bool(resource_failure), resource_observation_complete=resource_complete,
                      resource_observation_error=resource_error,
                      capacity_thresholds_applied=False)
    except Exception as error:
        report.update(fault(error))
    return report


def fresh_json(path, value):
    """Atomic creation without replacing an earlier receipt or following aliases."""
    path = absolute(os.path.abspath(path))
    parent = os.path.dirname(path)
    parent_identity = os.lstat(parent)
    need(os.path.realpath(parent) == parent and stat.S_ISDIR(parent_identity.st_mode), "receipt_parent_aliased")
    raw = (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()
    need(len(raw) <= JSON_LIMIT, "receipt_size_bound")
    temporary = ".native-resource-receipt-" + uuid.uuid4().hex
    directory = os.open(parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    fd = None
    try:
        need(identity(os.fstat(directory)) == identity(parent_identity), "receipt_parent_changed")
        fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o644, dir_fd=directory)
        os.fchmod(fd, 0o644)
        with os.fdopen(fd, "wb", closefd=False) as stream:
            stream.write(raw)
            stream.flush()
            os.fsync(fd)
        os.link(temporary, os.path.basename(path), src_dir_fd=directory, dst_dir_fd=directory, follow_symlinks=False)
        os.fsync(directory)
        need(identity(os.lstat(parent)) == identity(parent_identity) and os.path.realpath(parent) == parent, "receipt_parent_changed")
    finally:
        if fd is not None:
            os.close(fd)
            os.unlink(temporary, dir_fd=directory)
        os.close(directory)


def snapshot(plan, system=None, job_factory=None, plan_path=None, establish_baseline=False):
    report = {"schema": "prismpm/native-resource-snapshot/1", **receipt(plan), "complete": False}
    job = None
    try:
        observer = Observer(plan, system, establish_baseline)
        report["fast"] = observer.fast()
        if job_factory is None:
            report["allocation"] = complete_scan(plan, observer.system, establish_baseline)
        else:
            job = job_factory(plan_path, plan["output_root"])
            while True:
                value = job.poll()
                if value is not None:
                    report["allocation"] = value
                    break
                observer.system.sleep(1)
                report["last_fast"] = observer.fast()
        report["final_fast"] = observer.fast()
        need(report["allocation"].get("complete") is True
             and all(report["allocation"].get(k) == v for k, v in receipt(plan).items()), "scan_receipt_identity_changed")
        if plan_path is not None:
            need(bounded_json(plan_path) == plan, "plan_custody_changed")
        report["complete"] = True
        if establish_baseline:
            report["schema"] = "prismpm/native-resource-baseline/1"
    except Exception as error:
        report.update(fault(error))
    finally:
        if job:
            job.close()
    return report


class ScanJob:
    """The only child this monitor may cancel is its own read-only scanner."""
    def __init__(self, plan_path, output_root, establish_baseline=False):
        self.start = time.monotonic()
        self.fd, self.path = tempfile.mkstemp(prefix=".native-allocation-", dir=output_root)
        try:
            self.process = subprocess.Popen([sys.executable, "-I", "-B", os.path.abspath(__file__), "_baseline_scan" if establish_baseline else "_scan", plan_path],
                                            stdin=subprocess.DEVNULL, stdout=self.fd, stderr=subprocess.DEVNULL)
        except Exception:
            os.close(self.fd)
            os.unlink(self.path)
            raise
        self.retired = False

    def poll(self):
        need(time.monotonic() - self.start < SCAN_SECONDS, "scan_time_budget")
        if self.process.poll() is None:
            return None
        os.close(self.fd)
        self.retired = True
        try:
            value = bounded_json(self.path)
            need(self.process.returncode == 0 and value.get("complete") is True, value.get("reason", "scan_worker_failed"))
            return value
        finally:
            os.unlink(self.path)

    def close(self):
        if not self.retired:
            uncertainty = False
            if self.process.poll() is None:
                self.process.terminate()
                try:
                    self.process.wait(timeout=0.5)
                except subprocess.TimeoutExpired:
                    self.process.kill()
                    try:
                        self.process.wait(timeout=0.5)
                    except subprocess.TimeoutExpired:
                        uncertainty = True
            os.close(self.fd)
            os.unlink(self.path)
            self.retired = True
            need(not uncertainty, "scan_worker_retirement_unproven")


class SampleWriter:
    def __init__(self, path, maximum=SAMPLE_LIMIT):
        need(os.path.realpath(os.path.dirname(path)) == os.path.dirname(path), "sample_parent_aliased")
        self.fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o644)
        os.fchmod(self.fd, 0o644)
        self.path = path
        self.identity = identity(os.fstat(self.fd))
        self.size, self.maximum = 0, maximum

    def write(self, value):
        need(identity(os.lstat(self.path)) == self.identity and os.fstat(self.fd).st_nlink == 1
             and os.fstat(self.fd).st_size == self.size, "sample_output_custody_changed")
        raw = (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()
        need(self.size + len(raw) <= self.maximum, "sample_output_budget", maximum_bytes=self.maximum)
        view = memoryview(raw)
        while view:
            count = os.write(self.fd, view)
            need(count > 0, "sample_write_failed")
            self.size += count
            view = view[count:]
        os.fsync(self.fd)

    def close(self):
        os.close(self.fd)


def finished(plan):
    try:
        value = bounded_json(plan["finish_file"], 8192)
    except FileNotFoundError:
        return False
    closed(value, {"schema", "source_revision", "run_id", "run_attempt", "phase"}, "finish_fields")
    need(value["schema"] == "prismpm/native-resource-finish/1", "finish_schema")
    need(all(value[key] == plan[key] for key in ("source_revision", "run_id", "run_attempt", "phase")), "finish_identity_changed")
    return True


def watch(plan_path, plan, samples_path, stop_path, system=None, job_factory=ScanJob):
    system = system or System(plan["store_root"])
    observer, writer, job = Observer(plan, system), None, None
    final = False
    interrupted = []
    previous = {}
    def stop(signum, _frame):
        interrupted.append(signum)
    try:
        for name in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
            previous[name] = signal.signal(name, stop)
        need(not os.path.lexists(stop_path) and not os.path.lexists(plan["finish_file"]), "watch_receipt_not_fresh")
        initial_plan = os.lstat(plan_path)
        def same_plan():
            current = os.lstat(plan_path)
            need(identity(current) == identity(initial_plan) and current.st_nlink == 1
                 and current.st_size == initial_plan.st_size and current.st_mtime_ns == initial_plan.st_mtime_ns
                 and current.st_ctime_ns == initial_plan.st_ctime_ns and os.path.realpath(plan_path) == plan_path,
                 "plan_custody_changed")
        writer = SampleWriter(samples_path)
        next_scan = system.monotonic()
        while True:
            tick = system.monotonic()
            need(not interrupted, "monitor_interrupted", signal=interrupted[0] if interrupted else 0)
            same_plan()
            writer.write(observer.fast())
            if finished(plan) and not final:
                if job:
                    job.close()
                    job = None
                final = True
            if job is None and (final or tick >= next_scan):
                job = job_factory(plan_path, plan["output_root"])
                next_scan = tick + 60
            if job:
                value = job.poll()
                if value is not None:
                    need(value.get("complete") is True and all(value.get(k) == v for k, v in receipt(plan).items()), "scan_receipt_identity_changed")
                    writer.write(value)
                    job.close()
                    job = None
                    if final:
                        writer.write(observer.fast())
                        return 0
            system.sleep(max(0, 1 - (system.monotonic() - tick)))
    except Exception as error:
        fresh_json(stop_path, {"schema": "prismpm/native-resource-stop/1", **receipt(plan),
                               "observed_at_ms": system.now_ms(), "complete": False, **fault(error)})
        return 1
    finally:
        if job:
            job.close()
        if writer:
            writer.close()
        for name, handler in previous.items():
            signal.signal(name, handler)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("snapshot", "baseline", "watch", "retire", "_scan", "_baseline_scan"))
    parser.add_argument("plan")
    parser.add_argument("outputs", nargs="*")
    args = parser.parse_args(argv)
    expected = {"snapshot": 1, "baseline": 1, "watch": 2, "retire": 2, "_scan": 0, "_baseline_scan": 0}[args.mode]
    need(len(args.outputs) == expected, "cli_arguments")
    plan = validate_plan(bounded_json(os.path.abspath(args.plan)))
    for path in args.outputs:
        absolute(os.path.abspath(path))
        need(beneath(os.path.abspath(path), plan["output_root"]), "receipt_outside_output_root")
    if args.mode in ("snapshot", "baseline"):
        establishing = args.mode == "baseline"
        factory = (lambda path, root: ScanJob(path, root, True)) if establishing else ScanJob
        report = snapshot(plan, job_factory=factory, plan_path=os.path.abspath(args.plan), establish_baseline=establishing)
        fresh_json(args.outputs[0], report)
        return 0 if report["complete"] else 1
    if args.mode == "watch":
        return watch(os.path.abspath(args.plan), plan, *args.outputs)
    if args.mode == "retire":
        original = bounded_json(os.path.abspath(args.outputs[0]))
        report = retire(plan, original)
        fresh_json(args.outputs[1], report)
        return 0 if report["complete"] else 1
    try:
        report = complete_scan(plan, establish_baseline=args.mode == "_baseline_scan")
    except Exception as error:
        report = {"complete": False, **fault(error)}
    print(json.dumps(report, sort_keys=True, separators=(",", ":")))
    return 0 if report["complete"] else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        # Codes/errno only: never stringify Docker/kernel/config payloads.
        print(json.dumps(fault(error), separators=(",", ":")), file=sys.stderr)
        sys.exit(1)
