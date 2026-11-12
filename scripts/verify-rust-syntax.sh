#!/usr/bin/env bash
# Syntax gate for Rust changes when a local `cargo check` cannot run.
#
# The dev container is capped at 4.5 GiB, so rustc is SIGKILLed while compiling
# the botserver graph — even with CARGO_BUILD_JOBS=1. That cap is an
# environment-specific exception; the project default is 6 jobs
# (.cargo/config.toml, #2243). This script parses every
# changed Rust file with rustfmt (a real parse, not a formatting diff) and
# prints how to get the authoritative type-check from CI.
#
# Usage:
#   scripts/verify-rust-syntax.sh [base-ref]
#   scripts/verify-rust-syntax.sh --self-test   # prove the gate fails closed
#
# Exit codes: 0 all parsed; 1 at least one file failed to parse or git failed.
set -uo pipefail

if [ "${1:-}" = "--self-test" ]; then
    exec python3 "$(dirname "$0")/gate_selftests.py" --gate rust-syntax
fi

base="${1:-origin/main}"

if ! files=$(git diff --name-only --diff-filter=ACM "$base"...HEAD -- '*.rs'); then
    echo "verify-rust-syntax: cannot diff against '$base'" >&2
    exit 1
fi

if [ -z "$files" ]; then
    echo "verify-rust-syntax: no Rust changes against $base"
else
    failed=0
    count=0
    while IFS= read -r file; do
        [ -z "$file" ] && continue
        count=$((count + 1))
        if ! rustfmt --edition 2021 --emit stdout "$file" >/dev/null 2>/tmp/verify_rust_syntax.err; then
            echo "FAIL $file" >&2
            head -5 /tmp/verify_rust_syntax.err >&2
            failed=$((failed + 1))
        fi
    done <<< "$files"
    echo "verify-rust-syntax: $((count - failed))/$count parsed"
    if [ "$failed" -ne 0 ]; then
        exit 1
    fi
fi

cat <<'NOTE'
Type/compile gate (authoritative): push to main or dispatch the release build.

  git tag -f verify-local HEAD
  git push -f origin refs/tags/verify-local
  gh workflow run alm-release.yml -R generalbots/generalbots --ref verify-local
  gh run watch "$(gh run list -R generalbots/generalbots --workflow alm-release.yml --limit 1 --json databaseId --jq '.[0].databaseId')"

Do not retry a local `cargo check -p botserver` under the 4.5 GiB cgroup; it is
killed before it reports a Rust error.
NOTE
