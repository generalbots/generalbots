#!/usr/bin/env python3
"""Self-contained CVSS v4.0 Base (CVSS-B) vector parser and scorer.

Ported from the FIRST reference implementation (BSD-2-Clause):
  https://github.com/FIRSTdotorg/cvss-v4-calculator
Files: cvss_lookup.js, cvss_score.js, max_severity.js, max_composed.js
Specification: https://www.first.org/cvss/v4.0/specification-document (v1.2)

Only the Base metric group is required for a published advisory. Threat (E)
and Environmental (CR/IR/AR and modified) metrics are accepted so a scored
vector copied from the calculator still validates; absent values use the
specification defaults (E=X -> A, CR/IR/AR=X -> H, modified = base value).

No third-party Python dependencies. No network, no clock.
"""

from __future__ import annotations

import math
import re
import sys

# CVSS v4.0 macro-vector lookup table ("cvss_lookup.js", 270 equivalence
# classes). Compact form: one "key:score" token per entry, whitespace
# separated; parsed once at import.
_LOOKUP_RAW = """
000000:10 000001:9.9 000010:9.8 000011:9.5 000020:9.5 000021:9.2 000100:10 000101:9.6
000110:9.3 000111:8.7 000120:9.1 000121:8.1 000200:9.3 000201:9 000210:8.9 000211:8
000220:8.1 000221:6.8 001000:9.8 001001:9.5 001010:9.5 001011:9.2 001020:9 001021:8.4
001100:9.3 001101:9.2 001110:8.9 001111:8.1 001120:8.1 001121:6.5 001200:8.8 001201:8
001210:7.8 001211:7 001220:6.9 001221:4.8 002001:9.2 002011:8.2 002021:7.2 002101:7.9
002111:6.9 002121:5 002201:6.9 002211:5.5 002221:2.7 010000:9.9 010001:9.7 010010:9.5
010011:9.2 010020:9.2 010021:8.5 010100:9.5 010101:9.1 010110:9 010111:8.3 010120:8.4
010121:7.1 010200:9.2 010201:8.1 010210:8.2 010211:7.1 010220:7.2 010221:5.3 011000:9.5
011001:9.3 011010:9.2 011011:8.5 011020:8.5 011021:7.3 011100:9.2 011101:8.2 011110:8
011111:7.2 011120:7 011121:5.9 011200:8.4 011201:7 011210:7.1 011211:5.2 011220:5
011221:3 012001:8.6 012011:7.5 012021:5.2 012101:7.1 012111:5.2 012121:2.9 012201:6.3
012211:2.9 012221:1.7 100000:9.8 100001:9.5 100010:9.4 100011:8.7 100020:9.1 100021:8.1
100100:9.4 100101:8.9 100110:8.6 100111:7.4 100120:7.7 100121:6.4 100200:8.7 100201:7.5
100210:7.4 100211:6.3 100220:6.3 100221:4.9 101000:9.4 101001:8.9 101010:8.8 101011:7.7
101020:7.6 101021:6.7 101100:8.6 101101:7.6 101110:7.4 101111:5.8 101120:5.9 101121:5
101200:7.2 101201:5.7 101210:5.7 101211:5.2 101220:5.2 101221:2.5 102001:8.3 102011:7
102021:5.4 102101:6.5 102111:5.8 102121:2.6 102201:5.3 102211:2.1 102221:1.3 110000:9.5
110001:9 110010:8.8 110011:7.6 110020:7.6 110021:7 110100:9 110101:7.7 110110:7.5
110111:6.2 110120:6.1 110121:5.3 110200:7.7 110201:6.6 110210:6.8 110211:5.9 110220:5.2
110221:3 111000:8.9 111001:7.8 111010:7.6 111011:6.7 111020:6.2 111021:5.8 111100:7.4
111101:5.9 111110:5.7 111111:5.7 111120:4.7 111121:2.3 111200:6.1 111201:5.2 111210:5.7
111211:2.9 111220:2.4 111221:1.6 112001:7.1 112011:5.9 112021:3 112101:5.8 112111:2.6
112121:1.5 112201:2.3 112211:1.3 112221:0.6 200000:9.3 200001:8.7 200010:8.6 200011:7.2
200020:7.5 200021:5.8 200100:8.6 200101:7.4 200110:7.4 200111:6.1 200120:5.6 200121:3.4
200200:7 200201:5.4 200210:5.2 200211:4 200220:4 200221:2.2 201000:8.5 201001:7.5
201010:7.4 201011:5.5 201020:6.2 201021:5.1 201100:7.2 201101:5.7 201110:5.5 201111:4.1
201120:4.6 201121:1.9 201200:5.3 201201:3.6 201210:3.4 201211:1.9 201220:1.9 201221:0.8
202001:6.4 202011:5.1 202021:2 202101:4.7 202111:2.1 202121:1.1 202201:2.4 202211:0.9
202221:0.4 210000:8.8 210001:7.5 210010:7.3 210011:5.3 210020:6 210021:5 210100:7.3
210101:5.5 210110:5.9 210111:4 210120:4.1 210121:2 210200:5.4 210201:4.3 210210:4.5
210211:2.2 210220:2 210221:1.1 211000:7.5 211001:5.5 211010:5.8 211011:4.5 211020:4
211021:2.1 211100:6.1 211101:5.1 211110:4.8 211111:1.8 211120:2 211121:0.9 211200:4.6
211201:1.8 211210:1.7 211211:0.7 211220:0.8 211221:0.2 212001:5.3 212011:2.4 212021:1.4
212101:2.4 212111:1.2 212121:0.5 212201:1 212211:0.3 212221:0.1
"""
LOOKUP: dict[str, float] = {
    key: float(value)
    for token in _LOOKUP_RAW.split()
    for key, value in [token.split(":", 1)]
}

# Maximum severity distances (max_severity.js, values already +1).
MAX_SEVERITY = {
    "eq1": {0: 1, 1: 4, 2: 5},
    "eq2": {0: 1, 1: 2},
    "eq3eq6": {0: {0: 7, 1: 6}, 1: {0: 8, 1: 8}, 2: {1: 10}},
    "eq4": {0: 6, 1: 5, 2: 4},
}

# Highest-severity vectors of each macro-vector (max_composed.js).
MAX_COMPOSED = {
    "eq1": {
        0: ["AV:N/PR:N/UI:N/"],
        1: ["AV:A/PR:N/UI:N/", "AV:N/PR:L/UI:N/", "AV:N/PR:N/UI:P/"],
        2: ["AV:P/PR:N/UI:N/", "AV:A/PR:L/UI:P/"],
    },
    "eq2": {0: ["AC:L/AT:N/"], 1: ["AC:H/AT:N/", "AC:L/AT:P/"]},
    "eq3": {
        0: {
            "0": ["VC:H/VI:H/VA:H/CR:H/IR:H/AR:H/"],
            "1": ["VC:H/VI:H/VA:L/CR:M/IR:M/AR:H/", "VC:H/VI:H/VA:H/CR:M/IR:M/AR:M/"],
        },
        1: {
            "0": ["VC:L/VI:H/VA:H/CR:H/IR:H/AR:H/", "VC:H/VI:L/VA:H/CR:H/IR:H/AR:H/"],
            "1": [
                "VC:L/VI:H/VA:L/CR:H/IR:M/AR:H/",
                "VC:L/VI:H/VA:H/CR:H/IR:M/AR:M/",
                "VC:H/VI:L/VA:H/CR:M/IR:H/AR:M/",
                "VC:H/VI:L/VA:L/CR:M/IR:H/AR:H/",
                "VC:L/VI:L/VA:H/CR:H/IR:H/AR:M/",
            ],
        },
        2: {"1": ["VC:L/VI:L/VA:L/CR:H/IR:H/AR:H/"]},
    },
    "eq4": {
        0: ["SC:H/SI:S/SA:S/"],
        1: ["SC:H/SI:H/SA:H/"],
        2: ["SC:L/SI:L/SA:L/"],
    },
    "eq5": {0: ["E:A/"], 1: ["E:P/"], 2: ["E:U/"]},
}

# Metric abbreviations -> allowed values (Base + optional Threat/Environmental).
METRICS: dict[str, str] = {
    "AV": "NALP", "AC": "LH", "AT": "NP", "PR": "NLH", "UI": "NPA",
    "VC": "HLN", "VI": "HLN", "VA": "HLN",
    "SC": "HLN", "SI": "HLN", "SA": "HLN",
    "CR": "HMLX", "IR": "HMLX", "AR": "HMLX", "E": "APUX",
    "MAV": "NALPX", "MAC": "LHX", "MAT": "NPX", "MPR": "NLHX",
    "MUI": "NPAX", "MVC": "HLNX", "MVI": "HLNX", "MVA": "HLNX",
    "MSC": "HLNX", "MSI": "SHLNX", "MSA": "SHLNX",
    "S": "PNX", "AU": "NYX", "R": "AILX", "V": "DCX", "RE": "LMHX", "U": "ClearAmberGreenX",
}
BASE_METRICS = ("AV", "AC", "AT", "PR", "UI", "VC", "VI", "VA", "SC", "SI", "SA")

_LEVELS = {
    "AV": {"N": 0.0, "A": 0.1, "L": 0.2, "P": 0.3},
    "PR": {"N": 0.0, "L": 0.1, "H": 0.2},
    "UI": {"N": 0.0, "P": 0.1, "A": 0.2},
    "AC": {"L": 0.0, "H": 0.1},
    "AT": {"N": 0.0, "P": 0.1},
    "VC": {"H": 0.0, "L": 0.1, "N": 0.2},
    "VI": {"H": 0.0, "L": 0.1, "N": 0.2},
    "VA": {"H": 0.0, "L": 0.1, "N": 0.2},
    "SC": {"H": 0.1, "L": 0.2, "N": 0.3},
    "SI": {"S": 0.0, "H": 0.1, "L": 0.2, "N": 0.3},
    "SA": {"S": 0.0, "H": 0.1, "L": 0.2, "N": 0.3},
    "CR": {"H": 0.0, "M": 0.1, "L": 0.2},
    "IR": {"H": 0.0, "M": 0.1, "L": 0.2},
    "AR": {"H": 0.0, "M": 0.1, "L": 0.2},
}

_VECTOR_RE = re.compile(r"^[A-Za-z]+:[A-Za-z]+(?:/[A-Za-z]+:[A-Za-z]+)*$")


def parse_vector(vector: str) -> dict[str, str]:
    """Parse a CVSS v4.0 vector string into {metric: value}.

    Raises ValueError with a specific reason for malformed or incomplete
    vectors. Duplicate metrics and unknown abbreviations are rejected.
    """
    if not isinstance(vector, str) or not vector.startswith("CVSS:4.0/"):
        raise ValueError("vector must start with 'CVSS:4.0/'")
    body = vector[len("CVSS:4.0/"):]
    if not _VECTOR_RE.match(body):
        raise ValueError("vector has malformed syntax")
    metrics: dict[str, str] = {}
    for part in body.split("/"):
        name, value = part.split(":", 1)
        if name not in METRICS:
            raise ValueError(f"unknown metric '{name}'")
        if name in metrics:
            raise ValueError(f"metric '{name}' defined more than once")
        if value not in METRICS[name]:
            raise ValueError(f"invalid value '{value}' for metric '{name}'")
        metrics[name] = value
    for name in BASE_METRICS:
        if name not in metrics:
            raise ValueError(f"mandatory base metric '{name}' is missing")
    # The reference calculator always carries every metric with an explicit
    # "X" (Not Defined). Absent optional metrics behave the same way.
    for name in METRICS:
        metrics.setdefault(name, "X")
    return metrics


def _m(selected: dict[str, str], metric: str) -> str | None:
    """Apply CVSS 'm()' defaults: E=X->A, CR/IR/AR=X->H, modified=base."""
    value = selected.get(metric)
    if metric == "E" and value == "X":
        return "A"
    if metric in ("CR", "IR", "AR") and value == "X":
        return "H"
    modified = selected.get("M" + metric)
    if modified is not None and modified != "X":
        return modified
    return value


def _macro_vector(selected: dict[str, str]) -> str:
    """Compute the 6-digit macro-vector key (eq1..eq6)."""
    av, pr, ui = _m(selected, "AV"), _m(selected, "PR"), _m(selected, "UI")
    ac, at = _m(selected, "AC"), _m(selected, "AT")
    vc, vi, va = _m(selected, "VC"), _m(selected, "VI"), _m(selected, "VA")
    sc, si, sa = _m(selected, "SC"), _m(selected, "SI"), _m(selected, "SA")
    cr, ir, ar = _m(selected, "CR"), _m(selected, "IR"), _m(selected, "AR")
    e = _m(selected, "E")
    msi, msa = selected.get("MSI"), selected.get("MSA")

    if av == "N" and pr == "N" and ui == "N":
        eq1 = 0
    elif (av == "N" or pr == "N" or ui == "N") and av != "P":
        eq1 = 1
    else:
        eq1 = 2

    eq2 = 0 if (ac == "L" and at == "N") else 1

    if vc == "H" and vi == "H":
        eq3 = 0
    elif vc == "H" or vi == "H" or va == "H":
        eq3 = 1
    else:
        eq3 = 2

    if msi == "S" or msa == "S":
        eq4 = 0
    elif sc == "H" or si == "H" or sa == "H":
        eq4 = 1
    else:
        eq4 = 2

    eq5 = { "A": 0, "P": 1, "U": 2 }.get(e, 0)

    eq6 = 0 if ((cr == "H" and vc == "H") or (ir == "H" and vi == "H")
                or (ar == "H" and va == "H")) else 1

    return f"{eq1}{eq2}{eq3}{eq4}{eq5}{eq6}"


def _extract(metric: str, vector: str) -> str:
    """Extract one metric's value from a highest-severity vector string."""
    marker = metric + ":"
    idx = vector.find(marker)
    if idx < 0:
        return ""
    rest = vector[idx + len(marker):]
    slash = rest.find("/")
    return rest[:slash] if slash > 0 else rest


def _round_half_up(value: float) -> float:
    """JS Math.round semantics (round half away from zero), one decimal."""
    return math.floor(value * 10 + 0.5 + 1e-9) / 10


def base_score(vector: str) -> float:
    """Compute the CVSS v4.0 base score (one decimal, 0.0..10.0)."""
    selected = parse_vector(vector)
    if all(_m(selected, m) == "N" for m in ("VC", "VI", "VA", "SC", "SI", "SA")):
        return 0.0

    mv = _macro_vector(selected)
    value = LOOKUP.get(mv)
    if value is None:
        raise ValueError(f"no CVSS v4.0 lookup entry for macro-vector {mv}")

    eq1, eq2, eq3, eq4, eq5, eq6 = (int(c) for c in mv)
    key = lambda a, b, c, d, e, f: f"{a}{b}{c}{d}{e}{f}"
    next_eq1 = key(eq1 + 1, eq2, eq3, eq4, eq5, eq6)
    next_eq2 = key(eq1, eq2 + 1, eq3, eq4, eq5, eq6)
    next_eq4 = key(eq1, eq2, eq3, eq4 + 1, eq5, eq6)
    next_eq5 = key(eq1, eq2, eq3, eq4, eq5 + 1, eq6)

    if (eq3, eq6) == (1, 1) or (eq3, eq6) == (0, 1):
        next_eq3eq6 = key(eq1, eq2, eq3 + 1, eq4, eq5, eq6)
    elif (eq3, eq6) == (1, 0):
        next_eq3eq6 = key(eq1, eq2, eq3, eq4, eq5, eq6 + 1)
    elif (eq3, eq6) == (0, 0):
        left = LOOKUP.get(key(eq1, eq2, eq3, eq4, eq5, eq6 + 1))
        right = LOOKUP.get(key(eq1, eq2, eq3 + 1, eq4, eq5, eq6))
        # JS picks the higher defined lower macro; mirror its comparison.
        next_eq3eq6 = right if left is None or (right is not None and right > left) else left
    else:
        next_eq3eq6 = key(eq1, eq2, eq3 + 1, eq4, eq5, eq6 + 1)

    lower = {
        "eq1": LOOKUP.get(next_eq1),
        "eq2": LOOKUP.get(next_eq2),
        "eq3eq6": LOOKUP.get(next_eq3eq6) if isinstance(next_eq3eq6, str) else next_eq3eq6,
        "eq4": LOOKUP.get(next_eq4),
        "eq5": LOOKUP.get(next_eq5),
    }

    eq3_maxes = MAX_COMPOSED["eq3"][eq3].get(str(eq6), [])
    max_vectors = [
        a + b + c + d + e
        for a in MAX_COMPOSED["eq1"][eq1]
        for b in MAX_COMPOSED["eq2"][eq2]
        for c in eq3_maxes
        for d in MAX_COMPOSED["eq4"][eq4]
        for e in MAX_COMPOSED["eq5"][eq5]
    ]

    distance_groups = {
        "eq1": ("AV", "PR", "UI"),
        "eq2": ("AC", "AT"),
        "eq3eq6": ("VC", "VI", "VA", "CR", "IR", "AR"),
        "eq4": ("SC", "SI", "SA"),
    }
    distances = {name: 0.0 for name in (*distance_groups, "eq5")}
    for max_vector in max_vectors:
        candidate = {
            name: sum(_LEVELS[metric][_m(selected, metric)] - _LEVELS[metric][_extract(metric, max_vector)]
                      for metric in metrics)
            for name, metrics in distance_groups.items()
        }
        if any(value < 0 for value in candidate.values()):
            continue
        distances = {**candidate, "eq5": 0.0}
        break

    max_sev = {
        "eq1": MAX_SEVERITY["eq1"][eq1],
        "eq2": MAX_SEVERITY["eq2"][eq2],
        "eq3eq6": MAX_SEVERITY["eq3eq6"][eq3][eq6],
        "eq4": MAX_SEVERITY["eq4"][eq4],
    }
    normalized = []
    for name in ("eq1", "eq2", "eq3eq6", "eq4", "eq5"):
        if lower[name] is None:
            continue
        depth = max_sev.get(name, 1)
        normalized.append((value - lower[name]) * (distances[name] / (depth * 0.1)))
    mean_distance = sum(normalized) / len(normalized) if normalized else 0.0

    score = value - mean_distance
    score = min(10.0, max(0.0, score))
    return _round_half_up(score)


SEVERITY_RANGES = (
    ("none", 0.0, 0.0),
    ("low", 0.1, 3.9),
    ("medium", 4.0, 6.9),
    ("high", 7.0, 8.9),
    ("critical", 9.0, 10.0),
)


def severity_for(score: float) -> str:
    """CVSS v4.0 qualitative severity rating for a base score."""
    for name, low, high in SEVERITY_RANGES:
        if low <= score <= high:
            return name
    raise ValueError(f"score {score} outside 0.0..10.0")


# Cross-checked against the FIRST reference implementation (not the prose in
# the examples document: two of its printed scores are typos — the vectors
# below scored by the calculator are 9.3 and 9.2, not the printed 9.4/8.2).
_REFERENCE_VECTORS = (
    ("CVSS:4.0/AV:L/AC:L/AT:P/PR:L/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N", 7.3),
    ("CVSS:4.0/AV:N/AC:L/AT:P/PR:N/UI:P/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N", 7.7),
    ("CVSS:4.0/AV:N/AC:L/AT:P/PR:N/UI:P/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N/E:U", 5.2),
    ("CVSS:4.0/AV:N/AC:L/AT:P/PR:N/UI:N/VC:H/VI:L/VA:L/SC:N/SI:N/SA:N", 8.3),
    ("CVSS:4.0/AV:L/AC:L/AT:N/PR:N/UI:A/VC:L/VI:N/VA:N/SC:N/SI:N/SA:N", 4.6),
    ("CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:A/VC:N/VI:N/VA:N/SC:L/SI:L/SA:N", 5.1),
    ("CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:N/VI:N/VA:N/SC:L/SI:L/SA:N", 6.9),
    ("CVSS:4.0/AV:L/AC:L/AT:N/PR:H/UI:N/VC:N/VI:N/VA:N/SC:H/SI:N/SA:N", 5.9),
    ("CVSS:4.0/AV:L/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H/SC:H/SI:H/SA:H", 9.4),
    ("CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:N/VI:H/VA:N/SC:H/SI:H/SA:H", 9.3),
    ("CVSS:4.0/AV:P/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H/SC:N/SI:H/SA:N", 8.3),
    ("CVSS:4.0/AV:N/AC:L/AT:N/PR:H/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N/E:A", 8.6),
    ("CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:H/VI:N/VA:N/SC:N/SI:N/SA:N/E:A", 8.7),
    ("CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N/E:A", 9.3),
    ("CVSS:4.0/AV:N/AC:H/AT:P/PR:N/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N", 9.2),
    ("CVSS:4.0/AV:A/AC:L/AT:N/PR:N/UI:N/VC:N/VI:L/VA:N/SC:H/SI:N/SA:H", 6.4),
)


def self_test() -> int:
    failures = 0
    for vector, expected in _REFERENCE_VECTORS:
        actual = base_score(vector)
        if abs(actual - expected) > 1e-9:
            failures += 1
            print(f"FAIL {vector}: expected {expected}, got {actual}")
    for vector, reason in (
        ("CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H", "CVSS:4.0"),
        ("CVSS:4.0/AV:F/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N", "invalid value"),
        ("CVSS:4.0/AV:N/AC:L/PR:N/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N", "mandatory base"),
        ("CVSS:4.0/AV:N/AV:N/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N", "more than once"),
    ):
        try:
            parse_vector(vector)
        except ValueError as error:
            if reason not in str(error):
                failures += 1
                print(f"FAIL {vector}: expected rejection about '{reason}', got {error}")
        else:
            failures += 1
            print(f"FAIL {vector}: invalid vector was accepted")
    if base_score("CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:N/VI:N/VA:N/SC:N/SI:N/SA:N") != 0.0:
        failures += 1
        print("FAIL no-impact vector must score 0.0")
    if failures:
        print(f"self-test FAILED: {failures} case(s)")
        return 1
    print(f"self-test ok: {len(_REFERENCE_VECTORS)} reference vectors + 5 rejection cases")
    return 0


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("vector", nargs="?", help="print the CVSS v4.0 base score for a vector")
    args = parser.parse_args()
    if args.self_test:
        raise SystemExit(self_test())
    if args.vector:
        try:
            score = base_score(args.vector)
        except ValueError as error:
            print(f"error: {error}", file=sys.stderr)
            raise SystemExit(1)
        print(f"{score:.1f} {severity_for(score)}")
    else:
        parser.print_help()
