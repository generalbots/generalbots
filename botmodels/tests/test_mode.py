"""Unit tests for min/max tier resolution.

These exercise `src.core.hardware` and `src.core.mode` directly, so they need
neither torch nor psutil installed. Run with:

    python3 -m unittest discover tests
"""

import os
import unittest
from unittest.mock import patch

from src.core.hardware import HardwareProfile
from src.core.mode import BACKENDS, SHARED_BACKENDS, InvalidModeError, backends_for, resolve_mode


def profile(
    device="cpu", cuda=False, mps=False, vram_gb=None, ram_gb=64.0
) -> HardwareProfile:
    return HardwareProfile(
        device=device,
        cuda=cuda,
        mps=mps,
        vram_gb=vram_gb,
        ram_gb=ram_gb,
        cpu_count=8,
        platform="test",
        python="3.12.0",
    )


class ResolveSettingTests(unittest.TestCase):
    def test_explicit_min_is_env_sourced(self):
        mode, source, _ = resolve_mode("min")
        self.assertEqual(mode, "min")
        self.assertEqual(source, "env")

    def test_explicit_max_forced_without_detection(self):
        # No GPU present, yet max is forced: the override must win.
        mode, source, _ = resolve_mode("max")
        self.assertEqual(mode, "max")
        self.assertEqual(source, "env")

    def test_invalid_setting_raises(self):
        with self.assertRaises(InvalidModeError):
            resolve_mode("bogus")

    def test_none_defaults_to_auto(self):
        mode, source, _ = resolve_mode(None)
        self.assertEqual(source, "auto")
        self.assertIn(mode, ("min", "max"))

    def test_whitespace_and_case_normalised(self):
        mode, source, _ = resolve_mode("  MIN  ")
        self.assertEqual((mode, source), ("min", "env"))

    def test_uppercase_auto_accepted(self):
        mode, source, _ = resolve_mode("AUTO")
        self.assertEqual(source, "auto")


class AutoTierTests(unittest.TestCase):
    def test_no_gpu_resolves_min(self):
        with patch("src.core.mode.probe", return_value=profile(ram_gb=64.0)):
            mode, source, _ = resolve_mode("auto")
        self.assertEqual((mode, source), ("min", "auto"))

    def test_large_gpu_resolves_max(self):
        hw = profile(device="cuda", cuda=True, vram_gb=24.0, ram_gb=64.0)
        with patch("src.core.mode.probe", return_value=hw):
            mode, source, _ = resolve_mode("auto")
        self.assertEqual((mode, source), ("max", "auto"))

    def test_insufficient_vram_resolves_min(self):
        hw = profile(device="cuda", cuda=True, vram_gb=12.0, ram_gb=64.0)
        with patch("src.core.mode.probe", return_value=hw):
            mode, _source, _hw = resolve_mode("auto")
        self.assertEqual(mode, "min")

    def test_insufficient_ram_resolves_min(self):
        hw = profile(device="cuda", cuda=True, vram_gb=24.0, ram_gb=16.0)
        with patch("src.core.mode.probe", return_value=hw):
            mode, _source, _hw = resolve_mode("auto")
        self.assertEqual(mode, "min")

    def test_mps_keys_off_ram_not_vram(self):
        # Unified memory: get_device_properties total_memory is not meaningful,
        # so a large RAM figure alone must select max.
        hw = profile(device="mps", mps=True, vram_gb=None, ram_gb=64.0)
        with patch("src.core.mode.probe", return_value=hw):
            mode, _source, _hw = resolve_mode("auto")
        self.assertEqual(mode, "max")

    def test_mps_with_low_ram_resolves_min(self):
        hw = profile(device="mps", mps=True, ram_gb=16.0)
        with patch("src.core.mode.probe", return_value=hw):
            mode, _source, _hw = resolve_mode("auto")
        self.assertEqual(mode, "min")

    def test_unknown_ram_does_not_promote_to_max(self):
        # A container where RAM cannot be detected must not load a 7B model.
        hw = profile(device="cuda", cuda=True, vram_gb=48.0, ram_gb=None)
        with patch("src.core.mode.probe", return_value=hw):
            mode, _source, _hw = resolve_mode("auto")
        self.assertEqual(mode, "min")


class BackendTableTests(unittest.TestCase):
    def test_every_capability_resolves(self):
        for tier in ("min", "max"):
            resolved = backends_for(tier)
            for capability in ("image", "vision", "stt", "tts", "ocr", "video"):
                self.assertIn(capability, resolved)

    def test_min_keeps_established_models(self):
        resolved = backends_for("min")
        self.assertEqual(resolved["image"], "sd-turbo")
        self.assertEqual(resolved["vision"], "blip2")
        self.assertEqual(resolved["tts"], "piper")
        self.assertEqual(resolved["ocr"], "tesseract")
        self.assertEqual(resolved["video"], "zeroscope")

    def test_music_identical_in_both_tiers(self):
        self.assertEqual(backends_for("min")["music"], "ace-step-1.5")
        self.assertEqual(backends_for("max")["music"], "ace-step-1.5")

    def test_tiers_differ_per_capability(self):
        minimum = backends_for("min")
        maximum = backends_for("max")
        differing = [c for c in minimum if minimum[c] != maximum[c]]
        self.assertNotIn("music", differing)
        self.assertEqual(
            set(differing),
            {"image", "vision", "stt", "tts", "ocr", "video"},
        )

    def test_shared_backends_merged(self):
        for name in SHARED_BACKENDS:
            self.assertIn(name, backends_for("min"))
            self.assertIn(name, backends_for("max"))

    def test_table_covers_expected_capabilities(self):
        self.assertEqual(
            set(BACKENDS["min"]), {"image", "vision", "stt", "tts", "ocr", "video"}
        )


if __name__ == "__main__":
    unittest.main()