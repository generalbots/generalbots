"""Tests for media capability reporting without libzbar.

The desktop pythonbox deliberately does not install pyzbar when libzbar is
absent. These tests pin the contract that /api/health reports the missing
capability (with a remediation) instead of failing at import.
"""

import unittest
from unittest.mock import patch

from src.services import media_capabilities as capabilities


class MediaCapabilitiesTests(unittest.TestCase):
    def setUp(self):
        capabilities._checked = False
        capabilities._unavailable_reason = None
        capabilities._remediation = None

    def tearDown(self):
        capabilities._checked = False
        capabilities._unavailable_reason = None
        capabilities._remediation = None

    def test_missing_package_reports_pip_remediation(self):
        with patch.object(capabilities, "find_spec", return_value=None):
            with patch.object(capabilities, "_libzbar_present", return_value=True):
                payload = capabilities.media_capabilities()
        barcode = payload["barcode"]
        self.assertFalse(barcode["available"])
        self.assertEqual(barcode["detail"], "pyzbar is not installed")
        self.assertEqual(
            barcode["remediation"], capabilities.BARCODE_PIP_REMEDIATION
        )

    def test_missing_package_and_library_names_both_remediations(self):
        with patch.object(capabilities, "find_spec", return_value=None):
            with patch.object(capabilities, "_libzbar_present", return_value=False):
                payload = capabilities.media_capabilities()
        barcode = payload["barcode"]
        self.assertFalse(barcode["available"])
        self.assertIn("libzbar0", barcode["detail"])
        self.assertIn("apt-get install -y libzbar0", barcode["remediation"])
        self.assertIn("requirements-barcode.txt", barcode["remediation"])

    def test_missing_libzbar_reports_system_remediation(self):
        with patch.object(capabilities, "find_spec", return_value=object()):
            with patch.object(
                capabilities,
                "_load_pyzbar",
                side_effect=ImportError("Unable to find zbar shared library"),
            ):
                payload = capabilities.media_capabilities()
        barcode = payload["barcode"]
        self.assertFalse(barcode["available"])
        self.assertIn("zbar", barcode["detail"])
        self.assertEqual(barcode["remediation"], capabilities.LIBZBAR_REMEDIATION)

    def test_working_pyzbar_reports_available(self):
        with patch.object(capabilities, "find_spec", return_value=object()):
            with patch.object(capabilities, "_load_pyzbar", return_value=object()):
                payload = capabilities.media_capabilities()
        self.assertTrue(payload["barcode"]["available"])
        self.assertIsNone(payload["barcode"]["remediation"])

    def test_payload_never_raises_and_has_both_capabilities(self):
        with patch.object(capabilities, "find_spec", return_value=None):
            with patch.object(capabilities, "shutil") as shutil:
                shutil.which.return_value = None
                payload = capabilities.media_capabilities()
        self.assertEqual(set(payload), {"barcode", "ocr"})
        self.assertIn("available", payload["ocr"])


if __name__ == "__main__":
    unittest.main()
