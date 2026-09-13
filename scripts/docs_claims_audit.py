#!/usr/bin/env python3
"""Check that configuration keys documented in botbook exist in the code.

The book is the user-facing manual; the Rust source is the authority. A page that
documents a setting the code never reads is worse than a page that omits it,
because the reader configures something and nothing happens.

This checks the highest-risk claim class found in practice: a config key written
as a literal in the source. Verification is by string search for the quoted key,
so a key that is only mentioned in a comment still counts as existing -- that is
deliberate, since it keeps the false-positive rate at zero and still catches keys
that were invented outright.

Usage:
    python3 scripts/docs_claims_audit.py            # report, exit 1 on findings
    python3 scripts/docs_claims_audit.py --list     # also list verified keys
"""
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BOOK = os.path.join(ROOT, "botbook/src")
CODE_DIRS = ("botserver", "botlib", "botui")

# Prefixes that mark a genuine configuration key. Restricting the search to these
# keeps file names, CSS classes and prose out of the report.
KEY_PREFIX = (
    "rag", "kb", "bm25", "llm", "embedding", "sandbox", "session", "api",
    "email", "whatsapp", "player", "a2a", "delegate", "upload", "user",
    "extract", "script", "loop", "goto", "package", "download", "attachment",
    "feature", "drive", "history", "tool", "bot", "org", "sms", "telegram",
)

# Keys that are documented as having no effect on purpose. They exist in code
# (an unconnected crate) and the pages say so, so they are not findings.
DOCUMENTED_INERT = {
    "rag-hybrid-enabled", "rag-dense-weight", "rag-sparse-weight",
    "rag-reranker-enabled", "rag-reranker-model", "rag-reranker-top-n",
    "rag-rrf-k", "rag-cache-enabled", "rag-cache-ttl", "bm25-enabled",
    "bm25-k1", "bm25-b", "bm25-stemming", "bm25-stopwords",
}

KEY = re.compile(r"`([a-z][a-z0-9]*(?:-[a-z0-9]+)+)`")


def crate_names():
    """Crate package names, which are hyphenated but are not config keys."""
    names = set()
    crates = os.path.join(ROOT, "botserver/crates")
    if os.path.isdir(crates):
        names.update(os.listdir(crates))
    names.update({"botserver", "botlib", "botui", "botvibe", "botproducts"})
    return names


def book_pages():
    for base, _dirs, files in os.walk(BOOK):
        for name in files:
            if name.endswith(".md"):
                yield os.path.join(base, name)


def exists_in_code(key):
    pattern = f'"{key}"'
    for directory in CODE_DIRS:
        result = subprocess.run(
            ["grep", "-rqF", pattern, "--include=*.rs", "--include=*.js",
             directory],
            cwd=ROOT, capture_output=True,
        )
        if result.returncode == 0:
            return True
    return False


def main():
    show_all = "--list" in sys.argv
    crates = crate_names()
    findings, verified = [], []

    for path in book_pages():
        rel = os.path.relpath(path, ROOT)
        text = open(path, encoding="utf-8", errors="replace").read()
        for key in sorted(set(KEY.findall(text))):
            if not key.startswith(KEY_PREFIX):
                continue
            if key in DOCUMENTED_INERT or key in crates:
                continue
            if exists_in_code(key):
                verified.append((rel, key))
            else:
                findings.append((rel, key))

    if show_all:
        print(f"{len(verified)} documented keys found in the code:")
        for rel, key in verified:
            print(f"  {key:34} {rel}")

    if findings:
        print(f"\n{len(findings)} documented key(s) with no match in the source:")
        for rel, key in findings:
            print(f"  {key:34} {rel}")
        print("\nEither the key does not exist (remove it), or it is read under a "
              "different name (correct the page).")
        return 1
    print(f"all {len(verified)} documented config keys exist in the source")
    return 0


if __name__ == "__main__":
    sys.exit(main())
