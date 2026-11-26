"use strict";
/**
 * 10h_settings_key_models.js - "Modelos disponíveis" on the provider setup
 * screen (Settings > Modelo > "Servidor ou chave própria" > a provider).
 *
 * One key, many models: the server lists the models the key unlocks (in its
 * own order), the person switches them on and off, and "Salvar modelos"
 * stores the account. The switched-on models appear in the chat model menu.
 * A stored key loads the list by itself; "Atualizar" asks again.
 */
(function () {
var A = window.GBAssistant;
var FILTER_FROM = 12;
function gbT(k, fb) { return typeof window.gbT === "function" ? window.gbT(k, fb) : fb; }

var L = { "pt-BR": {
  title:"Modelos desta chave",
  hint:"Busque os modelos que esta chave libera e ligue os que devem aparecer no menu do chat.",
  stored:"Com o campo da chave vazio, a busca usa a chave guardada.",
  fetch:"Buscar modelos", fetching:"Buscando modelos…", filter:"Filtrar modelos",
  count:"%n de %t ligados", save:gbT("mdl-km-save", "Save models"), saving:"Salvando…",
  saved:"Modelos salvos. Os ligados aparecem no menu do chat.",
  fetchFail:"Não foi possível buscar os modelos (%e).", saveFail:"Não foi possível salvar (%e).",
  empty:"O provedor não devolveu nenhum modelo.", noMatch:gbT("mdl-km-no-match", "No model by that name."),
  unavailable:"Este servidor ainda não oferece modelos por chave.", said:"O provedor disse:",
  urlNeeded:"Informe o endereço do servidor.", keyNeeded:"Cole a chave de API." },
en: {
  title:"MODELS IN THE CHAT MENU", refresh:"Refresh", first:"Paste the key and test the connection to see the models.",
  hint:"Switch on the models of this key that should appear in the chat menu.",
  stored:"With the key field empty, the list uses the stored key.",
  fetch:"Refresh models", fetching:"Fetching models…", filter:"Filter models",
  count:"%n OF %t ON", saving:"Saving…",
  saved:"Saved.",
  fetchFail:"Could not fetch the models (%e).", saveFail:"Could not save (%e).",
  empty:"The provider returned no models.", noMatch:"No model by that name.",
  unavailable:"This server does not offer key-based models yet.", said:"The provider said:",
  urlNeeded:"Enter the server address.", keyNeeded:"Paste the API key." } };
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
  var savedOn = {};
  var filter = "";
  var available = false;

  box.innerHTML = '<section class="gb-of-km"><div class="gb-of-km-h"><span class="gb-of-k">' + (G ? G.sl() : '') + esc(g("title")) + '</span>' +
    (G ? G.help(g("hint") + " " + g("stored")) : '') + '<span class="gb-of-sp"></span><span class="gb-of-meta" id="gbKmCount"></span>' +
    '<button type="button" class="gb-of-btn sm" id="gbKmFetch">' + (G ? G.svg("M21 12a9 9 0 1 1-3-6.7L21 8M21 3v5h-5", 12, 1.9) : '') + esc(g("refresh")) + '</button></div>' +
    '<div id="gbKmBody"></div><div id="gbKmMsg" class="gb-of-say" role="status"></div></section>';
  var body = box.querySelector("#gbKmBody");
  var fetchBtn = box.querySelector("#gbKmFetch");

  function say(cls, text) {
    box.querySelector("#gbKmMsg").innerHTML = text ? '<p class="' + cls + '" style="padding:10px 16px;border-top:1px solid var(--id-line)">' + esc(text) + '</p>' : "";
  }
  function failText(key, d) {
    return M().failMsg(g(key), d && d.error) +
      (d && d.detail ? " " + g("said") + " “" + d.detail + "”" : "");
  }
  /* The key is stored for this account, or it is the main own-key setup's key for the same host. */
  function keyStored() {
    if (account && account.has_key) { return true; }
    var c = A._modelShared && A._modelShared.cfg;
    if (!c || c.mode !== "own_key" || !c.has_key || !c.endpoint_host) { return false; }
    try { return new URL(ctx.url()).hostname.toLowerCase() === String(c.endpoint_host).toLowerCase(); } catch (e) { return false; }
  }
  function countOn() { return all.filter(function (m) { return on[m]; }).length; }
  function rows() {
    var q = filter.trim().toLowerCase();
    var shown = all.filter(function (m) { return !q || m.toLowerCase().indexOf(q) >= 0; });
    if (!shown.length) { return '<p class="gb-of-hint" style="margin:0;padding:10px 16px;border-top:1px solid var(--id-line)">' + esc(g("noMatch")) + '</p>'; }
    return shown.map(function (m) {
      return '<label class="gb-of-km-r' + (on[m] ? ' on' : '') + '"><i></i><span><b>' + esc(m) + '</b>' +
        '<input type="checkbox" class="gb-of-tg" data-model="' + esc(m) + '"' + (on[m] ? ' checked' : '') + '/></span></label>';
    }).join("");
  }
  function updateCount() {
    var c = box.querySelector("#gbKmCount");
    if (c) { c.textContent = all.length ? g("count").replace("%n", String(countOn())).replace("%t", String(all.length)) : ""; }
  }
  function wireRows() {
    box.querySelectorAll(".gb-of-km-r input").forEach(function (cb) {
      cb.addEventListener("change", function () {
        on[cb.getAttribute("data-model")] = cb.checked;
        cb.closest(".gb-of-km-r").classList.toggle("on", cb.checked);
        updateCount();
        scheduleSave();
      });
    });
  }
  function scheduleSave() {
    clearTimeout(saveTimer);
    if (saving) { pendingSave = true; return; }
    saveTimer = setTimeout(save, 400);
  }
  function revertSwitches() {
    all.forEach(function (m) { on[m] = !!savedOn[m]; });
    render();
  }
  function render() {
    if (!all.length) { body.innerHTML = '<p class="gb-of-hint" style="margin:0;padding:10px 16px;border-top:1px solid var(--id-line)">' + esc(g("first")) + '</p>'; updateCount(); return; }
    body.innerHTML = '<div class="gb-of-km-list" role="group" aria-label="' + esc(g("title")) + '">' + rows() + '</div>' +
      (all.length > FILTER_FROM ? '<div class="gb-of-km-f"><div class="gb-of-srch"><label>' + (G ? G.svg("M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zM20 20l-3.5-3.5", 14) : '') +
        '<input type="search" id="gbKmFilter" placeholder="' + esc(g("filter")) + '" aria-label="' + esc(g("filter")) + '" value="' + esc(filter) + '"/></label></div></div>' : '');
    updateCount();
    wireRows();
    var f = box.querySelector("#gbKmFilter");
    if (f) {
      f.addEventListener("input", function () {
        filter = f.value;
        box.querySelector(".gb-of-km-list").innerHTML = rows();
        wireRows();
      });
    }
  }

  /* Loads the list by itself when a key is there to use. */
  function autoLoad() {
    if (ctx.url() && (keyStored() || ctx.key() || ctx.keyOptional)) { fetchModels(); }
  }
  function fetchModels() {
    if (!available) { say("gb-set-warn", g("unavailable")); return; }
    var url = ctx.url(), key = ctx.key();
    if (!url) { say("gb-set-warn", g("urlNeeded")); return; }
    if (!key && !ctx.keyOptional && !keyStored()) { say("gb-set-warn", g("keyNeeded")); return; }
    var req = { id: ctx.id, url: url }; if (key) { req.api_key = key; }
    fetchBtn.disabled = true;
    say("gb-set-hint", g("fetching"));
    A.apiFetch("/api/llm/accounts/models", { method: "POST", body: JSON.stringify(req) })
      .then(function (r) { if (!r.ok) { throw new Error("http_" + r.status); } return r.json(); })
      .then(function (d) {
        var got = (d && d.models) || [];
        if (!got.length) { say("gb-set-warn", d && d.error ? failText("fetchFail", d) : g("empty")); return; }
        var stale = all.filter(function (m) { return on[m] && got.indexOf(m) < 0; });
        all = stale.concat(got);
        say("", "");
        render();
      })
      .catch(function (e) {
        var msg = String(e && e.message || e);
        say("gb-set-warn", /http_40[45]/.test(msg) ? g("unavailable") : g("fetchFail").replace("%e", msg));
      })
      .then(function () { fetchBtn.disabled = false; });
  }

  function save() {
    if (!available) { say("gb-set-warn", g("unavailable")); return; }
    var url = ctx.url(), key = ctx.key();
    if (!url) { say("gb-set-warn", g("urlNeeded")); return; }
    if (!key && !ctx.keyOptional && !keyStored()) { say("gb-set-warn", g("keyNeeded")); return; }
    saving = true;
    var req = { id: ctx.id, name: ctx.name, url: url, enabled: all.filter(function (m) { return on[m]; }) };
    if (key) { req.api_key = key; }
    var snapshot = {};
    all.forEach(function (m) { snapshot[m] = !!on[m]; });
    say("gb-set-hint", g("saving"));
    A.apiFetch("/api/llm/accounts", { method: "PUT", body: JSON.stringify(req) })
      .then(function (r) {
        return r.json().catch(function () { return {}; }).then(function (d) {
          if (!r.ok || !d.ok) { throw { error: d.error || ("http_status_" + r.status), detail: d.detail }; }
        });
      })
      .then(function () {
        account = { id: ctx.id, has_key: true, enabled: req.enabled };
        all.forEach(function (m) { savedOn[m] = !!snapshot[m]; });
        say("gb-set-hint gb-set-ok", g("saved"));
        if (A.loadModels) { return A.loadModels().then(function () { if (A.renderModelPickers) { A.renderModelPickers(); } }); }
      })
      .catch(function (e) {
        say("gb-set-warn", e && e.error ? failText("saveFail", e) : g("saveFail").replace("%e", String(e && e.message || e)));
        pendingSave = false;
        revertSwitches();
      })
      .then(function () {
        saving = false;
        if (pendingSave) { pendingSave = false; scheduleSave(); }
      });
  }

  fetchBtn.addEventListener("click", fetchModels);

  /* The accounts endpoint may not be ported in this build. Probe it once and
     say so in words instead of offering controls that cannot work. */
  A.apiFetch("/api/llm/accounts")
    .then(function (r) { if (!r.ok) { throw new Error("http_" + r.status); } return r.json(); })
    .then(function (d) {
      available = true;
      account = ((d && d.accounts) || []).filter(function (a) { return a.id === ctx.id; })[0] || null;
      if (account) {
        all = (account.enabled || []).slice();
        all.forEach(function (m) { on[m] = true; savedOn[m] = true; });
      }
      render();
      if (keyStored()) { autoLoad(); }
    })
    .catch(function (e) {
      account = null;
      fetchBtn.disabled = true;
      say("gb-set-warn", g("unavailable"));
      console.debug("gbKeyModels: key accounts unavailable", e);
    });
}

A._keyModels = { mount: mount };
})();
