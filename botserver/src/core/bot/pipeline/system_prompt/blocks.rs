use crate::core::bot::pipeline;

/// The locale the client announced in its capability handshake, or `en`.
///
/// One resolver for every prompt block that needs it, so the capability block
/// and the response-language rule can never disagree about the locale.
#[must_use]
pub fn client_locale(client_caps: Option<&serde_json::Value>) -> String {
    client_caps
        .unwrap_or(&serde_json::Value::Null)
        .get("locale")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|locale| !locale.is_empty())
        .unwrap_or("en")
        .to_string()
}

/// #2096 — "answer in the user's language", in the shared builder.
///
/// Before this block existed the rule lived only in individual bot templates:
/// a bot shipped without that paragraph answered in the language its prompt was
/// authored in (English), whatever the user typed, and nothing in the platform
/// caught it. The block is language-agnostic on purpose — it names no locale —
/// so it works for all eight shipped locales and for a bot that serves users in
/// several of them at once.
#[must_use]
pub fn response_language_block(locale: &str) -> String {
    let locale = locale.trim();
    if locale.is_empty() {
        return [
            "RESPONSE LANGUAGE".to_string(),
            "- Always answer in the user's language: reply in the language of the user's message, not the language this prompt is written in.".to_string(),
            "- When the user's message is too short to tell (a number, a link, a single command), infer the language from context.".to_string(),
            "- Never translate code, identifiers, file paths, shell commands, SQL, API field names or URLs.".to_string(),
            "- Follow the user's explicit request to switch language for the rest of the conversation, and keep it until they switch again.".to_string(),
            "- Format dates, numbers and currency for that locale instead of assuming MM/DD/YYYY or `1,234.56`.".to_string(),
        ]
        .join("\n");
    }
    [
        "RESPONSE LANGUAGE".to_string(),
        format!(
            "- Always answer in the user's language (client locale hint: {locale}): reply in the language of the user's message, not the language this prompt is written in."
        ),
        "- When the user's message is too short to tell (a number, a link, a single command), use the client locale hint above.".to_string(),
        "- Never translate code, identifiers, file paths, shell commands, SQL, API field names or URLs.".to_string(),
        "- Follow the user's explicit request to switch language for the rest of the conversation, and keep it until they switch again.".to_string(),
        "- Format dates, numbers and currency for that locale instead of assuming MM/DD/YYYY or `1,234.56`.".to_string(),
    ]
    .join("\n")
}

/// #1939 — "ask, done" delivery policy: produce the artifact the user asked
/// for, in that format, using the open-source ecosystem instead of a
/// hand-rolled substitute. The safety envelope is unchanged: this text only
/// shapes the plan, every execution still passes the command/path/budget
/// guards.
///
/// #2562 — lines that reference the workspace sandbox or web search are
/// omitted when the corresponding feature (`agent-vm`, `research`/`web-search`)
/// is not compiled in, so the model never receives instructions it cannot act on.
#[must_use]
pub fn delivery_policy_block() -> String {
    let has_vm = cfg!(feature = "agent-vm");
    let has_web = cfg!(feature = "research") || cfg!(feature = "web-search");

    let mut lines = Vec::with_capacity(6);
    lines.push("DELIVERY POLICY (ask, done)");
    lines.push(
        "- When the user names a deliverable or file type (pdf, pptx, xlsx, csv, png, zip, .fig, code, \u{2026}), aim for that exact artifact and leave it where this client can fetch it.",
    );
    if has_web {
        lines.push(
            "- Prefer a maintained open-source library or command-line tool that already produces that format. If you are unsure which one is current, search the web first instead of guessing from memory, then use it.",
        );
    } else {
        lines.push(
            "- Prefer a maintained open-source library or command-line tool that already produces that format.",
        );
    }
    if has_vm {
        lines.push(
            "- Install what the task needs in the sandbox, but keep installs, execution and file writes inside the existing limits (packages allowlist, time and size budgets, path and command guards).",
        );
    }
    lines.push(
        "- Never substitute a different format silently. If the exact format cannot be produced reliably, say so and offer the closest alternative that you can verify.",
    );
    lines.push(
        "- Deliver quietly: no plan-only replies, no \u{201c}here is how you would do it\u{201d} when the work can be done now.",
    );
    lines.join("\n")
}

/// #2562 — inject today's date so authored prompts receive it without
/// hard-coding and the model never has to guess.  ISO date, weekday and
/// server local time; the fallback prompt in `prompt.rs` already has the date,
/// so the block is kept short to avoid doubling it.
#[must_use]
fn today_block() -> String {
    let now = chrono::Local::now();
    format!(
        "Today is {} ({}).",
        now.format("%Y-%m-%d"),
        now.format("%A"),
    )
}

/// #1975 — canvas drawing is a *selective* subsystem prompt: it is appended
/// only on turns that ask for a drawing, so ordinary turns keep the base
/// prompt short. The generated `<svg>` becomes a chat artifact whose
/// "Open in Canvas" action imports it as editable shapes (#1708).
#[must_use]
pub fn canvas_drawing_block(user_text: &str) -> &'static str {
    if wants_drawing(user_text) {
        CANVAS_DRAWING_PROMPT
    } else {
        ""
    }
}

/// #2108 — also read by `llm::svg_contract` so the drawing contract is
/// enforced on the reply, not only requested in the prompt.
pub(crate) fn wants_drawing(text: &str) -> bool {
    let lowered = text.to_lowercase();
    const NEEDLES: &[&str] = &[
        "svg",
        "draw",
        "drawing",
        "illustration",
        "illustrate",
        "logo",
        "diagram",
        "flowchart",
        "desenhe",
        "desenho",
        "desenhar",
        "ilustração",
        "ilustracao",
        "ícone",
        "icone",
        "vetor",
        "vector art",
        "paint",
        "sketch",
        // Scene requests ("a building with a glass sky, a whole city") are
        // drawing turns even without the word "draw" (#2007).
        "skyline",
        "cityscape",
        "city",
        "building",
        "landscape",
        "cartoon",
        "scene",
        "picture",
        "cidade",
        "prédio",
        "predio",
        "paisagem",
        "cenário",
        "cenario",
    ];
    NEEDLES.iter().any(|needle| lowered.contains(needle))
}

const CANVAS_DRAWING_PROMPT: &str = "CANVAS DRAWING SUBSYSTEM (selective)\n- When the user asks to draw, sketch, illustrate, design a logo/icon, or diagram something, deliver an SVG instead of prose.\n- Reply with exactly one fenced ```svg block containing a complete, standalone `<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 512 512\">…</svg>`. No text before or after the block.\n- The client draws every element the moment it arrives, so emission order is the animation order: build the scene back to front and finish one top-level element per concept (sky, ground, building body, windows, …) before starting the next.\n- Never wrap the drawing in a single top-level `<g>` and never emit one giant path: a wrapper that only closes at the end hides the whole scene until the answer finishes. Use `<g>` only for a small, self-contained label on a few finished shapes.\n- Leave `<defs>`, gradients, filters, clip paths and masks until the shapes they affect exist, and put text labels last.\n- Use only inline attributes and self-contained shapes/paths/text; never reference external files, fonts, scripts or images.\n- Keep it readable at small sizes: clean geometry, meaningful groups, and a title element.\n- The chat renders the block as an artifact with an \"Open in Canvas\" button; Canvas imports the SVG as editable shapes, so the user can tweak it and save it to Drive as a .draw file.\n- If the user asks for a different file format, honor that format; this block only applies to drawing requests.";

/// Per-session capability block (#1678). The model must not offer a surface
/// this client did not announce.
#[must_use]
pub fn capability_block(channel: &str, client_caps: Option<&serde_json::Value>) -> String {
    let caps = client_caps.unwrap_or(&serde_json::Value::Null);
    let surfaces = caps
        .get("surfaces")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            if crate::core::bot::pipeline::event_sink::enabled() {
                "text, gb.basic.v1".to_string()
            } else {
                "text".to_string()
            }
        });
    let motion = caps
        .get("motion")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("full");
    let locale = client_locale(client_caps);
    let input = caps
        .get("input")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("text,file");
    let max_live = caps
        .get("max_live_apps")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(2);
    format!(
        "CLIENT CAPABILITIES\nchannel={channel}; surfaces={surfaces}; max_live_apps={max_live}; motion={motion}; locale={locale}; input={input}.\nOnly use a surface listed above. If the client lists only `text`, answer in prose and never claim a card was shown. Respect reduced motion (no decorative animation) and the locale."
    )
}

/// The structured-interaction surface is advertised only when the client can
/// receive run events; otherwise the model would offer cards that this build
/// cannot render.
pub(crate) fn with_structured_tools(prompt: String, channel: &str) -> String {
    if channel != "web" || !crate::core::bot::pipeline::event_sink::enabled() {
        return prompt;
    }
    format!(
        "{prompt}\n\n{}",
        crate::core::bot::pipeline::surface_tools::instructions()
    )
}

/// #2033 — whether the web prompt carries the two-step API contract.
///
/// Default on. Only an explicit opt-out disables it, because a deployment can
/// be a `desktop-local` binary that still serves REST to remote users.
pub(super) fn api_catalog_enabled(value: Option<&str>) -> bool {
    match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("0") | Some("false") | Some("off") | Some("no") => false,
        _ => true,
    }
}
