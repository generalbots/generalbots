"""Auth regression tests for generated artifacts and API docs (#2806).

Generated names are timestamp + 8-hex (guessable), so /outputs must require the
API key just like the generation routes. Docs/OpenAPI leak the full schema and
are disabled in production. The tests build a minimal app from the same pure
helpers the real app uses, so no server or model weights are needed.
"""

import tempfile
import unittest
from pathlib import Path

from fastapi import FastAPI
from fastapi.testclient import TestClient

from src.api.dependencies import is_valid_api_key
from src.core.config import settings
from src.main import AuthenticatedStaticFiles, docs_urls_for


class AuthPredicateTests(unittest.TestCase):
    def test_valid_key_accepted(self):
        self.assertTrue(is_valid_api_key(settings.api_key))

    def test_missing_or_wrong_key_rejected(self):
        self.assertFalse(is_valid_api_key(None))
        self.assertFalse(is_valid_api_key(""))
        self.assertFalse(is_valid_api_key("not-the-key"))

    def test_docs_urls_disabled_in_production(self):
        self.assertEqual(
            docs_urls_for(True),
            {"docs_url": None, "redoc_url": None, "openapi_url": None},
        )

    def test_docs_urls_present_outside_production(self):
        urls = docs_urls_for(False)
        self.assertEqual(urls["docs_url"], "/api/docs")
        self.assertEqual(urls["redoc_url"], "/api/redoc")
        self.assertEqual(urls["openapi_url"], "/openapi.json")


class OutputsAndDocsAuthTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        (root / "images").mkdir()
        (root / "images" / "artifact.png").write_bytes(b"PNG")
        app = FastAPI(**docs_urls_for(True))
        app.mount(
            "/outputs",
            AuthenticatedStaticFiles(directory=str(root)),
            name="outputs",
        )
        self.client = TestClient(app)

    def tearDown(self):
        self.tmp.cleanup()

    def test_outputs_requires_api_key(self):
        denied = self.client.get("/outputs/images/artifact.png")
        self.assertEqual(denied.status_code, 401, denied.text)
        self.assertIn("detail", denied.json())
        self.assertNotEqual(denied.content, b"PNG")

    def test_outputs_serves_with_api_key(self):
        ok = self.client.get(
            "/outputs/images/artifact.png",
            headers={"X-API-Key": settings.api_key},
        )
        self.assertEqual(ok.status_code, 200, ok.text)
        self.assertEqual(ok.content, b"PNG")

    def test_docs_and_openapi_404_in_production(self):
        for path in ("/api/docs", "/api/redoc", "/openapi.json"):
            with self.subTest(path=path):
                self.assertEqual(self.client.get(path).status_code, 404)


if __name__ == "__main__":
    unittest.main()
