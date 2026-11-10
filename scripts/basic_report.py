#!/usr/bin/env python3
"""Human-readable report for the BASIC inventory (issue #2079).

Kept separate from `basic_parse.py` so both files stay well under the
450-line limit. Imported by `scripts/gen_basic_reference.py --report`.
"""

from __future__ import annotations


def report(inv: dict) -> None:
    keywords, functions = inv["keywords"], inv["functions"]
    syntax = [k for k in keywords if k["status"] != "directive"]
    print(f"composite keyword registrations: {len(syntax)}")
    print(f"distinct keyword forms: {len({k['name'] for k in syntax})}")
    print(f"registered functions (case-grouped): {len(functions)}")
    print(f"function aliases (lowercase forms): {sum(len(f['aliases']) for f in functions)}")
    print(f"rewrite aliases in convert_multiword_keywords.rs: {len(inv['aliases'])}")
    print(f"compiler directives: {len(inv['directives'])}")
    print(f"Rhai engine keywords: {len(inv['builtins'])}")
    print(f"documented pages that claim an unregistered name: {len(inv['doc_claims'])}")
    for claim in inv["doc_claims"]:
        print(f"  {claim['name']} -> botbook/src/04-basic-scripting/{claim['page']}")
    print(f"names advertised by get_all_keywords(): {len(inv['advertised'])}")
    print(f"advertised but NOT registered: {len(inv['documented_only'])}")
    for name in inv["documented_only"]:
        print(f"  {name} (used by {inv['template_usage'].get(name, 0)} .bas templates)")
    by_category: dict[str, int] = {}
    for k in keywords:
        by_category[k["category"]] = by_category.get(k["category"], 0) + 1
    print("\ncomposite keywords per category:")
    for c in sorted(by_category):
        print(f"  {c}: {by_category[c]}")
    by_crate: dict[str, int] = {}
    for k in keywords:
        by_crate[k["crate"]] = by_crate.get(k["crate"], 0) + 1
    print("\ncomposite keywords per crate:")
    for c in sorted(by_crate):
        print(f"  {c}: {by_crate[c]}")
    first_tokens: dict[str, int] = {}
    for k in keywords:
        first = k["name"].split()[0]
        first_tokens[first] = first_tokens.get(first, 0) + 1
    collisions = {t: n for t, n in first_tokens.items() if n > 1}
    print(f"\nfirst tokens shared by more than one registration: {len(collisions)}")
    print("  (Rhai keys custom syntax by first token; only the last registration wins)")
    for token in sorted(collisions):
        print(f"  {token}: {collisions[token]}")
    print(f"\nblock constructs ($block$): {sum(1 for k in keywords if k['block'])}")
    print(f"keywords with a doc page: {sum(1 for k in keywords if k['docs'])}")
