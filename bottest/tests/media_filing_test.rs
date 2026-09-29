//! Media-filing end-to-end contract (#1357).
//!
//! The media-filing flow is Telegram media → Drive `inbox/` → `[image]` /
//! `[document]` / `[voice]` / `[audio]` / `[video]` marker → LLM tool call →
//! `classify_media` → `media/{year}/{month}/{category}/`.
//!
//! This file pins the parts of that chain that can be verified with no network
//! and no running server: the shipped template files, the marker vocabulary the
//! prompt and the tool schema agree on, and the perception branch the script
//! takes for each media kind. Executing the BASIC itself needs the botbasic
//! compiler/DAG runtime plus a Drive stub, which the `bottemplates` unit tests
//! and the flag-gated dev run cover; here we assert the contract the runtime
//! resolves against, so a regression in the layout or the vocabulary fails the
//! build rather than only showing up once deployed.

use std::path::{Path, PathBuf};

/// The shipped template in the layout the Drive resolver expects
/// (`{bot}.gbai/{bot}.gbdialog`, see #1355).
fn media_filing_gbai() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bottemplates")
        .join("bots")
        .join("media-filing")
        .join("media-filing.gbai")
}

fn read(relative: &str) -> String {
    let path = media_filing_gbai().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} should be readable: {e}", path.display()))
}

/// The template must live at the path the runtime resolvers look up, or the
/// whole flow is unreachable (this is the layout bug fixed in #1355).
#[test]
fn template_ships_in_the_layout_the_resolvers_expect() {
    let gbai = media_filing_gbai();
    assert!(gbai.is_dir(), "missing {}", gbai.display());
    assert!(
        gbai.join("media-filing.gbdialog/classify_media.bas").is_file(),
        "missing the classify_media tool script"
    );
    assert!(
        gbai.join("media-filing.gbdialog/classify_media.mcp.json").is_file(),
        "missing the classify_media tool schema"
    );
    assert!(
        gbai.join("media-filing.gbot/PROMPT-TELEGRAM.md").is_file(),
        "missing the Telegram channel prompt"
    );
    assert!(
        gbai.join("media-filing.gbot/PROMPT-WHATSAPP.md").is_file(),
        "missing the WhatsApp channel prompt"
    );
    assert!(
        gbai.join("media-filing.gbdialog/start.bas").is_file(),
        "missing start.bas"
    );
}

/// The prompt, the marker vocabulary and the tool schema must agree: the model
/// can only call the tool with a marker the sender actually produces.
#[test]
fn markers_are_documented_in_the_prompt_and_the_tool_schema() {
    let prompt = read("media-filing.gbot/PROMPT-TELEGRAM.md");
    let schema = read("media-filing.gbdialog/classify_media.mcp.json");

    for marker in ["[image]", "[document]", "[voice]", "[audio]", "[video]"] {
        assert!(
            prompt.contains(marker),
            "PROMPT-TELEGRAM.md does not mention the {marker} marker"
        );
        assert!(
            schema.contains(marker),
            "classify_media.mcp.json does not mention the {marker} marker"
        );
    }
}

/// WhatsApp sends audio (voice notes included) and stickers, so its prompt must
/// document the same closed filing policy on its own marker set, with the one
/// media item = one `classify_media` call rule.
#[test]
fn whatsapp_prompt_documents_its_own_markers_and_filing_policy() {
    let prompt = read("media-filing.gbot/PROMPT-WHATSAPP.md");
    let schema = read("media-filing.gbdialog/classify_media.mcp.json");

    for marker in ["[image]", "[document]", "[audio]", "[video]", "[sticker]"] {
        assert!(
            prompt.contains(marker),
            "PROMPT-WHATSAPP.md does not mention the {marker} marker"
        );
        assert!(
            schema.contains(marker),
            "classify_media.mcp.json does not mention the {marker} marker"
        );
    }

    assert!(
        prompt.contains("classify_media"),
        "PROMPT-WHATSAPP.md must tell the model to call classify_media"
    );
    assert!(
        prompt.contains("Um item de mídia = uma chamada de ferramenta"),
        "PROMPT-WHATSAPP.md must enforce one media item = one tool call"
    );
}

/// The closed taxonomy keeps a model answer from creating an arbitrary folder.
#[test]
fn taxonomy_stays_closed_and_covers_audio_video() {
    let script = read("media-filing.gbdialog/classify_media.bas");
    let taxonomy_line = script
        .lines()
        .find(|line| line.trim_start().starts_with("TAXONOMY = "))
        .expect("classify_media.bas must declare the closed TAXONOMY");

    for category in [
        "invoice",
        "receipt",
        "contract",
        "identity",
        "report",
        "audio",
        "video",
        "unsorted",
    ] {
        assert!(
            taxonomy_line.contains(category),
            "closed taxonomy entry '{category}' is missing from: {taxonomy_line}"
        );
    }
}

/// Each media kind must reach the perception model that can actually read it;
/// audio and video are binary and must never reach the document extractor.
#[test]
fn each_media_kind_routes_to_its_own_perception_model() {
    let script = read("media-filing.gbdialog/classify_media.bas");

    assert!(
        script.contains("DESCRIBE IMAGE path"),
        "images must be perceived by the vision model"
    );
    assert!(
        script.contains("SPEECH TO TEXT path"),
        "audio must be transcribed by the speech-to-text model"
    );
    assert!(
        script.contains("DESCRIBE VIDEO path"),
        "video must be described by the video model"
    );
    assert!(
        script.contains("GET path"),
        "documents must be read through text extraction"
    );

    // The audio branch must be selected by extension before the document
    // fallback, or a voice note is handed to a text extractor and fails.
    let audio_guard = script.find("is_audio = 1").expect("audio kind detection is missing");
    let document_branch = script
        .find("content = GET path")
        .expect("document perception branch is missing");
    assert!(
        audio_guard < document_branch,
        "audio detection must precede the document branch"
    );

    // Binary media must be recognised by extension, not routed by accident.
    for extension in [".ogg", ".mp3", ".m4a", ".wav", ".mp4", ".mov", ".webm"] {
        assert!(
            script.contains(extension),
            "extension '{extension}' is not recognised by classify_media.bas"
        );
    }
}

/// The audit trail and the filing step must stay in place so a filed item is
/// reproducible and a real Drive failure still reaches the caller.
#[test]
fn filing_writes_the_audit_trail_and_reports_the_category() {
    let script = read("media-filing.gbdialog/classify_media.bas");

    assert!(
        script.contains("MOVE path, destination"),
        "the item is never moved out of inbox/"
    );
    assert!(
        script.contains("\"media/\" + STR(today.year)"),
        "filing must use the media/{{year}}/{{month}}/{{category}} layout"
    );
    for key in ["category=", "kind=", "path=", "caption=", "perception="] {
        assert!(
            script.contains(key),
            "the .meta.txt audit trail is missing the '{key}' field"
        );
    }
    assert!(
        script.contains("ON ERROR RESUME NEXT") && script.contains("ON ERROR GOTO 0"),
        "perception must degrade but filing must still surface real errors"
    );
    assert!(
        script.contains("category = \"unsorted\""),
        "classification must default to 'unsorted'"
    );
}

/// The tool must subscribe itself to the channel event: without the trigger an
/// upload the model never acts on stays in `inbox/` forever (observed on
/// beiner in production, 2026-09). The subscription is registered by the
/// compiler consuming this line at design time (botbasic_compiler ON EVENT).
#[test]
fn the_tool_subscribes_to_the_media_uploaded_event() {
    let script = read("media-filing.gbdialog/classify_media.bas");

    assert!(
        script.contains("ON EVENT \"media_uploaded\""),
        "classify_media.bas must declare ON EVENT \"media_uploaded\" so uploads file deterministically"
    );
}

/// The classifier only sees text: without the media kind a voice note captioned
/// "gravacao de teste" read as a generic recording and filed under "video"
/// (observed in production, 2026-09). The script derives the kind from the
/// extension and must state it in the prompt.
#[test]
fn the_classification_prompt_carries_the_media_kind() {
    let script = read("media-filing.gbdialog/classify_media.bas");

    assert!(
        script.contains("Tipo de midia: \" + kind"),
        "the prompt content must include the media kind derived from the extension"
    );
    assert!(
        script.contains("audio ou gravacao de voz sem categoria melhor = audio"),
        "the decision guide must map generic voice recordings to audio, not video"
    );
}

/// The event trigger and the model's own tool call race on the same upload;
/// the runner that arrives second must exit quietly instead of surfacing a
/// MOVE failure to the user.
#[test]
fn filing_is_idempotent_against_the_event_and_tool_call_race() {
    let script = read("media-filing.gbdialog/classify_media.bas");

    let idempotency = script
        .find("4.1 Idempotency")
        .expect("the idempotency guard section is missing");
    let guarded_move = script
        .find("MOVE path, destination\nIF ERROR THEN")
        .expect("MOVE must run trapped so a lost race exits quietly");
    assert!(
        idempotency < guarded_move,
        "the idempotency guard must precede the trapped MOVE"
    );
    assert!(
        script.contains("TALK \"Esse arquivo ja foi arquivado.\""),
        "the losing runner must tell the user the item is already filed, not error"
    );
    assert!(
        script.contains("    RETURN\n"),
        "the losing runner must RETURN early, not report an error"
    );
}
