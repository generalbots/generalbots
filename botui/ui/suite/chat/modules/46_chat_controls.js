"use strict";

// #2238 — unified chat controls: Web, Media type, Agent (VM), App automation.
// Compact popover in the composer; state is sent with every message and
// persisted per conversation (session-scoped key).
(function () {
  var DEFAULTS = {
    web: true,
    mediaType: "auto",
    agent: true,
    appAutomation: true,
  };

  var MEDIA_TYPES = [
    ["auto", "Auto"],
    ["app-inline", "Inline: app-inline (HTML/HTMX)"],
    ["markdown", "Inline: markdown/rich text"],
    ["svg-inline", "Inline: SVG"],
    ["docx", "Document: .docx"],
    ["xlsx", "Document: .xlsx"],
    ["pptx", "Document: .pptx"],
    ["pdf", "Document: .pdf"],
    ["csv", "Document: .csv"],
    ["md", "Document: .md/.txt"],
    ["html", "Document: .html"],
    ["png", "Image: .png"],
    ["jpg", "Image: .jpg/.jpeg"],
    ["svg", "Image: .svg (file)"],
    ["webp", "Image: .webp"],
    ["mp3", "Audio/Music: .mp3"],
    ["wav", "Audio/Music: .wav"],
    ["mp4", "Video: .mp4"],
    ["ocr-json", "Analysis: OCR/JSON"],
    ["qr-barcode", "Analysis: QR/barcode result"],
    ["image-description", "Analysis: image/video description"],
  ];

  function storageKey() {
    try {
      var bot = (window.ChatState && ChatState.currentBotId) || (window.__BOT_NAME__) || "default";
      var sid = (window.ChatState && ChatState.currentSessionId) || "global";
      return "gb-chat-controls:" + bot + ":" + sid;
    } catch (e) {
      return "gb-chat-controls:default:global";
    }
  }

  function read() {
    try {
      var raw = localStorage.getItem(storageKey());
      if (!raw) {
        // default: follow existing web search toggle state if available
        var webDefault = DEFAULTS.web;
        try {
          if (window.ChatWebSearch && typeof window.ChatWebSearch.isEnabled === "function") {
            webDefault = !!window.ChatWebSearch.isEnabled();
          }
        } catch (e2) {}
        return Object.assign({}, DEFAULTS, { web: webDefault });
      }
      var parsed = JSON.parse(raw);
      var agent = typeof parsed.agent === "boolean" ? parsed.agent : DEFAULTS.agent;
      var wanted = typeof parsed.appAutomationWanted === "boolean"
        ? parsed.appAutomationWanted
        : (typeof parsed.appAutomation === "boolean" ? parsed.appAutomation : DEFAULTS.appAutomation);
      return {
        web: typeof parsed.web === "boolean" ? parsed.web : DEFAULTS.web,
        mediaType: typeof parsed.mediaType === "string" ? parsed.mediaType : DEFAULTS.mediaType,
        agent: agent,
        // Live generation (UI hints into apps) needs the VM/agent loop.
        appAutomationWanted: wanted,
        appAutomation: agent && wanted,
      };
    } catch (e) {
      return Object.assign({}, DEFAULTS);
    }
  }

  function write(state) {
    try { localStorage.setItem(storageKey(), JSON.stringify(state)); } catch (e) {}
  }

  var state = read();

  function syncWebToggle() {
    // keep legacy web search toggle in sync if present
    try {
      if (window.ChatWebSearch && typeof window.ChatWebSearch.setEnabled === "function") {
        if (window.ChatWebSearch.isEnabled() !== state.web) {
          window.ChatWebSearch.setEnabled(state.web);
        }
      }
    } catch (e) {}
  }

  function getState() {
    // reflect latest web toggle if user used the standalone button
    try {
      if (window.ChatWebSearch && typeof window.ChatWebSearch.isEnabled === "function") {
        state.web = !!window.ChatWebSearch.isEnabled();
      }
    } catch (e) {}
    var agent = !!state.agent;
    var wanted = state.appAutomationWanted !== undefined
      ? !!state.appAutomationWanted
      : !!state.appAutomation;
    return {
      web: !!state.web,
      mediaType: state.mediaType || "auto",
      agent: agent,
      // Hard rule: no live generation without the VM/agent.
      appAutomation: agent && wanted,
    };
  }

  function setState(patch) {
    var next = Object.assign({}, patch || {});
    if (typeof next.appAutomation === "boolean") {
      next.appAutomationWanted = next.appAutomation;
    }
    state = Object.assign({}, state, next);
    if (typeof state.appAutomationWanted !== "boolean") {
      state.appAutomationWanted = !!state.appAutomation;
    }
    // Unchecking Agent (VM) disables live generation in the same action; the
    // preference is remembered so re-enabling the VM restores it.
    state.appAutomation = !!state.agent && !!state.appAutomationWanted;
    write(state);
    syncWebToggle();
    render();
  }

  function render() {
    var btn = document.getElementById("chatControlsBtn");
    if (btn) {
      var s = getState();
      var onCount = (s.web?1:0) + (s.agent?1:0) + (s.appAutomation?1:0);
      btn.title = "Chat controls: Web " + (s.web?"on":"off") + ", Agent " + (s.agent?"on":"off") + ", Live generation " + (s.appAutomation?"on":"off") + ", Media " + s.mediaType;
      btn.setAttribute("aria-label", btn.title);
      var badge = btn.querySelector(".chat-controls-badge");
      if (badge) badge.textContent = String(onCount);
    }
    var panel = document.getElementById("chatControlsPanel");
    if (!panel) return;
    var s = getState();
    var web = panel.querySelector("#cc-web");
    var agent = panel.querySelector("#cc-agent");
    var app = panel.querySelector("#cc-app");
    var media = panel.querySelector("#cc-media");
    if (web) web.checked = s.web;
    if (agent) agent.checked = s.agent;
    if (app) {
      app.checked = s.appAutomation;
      app.disabled = !s.agent;
      var row = app.closest ? app.closest(".chat-controls-row") : null;
      var hint = row ? row.querySelector("small") : null;
      if (hint) {
        hint.textContent = s.agent
          ? "Generate in apps (docs/canvas) via UI hints"
          : "Requires Agent (VM) — disabled while the VM is off";
      }
      if (row) row.style.opacity = s.agent ? "" : "0.55";
    }
    if (media) media.value = s.mediaType;
  }

  function closePanel() {
    var panel = document.getElementById("chatControlsPanel");
    if (panel) panel.style.display = "none";
  }

  function togglePanel() {
    var panel = document.getElementById("chatControlsPanel");
    if (!panel) return;
    panel.style.display = (panel.style.display === "block") ? "none" : "block";
    render();
  }

  function bind() {
    var form = document.getElementById("chatForm");
    var btn = document.getElementById("chatControlsBtn");
    if (!form || !btn || btn.dataset.ccBound) return;
    btn.dataset.ccBound = "1";

    // create panel
    var panel = document.createElement("div");
    panel.id = "chatControlsPanel";
    panel.className = "chat-controls-panel";
    panel.style.display = "none";
    panel.innerHTML =
      '<div class="chat-controls-title">Chat controls</div>' +
      '<label class="chat-controls-row"><input type="checkbox" id="cc-web"> <span>Web</span><small>Use web/search for current info</small></label>' +
      '<label class="chat-controls-row"><input type="checkbox" id="cc-agent"> <span>Agent (VM)</span><small>Allow workspace/VM execution</small></label>' +
      '<label class="chat-controls-row"><input type="checkbox" id="cc-app"> <span>Live generation</span><small>Generate in apps (docs/canvas) via UI hints</small></label>' +
      '<label class="chat-controls-row chat-controls-col"><span>Media type</span><small>Desired output/artifact</small>' +
      '<select id="cc-media"></select></label>';
    // insert near form
    var host = form.parentElement || form;
    host.style.position = host.style.position || "relative";
    host.appendChild(panel);

    var sel = panel.querySelector("#cc-media");
    if (sel) {
      MEDIA_TYPES.forEach(function (mt) {
        var opt = document.createElement("option");
        opt.value = mt[0];
        opt.textContent = mt[1];
        sel.appendChild(opt);
      });
    }

    btn.addEventListener("click", function (e) {
      e.preventDefault();
      e.stopPropagation();
      togglePanel();
    });

    panel.addEventListener("click", function (e) { e.stopPropagation(); });
    document.addEventListener("click", closePanel);

    var web = panel.querySelector("#cc-web");
    var agent = panel.querySelector("#cc-agent");
    var app = panel.querySelector("#cc-app");
    var media = panel.querySelector("#cc-media");
    if (web) web.addEventListener("change", function () { setState({ web: !!web.checked }); });
    if (agent) agent.addEventListener("change", function () { setState({ agent: !!agent.checked }); });
    if (app) app.addEventListener("change", function () {
      var current = getState();
      if (!current.agent) {
        // Defensive: a stale DOM could still dispatch a change event.
        app.checked = false;
        return;
      }
      setState({ appAutomation: !!app.checked });
    });
    if (media) media.addEventListener("change", function () { setState({ mediaType: media.value || "auto" }); });

    // sync when web toggle button changes externally
    window.addEventListener("gb-web-search-changed", function () {
      if (window.ChatWebSearch) {
        state.web = !!window.ChatWebSearch.isEnabled();
        write(state);
        render();
      }
    });

    render();
  }

  window.GBChatControls = {
    getState: getState,
    setState: setState,
    refresh: function () { state = read(); render(); },
  };

  window.addEventListener("gb-chat-ready", bind);
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bind);
  } else {
    bind();
  }
})();
