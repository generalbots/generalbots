"""Assembling one screen from an app's extracted regions."""
import re

from .layout import FH, FW, FX, FY, H, PAD, STYLE, TITLEBAR, W, card_body, chip_band, chrome, composer_band, esc, list_body, panel_grid, sidebar_band, table_body, thread_body, toolbar_band
import re
from .extract import app_root, columns_of, composer, controls, inline_hx, is_conversation, kind_of, panel_titles, sections, visible
from .markup import find_all, parse, shadow_tree, strip_code


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

    # A conversation surface is declared in static markup; its scripts only
    # fill it, so it is never replaced by recovered fragments. The wrapper may
    # hold the thread rather than being the thread, so descendants are checked.
    # Only the surface itself counts: an app that merely embeds a chat panel
    # (analytics) is not a conversation app.
    thread_nodes = ([body] if is_conversation(body) else []) + \
        [k for k in body.kids if is_conversation(k)]
    is_thread = bool(thread_nodes)
    if is_thread:
        body = thread_nodes[0]

    if not is_thread and shadow.kids and not columns and len(names) < 2:
        # A static shell that declares fewer than two named sections is not the
        # whole interface: several apps build their real lists, folders and
        # tables in script strings (the mail folders, the fraud and tax
        # columns). That recovered markup is used instead of the bare shell.
        body = shadow
        columns = columns_of(body)
        body_actions = controls(body, labels)
        names = sections(body, labels)

    body_y = y + 14
    avail = bottom - body_y - PAD

    # A dashboard is a set of sibling titled panels; master-detail is a list
    # beside a selected record. Only the former has several titled panels.
    tiles = panel_titles(body, labels)
    panes = [k for k in body.kids if visible(k)
             and re.search(r"(list|panel|detail|column|board|pane)", k.cls(),
                           re.I)]

    # A conversation surface is a message thread with a composer beneath it.
    if is_thread and not columns:
        placeholder = composer(root, labels)
        send = bool(find_all(root, lambda n: n.tag == "button"
                             and n.attrs.get("id") == "sendBtn"))
        thread_h = avail - (58 if placeholder else 0)
        parts.append(thread_body(cx + PAD, body_y, cw - 32, thread_h))
        if placeholder:
            parts.append(composer_band(placeholder, cx + PAD,
                                       body_y + thread_h + 14, cw - 32, send))
    elif len(tiles) >= 2 and not columns:
        parts.append(panel_grid(tiles, cx + PAD, body_y, cw - 32, avail))
    elif len(panes) >= 2 and not columns:
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
