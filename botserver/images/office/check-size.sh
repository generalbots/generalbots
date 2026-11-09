#!/bin/sh
# #2062 — size guard for the office workspace image.
#
# The issue caps the office profile at +200 MB over today's base image. Run
# inside each image and compare the two numbers:
#
#   du -sb / | cut -f1     # base
#   du -sb / | cut -f1     # office
#
# or let this script do it when both aliases exist (Incus):
#
#   GB_BASE_IMAGE=images:debian/13 GB_OFFICE_IMAGE=gb-workspace-office \
#     botserver/images/office/check-size.sh
#
# Exits non-zero when the office image exceeds base + 200 MB, so CI can gate a
# Dockerfile change that reintroduces a large stack (LibreOffice, ffmpeg, torch).
set -eu

LIMIT_MB="${GB_VM_SIZE_LIMIT_MB:-200}"

if [ -n "${GB_BASE_IMAGE:-}" ] && [ -n "${GB_OFFICE_IMAGE:-}" ] && command -v incus >/dev/null 2>&1; then
    base=$(incus image info "${GB_BASE_IMAGE}" --format csv 2>/dev/null | awk -F, '{print $5}' | head -1 || true)
    office=$(incus image info "${GB_OFFICE_IMAGE}" --format csv 2>/dev/null | awk -F, '{print $5}' | head -1 || true)
    if [ -n "${base:-}" ] && [ -n "${office:-}" ]; then
        delta_mb=$(( (office - base) / 1000000 ))
        echo "base=${base} bytes office=${office} bytes delta=${delta_mb} MB limit=+${LIMIT_MB} MB"
        if [ "${delta_mb}" -gt "${LIMIT_MB}" ]; then
            echo "office profile exceeds the +${LIMIT_MB} MB budget" >&2
            exit 1
        fi
        echo "office profile within budget"
        exit 0
    fi
fi

# Standalone mode: report the installed size of this filesystem; the caller
# records the base number separately.
size=$(du -sb / 2>/dev/null | cut -f1)
echo "installed_bytes=${size}"
echo "compare against the base image and allow +${LIMIT_MB} MB"
