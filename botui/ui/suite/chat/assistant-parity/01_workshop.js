"use strict";
/**
 * 01_workshop.js — chat-side port of the gbgov assistant "Assistant workshop".
 * Mounts one button in chat's existing header toolbar and opens a dialog with
 * the setup dashboard, the two-question chooser, the four-mode comparison and
 * a hardware verdict. Chat's DOM, renderer and native model picker stay intact.
 */
(function () {
  var P = window.GBChatAssistantParity = window.GBChatAssistantParity || {};
  function T(key, fallback) { return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback; }
  function esc(value) { return String(value == null ? "" : value).replace(/&/g, "&amp;").replace(/</g, "&lt;")
    .replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#39;"); }
  function gb(value) { return String(Math.round((Number(value) || 0) * 10) / 10) + " GB"; }
  function pairText(pair) { return T(pair[0], pair[1]); }
  function authHeaders() {
    try {
      if (window.GBShared && window.GBShared.auth && typeof window.GBShared.auth.headers === "function") {
        return window.GBShared.auth.headers();
      }
    } catch (error) { /* fall through to the chat token */ }
    try {
      var token = (typeof window.getChatAccessToken === "function" && window.getChatAccessToken()) ||
        (window.GBSecurity && window.GBSecurity.getToken && window.GBSecurity.getToken()) ||
        sessionStorage.getItem("gb-access-token") || "";
      return token ? { Authorization: "Bearer " + token } : {};
    } catch (error) { return {}; }
  }
  function getJson(path) {
    try {
      return fetch(path, { cache: "no-store", credentials: "same-origin", headers: authHeaders() })
        .then(function (response) { return response.ok
          ? response.json().then(function (data) { return { data: data }; }) : { status: response.status }; })
        .catch(function (error) { return { error: (error && error.message) || String(error) }; });
    } catch (error) { return Promise.resolve({ error: (error && error.message) || String(error) }); }
  }
  function findMode(modes, id) {
    var list = Array.isArray(modes) ? modes : [];
    for (var i = 0; i < list.length; i++) { if (list[i] && list[i].id === id) return list[i]; }
    return {};
  }
  function load() {
    return Promise.all([
      getJson("/api/llm/config"), getJson("/api/llm/config/modes"), getJson("/api/llm/runtime/status")
    ]).then(function (res) {
      var cfg = res[0].data || {};
      var modes = (res[1].data && (res[1].data.modes || res[1].data)) || [];
      var rt = res[2].data || {};
      var hw = findMode(modes, "local").hardware || rt.hardware || {};
      var failed = res[0].status || res[0].error || "";
      var data = { cfg: cfg, modes: modes, rt: rt, hw: hw, error: failed ? String(failed) : "" };
      data.verdict = verdict(data);
      return data;
    });
  }
  function announce(root, message) {
    var status = root && root.querySelector(".gb-parity-status");
    if (status) status.textContent = message || "";
  }
  function reload(root) {
    var btn = root && root.querySelector("[data-gb-parity-retry]");
    if (btn) btn.disabled = true;
    announce(root, T("chat-parity-loading", "Loading the model setup…"));
    load().then(function (data) { if (root && root.isConnected) render(root, data); });
  }

  /* Icons are shapes only — no user-facing words. */
  var ICONS = {
    wrench: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>',
    local: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>',
    own_key: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>',
    acp: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="4" y="8" width="16" height="12" rx="3"/><path d="M12 8V4M8 4h8M9 14h.01M15 14h.01"/></svg>',
    free: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>',
    lock: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>',
    globe: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15 15 0 0 1 0 20M12 2a15 15 0 0 0 0 20"/></svg>',
    server: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="2" y="3" width="20" height="7" rx="2"/><rect x="2" y="14" width="20" height="7" rx="2"/><path d="M6 6.5h.01M6 17.5h.01"/></svg>',
    none: '<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="12" r="10"/><path d="M9.1 9a3 3 0 0 1 5.8 1c0 2-3 3-3 3M12 17h.01"/></svg>'
  };
  var MODES = ["acp", "local", "own_key", "free"];
  var MODE_TEXT = { acp: ["chat-parity-where-acp", "Agent (ACP)"], local: ["chat-parity-where-local", "On this computer"],
    own_key: ["chat-parity-where-own", "Own server or key"], free: ["chat-parity-where-free", "Free model"] };
  var MODE_SUB = { acp: ["chat-parity-sub-acp", "Your agent subscription"], local: ["chat-parity-sub-local", "Private and offline"],
    own_key: ["chat-parity-sub-own", "Your organization’s"], free: ["chat-parity-sub-free", "Ready to use"] };
  var ROWS = [["privacy", "chat-parity-row-privacy", "Your text"], ["offline", "chat-parity-row-offline", "Without internet"],
    ["cost", "chat-parity-row-cost", "Cost"], ["docs", "chat-parity-row-docs", "PDF, Word and text"],
    ["ocr", "chat-parity-row-ocr", "Text in images and scans"], ["web", "chat-parity-row-web", "Web search"],
    ["photos", "chat-parity-row-photos", "Understand photos and charts"], ["needs", "chat-parity-row-needs", "You need"]];

  function machineFits(hw) { return !!(hw && hw.cuda_usable && Number(hw.vram_gb) >= 6); }
  function isWindows() { return /Windows/i.test(navigator.userAgent || ""); }
  function webSearchOn() {
    try { var state = window.ChatState || window.GBChatState || {};
      return !!(state.webSearchOn || (state.webSearch && state.webSearch.enabled));
    } catch (error) { return false; }
  }
  function shortName(model) {
    var value = String(model || "");
    if (!value) return "—";
    var slash = value.lastIndexOf("/"); if (slash >= 0) value = value.slice(slash + 1);
    var colon = value.indexOf(":"); if (colon > 0) value = value.slice(0, colon);
    return value.replace(/\.gguf$/i, "") || "—";
  }
  function mark(kind) { return '<span class="gb-parity-mark gb-parity-mark-' + esc(kind) + '" aria-hidden="true"></span>'; }
  function gauge(level, label) {
    var segments = "";
    for (var i = 1; i <= 5; i++) { segments += '<i class="' + (i <= level ? "gb-parity-on" : "") + '"></i>'; }
    return '<span class="gb-parity-gauge gb-parity-level-' + level + '" role="img" aria-label="' +
      esc(label + ": " + level + "/5") + '">' + segments + "</span>";
  }
  function chip(on, key, fallback) {
    return '<span class="gb-parity-chip' + (on ? " gb-parity-on" : "") + '">' + mark(on ? "yes" : "no") + esc(T(key, fallback)) + "</span>";
  }
  function privacyLevel(mode, scope) {
    if (mode === "local" || scope === "machine") return 5;
    if (scope === "network") return 4;
    if (mode === "own_key") return 2;
    return 1;
  }
  function privacyText(cfg) {
    var scope = cfg && cfg.endpoint_scope, host = (cfg && cfg.endpoint_host) || "";
    if (scope === "machine") return esc(T("chat-parity-priv-local", "Your conversations stay on this computer. Nothing is sent outside."));
    if (scope === "network") return esc(T("chat-parity-priv-net", "Your conversations are sent to your organization’s server (%h).").replace("%h", host));
    if (scope === "external") return esc(T("chat-parity-priv-ext", "Your conversations are sent to an external host (%h).").replace("%h", host));
    return "";
  }
  function verdict(data) {
    var hw = data.hw || {};
    if (!hw.device && !hw.cuda_usable && !hw.ram_gb) {
      return { tone: "none", short: T("chat-parity-verdict-none", "This computer could not be examined"), long: "" };
    }
    if (!hw.cuda_usable) {
      return { tone: "bad", short: T("chat-parity-verdict-bad-gpu", "No suitable graphics card"),
        long: T("chat-parity-verdict-bad-gpu-long", "No NVIDIA graphics card with CUDA was found. Local models would run on the processor, much slower: one answer can take minutes. To keep your text inside your network, prefer your organization’s server.") };
    }
    var have = Number(hw.vram_gb) || 0;
    if (have < 6) {
      return { tone: "bad", short: T("chat-parity-verdict-low-vram", "Card with too little memory (%v)").replace("%v", gb(have)),
        long: T("chat-parity-verdict-low-vram-long", "The card has %v of video memory; the smallest local model needs %m.")
          .replace("%v", gb(have)).replace("%m", "6 GB") };
    }
    var good = have >= 10;
    return { tone: good ? "good" : "mid", short: T(good ? "chat-parity-verdict-good" : "chat-parity-verdict-mid",
      good ? "Good for local models" : "Suitable, at the limit"),
      long: T("chat-parity-verdict-capacity", "This card runs models that need up to %v of video memory.").replace("%v", gb(have)) };
  }
  function cell(kind, key, fallback) { return { k: kind, key: key, fb: fallback }; }
  function facts(mode, data) {
    var cfg = data.cfg || {}, scope = cfg.mode === "own_key" ? cfg.endpoint_scope : "";
    var ocr = isWindows() ? cell("yes", "chat-parity-reads", "Reads") : cell("no", "chat-parity-no-read", "Does not read");
    var web = cell("part", "chat-parity-optional", "Optional, you turn it on");
    var photo = cell("no", "chat-parity-not-yet", "Not yet");
    if (mode === "local") {
      var note = machineFits(data.hw)
        ? T("chat-parity-machine-ok", "This computer has: %d, %v GB").replace("%d", data.hw.device || "").replace("%v", String(Number(data.hw.vram_gb) || 0))
        : T("chat-parity-machine-slow", "This computer has no suitable graphics card: it would be slow");
      if (data.verdict && data.verdict.short) { note += " · " + data.verdict.short; }
      return { privacy: cell("yes", "chat-parity-stay-here", "Stays on this computer"), offline: cell("yes", "chat-parity-works", "Works"),
        cost: cell("yes", "chat-parity-cost-free", "Free"), docs: cell("yes", "chat-parity-reads", "Reads"), ocr: ocr, web: web, photos: photo,
        needs: cell(machineFits(data.hw) ? "yes" : "no", "chat-parity-need-local",
          "A graphics card with 6 GB or more and a 4 to 6 GB download"), note: note };
    }
    if (mode === "own_key") {
      return { privacy: cell(scope === "network" ? "yes" : "part", scope === "network" ? "chat-parity-go-net" : "chat-parity-go-server",
          scope === "network" ? "Stays in your organization’s network" : "Goes to the server you set"),
        offline: cell("part", "chat-parity-only-net", "Only with a server on your network"),
        cost: cell("part", "chat-parity-cost-contract", "Per your contract"), docs: cell("yes", "chat-parity-reads", "Reads"),
        ocr: ocr, web: web, photos: photo, needs: cell("part", "chat-parity-need-own", "The server address and a key") };
    }
    if (mode === "acp") {
      var agent = cell("part", "chat-parity-depends-agent", "Depends on the agent");
      return { privacy: cell("part", "chat-parity-go-agent", "Goes to the agent’s provider"), offline: cell("no", "chat-parity-no-work", "Does not work"),
        cost: cell("part", "chat-parity-cost-sub", "Your subscription"), docs: cell("yes", "chat-parity-agent-does", "The agent does it"),
        ocr: agent, web: cell("yes", "chat-parity-agent-does", "The agent does it"), photos: agent,
        needs: cell("part", "chat-parity-need-acp", "Install the agent and sign in to your account") };
    }
    return { privacy: cell("no", "chat-parity-go-ext", "Goes to an external provider"), offline: cell("no", "chat-parity-no-work", "Does not work"),
      cost: cell("yes", "chat-parity-cost-free-limit", "Free, with usage limits"), docs: cell("yes", "chat-parity-reads", "Reads"),
      ocr: ocr, web: web, photos: photo, needs: cell("yes", "chat-parity-need-free", "Nothing") };
  }

  /* ---- The four blocks ---- */
  function dashboard(data) {
    var cfg = data.cfg || {}, mode = cfg.mode || "", where = mode ? pairText(MODE_TEXT[mode] || MODE_TEXT.free) : "";
    var html = '<section class="gb-parity-section gb-parity-dash"><div class="gb-parity-dash-main"><div><span class="gb-parity-kicker">' +
      esc(T("chat-parity-engine", "Engine in use")) + '</span><div class="gb-parity-engine">' + esc(shortName(cfg.model)) + "</div></div>" +
      (where ? '<span class="gb-parity-where">' + (ICONS[mode] || "") + esc(where) + "</span>" : "") + "</div>";
    if (mode) {
      var privacy = T("chat-parity-privacy", "Privacy");
      html += '<div class="gb-parity-row"><span class="gb-parity-label">' + esc(privacy) + "</span>" +
        gauge(privacyLevel(mode, cfg.endpoint_scope), privacy) + '<span class="gb-parity-text">' + privacyText(cfg) + "</span></div>";
    }
    var chips = chip(true, "chat-parity-feature-docs", "Documents") + chip(isWindows(), "chat-parity-feature-ocr", "Text in images") +
      chip(webSearchOn(), "chat-parity-feature-web", "Web search") + chip(false, "chat-parity-feature-photos", "Photos and charts");
    html += '<div class="gb-parity-row"><span class="gb-parity-label">' + esc(T("chat-parity-features", "Features on")) +
      '</span><span class="gb-parity-chips">' + chips + "</span></div></section>";
    return html;
  }
  function guide() {
    return '<section class="gb-parity-section gb-parity-guide"><div class="gb-parity-guide-head"><strong>' +
      esc(T("chat-parity-guide", "Not sure which to pick?")) + '</strong> <span class="gb-parity-hint">' +
      esc(T("chat-parity-guide-sub", "Answer two questions.")) + '</span></div><div class="gb-parity-guide-body" aria-live="polite"></div></section>';
  }
  function compare(data) {
    var active = (data.cfg && data.cfg.mode) || "", all = {};
    MODES.forEach(function (mode) { all[mode] = facts(mode, data); });
    var html = '<h3 class="gb-parity-section-title">' + esc(T("chat-parity-where", "Choose the engine")) + '</h3><p class="gb-parity-hint">' +
      esc(T("chat-parity-where-sub", "Compare the four ways to use the assistant. Select one to see what it needs.")) +
      '</p><div class="gb-parity-compare"><div class="gb-parity-grid" role="table"><div class="gb-parity-corner" role="columnheader"></div>';
    MODES.forEach(function (mode) {
      var available = findMode(data.modes, mode).available !== false;
      html += '<button type="button" class="gb-parity-head" data-mode="' + mode + '" data-col="' + mode + '"' +
        (active === mode ? ' data-inuse="1"' : "") + (available ? "" : " disabled") + ' role="columnheader">' +
        (ICONS[mode] || "") + '<span class="gb-parity-head-label">' + esc(pairText(MODE_TEXT[mode])) + '</span><span class="gb-parity-head-sub">' +
        esc(pairText(MODE_SUB[mode])) + '</span><span class="gb-parity-badges">' +
        (active === mode ? '<span class="gb-parity-badge gb-parity-badge-inuse">' + esc(T("chat-parity-in-use", "In use")) + "</span>" : "") +
        '<span class="gb-parity-badge gb-parity-badge-sugg">' + esc(T("chat-parity-suggested", "Suggested")) + "</span>" +
        (mode === "acp" ? '<span class="gb-parity-badge gb-parity-badge-soon">' + esc(T("chat-parity-coming-soon", "Coming soon")) + "</span>" : "") +
        "</span></button>";
    });
    ROWS.forEach(function (row) {
      html += '<div class="gb-parity-rowlabel" role="rowheader">' + esc(T(row[1], row[2])) + "</div>";
      MODES.forEach(function (mode) {
        var fact = all[mode][row[0]], note = row[0] === "needs" && all[mode].note
          ? '<small class="gb-parity-note">' + esc(all[mode].note) + "</small>" : "";
        html += '<div class="gb-parity-cell" data-col="' + mode + '" role="cell">' + mark(fact.k) + "<span>" +
          esc(T(fact.key, fact.fb)) + note + "</span></div>";
      });
    });
    return html + "</div></div>";
  }
  function mrow(label, value) {
    return '<div class="gb-parity-mrow"><span class="gb-parity-mkey">' + esc(label) + '</span><div class="gb-parity-mval">' + value + "</div></div>";
  }
  function machineCard(data) {
    var hw = data.hw || {}, v = data.verdict || verdict(data);
    var html = '<section class="gb-parity-section gb-parity-machine"><div class="gb-parity-mhead"><span class="gb-parity-kicker">' +
      esc(T("chat-parity-machine", "Your computer")) + '</span><span class="gb-parity-hint">' +
      esc(T("chat-parity-checked-now", "checked just now")) + "</span></div>";
    html += mrow(T("chat-parity-graphics-card", "Graphics card"), "<strong>" + esc(hw.device || T("chat-parity-none-found", "none found")) + "</strong>");
    html += mrow("CUDA", '<span class="gb-parity-inline">' + (hw.cuda_usable
      ? mark("yes") + esc(T("chat-parity-cuda-yes", "available: the model uses the card"))
      : mark("no") + esc(T("chat-parity-cuda-no", "unavailable: the model would use the processor"))) + "</span>");
    if (Number(hw.vram_gb)) { html += mrow(T("chat-parity-video-memory", "Video memory"), "<strong>" + esc(gb(hw.vram_gb)) + "</strong>"); }
    html += mrow(T("chat-parity-system-memory", "System memory"), esc(gb(hw.ram_gb)));
    if (hw.driver) { html += mrow(T("chat-parity-driver", "Driver"), esc(hw.driver)); }
    html += '<div class="gb-parity-verdict gb-parity-' + esc(v.tone) + '"><strong>' + esc(v.short) + "</strong>" +
      (v.long ? "<span>" + esc(v.long) + "</span>" : "") + "</div><p class=\"gb-parity-rule\">" +
      esc(T("chat-parity-rule", "How we measure: each model needs a minimum of video memory (the model file plus room for the conversation), from 6 GB for the smallest to 32 GB for the largest. With 25% more than the minimum it runs comfortably.")) + "</p></section>";
    return html;
  }

  /* ---- Guide behaviour ---- */
  function bindGuide(root, data) {
    var box = root && root.querySelector(".gb-parity-guide-body");
    if (!box) return;
    var hw = data.hw || {};
    function choice(index, icon, title, desc) {
      return '<button type="button" class="gb-parity-choice" data-a="' + index + '"><span class="gb-parity-choice-icon">' + icon +
        '</span><span class="gb-parity-choice-text"><strong>' + esc(title) + "</strong><small>" + esc(desc) + "</small></span></button>";
    }
    function ask(step, question, help, options, back) {
      box.innerHTML = '<div class="gb-parity-step"><span class="gb-parity-dots"><i class="gb-parity-on"></i><i class="' +
        (step > 1 ? "gb-parity-on" : "") + '"></i></span>' + esc(T("chat-parity-step", "Question %n of 2").replace("%n", String(step))) +
        (back ? '<button type="button" class="gb-parity-back">' + esc(T("chat-parity-back", "Back")) + "</button>" : "") +
        '</div><p class="gb-parity-q">' + esc(question) + '</p><p class="gb-parity-hint gb-parity-help">' + esc(help) +
        '</p><div class="gb-parity-choices">' + options.map(function (option, index) { return choice(index, option.icon, option.title, option.desc); }).join("") + "</div>";
      options.forEach(function (option, index) {
        var button = box.querySelector('[data-a="' + index + '"]');
        if (button) button.addEventListener("click", option.go);
      });
      if (back) { var backButton = box.querySelector(".gb-parity-back"); if (backButton) backButton.addEventListener("click", back); }
      var first = box.querySelector(".gb-parity-choice");
      if (first && step > 1) { first.focus(); }
    }
    function result(mode, text) {
      var grid = root.querySelector(".gb-parity-grid");
      if (grid) { grid.setAttribute("data-rec", mode); if (grid.scrollIntoView) { grid.scrollIntoView({ block: "nearest" }); } }
      var fact = facts(mode, data);
      var gains = ROWS.filter(function (row) { return ["privacy", "offline", "cost", "needs"].indexOf(row[0]) >= 0; })
        .map(function (row) { var value = fact[row[0]];
          return "<li>" + mark(value.k) + "<span><b>" + esc(T(row[1], row[2])) + ":</b> " + esc(T(value.key, value.fb)) + "</span></li>"; }).join("");
      box.innerHTML = '<div class="gb-parity-result"><span class="gb-parity-kicker">' +
        esc(T("chat-parity-pick", "Our suggestion for you")) + '</span><div class="gb-parity-result-name">' + (ICONS[mode] || "") +
        esc(pairText(MODE_TEXT[mode])) + '</div><p class="gb-parity-rec">' + esc(text) + '</p><p class="gb-parity-gains-title">' +
        esc(T("chat-parity-gains", "What you get with this option")) + '</p><ul class="gb-parity-gains">' + gains +
        '</ul><div class="gb-parity-actions"><button type="button" class="gb-parity-btn gb-parity-btn-accent" data-go="1">' +
        esc(T("chat-parity-show-me", "See how to turn it on")) + '</button><button type="button" class="gb-parity-btn" data-again="1">' +
        esc(T("chat-parity-restart", "Start over")) + "</button></div></div>";
      var go = box.querySelector('[data-go="1"]');
      if (go) { go.addEventListener("click", function () {
        if (grid) { grid.setAttribute("data-view", mode); }
        announce(root, pairText(MODE_TEXT[mode]) + " — " + T(fact.needs.key, fact.needs.fb));
      }); }
      var again = box.querySelector('[data-again="1"]');
      if (again) { again.addEventListener("click", start); }
      announce(root, pairText(MODE_TEXT[mode]) + " — " + T("chat-parity-pick", "Our suggestion for you"));
    }
    function secretBranch() {
      ask(2, T("chat-parity-q2-secret", "Does your organization have an AI server inside its internal network?"),
        T("chat-parity-q2-secret-help", "Ask your IT team. It is an address such as https://ai.agency.gov."), [
          { icon: ICONS.server, title: T("chat-parity-q2-yes", "Yes"), desc: T("chat-parity-q2-secret-yes-desc", "IT provides the address and the key"),
            go: function () { result("own_key", T("chat-parity-rec-own-net", "Use your organization’s server: your text stays inside the network, and IT looks after the capacity.")); } },
          { icon: ICONS.local, title: T("chat-parity-q2-no", "No, or I don’t know"), desc: T("chat-parity-q2-secret-no-desc", "Use this computer only"),
            go: function () {
              if (!machineFits(hw)) { result("own_key", T("chat-parity-rec-local-slow", "This machine has no graphics card for a local model. Use your organization’s server, inside your network.")); return; }
              result("local", Number(hw.vram_gb) >= 10
                ? T("chat-parity-rec-local", "Use On this computer: nothing leaves it, and this machine has a graphics card for it.")
                : T("chat-parity-rec-local-tight", "Use On this computer: nothing leaves it. The graphics card handles the smaller models; close other heavy programs first."));
            } }
        ], start);
    }
    function openBranch() {
      ask(2, T("chat-parity-q2-open", "Does your organization provide an AI server or key?"),
        T("chat-parity-q2-open-help", "For example an OpenAI or Azure key your organization pays for, or its own server."), [
          { icon: ICONS.server, title: T("chat-parity-q2-yes", "Yes"), desc: T("chat-parity-q2-yes-desc", "I have the address and the key"),
            go: function () { result("own_key", T("chat-parity-rec-own", "Use your organization’s server or key: the contract sets the cost and the limits.")); } },
          { icon: ICONS.none, title: T("chat-parity-q2-no", "No, or I don’t know"), desc: T("chat-parity-q2-no-desc", "I want to start without setting anything up"),
            go: function () { result("free", T("chat-parity-rec-free", "Start with the free model: nothing to set up. For confidential matters, prefer On this computer.")); } }
        ], start);
    }
    function start() {
      var grid = root.querySelector(".gb-parity-grid");
      if (grid) { grid.removeAttribute("data-rec"); }
      ask(1, T("chat-parity-q1", "Can what you write leave this computer?"),
        T("chat-parity-q1-help", "Think of case files, personal data and internal documents. If in doubt, assume they cannot."), [
          { icon: ICONS.lock, title: T("chat-parity-q1-no", "No, it must stay here"), desc: T("chat-parity-q1-no-desc", "Case files, personal data, internal documents"), go: secretBranch },
          { icon: ICONS.globe, title: T("chat-parity-q1-yes", "Yes, it can"), desc: T("chat-parity-q1-yes-desc", "Public text, drafts, research"), go: openBranch }
        ], null);
    }
    start();
  }
  function bindCompare(root, data) {
    var grid = root && root.querySelector(".gb-parity-grid");
    if (!grid) return;
    Array.prototype.forEach.call(root.querySelectorAll(".gb-parity-head"), function (button) {
      button.addEventListener("click", function () {
        var mode = button.getAttribute("data-mode");
        if (!mode) return;
        grid.setAttribute("data-view", mode);
        var need = facts(mode, data).needs;
        announce(root, pairText(MODE_TEXT[mode]) + " — " + T(need.key, need.fb));
      });
    });
  }

  /* ---- Styles: scoped to gb-parity-* so chat's own look is untouched ---- */
  var STYLE_ID = "gb-parity-workshop-styles";
  function ensureStyles() {
    if (document.getElementById(STYLE_ID)) return;
    var css = [
      '.gb-parity-workshop-btn{display:inline-flex;align-items:center;justify-content:center;width:28px;height:28px;padding:0;border:0;border-radius:6px;background:transparent;color:inherit;cursor:pointer}.gb-parity-workshop-btn:hover{background:var(--id-accent-soft,rgba(0,0,0,.06))}.gb-parity-workshop-btn:focus-visible{outline:2px solid var(--id-focus,#2563eb);outline-offset:2px}',
      '.gb-parity-workshop{position:fixed;inset:0;z-index:2147483000;display:flex;align-items:center;justify-content:center}.gb-parity-backdrop{position:absolute;inset:0;background:rgba(0,0,0,.45)}.gb-parity-panel{position:relative;width:calc(100% - 32px);max-width:960px;max-height:88vh;overflow:auto;padding:20px 22px;border-radius:14px;background:var(--id-surface-solid,#fff);color:var(--id-ink,#1c2530);box-shadow:0 18px 60px rgba(0,0,0,.28)}',
      '.gb-parity-head{display:flex;align-items:flex-start;justify-content:space-between;gap:16px;margin-bottom:8px}.gb-parity-title{margin:0;font-size:var(--id-text-2xl,20px);font-weight:600}.gb-parity-lead{margin:2px 0 0;font-size:var(--id-text-sm,13px);color:var(--id-muted,#64748b)}.gb-parity-close{flex:none;width:30px;height:30px;border:0;border-radius:6px;background:transparent;color:inherit;font-size:20px;line-height:1;cursor:pointer}.gb-parity-close:hover{background:var(--id-accent-soft,rgba(0,0,0,.06))}',
      '.gb-parity-status{min-height:18px;margin-bottom:8px;font-size:var(--id-text-sm,13px);color:var(--id-muted,#64748b)}.gb-parity-section{margin:12px 0;padding:14px;border:1px solid var(--id-line,#d7dde5);border-radius:10px;background:var(--id-surface,#fbfcfe)}.gb-parity-kicker{display:block;font-size:var(--id-text-xs,11px);letter-spacing:.08em;text-transform:uppercase;font-weight:600;color:var(--id-muted,#64748b)}.gb-parity-hint{font-size:var(--id-text-sm,13px);color:var(--id-muted,#64748b)}.gb-parity-section-title{margin:16px 0 2px;font-size:var(--id-text-lg,16px);font-weight:600}.gb-parity-warn{padding:10px 12px;border-radius:8px;background:var(--id-warn-soft,#fff7e6);color:var(--id-warn-text,#8a5a00)}',
      '.gb-parity-dash-main{display:flex;align-items:end;justify-content:space-between;gap:12px;flex-wrap:wrap;padding-bottom:10px;margin-bottom:8px;border-bottom:1px solid var(--id-line-soft,#e8edf3)}.gb-parity-engine{margin-top:2px;font-size:var(--id-text-2xl,20px);font-weight:600;overflow-wrap:anywhere}.gb-parity-where{display:inline-flex;align-items:center;gap:6px;padding:4px 10px;border-radius:999px;background:var(--id-accent-soft,#eef4ff);font-size:var(--id-text-sm,13px)}.gb-parity-row{display:flex;align-items:center;gap:12px;flex-wrap:wrap;padding:6px 0;font-size:var(--id-text-sm,13px)}.gb-parity-label{min-width:110px;font-weight:500;color:var(--id-muted,#64748b)}.gb-parity-text{flex:1 1 260px}',
      '.gb-parity-gauge{display:inline-flex;gap:3px;flex:none}.gb-parity-gauge i{width:14px;height:8px;border-radius:2px;background:var(--id-line,#d7dde5)}.gb-parity-gauge i.gb-parity-on{background:var(--id-accent,#2563eb)}.gb-parity-level-1 i.gb-parity-on,.gb-parity-level-2 i.gb-parity-on{background:var(--id-warn,#d97706)}.gb-parity-level-4 i.gb-parity-on,.gb-parity-level-5 i.gb-parity-on{background:var(--id-success,#16a34a)}.gb-parity-mark{flex:none;display:inline-flex;align-items:center;justify-content:center;width:16px;height:16px;border-radius:50%;font-size:11px;font-weight:700}.gb-parity-mark-yes{background:var(--id-success-soft,#dcfce7);color:var(--id-success-text,#166534)}.gb-parity-mark-yes::before{content:"\\2713"}',
      '.gb-parity-mark-part{background:var(--id-warn-soft,#fef3c7);color:var(--id-warn-text,#92400e)}.gb-parity-mark-part::before{content:"~"}.gb-parity-mark-no{background:var(--id-fill,#eef1f5);color:var(--id-muted,#64748b)}.gb-parity-mark-no::before{content:"\\2013"}.gb-parity-chips,.gb-parity-badges{display:flex;flex-wrap:wrap;gap:6px}.gb-parity-chip{display:inline-flex;align-items:center;gap:5px;padding:3px 9px;border:1px solid var(--id-line,#d7dde5);border-radius:999px;font-size:var(--id-text-xs,11px);color:var(--id-muted,#64748b)}.gb-parity-chip.gb-parity-on{color:var(--id-ink,#1c2530);background:var(--id-surface-solid,#fff)}',
      '.gb-parity-badge{padding:1px 7px;border:1px solid var(--id-line,#d7dde5);border-radius:999px;font-size:var(--id-text-xs,11px)}.gb-parity-badge-inuse{background:var(--id-accent-soft,#eef4ff)}.gb-parity-badge-sugg{display:none;background:var(--id-success-soft,#dcfce7);color:var(--id-success-text,#166534)}.gb-parity-badge-soon{color:var(--id-muted,#64748b)}.gb-parity-compare{overflow:auto;border:1px solid var(--id-line,#d7dde5);border-radius:10px;background:var(--id-surface-solid,#fff)}.gb-parity-grid{display:grid;grid-template-columns:minmax(110px,.9fr) repeat(4,minmax(130px,1fr));min-width:720px}.gb-parity-corner{border-bottom:1px solid var(--id-line-soft,#e8edf3)}',
      '.gb-parity-head{display:flex;flex-direction:column;align-items:flex-start;gap:4px;min-width:0;margin:10px 6px;padding:10px;border:1px solid var(--id-line,#d7dde5);border-radius:10px;background:var(--id-surface-solid,#fff);color:inherit;text-align:left;cursor:pointer;font:inherit}.gb-parity-head:hover:not(:disabled){border-color:var(--id-accent,#2563eb)}.gb-parity-head:disabled{cursor:not-allowed;opacity:.55}.gb-parity-head svg{color:var(--id-accent,#2563eb)}.gb-parity-head-label{font-weight:600}.gb-parity-head-sub{font-size:var(--id-text-xs,11px);color:var(--id-muted,#64748b)}.gb-parity-head:hover .gb-parity-head-sub{color:inherit}',
      '.gb-parity-grid[data-view="local"] [data-col="local"],.gb-parity-grid[data-view="own_key"] [data-col="own_key"],.gb-parity-grid[data-view="free"] [data-col="free"],.gb-parity-grid[data-view="acp"] [data-col="acp"]{background:var(--id-accent-soft,#eef4ff)}.gb-parity-grid[data-rec="local"] [data-mode="local"] .gb-parity-badge-sugg,.gb-parity-grid[data-rec="own_key"] [data-mode="own_key"] .gb-parity-badge-sugg,.gb-parity-grid[data-rec="free"] [data-mode="free"] .gb-parity-badge-sugg,.gb-parity-grid[data-rec="acp"] [data-mode="acp"] .gb-parity-badge-sugg{display:inline-block}',
      '.gb-parity-rowlabel{padding:10px;font-size:var(--id-text-sm,13px);font-weight:500;border-bottom:1px solid var(--id-line-soft,#e8edf3)}.gb-parity-cell{display:flex;gap:8px;align-items:flex-start;padding:10px;border-left:1px solid var(--id-line-soft,#e8edf3);border-bottom:1px solid var(--id-line-soft,#e8edf3);font-size:var(--id-text-sm,13px)}.gb-parity-note{display:block;margin-top:3px;font-size:var(--id-text-xs,11px);color:var(--id-muted,#64748b)}.gb-parity-guide{border-style:dashed}.gb-parity-guide-head{margin-bottom:8px}',
      '.gb-parity-step{display:flex;align-items:center;gap:8px;margin-bottom:8px;font-size:var(--id-text-xs,11px);font-weight:600;letter-spacing:.04em;text-transform:uppercase;color:var(--id-muted,#64748b)}.gb-parity-dots{display:inline-flex;gap:4px}.gb-parity-dots i{width:18px;height:4px;border-radius:2px;background:var(--id-line,#d7dde5)}.gb-parity-dots i.gb-parity-on{background:var(--id-accent,#2563eb)}.gb-parity-back{margin-left:auto;border:0;background:transparent;color:var(--id-accent,#2563eb);cursor:pointer;font:inherit;text-transform:none;letter-spacing:0}.gb-parity-q{margin:0 0 8px;font-size:var(--id-text-base,14px);font-weight:600}.gb-parity-help{margin:-4px 0 12px}',
      '.gb-parity-choices{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:10px}.gb-parity-choice{display:flex;align-items:flex-start;gap:12px;padding:12px;border:1px solid var(--id-line,#d7dde5);border-radius:10px;background:var(--id-surface-solid,#fff);color:inherit;text-align:left;cursor:pointer;font:inherit}.gb-parity-choice:hover{border-color:var(--id-accent,#2563eb)}.gb-parity-choice-icon{flex:none;display:inline-flex;align-items:center;justify-content:center;width:40px;height:40px;border-radius:10px;background:var(--id-accent-soft,#eef4ff);color:var(--id-accent,#2563eb)}.gb-parity-choice-text{display:flex;flex-direction:column;gap:2px}.gb-parity-choice-text small{color:var(--id-muted,#64748b)}',
      '.gb-parity-result-name{display:flex;align-items:center;gap:8px;margin:4px 0 8px;font-size:var(--id-text-xl,18px);font-weight:600}.gb-parity-result-name svg{color:var(--id-accent,#2563eb)}.gb-parity-rec{margin:0 0 12px;padding-left:12px;border-left:3px solid var(--id-accent,#2563eb)}.gb-parity-gains-title{margin:0 0 6px;font-size:var(--id-text-sm,13px);font-weight:600}.gb-parity-gains{list-style:none;margin:0 0 12px;padding:0;display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:6px 16px}.gb-parity-gains li{display:flex;gap:8px;align-items:flex-start;font-size:var(--id-text-sm,13px)}.gb-parity-actions{display:flex;gap:8px;flex-wrap:wrap}',
      '.gb-parity-btn{padding:6px 12px;border:1px solid var(--id-line,#d7dde5);border-radius:8px;background:var(--id-surface-solid,#fff);color:inherit;cursor:pointer;font:inherit;font-size:var(--id-text-sm,13px)}.gb-parity-btn-accent{border-color:var(--id-accent,#2563eb);background:var(--id-accent,#2563eb);color:#fff}.gb-parity-mhead{display:flex;align-items:center;justify-content:space-between;margin-bottom:8px}.gb-parity-mrow{display:grid;grid-template-columns:150px 1fr;gap:12px;padding:6px 0;border-bottom:1px solid var(--id-line-soft,#e8edf3);font-size:var(--id-text-sm,13px)}.gb-parity-mkey{color:var(--id-muted,#64748b)}.gb-parity-inline{display:inline-flex;align-items:center;gap:6px}',
      '.gb-parity-verdict{display:flex;flex-direction:column;gap:2px;margin-top:12px;padding:10px;border-left:4px solid var(--id-line,#d7dde5);border-radius:8px;background:var(--id-fill,#eef1f5);font-size:var(--id-text-sm,13px)}.gb-parity-verdict.gb-parity-good{border-left-color:var(--id-success,#16a34a);background:var(--id-success-soft,#dcfce7)}.gb-parity-verdict.gb-parity-mid{border-left-color:var(--id-warn,#d97706);background:var(--id-warn-soft,#fef3c7)}.gb-parity-verdict.gb-parity-bad{border-left-color:var(--id-danger,#dc2626);background:var(--id-danger-soft,#fee2e2)}.gb-parity-rule{margin:8px 0 0;font-size:var(--id-text-xs,11px);color:var(--id-muted,#64748b)}@media (max-width:640px){.gb-parity-mrow{grid-template-columns:1fr;gap:2px}}'
    ].join("");
    var style = document.createElement("style");
    style.id = STYLE_ID;
    style.textContent = css;
    (document.head || document.documentElement).appendChild(style);
  }

  /* ---- Dialog lifecycle ---- */
  var dialog = null, escapeHandler = null, lastTrigger = null;
  function close() {
    if (!dialog) return;
    if (escapeHandler) { document.removeEventListener("keydown", escapeHandler); escapeHandler = null; }
    var node = dialog;
    dialog = null;
    if (node.parentNode) { node.parentNode.removeChild(node); }
    if (lastTrigger && typeof lastTrigger.focus === "function") { lastTrigger.focus(); }
    lastTrigger = null;
  }
  function render(root, data) {
    var body = root && root.querySelector(".gb-parity-body");
    if (!body) return;
    var failed = T("chat-parity-load-failed", "The model configuration could not be loaded right now.");
    var html = data.error ? '<div class="gb-parity-warn">' + esc(failed) +
      ' <button type="button" class="gb-parity-btn" data-gb-parity-retry">' + esc(T("chat-parity-retry", "Try again")) + "</button></div>" : "";
    body.innerHTML = html + dashboard(data) + guide() + compare(data) + machineCard(data);
    var retry = body.querySelector("[data-gb-parity-retry]");
    if (retry) { retry.addEventListener("click", function () { reload(root); }); }
    bindGuide(root, data);
    bindCompare(root, data);
    announce(root, data.error ? failed : T("chat-parity-ready", "Setup loaded."));
  }
  function show() {
    if (dialog) { var existing = dialog.querySelector(".gb-parity-close"); if (existing) { existing.focus(); } return; }
    ensureStyles();
    lastTrigger = document.activeElement;
    var overlay = document.createElement("div");
    overlay.className = "gb-parity-workshop";
    overlay.setAttribute("role", "dialog");
    overlay.setAttribute("aria-modal", "true");
    overlay.setAttribute("data-i18n-aria-label", "chat-parity-workshop-title");
    overlay.setAttribute("aria-label", T("chat-parity-workshop-title", "Assistant workshop"));
    overlay.innerHTML = '<div class="gb-parity-backdrop" data-gb-parity-close></div><div class="gb-parity-panel"><div class="gb-parity-head"><div>' +
      '<h2 class="gb-parity-title" data-i18n="chat-parity-workshop-title">' + esc(T("chat-parity-workshop-title", "Assistant workshop")) + "</h2>" +
      '<p class="gb-parity-lead" data-i18n="chat-parity-workshop-lead">' + esc(T("chat-parity-workshop-lead", "Tune where and how your assistant thinks.")) + "</p></div>" +
      '<button type="button" class="gb-parity-close" data-gb-parity-close data-i18n-aria-label="chat-parity-close" data-i18n-title="chat-parity-close" aria-label="' +
      esc(T("chat-parity-close", "Close")) + '" title="' + esc(T("chat-parity-close", "Close")) + '">&times;</button></div>' +
      '<div class="gb-parity-status" role="status" aria-live="polite"></div><div class="gb-parity-body"></div></div>';
    document.body.appendChild(overlay);
    dialog = overlay;
    Array.prototype.forEach.call(overlay.querySelectorAll("[data-gb-parity-close]"), function (node) { node.addEventListener("click", close); });
    escapeHandler = function (event) { if (event.key === "Escape") { event.preventDefault(); close(); } };
    document.addEventListener("keydown", escapeHandler);
    var closeButton = overlay.querySelector(".gb-parity-close");
    if (closeButton) { closeButton.focus(); }
    announce(overlay, T("chat-parity-loading", "Loading the model setup…"));
    load().then(function (data) { if (dialog === overlay) { render(overlay, data); } });
  }
  function mount() {
    var host = document.getElementById("chatHeaderActions");
    if (!host || host.querySelector(".gb-parity-workshop-btn")) return;
    var label = T("chat-parity-workshop", "Assistant workshop"), button = document.createElement("button");
    button.type = "button";
    button.className = "gb-parity-workshop-btn";
    button.setAttribute("data-i18n", "chat-parity-workshop");
    button.title = label;
    button.setAttribute("aria-label", label);
    button.innerHTML = ICONS.wrench;
    button.addEventListener("click", show);
    host.appendChild(button);
  }
  P.workshop = { mount: function () { mount(); }, open: function () { show(); } };
  if (typeof window.addEventListener === "function") { window.addEventListener("gb-chat-ready", mount); }
})();
