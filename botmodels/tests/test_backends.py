"""Unit tests for backend load semantics.

Verifies the fixes from issue 1516: a failed load raises BackendNotLoadedError
rather than leaving None to be called, and a failure is not retried on every
request (which previously re-ran from_pretrained each time).
"""

import unittest

from src.services.backends.base import Backend
from src.services.errors import BackendNotLoadedError, UnsupportedBackendError


class FlakyBackend(Backend):
    name = "flaky"
    capability = "test"

    def __init__(self):
        super().__init__()
        self.load_calls = 0
        self.should_fail = True

    def _load(self):
        self.load_calls += 1
        if self.should_fail:
            raise RuntimeError("weights not found at ./models/nope")
        self.value = "loaded"


class SuccessBackend(Backend):
    name = "ok"
    capability = "test"

    def __init__(self):
        super().__init__()
        self.load_calls = 0

    def _load(self):
        self.load_calls += 1
        self.value = "loaded"


class LoadFailureTests(unittest.TestCase):
    def test_failed_load_raises_typed_error(self):
        backend = FlakyBackend()
        with self.assertRaises(BackendNotLoadedError) as ctx:
            backend.ensure_loaded()
        self.assertEqual(ctx.exception.backend, "flaky")
        self.assertEqual(ctx.exception.capability, "test")
        self.assertIn("weights not found", ctx.exception.cause)

    def test_failure_is_not_retried_on_later_calls(self):
        """The retry-storm bug: every request re-ran from_pretrained."""
        backend = FlakyBackend()
        for _ in range(5):
            with self.assertRaises(BackendNotLoadedError):
                backend.ensure_loaded()
        self.assertEqual(backend.load_calls, 1)

    def test_error_recorded_and_cleared_by_reset(self):
        backend = FlakyBackend()
        with self.assertRaises(BackendNotLoadedError):
            backend.ensure_loaded()
        self.assertIsNotNone(backend.load_error)

        backend.reset()
        self.assertIsNone(backend.load_error)
        self.assertFalse(backend.loaded)

        backend.should_fail = False
        backend.ensure_loaded()
        self.assertTrue(backend.loaded)

    def test_loaded_flag_set_once(self):
        backend = SuccessBackend()
        backend.ensure_loaded()
        backend.ensure_loaded()
        self.assertEqual(backend.load_calls, 1)


class ThreadSafetyTests(unittest.TestCase):
    def test_concurrent_loads_trigger_single_from_pretrained(self):
        """Guards the unlocked-singleton race on first request."""
        import threading

        backend = SuccessBackend()
        errors: list[Exception] = []

        def worker():
            try:
                backend.ensure_loaded()
            except Exception as exc:  # noqa: BLE001
                errors.append(exc)

        threads = [threading.Thread(target=worker) for _ in range(12)]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join()

        self.assertEqual(errors, [])
        self.assertEqual(backend.load_calls, 1)


class StatusTests(unittest.TestCase):
    def test_status_reports_error(self):
        backend = FlakyBackend()
        backend.ensure_loaded_error = None
        with self.assertRaises(BackendNotLoadedError):
            backend.ensure_loaded()

        status = backend.status()
        self.assertEqual(status["backend"], "flaky")
        self.assertFalse(status["loaded"])
        self.assertTrue(status["available"])
        self.assertIsNotNone(status["error"])

    def test_status_does_not_trigger_load(self):
        backend = SuccessBackend()
        backend.status()
        self.assertEqual(backend.load_calls, 0)


class ErrorMessageTests(unittest.TestCase):
    def test_unsupported_backend_lists_available(self):
        with self.assertRaises(UnsupportedBackendError) as ctx:
            raise UnsupportedBackendError("image", "nope", ["sd-turbo", "qwen-image"])
        message = str(ctx.exception)
        self.assertIn("nope", message)
        self.assertIn("sd-turbo", message)


if __name__ == "__main__":
    unittest.main()