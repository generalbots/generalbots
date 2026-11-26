"use strict";
/**
 * settings-limits.js — Usage & limits in classic Settings. Ported from the
 * gbgov assistant's 10i_settings_limits.js and 10k_limits_refresh.js.
 *
 * Four engine types share one vocabulary (settings-engine-names.js); each
 * shows the tightest quota /api/llm/limits reports, with a Refresh control that
 * says when the numbers were last read. The endpoint is best-effort: a server
 * that does not ship it shows an honest notice instead of a broken panel.
 */
(function () {
  var NAMES = window.GBSettingsEngineNames;
  var KIND = { acp: "subscription", own_key: "api_key", free: "free", local: "local" };
  var WINDOW_KEY = {
    "5h": "lim-w-5h", weekly: "lim-w-weekly", daily: "lim-w-daily",
    requests: "lim-w-requests", tokens: "lim-w-tokens",
    input_tokens: "lim-w-input", output_tokens: "lim-w-output", plan: "lim-w-plan"
  };
  var WINDOW_FALLBACK = {
    "5h": "5-hour window", weekly: "Weekly limit", daily: "Daily limit",
    requests: "Requests", tokens: "Tokens", input_tokens: "Input tokens",
    output_tokens: "Output tokens", plan: "Plan quota"
  };
  var STATUS = { allowed: "lim-within", allowed_warning: "lim-close", rejected: "lim-full" };
  var NONE = {
    acp: "lim-none-acp", own_key: "lim-none-key", free: "lim-none-free", local: "lim-none-local"
  };
  var reported = [];
  var selected = "acp";
  var lastAt = 0;
  var refreshUi = null;

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function esc(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }
  function token() {
    if (typeof window.getGBAccessToken === "function") {
      var value = window.getGBAccessToken();
      if (value) { return value; }
    }
    return localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") || "";
  }
  function json(path, options) {
    var headers = { "Content-Type": "application/json" };
    var value = token();
    if (value) { headers.Authorization = "Bearer " + value; }
    return fetch(path, Object.assign({ headers: headers }, options || {})).then(function (response) {
      return response.json().catch(function () { return {}; }).then(function (body) {
        if (!response.ok) {
          var err = new Error(body.error || "HTTP " + response.status);
          err.status = response.status;
          throw err;
        }
        return body;
      });
    });
  }

  /* ---- pure helpers (published for the offline self-test) ---- */
  function pctOf(windowData) {
    if (typeof windowData.used_percent === "number") {
      return Math.max(0, Math.min(100, Math.round(windowData.used_percent)));
    }
    if (typeof windowData.limit === "number" && windowData.limit > 0 && typeof windowData.remaining === "number") {
      return Math.max(0, Math.min(100, Math.round((windowData.limit - windowData.remaining) / windowData.limit * 100)));
    }
    return null;
  }
  function tightest(engine) {
    var best = null;
    ((engine && engine.windows) || []).forEach(function (w) {
      var pct = pctOf(w);
      if (pct !== null && (best === null || pct > best)) { best = pct; }
    });
    return best;
  }
  function statusText(status) {
    var key = STATUS[status];
    return key ? t(key, "") : "";
  }
  function windowName(name) {
    var key = WINDOW_KEY[name];
    return key ? t(key, WINDOW_FALLBACK[name] || String(name || "")) : String(name || "");
  }
  function hhmm(date) {
    return String(date.getHours()).padStart(2, "0") + ":" + String(date.getMinutes()).padStart(2, "0");
  }
  function updatedText(at) {
    return at ? t("lim-updated", "Updated at ") + hhmm(new Date(at)) : "";
  }
  function resetText(iso) {
    var date = iso ? new Date(iso) : null;
    var now = new Date();
    if (!date || isNaN(date.getTime()) || date <= now) { return ""; }
    if (date.getHours() === 0 && date.getMinutes() === 0 && date - now <= 86400000) {
      return t("lim-renews-midnight", "RENEWS AT MIDNIGHT");
    }
    if (date.toDateString() === now.toDateString()) {
      return t("lim-renews-today", "RENEWS AT %s").replace("%s", hhmm(date));
    }
    return t("lim-renew", "RENEWS %s").replace("%s", date.toLocaleDateString(undefined, { day: "2-digit", month: "2-digit" }) + " " + hhmm(date));
  }
  function valueText(w) {
    var hasLimit = typeof w.limit === "number";
    var hasRemaining = typeof w.remaining === "number";
    if (hasLimit && hasRemaining) {
      return t("lim-left", "%n of %t left").replace("%n", String(w.remaining)).replace("%t", String(w.limit));
    }
    if (hasRemaining) { return t("lim-remaining", "%n left").replace("%n", String(w.remaining)); }
    if (hasLimit) { return t("lim-limit", "Limit %n").replace("%n", String(w.limit)); }
    return "";
  }
  function bars(engine) {
    return ((engine && engine.windows) || []).map(function (w) {
      var pct = pctOf(w);
      var said = statusText(w.status);
      if (pct === null && !said) { return null; }
      return {
        label: windowName(w.name),
        pct: pct,
        value: [said, valueText(w)].filter(Boolean).join(" · "),
        reset: resetText(w.resets_at)
      };
    }).filter(Boolean);
  }
  function engineName(type, tier) {
    return NAMES && typeof NAMES.engineName === "function" ? NAMES.engineName(type, tier) : type;
  }

  /* ---- rendering ---- */
  function ofKind(type) {
    return reported.filter(function (e) { return e.kind === KIND[type]; });
  }
  function chipPercent(type) {
    var list = ofKind(type);
    if (type === "local" || (list.length && list.every(function (e) { return e.unlimited; }))) { return "∞"; }
    var top = null;
    list.forEach(function (e) {
      var pct = tightest(e);
      if (pct !== null && (top === null || pct > top)) { top = pct; }
    });
    return top === null ? "—" : top + "%";
  }
  function drawChips(root) {
    if (!root) { return; }
    root.innerHTML = NAMES.TYPES.map(function (type) {
      var pct = chipPercent(type);
      var cls = pct === "∞" ? "is-inf" : pct === "—" ? "is-none" : parseInt(pct, 10) >= 85 ? "is-high" : "is-low";
      return '<button type="button" class="gb-lim-type ' + cls + '" data-limit="' + esc(type) + '"' +
        ' aria-pressed="' + (selected === type ? "true" : "false") + '">' +
        '<strong>' + esc(engineName(type, "group")) + "</strong>" +
        '<span class="gb-lim-pct">' + esc(pct) + "</span></button>";
    }).join("");
  }
  function barRow(bar) {
    var pct = bar.pct;
    var width = pct === null ? 0 : pct;
    return '<div class="gb-lim-bar"><div class="gb-lim-bar-top"><span>' + esc(bar.label) + "</span>" +
      '<b>' + (pct === null ? "" : esc(String(pct) + "%")) + "</b></div>" +
      '<div class="gb-lim-track"><i style="width:' + width + '%"></i></div>' +
      '<div class="gb-lim-bar-foot"><span>' + esc(bar.value) + "</span><span>" + esc(bar.reset) + "</span></div></div>";
  }
  function drawDetail(root) {
    if (!root) { return; }
    var name = document.getElementById("gbLimitsName");
    if (name) { name.textContent = engineName(selected, "group"); }
    var list = ofKind(selected);
    var blocks = [];
    list.forEach(function (engine) {
      var rows = bars(engine);
      var head = '<div class="gb-lim-who"><strong>' + esc(engine.label || engine.provider || selected) + "</strong>";
      if (engine.unlimited) { head += '<span class="gb-lim-inf">∞</span>'; }
      head += "</div>";
      blocks.push('<div class="gb-lim-engine">' + head +
        (rows.length ? rows.map(barRow).join("")
          : '<p class="gb-lim-none">' + esc(t("lim-no-data", "This engine has not reported limits yet.")) + "</p>") +
        "</div>");
    });
    if (selected === "local") {
      blocks.unshift('<p class="gb-lim-none">' + esc(t(NONE.local, "The local model has no usage quota. The limit is this computer's capacity.")) + "</p>");
    }
    if (!blocks.length) {
      blocks.push('<p class="gb-lim-none">' + esc(t(NONE[selected], "This engine has not reported limits yet.")) + "</p>");
    }
    root.innerHTML = blocks.join("");
  }
  function refreshControl(host, onDone) {
    if (!host) { return null; }
    var wrap = document.createElement("div");
    wrap.className = "gb-lim-refresh";
    var button = document.createElement("button");
    button.type = "button";
    button.className = "btn-secondary btn-sm gb-lim-refresh-btn";
    button.title = t("lim-refresh-tip", "Read the limits again now");
    button.textContent = t("lim-refresh", "Refresh");
    var status = document.createElement("span");
    status.className = "gb-lim-refresh-at";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.textContent = updatedText(lastAt);
    button.addEventListener("click", function () {
      if (button.disabled) { return; }
      button.disabled = true;
      button.setAttribute("aria-busy", "true");
      status.textContent = t("lim-refreshing", "Refreshing…");
      load(true).then(function (data) {
        button.disabled = false;
        button.removeAttribute("aria-busy");
        status.textContent = data && data.failed
          ? t("lim-refresh-fail", "Could not refresh now.")
          : updatedText(lastAt);
        if (onDone) { onDone(data); }
      });
    });
    wrap.appendChild(button);
    wrap.appendChild(status);
    host.innerHTML = "";
    host.appendChild(wrap);
    return { sync: function () { if (!button.disabled) { status.textContent = updatedText(lastAt); } } };
  }

  function load(fresh) {
    var status = document.getElementById("gbLimitsStatus");
    if (status) { status.textContent = t("lim-loading", "Loading limits…"); }
    return Promise.all([
      json(fresh ? "/api/llm/limits?refresh=1" : "/api/llm/limits"),
      json("/api/llm/config").catch(function () { return null; })
    ]).then(function (results) {
      reported = (results[0] && results[0].engines) || [];
      lastAt = Date.now();
      if (status) { status.textContent = ""; }
      if (refreshUi) { refreshUi.sync(); }
      repaint();
      return { engines: reported, at: lastAt };
    }).catch(function (error) {
      reported = [];
      if (status) {
        status.textContent = error && error.status === 404
          ? t("lim-unavailable", "This server does not expose usage limits.")
          : t("lim-load-fail", "Could not load limits: %e").replace("%e", (error && error.message) || String(error));
      }
      repaint();
      return { engines: [], failed: true };
    });
  }
  function repaint() {
    drawChips(document.getElementById("gbLimitsTypes"));
    drawDetail(document.getElementById("gbLimitsDetail"));
  }
  function bind() {
    var root = document.getElementById("gbLimitsTypes");
    if (!root || root.dataset.bound) { return; }
    root.dataset.bound = "1";
    document.addEventListener("click", function (event) {
      var chip = event.target && event.target.closest ? event.target.closest(".gb-lim-type") : null;
      if (!chip || !chip.closest("#gbLimitsTypes")) { return; }
      selected = chip.getAttribute("data-limit");
      drawChips(document.getElementById("gbLimitsTypes"));
      drawDetail(document.getElementById("gbLimitsDetail"));
    });
    refreshUi = refreshControl(document.getElementById("gbLimitsRefresh"), repaint);
  }

  window.loadLimitsPanel = function () {
    if (!document.getElementById("limits-section")) { return Promise.resolve(); }
    bind();
    return load(false);
  };

  var previousHook = window.GBSettingsOnSection;
  window.GBSettingsOnSection = function (section) {
    if (typeof previousHook === "function") {
      try { previousHook(section); } catch (e) { /* an earlier hook is optional */ }
    }
    if (section === "limits") { window.loadLimitsPanel(); }
  };

  window.GBSettingsLimits = {
    pctOf: pctOf, tightest: tightest, statusText: statusText, windowName: windowName,
    updatedText: updatedText, resetText: resetText, valueText: valueText, bars: bars,
    KIND: KIND
  };
})();
