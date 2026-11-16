#!/usr/bin/env python3
"""Guard (#2303): flag hardcoded PT-BR user-facing strings bypassing i18n in suite JS/HTML.

Targets common PT-BR UI tokens that must go through data-i18n / gbT:
  Salvar, Cancelar, Excluir, Arquivo, Planilha, Responder, Carregando, Erro ao, Nenhum/Nenhuma

Allows: translation catalogs (assistant/i18n/*.js), data-i18n/gbT usage, comments,
console-only debug lines, and inline bilingual helpers/tables where appropriate.
"""
from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1] / "botui" / "ui" / "suite"
EXCLUDED_PARTS = {"vendor", "tests"}
SUFFIXES = {".html", ".js"}
PATTERNS = [
    r"\bSalvar\b", r"\bCancelar\b", r"\bExcluir\b", r"\bArquivo\b", r"\bPlanilha\b",
    r"\bResponder\b", r"\bCarregando\b", r"\bErro ao\b", r"\bNenhum\b", r"\bNenhuma\b",
]
RX = re.compile("|".join(PATTERNS))

issues = []
for p in ROOT.rglob("*"):
    if not p.is_file() or p.suffix not in SUFFIXES:
        continue
    if any(part in EXCLUDED_PARTS for part in p.parts) or p.name.endswith(".orig"):
        continue
    rel = p.relative_to(ROOT)
    # Translation catalogs contain PT-BR values by design.
    rel_parts = rel.parts
    if len(rel_parts) >= 3 and rel_parts[0] == "assistant" and rel_parts[1] == "i18n":
        continue
    # Module-local bilingual translation tables (not user-facing bypass).
    if rel.as_posix() == "assistant/modules/10_settings_model.js":
        continue
    try:
        lines = p.read_text(encoding="utf-8", errors="ignore").splitlines()
    except Exception:
        continue
    for i, line in enumerate(lines, 1):
        if not RX.search(line):
            continue
        stripped = line.strip()
        if stripped.startswith("//") or stripped.startswith("*") or stripped.startswith("<!--") or stripped.startswith("/*"):
            continue
        # console-only debug (not rendered to user)
        if re.search(r'console\.(error|warn|log|debug)\s*\(', line) and not re.search(r'(innerHTML|textContent|insertAdjacentHTML|outerHTML|showToast|\.toast|alert\(|confirm\(|prompt\(|GBModal|\.text\b|placeholder|aria-label|title\s*=|setAttribute\(\s*["\']title)', line):
            continue
        # HTML/JS i18n-resolved: data-i18n elements are translated at runtime.
        if "data-i18n" in line:
            continue
        # translation/catalog usage
        if "gbT(" in line or "GBI18N" in line:
            if not re.search(r'(innerHTML\s*=|textContent\s*=|insertAdjacentHTML\s*\(|showToast\(|\.alert\(|\.confirm\()', line):
                continue
        # inline bilingual helper t("PT","EN") used as localization (not bypassing)
        if re.search(r'\bt\s*\(\s*["\'][^"\']*["\']\s*,\s*["\'][^"\']*["\']\s*\)', line):
            if not re.search(r'(innerHTML\s*=|textContent\s*=|showToast\(|\.alert\(|\.confirm\()', line):
                continue
        # app-i18n/catalog lookups
        if 'window.gbT ? window.gbT' in line or 'return window.gbT' in line:
            continue
        issues.append((str(rel), i, line.rstrip()))

if issues:
    print(f"FAIL: {len(issues)} hardcoded PT-BR user-facing string(s) bypassing i18n in suite")
    for path, line, text in issues:
        print(f"{path}:{line}: {text[:200]}")
    sys.exit(1)
print("OK: no targeted hardcoded PT-BR user-facing strings bypassing i18n in suite (JS/HTML)")
