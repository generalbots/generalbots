/**
 * [STATIC] #2448 criterion 4 — "One command undoes the last agent batch".
 *
 * The browser UAT ("Undo the last agent batch" -> "There is nothing to undo")
 * found no *undo* verb in the chat/agent catalog. These guards characterise
 * the exact blocker against shipped source and FAIL when the documented gap
 * changes, so a real implementation must replace them with runtime tests:
 *   (a) no batch            -> explicit not-found/error
 *   (b) recorded batch      -> undo invoked once with the right batch id
 *   (c) foreign user's batch -> denied
 *
 * This is a gap marker, not a simulated implementation: no user-facing string,
 * no server, no browser, no cargo.
 */
"use strict";

var h = require("./00_harness");
var fs = h.fs;
var path = h.path;

var COMMANDS_DIR = "botserver/src/apps/commands";
var VERB_TABLE_DIR = COMMANDS_DIR + "/verb_endpoints/table";
var EDITOR_MOD = "botserver/crates/boteditor/src/handlers/mod.rs";
var EDITOR_EDITS = "botserver/crates/boteditor/src/handlers/edits.rs";
var EDITOR_ROUTES = "botserver/crates/boteditor/src/routes.rs";
var EDITOR_INVENTORY =
  "botserver/src/core/bot/endpoint_inventory_directory/editor.rs";
var DOCS_HISTORY = "botserver/crates/botdocs/src/handlers_api/history.rs";
var SHEET_UNDO = "botserver/crates/botsheet/src/collaboration/undo.rs";
var SLIDES_UNDO = "botui/ui/suite/slides/modules/09_undo.js";
var CORE_BOT_DIR = "botserver/src/core/bot";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(
    h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath
  );
}

/** Repo-relative .rs files under a directory, in deterministic order. */
function rustSources(relDir) {
  var root = path.join(h.REPO_DIR, relDir);
  var found = [];
  (function walk(dir) {
    fs.readdirSync(dir, { withFileTypes: true })
      .sort(function (a, b) { return a.name < b.name ? -1 : a.name > b.name ? 1 : 0; })
      .forEach(function (entry) {
        var full = path.join(dir, entry.name);
        if (entry.isDirectory()) walk(full);
        else if (entry.name.slice(-3) === ".rs") {
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
  rustSources(relDir).forEach(function (relPath) {
    var src = h.readRepo(relPath);
    var match = re.exec(src);
    while (match !== null) {
      names.push(match[1]);
      match = re.exec(src);
    }
  });
  return names;
}

function hasUndo(name) {
  return /undo/i.test(name);
}

// 1. The catalog really has zero undo verbs — neither a declared command nor
//    a bound endpoint. This is the UAT's "no *undo* catalog verb at all".
h.staticTest("#2448 chat-native undo verb is declared, zero-param and dispatched in-process", function () {
  var commands = catalogNames(COMMANDS_DIR, "cmd");
  var bindings = catalogNames(VERB_TABLE_DIR, "bind");
  h.assert(commands.length > 0 && bindings.length > 0, "catalog scan is not vacuous");
  h.assert(commands.filter(hasUndo).length > 0, "no undo command is declared any more");
  h.assert(commands.indexOf("docs.undo") !== -1, "docs.undo is not in the command catalog");
  // Zero params: no prompt-supplied id can select a scope or a batch.
  contains("botserver/src/apps/commands/app_commands_extra/table/undo_ops.rs", "&[]", "undo verb takes no params");
  // Scope comes only from the verified chat identity and a single-use consume.
  contains("botserver/src/core/bot/undo/lookup.rs", "consume_latest", "latest batch is consumed");
  contains("botserver/src/main_module/routes/collab_routes/undo.rs", "verified_scope", "scope is server-derived");
  contains("botserver/src/core/bot/api_catalog/execute_command.rs", '"docs.undo" =>', "verb has a dispatch arm");
  // In-process dispatch: no REST binding to an inventory-absent route.
  h.assertEqual(bindings.filter(hasUndo).length, 0, "undo must not be a REST-bound verb");
});


// 2. The only undo route the LLM can even see is the editor's: it is in the
//    endpoint inventory, but no verb binds to it, its handler answers with an
//    HTML blob instead of typed JSON, and its stack is rebuilt on every call.
h.knownBug("the only registered undo endpoint is a non-typed, per-request editor handler", function () {
  contains(EDITOR_INVENTORY, 'path: "/api/editor/undo"', "editor undo is registered in the inventory");
  contains(
    EDITOR_ROUTES,
    '.route("/api/editor/undo", post(handlers::handle_undo))',
    "editor undo is mounted on the real router"
  );

  var boundSources = rustSources(VERB_TABLE_DIR)
    .map(h.readRepo)
    .join("\n");
  h.assert(
    boundSources.indexOf("/api/editor/undo") === -1,
    "no curated verb binds to /api/editor/undo"
  );

  var handler = h.readRepo(EDITOR_EDITS);
  h.assert(
    handler.indexOf("Query(query): Query<EditorQuery>") !== -1 &&
      handler.indexOf("Form(payload): Form<FormContent>") !== -1,
    "handler reads a path query and a form content blob, not a typed undo request"
  );
  h.assert(
    handler.indexOf("Html(html_escape(&restored))") !== -1,
    "handler answers with an HTML free-text blob, not typed JSON"
  );
  h.assert(
    handler.indexOf("batch_id") === -1,
    "handler has no batch id to resolve or replay"
  );

  contains(EDITOR_MOD, "pub fn editor_registry() -> EditorRegistry {", "registry factory present");
  contains(
    EDITOR_MOD,
    "Arc::new(Mutex::new(HashMap::new()))",
    "registry factory recreates the undo stack on every call"
  );
});

// 3. The other three undo paths carry no batch identity either: Docs is keyed
//    by document id, Sheets is keyed by actor and cell, Slides is browser-only.
h.knownBug("Docs, Sheets and Slides undo carry no batch identity", function () {
  contains(DOCS_HISTORY, "pub id: String,", "Docs history is keyed by a document id");
  contains(DOCS_HISTORY, '"undo" => entry.undo(),', "Docs undo walks a per-document stack");
  h.assert(
    h.readRepo(DOCS_HISTORY).indexOf("batch_id") === -1,
    "Docs history has no batch id"
  );

  contains(
    SHEET_UNDO,
    "pub fn undo(&mut self, actor: &str, state: &mut CellState) -> bool {",
    "Sheets undo is per actor and per cell"
  );
  h.assert(
    h.readRepo(SHEET_UNDO).indexOf("batch_id") === -1,
    "Sheets undo manager has no batch id"
  );

  contains(SLIDES_UNDO, "window.SlidesUndo = {", "Slides undo is browser-local");
  h.assert(
    h.readRepo(SLIDES_UNDO).indexOf("batch_id") === -1,
    "Slides undo has no batch id"
  );

  var inventorySources = rustSources(CORE_BOT_DIR)
    .map(h.readRepo)
    .join("\n");
  h.assert(
    inventorySources.indexOf("/api/docs/history") === -1,
    "per-document Docs history is not exposed through the chat endpoint inventory"
  );
});

h.note(
  "FOLLOW-UP #2448 criterion 4: add a batch id + user scope to the undo store, " +
  "a typed JSON undo verb, and runtime tests for no-batch / recorded-batch / foreign-user."
);
