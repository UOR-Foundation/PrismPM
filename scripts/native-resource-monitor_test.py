"""No Docker, mount, SDK, network or privileged operation is used by these units."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import stat
import tempfile
import subprocess
from types import SimpleNamespace
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("native_resource_monitor", Path(__file__).with_name("native-resource-monitor.py"))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


def info(dev, ino, mode=stat.S_IFDIR | 0o755, size=0, blocks=8, nlink=1, rdev=0):
    return SimpleNamespace(st_dev=dev, st_ino=ino, st_mode=mode, st_size=size,
                           st_blocks=blocks, st_nlink=nlink, st_rdev=rdev,
                           st_mtime_ns=100, st_ctime_ns=100)


def fixture(owner=True):
    outer, inner = os.makedev(8, 1), os.makedev(7, 9)
    plan = {"schema": "prismpm/native-resource-plan/1", "phase": "P", "arch": "amd64",
            "source_revision": "a" * 40, "run_id": "123", "run_attempt": "2",
            "work_end_ms": 1_000_000, "whole_end_ms": 2_000_000,
            "output_root": "/owned/evidence", "store_root": "/owned/store",
            "docker_root": "/docker", "loop_device": "/dev/loop9",
            "backing": {"path": "/owned/backing.ext4", "dev": outer, "ino": 33,
                        "size": m.STORE_BYTES, "blocks_min": m.STORE_BYTES},
            "participating_paths": ["/owned/store/source", "/owned/store/tmp", "/owned/store/outputs"],
            "finish_file": "/owned/evidence/finished.json", "owner": None,
            "outside_baseline": {"path": "/owned/evidence/baseline.json", "sha256": "e" * 64}}
    if owner:
        plan["owner"] = {"name": "buildx_buildkit_unit0", "cid": "b" * 64,
                         "label_key": "org.prismpm.native-resource-owner", "label_value": "owned-uuid",
                         "image_id": "sha256:" + "c" * 64, "memory_bytes": 12 * m.GIB,
                         "state": "/owned/store/state", "run": "/owned/store/run", "config_root": "/owned/store/buildx"}
    return plan


class FakeSystem:
    def __init__(self, plan):
        self.plan = plan
        self.tick = 0
        self.ms = 100
        self.stats = {}
        for i, path in enumerate(["/owned", plan["output_root"], "/docker", "/owned/store", *plan["participating_paths"],
                                  "/owned/store/state", "/owned/store/run", "/owned/store/buildx"]):
            dev = os.makedev(7, 9) if m.beneath(path, plan["store_root"]) else plan["backing"]["dev"]
            self.stats[path] = info(dev, i + 100)
        self.stats[plan["backing"]["path"]] = info(plan["backing"]["dev"], 33, stat.S_IFREG | 0o600,
                                                       m.STORE_BYTES, m.STORE_BYTES // 512)
        self.stats["/dev/loop9"] = info(5, 88, stat.S_IFBLK | 0o600, rdev=os.makedev(7, 9))
        self.stats["/proc/23/root/var/lib/buildkit"] = self.stats["/owned/store/state"]
        self.stats["/proc/23/root/run"] = self.stats["/owned/store/run"]
        self.stats["/proc/23/root/tmp"] = info(55, 444)
        self.group = "/sys/fs/cgroup/docker/owned"
        self.stats[self.group] = info(44, 555)
        self.stats["/sys/fs/cgroup/docker"] = info(44, 550)
        self.aliases = {}
        self.free = {plan["backing"]["dev"]: 14 * m.GIB, os.makedev(7, 9): 30 * m.GIB, 55: 268435456}
        self.texts = {
            "/proc/self/mountinfo": "10 1 7:9 / /owned/store rw,relatime - ext4 /dev/loop9 rw\n"
                                    "11 1 0:44 / /sys/fs/cgroup rw - cgroup2 cgroup rw\n",
            "/sys/dev/block/7:9/loop/backing_file": "/owned/backing.ext4\n",
            "/proc/meminfo": "MemTotal: 16777216 kB\nMemAvailable: 8388608 kB\n",
            "/proc/23/stat": "23 (buildkitd) " + " ".join(["S"] + ["0"] * 18 + ["123"] + ["0"] * 4),
            "/proc/23/cgroup": "0::/docker/owned\n",
            "/proc/23/mountinfo": "20 1 0:99 / / ro - overlay overlay rw\n"
                                  "21 20 7:9 /state /var/lib/buildkit rw - ext4 /dev/loop9 rw\n"
                                  "22 20 7:9 /run /run rw - ext4 /dev/loop9 rw\n"
                                  "23 20 0:55 / /tmp rw,noexec,nosuid,nodev - tmpfs tmpfs rw,size=262144k\n",
        }
        for key, value in {"memory.current": "1024", "memory.peak": "2048", "memory.max": str(12 * m.GIB),
                           "memory.swap.max": "0", "pids.current": "4", "pids.max": "4096",
                           "cpu.max": "200000 100000", "memory.events": "low 0\nhigh 0\nmax 0\noom 0\noom_kill 0\noom_group_kill 0\n",
                           "pids.events": "max 0\n", "cpu.stat": "usage_usec 5\nuser_usec 3\nsystem_usec 2\n"}.items():
            self.texts[self.group + "/" + key] = value
        image_config = {"Entrypoint": ["buildkitd"], "Cmd": None, "User": "", "WorkingDir": "", "Labels": {},
                        "Env": ["NEVER_DUMP_SECRET=private-fixture"]}
        self.image = {"Id": "sha256:" + "c" * 64, "Os": "linux", "Architecture": "amd64",
                      "RepoDigests": [m.BUILDKIT], "Config": image_config}
        self.container = {"Id": "b" * 64, "Name": "/buildx_buildkit_unit0", "Image": self.image["Id"],
                          "Config": {**image_config, "Image": self.image["Id"], "Env": ["NEVER_DUMP_SECRET=private-fixture"],
                                     "Labels": {"org.prismpm.native-resource-owner": "owned-uuid"}},
                          "HostConfig": {"Privileged": True, "ReadonlyRootfs": True, "Init": True, "NanoCpus": 2_000_000_000,
                                         "Memory": 12 * m.GIB, "MemorySwap": 12 * m.GIB, "PidsLimit": 4096,
                                         "NetworkMode": "bridge", "LogConfig": {"Type": "local", "Config": {"max-size": "16m", "max-file": "2"}},
                                         "Tmpfs": {"/tmp": "rw,noexec,nosuid,nodev,size=268435456"}},
                          "Mounts": [{"Type": "bind", "Source": "/owned/store/state", "Destination": "/var/lib/buildkit", "RW": True},
                                     {"Type": "bind", "Source": "/owned/store/run", "Destination": "/run", "RW": True}],
                          "State": {"Running": True, "Pid": 23, "OOMKilled": False, "Restarting": False, "Dead": False}, "NetworkSettings": {"Networks": {"bridge": {"EndpointID": "owned"}}}}
        self.docker_calls = []

    def lstat(self, path):
        if path not in self.stats:
            raise FileNotFoundError(2, "fixture missing")
        return self.stats[path]

    def realpath(self, path):
        return self.aliases.get(path, path)

    def statvfs(self, path):
        value = self.lstat(path)
        free = self.free[value.st_dev]
        capacity = 268435456 if value.st_dev == 55 else (40 * m.GIB if value.st_dev == os.makedev(7, 9) else 80 * m.GIB)
        return SimpleNamespace(f_frsize=4096, f_bsize=4096, f_blocks=capacity // 4096,
                               f_bfree=free // 4096, f_bavail=free // 4096)

    def text(self, path, maximum=65536):
        if path not in self.texts:
            raise FileNotFoundError(2, "fixture missing")
        return self.texts[path]

    def docker(self, args):
        self.docker_calls.append(args)
        if args[0] == "info":
            return self.plan["docker_root"]
        return copy.deepcopy(self.container if args[0] == "container" else self.image)

    def baseline(self, reference):
        sample = {"owner": None, "docker_root": self.plan["docker_root"],
                  "backing": {"dev": self.plan["backing"]["dev"], "ino": self.plan["backing"]["ino"], "size_bytes": m.STORE_BYTES}}
        return {"schema": "prismpm/native-resource-baseline/1", "complete": True, **m.receipt(self.plan),
                "fast": sample, "final_fast": sample,
                "allocation": {"complete": True, "outside_allocated_bytes": 3 * m.GIB,
                               "outside_allocated_bytes_by_dev": {str(self.plan["backing"]["dev"]): 3 * m.GIB}}}

    def store_empty(self, path):
        return True

    def now_ms(self):
        return self.ms

    def monotonic(self):
        return self.tick

    def sleep(self, seconds):
        self.tick += seconds
        self.ms += int(seconds * 1000)


class PlanAndKernelTests(unittest.TestCase):
    def rejected(self, change, reason=None, second=False):
        plan = fixture()
        system = FakeSystem(plan)
        observer = m.Observer(plan, system)
        if second:
            observer.fast()
        change(plan, system)
        with self.assertRaises((m.Unavailable, OSError)) as caught:
            observer.fast()
        if reason:
            self.assertEqual(caught.exception.reason, reason)
        return caught.exception

    def test_closed_plan_and_owned_paths(self):
        m.validate_plan(fixture())
        for mutate in [lambda p: p.update(extra=True), lambda p: p["backing"].update(blocks_min=0),
                       lambda p: p.update(participating_paths=["/outside"]),
                       lambda p: p["owner"].update(state="/outside"),
                       lambda p: p.update(source_revision="latest"), lambda p: p.update(run_attempt="02")]:
            value = fixture()
            mutate(value)
            with self.assertRaises(m.Unavailable):
                m.validate_plan(value)

    def test_actual_devices_capacity_cgroup_and_safe_hashes(self):
        plan = fixture()
        system = FakeSystem(plan)
        row = m.Observer(plan, system).fast()
        self.assertEqual(len(row["filesystems"]), 2)
        self.assertEqual(row["backing"]["allocated_bytes"], 40 * m.GIB)
        self.assertEqual(row["owner"]["memory"]["memory.max"], 12 * m.GIB)
        self.assertEqual(row["owner"]["cpu_max"], [200000, 100000])
        self.assertEqual(row["owner"]["cgroup_path"], system.group)
        self.assertFalse(row["absolute_peak_claimed"])
        text = json.dumps(row)
        self.assertNotIn("NEVER_DUMP_SECRET", text)
        self.assertNotIn("private-fixture", text)
        self.assertEqual(system.docker_calls[0], ["info", "--format", "{{json .DockerRootDir}}"])

    def test_store_only_phase_has_no_fake_container_or_cgroup_values(self):
        plan = fixture(False)
        row = m.Observer(plan, FakeSystem(plan)).fast()
        self.assertIsNone(row["owner"])

    def test_backing_holes_identity_links_and_loop_drift(self):
        self.rejected(lambda p, s: setattr(s.stats[p["backing"]["path"]], "st_blocks", m.STORE_BYTES // 512 - 1), "backing_hole")
        self.rejected(lambda p, s: setattr(s.stats[p["backing"]["path"]], "st_ino", 99), "backing_identity_changed")
        self.rejected(lambda p, s: setattr(s.stats[p["backing"]["path"]], "st_nlink", 2), "backing_not_independent_file")
        self.rejected(lambda p, s: s.texts.update({"/sys/dev/block/7:9/loop/backing_file": "/other"}), "loop_backing_changed")
        self.rejected(lambda p, s: s.aliases.update({p["backing"]["path"]: "/other"}), "backing_path_aliased")

    def test_mount_coverage_and_discard_drift(self):
        self.rejected(lambda p, s: s.texts.update({"/proc/self/mountinfo": s.texts["/proc/self/mountinfo"].replace("relatime", "discard")}), "discard_not_permitted")
        self.rejected(lambda p, s: s.texts.update({"/proc/self/mountinfo": s.texts["/proc/self/mountinfo"] + "12 10 8:2 / /owned/store/tmp rw - ext4 /dev/other rw\n"}), "nested_mount_in_store")
        self.rejected(lambda p, s: s.container["Mounts"][0].update(Source="/outside"), "owner_mount_changed")
        self.rejected(lambda p, s: s.container["Mounts"].append({"Type": "volume", "Source": "/unmeasured", "Destination": "/extra", "RW": True}), "owner_mount_count")
        self.rejected(lambda p, s: setattr(s.stats[p["owner"]["config_root"]], "st_ino", 12345), "participating_identity_or_configuration_drift", second=True)

    def test_backing_reserve_is_not_inner_available_stop(self):
        self.rejected(lambda p, s: s.free.update({p["backing"]["dev"]: 12 * m.GIB - 4096}), "backing_reserve")
        self.rejected(lambda p, s: s.free.update({os.makedev(7, 9): m.GIB - 4096}), "inner_available_stop")
        plan = fixture()
        system = FakeSystem(plan)
        system.free[os.makedev(7, 9)] = 2 * m.GIB
        observer = m.Observer(plan, system)
        row = observer.fast()
        self.assertGreater(row["filesystems"][0]["available_bytes"], 12 * m.GIB)
        system.free[os.makedev(7, 9)] = m.GIB
        row = observer.fast()
        self.assertEqual(row["sampled_minimum_available_bytes"][str(os.makedev(7, 9))], m.GIB)

    def test_actual_docker_root_on_another_filesystem_also_needs_outer_reserve(self):
        plan = fixture()
        system = FakeSystem(plan)
        device = os.makedev(8, 2)
        system.stats[plan["docker_root"]].st_dev = device
        system.free[device] = 12 * m.GIB - 4096
        with self.assertRaises(m.Unavailable) as caught:
            m.Observer(plan, system).fast()
        self.assertEqual(caught.exception.reason, "outer_filesystem_reserve")
        self.assertEqual(caught.exception.details["dev"], device)

    def test_memory_cpu_swap_pids_limits_and_events_are_kernel_readbacks(self):
        for name, text, reason in [("memory.max", "1", "kernel_limits_changed"),
                                   ("memory.swap.max", "4096", "kernel_limits_changed"),
                                   ("pids.max", "max", "cgroup_limit_unavailable"),
                                   ("cpu.max", "100000 100000", "kernel_cpu_limit_changed"),
                                   ("memory.events", "low 0\nhigh 0\nmax 0\noom 1\noom_kill 0\noom_group_kill 0", "owner_oom_event"),
                                   ("pids.events", "max 1", "owner_pid_limit_event")]:
            with self.subTest(name=name):
                self.rejected(lambda p, s: s.texts.update({s.group + "/" + name: text}), reason)
        self.rejected(lambda p, s: s.container["HostConfig"].update(NanoCpus=0), "owner_limit_or_config_changed")
        self.rejected(lambda p, s: s.texts.pop(s.group + "/memory.peak"))
        self.rejected(lambda p, s: s.texts.update({s.group + "/memory.events": "low 0\nhigh 0\nmax 0\noom 0\noom_kill 0"}), "kernel_events_unavailable")
        self.rejected(lambda p, s: s.texts.update({"/proc/23/cgroup": "2:memory:/legacy"}), "cgroup_v2_required")
        self.rejected(lambda p, s: s.texts.update({"/proc/meminfo": "MemTotal: 16777216 kB\nMemAvailable: 1024 kB"}), "host_memory_reserve")

    def test_host_memory_minus_two_gib_caps_a_smaller_machine(self):
        plan = fixture()
        system = FakeSystem(plan)
        system.texts["/proc/meminfo"] = "MemTotal: 10485760 kB\nMemAvailable: 8388608 kB"
        plan["owner"]["memory_bytes"] = 8 * m.GIB
        system.container["HostConfig"].update(Memory=8 * m.GIB, MemorySwap=8 * m.GIB)
        system.texts[system.group + "/memory.max"] = str(8 * m.GIB)
        self.assertEqual(m.Observer(plan, system).fast()["owner"]["memory"]["memory.max"], 8 * m.GIB)

    def test_actual_bind_inode_and_readonly_namespace_are_verified(self):
        self.rejected(lambda p, s: s.stats.update({"/proc/23/root/var/lib/buildkit": info(os.makedev(7, 9), 999)}),
                      "owner_bind_source_identity_changed")
        self.rejected(lambda p, s: s.texts.update({"/proc/23/mountinfo": s.texts["/proc/23/mountinfo"].replace("/ / ro", "/ / rw")}),
                      "owner_root_not_readonly")
        self.rejected(lambda p, s: s.texts.update({"/proc/23/mountinfo": s.texts["/proc/23/mountinfo"].replace("/state /var/lib/buildkit", "/unmeasured /var/lib/buildkit")}),
                      "owner_kernel_bind_changed")
        self.rejected(lambda p, s: s.free.update({55: 268435456 + 4096}), "owner_kernel_tmpfs_capacity_unavailable")
        self.rejected(lambda p, s: s.container["HostConfig"].update(IpcMode="container:foreign"), "owner_extra_configuration")

    def test_peak_reset_cannot_hide_a_previous_observation(self):
        self.rejected(lambda p, s: s.texts.update({s.group + "/memory.peak": "1024"}),
                      "kernel_memory_peak_regressed", second=True)

    def test_actual_config_fingerprint_detects_drift_without_dumping_values(self):
        error = self.rejected(lambda p, s: s.container["Config"]["Env"].append("SECRET=changed"),
                              "owner_image_defaults_changed", second=True)
        self.assertNotIn("SECRET", json.dumps(m.fault(error)))
        self.rejected(lambda p, s: s.container["HostConfig"].update(Ulimits=[{"Name": "nofile", "Soft": 10, "Hard": 10}]),
                      "participating_identity_or_configuration_drift", second=True)
        self.rejected(lambda p, s: s.container["State"].update(Pid=24))
        self.rejected(lambda p, s: s.image.update(RepoDigests=[]), "owner_image_digest_unbound")

    def test_missing_observation_and_deadline_are_never_zero(self):
        plan = fixture()
        system = FakeSystem(plan)
        system.stats.pop(plan["docker_root"])
        report = m.snapshot(plan, system)
        self.assertFalse(report["complete"])
        self.assertEqual(report["reason"], "observation_os_error")
        self.assertNotIn("allocation", report)
        self.rejected(lambda p, s: setattr(s, "ms", p["work_end_ms"]), "work_deadline")
        self.rejected(lambda p, s: setattr(s, "ms", p["whole_end_ms"]), "whole_deadline")


class AllocatedWalkTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "data").write_bytes(b"secret-not-read" * 400)
        (self.root / "sub").mkdir()
        os.link(self.root / "data", self.root / "sub/hard")
        (self.root / "sub/link").symlink_to("/outside/not-followed")

    def test_complete_walk_counts_real_blocks_deduplicates_and_does_not_open_files(self):
        expected = {}
        for path in [self.root, self.root / "data", self.root / "sub", self.root / "sub/hard", self.root / "sub/link"]:
            value = path.lstat()
            expected[(value.st_dev, value.st_ino)] = value.st_blocks * 512
        with patch("builtins.open", side_effect=AssertionError("scanner must not read file contents")):
            row = m.scan_allocations([str(self.root), str(self.root / "sub")])
        self.assertTrue(row["complete"])
        self.assertEqual(row["observed_entries"], 5)
        self.assertEqual(row["unique_inodes"], 4)
        self.assertEqual(row["symlinks"], 1)
        self.assertEqual(sum(row["allocated_bytes_by_dev"].values()), sum(expected.values()))
        self.assertEqual(row["walked_roots"], [str(self.root)])
        self.assertFalse(row["instantaneous_snapshot_claimed"])

    def test_root_alias_missing_permission_entry_and_time_limits_fail(self):
        alias = self.root / "alias"
        alias.symlink_to(self.root / "sub", target_is_directory=True)
        for roots, kwargs, reason in [([str(alias)], {}, "scan_root_aliased"),
                                      ([str(self.root)], {"maximum": 2}, "scan_entry_budget")]:
            with self.subTest(reason=reason), self.assertRaises(m.Unavailable) as caught:
                m.scan_allocations(roots, **kwargs)
            self.assertEqual(caught.exception.reason, reason)
        with self.assertRaises(FileNotFoundError):
            m.scan_allocations([str(self.root / "absent")])
        with patch.object(os, "scandir", side_effect=PermissionError(13, "fixture")), self.assertRaises(PermissionError):
            m.scan_allocations([str(self.root)])
        clock = iter(range(100))
        fake = m.System()
        fake.monotonic = lambda: next(clock)
        with self.assertRaises(m.Unavailable) as caught:
            m.scan_allocations([str(self.root)], system=fake, seconds=2)
        self.assertEqual(caught.exception.reason, "scan_time_budget")

    def test_directory_identity_is_rechecked_after_scanning(self):
        class Replaced(m.System):
            seen = 0
            def lstat(inner, path):
                value = os.lstat(path)
                if path == str(self.root):
                    inner.seen += 1
                    if inner.seen > 1:
                        return info(value.st_dev, value.st_ino + 1, value.st_mode)
                return value
        with self.assertRaises(m.Unavailable) as caught:
            m.scan_allocations([str(self.root)], system=Replaced())
        self.assertEqual(caught.exception.reason, "scan_directory_custody_changed")

    def test_only_declared_merged_overlay_view_is_not_traversed(self):
        merged = self.root / "merged"
        merged.mkdir()
        (merged / "duplicate").write_bytes(b"duplicate view")
        row = m.scan_allocations([str(self.root)], exclusions=[str(merged)])
        self.assertEqual(row["omitted_merged_overlay_views"], [str(merged)])
        self.assertEqual(row["observed_entries"], 6)

    def test_outside_budget_subtracts_backing_and_keeps_inner_device_separate(self):
        plan = fixture(False)
        system = FakeSystem(plan)
        inner = os.makedev(7, 9)
        for outside, reason in [(3 * m.GIB + m.OUTSIDE_LIMIT, None), (3 * m.GIB + m.OUTSIDE_LIMIT + 4096, "outside_storage_growth_budget"),
                                (-m.STORE_BYTES + 4096, "backing_not_in_complete_scan")]:
            row = {"complete": True, "allocated_bytes_by_dev": {str(plan["backing"]["dev"]): m.STORE_BYTES + outside,
                                                                  str(inner): 39 * m.GIB}}
            with self.subTest(outside=outside), patch.object(m, "scan_allocations", return_value=row):
                if reason:
                    with self.assertRaises(m.Unavailable) as caught:
                        m.complete_scan(plan, system)
                    self.assertEqual(caught.exception.reason, reason)
                else:
                    report = m.complete_scan(plan, system)
                    self.assertEqual(report["outside_allocated_bytes"], outside)
                    self.assertEqual(report["allocated_bytes_by_dev"][str(inner)], 39 * m.GIB)


class ReceiptTests(unittest.TestCase):
    def test_fresh_atomic_receipt_never_overwrites_and_size_limit_does_not_wrap(self):
        with tempfile.TemporaryDirectory() as root:
            path = os.path.join(root, "stop.json")
            m.fresh_json(path, {"reason": "unit"})
            with self.assertRaises(FileExistsError):
                m.fresh_json(path, {"reason": "replacement"})
            self.assertEqual(json.loads(Path(path).read_text()), {"reason": "unit"})
            self.assertEqual(list(Path(root).glob(".native-resource-receipt-*")), [])
            output = os.path.join(root, "samples.jsonl")
            writer = m.SampleWriter(output, maximum=64)
            try:
                writer.write({"n": 1})
                with self.assertRaises(m.Unavailable) as caught:
                    writer.write({"payload": "x" * 64})
                self.assertEqual(caught.exception.reason, "sample_output_budget")
            finally:
                writer.close()
            self.assertEqual(Path(output).read_text(), '{"n":1}\n')

    def test_receipts_are_readable_under_restrictive_sudo_umask_and_samples_keep_custody(self):
        with tempfile.TemporaryDirectory() as root:
            prior = os.umask(0o077)
            try:
                path = os.path.join(root, "receipt.json")
                m.fresh_json(path, {"unit": True})
                self.assertEqual(os.stat(path).st_mode & 0o777, 0o644)
                output = os.path.join(root, "samples.jsonl")
                writer = m.SampleWriter(output)
                self.assertEqual(os.stat(output).st_mode & 0o777, 0o644)
                os.unlink(output)
                Path(output).write_text("substitution")
                try:
                    with self.assertRaises(m.Unavailable) as caught:
                        writer.write({"must": "refuse"})
                    self.assertEqual(caught.exception.reason, "sample_output_custody_changed")
                finally:
                    writer.close()
            finally:
                os.umask(prior)

    def test_finished_receipt_binds_original_source_run_attempt_and_phase(self):
        with tempfile.TemporaryDirectory() as root:
            plan = fixture(False)
            plan["finish_file"] = os.path.join(root, "finish.json")
            self.assertFalse(m.finished(plan))
            value = {"schema": "prismpm/native-resource-finish/1", **{k: plan[k] for k in ("source_revision", "run_id", "run_attempt", "phase")}}
            m.fresh_json(plan["finish_file"], value)
            self.assertTrue(m.finished(plan))
            plan["run_attempt"] = "3"
            with self.assertRaises(m.Unavailable) as caught:
                m.finished(plan)
            self.assertEqual(caught.exception.reason, "finish_identity_changed")

    def test_errors_do_not_expose_environment_or_docker_payloads(self):
        for error in [ValueError("SECRET=must-not-appear"), KeyError("TOKEN=value"), OSError(13, "PRIVATE=error")]:
            text = json.dumps(m.fault(error))
            for key in ("SECRET", "TOKEN", "PRIVATE"):
                self.assertNotIn(key, text)


class BaselineAndRetirementTests(unittest.TestCase):
    def allocation_row(self, plan, outside):
        return {"complete": True, "allocated_bytes_by_dev": {str(plan["backing"]["dev"]): m.STORE_BYTES + outside,
                                                              str(os.makedev(7, 9)): m.GIB}}

    def test_baseline_precedes_owners_and_retains_large_image_allocation(self):
        plan = fixture(False)
        plan["outside_baseline"] = None
        system = FakeSystem(plan)
        with patch.object(m, "scan_allocations", return_value=self.allocation_row(plan, 8 * m.GIB)):
            baseline = m.snapshot(plan, system, establish_baseline=True)
        self.assertTrue(baseline["complete"])
        self.assertEqual(baseline["schema"], "prismpm/native-resource-baseline/1")
        self.assertEqual(baseline["allocation"]["outside_allocated_bytes"], 8 * m.GIB)
        self.assertEqual(baseline["allocation"]["outside_positive_growth_bytes"], 0)
        with self.assertRaises(m.Unavailable) as caught:
            m.Observer(plan, system).fast()
        self.assertEqual(caught.exception.reason, "outside_baseline_missing")
        plan["outside_baseline"] = fixture(False)["outside_baseline"]
        system.baseline = lambda reference: copy.deepcopy(baseline)
        with patch.object(m, "scan_allocations", return_value=self.allocation_row(plan, 9 * m.GIB)):
            result = m.complete_scan(plan, system)
        self.assertEqual(result["outside_positive_growth_bytes"], m.GIB)
        self.assertEqual(result["outside_allocated_bytes"], 9 * m.GIB)

    def test_baseline_rejects_populated_store_wrong_original_identity_and_hash(self):
        plan = fixture(False)
        system = FakeSystem(plan)
        original = system.baseline(plan["outside_baseline"])
        for field in ("source_revision", "run_id", "phase"):
            changed = copy.deepcopy(original)
            changed[field] = "foreign"
            system.baseline = lambda reference: changed
            with self.subTest(field=field), self.assertRaises(m.Unavailable):
                m.load_baseline(plan, system)
        with patch.object(m, "bounded_json", return_value=({}, b"changed")), self.assertRaises(m.Unavailable) as caught:
            m.System.baseline(plan["outside_baseline"])
        self.assertEqual(caught.exception.reason, "outside_baseline_digest_changed")
        plan["outside_baseline"] = None
        system.store_empty = lambda path: False
        with patch.object(m, "scan_allocations", return_value=self.allocation_row(plan, 8 * m.GIB)), self.assertRaises(m.Unavailable) as caught:
            m.complete_scan(plan, system, establish_baseline=True)
        self.assertEqual(caught.exception.reason, "baseline_store_not_empty")

    def stopped(self):
        plan = fixture()
        system = FakeSystem(plan)
        sample = m.Observer(plan, system).fast()
        original = {"schema": "prismpm/native-resource-snapshot/1", **m.receipt(plan), "complete": True,
                    "fast": copy.deepcopy(sample), "final_fast": copy.deepcopy(sample), "allocation": {"complete": True}}
        system.container["State"].update(Running=False, Pid=0)
        system.texts[system.group + "/cgroup.events"] = "populated 0\nfrozen 0\n"
        system.texts[system.group + "/cgroup.procs"] = ""
        return plan, system, original

    def test_terminal_empty_cgroup_is_cleanup_authority_not_a_new_fit_claim(self):
        plan, system, original = self.stopped()
        # Resource thresholds must not stop exact proven cleanup after ENOSPC.
        system.free[os.makedev(7, 9)] = 0
        result = m.retire(plan, original, system)
        self.assertTrue(result["complete"])
        self.assertEqual(result["original_cgroup"]["state"], "empty")
        self.assertFalse(result["capacity_thresholds_applied"])
        self.assertFalse(result["resource_failure"])
        self.assertTrue(result["resource_observation_complete"])

    def test_terminal_absence_requires_original_parent_and_does_not_invent_counters(self):
        plan, system, original = self.stopped()
        system.stats.pop(system.group)
        result = m.retire(plan, original, system)
        self.assertTrue(result["complete"])
        self.assertEqual(result["original_cgroup"]["state"], "absent")
        self.assertFalse(result["resource_observation_complete"])
        self.assertIsNone(result["memory_events"])
        self.assertEqual(result["resource_observation_error"], {"reason": "terminal_cgroup_absent", "details": {}})
        system.stats["/sys/fs/cgroup/docker"].st_ino += 1
        result = m.retire(plan, original, system)
        self.assertFalse(result["complete"])
        self.assertEqual(result["reason"], "retirement_cgroup_parent_changed")

    def test_live_foreign_replaced_or_unmeasured_terminal_owner_refuses_cleanup(self):
        mutations = [
            (lambda p, s, o: s.container["State"].update(Running=True, Pid=23), "retirement_container_still_running"),
            (lambda p, s, o: s.texts.update({s.group + "/cgroup.events": "populated 1\nfrozen 0"}), "retirement_cgroup_populated"),
            (lambda p, s, o: s.texts.update({s.group + "/cgroup.procs": "99\n"}), "retirement_cgroup_populated"),
            (lambda p, s, o: setattr(s.stats[s.group], "st_ino", 999), "retirement_cgroup_replaced"),
            (lambda p, s, o: s.container["Config"].update(Env=["changed"]), "retirement_configuration_changed"),
            (lambda p, s, o: o.update(source_revision="f" * 40), "retirement_original_report"),
        ]
        for mutate, reason in mutations:
            plan, system, original = self.stopped()
            mutate(plan, system, original)
            with self.subTest(reason=reason):
                result = m.retire(plan, original, system)
                self.assertFalse(result["complete"])
                self.assertEqual(result["reason"], reason)
        plan, system, original = self.stopped()
        system.texts.pop(system.group + "/cgroup.events")
        result = m.retire(plan, original, system)
        self.assertFalse(result["complete"])
        self.assertEqual(result["reason"], "observation_os_error")

    def test_late_oom_pid_or_unknown_counters_preserve_cleanup_but_fail_resource_measurement(self):
        for mode in ("oomflag", "oomevent", "partialoomevent", "pids", "missing"):
            plan, system, original = self.stopped()
            if mode == "oomflag":
                system.container["State"]["OOMKilled"] = True
            elif mode == "oomevent":
                system.texts[system.group + "/memory.events"] = "oom 1\noom_kill 0\noom_group_kill 0"
            elif mode == "pids":
                system.texts[system.group + "/pids.events"] = "max 1"
            elif mode == "partialoomevent":
                system.texts[system.group + "/memory.events"] = "oom 1"
            else:
                system.texts.pop(system.group + "/memory.events")
            with self.subTest(mode=mode):
                result = m.retire(plan, original, system)
                self.assertTrue(result["complete"])
                self.assertTrue(result["resource_failure"] or not result["resource_observation_complete"])
                if mode != "missing":
                    self.assertTrue(result["resource_failure"])


class WatchTests(unittest.TestCase):
    def setup_watch(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        plan = fixture(False)
        plan["output_root"] = str(root)
        plan["outside_baseline"]["path"] = str(root / "baseline.json")
        plan["finish_file"] = str(root / "finish.json")
        path = root / "plan.json"
        path.write_text(json.dumps(plan))
        return root, plan, str(path), FakeSystem(plan)

    @staticmethod
    def finish(plan):
        m.fresh_json(plan["finish_file"], {"schema": "prismpm/native-resource-finish/1",
                                         **{k: plan[k] for k in ("source_revision", "run_id", "run_attempt", "phase")}})

    @staticmethod
    def jobs(plan, system, delay=2, error=None):
        records = []
        class Job:
            def __init__(self, *_args):
                self.started = system.tick
                self.closed = False
                records.append(self)
            def poll(self):
                if error:
                    raise error
                if system.tick - self.started < delay:
                    return None
                return {"kind": "allocation", "complete": True, **m.receipt(plan),
                        "allocated_bytes_by_dev": {str(plan["backing"]["dev"]): m.STORE_BYTES}}
            def close(self):
                self.closed = True
        return Job, records

    def test_fast_samples_continue_during_scan_and_finish_requires_final_complete_scan(self):
        root, plan, path, system = self.setup_watch()
        jobs, records = self.jobs(plan, system)
        prior = system.sleep
        def sleep(seconds):
            prior(seconds)
            if system.tick >= 3 and not Path(plan["finish_file"]).exists():
                self.finish(plan)
        system.sleep = sleep
        status = m.watch(path, plan, str(root / "samples"), str(root / "stop"), system, jobs)
        self.assertEqual(status, 0)
        rows = [json.loads(line) for line in (root / "samples").read_text().splitlines()]
        fast = [row["observed_at_ms"] for row in rows if row["kind"] == "fast"]
        self.assertTrue(all(0 <= b - a <= 1000 for a, b in zip(fast, fast[1:])))
        self.assertEqual(len([row for row in rows if row["kind"] == "allocation"]), 2)
        self.assertEqual(len(records), 2)
        self.assertTrue(all(job.closed for job in records))
        self.assertFalse((root / "stop").exists())

    def test_complete_allocated_scan_is_repeated_after_sixty_seconds(self):
        root, plan, path, system = self.setup_watch()
        jobs, records = self.jobs(plan, system, delay=2)
        prior = system.sleep
        def sleep(seconds):
            prior(seconds)
            if system.tick >= 61 and not Path(plan["finish_file"]).exists():
                self.finish(plan)
        system.sleep = sleep
        self.assertEqual(m.watch(path, plan, str(root / "samples"), str(root / "stop"), system, jobs), 0)
        self.assertEqual([job.started for job in records], [0, 60, 61])

    def test_fast_reserve_failure_cancels_only_owned_scan_and_writes_stop(self):
        root, plan, path, system = self.setup_watch()
        jobs, records = self.jobs(plan, system)
        prior = system.sleep
        def sleep(seconds):
            prior(seconds)
            system.free[os.makedev(7, 9)] = 0
        system.sleep = sleep
        self.assertEqual(m.watch(path, plan, str(root / "samples"), str(root / "stop"), system, jobs), 1)
        stop = json.loads((root / "stop").read_text())
        self.assertEqual(stop["reason"], "inner_available_stop")
        self.assertFalse(stop["complete"])
        self.assertTrue(records[0].closed)
        self.assertEqual((root / "stop").stat().st_mode & 0o777, 0o644)

    def test_incomplete_scan_never_becomes_zero_or_a_successful_finish(self):
        root, plan, path, system = self.setup_watch()
        jobs, records = self.jobs(plan, system, error=m.Unavailable("scan_entry_budget"))
        self.assertEqual(m.watch(path, plan, str(root / "samples"), str(root / "stop"), system, jobs), 1)
        self.assertEqual(json.loads((root / "stop").read_text())["reason"], "scan_entry_budget")
        rows = [json.loads(line) for line in (root / "samples").read_text().splitlines()]
        self.assertFalse(any(row.get("kind") == "allocation" for row in rows))
        self.assertTrue(records[0].closed)

    def test_original_plan_replacement_stops_even_with_valid_new_json(self):
        root, plan, path, system = self.setup_watch()
        jobs, records = self.jobs(plan, system)
        prior = system.sleep
        def sleep(seconds):
            prior(seconds)
            Path(path).write_text(json.dumps(plan) + "\n")
        system.sleep = sleep
        self.assertEqual(m.watch(path, plan, str(root / "samples"), str(root / "stop"), system, jobs), 1)
        self.assertEqual(json.loads((root / "stop").read_text())["reason"], "plan_custody_changed")
        self.assertTrue(records[0].closed)

    def test_snapshot_worker_budget_writes_incomplete_report(self):
        root, plan, path, system = self.setup_watch()
        jobs, records = self.jobs(plan, system, error=m.Unavailable("scan_time_budget"))
        report = m.snapshot(plan, system, jobs, path)
        self.assertFalse(report["complete"])
        self.assertEqual(report["reason"], "scan_time_budget")
        self.assertNotIn("allocation", report)
        self.assertTrue(records[0].closed)

    def test_scanner_owns_and_cancels_only_its_direct_child_after_hard_deadline(self):
        class Process:
            returncode = None
            terminated = False
            def poll(self):
                return self.returncode
            def terminate(self):
                self.terminated = True
            def wait(self, timeout):
                self.returncode = -15
        with tempfile.TemporaryDirectory() as root:
            process = Process()
            with patch.object(m.subprocess, "Popen", return_value=process) as spawn, patch.object(m.time, "monotonic", side_effect=[0, 30]):
                job = m.ScanJob("/declared/plan.json", root)
                with self.assertRaises(m.Unavailable) as caught:
                    job.poll()
                self.assertEqual(caught.exception.reason, "scan_time_budget")
                job.close()
            self.assertTrue(process.terminated)
            self.assertEqual(spawn.call_args.args[0][-2:], ["_scan", "/declared/plan.json"])
            self.assertEqual(list(Path(root).iterdir()), [])


if __name__ == "__main__":
    unittest.main()
