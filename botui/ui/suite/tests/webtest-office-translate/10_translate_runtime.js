/**
 * #2511 RUNTIME verification of the REAL shipped docs translation module.
 *
 * botui/ui/suite/docs/modules/17_translate.js is read from disk and executed in
 * a vm sandbox with inert DOM/window stubs and a captured fetch. Every assertion
 * below observes an actual call or mutation made by that shipped code.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var assertEqual = h.assertEqual;
var makeTranslateFixture = h.makeTranslateFixture;

test("RUNTIME #2511: shipped translation module loads and binds its window exports", function () {
  var f = makeTranslateFixture();
  ["activeDocIdTranslate", "translateDocument", "openTranslateModal", "closeTranslateModal"].forEach(function (name) {
    assert(typeof f.ctx[name] === "function", "global " + name + " must be defined by the shipped module");
  });
  ["openTranslateModal", "closeTranslateModal", "GBDocsTranslate", "translateDocument"].forEach(function (name) {
    assert(typeof f.window[name] === "function", "window." + name + " must be exported");
  });
});

test("[UAT:DOC-07] RUNTIME #2511: undo history is seeded by autosaving the ORIGINAL html before translate", async function () {
  var original = "<h1>Contrato</h1><p>As partes <strong> concordam </strong>.</p>";
  var f = makeTranslateFixture({ html: original });
  var ok = await f.ctx.translateDocument("Portuguese");
  assertEqual(ok, true, "translateDocument must report success");
  assert(f.calls.length >= 2, "at least an autosave and a translate call must be made");
  assertEqual(f.calls[0].url, "/api/docs/autosave", "first call must seed history");
  assertEqual(f.calls[0].body.id, "contract-1", "history seed must use the active doc id");
  assertEqual(f.calls[0].body.content, original, "history seed must contain the untouched original html");
  assertEqual(f.calls[1].url, "/api/docs/ai/translate", "second call must request translation");
  assertEqual(f.calls[1].body.action, "translate", "translate action must be sent");
  assertEqual(f.calls[1].body.prompt, "Portuguese", "target language must be the prompt");
  assertEqual(f.calls[1].body.translate_lang, "Portuguese", "translate_lang must carry the target language");
  assertEqual(f.calls[1].body.selected_text, original, "the original html must be sent for translation");
});

test("[UAT:DOC-07] RUNTIME #2511: HTML translation is applied verbatim with inline formatting intact", async function () {
  var translated = "<p>Titulo <strong>mundo</strong> e <em>estilo</em></p>";
  var f = makeTranslateFixture({ translateResponse: { content: translated } });
  var ok = await f.ctx.translateDocument("Portuguese");
  assertEqual(ok, true, "translation must succeed");
  assertEqual(f.article.innerHTML, translated, "returned HTML must be applied verbatim");
  assert(f.article.innerHTML.indexOf("<strong>mundo</strong>") >= 0, "strong markup must survive apply");
  assert(f.article.innerHTML.indexOf("<em>estilo</em>") >= 0, "emphasis markup must survive apply");
  assert(f.article.events.length === 1, "one input event must be dispatched");
  assertEqual(f.article.events[0].type, "input", "event type must be input");
  assertEqual(f.article.events[0].bubbles, true, "input event must bubble");
  assertEqual(f.pageCountCalls(), 1, "page count must be refreshed after apply");
  assertEqual(f.calls[2].url, "/api/docs/autosave", "third call must save the translated content");
  assertEqual(f.calls[2].body.content, translated, "translated html must be autosaved");
  assertEqual(f.statusCalls[f.statusCalls.length - 1].message, "Translated to Portuguese", "success status must name the target language");
  assertEqual(f.statusCalls[f.statusCalls.length - 1].isError, false, "success must not be an error status");
  assertEqual(f.snapshots.length, 1, "one collaboration snapshot must be taken");
  assertEqual(f.snapshots[0].resourceType, "docs", "snapshot resource type");
  assertEqual(f.snapshots[0].resourceId, "contract-1", "snapshot resource id");
  assertEqual(f.snapshots[0].content, translated, "snapshot must capture the translated markup");
});

test("RUNTIME #2511: content/response/text with a success status are applied", async function () {
  var fields = ["content", "response", "text"];
  for (var i = 0; i < fields.length; i++) {
    var field = fields[i];
    var payload = { result: "success" };
    payload[field] = "<p>" + field + "</p>";
    var f = makeTranslateFixture({ translateResponse: payload });
    var ok = await f.ctx.translateDocument("Spanish");
    assertEqual(ok, true, field + " payload must translate successfully");
    assertEqual(f.article.innerHTML, "<p>" + field + "</p>", field + " payload must be applied");
  }
});

test("RUNTIME #2457: an unavailable/error response is refused, not applied", async function () {
  var initial = "<p>Hello <strong>world</strong></p>";
  var f = makeTranslateFixture({
    translateResponse: { result: "unavailable", content: "<p>fake translation</p>", error: "LLM 429" }
  });
  assertEqual(f.article.innerHTML, initial, "fixture starts with original content");
  var ok = await f.ctx.translateDocument("Spanish");
  assertEqual(ok, false, "unavailable result must not report success");
  assertEqual(f.article.innerHTML, initial, "unavailable result must not overwrite content");
});

test("RUNTIME #2511: plain text is routed through docsRenderBlocks when the renderer is present", async function () {
  var rendered = [];
  var f = makeTranslateFixture({
    translateResponse: { text: "plain **bold** text" },
    renderBlocks: function (text) {
      rendered.push(text);
      return "<p>RENDERED:" + text + "</p>";
    }
  });
  var ok = await f.ctx.translateDocument("French");
  assertEqual(ok, true, "translation must succeed");
  assertEqual(rendered.length, 1, "markdown-lite renderer must be called once");
  assertEqual(rendered[0], "plain **bold** text", "renderer must receive the raw translated text");
  assertEqual(f.article.innerHTML, "<p>RENDERED:plain **bold** text</p>", "renderer output must be applied");
});

test("RUNTIME #2511: an unavailable translation leaves the document untouched and reports an error", async function () {
  var original = "<p>Keep me <b>intact</b></p>";
  var f = makeTranslateFixture({ html: original, translateResponse: { ok: false, status: 503 } });
  var ok = await f.ctx.translateDocument("German");
  assertEqual(ok, false, "failed translation must report false");
  assertEqual(f.article.innerHTML, original, "failed translation must not modify the article");
  assertEqual(f.article.events.length, 0, "no input event may be dispatched on failure");
  assert(f.calls.length === 2, "only the history autosave and the failed translate call may be made");
  var last = f.statusCalls[f.statusCalls.length - 1];
  assertEqual(last.message, "Translation unavailable", "failure status must be explicit");
  assertEqual(last.isError, true, "failure status must be flagged as an error");
  assertEqual(f.snapshots.length, 0, "no snapshot may be taken for a failed translation");
});

test("RUNTIME #2511: missing active article returns false without any network call", async function () {
  var f = makeTranslateFixture({ noActiveArticle: true });
  var ok = await f.ctx.translateDocument("English");
  assertEqual(ok, false, "no article means no translation");
  assertEqual(f.calls.length, 0, "no fetch may be issued without an article");
});

test("RUNTIME #2511: doc id falls back to current when no data-doc-id article is found", async function () {
  var f = makeTranslateFixture({ noArticle: true });
  assertEqual(f.ctx.activeDocIdTranslate(), "current", "fallback id must be current");
  await f.ctx.translateDocument("Italian");
  assertEqual(f.calls[0].body.id, "current", "autosave must use the fallback id");
});

test("RUNTIME #2511: translate modal wires language select, apply button and close", async function () {
  var f = makeTranslateFixture({ translateResponse: { content: "<p>Traducido</p>" } });
  f.ctx.openTranslateModal();
  assert(f.modalHost.innerHTML.indexOf("translate-lang") >= 0, "modal must render the language select");
  assert(f.modalHost.innerHTML.indexOf("translate-apply") >= 0, "modal must render the apply button");
  assert(typeof f.applyButton.onclick === "function", "apply button must be wired");
  f.langSelect.value = "Spanish";
  await f.applyButton.onclick();
  assertEqual(f.article.innerHTML, "<p>Traducido</p>", "apply must translate using the selected language");
  assertEqual(f.calls[1].body.translate_lang, "Spanish", "selected language must drive translation");
  assertEqual(f.modalHost.innerHTML, "", "modal must close after applying");
  f.ctx.openTranslateModal();
  f.ctx.closeTranslateModal();
  assertEqual(f.modalHost.innerHTML, "", "closeTranslateModal must clear the host");
});

// ── KNOWN BUGS: observed shipped behaviour, reported but never patched ──

test("KNOWN BUG #2511: plain-text server output erases all source formatting (asserted, not fixed)", async function () {
  var original = "<h1>Contrato</h1><p>As partes <strong>concordam</strong>.</p>";
  var f = makeTranslateFixture({ html: original, translateResponse: { content: "Contrato As partes concordam." } });
  var ok = await f.ctx.translateDocument("Portuguese");
  assertEqual(ok, true, "shipped code reports success despite destroying markup");
  assertEqual(f.article.innerHTML, "Contrato As partes concordam.", "observed: raw plain text replaces the article");
  assert(f.article.innerHTML.indexOf("<h1>") < 0, "observed: original heading is lost");
  assert(f.article.innerHTML.indexOf("<strong>") < 0, "observed: original inline style is lost");
  assertEqual(f.statusCalls[f.statusCalls.length - 1].message, "Translated to Portuguese", "observed: no formatting warning is raised");
});

test("KNOWN BUG #2511: translation never registers a language variant or updates DocsLang state", async function () {
  var f = makeTranslateFixture({ withLangModules: true, translateResponse: { content: "<p>Traduzido</p>" } });
  var ns = f.window.DocsLang;
  assertEqual(ns.state.language, "en", "precondition: default language state is English");
  var ok = await f.ctx.translateDocument("Portuguese");
  assertEqual(ok, true, "translation must succeed for the observation");
  assertEqual(ns.state.language, "en", "observed: translateDocument leaves the language state untouched");
  assertEqual(f.window.localStorage.getItem("docs2484:contract-1:language"), null,
    "observed: no per-document language variant is persisted");
  assertEqual(f.calls[1].body.translate_lang, "Portuguese", "observed: translate uses a full language name");
  assert(ns.LANGUAGES.some(function (row) { return row[0] === "pt"; }),
    "observed: DocsLang catalog uses ISO codes such as pt, disjoint from the translate modal vocabulary");
});

test("KNOWN BUG #2511: swallowed autosave failure applies translation with no undo history", async function () {
  var original = "<p>Original <b>safe</b></p>";
  var f = makeTranslateFixture({
    html: original,
    autosaveFails: true,
    translateResponse: { content: "<p>Traduzido</p>" }
  });
  var ok = await f.ctx.translateDocument("Spanish");
  assertEqual(ok, true, "observed: translation applies even though history seeding failed");
  assertEqual(f.article.innerHTML, "<p>Traduzido</p>", "observed: content was replaced");
  assertEqual(f.calls[0].url, "/api/docs/autosave", "observed: the failing history autosave was still attempted");
  assertEqual(f.snapshots.length, 1, "observed: only the post-translation snapshot exists; no undo seed reached the server");
  assertEqual(f.statusCalls[f.statusCalls.length - 1].isError, false, "observed: no warning that undo history is missing");
});
