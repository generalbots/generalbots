/**
 * Static + executed self-test for the Drive/assistant parity slice.
 * Run with: node botui/ui/suite/tests/webtest-drive-assistant-parity.js
 *
 * No browser, no server, no paid model calls: it reads source and catalogs
 * and executes the pure rendering helpers in a tiny DOM stub.
 */
"use strict";

var assert = require("assert");
var fs = require("fs");
var path = require("path");
var vm = require("vm");

var root = path.resolve(__dirname, "../../../..");
var locales = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var badgePath = "botui/ui/suite/drive/modules/06_file_badge.js";
var convPath = "botui/ui/suite/drive/modules/05_files_conversation.js";
var htmlPath = "botui/ui/suite/drive/drive.html";
var loaderPath = "botui/ui/suite/drive/drive-module-loader.js";
var initPath = "botui/ui/suite/drive/modules/99_init.js";

function read(rel) { return fs.readFileSync(path.join(root, rel), "utf8"); }

function catalogs() {
    var all = {};
    locales.forEach(function (loc) {
        var dir = path.join(root, "botlib/locales", loc);
        all[loc] = fs.readdirSync(dir).filter(function (f) { return /\.ftl$/.test(f); })
            .map(function (f) { return fs.readFileSync(path.join(dir, f), "utf8"); }).join("\n");
    });
    return all;
}

function assertKeyInAll(localesText, key, where) {
    locales.forEach(function (loc) {
        var pattern = new RegExp("^" + key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + "\\s*=", "m");
        assert(pattern.test(localesText[loc]), where + ": missing i18n key " + key + " in " + loc);
    });
}

var all = catalogs();

// 1. Every key introduced by the new modules exists in all eight catalogs.
var jsKeys = [
    "sidebar-conversations", "email-attachments", "chat-no-conversations-yet",
    "drive-state-no-recent-files", "action-download", "drive-loading-files",
    "chat-parity-attach-download-failed", "drive-list-error-title"
];
jsKeys.forEach(function (key) { assertKeyInAll(all, key, "JS"); });

// 2. The two new top tabs carry data-i18n keys catalogued everywhere.
var html = read(htmlPath);
// Build the attribute name from fragments: spelling the attribute literally
// in this test would itself be parsed as a dynamic key by the catalogue linter.
var dataAttr = "data-" + "i18n";
["sidebar-conversations", "email-attachments"].forEach(function (key) {
    var pattern = new RegExp(dataAttr + '="' + key + '"');
    assert(pattern.test(html), "HTML: missing data-i18n attribute for " + key);
    assertKeyInAll(all, key, "HTML");
});

// 3. Loader + init wire the surfaces; tab restoration survives a reload.
var loader = read(loaderPath);
assert(loader.indexOf("/suite/drive/modules/06_file_badge.js") >= 0, "loader: badge module");
assert(loader.indexOf("/suite/drive/modules/05_files_conversation.js") >= 0, "loader: conversation module");
var events = read("botui/ui/suite/drive/modules/07_events.js");
assert(events.indexOf('case "conversations":') >= 0, "click: opens conversations tab");
assert(events.indexOf('case "attachments":') >= 0, "click: opens attachments tab");
var init = read(initPath);
assert(init.indexOf('case "conversations":') >= 0, "init: restores conversations tab");
assert(init.indexOf('case "attachments":') >= 0, "init: restores attachments tab");

// 4. Execute the pure badge renderer; extension labels are format names, not copy.
function sandbox(extra) {
    var scope = {
        window: {}, document: { addEventListener: function () {}, getElementById: function () { return null; } },
        sessionStorage: { getItem: function () { return null; } },
        localStorage: { getItem: function () { return null; } },
        escapeHtml: function (s) { return String(s); },
        console: console
    };
    Object.keys(extra || {}).forEach(function (k) { scope[k] = extra[k]; });
    scope.window = Object.assign(scope.window, (extra || {}).window || {});
    vm.createContext(scope);
    return scope;
}

var badgeScope = sandbox({ renderFiles: function () {} });
vm.runInContext(read(badgePath), badgeScope, { filename: badgePath });
var badge = badgeScope.driveFileBadge;
assert(/DOC/.test(badge("report.docx", false)), "docx badge says DOC");
assert(/XLS/.test(badge("plan.xlsx", false)), "xlsx badge says XLS");
assert(/PDF/.test(badge("a.pdf", false)), "pdf badge says PDF");
assert(/drive-file-badge-danger/.test(badge("a.pdf", false)), "pdf tone");
assert(/drive-file-badge-image/.test(badge("pic.png", false)), "image tone");
assert(/drive-file-badge-dir/.test(badge("folder", true)), "folder badge");
assert(!/undefined/.test(badge("noext", false)), "no undefined in badge");

// 5. Conversation folder titles drop the id suffix and the `conversation-` prefix.
var convScope = sandbox({});
vm.runInContext(read(convPath), convScope, { filename: convPath });
var title = convScope.driveConversationTitle;
assert.strictEqual(title("quarterly-report-1a2b3c4d"), "quarterly report", "slug + id8");
assert.strictEqual(title("conversation-1a2b3c4d"), "Conversations", "id-only falls back to tab label");
assert.strictEqual(title("budget-2026"), "budget 2026", "slug without id");

// 6. Both loaders use the generic, shipped endpoints (no gbgov-only API).
var conv = read(convPath);
assert(conv.indexOf('/list?" + driveListParams(root)') >= 0, "conversation folders use /api/files/list");
assert(conv.indexOf("/api/chat/history/sessions?") >= 0, "attachments use chat history");
assert(conv.indexOf("/api/chat/attachments/preview?download=1") >= 0, "inbox download uses chat preview");

console.log("webtest-drive-assistant-parity: all assertions passed");
