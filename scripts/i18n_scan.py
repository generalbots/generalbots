#!/usr/bin/env python3
"""Hardcoded user-facing English literal scanner (#2096).

One implementation shared by `scripts/i18n_lint.py` (the CI gate) and the
migration tooling, so "what the gate flags" and "what gets localized" can never
drift apart.

A *literal* is a text node in a suite/cloud/login HTML file, or an English
string that a JS file assigns to a user-visible slot (textContent, placeholder,
title, aria-label, alert/confirm/toast). The scanner skips:

* `vendor/`, `wizard/` and the other non-catalogue subtrees (they ship their own
  dictionaries),
* `script`, `style`, `pre`, `code`, `kbd`, `samp`, `svg`, `math`, `textarea`
  and `<title>` (machine content, code samples, or a document title that the
  `data-i18n-document-title` attribute owns),
* any element already carrying a `data-i18n*` attribute, and every descendant
  of such an element,
* proper nouns, brand names and technical identifiers from `ALLOWLIST`, plus
  version strings, URLs, e-mail addresses and pure numbers.
"""

from __future__ import annotations

import html
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UI = ROOT / "botui" / "ui"
APPS = ("suite", "cloud", "login")

SKIP_DIRS = frozenset(
    {
        "vendor", "wizard", "embedded",
        "minimal", "public", "themes",
        "shared", "dist", "node_modules",
    }
)

SKIP_TAGS = frozenset(
    {"script", "style", "pre", "code", "kbd", "samp", "svg", "math", "title", "textarea"}
)

I18N_ATTRS = (
        "data-i18n", "data-i18n-html", "data-i18n-placeholder",
        "data-i18n-title", "data-i18n-aria-label", "data-i18n-value",
        "data-i18n-label", "data-i18n-document-title",
)

# Proper nouns, product names, protocol/standard identifiers and the acronyms
# that every locale keeps verbatim. A literal is exempt only when the whole
# normalized string is one of these — "General Bots is ready" is still flagged.
ALLOWLIST = frozenset(
    {
        "General Bots", "General Bots Cloud", "General Bots Desktop",
        "General Bots 6", "General Bots Suite", "BotBook",
        "BotServer", "BotUI", "GitHub",
        "GitLab", "Google", "Google Drive",
        "Google Calendar", "Microsoft", "Microsoft 365",
        "Microsoft Teams", "OpenAI", "Anthropic",
        "DeepSeek", "Qwen", "Gemma",
        "Llama", "Mistral", "Groq",
        "WhatsApp", "Telegram", "Slack",
        "Discord", "Zoom", "Stripe",
        "PayPal", "PIX", "NFe",
        "NFSe", "SPED", "Zitadel",
        "MinIO", "Vault", "PostgreSQL",
        "Valkey", "Redis", "Qdrant",
        "LiveKit", "Stalwart", "Forgejo",
        "Incus", "Caddy", "CoreDNS",
        "Docker", "Kubernetes", "Tauri",
        "HTMX", "BASIC", "CSV",
        "PDF", "JSON", "API",
        "URL", "SLA", "SMTP",
        "IMAP", "DNS", "SSL",
        "TLS", "SSO", "MFA",
        "TOTP", "QR", "LDAP",
        "OAuth", "JWT", "UUID",
        "SQL", "CSS", "HTML",
        "XML", "YAML", "ISO",
        "SOC 2", "SOC 2 Type II", "ISO 27001",
        "GDPR", "LGPD", "HIPAA",
        "PCI DSS", "English", "Deutsch",
        "Español", "Français", "Português",
        "Italiano", "Acme Corporation", "Rio de Janeiro",
        "New York", "São Paulo", "Sarah Johnson",
        "Mike Chen", "Emily Davis", "software_development",
        "Localhost", "Ok", "OK",
        "PT-BR", "en", "pt-BR",
        "es", "fr", "de",
        "ja", "ko", "zh-CN",
    }
)

ALLOWED_PATTERNS = (
    re.compile(r"^v?\d+(?:[.\-/]\d+)*$"),          # versions, dates, plain numbers
    re.compile(r"^[\w.+-]+@[\w.-]+\.\w+$"),        # e-mail addresses
    re.compile(r"^(?:https?://|www\.)\S+$"),       # URLs
    re.compile(r"^(?:ISO|IEC|RFC|IEEE)\s?\d+(?:[-:]\d+)*$"),
    re.compile(r"^(?:v?\d+(?:\.\d+)*\s?(?:MB|GB|TB|KB|ms|s|min|h|px|%)?)$"),
)

WORD_RE = re.compile(r"[A-Za-z]{2,}")
# A text node that already carries Portuguese/Spanish (accented letters or an
# unambiguous stopword) is somebody's finished translation, not a source string.
TRANSLATED_RE = re.compile(r"[áàâãäçéêíóôõúüñÁÀÂÃÄÇÉÊÍÓÔÕÚÜÑßøåæœÁÉÍÓÚ]")
TRANSLATED_WORDS = frozenset(
    {
        "carregando", "cancelar", "salvar", "atualizar", "emitir", "relatórios",
        "relatorios", "configurações", "configuracoes", "gestão", "gestao",
        "pendentes", "emitidas", "senha", "usuário", "usuario", "página", "pagina",
        "próximo", "proximo", "selecione", "nenhum", "nenhuma", "disponível",
        "disponivel", "indisponível", "indisponivel", "serviço", "servico", "mês",
        "hoje", "ontem", "início", "inicio", "bem-vindo", "pesquisar", "excluir",
        "adicionar", "remover", "editar", "criar", "fechar", "abrir", "enviar",
        "voltar", "cargando", "guardar", "actualizar", "eliminar", "añadir",
        "cerrar", "buscar", "ningún", "ningun", "seleccionar", "contraseña",
        "contraseina", "ayer", "mes", "dia", "dias", "ano", "anos", "todos",
        "todas", "novo", "nova", "nuevo", "nueva", "título", "titulo", "total",
        "valor", "cliente", "clientes", "pedidos", "estoque", "estoque-",
        "funcionário", "funcionarios", "empresa", "empresas", "usuários",
        "agendar", "alertas", "recentes", "alta", "baixa", "solicitar",
        "equipe", "chave", "fiscal", "brasileiro", "hora", "nome", "telefone",
        "gerar", "cnpj", "cpf", "cep", "boleto", "saldo", "filial", "movimentação",
        "movimentacao", "estoque", "estoque-minimo", "cadastro", "painel",
    }
)
TAG_RE = re.compile(r"<(/?)([a-zA-Z][\w:.-]*)((?:[^<>\"']|\"[^\"]*\"|'[^']*')*)>", re.S)
BLOCK_RE = re.compile(
    r"<script\b[^>]*>.*?</script\s*>|<style\b[^>]*>.*?</style\s*>|<!--.*?-->",
    re.S | re.I,
)
VOID_TAGS = frozenset(
    {
        "area", "base", "br", "col", "embed", "hr", "img", "input",
        "link", "meta", "param", "source", "track", "wbr",
    }
)

INLINE_SCRIPT_RE = re.compile(r"<script\b[^>]*>(.*?)</script\s*>", re.S | re.I)

JS_PATTERNS = (
    re.compile(
        r"\.(?:textContent|innerText|innerHTML|placeholder|title)\s*=\s*([\"'`])"
        r"((?:\\.|(?!\1).){2,}?)\1"
    ),
    re.compile(
        r"setAttribute\(\s*[\"'](?:title|placeholder|aria-label)[\"']\s*,\s*([\"'`])"
        r"((?:\\.|(?!\1).){2,}?)\1"
    ),
    re.compile(
        r"\b(?:alert|confirm|notify|toast|showToast|showError|showMessage)\(\s*([\"'`])"
        r"((?:\\.|(?!\1).){2,}?)\1"
    ),
)

CODE_LEAK_RE = re.compile(r"[{}%$\\<>]|=>|\bvar\b|\bfunction\b|\breturn\b|\bnull\b")


def normalize(text: str) -> str:
    """Collapse whitespace and resolve HTML entities to the rendered text."""
    return " ".join(html.unescape(text).split())


def is_allowlisted(text: str) -> bool:
    """Proper nouns survive translation; so do emoji-prefixed brand labels."""
    if text in ALLOWLIST or text.lower() in _ALLOWLIST_LOWER:
        return True
    stripped = re.sub(r"[^\w\s]", "", text).strip()
    if stripped and (stripped in ALLOWLIST or stripped.lower() in _ALLOWLIST_LOWER):
        return True
    return any(pattern.match(text) for pattern in ALLOWED_PATTERNS)


def already_translated(text: str) -> bool:
    """True when the literal is a finished pt-BR/es string, not English."""
    if TRANSLATED_RE.search(text):
        return True
    words = re.findall(r"[A-Za-zÀ-ÿ-]+", text.lower())
    return any(word in TRANSLATED_WORDS for word in words)


def looks_english(text: str) -> bool:
    """Heuristic: does this literal need a translator?"""
    if not text or is_allowlisted(text) or already_translated(text):
        return False
    if text[0].isdigit():
        # "0 connected", "3 failing": the leading count is data the JS refreshes.
        return False
    if CODE_LEAK_RE.search(text):
        return False
    words = WORD_RE.findall(text)
    if not words:
        return False
    if len(words) == 1 and len(words[0]) < 4:
        return False
    return True


_ALLOWLIST_LOWER = frozenset(name.lower() for name in ALLOWLIST)


class Literal:
    """One hardcoded literal found in a UI file."""

    __slots__ = ("path", "line", "text", "tag", "insert_at", "simple")

    def __init__(self, path, line, text, tag="", insert_at=None, simple=True):
        self.path = path
        self.line = line
        self.text = text
        self.tag = tag
        self.insert_at = insert_at
        self.simple = simple

    def __repr__(self):  # pragma: no cover - debugging aid
        return f"Literal({self.path}:{self.line}, {self.text!r})"


def _line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def scan_html(text: str, path: str = "") -> list:
    """Return every English text node that a translator should own."""
    masked = list(text)
    for match in BLOCK_RE.finditer(text):
        masked[match.start():match.end()] = " " * (match.end() - match.start())
    masked = "".join(masked)

    # Inline <script> blocks are JS, not markup: scan them with the JS rules so
    # a page that inlines its module does not escape the gate.
    literals = []
    for match in INLINE_SCRIPT_RE.finditer(text):
        opening = text[match.start():match.start() + 200]
        if "src=" in opening.split(">")[0]:
            continue
        base_line = text.count("\n", 0, match.start())
        for literal in scan_js(match.group(1), path):
            literal.line += base_line
            literals.append(literal)
    stack = []          # open elements: dict(tag, blocked, insert_at, kids, texts, literal)
    cursor = 0

    for match in TAG_RE.finditer(masked):
        segment = text[cursor:match.start()]
        if segment.strip() and stack:
            top = stack[-1]
            if not top["blocked"]:
                normalized = normalize(segment)
                if looks_english(normalized):
                    literal = Literal(
                        path,
                        _line_of(text, cursor),
                        normalized,
                        top["tag"],
                        top["insert_at"],
                        True,
                    )
                    literals.append(literal)
                    top["literal"] = literal
                    top["texts"] += 1
        cursor = match.end()

        closing, tag, attrs = match.group(1), match.group(2).lower(), match.group(3)
        if closing:
            if not stack:
                continue
            open_element = stack.pop()
            if open_element["literal"] is not None and (
                open_element["kids"] > 0 or open_element["texts"] > 1
            ):
                # Mixed content: the parent cannot carry a plain data-i18n key.
                open_element["literal"].simple = False
            continue
        if tag in VOID_TAGS:
            continue
        blocked = (
            tag in SKIP_TAGS
            or (stack and stack[-1]["blocked"])
            or any(attr in attrs.lower() for attr in I18N_ATTRS)
        )
        if stack:
            stack[-1]["kids"] += 1
        stack.append(
            {
                "tag": tag,
                "blocked": blocked,
                "insert_at": match.end() - 1,
                "kids": 0,
                "texts": 0,
                "literal": None,
            }
        )

    return literals


TAG_STRIP_RE = re.compile(r"<[^>]*>")


def _html_string_text(value: str) -> list:
    """Text nodes hidden inside an HTML string a JS file assigns to innerHTML."""
    if "<" not in value:
        return []
    found = []
    for segment in TAG_STRIP_RE.split(value):
        candidate = normalize(segment)
        if looks_english(candidate):
            found.append(candidate)
    return found


def scan_js(text: str, path: str = "") -> list:
    literals = []
    for pattern in JS_PATTERNS:
        for match in pattern.finditer(text):
            literal = normalize(match.group(2))
            if "${" in literal:
                continue
            line = _line_of(text, match.start())
            if looks_english(literal):
                literals.append(Literal(path, line, literal, "js"))
                continue
            # innerHTML = '<p>No invoices yet.</p>' — the markup is fine, the
            # English text node inside it is not.
            for hidden in _html_string_text(literal):
                literals.append(Literal(path, line, hidden, "js"))
    return literals


def ui_files(app: str):
    base = UI / app
    if not base.exists():
        return []
    files = sorted(list(base.rglob("*.html")) + list(base.rglob("*.js")))
    selected = []
    for path in files:
        relative = path.relative_to(base)
        if SKIP_DIRS.intersection(relative.parts[:-1]):
            continue
        selected.append(path)
    return selected


def scan_file(path: Path) -> list:
    try:
        text = path.read_text(encoding="utf-8")
    except (UnicodeDecodeError, OSError):
        return []
    relative = path.relative_to(ROOT).as_posix() if path.is_absolute() else str(path)
    if path.suffix == ".html":
        return scan_html(text, relative)
    return scan_js(text, relative)


def scan_app(app: str) -> list:
    found = []
    for path in ui_files(app):
        found.extend(scan_file(path))
    return found


def scan_all() -> dict:
    return {app: scan_app(app) for app in APPS}
