"""GPU gate, tier defaults and backend override tests.

The max-tier checkpoints are ready to load on a GPU host but must never be
attempted on the CPU test host. These tests pin that contract.
"""

import os
import unittest
from unittest.mock import patch

from src.core.config import settings
from src.services.backends.base import Backend
from src.services.backends.registry import build
from src.services.backends.sources import default_stt_model
from src.services.errors import BackendNotLoadedError


class DummyGpuBackend(Backend):
    name = "dummy-gpu"
    capability = "vision"
    requires_gpu = True

    def is_available(self) -> bool:
        return True

    def _load(self) -> None:
        self.value = "loaded"


class GpuGateTests(unittest.TestCase):
    def test_cpu_refuses_gpu_backend_before_loading(self):
        with patch.object(settings, "device", "cpu"):
            backend = DummyGpuBackend()
            with self.assertRaises(BackendNotLoadedError) as ctx:
                backend.ensure_loaded()
            self.assertIn("requires a GPU", str(ctx.exception))
            self.assertFalse(backend.loaded)

    def test_cuda_allows_the_same_backend(self):
        with patch.object(settings, "device", "cuda"):
            backend = DummyGpuBackend()
            backend.ensure_loaded()
            self.assertTrue(backend.loaded)

    def test_stt_default_follows_the_device(self):
        with patch.object(settings, "device", "cpu"):
            self.assertEqual(default_stt_model(), "small")
        with patch.object(settings, "device", "cuda"):
            self.assertEqual(default_stt_model(), "large-v3")
        with patch.object(settings, "device", "mps"):
            self.assertEqual(default_stt_model(), "large-v3")


class BackendOverrideTests(unittest.TestCase):
    def test_known_override_wins_over_the_tier(self):
        with patch.dict(os.environ, {"BOTMODELS_VISION_BACKEND": "moondream"}):
            with patch(
                "src.services.backends.registry.backends_for",
                return_value={"vision": "blip"},
            ):
                self.assertEqual(build("vision").name, "moondream")

    def test_unknown_override_falls_back_to_the_tier(self):
        with patch.dict(os.environ, {"BOTMODELS_VISION_BACKEND": "does-not-exist"}):
            with patch(
                "src.services.backends.registry.backends_for",
                return_value={"vision": "blip"},
            ):
                self.assertEqual(build("vision").name, "blip")


if __name__ == "__main__":
    unittest.main()
