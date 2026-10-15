"""Synthetic DATA parser controls only; these never stand in for SDK execution."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("sdk_scheduling_observe", Path(__file__).with_name("observe.py"))
M = importlib.util.module_from_spec(spec)
spec.loader.exec_module(M)


def stamp(seconds):
    return f"2026-10-15T00:00:{seconds:02d}Z"


def job(name, index, start=1, end=50):
    readonly = M.CALLS[name] == "invoke-readonly" or name.startswith("default-")
    steps = [{"number": 3, "name": "UOR-Foundation/PrismPM/action@f785ee46d794886c29b7efe80c2b261e48733a15",
              "status": "completed", "conclusion": "success", "started_at": stamp(start + 1), "completed_at": stamp(end - 1)}]
    if readonly:
        steps.insert(0, {"number": 2, "name": "Fetch locked SDK inputs", "status": "completed", "conclusion": "success"})
    return {"id": index, "name": name + " / " + M.CALLS[name], "status": "completed", "conclusion": "success",
            "started_at": stamp(start), "completed_at": stamp(end), "steps": steps}


def data():
    names = list(M.CALLS)
    jobs = [job(name, index + 1) for index, name in enumerate(names)]
    older = next(row for row in jobs if row["name"].startswith("keyed-older /"))
    older["conclusion"] = "cancelled"
    older["completed_at"] = stamp(30)
    older["steps"][-1]["conclusion"] = "cancelled"
    older["steps"][-1]["completed_at"] = stamp(29)
    newer = next(row for row in jobs if row["name"].startswith("keyed-newer /"))
    newer["started_at"] = stamp(31)
    newer["steps"][-1]["started_at"] = stamp(32)
    for index, name in enumerate(("keyed-older", "keyed-isolated", "keyed-newer")):
        jobs.append({"id": 20 + index, "name": name + " / validate-invocation-key", "status": "completed", "conclusion": "success"})
    observation = {"run_id": 1, "run_attempt": 1, "source": "a" * 40, "observed_at": stamp(20),
                   "operations": [{"name": name, "job_id": M.call_job(jobs, name)["id"], "step_number": 3, "started_at": stamp(2)}
                                  for name in ("keyed-older", "keyed-isolated")]}
    return jobs, observation


class ParserControls(unittest.TestCase):
    def finish(self, jobs, observation):
        with tempfile.TemporaryDirectory() as directory:
            class DataOnly(M.RetainedEvidence):
                source = "a" * 40
                run_id = 1
                output = Path(directory)
                def __init__(self):
                    super().__init__(self.output)
                def snapshot(self):
                    return copy.deepcopy(jobs)
            with patch.dict(os.environ, {"REPLACEMENT_OBSERVATION": json.dumps(observation)}):
                M.finish(DataOnly())
            return json.loads((Path(directory) / "result.json").read_text())

    def test_complete_metadata_parser_positive_is_not_sdk_acceptance(self):
        jobs, observation = data()
        result = self.finish(jobs, observation)
        self.assertFalse(result["sdk_product_accepted"])
        self.assertEqual(result["sdk_invocations"], 7)

    def test_default_sibling_cancellation_refused(self):
        jobs, observation = data()
        M.call_job(jobs, "default-check")["conclusion"] = "cancelled"
        with self.assertRaisesRegex(AssertionError, "did not succeed"):
            self.finish(jobs, observation)

    def test_mutating_sibling_cancellation_refused(self):
        jobs, observation = data()
        M.call_job(jobs, "mutating-fetch-b")["conclusion"] = "cancelled"
        with self.assertRaisesRegex(AssertionError, "did not succeed"):
            self.finish(jobs, observation)

    def test_default_serial_jobs_do_not_prove_independence(self):
        jobs, observation = data()
        first = M.call_job(jobs, "default-check")
        first["completed_at"] = stamp(1)
        first["steps"][-1].update(started_at=stamp(1), completed_at=stamp(1))
        with self.assertRaisesRegex(AssertionError, "did not overlap"):
            self.finish(jobs, observation)

    def test_mutating_serial_jobs_do_not_prove_independence(self):
        jobs, observation = data()
        first = M.call_job(jobs, "mutating-fetch-a")
        first["completed_at"] = stamp(1)
        first["steps"][-1].update(started_at=stamp(1), completed_at=stamp(1))
        with self.assertRaisesRegex(AssertionError, "did not overlap"):
            self.finish(jobs, observation)

    def test_old_same_key_success_is_not_supersession(self):
        jobs, observation = data()
        M.call_job(jobs, "keyed-older")["conclusion"] = "success"
        with self.assertRaisesRegex(AssertionError, "was not cancelled"):
            self.finish(jobs, observation)

    def test_old_post_operation_cancellation_is_not_operation_kill(self):
        jobs, observation = data()
        M.call_job(jobs, "keyed-older")["steps"][-1]["conclusion"] = "success"
        with self.assertRaisesRegex(AssertionError, "not interrupted"):
            self.finish(jobs, observation)

    def test_isolated_key_cancellation_refused(self):
        jobs, observation = data()
        M.call_job(jobs, "keyed-isolated")["conclusion"] = "cancelled"
        with self.assertRaisesRegex(AssertionError, "did not succeed"):
            self.finish(jobs, observation)

    def test_isolated_key_completed_before_replacement_is_no_witness(self):
        jobs, observation = data()
        row = M.call_job(jobs, "keyed-isolated")
        row["completed_at"] = stamp(25)
        row["steps"][-1]["completed_at"] = stamp(24)
        with self.assertRaisesRegex(AssertionError, "did not survive"):
            self.finish(jobs, observation)

    def test_forged_observation_owner_refused(self):
        jobs, observation = data()
        observation["operations"][0]["job_id"] = 999
        with self.assertRaisesRegex(AssertionError, "changed owner"):
            self.finish(jobs, observation)

    def test_wrong_observation_source_refused(self):
        jobs, observation = data()
        observation["source"] = "b" * 40
        with self.assertRaisesRegex(AssertionError, "authority"):
            self.finish(jobs, observation)

    def test_missing_actual_sdk_job_refused(self):
        jobs, observation = data()
        jobs.remove(M.call_job(jobs, "default-build"))
        with self.assertRaisesRegex(AssertionError, "missing or duplicate"):
            self.finish(jobs, observation)

    def test_unknown_job_refused(self):
        jobs, observation = data()
        jobs.append({"id": 100, "name": "unregistered-feature"})
        with self.assertRaisesRegex(AssertionError, "unexpected job"):
            self.finish(jobs, observation)

    def test_missing_real_key_validation_refused(self):
        jobs, observation = data()
        jobs[:] = [row for row in jobs if row["name"] != "keyed-newer / validate-invocation-key"]
        with self.assertRaisesRegex(AssertionError, "missing key validator"):
            self.finish(jobs, observation)

    def test_duplicate_json_members_refused(self):
        with self.assertRaisesRegex(AssertionError, "duplicate JSON"):
            M.unique_json('{"id":1,"id":2}')

    def test_actual_fetch_success_required(self):
        row = job("default-check", 1)
        row["steps"][0]["conclusion"] = "failure"
        with self.assertRaisesRegex(AssertionError, "fetch has not succeeded"):
            M.successful(row, True)


class RetainedEvidenceControls(unittest.TestCase):
    """Only DATA accounting controls; no provider fact or SDK execution is mocked in."""
    def test_body_and_transport_sidecar_exact_boundary_are_both_charged(self):
        with tempfile.TemporaryDirectory() as directory:
            evidence = M.RetainedEvidence(directory)
            with patch.object(M, "MAX_RETAINED", 7):
                evidence.publish_many([("0000.json", b"{}"), ("0000-transport.json", b"12345")])
            self.assertEqual(evidence.retained, 7)
            self.assertEqual(sum(p.stat().st_size for p in Path(directory).iterdir()), 7)

    def test_sidecar_over_boundary_refuses_before_either_file_is_written(self):
        with tempfile.TemporaryDirectory() as directory:
            evidence = M.RetainedEvidence(directory)
            with patch.object(M, "MAX_RETAINED", 6):
                with self.assertRaisesRegex(AssertionError, "evidence exceeds bound"):
                    evidence.publish_many([("0000.json", b"{}"), ("0000-transport.json", b"12345")])
            self.assertEqual(evidence.retained, 0)
            self.assertEqual(list(Path(directory).iterdir()), [])

    def actual_get_data(self, directory, limit):
        # Synthetic HTTP DATA exercises the unchanged real get() write path;
        # it supplies no source-bound provider observation or SDK result.
        endpoint = "https://api.github.com/repos/UOR-Foundation/PrismPM/actions/runs/1"
        headers = {"Content-Type": "application/json", "X-Unit-Data": "sidecar"}
        class Response:
            status = 200
            url = endpoint
            def __enter__(self): return self
            def __exit__(self, *args): pass
            def read(self, cap):
                self.cap = cap
                return b"{}"
        response = Response(); response.headers = headers
        provider = M.Provider.__new__(M.Provider)
        M.RetainedEvidence.__init__(provider, directory)
        provider.run_id = 1; provider.sequence = 0
        with patch.dict(os.environ, {"GH_TOKEN": "synthetic-data-not-a-credential"}), patch.object(M, "MAX_RETAINED", limit), patch.object(M.urllib.request, "build_opener") as opener:
            opener.return_value.open.return_value = response
            result = provider.get("")
        return provider, result

    def test_actual_get_charges_full_transport_and_response_at_exact_boundary(self):
        endpoint = "https://api.github.com/repos/UOR-Foundation/PrismPM/actions/runs/1"
        headers = {"Content-Type": "application/json", "X-Unit-Data": "sidecar"}
        transport = (json.dumps({"url": endpoint, "status": 200, "headers": headers}, indent=2) + "\n").encode("utf-8")
        with tempfile.TemporaryDirectory() as directory:
            provider, result = self.actual_get_data(directory, 2 + len(transport))
            self.assertEqual(result, {})
            self.assertEqual(provider.retained, sum(p.stat().st_size for p in Path(directory).iterdir()))
            self.assertEqual(provider.retained, 2 + len(transport))

    def test_actual_get_sidecar_overflow_is_not_body_only_success(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(AssertionError, "evidence exceeds bound"):
                self.actual_get_data(directory, 2)
            self.assertEqual(list(Path(directory).iterdir()), [])

    def test_observation_includes_artifact_and_exact_workflow_output_bytes(self):
        value = {"source": "a" * 40, "operations": []}
        artifact = (json.dumps(value, indent=2) + "\n").encode("utf-8")
        line = ("observation=" + json.dumps(value, separators=(",", ":")) + "\n").encode("utf-8")
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "artifacts"; target.mkdir()
            output = Path(directory) / "workflow-output"
            evidence = M.RetainedEvidence(target)
            with patch.object(M, "MAX_RETAINED", len(artifact) + len(line)):
                evidence.publish_observation(value, output)
            self.assertEqual(evidence.retained, len(artifact) + len(line))
            self.assertEqual((target / "observation.json").read_bytes(), artifact)
            self.assertEqual(output.read_bytes(), line)

    def test_workflow_output_would_exceed_boundary_even_if_artifact_fits(self):
        value = {"run_id": 1}
        artifact = (json.dumps(value, indent=2) + "\n").encode("utf-8")
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "artifacts"; target.mkdir()
            output = Path(directory) / "workflow-output"
            evidence = M.RetainedEvidence(target)
            with patch.object(M, "MAX_RETAINED", len(artifact)):
                with self.assertRaisesRegex(AssertionError, "evidence exceeds bound"):
                    evidence.publish_observation(value, output)
            self.assertFalse(output.exists())
            self.assertEqual(list(target.iterdir()), [])
            self.assertEqual(evidence.retained, 0)

    def test_result_json_cannot_escape_an_already_consumed_budget(self):
        value = {"sdk_product_accepted": False}
        encoded = (json.dumps(value, indent=2) + "\n").encode("utf-8")
        with tempfile.TemporaryDirectory() as directory:
            evidence = M.RetainedEvidence(directory)
            with patch.object(M, "MAX_RETAINED", len(encoded)):
                evidence.publish_many([("prior.json", b"x")])
                with self.assertRaisesRegex(AssertionError, "evidence exceeds bound"):
                    evidence.publish_json("result.json", value)
            self.assertEqual(evidence.retained, 1)
            self.assertFalse((Path(directory) / "result.json").exists())

    def test_duplicate_owned_member_never_overwrites_original_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            evidence = M.RetainedEvidence(directory)
            evidence.publish_many([("result.json", b"original")])
            with self.assertRaisesRegex(AssertionError, "duplicate evidence"):
                evidence.publish_many([("result.json", b"replacement")])
            self.assertEqual((Path(directory) / "result.json").read_bytes(), b"original")
            self.assertEqual(evidence.retained, len(b"original"))


if __name__ == "__main__":
    unittest.main()
