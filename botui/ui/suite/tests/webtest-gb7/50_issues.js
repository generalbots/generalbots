/**
 * webtest_gb7 — issue coverage, drive checks and size guards
 * Split from webtest_gb7.js for #1731; keeps the original test order.
 */
"use strict";

const {
  test,
  assert,
  readFile,
  loadScript,
  memoryStorage,
} = require("./00_harness");

// ═══════════════════════════════════════════════════════════════════
// ISSUE COVERAGE (one test per open issue)
// ═══════════════════════════════════════════════════════════════════

test("ISSUE #1632: stored Vibe pins are dropped by the launcher", function () {
  var storage = memoryStorage();
  storage.setItem("gb-pinned-launcher", JSON.stringify([
    { kind: "app", appId: "vibe" },
    { kind: "app", appId: "vibe-code" },
    { kind: "app", appId: "chat" },
  ]));
  var window = {};
  var context = loadScript("js/pinned-launcher.js", {
    window: window,
    localStorage: storage,
    document: { addEventListener: function () {}, getElementById: function () { return null; } },
    console: console,
  });
  var list = context.window.GBPinnedLauncher.list();
  assert(list.length === 1 && list[0].appId === "chat",
    "Vibe entries must be purged, got " + JSON.stringify(list));
});

test("ISSUE #1633: payload trailer reader and sidecar stack fallback exist", function () {
  var payload = readFile("../../../botapp/src/desktop/payload.rs");
  assert(payload.indexOf("GB7PAYLOAD!!") !== -1, "trailer magic missing");
  assert(payload.indexOf("payload_digest") !== -1, "checksum verification missing");
  assert(payload.indexOf("--extract-only") !== -1, "offline extraction flag missing");
  var sidecar = readFile("../../../botapp/src/desktop/sidecar.rs");
  assert(sidecar.indexOf("Self::stack_path().join(name)") !== -1,
    "sidecar does not fall back to the extracted stack");
});

test("ISSUE #1634: release workflow verifies the stable aliases", function () {
  var workflow = readFile("../../../.github/workflows/desktop-binaries.yml");
  assert(workflow.indexOf("Verify the stable download aliases") !== -1, "alias check missing");
  assert(workflow.indexOf("sha256sum -c SHA256SUMS-linux --ignore-missing") !== -1,
    "checksum verification missing");
  assert(workflow.indexOf("make_latest: true") !== -1, "latest release not pinned");
});

test("ISSUE #1635: web sources become structured citations", function () {
  var web = readFile("../../../botserver/src/core/bot/pipeline/web_context.rs");
  assert(web.indexOf("pub async fn search_and_inject") !== -1 &&
    web.indexOf("-> Vec<serde_json::Value>") !== -1, "search does not return citations");
  var stream = readFile("../../../botserver/src/core/bot/ws/stream.rs");
  assert(stream.indexOf("pending_citations") !== -1, "final frame does not drain citations");
  var exec = readFile("../../../botserver/src/core/bot/pipeline/exec.rs");
  assert(exec.indexOf("pending_citations") !== -1, "turn does not store citations");
});

test("ISSUE #1637: model tiers select real models and the picker is hydrated", function () {
  var llm = readFile("../../../botserver/src/core/bot/pipeline/enriched_llm.rs");
  assert(llm.indexOf("llm-model-{tier}") !== -1, "per-tier model key missing");
  var handlers = readFile("../../../botserver/src/main_module/routes/chat_handlers.rs");
  assert(handlers.indexOf("/api/chat/models") !== -1, "tier endpoint missing");
  var init = readFile("chat/chat-model-picker.js");
  assert(init.indexOf("/api/chat/models") !== -1, "picker does not hydrate from the endpoint");
  assert(init.indexOf("option.disabled") !== -1, "unconfigured tiers are not disabled");
});

test("ISSUE #1641: attachment content is extracted and injected", function () {
  var text = readFile("../../../botserver/src/core/bot/pipeline/attachment_text.rs");
  assert(text.indexOf("pub fn extract_excerpt") !== -1, "excerpt extractor missing");
  assert(text.indexOf("fn docx_text") !== -1, "DOCX reader missing");
  assert(text.indexOf("is_media") !== -1, "media honesty check missing");
  var exec = readFile("../../../botserver/src/core/bot/pipeline/exec.rs");
  assert(exec.indexOf("turn_hooks::ingest_attachment") !== -1, "attachment not ingested");
  assert(exec.indexOf("attachment_note") !== -1, "attachment content not injected");
});

test("ISSUE #1644: browser actions have a producer, consent and feedback", function () {
  var core = readFile("../../../botserver/src/core/computer_use.rs");
  assert(core.indexOf("propose_from_text") !== -1, "URL producer missing");
  assert(core.indexOf("take_completed_for_prompt") !== -1, "results not fed back");
  var routes = readFile("../../../botserver/src/main_module/routes/computer_use_routes.rs");
  assert(routes.indexOf("/approve") !== -1 && routes.indexOf("/execute") !== -1,
    "consent/execute endpoints missing");
  var client = readFile("chat/modules/60_computer_use.js");
  assert(client.indexOf("/execute") !== -1 && client.indexOf("case \"screenshot\"") === -1,
    "client still fakes screenshots or lacks execute");
});

test("ISSUE #1645: terminal consent is bound to an audit record", function () {
  var routes = readFile("../../../botserver/src/main_module/routes/computer_use_routes.rs");
  assert(routes.indexOf("body.action_id") !== -1, "execution not bound to an action id");
  assert(routes.indexOf("/api/terminal/audit") !== -1, "audit endpoint missing");
  var migration = readFile("../../../botserver/migrations/6.8.3-terminal-audit/up.sql");
  assert(migration.indexOf("CREATE TABLE IF NOT EXISTS terminal_audit") !== -1,
    "audit table missing");
  var client = readFile("chat/modules/61_terminal_use.js");
  assert(client.indexOf("/api/terminal/audit") !== -1, "audit panel not reachable");
});

test("ISSUE #1649: the published page is generated from the record", function () {
  var record = JSON.parse(readFile("../../../desktop/gb7/features.json"));
  var ids = record.features.map(function (f) { return f.id; });
  assert(ids.length === 14, "expected 14 features, got " + ids.length);
  assert(ids.indexOf("GB7-22") === -1 && ids.indexOf("GB7-24") === -1,
    "removed features still listed");
  record.features.forEach(function (feature) {
    assert(feature.evidence && feature.evidence.issue, feature.id + " has no issue link");
    if (feature.status === "shipped") {
      assert(feature.evidence.e2e, feature.id + " is shipped without E2E evidence");
      assert(feature.last_verified && feature.last_verified.date,
        feature.id + " has no verification date");
    }
  });
});

test("ISSUE #1650: the epic definition of done is recorded", function () {
  var record = JSON.parse(readFile("../../../desktop/gb7/features.json"));
  assert(record.epic && record.epic.issue === "#1650", "epic metadata missing");
  assert(Array.isArray(record.epic.definition_of_done) &&
    record.epic.definition_of_done.length >= 4, "definition of done missing");
  assert(record.epic.removed_from_scope.indexOf("GB7-22") !== -1,
    "removed scope not recorded");
});

test("SIZE: touched chat and pipeline files stay under the 450-line limit", function () {
  var files = [
    "chat/chat-init.js",
    "chat/chat-send.js",
    "chat/chat-session.js",
    "chat/chat-messages.js",
    "chat/chat-render.js",
    "chat/chat-artifacts.js",
    "../../../botserver/src/core/bot/pipeline/exec.rs",
    "../../../botserver/src/core/bot/pipeline/system_prompt.rs",
    "../../../botserver/src/core/bot/pipeline/start_gate.rs",
    "../../../botserver/src/core/bot/pipeline/turn_context.rs",
    "../../../botserver/src/core/bot/pipeline/agent_vm_context.rs",
    "../../../botserver/src/main_module/routes/computer_use_routes.rs"
  ];
  files.forEach(function (file) {
    var lines = readFile(file).split("\n").length;
    assert(lines <= 450, file + " has " + lines + " lines (max 450)");
  });
});

test("DRIVE: tabs are simplified to My Files, Shared, Connected and Public", function () {
  var html = readFile("drive/drive.html");
  var tabs = (html.match(/data-tab="([a-z]+)"/g) || []).map(function (entry) {
    return entry.split('"')[1];
  });
  assert(tabs.join(",") === "myfiles,shared,external,public",
    "unexpected tabs: " + tabs.join(","));
  assert(html.indexOf('data-tab="myfiles" id="tab-myfiles"') !== -1 &&
    html.indexOf("top-tab active") !== -1, "My Files must be the active tab");
  assert(html.indexOf("gbSyncStatusBtn") === -1, "offline sync button still present");
  var state = readFile("drive/modules/01_state.js");
  assert(state.indexOf('let currentTab = "myfiles";') !== -1, "default tab is not My Files");
});

test("DRIVE: share by email is wired to the server and the sender picker", function () {
  var modals = readFile("drive/modules/08_modals.js");
  assert(modals.indexOf('id="share-email-to"') !== -1, "recipient input missing");
  assert(modals.indexOf('id="share-email-sender"') !== -1, "sender picker missing");
  assert(modals.indexOf("/share/email/accounts") !== -1, "sender lookup not wired");
  assert(modals.indexOf('"/share/email"') !== -1, "email send not wired");
  assert(modals.indexOf("origin: window.location.origin") !== -1, "absolute link origin missing");
  var init = readFile("drive/modules/99_init.js");
  assert(init.indexOf("sendShareEmail: sendShareEmail") !== -1, "email action not exported");
  var routes = readFile("../../../botserver/src/main_module/routes/api_setup.rs");
  assert(routes.indexOf("/api/files/share/email") !== -1, "email route not registered");
  var handler = readFile("../../../botserver/crates/botdrive/src/drive_handlers/share_email.rs");
  assert(handler.indexOf("botemail::oauth::resolve_credentials") !== -1,
    "connected account credentials not used");
  assert(handler.indexOf("get_email_config") !== -1, "native Stalwart fallback missing");
});

test("ISSUE #1635: sources are delivered after the typed stream", function () {
  var exec = readFile("../../../botserver/src/core/bot/pipeline/exec.rs");
  assert(exec.indexOf('"type": "citations"') !== -1, "post-stream citations frame missing");
  assert(exec.indexOf("send_raw_json") !== -1, "citations frame not sent to the client");
  var client = readFile("chat/modules/30_citations.js");
  assert(client.indexOf('d.type === "citations"') !== -1, "client ignores the citations frame");
  assert(client.indexOf("GBRenderCitations(last") !== -1, "chips are not attached to the last message");
});

test("ISSUE #1645: terminal output is read from the recorded action", function () {
  var src = readFile("chat/modules/61_terminal_use.js");
  assert(src.indexOf("response.result || response") !== -1,
    "terminal client does not unwrap the recorded action result");
  assert(src.indexOf("result.stdout") !== -1, "stdout not rendered");
});

test("ISSUE #1638: search walks nested keys, not only top-level folders", function () {
  var src = readFile("../../../botserver/crates/botdrive/src/drive_handlers/search_files.rs");
  assert(src.indexOf("key.rsplit('/').next()") !== -1, "nested key names are not inspected");
  assert(src.indexOf("build_file_list_items(&prefix") === -1, "the collapsing listing helper is still used");
});
