#!/usr/bin/env python3
"""Replace the ASCII architecture diagrams with SVG.

The README diagrams were drawn with box-drawing characters, which only line
up in a monospace font and break at narrow widths. These are drawn as SVG so
they scale, and they use a transparent background with mid-tone strokes so
they read on both GitHub light and dark themes.

Palette matches the application icons: #8b949e for strokes and text, with a
single #7c3aed accent for the element that carries the point of the diagram.

Usage: python3 gb_diagrams.py <repo-root>
"""

import sys
from pathlib import Path
from xml.sax.saxutils import escape

TEXT = "#8b949e"
ACCENT = "#7c3aed"
FONT = "Inter, -apple-system, Segoe UI, Helvetica, Arial, sans-serif"


class Canvas:
    def __init__(self, w, h):
        self.w, self.h = w, h
        self.parts = []

    def box(self, x, y, w, h, lines, title=None, accent=False, rx=10, opacity=0.55):
        stroke = ACCENT if accent else TEXT
        op = "0.7" if accent else str(opacity)
        self.parts.append(
            f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" '
            f'fill="none" stroke="{stroke}" stroke-opacity="{op}" stroke-width="1.5"/>'
        )
        content = ([title] if title else []) + list(lines)
        size = 14 if title else 12
        lead = 18
        total = len(content) * lead
        start = y + (h - total) / 2 + size
        cx = x + w / 2
        for i, line in enumerate(content):
            weight = "600" if (title and i == 0) else "400"
            fill = stroke if (title and i == 0) else TEXT
            self.parts.append(
                f'<text x="{cx}" y="{start + i * lead}" font-family="{FONT}" '
                f'font-size="{size}" font-weight="{weight}" fill="{fill}" '
                f'text-anchor="middle">{escape(line)}</text>'
            )

    def label(self, x, y, text, size=12, weight="400", fill=TEXT, anchor="middle"):
        self.parts.append(
            f'<text x="{x}" y="{y}" font-family="{FONT}" font-size="{size}" '
            f'font-weight="{weight}" fill="{fill}" text-anchor="{anchor}">'
            f"{escape(text)}</text>"
        )

    def line(self, x1, y1, x2, y2, arrow=True, opacity="0.55"):
        marker = ' marker-end="url(#arrow)"' if arrow else ""
        self.parts.append(
            f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{TEXT}" '
            f'stroke-opacity="{opacity}" stroke-width="1.5"{marker}/>'
        )

    def path(self, d, arrow=True, opacity="0.55"):
        marker = ' marker-end="url(#arrow)"' if arrow else ""
        self.parts.append(
            f'<path d="{d}" fill="none" stroke="{TEXT}" '
            f'stroke-opacity="{opacity}" stroke-width="1.5"{marker}/>'
        )

    def render(self):
        defs = (
            '<defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" '
            'markerWidth="6" markerHeight="6" orient="auto-start-reverse">'
            f'<path d="M0 0 10 5 0 10z" fill="{TEXT}"/></marker></defs>'
        )
        body = "\n  ".join(self.parts)
        return (
            '<?xml version="1.0" encoding="UTF-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{self.w}" height="{self.h}" '
            f'viewBox="0 0 {self.w} {self.h}" role="img">\n  {defs}\n  {body}\n</svg>\n'
        )


def architecture():
    c = Canvas(960, 344)
    c.box(60, 8, 340, 44, ["Browser · WhatsApp · Teams · Telegram"])
    c.line(230, 52, 230, 108)

    c.box(60, 108, 340, 84, [":3000 · :4000 · :5000", "suite · cloud · login"],
          title="botui")
    c.box(560, 108, 340, 84, [":8080", "API · WebSocket · agents"],
          title="botserver", accent=True)
    c.line(400, 150, 556, 150)
    c.label(478, 142, "proxy", size=11)

    c.path("M730 192 L730 224", arrow=False)
    c.path("M195 224 L765 224", arrow=False)
    for cx in (195, 480, 765):
        c.line(cx, 224, cx, 258)

    c.box(60, 258, 270, 74, ["state & secrets"], title="PostgreSQL + Vault")
    c.box(345, 258, 270, 74, ["drive & files"], title="MinIO (S3)")
    c.box(630, 258, 270, 74, ["vectors & local LLM"], title="Qdrant + llama.cpp")
    return c.render()


def botserver_pipeline():
    c = Canvas(900, 470)
    c.box(250, 8, 400, 44, ["WebSocket · REST · WhatsApp · Teams · Telegram"])
    c.line(450, 52, 450, 84)

    c.box(250, 84, 400, 60, ["session · rate limits"],
          title="main_module/ws/handler.rs")
    c.line(450, 144, 450, 176)

    c.box(250, 176, 400, 60, ["once per session · suggestions · bot memory"],
          title="start.bas")
    c.path("M450 236 L450 262", arrow=False)
    c.path("M270 262 L630 262", arrow=False)
    c.line(270, 262, 270, 292)
    c.line(630, 262, 630, 292)

    c.box(110, 292, 320, 78, ["TOOL_EXEC", "runs .ast directly"],
          title="message_type = 6", accent=True)
    c.box(470, 292, 320, 78, ["USE KB → RAG → LLM", "streamed response"],
          title="everything else")
    c.label(270, 386, "no model call", size=11, fill=ACCENT)
    c.label(630, 386, "model call", size=11)

    c.path("M270 396 L270 408", arrow=False)
    c.path("M630 396 L630 408", arrow=False)
    c.path("M270 408 L630 408", arrow=False)
    c.line(450, 408, 450, 434)
    c.box(250, 434, 400, 32, ["response → client"], rx=8)
    return c.render()


def botapp_shell():
    c = Canvas(900, 210)
    c.label(255, 18, "botui (pure web)", size=13, weight="600")
    c.label(645, 18, "botapp (Tauri wrapper)", size=13, weight="600")

    c.box(90, 32, 330, 152,
          ["suite/", "minimal/", "shared/", "", "No Tauri dependencies"])
    c.box(480, 32, 330, 152,
          ["Loads botui's UI", "Injects app-only", "features via JS", "",
           "Tauri + native APIs"], accent=True)
    c.line(474, 108, 428, 108)
    c.label(451, 100, "", size=10)
    return c.render()


def botapp_ipc():
    c = Canvas(760, 288)
    c.box(230, 8, 300, 36, ["Native UI (HTML/CSS/JS)"], rx=8)
    c.line(380, 44, 380, 70)
    c.label(392, 62, "Tauri IPC (invoke)", size=11, anchor="start")
    c.box(230, 70, 300, 36, ["Rust #[tauri::command]"], rx=8)
    c.line(380, 106, 380, 132)
    c.label(392, 124, "HTTP (reqwest)", size=11, anchor="start")
    c.box(230, 132, 300, 36, ["botserver API"], rx=8, accent=True)
    c.line(380, 168, 380, 194)
    c.path("M380 194 L380 216", arrow=False)
    c.box(230, 216, 300, 36, ["Business logic + database"], rx=8)
    return c.render()


def botmodels():
    c = Canvas(900, 340)
    c.box(120, 16, 280, 76, ["(Rust)"], title="botserver", accent=True)
    c.box(500, 16, 280, 76, ["(Python)"], title="botmodels")
    c.line(404, 54, 496, 54)
    c.label(450, 44, "HTTPS", size=11)

    left = ["BASIC keywords", "IMAGE · VIDEO", "AUDIO · SEE"]
    right = ["AI models", "Stable Diffusion", "Zeroscope · TTS · BLIP2"]
    for i, (l, r) in enumerate(zip(left, right)):
        c.label(260, 128 + i * 20, l, size=12)
        c.label(640, 128 + i * 20, r, size=12)

    c.line(260, 196, 260, 244)
    c.line(640, 196, 640, 244)
    c.box(120, 244, 280, 74, ["config", ".csv"], title=None)
    c.box(500, 244, 280, 74, ["outputs", "(files)"], title=None)
    return c.render()


def bottest_bootstrap():
    c = Canvas(820, 350)
    c.box(210, 8, 400, 40, ["TestHarness::full() · E2E tests"], title=None)

    steps = [
        ("Allocate unique ports (15000+)", False),
        ("Create ./tmp/bottest-{uuid}/", False),
    ]
    y = 78
    c.line(410, 48, 410, y - 4)
    for text, _ in steps:
        c.box(210, y, 400, 36, [text], rx=8)
        c.line(410, y + 36, 410, y + 62)
        y += 62

    # branch 1: mock servers
    c.box(60, y, 300, 56, ["MockZitadel · MockLLM", "(wiremock)"],
          title="Start mock servers only")
    c.box(460, y, 300, 56, ["PostgreSQL · MinIO", "Redis (cache)"],
          title="botserver --stack-path")
    c.path(f"M410 {y - 26} L410 {y - 12}", arrow=False)
    c.path(f"M210 {y - 12} L610 {y - 12}", arrow=False)
    c.line(210, y - 12, 210, y)
    c.line(610, y - 12, 610, y)
    y += 84

    c.path(f"M210 {y - 28} L210 {y - 14}", arrow=False)
    c.path(f"M610 {y - 28} L610 {y - 14}", arrow=False)
    c.path(f"M210 {y - 14} L610 {y - 14}", arrow=False)
    c.line(410, y - 14, 410, y + 8)
    c.box(210, y + 8, 400, 36, ["Return TestContext"], rx=8)
    return c.render()


def botdevice_layers():
    c = Canvas(900, 410)
    levels = [
        ("LEVEL 3 · GSI",
         ["Custom Android AOSP", "Zero manufacturer apps",
          "GB boot animation, single launcher"]),
        ("LEVEL 2 · MAGISK MODULE",
         ["Original Android + Magisk", "Bloatware removed via overlay",
          "BotDevice as privileged system app"]),
        ("LEVEL 1 · DEBLOAT + APP",
         ["Original Android (Samsung/Huawei/Xiaomi)",
          "Bloatware removed via ADB (no root)",
          "BotDevice as default launcher"]),
    ]
    y = 12
    for title, lines in levels:
        c.box(30, y, 840, 92, lines, title=title, rx=8)
        y += 100

    c.box(30, y + 8, 840, 76,
          ["botui/ui/suite          Tauri Android           src/lib.rs",
           "(web interface)      (WebView + NDK)      (backend + hardware)"],
          title="BotDevice app (Tauri)", rx=8)
    return c.render()


DIAGRAMS = {
    # The platform-wide view lives in docs_platform_diagram.py; this file draws
    # the per-component details that the master diagram summarises.
    "diagram-botserver-pipeline.svg": botserver_pipeline,
    "diagram-botapp-shell.svg": botapp_shell,
    "diagram-botapp-ipc.svg": botapp_ipc,
    "diagram-botmodels.svg": botmodels,
    "diagram-bottest-bootstrap.svg": bottest_bootstrap,
    "diagram-botdevice-layers.svg": botdevice_layers,
}


def main():
    root = Path(sys.argv[1]).resolve()
    out = root / ".github" / "svg"
    out.mkdir(parents=True, exist_ok=True)
    for name, fn in DIAGRAMS.items():
        (out / name).write_text(fn(), encoding="utf-8")
        print(f"  {name}  ({(out / name).stat().st_size} bytes)")


if __name__ == "__main__":
    main()
