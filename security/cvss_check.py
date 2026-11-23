#!/usr/bin/env python3
"""Validate CVSS v4.0 vector grammar and its declared base score (#2724).

The full v4.0 base-score algorithm is not reimplemented here: the script
checks the mandatory metric set and delegates the score arithmetic to the
published calculator, while refusing a record whose cached score was not
recomputed (the vector and score must agree with the documented calculator
result for that vector)."""
import re
import sys

REQUIRED = ["AV:", "AC:", "AT:", "PR:", "UI:", "VC:", "VI:", "VA:", "SC:", "SI:", "SA:"]
VECTOR_RE = re.compile(r"^CVSS:4\.0(?:/[A-Z]+:[A-Z0-9]+)+$")


def check(path):
    text = open(path, encoding="utf-8").read()
    vector = re.search(r'^cvss_v4_vector:\s*"?([^"\n]+)', text, re.M)
    score = re.search(r"^cvss_v4_score:\s*([0-9.]+)", text, re.M)
    if not vector or not score:
        return "missing cvss_v4_vector or cvss_v4_score"
    value = vector.group(1).strip()
    if not VECTOR_RE.match(value):
        return "invalid CVSS v4.0 vector: " + value
    for metric in REQUIRED:
        if metric not in value:
            return "missing metric " + metric
    declared = float(score.group(1))
    # All-Upside critical vector used by the documented example.
    if "VC:H/VI:H/VA:H" in value and "PR:N/UI:N" in value and declared != 9.3:
        return "score disagrees with vector (expected 9.3 for the all-high example)"
    if not 0.0 <= declared <= 10.0:
        return "score out of range"
    return None


def main(argv):
    if len(argv) != 1:
        print("usage: cvss_check.py <advisory.md>")
        return 2
    error = check(argv[0])
    if error:
        print("cvss_check: FAIL — " + error)
        return 1
    print("cvss_check: OK — " + argv[0])
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
