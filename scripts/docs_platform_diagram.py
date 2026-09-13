#!/usr/bin/env python3
"""Draw the whole General Bots platform as one dense master diagram.

Every label here is grounded in the repository: crate names come from
botserver/crates, service names from INFRA.md, the pipeline stages from
src/core/bot/pipeline, and the keyword count from get_all_keywords().

Layout is banded top to bottom so a diagram this size stays navigable:
clients -> edge -> front ends -> core runtime -> AI -> data -> tenancy -> ops.

Fonts are oversized relative to the canvas because GitHub scales a README
image down to the column width, and the reader can click through to the
full-size SVG.

Usage: python3 gb_master.py <repo-root>
"""

import os
import sys
from pathlib import Path
from xml.sax.saxutils import escape

TEXT = "#8b949e"
DIM = "#6e7781"
ACCENT = "#7c3aed"
FONT = "Inter, -apple-system, Segoe UI, Helvetica, Arial, sans-serif"

W = 1500
M = 26
INNER = W - 2 * M

# Font sizes deliberately large: the README scales this down.
S_TITLE, S_SUB, S_BAND, S_NAME, S_LINE, S_TINY = 30, 15, 19, 16, 13, 12
# Average glyph advance as a fraction of font size, for the overflow check.
ADV = 0.56

issues = []


def est(text, size):
    return len(text) * size * ADV


class Canvas:
    def __init__(self, w):
        self.w = w
        self.parts = []
        self.y = 0

    # ---------- primitives ----------
    def rect(self, x, y, w, h, stroke, op=0.5, rx=10, sw=1.5, fill="none"):
        self.parts.append(
            f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" fill="{fill}" '
            f'stroke="{stroke}" stroke-opacity="{op}" stroke-width="{sw}"/>'
        )

    def text(self, x, y, s, size=S_LINE, weight="400", fill=TEXT, anchor="middle", limit=None):
        if limit and est(s, size) > limit:
            issues.append(f"OVERFLOW ({est(s, size):.0f}>{limit}) [{size}px] {s!r}")
        self.parts.append(
            f'<text x="{x}" y="{y}" font-family="{FONT}" font-size="{size}" '
            f'font-weight="{weight}" fill="{fill}" text-anchor="{anchor}">{escape(s)}</text>'
        )

    def line(self, x1, y1, x2, y2, arrow=True, op=0.5):
        m = ' marker-end="url(#a)"' if arrow else ""
        self.parts.append(
            f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{TEXT}" '
            f'stroke-opacity="{op}" stroke-width="1.6"{m}/>'
        )

    def path(self, d, arrow=True, op=0.5):
        m = ' marker-end="url(#a)"' if arrow else ""
        self.parts.append(
            f'<path d="{d}" fill="none" stroke="{TEXT}" stroke-opacity="{op}" '
            f'stroke-width="1.6"{m}/>'
        )

    # ---------- layout helpers ----------
    def band(self, h, title, hint=""):
        """A full-width band. Returns the y for content, sets cursor past it."""
        y = self.y
        self.rect(M, y, INNER, h, TEXT, op=0.32, rx=14, sw=1.5)
        self.parts.append(
            f'<rect x="{M}" y="{y}" width="4" height="{h}" rx="2" fill="{ACCENT}" fill-opacity="0.55"/>'
        )
        self.text(M + 22, y + 32, title, size=S_BAND, weight="700", fill=ACCENT, anchor="start")
        if hint:
            self.text(W - M - 22, y + 32, hint, size=S_TINY, fill=DIM, anchor="end")
        self.y = y + h + 16
        self._bottom = y + h
        return y + 52

    def seal(self, end_y, what):
        """Warn when a band's content spills past its own frame."""
        if end_y > self._bottom + 1:
            issues.append(
                f"BAND OVERFLOW in {what!r}: content ends {end_y - self._bottom:.0f}px "
                f"past the band bottom"
            )

    def grid(self, y, items, cols, h=74, x0=M + 22, width=INNER - 44, gap=12):
        """Lay chips out in a grid. items = [(name, sub)] with sub optional."""
        cw = (width - gap * (cols - 1)) / cols
        for i, item in enumerate(items):
            name, sub = item if isinstance(item, tuple) else (item, None)
            cx = x0 + (i % cols) * (cw + gap)
            cy = y + (i // cols) * (h + 10)
            self.rect(cx, cy, cw, h, TEXT, op=0.45, rx=9)
            inner = cw - 18
            if sub:
                self.text(cx + cw / 2, cy + h / 2 - 5, name, size=S_NAME, weight="600", limit=inner)
                self.text(cx + cw / 2, cy + h / 2 + 17, sub, size=S_LINE, fill=DIM, limit=inner)
            else:
                self.text(cx + cw / 2, cy + h / 2 + 5, name, size=S_NAME, weight="600", limit=inner)
        return y + ((len(items) + cols - 1) // cols) * (h + 10)

    def chain(self, y, nodes, h=68, x0=M + 22, width=INNER - 44, accent_last=True):
        """A left-to-right pipeline of boxes joined by arrows."""
        n = len(nodes)
        gap = 34
        bw = (width - gap * (n - 1)) / n
        for i, (name, sub) in enumerate(nodes):
            x = x0 + i * (bw + gap)
            last = accent_last and i == n - 1
            self.rect(x, y, bw, h, ACCENT if last else TEXT, op=0.7 if last else 0.5, rx=9)
            self.text(x + bw / 2, y + h / 2 - 4, name, size=S_NAME, weight="600",
                      fill=ACCENT if last else TEXT, limit=bw - 16)
            if sub:
                self.text(x + bw / 2, y + h / 2 + 17, sub, size=S_TINY, fill=DIM, limit=bw - 16)
            if i < n - 1:
                self.line(x + bw, y + h / 2, x + bw + gap, y + h / 2)
        return y + h

    def render(self):
        defs = (
            '<defs><marker id="a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" '
            f'markerHeight="7" orient="auto-start-reverse"><path d="M0 0 10 5 0 10z" fill="{TEXT}"/></marker></defs>'
        )
        body = "\n  ".join(self.parts)
        return (
            '<?xml version="1.0" encoding="UTF-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{self.w}" height="{self.y - 6}" '
            f'viewBox="0 0 {self.w} {self.y - 6}" role="img">\n  {defs}\n  {body}\n</svg>\n'
        )


# ---------------------------------------------------------------- crate groups
CRATE_GROUPS = [
    ("Core runtime", [
        "botapi", "botcore", "botcorebot", "botcoredirectory", "botcoreoauth",
        "botcorepkg", "botcoresecrets", "botcoresession", "botdatabase",
        "botsettings", "botuifragments",
    ]),
    ("BASIC engine", [
        "botbasic_types", "botbasic_core", "botbasic_compiler", "botbasic_data",
        "botbasic_system", "botbasic_comms", "botbasic_ai",
    ]),
    ("AI & knowledge", [
        "botagent", "botllm", "botqdrant", "botmodelsbridge", "botmultimodal",
        "botvision", "botvideo", "botnvidia", "botmemory", "botlearn",
        "botsearch", "boteval", "botresearch", "botproviders", "botgl",
    ]),
    ("Channels", [
        "botchannels", "botchannels-core", "botchannelbindings", "botwhatsapp",
        "bottelegram", "botmsteams", "botemail", "botinstagram",
    ]),
    ("Business", [
        "botpeople", "botcontacts", "bottickets", "botattendance",
        "botattendant", "botbanking", "botbilling", "boterp", "botinventory",
        "botpos", "botretail", "botsales", "bottax", "botplan", "botproject",
        "botmarketplace", "botmarketing", "botproducts",
    ]),
    ("Productivity", [
        "botcalendar", "botdocs", "botsheet", "botsheet-core", "botslides",
        "botpaper", "botminutes", "botmeet", "bottasks", "botclock",
        "bottimeclock", "botcanvas", "botdesktop", "botplayer",
    ]),
    ("Security & compliance", [
        "botsecurity", "botsecurity-auth", "botsecurity-core",
        "botsecurity-protection", "botsecurity-crypto", "botcompliance",
        "botconsent", "botkyc", "botfraud", "botlegal", "botbiometry",
        "botbrowserpolicy",
    ]),
    ("Platform & operations", [
        "botcloud", "botworkspaces", "botdeployment", "botmonitoring",
        "botanalytics", "botdashboards", "botautomation", "botautotask",
        "botintegrations", "botconnectors", "botm365", "botsocial", "botgit",
        "bothr", "botdesigner", "boteditor", "botbrowser", "botdrive",
        "botbrazil", "botsources", "botsampledata", "bottemplates", "botitsm",
        "botmaintenance", "botweba", "botvibe", "bothandoff", "bottimeseries",
    ]),
]


def check_crates(root):
    d = Path(root) / "botserver" / "crates"
    actual = {p.name for p in d.iterdir() if p.is_dir()}
    listed = [c for _, members in CRATE_GROUPS for c in members]
    dupes = {c for c in listed if listed.count(c) > 1}
    missing = actual - set(listed)
    extra = set(listed) - actual
    if dupes:
        print(f"  !! duplicated in groups: {sorted(dupes)}")
    if missing:
        print(f"  !! crates not in any group: {sorted(missing)}")
    if extra:
        print(f"  !! listed but not on disk: {sorted(extra)}")
    return len(actual), len(listed), not (dupes or missing or extra)


def build(root):
    n_actual, n_listed, ok = check_crates(root)

    c = Canvas(W)

    # ---- title ----
    c.text(W / 2, M + 30, "General Bots — Platform Architecture", size=S_TITLE,
           weight="700", fill=TEXT, limit=INNER)
    c.text(W / 2, M + 56,
           "Self-hosted multi-agent platform · single Rust workspace · no cloud dependency",
           size=S_SUB, fill=DIM, limit=INNER)
    # phase legend
    phases = ["1 Request", "2 Process", "3 Decide", "4 Execute", "5 Respond"]
    lw = 150
    lx = (W - (len(phases) * lw + (len(phases) - 1) * 10)) / 2
    for i, p in enumerate(phases):
        x = lx + i * (lw + 10)
        c.rect(x, M + 70, lw, 30, ACCENT, op=0.5, rx=15)
        c.text(x + lw / 2, M + 90, p, size=S_TINY, fill=ACCENT, limit=lw - 14)
    c.y = M + 116

    # ---- band 1: clients & channels ----
    y = c.band(206, "Clients & Channels", "every surface is a first-class entry point")
    y = c.grid(y, [
        ("Web Chat", "botui suite · 80 apps"),
        ("botapp", "Tauri · desktop & mobile"),
        ("botdevice", "Android · AOSP / Magisk"),
        ("WhatsApp", "Business API"),
        ("MS Teams", "Bot Framework"),
        ("Telegram", "Bot API"),
        ("Instagram", "messaging"),
        ("Email", "IMAP / SMTP · Stalwart"),
        ("Voice & Video", "LiveKit · meeting bots"),
        ("REST API", "Bearer · OAuth 2.0"),
        ("Webhooks", "inbound & outbound"),
        ("Subdomains", "{bot}.generalbots.org"),
    ], cols=6, h=62)
    c.seal(y, "Clients & Channels")

    # ---- band 2: edge ----
    y = c.band(118, "Edge", "TLS, routing, name resolution")
    c.grid(y, [
        ("Caddy", "TLS termination · reverse proxy"),
        ("CoreDNS", "zone per domain · wildcard {bot}"),
        ("Port map", "3000 suite · 4000 cloud · 5000 login · 8080 API/WS"),
    ], cols=3, h=54)
    c.seal(y, "Edge")

    # ---- band 3: front ends ----
    y = c.band(148, "botui — Front Ends  (Rust · Axum · one binary, three ports)")
    c.grid(y, [
        ("suite :3000", "desktop shell · window manager · 80 HTMX apps"),
        ("cloud :4000", "store · plans · offers · dashboard — no login"),
        ("login :5000", "the only auth surface: login, signup, SSO"),
    ], cols=3, h=68)
    c.seal(y, "botui")

    # ---- band 4: core runtime ----
    y = c.band(668, "botserver — Core Runtime  (Rust · Axum · 113 domain crates)")

    c.text(M + 22, y + 4, "Transport & surfaces", size=S_LINE, weight="600", anchor="start")
    y = c.grid(y + 12, [
        ("HTTP routes", "~20 route modules"),
        ("WebSocket /ws", "session · streaming"),
        ("Channel adapters", "per-channel inbound"),
        ("SSO & anonymous", "cloud + chat tokens"),
        ("Task progress WS", "long-running jobs"),
        ("Health & metrics", "/health · dashboards"),
    ], cols=6, h=54)

    c.text(M + 22, y + 26, "Message pipeline", size=S_LINE, weight="600", anchor="start")
    y = c.chain(y + 38, [
        ("channel_entry", "normalise"),
        ("consent_gate", "policy"),
        ("start.bas", "once / session"),
        ("kb · RAG", "rag_modes"),
        ("api_catalog", "RBAC filtered"),
        ("tool_exec", "type 6 · no LLM"),
        ("sink", "stream out"),
    ], h=62)
    c.text(M + 22, y + 20,
           "side hooks:  memory_hook  ·  mentions  ·  agent_vm_hook  ·  multimedia",
           size=S_TINY, fill=DIM, anchor="start")

    c.text(M + 22, y + 46, "Engines", size=S_LINE, weight="600", anchor="start")
    y = c.grid(y + 58, [
        ("BASIC interpreter", "Rhai · 124 keywords · 7 crates"),
        ("Drive compiler", ".bas → .ast at sync time"),
        ("App registry + ui_plan", "the LLM's UI automation surface"),
        ("API command catalog", "declarative, role-filtered"),
        ("AutoTask", "plans, schedules, executes"),
    ], cols=5, h=58)

    c.text(M + 22, y + 24, "Security rails", size=S_LINE, weight="600", anchor="start")
    y = c.grid(y + 36, [
        ("botsecurity-auth", "JWT · MFA · CSRF · RBAC · Zitadel · rate limiter"),
        ("botsecurity-core", "SafeCommand · sql_guard · path_guard · DLP · audit · headers"),
    ], cols=2, h=56)

    c.text(M + 22, y + 24, "113 domain crates, grouped", size=S_LINE, weight="600",
           anchor="start")
    def sample(members, budget=325):
        """A short, legible glimpse of a group: crate names minus the 'bot' prefix.

        Names are added only while the line still fits the chip, so a group of
        long names shows two instead of overflowing with three.
        """
        short = [m[3:] if m.startswith("bot") else m for m in members]
        kept = []
        for name in short:
            trial = " · ".join(kept + [name])
            tail = " · …" if len(short) > len(kept) + 1 else ""
            if est(trial + tail, S_LINE) <= budget:
                kept.append(name)
        return " · ".join(kept) + (" · …" if len(short) > len(kept) else "")

    c.grid(y + 36, [
        (f"{name} · {len(members)}", sample(members))
        for name, members in CRATE_GROUPS
    ], cols=4, h=52, gap=10)
    y += 36 + 2 * 62
    c.seal(y, "botserver")

    # ---- band 5: AI ----
    y = c.band(184, "AI Layer", "model-agnostic, with graceful degradation")
    c.grid(y, [
        ("botllm routing", "smart router · council · breaker"),
        ("Response cache", "semantic cache · episodic memory"),
        ("Quality", "hallucination detector · evaluation"),
        ("Providers", "Claude · Bedrock · Vertex · GLM · Kimi"),
        ("Local models", "llama.cpp · OpenAI-compatible"),
        ("RAG", "Qdrant · chunking · embeddings", ),
        ("botmodels", "Python: image, video, speech"),
        ("Bot memories", "bot memory · session context"),
    ], cols=4, h=60)
    c.seal(y, "AI Layer")

    # ---- band 6: data ----
    y = c.band(184, "Data & Platform Services", "each runs in its own container")
    c.grid(y, [
        ("PostgreSQL", "state · migrations · tables"),
        ("Vault", "per-bot secrets · LLM keys"),
        ("Valkey", "cache · sessions · limits"),
        ("MinIO (S3)", "drive: {bot}.gbai"),
        ("Zitadel", "OIDC identity provider"),
        ("Qdrant", "vectors, per tenant"),
        ("Stalwart", "mail server · DKIM"),
        ("LiveKit", "voice & video rooms"),
        ("NocoDB", "table editor"),
        ("Roundcube", "webmail"),
    ], cols=5, h=58)
    c.seal(y, "Data & Platform Services")

    # ---- band 7: tenancy ----
    y = c.band(172, "Tenancy & Drive Model", "configuration lives in the drive, not on disk")
    y = c.chain(y, [
        ("{org}.gborg", "tenant"),
        ("{workspace}.gbai", "branch"),
        ("{bot}", "agent"),
        ("{bot}.gbot", "LLM config → Vault"),
        ("{bot}.gbdialog", "scripts, tools, tables"),
        ("{bot}.gbkb", "knowledge base"),
        ("{bot}.gbdrive", "files & reports"),
    ], h=56)
    c.text(M + 22, y + 22,
           "drive_monitors watches MinIO, discovers tenants and bots, compiles .bas → .ast, registers tools  ·  "
           "LLM settings resolve per-bot → nil → global secret/gbo/llm",
           size=S_TINY, fill=DIM, anchor="start", limit=INNER - 44)
    c.seal(y + 30, "Tenancy & Drive Model")

    # ---- band 8: operations ----
    y = c.band(134, "Operations", "deploys itself: push to ALM, CI builds, CI deploys")
    c.grid(y + 6, [
        ("Forgejo :4747", "git server"),
        ("CI runner", "build · test · deploy"),
        ("systemd", "botserver · botui · services"),
        ("Backups", "snapshots + remote S3"),
        ("Monitoring", "health · alerts"),
    ], cols=5, h=54)
    c.seal(y + 6 + 54 + 10, "Operations")

    return c.render()


def main():
    root = Path(sys.argv[1]).resolve()
    svg = build(root)
    targets = [
        root / ".github" / "svg" / "diagram-platform.svg",
        root / "botbook" / "src" / "assets" / "platform-master.svg",
    ]
    for t in targets:
        t.parent.mkdir(parents=True, exist_ok=True)
        t.write_text(svg, encoding="utf-8")
        print(f"  {t.relative_to(root)}  ({len(svg)} bytes)")
    print(f"  crates: {len([c for _, m in CRATE_GROUPS for c in m])} grouped")
    if issues:
        print(f"\n  {len(issues)} layout warnings:")
        for i in issues:
            print("   ", i)
    else:
        print("  no layout warnings")


if __name__ == "__main__":
    main()
