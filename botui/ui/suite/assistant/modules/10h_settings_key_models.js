"use strict";
/**
 * 10h_settings_key_models.js - "Modelos desta chave" on the provider setup
 * screen (Settings > Modelo > "Servidor ou chave própria" > a provider).
 *
 * One key, many models: "Buscar modelos" asks the server for the list the key
 * unlocks, the person switches models on and off, and "Salvar modelos" stores
 * the account. The switched-on models appear in the chat model menu, grouped
 * by provider. Opening the screen only reads the stored account; the provider
 * is contacted only when the person clicks "Buscar modelos".
 */
(function () {
var A = window.GBAssistant;
var FILTER_FROM = 12;

var L = { "pt-BR": {
  title:"Modelos desta chave",
  hint:"Busque os modelos que esta chave libera e ligue os que devem aparecer no menu do chat.",
  stored:"Com o campo da chave vazio, a busca usa a chave guardada.",
  fetch:"Buscar modelos", fetching:"Buscando modelos…", filter:"Filtrar modelos",
  count:"%n de %t ligados", save:"Salvar modelos", saving:"Salvando…",
  saved:"Modelos salvos. Os ligados aparecem no menu do chat.",
  fetchFail:"Não foi possível buscar os modelos (%e).", saveFail:"Não foi possível salvar (%e).",
  empty:"O provedor não devolveu nenhum modelo.", noMatch:"Nenhum modelo com esse nome." },
en: {
  title:"Models for this key",
  hint:"Fetch the models this key unlocks and switch on the ones that should appear in the chat menu.",
  stored:"With the key field empty, the search uses the stored key.",
  fetch:"Fetch models", fetching:"Fetching models…", filter:"Filter models",
  count:"%n of %t on", save:"Save models", saving:"Saving…",
  saved:"Models saved. The ones switched on appear in the chat menu.",
  fetchFail:"Could not fetch the models (%e).", saveFail:"Could not save (%e).",
  empty:"The provider returned no models.", noMatch:"No model by that name." } };
function M() { return A._modelProviders; }
function g(k) {
  var l = A.locale && A.locale.startsWith("pt") ? L["pt-BR"] : L.en;
  return l[k] || L.en[k] || M().t(k);
}
function esc(s) { return M().esc(s); }

/**
 * Mounts the block into `box`. `ctx` = { id, name, keyOptional,
 * url(): the chat address in use, key(): the key typed in the form }.
 */
function mount(box, ctx) {
  var account = null;
  var all = [];
  var on = {};
  var filter = "";

  box.innerHTML = '<section class="gb-km">' +
    '<div class="gb-km-head"><div class="gb-prov-title gb-km-title">' + esc(g("title")) + '</div>' +
      '<button type="button" class="gb-set-btn-sm" id="gbKmFetch">' + esc(g("fetch")) + '</button></div>' +
    '<p class="gb-set-hint" id="gbKmHint">' + esc(g("hint")) + '</p>' +
    '<div id="gbKmBody"></div><div id="gbKmMsg" role="status"></div></section>';
  var body = box.querySelector("#gbKmBody");
  var fetchBtn = box.querySelector("#gbKmFetch");

  function say(cls, text) {
    box.querySelector("#gbKmMsg").innerHTML = text ? '<p class="' + cls + '">' + esc(text) + '</p>' : "";
  }
  function failText(key, d) {
    return g(key).replace("%e", M().reason(d && d.error)) +
      (d && d.detail ? " " + g("said") + " “" + d.detail + "”" : "");
  }
  function countOn() { return all.filter(function (m) { return on[m]; }).length; }
  function rows() {
    var q = filter.trim().toLowerCase();
    var shown = all.filter(function (m) { return !q || m.toLowerCase().indexOf(q) >= 0; });
    if (!shown.length) { return '<p class="gb-set-hint gb-km-none">' + esc(g("noMatch")) + '</p>'; }
    return shown.map(function (m) {
      return '<label class="gb-km-row"><span class="gb-km-id">' + esc(m) + '</span>' +
        '<input type="checkbox" class="gb-set-toggle" data-model="' + esc(m) + '"' + (on[m] ? ' checked' : '') + '/></label>';
    }).join("");
  }
  function updateCount() {
    var c = box.querySelector("#gbKmCount");
    if (c) { c.textContent = g("count").replace("%n", String(countOn())).replace("%t", String(all.length)); }
  }
  function wireRows() {
    box.querySelectorAll(".gb-km-row input").forEach(function (cb) {
      cb.addEventListener("change", function () { on[cb.getAttribute("data-model")] = cb.checked; updateCount(); });
    });
  }
  function render() {
    if (!all.length) { body.innerHTML = ""; return; }
    body.innerHTML =
      (all.length > FILTER_FROM ? '<label class="gb-prov-search gb-km-filter"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="11" cy="11" r="7"/><path d="M20 20l-3.5-3.5"/></svg>' +
        '<input type="search" id="gbKmFilter" placeholder="' + esc(g("filter")) + '" aria-label="' + esc(g("filter")) + '" value="' + esc(filter) + '"/></label>' : '') +
      '<div class="gb-km-list" role="group" aria-label="' + esc(g("title")) + '">' + rows() + '</div>' +
      '<div class="gb-km-foot"><span class="gb-set-hint" id="gbKmCount"></span>' +
        '<button type="button" class="gb-set-btn-sm gb-set-btn-accent" id="gbKmSave">' + esc(g("save")) + '</button></div>';
    updateCount();
    wireRows();
    var f = box.querySelector("#gbKmFilter");
    if (f) {
      f.addEventListener("input", function () {
        filter = f.value;
        box.querySelector(".gb-km-list").innerHTML = rows();
        wireRows();
      });
    }
    box.querySelector("#gbKmSave").addEventListener("click", save);
  }

  function fetchModels() {
    var url = ctx.url(), key = ctx.key();
    if (!url) { say("gb-set-warn", g("urlNeeded")); return; }
    if (!key && !ctx.keyOptional && !(account && account.has_key)) { say("gb-set-warn", g("keyNeeded")); return; }
    var req = { id: ctx.id, url: url }; if (key) { req.api_key = key; }
    fetchBtn.disabled = true;
    say("gb-set-hint", g("fetching"));
    A.apiFetch("/api/llm/accounts/models", { method: "POST", body: JSON.stringify(req) })
      .then(function (r) { return r.json(); })
      .then(function (d) {
        var got = (d && d.models) || [];
        if (!got.length) { say("gb-set-warn", d && d.error ? failText("fetchFail", d) : g("empty")); return; }
        var stale = all.filter(function (m) { return on[m] && got.indexOf(m) < 0; });
        all = stale.concat(got);
        say("", "");
        render();
      })
      .catch(function (e) { say("gb-set-warn", g("fetchFail").replace("%e", String(e && e.message || e))); })
      .then(function () { fetchBtn.disabled = false; });
  }

  function save() {
    var url = ctx.url(), key = ctx.key();
    if (!url) { say("gb-set-warn", g("urlNeeded")); return; }
    if (!key && !ctx.keyOptional && !(account && account.has_key)) { say("gb-set-warn", g("keyNeeded")); return; }
    var btn = box.querySelector("#gbKmSave");
    var req = { id: ctx.id, name: ctx.name, url: url, enabled: all.filter(function (m) { return on[m]; }) };
    if (key) { req.api_key = key; }
    btn.disabled = true;
    say("gb-set-hint", g("saving"));
    A.apiFetch("/api/llm/accounts", { method: "PUT", body: JSON.stringify(req) })
      .then(function (r) {
        return r.json().catch(function () { return {}; }).then(function (d) {
          if (!r.ok || !d.ok) { throw { error: d.error || ("http_status_" + r.status), detail: d.detail }; }
        });
      })
      .then(function () {
        account = { id: ctx.id, has_key: true, enabled: req.enabled };
        say("gb-set-hint gb-set-ok", g("saved"));
        if (A.loadModels) { return A.loadModels().then(function () { if (A.renderModelPickers) { A.renderModelPickers(); } }); }
      })
      .catch(function (e) {
        say("gb-set-warn", e && e.error ? failText("saveFail", e) : g("saveFail").replace("%e", String(e && e.message || e)));
      })
      .then(function () { btn.disabled = false; });
  }

  fetchBtn.addEventListener("click", fetchModels);

  A.apiFetch("/api/llm/accounts")
    .then(function (r) { return r.ok ? r.json() : {}; })
    .then(function (d) {
      account = ((d && d.accounts) || []).filter(function (a) { return a.id === ctx.id; })[0] || null;
      if (!account) { return; }
      if (account.has_key) { box.querySelector("#gbKmHint").textContent = g("hint") + " " + g("stored"); }
      all = (account.enabled || []).slice();
      all.forEach(function (m) { on[m] = true; });
      render();
    })
    .catch(function () { account = null; });
}

A._keyModels = { mount: mount };
})();
