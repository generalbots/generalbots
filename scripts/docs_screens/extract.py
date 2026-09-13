"""Reading an app's real regions, labels, columns and actions."""
import os
import re

from .markup import clean_label, find_all, label_of, parse, strip_code, text_of
from .paths import REGISTRY, SUITE


TRANSIENT = re.compile(r"modal|overlay|dropdown|tooltip|toast|backdrop|"
                       r"context-menu|slash-menu|popover|loader|spinner|"
                       r"dialog|mask|drawer|lightbox", re.I)


SIDEBAR = re.compile(r"sidebar|aside|nav-rail|explorer", re.I)


HEADER = re.compile(r"header|toolbar|topbar|appbar|titlebar", re.I)


BAND = re.compile(r"filter|tabs|tab-bar|status-bar|subheader|breadcrumb|"
                  r"search|toolbar-row|quick-|actions", re.I)


MAIN = re.compile(r"main|content|canvas|body|list|grid|table|workspace|view|"
                  r"editor|surface|area", re.I)


SHARED_DIRS = {"partials", "js", "widgets"}


def resolve_entry(app_id, url):
    """The app's entry document plus the modules that build its surface.

    A few surfaces live in a shared folder (chat is ``partials/chat.html``) or
    are built entirely by scripts held in the app's own folder. Sweeping a
    shared folder would pull every other app's markup into the figure, so
    module collection is scoped to the app's own directory.
    """
    rel = url.split("?")[0].replace("/suite/", "", 1)
    entry = os.path.join(SUITE, rel)
    if not os.path.exists(entry):
        return None, []
    folder = os.path.dirname(entry)
    if os.path.basename(folder) in SHARED_DIRS:
        own = os.path.join(SUITE, app_id)
        if os.path.isdir(own):
            folder = own
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


TITLE_HOLDER = re.compile(r"(panel-header|card-header|widget-header|"
                          r"panel-title|card-title|section-header)", re.I)


PANEL = re.compile(r"(panel|card|widget|tile|module|block)", re.I)


THREAD_NAME = re.compile(r"^(messages|message-list|message-thread|thread|"
                         r"conversation|chat-log|chat-messages)$", re.I)


def sections(node, labels):
    """Real section or module names declared inside a region.

    A section name appears either as a heading element, as a translatable
    span, or as the title bar of a panel (the dominant dashboard pattern,
    where the title is a plain span beside the panel icon).
    """
    raw = []
    for h in find_all(node, lambda n: n.tag in ("h1", "h2", "h3", "h4")):
        raw.append(text_of(h, 60))
    for holder in find_all(node, lambda n: TITLE_HOLDER.search(n.cls())):
        raw.append(text_of(holder, 60))
    for el in find_all(node, lambda n: n.attrs.get("data-i18n")):
        raw.append(label_of(el, labels))
    out, seen = [], set()
    for name in (clean_label(n) for n in raw):
        if name and name.lower() not in seen:
            seen.add(name.lower())
            out.append(name)
    return out[:6]


def panel_titles(body, labels):
    """Titled panels declared as body children (dashboard grid layout).

    A dashboard is a set of sibling panels that each carry their own title;
    unlike master-detail there is no selected record, so the layout is a grid
    of titled panels rather than a list beside a detail pane.
    """
    out = []
    for kid in body.kids:
        if not visible(kid) or not PANEL.search(kid.cls()):
            continue
        title = ""
        for holder in find_all(kid, lambda n: TITLE_HOLDER.search(n.cls())):
            title = clean_label(text_of(holder, 40))
            if title:
                break
        if not title:
            for h in find_all(kid, lambda n: n.tag in ("h1", "h2", "h3", "h4")):
                title = clean_label(text_of(h, 40))
                if title:
                    break
        if title:
            out.append(title)
    seen, uniq = set(), []
    for name in out:
        if name.lower() not in seen:
            seen.add(name.lower())
            uniq.append(name)
    return uniq


def is_conversation(node):
    """Whether a node is the message thread of a conversation surface."""
    if THREAD_NAME.match((node.attrs.get("id") or "").strip()):
        return True
    return any(THREAD_NAME.match(tok) for tok in node.cls().split())


def composer(root, labels):
    """The composer's real placeholder text, when the app declares one."""
    for form in find_all(root, lambda n: n.tag == "form"):
        for inp in find_all(form, lambda n: n.tag in ("input", "textarea")):
            key = inp.attrs.get("data-i18n-placeholder")
            text = clean_label(labels.get(key, ""), 44) if key else ""
            if not text:
                text = clean_label(inp.attrs.get("placeholder", ""), 44)
            if text:
                return text
    return ""


def columns_of(node):
    """Real table column headers, sanitised like any other label."""
    out = []
    for th in find_all(node, lambda n: n.tag == "th"):
        name = clean_label(text_of(th, 40), limit=18)
        if name and name.lower() not in [o.lower() for o in out]:
            out.append(name)
    return out[:6]


REGISTRY_ENTRY = re.compile(
    r'(?:widget_)?app\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,'
    r'\s*"#[0-9a-fA-F]{3,8}"\s*,\s*"([^"]+)"', re.S)


def load_catalog():
    """The app catalogue, read from the registry that defines it.

    ``botserver/src/apps/registry.rs`` is authoritative -- it is the list the
    suite actually renders. There is deliberately no committed snapshot to fall
    back on, because a copy of the catalogue would drift from the registry the
    figures are supposed to document.
    """
    if not os.path.exists(REGISTRY):
        raise SystemExit(f"app registry not found: {REGISTRY}")
    text = open(REGISTRY, encoding="utf-8", errors="replace").read()
    seen, apps = set(), []
    for m in REGISTRY_ENTRY.finditer(text):
        app_id = m.group(1)
        if app_id in seen:
            continue
        seen.add(app_id)
        apps.append({"id": app_id, "title": m.group(2),
                     "category": m.group(3), "url": m.group(4)})
    if not apps:
        raise SystemExit(f"no apps parsed from {REGISTRY}")
    return apps
