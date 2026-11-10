#!/usr/bin/env python3
"""Inventory of the registered BASIC surface (issue #2079).

Parses the actual registrations, never a hand-written list:

- `register_custom_syntax([...])` in `botserver/crates/botbasic_*/src/**` and
  `botserver/src/basic/keywords/**` — composite keyword forms;
- `register_fn("NAME", ...)` — function names (grouped case-insensitively);
- `convert_multiword_keywords.rs` — the alias/rewrite table;
- `get_all_keywords()` — names the runtime advertises.

A name that appears only in `get_all_keywords()` is *not* implemented; it is
reported as `documented-only` so the reference can never claim otherwise.
Standard library only; imported by `scripts/gen_basic_reference.py`.
"""

from __future__ import annotations

import re
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
BOTSERVER = REPO / "botserver"
CRATES = BOTSERVER / "crates"
LOCAL_KEYWORDS = BOTSERVER / "src" / "basic" / "keywords"
KEYWORDS_MOD = LOCAL_KEYWORDS / "mod.rs"
ALIASES = CRATES / "botbasic_compiler" / "src" / "syntax_transforms" / "convert_multiword_keywords.rs"
DOCS = REPO / "botbook" / "src" / "04-basic-scripting"
TEMPLATES = REPO / "bottemplates"

# (path fragment, category) — first match wins, checked in this order.
CATEGORY_RULES = [
    ("http_operations", "http"), ("graphql", "http"), ("web_data", "http"),
    ("/http", "http"), ("api_tool", "http"), ("rest", "http"),
    ("file_ops", "files"), ("document_text", "files"), ("drive", "files"),
    ("pdf", "files"), ("office", "files"), ("shortcut", "files"),
    ("use_kb", "kb"), ("think_kb", "kb"), ("clear_kb", "kb"),
    ("kb_statistics", "kb"), ("knowledge", "kb"), ("episodic_memory", "kb"),
    ("enhanced_memory", "kb"), ("remember", "kb"), ("/search", "kb"),
    ("save_from_unstructured", "kb"),
    ("multimodal", "media"), ("video", "media"), ("qrcode", "media"),
    ("face_api", "media"), ("speech", "media"), ("image", "media"),
    ("social", "social"), ("sms", "social"), ("send_mail", "social"),
    ("send_template", "social"), ("universal_messaging", "social"),
    ("messaging", "social"),
    ("agent_reflection", "agent"), ("orchestration", "agent"),
    ("workflow", "agent"), ("mcp", "agent"), ("delegate", "agent"),
    ("human_approval", "agent"), ("model_routing", "agent"),
    ("llm", "agent"), ("chatbot", "agent"),
    ("banking", "industry"), ("kyc", "industry"), ("inventory", "industry"),
    ("m365", "industry"), ("products", "industry"), ("crm", "industry"),
    ("lead_scoring", "industry"), ("attendant", "industry"),
    ("for_next", "control"), ("switch_case", "control"),
    ("procedures", "control"), ("errors", "control"), ("wait", "control"),
    ("dag", "control"),
    ("talk", "dialog"), ("hear", "dialog"), ("print", "dialog"),
    ("set_context", "dialog"), ("set_user", "dialog"),
]
CRATE_CATEGORY = {
    "core": "language", "data": "data", "comms": "messaging", "system": "system",
    "ai": "ai", "types": "types", "compiler": "compiler",
}
FN_RE = re.compile(r"register_fn\(\s*\"([^\"]+)\"")
PLACEHOLDER = re.compile(r"^\$[a-z_]+\$$")


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def source_files() -> list[Path]:
    files: list[Path] = []
    for crate in sorted(CRATES.glob("botbasic_*")):
        for path in sorted((crate / "src").rglob("*.rs")):
            name = path.name
            if name.endswith("_tests.rs") or name == "tests.rs":
                continue
            files.append(path)
    for path in sorted(LOCAL_KEYWORDS.rglob("*.rs")):
        if not path.name.endswith("_tests.rs"):
            files.append(path)
    return files


def categorize(path: Path) -> str:
    posix = path.as_posix()
    for fragment, category in CATEGORY_RULES:
        if fragment in posix:
            return category
    for part in path.parts:
        if part.startswith("botbasic_"):
            return CRATE_CATEGORY.get(part.removeprefix("botbasic_"), "system")
    return "botserver" if "botserver/src/basic" in posix else "system"


def module_doc(text: str) -> str:
    lines = []
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("//!"):
            lines.append(stripped[3:].strip())
        elif stripped and lines:
            break
        elif stripped and not lines and not stripped.startswith("#!"):
            break
    return " ".join(lines)


def enclosing_doc(lines: list[str], idx: int) -> str:
    """Doc comment of the function that owns the registration at `idx`."""
    decl = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+\w+")
    for i in range(idx, -1, -1):
        if decl.match(lines[i]):
            docs = []
            j = i - 1
            while j >= 0 and (lines[j].strip().startswith("///") or lines[j].strip().startswith("#[")):
                if lines[j].strip().startswith("///"):
                    docs.insert(0, lines[j].strip()[3:].strip())
                j -= 1
            return " ".join(docs)
    return ""


def first_sentence(text: str) -> str:
    text = " ".join(text.split())
    if not text:
        return ""
    cut = re.search(r"[a-z0-9)]\.\s", text)
    return text[:cut.start() + 2].strip() if cut else text


def array_strings(text: str, start: int) -> list[str]:
    """Quoted items of the array literal that starts at/after `start`."""
    open_idx = text.find("[", start)
    if open_idx < 0:
        return []
    depth, i, items, current = 0, open_idx, [], []
    while i < len(text):
        ch = text[i]
        if ch == '"':
            j = i + 1
            while j < len(text) and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            if depth == 1:
                items.append(text[i + 1:j])
            i = j + 1
            continue
        if ch == "[":
            depth += 1
        elif ch == "]":
            depth -= 1
            if depth == 0:
                break
        i += 1
    del current
    return items


def form_name(parts: list[str]) -> tuple[str, str]:
    """(keyword name, signature) from the registration array parts."""
    literals, signature = [], []
    for part in parts:
        if PLACEHOLDER.match(part):
            signature.append(part)
            continue
        words = part.split()
        literals.extend(w.upper() for w in words if re.search(r"[A-Za-z0-9]", w))
        signature.extend(words)
    return " ".join(literals), " ".join(signature) or " ".join(literals)


def slug(name: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")


def doc_page(name: str, existing: set[str]) -> str:
    for candidate in (f"keyword-{slug(name)}.md",):
        if candidate in existing:
            return f"../{candidate}"
    return ""


def parse_registrations() -> list[dict]:
    existing = {p.name for p in DOCS.glob("keyword-*.md")}
    found: list[dict] = []
    for path in source_files():
        text = read(path)
        lines = text.splitlines()
        mod_doc = module_doc(text)
        for m in re.finditer(r"register_custom_syntax(?:_with_state_raw)?\s*\(", text):
            parts = array_strings(text, m.end())
            if not parts:
                continue
            name, signature = form_name(parts)
            if not name:
                continue
            idx = text[:m.start()].count("\n")
            purpose = first_sentence(enclosing_doc(lines, idx) or mod_doc)
            found.append({
                "name": name, "signature": signature, "parts": parts,
                "crate": next((p.removeprefix("botbasic_") for p in path.parts if p.startswith("botbasic_")), "botserver"),
                "category": categorize(path),
                "source": path.relative_to(REPO).as_posix(),
                "line": idx + 1, "purpose": purpose,
                "docs": doc_page(name, existing),
                "block": "$block$" in parts,
            })
    return found


def parse_functions() -> list[dict]:
    existing = {p.name for p in DOCS.glob("keyword-*.md")}
    grouped: dict[str, dict] = {}
    for path in source_files():
        text = read(path)
        lines = text.splitlines()
        mod_doc = module_doc(text)
        for m in FN_RE.finditer(text):
            raw = m.group(1)
            canonical = raw.upper()
            idx = text[:m.start()].count("\n")
            entry = grouped.setdefault(canonical, {
                "name": canonical, "aliases": set(), "sources": set(), "purpose": "",
                "category": categorize(path), "docs": doc_page(raw, existing),
            })
            if raw != canonical:
                entry["aliases"].add(raw)
            entry["sources"].add(path.relative_to(REPO).as_posix())
            if not entry["purpose"]:
                entry["purpose"] = first_sentence(enclosing_doc(lines, idx) or mod_doc)
    out = []
    for name in sorted(grouped):
        entry = grouped[name]
        entry["aliases"] = sorted(entry["aliases"])
        entry["sources"] = sorted(entry["sources"])
        out.append(entry)
    return out


def parse_aliases() -> list[dict]:
    """Rewrite table entries: pattern -> function name, arity and argument names."""
    if not ALIASES.is_file():
        return []
    text = read(ALIASES)
    out = []
    pattern = re.compile(
        r'\(r#"(?P<pat>[^"]+)"#\s*,\s*(?P<min>\d+)\s*,\s*(?P<max>\d+)\s*,\s*vec!\[(?P<args>[^\]]*)\]')
    for m in pattern.finditer(text):
        pat = m.group("pat")
        words = [w for w in re.split(r"\\s\+", pat) if w]
        if not words or not re.match(r"^[A-Za-z ]+$", " ".join(words)):
            continue
        name = " ".join(w.upper() for w in words)
        args = [a.strip().strip('"') for a in m.group("args").split(",") if a.strip()]
        target = name.lower().replace(" ", "_")
        out.append({"name": name, "function": target, "min": int(m.group("min")),
                    "max": int(m.group("max")), "args": args})
    return sorted(out, key=lambda a: a["name"])


def parse_advertised() -> list[str]:
    if not KEYWORDS_MOD.is_file():
        return []
    text = read(KEYWORDS_MOD)
    m = re.search(r"fn get_all_keywords\(\)[^{]*\{", text)
    if not m:
        return []
    end = text.find("\n}", m.end())
    body = text[m.end():end if end > 0 else len(text)]
    return sorted({" ".join(s.split()) for s in re.findall(r'"([^"]+)"', body)})


def parse_builtins() -> list[str]:
    """Rhai engine keywords: provided by the engine, not registered by us."""
    path = CRATES / "botbasic_compiler" / "src" / "syntax_transforms" / "convert_while_wend_syntax.rs"
    if not path.is_file():
        return []
    text = read(path)
    m = re.search(r"rhai_builtins\s*=\s*\[(.*?)\];", text, re.S)
    if not m:
        return []
    return sorted(set(re.findall(r'"([^"]+)"', m.group(1))))


def parse_directives() -> list[dict]:
    """Compiler directives: consumed by preprocess_basic.rs, not the Rhai engine."""
    out = []
    for name in ("directive_conflicts.rs", "basic_errors.rs"):
        path = CRATES / "botbasic_compiler" / "src" / name
        if not path.is_file():
            continue
        for m in re.finditer(r'Directive::\w+\s*=>\s*"([^"]+)"', read(path)):
            out.append({"name": m.group(1),
                        "source": path.relative_to(REPO).as_posix()})
    unique: dict[str, dict] = {}
    for entry in out:
        unique.setdefault(entry["name"], entry)
    return [unique[name] for name in sorted(unique)]


def template_usage(names: set[str]) -> dict[str, int]:
    counts: dict[str, int] = {}
    if not TEMPLATES.is_dir():
        return counts
    for path in sorted(TEMPLATES.rglob("*.bas")):
        body = read(path).upper()
        for name in names:
            if name in body:
                counts[name] = counts.get(name, 0) + 1
    return counts


def form_prefixes(parts: list[str]) -> set[str]:
    """Literal prefixes of a registration array (`FOR EACH $x$ IN $y$` -> FOR, FOR EACH, ...)."""
    out, words = set(), []
    for part in parts:
        if PLACEHOLDER.match(part):
            if words:
                out.add(" ".join(words))
            continue
        words.extend(w.upper() for w in part.split())
        out.add(" ".join(words))
    return out


def signature_matches(signature: str, words: list[str]) -> bool:
    """Does `signature` accept the literal `words` (placeholders absorb words)?"""
    tokens = signature.split()
    memo: dict[tuple[int, int], bool] = {}

    def walk(i: int, j: int) -> bool:
        key = (i, j)
        if key in memo:
            return memo[key]
        if i == len(tokens):
            result = j == len(words)
        elif PLACEHOLDER.match(tokens[i]):
            result = any(walk(i + 1, k) for k in range(j + 1, len(words) + 1))
        else:
            result = j < len(words) and tokens[i].upper() == words[j].upper() and walk(i + 1, j + 1)
        memo[key] = result
        return result

    return walk(0, 0)


def build_inventory() -> dict:
    keywords = parse_registrations()
    functions = parse_functions()
    aliases = parse_aliases()
    implemented: set[str] = set()

    def add(name: str) -> None:
        for form in (name, name.replace("_", " "), name.replace(" ", "_")):
            implemented.add(" ".join(form.upper().split()))

    for entry in keywords:
        add(entry["name"])
        for prefix in form_prefixes(entry["parts"]):
            add(prefix)
    for entry in functions:
        add(entry["name"])
        for alias in entry["aliases"]:
            add(alias)
    for entry in aliases:
        add(entry["name"])
        add(entry["function"])
    signatures = [k["signature"] for k in keywords]
    signatures += [f["name"].replace("_", " ") for f in functions]
    advertised = parse_advertised()
    missing = []
    for name in advertised:
        normalized = " ".join(name.replace("_", " ").upper().split())
        if normalized in implemented:
            continue
        if any(signature_matches(sig, normalized.split()) for sig in signatures):
            continue
        missing.append(name)
    usage = template_usage(set(missing))
    for entry in keywords:
        entry["status"] = "registered"
    alias_targets = {a["function"] for a in aliases}
    for entry in functions:
        entry["status"] = "alias-of-rewrite" if entry["name"].lower() in alias_targets else "registered"
    directives = parse_directives()
    existing = {p.name for p in DOCS.glob("keyword-*.md")}
    for directive in directives:
        add(directive["name"])
        keywords.append({
            "name": directive["name"], "signature": directive["name"],
            "parts": [directive["name"]], "crate": "botbasic_compiler",
            "category": "control", "source": directive["source"], "line": 0,
            "purpose": "Compiler directive: consumed by `preprocess_basic.rs` before "
                       "evaluation, not a runtime syntax.",
            "docs": f"../keyword-{slug(directive['name'])}.md"
                    if f"keyword-{slug(directive['name'])}.md" in existing else "",
            "block": False, "status": "directive",
        })
    missing = [a for a in missing
               if " ".join(a.replace("_", " ").upper().split()) not in implemented]
    titles: dict[str, str] = {}
    for page in sorted(DOCS.glob("keyword-*.md")):
        body = read(page)
        if "not registered" in body.lower():
            continue  # the page already states the form has no registration
        for line in body.splitlines():
            if line.startswith("# "):
                title = re.sub(r"\s*[🟢🟡🔴].*$", "", line[2:]).strip()
                titles.setdefault(title.upper(), page.name)
                break
    builtins = parse_builtins()
    missing = [a for a in missing if a.upper() not in set(builtins)]
    claims = [{"name": n, "page": titles[n.upper()]}
              for n in missing if n.upper() in titles]
    unique: dict[tuple, dict] = {}
    for entry in keywords:
        unique.setdefault((entry["name"], entry["signature"], entry["source"]), entry)
    keywords = list(unique.values())
    return {
        "keywords": sorted(keywords, key=lambda k: (k["category"], k["name"], k["line"])),
        "functions": functions,
        "aliases": aliases,
        "directives": directives,
        "builtins": builtins,
        "doc_claims": claims,
        "advertised": advertised,
        "documented_only": missing,
        "template_usage": usage,
    }
