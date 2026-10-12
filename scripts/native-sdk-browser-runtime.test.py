"""Real bounded subprocess controls; not browser or SDK qualification."""
import importlib.util
import pathlib
import sys
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('readiness', pathlib.Path(__file__).with_name('native-sdk-browser-runtime.py'))
readiness = importlib.util.module_from_spec(spec)
spec.loader.exec_module(readiness)


class SubprocessCustody(unittest.TestCase):
    def test_actual_success_and_complete_log(self):
        with tempfile.TemporaryFile() as log:
            result = readiness.execute([sys.executable, '-c', 'print("actual child")'], log=log)
            log.seek(0)
            self.assertEqual(result, 'actual child\n')
            self.assertEqual(log.read(), result.encode())

    def test_actual_nonzero_is_refused_and_preserved(self):
        with tempfile.TemporaryFile() as log:
            with self.assertRaisesRegex(ValueError, 'exited 7'):
                readiness.execute([sys.executable, '-c', 'print("first failure");raise SystemExit(7)'], log=log)
            log.seek(0)
            self.assertEqual(log.read(), b'first failure\n')

    def test_real_stderr_progress_is_preserved_without_becoming_identity(self):
        with tempfile.TemporaryFile() as log:
            result = readiness.execute([sys.executable, '-c',
                'import sys;print("pull progress",file=sys.stderr);print("actual identity")'], log=log)
            self.assertEqual(result, 'actual identity\n')
            log.seek(0)
            recorded = log.read()
            self.assertIn(b'pull progress\n', recorded)
            self.assertIn(b'actual identity\n', recorded)

    def test_actual_output_bound_refuses_before_log_allocation(self):
        with tempfile.TemporaryFile() as log:
            with self.assertRaisesRegex(ValueError, 'bounded command output'):
                readiness.execute([sys.executable, '-c', 'print("x"*65536)'], limit=16, log=log)
            self.assertLessEqual(log.tell(), 16)

    def test_actual_timeout_reaps_the_original_client(self):
        actual = readiness.subprocess.Popen
        observed = []
        def retain(*args, **kwargs):
            child = actual(*args, **kwargs)
            observed.append(child)
            return child
        # Observe the genuine Popen; do not replace its execution or status.
        with patch.object(readiness.subprocess, 'Popen', retain):
            with self.assertRaisesRegex(ValueError, 'bounded command deadline'):
                readiness.execute([sys.executable, '-c', 'import time;time.sleep(30)'], timeout=0.1)
        self.assertEqual(len(observed), 1)
        self.assertIsNotNone(observed[0].poll())
        self.assertTrue(observed[0].stdout.closed)
        self.assertTrue(observed[0].stderr.closed)

    def test_cleanup_log_failure_does_not_prevent_real_command_completion(self):
        class RefusedLog:
            def write(self, block):
                raise OSError('recording refused')
        observed = []
        result = readiness.execute([sys.executable, '-c', 'print("real cleanup client")'],
            log=RefusedLog(), on_log_error=observed.append)
        self.assertEqual(result, 'real cleanup client\n')
        self.assertEqual(len(observed), 1)
        self.assertIsInstance(observed[0], OSError)


if __name__ == '__main__':
    unittest.main()
