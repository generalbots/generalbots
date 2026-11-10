//! Chat system-prompt assembly (#1635/#1637 follow-up).
//!
//! Split out of `exec.rs` so the pipeline entry point stays readable and every
//! pipeline file honors the repository's 450-line limit. Behaviour is
//! unchanged: the channel prompt, active switchers, persisted user memory, the
//! role resolved from RBAC and the channel-specific action surface are
//! composed in the same order as before.

use std::sync::Arc;

use uuid::Uuid;

use botcore::shared::state::AppState;

/// Builds the system prompt for one turn.
///
/// * `channel` is the sink's channel type (`web`, `whatsapp`, …).
/// * `active_switchers` are the composer's active switcher ids; they append
///   their own instructions after the base prompt.
pub(super) fn build(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    bot_name: &str,
    channel: &str,
    active_switchers: &[String],
    client_caps: Option<&serde_json::Value>,
) -> String {
    let base_system_prompt =
        crate::core::bot::ws::message::load_system_prompt_for_channel(bot_name, channel);
    let system_prompt = if !active_switchers.is_empty() {
        let switcher_prompts = crate::basic::keywords::switcher::resolve_active_switchers(
            state.cache.as_ref(),
            &bot_uuid.to_string(),
            &session_id.to_string(),
            active_switchers,
        );
        if switcher_prompts.is_empty() {
            base_system_prompt
        } else {
            format!("{base_system_prompt}\n\n{switcher_prompts}")
        }
    } else {
        base_system_prompt
    };
    // GB7-16 — persisted user preferences are part of the system prompt, so a
    // fresh conversation starts with what the user already taught the bot.
    let memory_block =
        crate::core::user_memory::render_for_prompt(&state.conn, user_id, Some(bot_uuid));
    let system_prompt = if memory_block.is_empty() {
        system_prompt
    } else {
        format!("{system_prompt}{memory_block}")
    };
    // #1681 — installed Agent Skills packages are advertised as guidance.
    #[cfg(feature = "sources")]
    let skills_block = match state.conn.get() {
        Ok(mut conn) => {
            botsources::handlers::sources_handlers::render_for_prompt(&mut conn, bot_uuid)
        }
        Err(error) => {
            log::warn!("skills prompt skipped: pool error: {error}");
            String::new()
        }
    };
    #[cfg(not(feature = "sources"))]
    let skills_block = String::new();
    let system_prompt = format!("{system_prompt}{skills_block}");
    let role = crate::security::user_role::resolve_user_role(&state.conn, user_id);
    // #2033 — the REST catalogue is a capability of the deployment, not of the
    // build flavour. A `desktop-local` binary served over the network (the
    // production bot) still answers REST requests, so suppressing the catalogue
    // there left the model unable to detect any app-data request. The default
    // is on; a single-user install that deliberately drives only the agent VM
    // can opt out with GB_PROMPT_API_CATALOG=0.
    let include_api_catalog = api_catalog_enabled(
        std::env::var("GB_PROMPT_API_CATALOG").ok().as_deref(),
    );
    let channel_prompt = if channel == "web" {
        if include_api_catalog {
            format!(
                "{system_prompt}\n\n{}\n\n{}",
                crate::core::bot::api_catalog::api_command_instructions(&role),
                crate::main_module::ui_plan::ui_automation_instructions(client_caps),
            )
        } else {
            system_prompt
        }
    } else if channel == "whatsapp" {
        format!(
            "{system_prompt}\n\n---\nThis conversation is on WhatsApp.\n\n{}",
            crate::core::bot::api_catalog::api_command_instructions(&role),
        )
    } else {
        format!(
            "{system_prompt}\n\n{}",
            crate::core::bot::api_catalog::api_command_instructions(&role),
        )
    };
    // #1939 — the delivery policy only makes sense where the model can act
    // (web/desktop); a channel without an action surface gets prose only.
    let delivery = if channel == "web" {
        delivery_policy_block()
    } else {
        String::new()
    };
    // #2096 — the response-language rule is a platform property: it is appended
    // here, after every channel-specific block, so a bot whose own `PROMPT.md`
    // omits it still answers in the user's language.
    let language = response_language_block(&client_locale(client_caps));
    let blocks = [capability_block(channel, client_caps), language, delivery];
    let appended = blocks
        .iter()
        .map(String::as_str)
        .filter(|block| !block.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{channel_prompt}\n\n{appended}")
}

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

/// #2096 — “answer in the user's language”, in the shared builder.
///
/// Before this block existed the rule lived only in individual bot templates:
/// a bot shipped without that paragraph answered in the language its prompt was
/// authored in (English), whatever the user typed, and nothing in the platform
/// caught it. The block is language-agnostic on purpose — it names no locale —
/// so it works for all eight shipped locales and for a bot that serves users in
/// several of them at once.
#[must_use]
pub fn response_language_block(locale: &str) -> String {
    let hint = if locale.trim().is_empty() {
        "the user's language".to_string()
    } else {
        format!("the user's language (client locale hint: {locale})")
    };
    [
        "RESPONSE LANGUAGE".to_string(),
        format!("- Always answer in {hint}: reply in the language of the user's message, not the language this prompt is written in."),
        "- When the user's message is too short to tell (a number, a link, a single command), use the client locale hint above.".to_string(),
        "- Never translate code, identifiers, file paths, shell commands, SQL, API field names or URLs.".to_string(),
        "- Follow the user's explicit request to switch language for the rest of the conversation, and keep it until they switch again.".to_string(),
        "- Format dates, numbers and currency for that locale instead of assuming MM/DD/YYYY or `1,234.56`.".to_string(),
    ]
    .join("\n")
}

/// #1939 — “ask, done” delivery policy: produce the artifact the user asked
/// for, in that format, using the open-source ecosystem instead of a
/// hand-rolled substitute. The safety envelope is unchanged: this text only
/// shapes the plan, every execution still passes the command/path/budget
/// guards.
#[must_use]
pub fn delivery_policy_block() -> String {
    [
        "DELIVERY POLICY (ask, done)",
        "- When the user names a deliverable or file type (pdf, pptx, xlsx, csv, png, zip, .fig, code, …), aim for that exact artifact and leave it where this client can fetch it.",
        "- Prefer a maintained open-source library or command-line tool that already produces that format. If you are unsure which one is current, search the web first instead of guessing from memory, then use it.",
        "- Install what the task needs in the sandbox, but keep installs, execution and file writes inside the existing limits (packages allowlist, time and size budgets, path and command guards).",
        "- Never substitute a different format silently. If the exact format cannot be produced reliably, say so after checking the available open-source paths and offer the closest alternative that you can verify.",
        "- Deliver quietly: no plan-only replies, no “here is how you would do it” when the work can be done now.",
    ]
    .join("\n")
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
        "svg", "draw", "drawing", "illustration", "illustrate", "logo", "diagram",
        "flowchart", "desenhe", "desenho", "desenhar", "ilustração", "ilustracao",
        "ícone", "icone", "vetor", "vector art", "paint", "sketch",
        // Scene requests ("a building with a glass sky, a whole city") are
        // drawing turns even without the word "draw" (#2007).
        "skyline", "cityscape", "city", "building", "landscape", "cartoon",
        "scene", "picture", "cidade", "prédio", "predio", "paisagem", "cenário",
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
            if super::event_sink::enabled() {
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
    if channel != "web" || !super::event_sink::enabled() {
        return prompt;
    }
    format!("{prompt}\n\n{}", super::surface_tools::instructions())
}

/// #2033 — whether the web prompt carries the two-step API contract.
///
/// Default on. Only an explicit opt-out disables it, because a deployment can
/// be a `desktop-local` binary that still serves REST to remote users.
fn api_catalog_enabled(value: Option<&str>) -> bool {
    match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("0") | Some("false") | Some("off") | Some("no") => false,
        _ => true,
    }
}

#[cfg(test)]
mod prompt_assembly_tests {
    use super::{
        canvas_drawing_block, capability_block, client_locale, delivery_policy_block,
        response_language_block, with_structured_tools,
    };

    #[test]
    fn capability_block_uses_the_client_defaults() {
        let block = capability_block("web", None);
        assert!(block.starts_with("CLIENT CAPABILITIES\nchannel=web; surfaces="));
        assert!(block.contains("max_live_apps=2; motion=full; locale=en; input=text,file."));
        assert!(block.contains("Only use a surface listed above."));
        // A null/non-object caps payload must not panic and keeps the defaults.
        assert!(capability_block("whatsapp", Some(&serde_json::Value::Null))
            .contains("channel=whatsapp;"));
    }

    #[test]
    fn capability_block_reports_only_the_announced_surfaces() {
        let caps = serde_json::json!({
            "surfaces": ["text", "gb.app.v1", 7, null],
            "motion": "reduced",
            "locale": "pt-BR",
            "input": "text",
            "max_live_apps": 4,
        });
        let block = capability_block("web", Some(&caps));
        assert!(block.contains("surfaces=text, gb.app.v1; max_live_apps=4"));
        assert!(block.contains("motion=reduced; locale=pt-BR; input=text."));
    }

    #[test]
    fn canvas_prompt_is_only_appended_for_drawing_turns() {
        for drawing in ["desenhe um gato", "DRAW A LOGO", "svg of a skyline"] {
            let block = canvas_drawing_block(drawing);
            assert!(block.starts_with("CANVAS DRAWING SUBSYSTEM"), "turn {drawing}");
            assert!(block.contains("Open in Canvas"));
        }
        for prose in ["qual é o faturamento do mês?", "send the invoice by email"] {
            assert_eq!(canvas_drawing_block(prose), "", "turn {prose}");
        }
    }

    #[test]
    fn delivery_policy_names_the_artifact_rule() {
        let block = delivery_policy_block();
        assert!(block.starts_with("DELIVERY POLICY (ask, done)"));
        assert!(block.contains("Never substitute a different format silently."));
        assert!(block.contains("Deliver quietly"));
    }

    #[test]
    fn client_locale_falls_back_to_english() {
        assert_eq!(client_locale(None), "en");
        assert_eq!(client_locale(Some(&serde_json::Value::Null)), "en");
        assert_eq!(client_locale(Some(&serde_json::json!({}))), "en");
        assert_eq!(
            client_locale(Some(&serde_json::json!({ "locale": "  " }))),
            "en"
        );
        assert_eq!(
            client_locale(Some(&serde_json::json!({ "locale": 7 }))),
            "en"
        );
    }

    #[test]
    fn client_locale_reads_the_announced_locale() {
        let caps = serde_json::json!({ "locale": " pt-BR " });
        assert_eq!(client_locale(Some(&caps)), "pt-BR");
        // The capability block and the language rule must never disagree.
        assert!(capability_block("web", Some(&caps)).contains("locale=pt-BR;"));
        assert!(response_language_block(&client_locale(Some(&caps)))
            .contains("client locale hint: pt-BR"));
    }

    #[test]
    fn response_language_block_states_the_rule_for_every_locale() {
        // #2096 — the rule is a platform property: it must name no locale of its
        // own, so all eight shipped locales get the same instruction.
        for locale in ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"] {
            let block = response_language_block(locale);
            assert!(block.starts_with("RESPONSE LANGUAGE"), "locale {locale}");
            assert!(
                block.contains("reply in the language of the user's message"),
                "locale {locale}"
            );
            assert!(
                block.contains(&format!("client locale hint: {locale}")),
                "locale {locale}"
            );
        }
        // An unknown/empty hint still produces a usable rule.
        let block = response_language_block("");
        assert!(block.contains("answer in the user's language"));
        assert!(!block.contains("client locale hint"));
    }

    #[test]
    fn response_language_block_protects_non_translatable_payloads() {
        let block = response_language_block("en");
        for kept in ["code", "file paths", "shell commands", "SQL", "URLs"] {
            assert!(block.contains(kept), "missing {kept}");
        }
        assert!(block.contains("MM/DD/YYYY"));
        assert!(block.contains("switch language"));
    }

    #[test]
    fn structured_tool_instructions_stay_off_non_web_channels() {
        // Only the web channel can render run events; every other channel gets
        // the prompt untouched regardless of the runtime feature flag.
        assert_eq!(
            with_structured_tools("base".to_string(), "whatsapp"),
            "base"
        );
    }
}

#[cfg(test)]
mod api_catalog_gate_tests {
    use super::api_catalog_enabled;

    #[test]
    fn catalogue_is_on_by_default_and_for_unset_values() {
        assert!(api_catalog_enabled(None));
        assert!(api_catalog_enabled(Some("")));
        assert!(api_catalog_enabled(Some("1")));
        assert!(api_catalog_enabled(Some("true")));
    }

    #[test]
    fn only_an_explicit_opt_out_hides_the_catalogue() {
        for value in ["0", "false", "off", "no", " FALSE "] {
            assert!(!api_catalog_enabled(Some(value)), "value {value}");
        }
    }
}
