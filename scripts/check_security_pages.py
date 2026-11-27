#!/usr/bin/env python3
"""Offline contract gate for the PT-BR-first security bulletin pages (#2716).

Checks, without a browser or a network:

* all lang/*.json catalogues parse, are flat string maps with identical keys;
* every data-i18n* key used by the pages exists in every catalogue, and the
  inline PT-BR fallback text matches the authoritative catalogue exactly;
* no bare user-visible HTML text and no `[key]` placeholders;
* one <h1> per page, sequential headings, skip link, <details>/<summary>,
  descriptive links, a keyboard-reachable reporting contact and an explicit
  authoritative-locale marker;
* focus/contrast CSS ratios (4.5:1 text, 3:1 non-text) recomputed from the
  stylesheet's custom properties.

Usage: python3 scripts/check_security_pages.py
"""

from __future__ import annotations

import json
import re
import sys
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PAGES = ROOT / "security" / "pages"
LANG_DIR = PAGES / "lang"
HTML_FILES = sorted((PAGES / "security").glob("*.html"))
CSS_FILE = PAGES / "security" / "security.css"
JS_FILE = PAGES / "i18n.js"

AUTHORITATIVE = "pt-BR"
SECOND_LOCALE = "en"
REQUIRED_LOCALES = {
    "pt-BR", "en", "es", "fr", "de", "it", "ru", "ja", "ko", "zh-CN",
}
I18N_ATTRS = (
    "data-i18n", "data-i18n-aria-label", "data-i18n-title",
    "data-i18n-placeholder",
)
KEY_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
PLACEHOLDER_RE = re.compile(r"^\[[a-zA-Z0-9._-]+\]$")
GENERIC_LINKS = {
    "click here", "here", "more", "link", "read more", "saiba mais", "aqui",
    "clique aqui", "más", "plus", "hier", "weiter", "ここ", "여기", "点击这里",
}
CONTACT = "security@generalbots.org"
VOID_TAGS = frozenset({
    "area", "base", "br", "col", "embed", "hr", "img", "input",
    "link", "meta", "param", "source", "track", "wbr",
})

TEXT_PAIRS = (
    ("--gb-text", "--gb-bg"),
    ("--gb-muted", "--gb-bg"),
    ("--gb-link", "--gb-bg"),
    ("--gb-link-hover", "--gb-bg"),
    ("--gb-text", "--gb-surface"),
    ("--gb-muted", "--gb-surface"),
    ("--gb-link", "--gb-surface"),
    ("--gb-accent-text", "--gb-accent-bg"),
    ("--gb-border", "--gb-bg"),
)
NONTEXT_PAIRS = (
    ("--gb-focus", "--gb-bg"),
    ("--gb-focus", "--gb-surface"),
    ("--gb-link", "--gb-bg"),
    ("--gb-border", "--gb-bg"),
)


def relative(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


class PageParser(HTMLParser):
    """Collect the structural and i18n facts we need from one HTML page."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.errors: list[str] = []
        self.stack: list[tuple[str, dict[str, str | None]]] = []
        self.headings: list[int] = []
        self.ids: set[str] = set()
        self.keys: set[str] = set()
        self.inline_text: dict[str, str] = {}
        self.inline_attrs: dict[tuple[str, str], str] = {}
        self.first_focusable: tuple[str, dict[str, str | None]] | None = None
        self.main_id = False
        self.main_focusable = False
        self.skip_href: str | None = None
        self.switcher_locales: set[str] = set()
        self.details = 0
        self.summaries = 0
        self.links: list[tuple[str | None, str, bool]] = []
        self.link_stack: list[dict[str, object]] = []
        self.html_lang: str | None = None
        self.authoritative: bool = False
        self.contact_found = False
        self.ignored_depth = 0

    def error(self, message: str) -> None:
        self.errors.append(message)

    def current_key(self) -> str | None:
        for _tag, attrs in reversed(self.stack):
            if "data-i18n" in attrs:
                return attrs["data-i18n"]
        return None

    def handle_starttag(self, tag: str, raw_attrs: list[tuple[str, str | None]]) -> None:
        attrs = {name.lower(): value for name, value in raw_attrs}
        if tag not in VOID_TAGS:
            self.stack.append((tag, attrs))

        if tag in {"script", "style"}:
            self.ignored_depth += 1
        if tag == "html":
            self.html_lang = attrs.get("lang")
        if re.fullmatch(r"h[1-6]", tag) is not None:
            self.headings.append(int(tag[1]))
        if attrs.get("id"):
            identifier = attrs["id"]
            if identifier in self.ids:
                self.error(f"duplicate id {identifier!r}")
            self.ids.add(identifier)
        if attrs.get("data-authoritative-locale") == AUTHORITATIVE:
            self.authoritative = True
        if attrs.get("data-locale"):
            self.switcher_locales.add(str(attrs["data-locale"]))
        if tag == "main" and attrs.get("id") == "main":
            self.main_id = True
            self.main_focusable = "tabindex" in attrs
        if tag == "details":
            self.details += 1
        if tag == "summary":
            self.summaries += 1
        if tag == "img" and "alt" not in attrs:
            self.error("image without alt attribute")

        for attribute in I18N_ATTRS:
            key = attrs.get(attribute)
            if key is None:
                continue
            if not isinstance(key, str) or KEY_RE.fullmatch(key) is None:
                self.error(f"invalid i18n key {key!r} in {attribute}")
            else:
                self.keys.add(key)
            if attribute != "data-i18n":
                self.inline_attrs[(key, attribute)] = ""

        if tag in {"a", "button", "summary", "input", "select", "textarea"}:
            if self.first_focusable is None:
                self.first_focusable = (tag, attrs)
        if tag == "a":
            href = attrs.get("href")
            if not href or href.startswith("javascript:"):
                self.error("link without a safe href")
            if attrs.get("target") == "_blank":
                self.error("link opens a new tab without warning")
            if "skip-link" in (attrs.get("class") or "").split():
                self.skip_href = href
            if href and CONTACT in href:
                self.contact_found = True
            self.link_stack.append({
                "href": href,
                "text": "",
                "i18n": self.current_key() is not None or "data-i18n" in attrs,
            })

    def handle_endtag(self, tag: str) -> None:
        if tag in {"script", "style"} and self.ignored_depth > 0:
            self.ignored_depth -= 1
        if tag == "a" and self.link_stack:
            href, text, has_i18n = self.link_stack.pop()
            normalized = " ".join(str(text).split()).strip().casefold()
            if normalized in GENERIC_LINKS:
                self.error(f"non-descriptive link text {str(text).strip()!r}")
            if not normalized:
                self.error(f"link {href!r} has no text")
            self.links.append((href, normalized, has_i18n))
        if self.stack and self.stack[-1][0] == tag:
            self.stack.pop()

    def handle_data(self, data: str) -> None:
        if self.ignored_depth > 0 or not data.strip():
            return
        key = self.current_key()
        if key is None:
            self.error(f"bare user-visible text without data-i18n: {data.strip()[:60]!r}")
            return
        self.inline_text[key] = self.inline_text.get(key, "") + data
        for link in self.link_stack:
            link["text"] = str(link["text"]) + data


def luminance(color: str) -> float:
    value = color.strip().lstrip("#")
    if len(value) != 6:
        raise ValueError(f"not a 6-digit hex colour: {color!r}")
    channels = []
    for index in (0, 2, 4):
        component = int(value[index:index + 2], 16) / 255.0
        linear = component / 12.92 if component <= 0.03928 else ((component + 0.055) / 1.055) ** 2.4
        channels.append(linear)
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]


def contrast(first: str, second: str) -> float:
    light, dark = sorted((luminance(first), luminance(second)), reverse=True)
    return (light + 0.05) / (dark + 0.05)


def css_variables(css: str) -> dict[str, str]:
    root = re.search(r":root\s*\{([^}]*)\}", css)
    if root is None:
        return {}
    return {
        name.strip(): value.strip()
        for name, value in re.findall(r"(--[\w-]+)\s*:\s*([^;]+);", root.group(1))
    }


def check_catalogs(errors: list[str]) -> dict[str, dict[str, str]]:
    paths = sorted(LANG_DIR.glob("*.json"))
    found = {path.stem for path in paths}
    missing_locales = sorted(REQUIRED_LOCALES - found)
    if missing_locales:
        errors.append(f"missing locale catalogues: {missing_locales}")

    catalogs: dict[str, dict[str, str]] = {}
    for path in paths:
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError) as exc:
            errors.append(f"{relative(path)}: cannot parse ({exc})")
            continue
        if not isinstance(data, dict) or not all(
            isinstance(key, str) and isinstance(value, str)
            for key, value in data.items()
        ):
            errors.append(f"{relative(path)}: expected a flat string map")
            continue
        catalogs[path.stem] = data
        for key, value in data.items():
            if KEY_RE.fullmatch(key) is None:
                errors.append(f"{relative(path)}: invalid key {key!r}")
            if PLACEHOLDER_RE.fullmatch(value.strip()):
                errors.append(f"{relative(path)}: {key} renders a [key] placeholder")

    if catalogs:
        key_sets = {locale: set(data) for locale, data in catalogs.items()}
        reference = key_sets[AUTHORITATIVE] if AUTHORITATIVE in key_sets else set(next(iter(key_sets.values())))
        for locale, keys in sorted(key_sets.items()):
            if keys != reference:
                errors.append(
                    f"{locale}.json key set differs: "
                    f"missing={sorted(reference - keys)} extra={sorted(keys - reference)}"
                )
        if SECOND_LOCALE in catalogs and catalogs[SECOND_LOCALE] == catalogs.get(AUTHORITATIVE):
            errors.append("en and pt-BR catalogues are identical; EN translation is missing")
    return catalogs


def check_pages(errors: list[str], catalogs: dict[str, dict[str, str]]) -> None:
    if not HTML_FILES:
        errors.append("no bulletin HTML pages found")
    used_keys: set[str] = set()
    for path in HTML_FILES:
        parser = PageParser()
        try:
            parser.feed(path.read_text(encoding="utf-8"))
            parser.close()
        except OSError as exc:
            errors.append(f"{relative(path)}: cannot read ({exc})")
            continue
        label = relative(path)
        for message in parser.errors:
            errors.append(f"{label}: {message}")

        if parser.html_lang != AUTHORITATIVE:
            errors.append(f"{label}: <html lang> must be {AUTHORITATIVE!r} (PT-BR first)")
        if parser.headings.count(1) != 1 or (parser.headings and parser.headings[0] != 1):
            errors.append(f"{label}: expected exactly one <h1> and it must come first")
        previous = 0
        for level in parser.headings:
            if previous and level > previous + 1:
                errors.append(f"{label}: heading jump h{previous} -> h{level}")
            previous = level
        if not parser.main_id or not parser.main_focusable:
            errors.append(f"{label}: main#main must exist and be focusable for the skip link")
        if parser.first_focusable is None:
            errors.append(f"{label}: first focusable element must be the skip link to #main")
        else:
            tag, attrs = parser.first_focusable
            classes = (attrs.get("class") or "").split()
            if tag != "a" or "skip-link" not in classes or parser.skip_href != "#main":
                errors.append(f"{label}: first focusable element must be the skip link to #main")
        if parser.details == 0 or parser.details != parser.summaries:
            errors.append(f"{label}: FAQs must use matched <details>/<summary> pairs")
        if not parser.authoritative:
            errors.append(f"{label}: missing data-authoritative-locale={AUTHORITATIVE!r} marker")
        if not parser.contact_found:
            errors.append(f"{label}: reporting contact {CONTACT!r} is not a reachable link")
        for locale in sorted(REQUIRED_LOCALES - parser.switcher_locales):
            errors.append(f"{label}: language switcher lacks {locale}")
        if 'data-i18n="authoritative-note"' not in path.read_text(encoding="utf-8"):
            errors.append(f"{label}: authoritative-note marker is missing")

        used_keys.update(parser.keys)
        authoritative = catalogs.get(AUTHORITATIVE, {})
        reference = catalogs.get("en", {})
        for key in sorted(parser.keys):
            for locale, catalog in sorted(catalogs.items()):
                if key not in catalog:
                    errors.append(f"{label}: key {key!r} missing from {locale}.json")
            inline = parser.inline_text.get(key)
            if inline is not None and key in authoritative:
                expected = " ".join(authoritative[key].split())
                if " ".join(inline.split()) != expected:
                    errors.append(
                        f"{label}: inline PT-BR text for {key!r} differs from pt-BR.json"
                    )
            if key not in authoritative and key in reference and inline is not None:
                errors.append(f"{label}: key {key!r} has no authoritative PT-BR text")

        for (key, attribute), inline in sorted(parser.inline_attrs.items()):
            catalog = catalogs.get(AUTHORITATIVE, {})
            if key in catalog and " ".join(inline.split()) != " ".join(catalog[key].split()) and inline:
                errors.append(f"{label}: inline {attribute} for {key!r} differs from pt-BR.json")

        for href, text, has_i18n in parser.links:
            if not has_i18n:
                errors.append(f"{label}: link {href!r} is not covered by a data-i18n key")

    for key in sorted(used_keys):
        for locale, catalog in sorted(catalogs.items()):
            if key not in catalog:
                errors.append(f"key {key!r} used by the pages but missing from {locale}.json")


def check_styles(errors: list[str]) -> None:
    css = CSS_FILE.read_text(encoding="utf-8")
    variables = css_variables(css)
    for name, _ in TEXT_PAIRS + NONTEXT_PAIRS:
        if name not in variables:
            errors.append(f"{relative(CSS_FILE)}: missing custom property {name}")
    for name, background in TEXT_PAIRS:
        if name in variables and background in variables:
            ratio = contrast(variables[name], variables[background])
            if ratio < 4.5:
                errors.append(f"contrast {name} on {background}: {ratio:.2f}:1 < 4.5:1")
    for name, background in NONTEXT_PAIRS:
        if name in variables and background in variables:
            ratio = contrast(variables[name], variables[background])
            if ratio < 3.0:
                errors.append(f"contrast {name} on {background}: {ratio:.2f}:1 < 3:1")
    if ":focus-visible" not in css or "outline:" not in css:
        errors.append(f"{relative(CSS_FILE)}: visible focus style is missing")
    if ".skip-link" not in css:
        errors.append(f"{relative(CSS_FILE)}: skip-link style is missing")


def check_loader(errors: list[str]) -> None:
    script = JS_FILE.read_text(encoding="utf-8")
    for needle in ('[data-i18n]', '"pt-BR"', '"en"', "root.lang = locale", "data-authoritative-locale"):
        if needle not in script:
            errors.append(f"{relative(JS_FILE)}: missing loader contract {needle!r}")


def main() -> int:
    errors: list[str] = []
    catalogs = check_catalogs(errors)
    check_pages(errors, catalogs)
    check_styles(errors)
    check_loader(errors)

    if errors:
        print(f"FAIL: {len(errors)} problem(s)")
        for error in errors:
            print(f"  - {error}")
        return 1
    print(
        "OK: 8 catalogues / "
        f"{len(next(iter(catalogs.values()))) if catalogs else 0} keys, "
        f"{len(HTML_FILES)} pages, skip link, one h1, sequential headings, "
        "details/summary, authoritative marker, contact link, contrast and focus"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
