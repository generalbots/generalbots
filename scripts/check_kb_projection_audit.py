#!/usr/bin/env python3
"""Offline static audit of the #2499 structured-KB projection.

Source-text checks only: no cargo, no network, no browser. Every assertion is
non-vacuous: a missing struct, function or test makes it FAIL rather than pass.

Usage:
    python3 scripts/check_kb_projection_audit.py [--root /path/to/source-copy]

--root (or KB_PROJECTION_AUDIT_ROOT) exists so a /tmp copy can be mutated to
prove the audit is non-vacuous without touching the working tree.
"""

import argparse
import os
import re
import sys
from pathlib import Path

DEFAULT_ROOT = Path(__file__).resolve().parent.parent

FILES = {
    "chunk": "botserver/crates/botcore/src/kb/structured/chunk.rs",
    "upsert": "botserver/crates/botcore/src/kb/structured/upsert.rs",
    "citation": "botserver/crates/botcore/src/kb/structured/citation.rs",
    "vector_index": "botserver/crates/botcore/src/kb/structured/vector_index.rs",
    "adapter": "botserver/crates/botcore/src/kb/structured/vector_index/adapter.rs",
    "state": "botserver/crates/botdocs/src/state.rs",
    "crud": "botserver/crates/botdocs/src/handlers_api/crud.rs",
}

CITATION_TESTS = [
    ("source_ref_address_includes_page_when_present", "page duplication regression"),
    ("source_ref_address_formats_every_kind_without_page", "address formats"),
    ("source_ref_address_appends_page_for_every_kind", "address formats"),
    ("citation_label_is_locale_neutral", "locale neutrality"),
    ("citation_label_preserves_version_verbatim", "version verbatim"),
]


class Audit:
    def __init__(self):
        self.results = []

    def check(self, name, ok, detail=""):
        self.results.append((name, bool(ok), detail))

    @property
    def passed(self):
        return sum(1 for _, ok, _ in self.results if ok)

    def report(self):
        for name, ok, detail in self.results:
            status = "PASS" if ok else "FAIL"
            suffix = f" — {detail}" if detail else ""
            print(f"{status} {name}{suffix}")
        print(f"{self.passed}/{len(self.results)} assertions passed")
        return 0 if self.passed == len(self.results) else 1


def load_sources(root):
    texts, missing = {}, []
    for key, relative in FILES.items():
        path = root / relative
        try:
            texts[key] = path.read_text(encoding="utf-8")
        except OSError:
            texts[key] = ""
            missing.append(relative)
    return texts, missing


def fn_body(text, signature):
    """Return (body, signature_text) for the first function matching signature."""
    match = re.search(signature, text, re.S)
    if match is None:
        return None, None
    opening = match.end() - 1
    depth = 0
    for index in range(opening, len(text)):
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                return text[opening + 1:index], text[match.start():match.end()]
    return None, text[match.start():match.end()]


def struct_fields(text, name):
    match = re.search(r"pub struct " + name + r"\b.*?\{(.*?)\n\}", text, re.S)
    if match is None:
        return None
    return re.findall(r"^\s*pub\s+([A-Za-z_]\w*)\s*:", match.group(1), re.M)


def audit_acl_order(audit, chunk):
    body, _ = fn_body(chunk, r"pub fn allows\b.*?\{")
    if body is None:
        audit.check("acl_fail_closed_tenant_first", False, "AclMetadata::allows body not found")
        return
    tenant = re.search(
        r"if\s+self\.tenant_id\s*!=\s*Self::tenant_of\(user\)\s*\{\s*return false;\s*\}",
        body,
        re.S,
    )
    public = re.search(r"if\s+self\.is_public\s*\{", body)
    allow_lists = [re.search(r"\b" + field + r"\b", body)
                   for field in ("allowed_users", "allowed_roles", "allowed_groups")]
    ok = (tenant is not None and public is not None and all(allow_lists)
          and tenant.start() < public.start()
          and all(tenant.start() < item.start() for item in allow_lists))
    detail = "tenant denial precedes public branch and every allow-list"
    if not ok:
        detail = "tenant mismatch must deny BEFORE is_public / allow-list branches"
    audit.check("acl_fail_closed_tenant_first", ok, detail)


def audit_payload_shapes(audit, texts):
    fields = struct_fields(texts["upsert"], "UpsertPoint")
    ok = fields is not None and set(fields) == {"id", "vector", "payload"}
    audit.check("upsert_point_no_content_field", ok, f"fields={fields}")

    fields = struct_fields(texts["adapter"], "QdrantPayload")
    expected = {"stable_id", "tenant_id", "document_path", "section", "citation", "acl"}
    ok = (fields is not None and set(fields) == expected
          and "citation" in fields and "acl" in fields and "content" not in fields)
    audit.check("qdrant_payload_citation_acl_no_content", ok, f"fields={fields}")

    fields = struct_fields(texts["vector_index"], "SearchHit")
    ok = (fields is not None and set(fields) == {"id", "source", "section", "score"}
          and "content" not in fields)
    audit.check("search_hit_citation_only", ok, f"fields={fields}")


def audit_search_filter_order(audit, vector_index):
    body, _ = fn_body(vector_index, r"pub fn search\b.*?\{")
    if body is None:
        audit.check("search_tenant_then_acl_before_scoring", False,
                    "InMemoryKbIndex::search body not found")
        return
    tenant = re.search(
        r"organization\.as_deref\(\)\s*==\s*Some\(point\.tenant_id\.as_str\(\)\)", body
    )
    acl = re.search(r"point\.acl\.allows\(user\)", body)
    score = re.search(r"\bcosine\(query_embedding", body)
    ok = (tenant is not None and acl is not None and score is not None
          and body.count(".filter(") >= 2
          and tenant.start() < acl.start() < score.start())
    detail = "tenant filter -> acl.allows(user) filter -> cosine scoring"
    if not ok:
        detail = "search must filter tenant, then allows(user), BEFORE scoring"
    audit.check("search_tenant_then_acl_before_scoring", ok, detail)


def audit_save_hooks(audit, texts):
    handlers = ("handle_docs_save", "handle_create_document", "handle_save_document")
    hooked, bad = [], []
    for name in handlers:
        body, _ = fn_body(texts["crud"], r"pub async fn " + name + r"\b.*?\{")
        if body is None or "project_to_kb" not in body:
            bad.append(f"{name}:no-hook")
            continue
        save = body.find("save_document_to_drive")
        hook = body.find("project_to_kb")
        error = body.find("return Err", save)
        if not (0 <= save < error < hook):
            bad.append(f"{name}:hook-before-save-success")
        else:
            hooked.append(name)
    ok = len(hooked) == 3 and not bad
    detail = "3 successful-save paths: " + ",".join(hooked) if ok else f"bad={bad or hooked}"
    audit.check("project_to_kb_on_three_successful_saves", ok, detail)

    body, signature = fn_body(texts["state"], r"pub fn project_to_kb\b.*?\{")
    if body is None:
        audit.check("project_to_kb_log_and_continue", False, "project_to_kb body not found")
        return
    guard = re.search(r"if\s+let\s+Err\(e\)\s*=\s*\(self\.kb_indexer\)\(points\)", body)
    warn = re.search(r"log::warn!\(", body)
    ok = (signature is not None and "Result" not in signature
          and guard is not None and warn is not None
          and guard.start() < warn.start()
          and "return Err" not in body and "unwrap" not in body)
    detail = "indexer error logged with warn!, save returns unit"
    if not ok:
        detail = "projection failure must be logged and swallowed, not returned"
    audit.check("project_to_kb_log_and_continue", ok, detail)


def audit_stable_ids(audit, texts):
    chunk, upsert = texts["chunk"], texts["upsert"]
    ok = ("pub fn stable_id(" in chunk
          and "id: self.point_id()" in upsert
          and "self.id.clone()" in upsert)
    detail = "stable_id() -> point_id() -> UpsertPoint.id" if ok else "stable id chain broken"
    audit.check("stable_id_flows_to_upsert_point", ok, detail)


def audit_citation_tests(audit, citation):
    for name, category in CITATION_TESTS:
        ok = re.search(r"fn\s+" + name + r"\s*\(", citation) is not None
        audit.check("test_" + name, ok, category)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=os.environ.get("KB_PROJECTION_AUDIT_ROOT"),
                        help="source root override (defaults to the repository root)")
    args = parser.parse_args()
    root = Path(args.root).expanduser().resolve() if args.root else DEFAULT_ROOT

    audit = Audit()
    texts, missing = load_sources(root)
    audit.check("source_files_present", not missing, f"missing={missing}" if missing else "7 files")

    audit_acl_order(audit, texts["chunk"])
    audit_payload_shapes(audit, texts)
    audit_search_filter_order(audit, texts["vector_index"])
    audit_save_hooks(audit, texts)
    audit_stable_ids(audit, texts)
    audit_citation_tests(audit, texts["citation"])

    print(f"root: {root}")
    return audit.report()


if __name__ == "__main__":
    sys.exit(main())
