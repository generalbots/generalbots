#!/usr/bin/env python3
"""Regenerate the suite app screen SVGs from the real application source.

This is a documentation generator, so it is engineered to avoid inventing
anything. Each mockup is produced from two authoritative inputs:

1. ``botui/ui/suite/<app>/`` markup -- the real region hierarchy. An app
   declares its layout as ``gb-app`` children (sidebar, header, filter band,
   main area); ``hx-get`` partials are followed and inlined, so HTMX-composed
   apps render with the controls their partials actually define.
2. ``botlib/locales/en/ui.ftl`` -- the Fluent catalogue that turns every
   ``data-i18n`` key into its shipped English string.

Regions, labels, columns and actions are therefore real. Row payloads are
drawn as neutral skeleton blocks: the figure documents the interface without
fabricating records the product does not have.

Usage:
    python3 scripts/docs_app_screens.py --report      # extraction summary
    python3 scripts/docs_app_screens.py --check       # verify only, no writes
    python3 scripts/docs_app_screens.py               # rewrite every screen
    python3 scripts/docs_app_screens.py tasks drive   # rewrite a subset
"""
import json
import os
import re
import sys
from html.parser import HTMLParser

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SUITE = os.path.join(ROOT, "botui/ui/suite")
OUT = os.path.join(ROOT, "botbook/src/assets/suite")
FTL = os.path.join(ROOT, "botlib/locales/en/ui.ftl")

W, H = 900, 600
FX, FY, FW, FH = 30, 40, 840, 530
TITLEBAR = 44
PAD = 16
SKIP_TAGS = {"script", "style", "template", "link", "meta", "svg", "path",
             "br", "img", "input", "hr", "g", "defs", "symbol", "use"}
INVISIBLE = re.compile(r"display\s*:\s*none|visibility\s*:\s*hidden", re.I)
TRANSIENT = re.compile(r"modal|overlay|dropdown|tooltip|toast|backdrop|"
                       r"context-menu|slash-menu|popover|loader|spinner", re.I)
SIDEBAR = re.compile(r"sidebar|aside|nav-rail|explorer", re.I)
HEADER = re.compile(r"header|toolbar|topbar|appbar|titlebar", re.I)
BAND = re.compile(r"filter|tabs|tab-bar|status-bar|subheader|breadcrumb|"
                  r"search|toolbar-row|quick-|actions", re.I)
MAIN = re.compile(r"main|content|canvas|body|list|grid|table|workspace|view|"
                  r"editor|surface|area", re.I)
FTL_LINE = re.compile(r"^([a-z0-9][a-z0-9-]*)\s*=\s*(.+)$")
FTL_SKIP = re.compile(r"\{\s*\$|->")


class Node:
    __slots__ = ("tag", "attrs", "kids", "text")

    def __init__(self, tag, attrs):
        self.tag = tag
        self.attrs = attrs
        self.kids = []
        self.text = []

    def cls(self):
        return self.attrs.get("class", "") or ""


class Tree(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.root = Node("root", {})
        self.stack = [self.root]

    def handle_starttag(self, tag, attrs):
        if tag in SKIP_TAGS:
            return
        node = Node(tag, dict(attrs))
        self.stack[-1].kids.append(node)
        if tag not in ("br", "img", "input", "hr"):
            self.stack.append(node)

    def handle_endtag(self, tag):
        if tag in SKIP_TAGS:
            return
        for i in range(len(self.stack) - 1, 0, -1):
            if self.stack[i].tag == tag:
                del self.stack[i:]
                break

    def handle_data(self, data):
        text = re.sub(r"\s+", " ", data).strip()
        if text:
            self.stack[-1].text.append(text)


def load_ftl():
    labels = {}
    for raw in open(FTL, encoding="utf-8").read().splitlines():
        if not raw.strip() or raw.startswith(("#", " ", "\t")):
            continue
        m = FTL_LINE.match(raw)
        if not m:
            continue
        val = m.group(2).strip()
        if FTL_SKIP.search(val):
            continue
        labels[m.group(1)] = val.replace("&amp;", "&")
    return labels


JS_TAGS = ("div|span|button|section|header|footer|aside|main|table|thead|"
           "tbody|tr|th|td|ul|ol|li|h[1-4]|label|nav|article")
# Each fragment is re-closed: a script emits open tags and text separately, so
# joining them verbatim would nest sibling controls and merge their labels.
JS_MARKUP = re.compile(
    rf"<({JS_TAGS})\b([^>]*)>([^<]*)<", re.S)


def js_markup(raw):
    """Recover markup a script builds at runtime.

    Several apps (Meet, Canvas, Player) return their surface from JavaScript
    template strings. Those fragments are real UI and are parsed as such, with
    every fragment explicitly closed so sibling labels stay distinct.
    """
    return [f"<{tag}{attrs}>{text}</{tag}>"
            for tag, attrs, text in JS_MARKUP.findall(raw)
            if text.strip() or "data-i18n" in attrs]


def shadow_tree(scripts):
    """One tree over the markup every script of the app builds.

    Fragments are gathered across all modules: an app whose surface is split
    over many scripts would fall short of the threshold file by file.
    """
    frags = []
    for path in scripts:
        if not path.endswith(".js") or not os.path.exists(path):
            continue
        frags.extend(js_markup(open(path, encoding="utf-8",
                                  errors="replace").read()))
    node = Node("shadow", {})
    if len(frags) >= 3:
        node.kids = parse(" ".join(frags)).kids
    return node


def strip_code(raw):
    raw = re.sub(r"<script\b[^>]*>.*?</script>", " ", raw, flags=re.S | re.I)
    return re.sub(r"<style\b[^>]*>.*?</style>", " ", raw, flags=re.S | re.I)


def parse(text):
    tree = Tree()
    tree.feed(text)
    return tree.root


def find_all(node, pred, out=None):
    out = [] if out is None else out
    for kid in node.kids:
        if pred(kid):
            out.append(kid)
        find_all(kid, pred, out)
    return out


def text_of(node, limit=64):
    parts = list(node.text)
    for kid in node.kids:
        parts.append(text_of(kid, limit))
    joined = re.sub(r"\s+", " ", " ".join(p for p in parts if p)).strip()
    return joined[:limit]


COUNT_SUFFIX = re.compile(r"\s*[-–]\s*$")
NOISE = re.compile(r"^(loading|failed to load|service unavailable|no data|"
                   r"error|untitled)\b", re.I)
# Concatenation artefacts: a recovered label that still carries source-code
# punctuation is a broken expression, not interface text.
CODEY = re.compile(r"[+\"'`=<>(){}\[\]]")
# Escapes survive as literal characters when markup is recovered from a
# JavaScript template string, so they must be folded explicitly.
JS_ESCAPE = re.compile(r"\\[nrt]|\\u[0-9a-fA-F]{4}|\\['\"]")


def clean_label(text, limit=30):
    """Reduce a source string to a usable control label, or drop it.

    Suite markup nests the visible label beside a runtime count placeholder
    (``<span class="pill-count">-</span>``) and repeats tab labels inside a
    badge. Labels recovered from script-generated markup additionally carry
    escape sequences and can be nothing but emoji. None of that belongs in a
    figure, so anything without readable words is discarded outright.
    """
    if not text:
        return ""
    text = JS_ESCAPE.sub(" ", text)
    text = re.sub(r"\s+", " ", text).strip()
    text = COUNT_SUFFIX.sub("", text).strip()
    words = text.split()
    half = len(words) // 2
    if half and words[:half] == words[half:]:
        text = " ".join(words[:half])
    if len(re.findall(r"[A-Za-z]{2,}", text)) < 1 or NOISE.search(text):
        return ""
    if CODEY.search(text):
        return ""
    if len(text) > limit:
        head = text[:limit].rsplit(" ", 1)[0]
        text = head if len(head) >= 8 else text[:limit].rstrip()
    return text.strip()


def label_of(node, labels):
    key = node.attrs.get("data-i18n")
    if key and key in labels:
        return clean_label(labels[key])
    return clean_label(text_of(node, 60))


def resolve_entry(app_id, url):
    rel = url.split("?")[0].replace("/suite/", "", 1)
    entry = os.path.join(SUITE, rel)
    if not os.path.exists(entry):
        return None, []
    folder = os.path.dirname(entry)
    files = [entry]
    if os.path.isdir(folder):
        for name in sorted(os.listdir(folder)):
            if name.endswith((".html", ".js")):
                p = os.path.join(folder, name)
                if p not in files:
                    files.append(p)
    return entry, files


def inline_hx(root, entry, depth=0):
    """Replace bare hx-get wrappers with the partial's own children."""
    if depth > 3:
        return root
    kids = []
    for kid in root.kids:
        target = kid.attrs.get("hx-get", "")
        own_text = text_of(kid)
        if target.startswith("/suite/") and not own_text and not kid.kids:
            path = os.path.join(SUITE, target.split("?")[0]
                                .replace("/suite/", "", 1))
            if os.path.exists(path):
                partial = parse(strip_code(
                    open(path, encoding="utf-8", errors="replace").read()))
                inline_hx(partial, path, depth + 1)
                kids.extend(partial.kids)
                continue
        inline_hx(kid, entry, depth + 1)
        kids.append(kid)
    root.kids = kids
    return root


STATE_PLACEHOLDER = re.compile(r"gb-state|loading|error-state|skeleton", re.I)


def visible(node):
    """Regions worth drawing.

    ``display:none`` is deliberately NOT treated as absent: suite apps ship
    their shells hidden and reveal them once hydrated, so those containers are
    the real interface. Only transient overlays and loading placeholders go.
    """
    cls = node.cls()
    if TRANSIENT.search(cls) or STATE_PLACEHOLDER.search(cls):
        return False
    return node.attrs.get("hidden") is None


def kind_of(node):
    cls = node.cls()
    if node.tag == "aside" or SIDEBAR.search(cls):
        return "sidebar"
    if node.tag == "header" or HEADER.search(cls):
        return "toolbar"
    if node.tag == "main" or MAIN.search(cls):
        return "main"
    if BAND.search(cls):
        return "band"
    return None


def app_root(root):
    for node in find_all(root, lambda n: n.attrs.get("data-gb-app")
                         or "gb-app" in n.cls()):
        return node
    return root


def controls(node, labels, kinds=("button", "a")):
    out = []
    for c in find_all(node, lambda n: n.tag in kinds):
        txt = label_of(c, labels)
        if txt and len(txt) <= 32:
            out.append(("label", txt))
        elif c.tag == "button":
            out.append(("icon", ""))
    seen, uniq = set(), []
    for kind, txt in out:
        key = (kind, txt.lower())
        if txt and key not in seen:
            seen.add(key)
            uniq.append((kind, txt))
    return uniq


def sections(node, labels):
    """Real section or module names declared inside a region."""
    raw = []
    for h in find_all(node, lambda n: n.tag in ("h1", "h2", "h3", "h4")):
        raw.append(text_of(h, 60))
    for el in find_all(node, lambda n: n.attrs.get("data-i18n")):
        raw.append(label_of(el, labels))
    out, seen = [], set()
    for name in (clean_label(n) for n in raw):
        if name and name.lower() not in seen:
            seen.add(name.lower())
            out.append(name)
    return out[:6]


def esc(t):
    return t.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


STYLE = """  <style>
    .bg { fill: #ffffff; }
    .titlebar { fill: #f8fafc; }
    .sidebar-bg { fill: #f8fafc; }
    .main-text { fill: #1e293b; font-family: system-ui, -apple-system, sans-serif; }
    .secondary-text { fill: #64748b; font-family: system-ui, -apple-system, sans-serif; }
    .muted-text { fill: #94a3b8; font-family: system-ui, -apple-system, sans-serif; }
    .white-text { fill: #ffffff; font-family: system-ui, -apple-system, sans-serif; }
    .accent-text { fill: #2563eb; font-family: system-ui, -apple-system, sans-serif; }
    .border { stroke: #e2e8f0; stroke-width: 1; fill: none; }
    .chip { fill: #f1f5f9; }
    .chip-active { fill: #dbeafe; }
    .row { fill: #f8fafc; }
    .bar { fill: #e2e8f0; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #0f172a; }
      .titlebar { fill: #1e293b; }
      .sidebar-bg { fill: #1e293b; }
      .main-text { fill: #e2e8f0; }
      .secondary-text { fill: #94a3b8; }
      .muted-text { fill: #64748b; }
      .accent-text { fill: #60a5fa; }
      .border { stroke: #334155; }
      .chip { fill: #334155; }
      .chip-active { fill: #1e3a5f; }
      .row { fill: #1e293b; }
      .bar { fill: #334155; }
    }
  </style>"""


def chrome(title, subtitle):
    return (f'  <text x="{W // 2}" y="25" text-anchor="middle" font-size="16" '
            f'font-weight="600" class="main-text">{esc(title)}</text>\n'
            f'  <rect x="{FX}" y="{FY}" width="{FW}" height="{FH}" rx="8" '
            f'class="bg"/>\n'
            f'  <rect x="{FX}" y="{FY}" width="{FW}" height="{FH}" rx="8" '
            f'class="border"/>\n'
            f'  <path d="M{FX} {FY + 12} a12 12 0 0 1 12 -12 h{FW - 24} '
            f'a12 12 0 0 1 12 12 v{TITLEBAR - 12} h-{FW} z" class="titlebar"/>\n'
            f'  <text x="{FX + PAD}" y="{FY + 21}" font-size="13.5" '
            f'font-weight="600" class="main-text">{esc(title)}</text>\n'
            f'  <text x="{FX + FW - 54}" y="{FY + 27}" text-anchor="middle" '
            f'font-size="13" class="muted-text">–</text>\n'
            f'  <text x="{FX + FW - 30}" y="{FY + 27}" text-anchor="middle" '
            f'font-size="13" class="muted-text">✕</text>')


def toolbar_band(items, x, y, w, title=""):
    out = []
    if title:
        out.append(f'  <text x="{x}" y="{y + 20}" font-size="13" '
                   f'font-weight="600" class="main-text">{esc(title)}</text>')
    cx = x + (len(title) * 7.5 + 20 if title else 0)
    icons = 0
    for kind, txt in items[:7]:
        if kind == "label" and txt:
            wide = 20 + len(txt) * 6.6
            if wide + cx > x + w:
                break
            out.append(f'  <rect x="{cx:.0f}" y="{y + 3}" width="{wide:.0f}" '
                       f'height="26" rx="6" class="chip"/>')
            out.append(f'  <text x="{cx + wide / 2:.0f}" y="{y + 20}" '
                       f'text-anchor="middle" font-size="11.5" '
                       f'class="secondary-text">{esc(txt)}</text>')
            cx += wide + 8
        else:
            icons += 1
    for _ in range(min(icons, 4)):
        if cx + 30 > x + w:
            break
        out.append(f'  <rect x="{cx:.0f}" y="{y + 3}" width="26" height="26" '
                   f'rx="6" class="chip"/>')
        out.append(f'  <rect x="{cx + 8:.0f}" y="{y + 12}" width="10" '
                   f'height="8" rx="2" class="bar"/>')
        cx += 34
    return "\n".join(out)


def chip_band(items, x, y, w):
    out, cx = [], x
    for i, (_, txt) in enumerate(items[:6]):
        if not txt:
            continue
        wide = 24 + len(txt) * 6.4
        if cx + wide > x + w:
            break
        cls = "chip-active" if i == 0 else "chip"
        tone = "accent-text" if i == 0 else "secondary-text"
        out.append(f'  <rect x="{cx:.0f}" y="{y}" width="{wide:.0f}" '
                   f'height="27" rx="13.5" class="{cls}"/>')
        out.append(f'  <text x="{cx + wide / 2:.0f}" y="{y + 18}" '
                   f'text-anchor="middle" font-size="11.5" class="{tone}">'
                   f'{esc(txt)}</text>')
        cx += wide + 9
    return "\n".join(out)


def table_body(columns, x, y, w, h):
    out = []
    step = w / max(len(columns), 1)
    out.append(f'  <rect x="{x}" y="{y}" width="{w}" height="32" rx="6" '
               f'class="chip"/>')
    for i, col in enumerate(columns):
        out.append(f'  <text x="{x + 12 + i * step:.0f}" y="{y + 21}" '
                   f'font-size="11.5" font-weight="600" class="secondary-text">'
                   f'{esc(col)}</text>')
    rows = max(1, int((h - 40) // 48))
    for r in range(rows):
        ry = y + 40 + r * 48
        out.append(f'  <rect x="{x}" y="{ry:.0f}" width="{w}" height="40" '
                   f'rx="6" class="row"/>')
        for i, _ in enumerate(columns):
            bw = step * (0.36 + 0.16 * ((i + r) % 3))
            out.append(f'  <rect x="{x + 12 + i * step:.0f}" y="{ry + 16:.0f}" '
                       f'width="{bw:.0f}" height="9" rx="4.5" class="bar"/>')
    return "\n".join(out)


def list_body(rows, x, y, w, h, names=()):
    """Master-detail list column: item rows, optionally titled per section."""
    out = []
    titles = list(names)
    for r in range(max(rows, 1)):
        ry = y + r * 52
        if ry + 44 > y + h:
            break
        out.append(f'  <rect x="{x}" y="{ry:.0f}" width="{w}" height="44" '
                   f'rx="7" class="row"/>')
        out.append(f'  <rect x="{x + 12}" y="{ry + 12:.0f}" width="20" '
                   f'height="20" rx="5" class="bar"/>')
        if r < len(titles):
            out.append(f'  <text x="{x + 42}" y="{ry + 27:.0f}" font-size="12" '
                       f'font-weight="500" class="main-text">'
                       f'{esc(titles[r])}</text>')
        else:
            out.append(f'  <rect x="{x + 42}" y="{ry + 15:.0f}" '
                       f'width="{w * 0.42:.0f}" height="9" rx="4.5" '
                       f'class="bar"/>')
        out.append(f'  <rect x="{x + 42}" y="{ry + 30:.0f}" '
                   f'width="{w * 0.26:.0f}" height="6" rx="3" class="bar"/>')
    return "\n".join(out)


def card_body(names, x, y, w, h):
    out, cols = [], 2 if len(names) > 1 else 1
    cw = (w - 16 * (cols - 1)) / cols
    for i, name in enumerate(names[:4]):
        cx = x + (i % cols) * (cw + 16)
        cy = y + (i // cols) * 92
        out.append(f'  <rect x="{cx:.0f}" y="{cy:.0f}" width="{cw:.0f}" '
                   f'height="76" rx="8" class="row"/>')
        out.append(f'  <text x="{cx + 14:.0f}" y="{cy + 26:.0f}" font-size="12.5" '
                   f'font-weight="600" class="main-text">{esc(name)}</text>')
        for k in range(2):
            bw = (cw - 44) * (0.9 - 0.26 * k)
            out.append(f'  <rect x="{cx + 14:.0f}" y="{cy + 40 + k * 14:.0f}" '
                       f'width="{bw:.0f}" height="7" rx="3.5" class="bar"/>')
    return "\n".join(out)


def columns_of(node):
    """Real table column headers, sanitised like any other label."""
    out = []
    for th in find_all(node, lambda n: n.tag == "th"):
        name = clean_label(text_of(th, 40), limit=18)
        if name and name.lower() not in [o.lower() for o in out]:
            out.append(name)
    return out[:6]


def sidebar_band(names, x, y, w, h):
    out = [f'  <rect x="{x}" y="{y:.0f}" width="{w}" height="{h:.0f}" '
           f'class="sidebar-bg"/>',
           f'  <line x1="{x + w}" y1="{y:.0f}" x2="{x + w}" y2="{y + h:.0f}" '
           f'class="border"/>']
    for i, name in enumerate(names[:7]):
        iy = y + 28 + i * 34
        if i == 0:
            out.append(f'  <rect x="{x + 10}" y="{iy - 17:.0f}" '
                       f'width="{w - 20}" height="28" rx="6" class="chip-active"/>')
        tone = "accent-text" if i == 0 else "secondary-text"
        out.append(f'  <circle cx="{x + 24}" cy="{iy - 3:.0f}" r="4.5" '
                   f'class="bar"/>')
        out.append(f'  <text x="{x + 38}" y="{iy + 1:.0f}" font-size="12.5" '
                   f'class="{tone}">{esc(name)}</text>')
    return "\n".join(out)


def render(entry, labels, title, scripts=()):
    """Build one screen from the app's real region hierarchy."""
    root = parse(strip_code(open(entry, encoding="utf-8",
                                 errors="replace").read()))
    root = app_root(root)
    inline_hx(root, entry)

    # Runtime-built markup, used only when the static tree is too thin.
    shadow = shadow_tree(scripts)

    regions = []
    for kid in root.kids:
        if not visible(kid):
            continue
        kind = kind_of(kid)
        if kind:
            regions.append((kind, kid))

    has_sidebar = any(k == "sidebar" for k, _ in regions)
    headers = [n for k, n in regions if k == "toolbar"][:1]
    bands = [n for k, n in regions if k == "band"]
    mains = [n for k, n in regions if k == "main"][:1]
    sides = [n for k, n in regions if k == "sidebar"][:1]
    if not mains:
        mains = [n for k, n in regions if kind_of(n) is None][:1]
        if not mains:
            mains = [root]

    parts = [f'<svg width="{W}" height="{H}" xmlns="http://www.w3.org/2000/svg" '
             f'role="img" aria-label="{esc(title)}">',
             '  <defs>',
             '    <linearGradient id="accentGrad" x1="0%" y1="0%" '
             'x2="100%" y2="100%">',
             '      <stop offset="0%" stop-color="#2563eb"/>',
             '      <stop offset="100%" stop-color="#60a5fa"/>',
             '    </linearGradient>',
             '  </defs>',
             STYLE,
             chrome(title, "")]

    top = FY + TITLEBAR
    bottom = FY + FH
    cx = FX + (200 if has_sidebar else 0)
    cw = FW - (200 if has_sidebar else 0)

    if sides:
        names = sections(sides[0], labels)
        if names:
            parts.append(sidebar_band(names, FX, top, 200, bottom - top))
        else:
            has_sidebar = False
            cx, cw = FX, FW

    y = top
    if headers:
        parts.append(f'  <line x1="{FX}" y1="{y}" x2="{FX + FW}" y2="{y}" '
                     f'class="border"/>')
        parts.append(toolbar_band(controls(headers[0], labels), cx + PAD, y, cw))
        y += 44
        parts.append(f'  <line x1="{FX}" y1="{y}" x2="{FX + FW}" y2="{y}" '
                     f'class="border"/>')

    main_top = y
    for band in bands:
        parts.append(chip_band(controls(band, labels), cx + PAD, y + 12, cw - 32))
        y += 46

    body = mains[0]
    columns = columns_of(body)
    body_actions = controls(body, labels)
    names = sections(body, labels)
    if shadow.kids and not columns and len(names) < 2:
        # Static shell carries too little surface (script-rendered app): fall
        # back to the markup the scripts themselves build.
        body = shadow
        columns = columns_of(body)
        body_actions = controls(body, labels)
        names = sections(body, labels)
    body_y = y + 14
    avail = bottom - body_y - PAD

    # Master-detail is the dominant suite layout: a list pane beside a pane
    # that shows the selected record. Both panes are declared in the markup.
    panes = [k for k in body.kids if visible(k)
             and re.search(r"(list|panel|detail|column|board|pane)", k.cls(),
                           re.I)]
    if len(panes) >= 2 and not columns:
        lw = (cw - 32) * 0.46
        parts.append(list_body(int(avail // 52), cx + PAD, body_y, lw,
                               avail, sections(panes[0], labels)))
        rx = cx + PAD + lw + 16
        rw = cw - 32 - lw - 16
        parts.append(card_body(sections(panes[1], labels) or ["Details"],
                               rx, body_y, rw, avail))
    elif columns:
        parts.append(table_body(columns, cx + PAD, body_y, cw - 32, avail))
    else:
        if body_actions:
            parts.append(toolbar_band(body_actions, cx + PAD, body_y, cw - 32))
            body_y += 42
            avail = bottom - body_y - PAD
        if names:
            parts.append(card_body(names, cx + PAD, body_y, cw - 32, avail))
        else:
            parts.append(list_body(int(avail // 52), cx + PAD, body_y,
                                   cw - 32, avail))

    parts.append("  <!-- regen: scripts/docs_app_screens.py -->")
    parts.append("</svg>")
    return "\n".join(parts) + "\n"


def load_catalog():
    local = os.path.join(ROOT, "botbook/apps.json")
    if os.path.exists(local):
        return json.load(open(local))
    return json.load(open("/tmp/catalog.json"))


def main():
    labels = load_ftl()
    catalog = load_catalog()
    mode = "--report" in sys.argv
    check = "--check" in sys.argv
    wanted = [a for a in sys.argv[1:] if not a.startswith("--")]

    written, failed, report = 0, [], []
    for app in catalog:
        app_id, url = app["id"], app["url"]
        if wanted and app_id not in wanted:
            continue
        entry, sources = resolve_entry(app_id, url)
        if not entry:
            failed.append((app_id, "no entry file"))
            continue
        title = labels.get(f"{app_id}-title") or app["title"]
        try:
            svg = render(entry, labels, title, sources)
        except Exception as exc:                      # noqa: BLE001
            failed.append((app_id, f"{type(exc).__name__}: {exc}"))
            continue
        regions = len(re.findall(r'class="(chip|row|sidebar-bg)"', svg))
        report.append((app_id, title, len(svg), regions))
        if mode or check:
            continue
        with open(os.path.join(OUT, f"{app_id}-screen.svg"), "w",
                  encoding="utf-8") as fh:
            fh.write(svg)
        written += 1

    if mode:
        print(f"{'app':24} {'title':24} bytes visuals")
        for app_id, title, size, regions in report:
            print(f"{app_id:24} {title[:23]:24} {size:5} {regions:4}")
    else:
        print(f"wrote {written} screens" if not check else
              f"checked {len(report)} screens")
    for app_id, err in failed:
        print(f"  FAILED {app_id}: {err}")


if __name__ == "__main__":
    main()
