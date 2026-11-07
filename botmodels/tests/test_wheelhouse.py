"""Unit tests for the CPU-tier wheelhouse helper.

These stay offline: they exercise planning, requirement parsing and inspection,
and run `build` with `--dry-run` so CI can verify the pip command without
downloading multi-gigabyte wheels.

Run with: python3 -m unittest discover tests
"""

import contextlib
import importlib.util
import io
import json
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent.parent / "scripts" / "wheelhouse.py"
SPEC = importlib.util.spec_from_file_location("wheelhouse", SCRIPT)
wheelhouse = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(wheelhouse)


class PlanTests(unittest.TestCase):
    def test_barcode_extra_is_part_of_the_cpu_tier(self):
        names = wheelhouse.cpu_tier_names()
        self.assertIn("pyzbar", names)
        self.assertIn("torch", names)
        self.assertNotIn("nvidia", names)

    def test_install_budget_has_a_documented_floor(self):
        self.assertEqual(
            wheelhouse.install_required_bytes(0),
            wheelhouse.MIN_FREE_GIB * wheelhouse.GIB,
        )

    def test_plan_json_is_deterministic_and_offline(self):
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            self.assertEqual(wheelhouse.main(["plan", "--json"]), 0)
        payload = json.loads(output.getvalue())
        self.assertEqual(payload["python"], "3.12")
        self.assertEqual(payload["abi"], "cp312")
        self.assertIn("download.pytorch.org/whl/cpu", payload["commands"][0])
        self.assertGreater(payload["estimated_wheelhouse_bytes"], 0)

    def test_build_dry_run_prints_both_passes_without_downloading(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                self.assertEqual(
                    wheelhouse.main(
                        ["build", "--output", tmp, "--python-version", "3.12", "--dry-run"]
                    ),
                    0,
                )
            text = output.getvalue()
            self.assertIn("pip download", text)
            self.assertIn("--only-binary=:all:", text)
            self.assertIn("manylinux_2_28_x86_64", text)
            self.assertEqual(list(Path(tmp).glob("*.whl")), [])


class InspectTests(unittest.TestCase):
    def test_inspect_sums_fake_wheels_and_uses_the_install_formula(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "torch-2.9.0+cpu-cp312-cp312-manylinux_2_28_x86_64.whl").write_bytes(
                b"x" * 1024
            )
            (root / "pyzbar-0.1.9-py3-none-any.whl").write_bytes(b"y" * 512)
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                self.assertEqual(wheelhouse.main(["inspect", tmp]), 0)
            self.assertIn("2 wheels", output.getvalue())
            self.assertEqual(wheelhouse.directory_bytes(root), 1536)

    def test_torch_versions_are_read_from_wheel_names(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "torch-2.9.0+cpu-cp312-cp312-manylinux_2_28_x86_64.whl").write_bytes(b"")
            (root / "torchaudio-2.9.0+cpu-cp312-cp312-manylinux_2_28_x86_64.whl").write_bytes(b"")
            versions = wheelhouse.torch_versions(root)
            self.assertEqual(versions["torch"], "2.9.0+cpu")
            self.assertEqual(versions["torchaudio"], "2.9.0+cpu")


if __name__ == "__main__":
    unittest.main()
