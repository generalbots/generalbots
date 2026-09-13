"""HTML parsing, script-recovered markup, and label sanitising."""
import os
import re
from html.parser import HTMLParser

from .paths import FTL


SKIP_TAGS = {"script", "style", "template", "link", "meta", "svg", "path",
             "br", "img", "input", "hr", "g", "defs", "symbol", "use"}


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
        # Inputs carry no children, but their placeholder is shipped interface
        # text (the chat composer), so they are kept as leaves.
        if tag in SKIP_TAGS and tag not in ("input", "textarea"):
            return
        node = Node(tag, dict(attrs))
        self.stack[-1].kids.append(node)
        if tag not in ("br", "img", "input", "hr", "textarea"):
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


CODEY = re.compile(r"[+\"'`=<>(){}\[\]]")


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
