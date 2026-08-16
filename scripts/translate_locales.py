#!/usr/bin/env python3
"""Complete or generate FTL locale catalogs from the English reference.

Pipeline:
  1. Parse every `botlib/locales/en/*.ftl` file, preserving comments, blank
     lines and key order.
  2. For each target locale, keep already-translated keys; translate only the
     missing ones via the Google Translate `gtx` endpoint (batched with
     newline joins, `{placeholders}` and URLs protected by sentinels so they
     are never altered).
  3. Write the catalog back in the same structural order as the English file,
     so parity diffs stay reviewable.

Usage:
    python3 scripts/translate_locales.py [--locales es,zh-CN,fr,de,ja,ko]

Re-running is idempotent: existing translations are preserved.
"""

import argparse
import json
import os
import re
import sys
import time
import urllib.parse
import urllib.request

KEY_RE = re.compile(r"^([a-zA-Z0-9_-]+)\s*=\s*(.*)$")
PLACEHOLDER_RE = re.compile(r"(\{[^}]*\})")
URL_RE = re.compile(r"(https?://\S+|[\w.+-]+@[\w.-]+\.[A-Za-z]{2,})")

# Values that must never be translated (brands, product names).
DO_NOT_TRANSLATE = {
    "General Bots",
    "GeneralBots",
    "General Bots AI",
    "WhatsApp",
    "Playwright",
    "MinIO",
    "PostgreSQL",
    "Vault",
    "HTMX",
    "Vibe",
    "GB",
}

BATCH_SIZE = 40
SENTINEL_PREFIX = "⟦"
SENTINEL_SUFFIX = "⟧"


def gtx_translate(lines: list[str], target: str) -> list[str]:
    """Translate a batch of lines via the gtx endpoint, preserving count.

    Retries transient network failures (up to 4 attempts) so a long batch job
    survives a dropped connection instead of aborting mid-catalog.
    """
    url = (
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=en"
        f"&tl={target}&dt=t&q={urllib.parse.quote(chr(10).join(lines))}"
    )
    request = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    last_error: Exception | None = None
    for attempt in range(4):
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                data = json.load(response)
            joined = "".join(segment[0] for segment in data[0] if segment[0])
            return joined.split("\n")
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as exc:
            last_error = exc
            time.sleep(1.5 * (attempt + 1))
    raise RuntimeError(f"gtx translate failed after retries: {last_error}")


def protect(text: str, sentinels: dict[str, str]) -> str:
    """Replace placeholders/URLs with sentinels so translation never alters them."""
    def stash(match: re.Match[str]) -> str:
        token = f"{SENTINEL_PREFIX}{len(sentinels)}{SENTINEL_SUFFIX}"
        sentinels[token] = match.group(1)
        return token

    text = PLACEHOLDER_RE.sub(stash, text)
    text = URL_RE.sub(stash, text)
    return text


def restore(text: str, sentinels: dict[str, str]) -> str:
    for token, original in sentinels.items():
        text = text.replace(token, original)
    return text


def translate_value(value: str, target: str, sentinels: dict[str, str]) -> str:
    stripped = value.strip()
    if not stripped or stripped in DO_NOT_TRANSLATE:
        return value
    # Values that are purely brand tokens / URLs / placeholders stay as-is.
    if re.fullmatch(r"(?:[A-Z][A-Za-z0-9]+|[0-9.]+|https?://\S+|{[^}]*})(?:\s+(?:[A-Z][A-Za-z0-9]+|[0-9.]+|https?://\S+|{[^}]*}))*", stripped):
        return value

    protected = protect(value, sentinels)
    if protected == value and not re.search(r"[A-Za-z]", value):
        return value
    translated = gtx_translate([protected], target)[0]
    return restore(translated, sentinels)


def process_file(en_path: str, out_path: str, target: str, existing: dict[str, str]) -> int:
    with open(en_path, encoding="utf-8") as fh:
        lines = fh.readlines()

    # Collect all keys defined in the English file, in order.
    pending: list[tuple[str, str, int]] = []
    for index, line in enumerate(lines):
        match = KEY_RE.match(line.rstrip("\n"))
        if match:
            key = match.group(1)
            value = match.group(2)
            if key not in existing:
                pending.append((key, value, index))

    if pending:
        print(f"  {os.path.basename(en_path)}: translating {len(pending)} missing keys")
        sentinels: dict[str, str] = {}
        batches = [pending[i : i + BATCH_SIZE] for i in range(0, len(pending), BATCH_SIZE)]
        translations: dict[str, str] = {}
        for batch in batches:
            protected = [protect(value, sentinels) for _, value, _ in batch]
            results = gtx_translate(protected, target)
            for (key, _, _), translated in zip(batch, results):
                translations[key] = restore(translated, sentinels)
            time.sleep(0.2)

    # Rebuild the file: copy comments/blank lines, emit existing or new values.
    output: list[str] = []
    for line in lines:
        match = KEY_RE.match(line.rstrip("\n"))
        if match:
            key = match.group(1)
            if key in existing:
                output.append(f"{key} = {existing[key]}\n")
            elif key in translations:
                output.append(f"{key} = {translations[key]}\n")
            else:
                output.append(line)
        else:
            output.append(line)

    with open(out_path, "w", encoding="utf-8") as fh:
        fh.writelines(output)
    return len(pending)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=".", help="repository root (default: .)")
    parser.add_argument(
        "--locales",
        default="es,zh-CN,fr,de,ja,ko",
        help="comma-separated locales to complete (default: es,zh-CN,fr,de,ja,ko)",
    )
    args = parser.parse_args()

    en_dir = os.path.join(args.root, "botlib", "locales", "en")
    en_files = sorted(f for f in os.listdir(en_dir) if f.endswith(".ftl"))
    if not en_files:
        print("error: no English reference files found", file=sys.stderr)
        return 1

    for locale in [part.strip() for part in args.locales.split(",") if part.strip()]:
        out_dir = os.path.join(args.root, "botlib", "locales", locale)
        os.makedirs(out_dir, exist_ok=True)
        existing: dict[str, str] = {}
        for name in os.listdir(out_dir):
            if not name.endswith(".ftl"):
                continue
            with open(os.path.join(out_dir, name), encoding="utf-8") as fh:
                for line in fh:
                    match = KEY_RE.match(line.rstrip("\n"))
                    if match:
                        existing[match.group(1)] = match.group(2)

        total = 0
        for en_file in en_files:
            out_path = os.path.join(out_dir, en_file)
            total += process_file(os.path.join(en_dir, en_file), out_path, locale, existing)
        print(f"locale '{locale}': {total} keys translated/added")

    return 0


if __name__ == "__main__":
    sys.exit(main())
