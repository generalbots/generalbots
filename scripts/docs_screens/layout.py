"""The drawing primitives: canvas geometry, styles, bands and bodies."""


W, H = 900, 600


FX, FY, FW, FH = 30, 40, 840, 530


TITLEBAR = 44


PAD = 16


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


def thread_body(x, y, w, h):
    """Conversation layout: alternating message bubbles, sender on the right."""
    out, row = [], 58
    for i in range(max(1, int(h // row))):
        ry = y + i * row
        if ry + 44 > y + h:
            break
        outgoing = i % 3 == 1
        bw = w * (0.42 if outgoing else 0.5)
        bx = x + w - bw if outgoing else x
        cls = "chip-active" if outgoing else "row"
        out.append(f'  <rect x="{bx:.0f}" y="{ry:.0f}" width="{bw:.0f}" '
                   f'height="44" rx="10" class="{cls}"/>')
        for k in range(2):
            bar = (bw - 28) * (0.86 - 0.3 * k)
            out.append(f'  <rect x="{bx + 14:.0f}" y="{ry + 13 + k * 13:.0f}" '
                       f'width="{bar:.0f}" height="7" rx="3.5" class="bar"/>')
    return "\n".join(out)


def composer_band(text, x, y, w, send):
    """The message composer, labelled with its own shipped placeholder."""
    out = [f'  <rect x="{x}" y="{y}" width="{w}" height="40" rx="20" '
           f'class="row"/>']
    if text:
        out.append(f'  <text x="{x + 18}" y="{y + 25}" font-size="12.5" '
                   f'class="muted-text">{esc(text)}</text>')
    if send:
        out.append(f'  <rect x="{x + w - 38}" y="{y + 6}" width="28" '
                   f'height="28" rx="14" class="chip-active"/>')
    return "\n".join(out)


def panel_grid(titles, x, y, w, h):
    """Dashboard layout: sibling titled panels, two per row."""
    out, cols = [], 2 if len(titles) > 1 else 1
    gap = 16
    pw = (w - gap * (cols - 1)) / cols
    rows = max(1, (min(len(titles), 4) + cols - 1) // cols)
    ph = min((h - gap * (rows - 1)) / rows, 170)
    for i, name in enumerate(titles[:4]):
        px = x + (i % cols) * (pw + gap)
        py = y + (i // cols) * (ph + gap)
        out.append(f'  <rect x="{px:.0f}" y="{py:.0f}" width="{pw:.0f}" '
                   f'height="{ph:.0f}" rx="8" class="row"/>')
        out.append(f'  <text x="{px + 14:.0f}" y="{py + 25:.0f}" font-size="12.5" '
                   f'font-weight="600" class="main-text">{esc(name)}</text>')
        out.append(f'  <line x1="{px:.0f}" y1="{py + 36:.0f}" '
                   f'x2="{px + pw:.0f}" y2="{py + 36:.0f}" class="border"/>')
        for k in range(3):
            bw = (pw - 28) * (0.94 - 0.2 * k)
            out.append(f'  <rect x="{px + 14:.0f}" y="{py + 48 + k * 15:.0f}" '
                       f'width="{bw:.0f}" height="8" rx="4" class="bar"/>')
    return "\n".join(out)


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
