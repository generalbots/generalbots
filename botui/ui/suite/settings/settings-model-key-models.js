"use strict";
/**
 * settings-model-key-models.js — "Models for this key" under the provider
 * setup screen in classic Settings > Model. Ported from gbgov's assistant
 * 10h module. discovery endpoints are best-effort: a missing backend shows an
 * honest "not available" notice instead of a broken button.
 */
(function () {
  var C = null;

  function t(key, fallback) { return C.t(key, fallback); }
  function esc(value) { return C.esc(value); }

  function mount(box, ctx) {
    var all = [], on = {}, filter = "", available = true;
    box.innerHTML = '<section class="gb-km">' +
      '<div class="gb-km-head"><div class="gb-prov-title gb-km-title">' + esc(t("mdl-km-title", "Models for this key")) + "</div>" +
      '<button type="button" class="btn-secondary btn-sm" id="gbKmFetch">' + esc(t("mdl-km-fetch", "Fetch models")) + "</button></div>" +
      '<p class="mdl-hint" id="gbKmHint">' + esc(t("mdl-km-hint", "Fetch the models this key unlocks and switch on the ones that should appear in the chat menu.")) + "</p>" +
      '<div id="gbKmBody"></div><div id="gbKmMsg" class="gb-prov-msg" role="status" aria-live="polite"></div></section>';
    var body = box.querySelector("#gbKmBody"), fetchBtn = box.querySelector("#gbKmFetch");

    function say(cls, text) {
      box.querySelector("#gbKmMsg").innerHTML = text ? '<p class="' + cls + '">' + esc(text) + "</p>" : "";
    }
    function missing(e) { return e && e.status === 404; }
    function unavailable() {
      available = false;
      fetchBtn.disabled = true;
      body.innerHTML = '<p class="mdl-hint">' + esc(t("mdl-km-unavailable", "Model discovery is not available on this server.")) + "</p>";
    }
    function ready() {
      if (!ctx.url()) { say("gb-prov-warn", t("mdl-prov-url-needed", "Enter the server address.")); return false; }
      if (!ctx.key() && !ctx.keyOptional) {
        say("gb-prov-warn", t("mdl-prov-key-needed", "Paste the API key."));
        return false;
      }
      return true;
    }
    function request() {
      var r = { id: ctx.id, url: ctx.url() }, key = ctx.key();
      if (key) r.api_key = key;
      return r;
    }
    function rows() {
      var q = filter.trim().toLowerCase();
      var shown = all.filter(function (m) { return !q || m.toLowerCase().indexOf(q) >= 0; });
      if (!shown.length) return '<p class="mdl-hint gb-km-none">' + esc(t("mdl-km-no-match", "No model by that name.")) + "</p>";
      return shown.map(function (m) {
        return '<label class="gb-km-row"><span class="gb-km-id">' + esc(m) + "</span>" +
          '<input type="checkbox" class="gb-km-toggle" data-model="' + esc(m) + '"' + (on[m] ? " checked" : "") + "/></label>";
      }).join("");
    }
    function wireRows() {
      box.querySelectorAll(".gb-km-row input").forEach(function (cb) {
        cb.addEventListener("change", function () {
          on[cb.getAttribute("data-model")] = cb.checked;
          box.querySelector("#gbKmCount").textContent = count();
        });
      });
    }
    function count() {
      return t("mdl-km-count", "%n of %t on")
        .replace("%n", String(all.filter(function (m) { return on[m]; }).length))
        .replace("%t", String(all.length));
    }
    function render() {
      if (!all.length) { body.innerHTML = ""; return; }
      var search = all.length > 12
        ? '<label class="gb-prov-search gb-km-filter"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">' +
          '<circle cx="11" cy="11" r="7"/><path d="M20 20l-3.5-3.5"/></svg>' +
          '<input type="search" id="gbKmFilter" placeholder="' + esc(t("mdl-km-filter", "Filter models")) +
          '" aria-label="' + esc(t("mdl-km-filter", "Filter models")) + '" value="' + esc(filter) + '"/></label>'
        : "";
      body.innerHTML = search +
        '<div class="gb-km-list" role="group" aria-label="' + esc(t("mdl-km-title", "Models for this key")) + '">' + rows() + "</div>" +
        '<div class="gb-km-foot"><span class="mdl-hint" id="gbKmCount">' + esc(count()) + "</span>" +
        '<button type="button" class="btn-primary btn-sm" id="gbKmSave">' + esc(t("mdl-km-save", "Save models")) + "</button></div>";
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
      if (!available || !ready()) return;
      fetchBtn.disabled = true;
      say("", t("mdl-km-fetching", "Fetching models…"));
      C.json("/api/llm/accounts/models", { method: "POST", body: JSON.stringify(request()) })
        .then(function (d) {
          var got = (d && d.models) || [];
          if (!got.length) {
            say("gb-prov-warn", d && d.error
              ? t("mdl-km-fetch-fail", "Could not fetch the models (%e).").replace("%e", C.reason(d.error))
              : t("mdl-km-empty", "The provider returned no models."));
            return;
          }
          all = all.filter(function (m) { return on[m] && got.indexOf(m) < 0; }).concat(got);
          say("", "");
          render();
        })
        .catch(function (e) {
          if (missing(e)) { unavailable(); return; }
          say("gb-prov-warn", t("mdl-km-fetch-fail", "Could not fetch the models (%e).").replace("%e", (e && e.message) || String(e)));
        })
        .then(function () { if (available) fetchBtn.disabled = false; });
    }
    function save() {
      if (!available || !ready()) return;
      var btn = box.querySelector("#gbKmSave"), r = request();
      r.name = ctx.name;
      r.enabled = all.filter(function (m) { return on[m]; });
      btn.disabled = true;
      say("", t("mdl-km-saving", "Saving…"));
      C.json("/api/llm/accounts", { method: "PUT", body: JSON.stringify(r) })
        .then(function () { say("gb-prov-ok", t("mdl-km-saved", "Models saved. The ones switched on appear in the chat menu.")); })
        .catch(function (e) {
          if (missing(e)) { unavailable(); return; }
          say("gb-prov-warn", t("mdl-km-save-fail", "Could not save (%e).").replace("%e", (e && (e.error || e.message)) || String(e)));
        })
        .then(function () { btn.disabled = false; });
    }
    fetchBtn.addEventListener("click", fetchModels);
    if (document.body.classList.contains("is-admin")) C.json("/api/llm/accounts")
      .then(function (d) {
        var account = ((d && d.accounts) || []).filter(function (a) { return a.id === ctx.id; })[0] || null;
        if (!account) return;
        if (account.has_key) {
          box.querySelector("#gbKmHint").textContent += " " + t("mdl-km-stored", "With the key field empty, the search uses the stored key.");
        }
        all = (account.enabled || []).slice();
        all.forEach(function (m) { on[m] = true; });
        render();
      })
      .catch(function (e) { if (missing(e)) unavailable(); });
  }

  window.GBSettingsKeyModels = {
    mount: function (box, ctx, callbacks) { C = callbacks; mount(box, ctx); }
  };
})();
