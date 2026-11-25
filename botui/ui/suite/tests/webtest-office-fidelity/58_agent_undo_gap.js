/**
 * [KNOWN BUG] #2448 criterion 4 — source-backed guards for the agent-batch undo gap.
 *
 * Source audit (2026-11-25):
 *   - `editor.undo.create` is the ONLY undo verb the LLM can see, and it is
 *     DERIVED, not curated: commands_derived.rs maps POST -> "create" and builds
 *     `<app>.<noun>.<verb>` from the endpoint inventory path `/api/editor/undo`.
 *     No declared or bound verb undoes an agent batch.
 *   - `handle_undo` is a no-op over a registry recreated on every request
 *     (`editor_registry()` -> `Arc::new(Mutex::new(HashMap::new()))`): it pushes
 *     the posted content, pops it and returns an HTML blob. It has neither a
 *     batch identity nor before/after content.
 *   - every candidate history store (editor, Docs, Sheets, Slides) lacks batch
 *     ids and before/after content, and no migration defines an
 *     `agent_edit_batches` table with `batch_id` / `before_content` columns.
 *   - the safety rule a real batch undo MUST reuse already ships in
 *     botsheet/src/collaboration/undo.rs::can_apply — an undo may only revert
 *     the caller's own winning op and must never clobber another author.
 *   - botbook designer.md:278 records the gap explicitly.
 *
 * Each case asserts the CURRENT degraded shape and PASSES as a documented
 * KNOWN BUG. NON-VACUITY: every case first proves the shipped source exists and
 * carries its real function/struct/route markers, and the catalog and migration
 * scans prove they found entries. The "gap" assertions are then scoped absence
 * checks — word-boundary aware, so `batch_idx` cannot be mistaken for
 * `batch_id` — restricted to botserver source/migrations, so this test file can
 * never self-match. When a real typed agent-batch undo lands an asserted-in gap
 * disappears, its knownBug fn throws, the case becomes FAIL and this marker
 * must be removed and replaced by a runtime test. No user-facing string, no
 * server, no browser, no cargo.
 */
"use strict";

var h = require("./00_harness");
var fs = h.fs;
var path = h.path;

var COMMANDS_DIR = "botserver/src/apps/commands";
var VERB_TABLE_DIR = COMMANDS_DIR + "/verb_endpoints/table";
var DERIVED = "botserver/src/core/bot/commands_derived.rs";
var EDITOR_INVENTORY = "botserver/src/core/bot/endpoint_inventory_directory/editor.rs";
var EDITOR_ROUTES = "botserver/crates/boteditor/src/routes.rs";
var EDITOR_MOD = "botserver/crates/boteditor/src/handlers/mod.rs";
var EDITOR_EDITS = "botserver/crates/boteditor/src/handlers/edits.rs";
var DOCS_HISTORY = "botserver/crates/botdocs/src/handlers_api/history.rs";
var SHEET_UNDO = "botserver/crates/botsheet/src/collaboration/undo.rs";
var SLIDES_UNDO = "botui/ui/suite/slides/modules/09_undo.js";
var DESIGNER = "botbook/src/05-multi-agent/designer.md";
var MIGRATIONS = "botserver/migrations";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(
    h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath
  );
}

/** Exact, non-empty source span between two literal markers (end exclusive). */
function between(relPath, startMarker, endMarker) {
  var src = h.readRepo(relPath);
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + "> in " + relPath);
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + "> in " + relPath);
  h.assert(end > start, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}

/** Repo-relative files with the given extension, in deterministic order. */
function sourceFiles(relDir, ext) {
  var root = path.join(h.REPO_DIR, relDir);
  var found = [];
  (function walk(dir) {
    fs.readdirSync(dir, { withFileTypes: true })
      .sort(function (a, b) { return a.name < b.name ? -1 : a.name > b.name ? 1 : 0; })
      .forEach(function (entry) {
        var full = path.join(dir, entry.name);
        if (entry.isDirectory()) walk(full);
        else if (entry.name.slice(-ext.length) === ext) {
          found.push(path.relative(h.REPO_DIR, full).split(path.sep).join("/"));
        }
      });
  })(root);
  return found;
}

/** Verb names extracted from `cmd(app, name, ...)` / `bind(name, ...)` calls. */
function catalogNames(relDir, kind) {
  var re = kind === "cmd"
    ? /\bcmd\(\s*"[^"]+",\s*"([^"]+)"/g
    : /\bbind\(\s*"([^"]+)"/g;
  var names = [];
  sourceFiles(relDir, ".rs").forEach(function (relPath) {
    var src = h.readRepo(relPath);
    re.lastIndex = 0;
    var match = re.exec(src);
    while (match !== null) {
      names.push(match[1]);
      match = re.exec(src);
    }
  });
  return names;
}

/** Word-boundary absence — `batch_idx` must not satisfy a `batch_id` check. */
function absentWords(src, words, what) {
  words.forEach(function (word) {
    var re = new RegExp("\\b" + word + "\\b");
    h.assert(!re.test(src),
      (what || "gap") + " must still be absent: found whole word <" + word + "> :: <" + src.slice(0, 200) + ">");
  });
}

// 1. The only undo verb the LLM can see is the DERIVED text-editor one: POST
//    maps to "create", so /api/editor/undo becomes editor.undo.create. There is
//    no declared/bound agent-batch undo verb and the handler is a per-request
//    HTML no-op. This case flips when a typed batch-undo verb is added.
h.knownBug("only the derived editor.undo.create text-editor verb exists; no agent-batch undo verb is declared or bound", function () {
  // Non-vacuity: prove the derivation that yields the sole undo verb.
  contains(DERIVED, "fn verb_for_method(method: &str) -> &'static str {",
    "HTTP-method -> verb derivation ships");
  contains(DERIVED, '"POST" => "create",', "POST maps to the create verb");
  contains(DERIVED, "fn command_name_for(app: &str, method: &str, path: &str) -> String {",
    "verb-name builder ships");
  contains(DERIVED, 'format!("{app}.{noun}.{}", verb_for_method(method))',
    "derived name format is <app>.<noun>.<verb>");
  contains(DERIVED, "pub fn derived_commands() -> Vec<DerivedCommand> {", "derived surface ships");
  var derive = between(DERIVED, "fn verb_for_method(", "#[cfg(test)]");
  h.assert(derive.indexOf("all_routes()") !== -1,
    "derived verbs come from the endpoint inventory :: <" + derive.slice(0, 300) + ">");

  // Non-vacuity: /api/editor/undo really is the registered, POST-mounted editor route.
  contains(EDITOR_INVENTORY, "// #2448 — one command undoes the last agent batch",
    "the editor undo inventory comment ships");
  contains(EDITOR_INVENTORY, 'path: "/api/editor/undo"', "editor undo is in the inventory");
  contains(EDITOR_ROUTES, '.route("/api/editor/undo", post(handlers::handle_undo))',
    "editor undo is POST-mounted");
  contains(EDITOR_EDITS, "pub async fn handle_undo(", "editor undo handler ships");
  var handler = between(EDITOR_EDITS, "pub async fn handle_undo(", "pub async fn handle_redo(");
  h.assert(handler.indexOf("let registry = editor_registry();") !== -1 &&
    handler.indexOf("Html(html_escape(&restored))") !== -1,
    "editor undo is a per-request HTML no-op :: <" + handler.slice(0, 300) + ">");
  contains(DESIGNER, "so it is not a persistent cross-request or agent-batch undo.",
    "designer.md records the gap");

  // Gap: the catalog has no batch undo verb. The scan is non-vacuous.
  var commands = catalogNames(COMMANDS_DIR, "cmd");
  var bindings = catalogNames(VERB_TABLE_DIR, "bind");
  h.assert(commands.length > 0 && bindings.length > 0, "catalog scan is not vacuous");
  var names = commands.concat(bindings);
  var batchUndo = names.filter(function (name) {
    return /undo/i.test(name) && /batch|agent|last/i.test(name);
  });
  h.assertEqual(batchUndo.length, 0,
    "no declared or bound verb may undo an agent batch :: <" + batchUndo.join(", ") + ">");
  [
    "office.undo_last_batch", "editor.undo.batch", "agent.undo.batch",
    "agent.batch.undo", "office.agent.undo_last_batch"
  ].forEach(function (candidate) {
    h.assert(names.indexOf(candidate) === -1,
      "batch-undo verb <" + candidate + "> is not shipped yet");
  });

  // Scoped absence: the derivation itself has no batch-undo special case.
  absentWords(derive, ["batch_id", "undo_batch", "batch_undo", "UndoBatch", "BatchUndo"],
    "agent-batch undo derivation in commands_derived.rs");
});

// 2. No durable agent-batch undo data model: every candidate store is keyed by
//    path/document/actor/cell or is browser-local, with no batch id and no
//    before/after content; no migration defines such a table or columns. This
//    case flips when a typed, persisted batch undo lands.
h.knownBug("no durable agent-batch undo data model: no batch id or before/after content in any candidate store", function () {
  // Non-vacuity: every candidate store ships its real undo implementation.
  contains(EDITOR_MOD, "pub struct EditorSession {", "editor session ships");
  contains(EDITOR_MOD, "pub undo_stack: Vec<String>,",
    "editor undo stack is a plain string vector, not typed records");
  contains(EDITOR_MOD, "pub type EditorRegistry = Arc<Mutex<HashMap<String, EditorSession>>>;",
    "editor registry type ships");
  contains(EDITOR_MOD, "pub fn editor_registry() -> EditorRegistry {", "registry factory ships");
  var factory = between(EDITOR_MOD, "pub fn editor_registry() -> EditorRegistry {", "pub(super) fn html_escape");
  h.assert(factory.indexOf("Arc::new(Mutex::new(HashMap::new()))") !== -1,
    "registry factory recreates the stack on every call :: <" + factory + ">");

  contains(EDITOR_EDITS, "let registry = editor_registry();",
    "undo uses the fresh per-request registry");
  var undo = between(EDITOR_EDITS, "pub async fn handle_undo(", "pub async fn handle_redo(");
  h.assert(undo.indexOf("Html(html_escape(&restored))") !== -1,
    "undo returns an HTML blob, not typed JSON :: <" + undo.slice(0, 300) + ">");

  contains(DOCS_HISTORY, "pub struct HistoryRequest {", "Docs history request ships");
  contains(DOCS_HISTORY, "pub id: String,", "Docs history is keyed by document id");
  contains(DOCS_HISTORY, '"undo" => entry.undo(),', "Docs undo walks a per-document stack");

  contains(SHEET_UNDO, "pub fn undo(&mut self, actor: &str, state: &mut CellState) -> bool {",
    "Sheets undo is per actor and per cell");
  contains(SHEET_UNDO, "fn can_apply(state: &CellState, record: &EditRecord, actor: &str) -> bool {",
    "the overwrite guard a real batch undo must reuse ships");
  contains(SHEET_UNDO, ".is_some_and(|winner| winner.actor == actor)",
    "undo never clobbers another author");
  contains(SLIDES_UNDO, "window.SlidesUndo = {", "Slides undo is browser-local");
  contains(DESIGNER, "One command undoes the last agent batch", "designer.md tracks criterion 4");

  // Gap: none of the candidate stores carries batch identity or before/after content.
  var stores = [EDITOR_MOD, EDITOR_EDITS, DOCS_HISTORY, SHEET_UNDO, SLIDES_UNDO];
  stores.forEach(function (relPath) {
    var src = h.readRepo(relPath);
    h.assert(src.length > 0, "candidate store must be non-empty: " + relPath);
    absentWords(src, ["batch_id", "before_content", "after_content", "agent_edit_batches"],
      "batch identity / before-after content in " + relPath);
  });

  // Gap: no durable table or columns for an agent edit batch exists in any migration.
  h.assert(h.fileExistsRepo("botserver/migrations/6.3.0-01-kb-fail-state/up.sql"),
    "migration corpus root is present");
  var sql = sourceFiles(MIGRATIONS, ".sql");
  h.assert(sql.length > 0, "migration scan is not vacuous");
  var allSql = sql.map(h.readRepo).join("\n");
  absentWords(allSql, ["agent_edit_batches", "batch_id", "before_content", "after_content"],
    "durable agent-batch undo table/columns in migrations");

  // Scoped absence: the editor handler has no typed batch-undo request/response.
  absentWords(h.readRepo(EDITOR_EDITS),
    ["UndoBatch", "BatchUndo", "UndoBatchRequest", "undo_batch", "batch_undo"],
    "typed agent-batch undo request in the editor handler");
});

h.note(
  "FOLLOW-UP #2448 criterion 4: add a typed agent-batch undo verb backed by a durable " +
  "agent_edit_batches(batch_id, before_content, after_content, user scope) store that reuses the " +
  "Sheets can_apply overwrite guard, then replace these two knownBug guards with runtime tests."
);
