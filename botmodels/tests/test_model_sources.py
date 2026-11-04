"""Model source resolution and tier selection tests.

No weights are loaded: these guard the routing decisions that previously made a
fresh install fail even with a warm Hub cache.
"""

import tempfile
import unittest
from pathlib import Path

from src.core.mode import BACKENDS
from src.services.backends.sources import (
    BLIP_CAPTION_MODEL,
    resolve_model_source,
)


class ResolveModelSourceTests(unittest.TestCase):
    def test_existing_local_path_wins(self):
        with tempfile.TemporaryDirectory() as tmp:
            self.assertEqual(resolve_model_source(tmp, "org/model"), tmp)

    def test_missing_local_path_falls_back_to_hub_id(self):
        missing = "/nonexistent/models/does-not-exist"
        self.assertEqual(resolve_model_source(missing, BLIP_CAPTION_MODEL), BLIP_CAPTION_MODEL)

    def test_blank_configuration_uses_hub_id(self):
        self.assertEqual(resolve_model_source("", BLIP_CAPTION_MODEL), BLIP_CAPTION_MODEL)
        self.assertEqual(resolve_model_source(None, BLIP_CAPTION_MODEL), BLIP_CAPTION_MODEL)

    def test_relative_path_is_expanded(self):
        with tempfile.TemporaryDirectory() as tmp:
            candidate = Path(tmp) / "weights"
            candidate.mkdir()
            self.assertEqual(resolve_model_source(str(candidate), "org/model"), str(candidate))


class TierSelectionTests(unittest.TestCase):
    def test_min_tier_uses_low_ram_captioner(self):
        self.assertEqual(BACKENDS["min"]["vision"], "blip")

    def test_min_tier_keeps_local_speech(self):
        self.assertEqual(BACKENDS["min"]["stt"], "faster-whisper")
        self.assertEqual(BACKENDS["min"]["tts"], "piper")


if __name__ == "__main__":
    unittest.main()
