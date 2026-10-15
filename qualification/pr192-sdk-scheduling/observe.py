#!/usr/bin/env python3
"""Read-only provider observations for one source-bound SDK scheduling experiment."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import sys
import time
import urllib.request
import urllib.error

REPOSITORY = "UOR-Foundation/PrismPM"
REPOSITORY_ID = 1351479570
BRANCH = "fix/pr192-real-sdk-caller-qualification-20261015"
WORKFLOW = ".github/workflows/osv-input-qualification.yml"
REUSABLE = "PhaseMirror/PrismPM/.github/workflows/sdk.yml@83b7d9a9905c1bfef787e418ec5e9cc8e0f46453"
REUSABLE_SHA = "83b7d9a9905c1bfef787e418ec5e9cc8e0f46453"
CALLS = {
    "default-check": "invoke", "default-build": "invoke",
    "mutating-fetch-a": "invoke", "mutating-fetch-b": "invoke",
    "keyed-older": "invoke-readonly", "keyed-isolated": "invoke-readonly",
    "keyed-newer": "invoke-readonly",
}
MAX_BODY = 2 * 1024 * 1024
MAX_RETAINED = 32 * 1024 * 1024


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, new_url):
        return None


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def timestamp(value):
    require(isinstance(value, str) and value.endswith("Z"), "missing provider timestamp")
    return datetime.datetime.fromisoformat(value.replace("Z", "+00:00"))


def unique_json(data):
    def object_hook(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "duplicate JSON member")
            result[key] = value
        return result
    return json.loads(data, object_pairs_hook=object_hook)


class RetainedEvidence:
    """Charge every encoded byte before any owned-file or workflow-output write."""
    def __init__(self, output):
        self.output = Path(output)
        self.retained = 0
        self.names = set()

    def publish_many(self, files, external_bytes=0):
        require(type(external_bytes) is int and external_bytes >= 0, "invalid external evidence size")
        selected = set()
        size = external_bytes
        for name, data in files:
            require(isinstance(name, str) and name not in ("", ".", "..") and Path(name).name == name
                    and "/" not in name and "\\" not in name and "\0" not in name, "unsafe evidence member")
            require(type(data) is bytes, "evidence must be encoded before reservation")
            require(name not in selected and name not in self.names, "duplicate evidence member")
            require(not (self.output / name).exists() and not (self.output / name).is_symlink(), "evidence member already exists")
            selected.add(name)
            size += len(data)
        require(self.retained + size <= MAX_RETAINED, "provider evidence exceeds bound")
        # Conservative on failure: a failed or partial publication never refunds
        # its reservation and cannot authorize another write beyond the cap.
        self.retained += size
        self.names.update(selected)
        for name, data in files:
            with open(self.output / name, "xb") as stream:
                require(stream.write(data) == len(data), "short evidence publication")

    def publish_json(self, name, value):
        self.publish_many([(name, (json.dumps(value, indent=2) + "\n").encode("utf-8"))])

    def publish_observation(self, value, github_output):
        artifact = (json.dumps(value, indent=2) + "\n").encode("utf-8")
        line = ("observation=" + json.dumps(value, separators=(",", ":")) + "\n").encode("utf-8")
        self.publish_many([("observation.json", artifact)], external_bytes=len(line))
        with open(github_output, "ab") as stream:
            require(stream.write(line) == len(line), "short workflow observation publication")


class Provider(RetainedEvidence):
    def __init__(self):
        self.source = os.environ["GITHUB_SHA"]
        self.run_id = int(os.environ["GITHUB_RUN_ID"])
        require(len(self.source) == 40 and all(c in "0123456789abcdef" for c in self.source), "invalid caller SHA")
        require(os.environ["GITHUB_REPOSITORY"] == REPOSITORY, "unexpected repository")
        require(os.environ["GITHUB_REF"] == "refs/heads/" + BRANCH, "unexpected branch")
        require(os.environ["GITHUB_RUN_ATTEMPT"] == "1", "reruns are not qualification")
        output = Path(os.environ["RUNNER_TEMP"]) / ("sdk-scheduling-" + sys.argv[1])
        output.mkdir(mode=0o700, exist_ok=False)
        super().__init__(output)
        self.sequence = 0

    def get(self, suffix):
        require(suffix in ("", "/jobs?filter=all&per_page=100"), "request outside exact run")
        request = urllib.request.Request(
            f"https://api.github.com/repos/{REPOSITORY}/actions/runs/{self.run_id}" + suffix,
            headers={"Authorization": "Bearer " + os.environ["GH_TOKEN"],
                     "Accept": "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28",
                     "User-Agent": "PrismPM-PR192-scheduling-qualification"})
        try:
            response = urllib.request.build_opener(NoRedirect()).open(request, timeout=30)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            status = response.status
            headers = dict(response.headers)
            data = response.read(MAX_BODY + 1)
            require(len(data) <= MAX_BODY, "provider response exceeds bound")
        transport = (json.dumps({"url": request.full_url, "status": status, "headers": headers}, indent=2) + "\n").encode("utf-8")
        self.publish_many([(f"{self.sequence:04d}.json", data), (f"{self.sequence:04d}-transport.json", transport)])
        self.sequence += 1
        require(status == 200 and response.url == request.full_url, "unexpected provider response")
        require("rel=\"next\"" not in headers.get("Link", ""), "paginated job inventory")
        return unique_json(data)

    def snapshot(self):
        run = self.get("")
        require(run["id"] == self.run_id and run["run_attempt"] == 1, "wrong run or attempt")
        require(run["repository"]["id"] == REPOSITORY_ID and run["repository"]["full_name"] == REPOSITORY, "wrong repository authority")
        require(run["head_sha"] == self.source and run["head_branch"] == BRANCH, "wrong caller source")
        require(run["path"] == WORKFLOW and run["event"] == "workflow_dispatch", "wrong workflow or trigger")
        references = run.get("referenced_workflows", [])
        require(references and all(row["path"] == REUSABLE and row["sha"] == REUSABLE_SHA for row in references), "wrong reusable workflow source")
        body = self.get("/jobs?filter=all&per_page=100")
        jobs = body["jobs"]
        require(body["total_count"] == len(jobs) and len(jobs) <= 40, "incomplete job inventory")
        require(len({job["id"] for job in jobs}) == len(jobs), "duplicate job")
        for job in jobs:
            require(job["run_id"] == self.run_id and job["run_attempt"] == 1 and job["head_sha"] == self.source, "wrong job source or attempt")
        return jobs


def call_job(jobs, name):
    matches = [job for job in jobs if job["name"] == name + " / " + CALLS[name]]
    require(len(matches) == 1, "missing or duplicate actual SDK job: " + name)
    return matches[0]


def operation(job, readonly):
    steps = job["steps"]
    require(len({step["number"] for step in steps}) == len(steps), "duplicate step")
    if readonly:
        fetches = [step for step in steps if step["name"] == "Fetch locked SDK inputs"]
        require(len(fetches) == 1, "missing actual fetch step")
        fetch = fetches[0]
        require(fetch["status"] == "completed" and fetch["conclusion"] == "success", "fetch has not succeeded")
        matches = [step for step in steps if step["number"] == fetch["number"] + 1]
    else:
        matches = [step for step in steps if "UOR-Foundation/PrismPM/action@f785ee46d794886c29b7efe80c2b261e48733a15" in step["name"]]
    require(len(matches) == 1 and not matches[0]["name"].startswith("Post "), "missing unambiguous actual operation step")
    return matches[0]


def successful(job, readonly):
    require(job["status"] == "completed" and job["conclusion"] == "success", "SDK job did not succeed")
    step = operation(job, readonly)
    require(step["status"] == "completed" and step["conclusion"] == "success", "SDK operation did not succeed")
    require(timestamp(job["started_at"]) <= timestamp(step["started_at"]) <= timestamp(step["completed_at"]) <= timestamp(job["completed_at"]), "operation outside job lifetime")
    start, end = timestamp(job["started_at"]), timestamp(job["completed_at"])
    require(start <= end, "reversed operation interval")
    return start, end


def overlap(intervals):
    require(max(interval[0] for interval in intervals) < min(interval[1] for interval in intervals), "genuine SDK jobs did not overlap")


def observe(provider):
    deadline = time.monotonic() + 350 * 60
    while True:
        require(time.monotonic() < deadline, "no genuine replacement opportunity within original job envelope")
        jobs = provider.snapshot()
        selected = []
        for name in ("keyed-older", "keyed-isolated"):
            matches = [job for job in jobs if job["name"] == name + " / invoke-readonly"]
            if not matches:
                break
            require(len(matches) == 1, "duplicate keyed job")
            job = matches[0]
            require(job["status"] != "completed", "keyed operation finished before replacement witness")
            try:
                step = operation(job, True)
            except AssertionError:
                break
            if step["status"] != "in_progress":
                break
            selected.append({"name": name, "job_id": job["id"], "step_number": step["number"], "started_at": step["started_at"]})
        if len(selected) == 2:
            observation = {"run_id": provider.run_id, "run_attempt": 1, "source": provider.source,
                           "observed_at": datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z"),
                           "operations": selected}
            provider.publish_observation(observation, os.environ["GITHUB_OUTPUT"])
            return
        time.sleep(min(20, max(0, deadline - time.monotonic())))


def finish(provider):
    jobs = provider.snapshot()
    controls = {"admit", "observe-replacement", "verify-scheduling"}
    allowed = controls | {name + " / " + branch for name in CALLS for branch in ("validate-invocation-key", "invoke", "invoke-readonly")}
    require(all(job["name"] in allowed for job in jobs), "unexpected job owner")
    require(len({job["name"] for job in jobs}) == len(jobs), "duplicate job name")
    for name, branch in CALLS.items():
        for job in jobs:
            if job["name"].startswith(name + " / ") and job["name"] != name + " / " + branch:
                expected = "success" if branch == "invoke-readonly" and job["name"].endswith(" / validate-invocation-key") else "skipped"
                require(job["status"] == "completed" and job["conclusion"] == expected, "unexpected conditional SDK branch")
        if branch == "invoke-readonly":
            require(any(job["name"] == name + " / validate-invocation-key" for job in jobs), "missing key validator")
    default = [successful(call_job(jobs, name), True) for name in ("default-check", "default-build")]
    mutating = [successful(call_job(jobs, name), False) for name in ("mutating-fetch-a", "mutating-fetch-b")]
    overlap(default)
    overlap(mutating)
    observation = unique_json(os.environ["REPLACEMENT_OBSERVATION"])
    require(observation["run_id"] == provider.run_id and observation["run_attempt"] == 1 and observation["source"] == provider.source, "wrong observation authority")
    timestamp(observation["observed_at"])
    require([row["name"] for row in observation["operations"]] == ["keyed-older", "keyed-isolated"], "wrong observed operations")
    older = call_job(jobs, "keyed-older")
    isolated = call_job(jobs, "keyed-isolated")
    newer = call_job(jobs, "keyed-newer")
    require(older["status"] == "completed" and older["conclusion"] == "cancelled", "older same-key job was not cancelled")
    old_step = operation(older, True)
    require(old_step["status"] == "completed" and old_step["conclusion"] == "cancelled", "actual old SDK operation was not interrupted")
    for row, job in zip(observation["operations"], (older, isolated)):
        step = operation(job, True)
        require(row["job_id"] == job["id"] and row["step_number"] == step["number"] and row["started_at"] == step["started_at"], "observation changed owner")
    isolated_interval = successful(isolated, True)
    newer_interval = successful(newer, True)
    require(timestamp(older["started_at"]) < newer_interval[0], "replacement preceded older job")
    require(max(timestamp(older["completed_at"]), newer_interval[0]) < isolated_interval[1], "different-key job did not survive actual replacement")
    # Local observation wall time is diagnostic only: do not invent a strict
    # clock/precision join with GitHub's rounded provider timestamps. The genuine
    # observed running states and immutable workflow DAG bind replacement order.
    result = {"scope": "PR192 historical-SDK scheduling only", "source": provider.source,
              "run_id": provider.run_id, "run_attempt": 1, "sdk_invocations": 7,
              "default_pair": "success-with-job-overlap", "mutating_pair": "success-with-job-overlap",
              "older_same_key": "cancelled", "newer_same_key": "success", "different_key": "success",
              "sdk_product_accepted": False, "current_release_accepted": False}
    provider.publish_json("result.json", result)


def admit():
    require(os.environ["ALLOW_EXPENSIVE_SDK_CALLS"] == "true", "maintainer cost authorization required")
    source = os.environ["GITHUB_SHA"]
    require(source == os.environ["QUALIFICATION_SOURCE_REVISION"], "dispatch does not name exact caller source")
    require(os.environ["GITHUB_REPOSITORY"] == REPOSITORY and os.environ["GITHUB_REF"] == "refs/heads/" + BRANCH, "wrong preparation repository/ref")
    require(os.environ["GITHUB_RUN_ATTEMPT"] == "1", "reruns are not qualification")
    root = Path(__file__).resolve().parent
    provenance = unique_json((root / "consumer-source.json").read_bytes())
    require(provenance["revision"] == "d0174e1d64339f73091fe4c59d5d6bf532a37d1f" and provenance["fixture"] == "HO-11", "wrong historical consumer")
    expected = set()
    for row in provenance["files"]:
        require(row["path"] not in expected and not row["path"].startswith("/") and all(part not in ("", ".", "..") for part in row["path"].split("/")), "unsafe or duplicate consumer member")
        expected.add(row["path"])
        path = root / "consumer" / row["path"]
        require(path.is_file() and not path.is_symlink(), "consumer member is not a regular file")
        data = path.read_bytes()
        require(len(data) == row["bytes"] and hashlib.sha256(data).hexdigest() == row["sha256"], "consumer member differs from historical source")
        require(hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest() == row["blob"], "consumer Git blob differs")
    actual = {str(path.relative_to(root / "consumer")) for path in (root / "consumer").rglob("*") if path.is_file()}
    require(actual == expected and len(expected) == 29, "incomplete consumer source closure")
    sdk_lock = unique_json((root / "consumer/prismpm.lock").read_bytes())
    require(sdk_lock["sdk_image"] == "ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21", "wrong original SDK lock image")
    require(sdk_lock["standards_lock"] == "sha256:" + hashlib.sha256((root / "consumer/standards.lock").read_bytes()).hexdigest(), "original SDK and authority lock disagree")
    print(json.dumps({"source": source, "sdk_calls": 7, "maximum_sdk_wave": 3,
                      "compressed_image_bytes_per_cold_amd64_runner": 4162398277,
                      "expanded_storage_and_authority_downloads": "additional; not inferred", "source_only_prerequisite": True}))


if __name__ == "__main__":
    require(sys.argv[1:] in (["admit"], ["observe"], ["finish"]), "expected admit, observe or finish")
    if sys.argv[1] == "admit":
        admit()
    else:
        provider = Provider()
        (observe if sys.argv[1] == "observe" else finish)(provider)
