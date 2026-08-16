"use strict";
/* slides events — sidebar, collab, auth, canvas styles, HTMX listeners */

document.addEventListener("click", function (e) {
  var tab = e.target.closest("[data-sidebar-tab]");
  if (tab) {
    var which = tab.dataset.sidebarTab;
    $$(".sidebar-tab").forEach(function (b) {
      b.classList.toggle("active", b === tab);
      b.style.background = b === tab ? "#1e293b" : "#0f172a";
      b.style.color = b === tab ? "#f8fafc" : "#94a3b8";
    });
    $$(".sidebar-content").forEach(function (c) {
      c.style.display = c.dataset.sidebarContent === which ? "flex" : "none";
    });
    try { sessionStorage.setItem(SIDEBAR_TAB_KEY, which); } catch (_) {}
  }
});

function initSidebar() {
  var saved = null;
  try { saved = sessionStorage.getItem(SIDEBAR_TAB_KEY); } catch (_) {}
  if (saved) {
    var btn = document.querySelector('[data-sidebar-tab="' + saved + '"]');
    if (btn) btn.click();
  }
}

function injectCanvasStyles() {
  if (document.getElementById("slides-canvas-styles")) return;
  var style = document.createElement("style");
  style.id = "slides-canvas-styles";
  style.textContent =
    ".sl-canvas-scroll{flex:1;display:flex;align-items:center;justify-content:center;overflow:auto;background:#020617;padding:32px;}"+
    ".sl-canvas{position:relative;width:960px;height:540px;background:#f8fafc;border-radius:6px;box-shadow:0 8px 32px rgba(0,0,0,0.4);transform-origin:center center;}"+
    ".sl-element{position:absolute;cursor:move;user-select:none;box-sizing:border-box;outline:none;}"+
    ".sl-element[data-type='title'] h1,.sl-element[data-type='text'] p{margin:0;padding:4px 8px;}"+
    ".sl-thumb{background:#1e293b;border:1px solid #334155;border-radius:4px;padding:8px;cursor:pointer;transition:border-color 0.15s;}"+
    ".sl-thumb:hover{border-color:#3b82f6;}"+
    ".sl-thumb.active{border-color:#3b82f6;background:#1e3a8a;}"+
    ".sl-thumb-preview{position:relative;width:100%;aspect-ratio:16/9;background:#f8fafc;border-radius:2px;overflow:hidden;}"+
    ".sl-thumb-num{font-size:11px;color:#94a3b8;margin-top:4px;text-align:center;}";
  document.head.appendChild(style);
}

function initAuth() {
  if (window.GBAuthGuard) GBAuthGuard.injectLoginButton(document.getElementById("gb-auth-button"));
}

// Single source of truth for the active presentation id. Persisted saves can
// capture a real UUID into window.__SLIDES_PRESENTATION_ID; until then the
// app operates on the "current" deck (the backend treats "current" as the
// literal id of the live presentation).
function getSlidesPresentationId() {
  return window.__SLIDES_PRESENTATION_ID || "current";
}
window.getSlidesPresentationId = getSlidesPresentationId;

function initCollab() {
  if (!window.GBCollab) return;
  var connStatus = document.getElementById("gb-conn-status");
  window.GBCollab.connect({
    app: "slides",
    docId: getSlidesPresentationId(),
    collaboratorsEl: document.getElementById("collaborators"),
    onConnect: function () {
      if (connStatus) { connStatus.className = "gb-connection-status online"; connStatus.style.display = "inline-flex"; connStatus.querySelector(".label").textContent = "online"; }
    },
    onDisconnect: function () {
      if (connStatus) { connStatus.className = "gb-connection-status offline"; connStatus.style.display = "inline-flex"; connStatus.querySelector(".label").textContent = "offline"; }
      if (window.SlidesPresence) window.SlidesPresence.clearAll();
    },
    onMessage: function (msg) {
      if (!msg || !window.SlidesPresence) return;
      if (msg.msg_type === "cursor") window.SlidesPresence.cursor(msg);
    },
    onSelection: function (msg) {
      if (window.SlidesPresence) window.SlidesPresence.selection(msg);
    },
    onTyping: function (msg) {
      if (!msg || !window.SlidesPresence) return;
      if (msg.msg_type === "typing_start") window.SlidesPresence.typing(msg);
      else if (msg.msg_type === "typing_stop") window.SlidesPresence.clearTyping(msg.user_id);
    },
    onPresence: function (users) {
      if (window.SlidesPresence) window.SlidesPresence.sync(users);
    },
    onEdit: function (msg) {
      if (!msg || !msg.content) return;
      try {
        var update = JSON.parse(msg.content);
        if (update.element_id && update.x !== undefined && update.y !== undefined) {
          var el = document.querySelector('[data-id="' + update.element_id + '"]');
          if (el) {
            el.style.left = update.x + "px";
            el.style.top = update.y + "px";
            if (update.width) el.style.width = update.width + "px";
            if (update.height) el.style.height = update.height + "px";
            if (update.rotation !== undefined) {
              el.dataset.rotation = update.rotation;
              el.style.transform = "rotate(" + update.rotation + "deg)";
            }
          }
        }
      } catch (_) {}
    }
  });
}

document.addEventListener("htmx:afterSwap", function (e) {
  if (e.target.id === "slides-content" || (e.target.closest && e.target.closest("#slides-content"))) {
    SlideCanvas.attach(document.getElementById("slides-content"));
  }
  if (e.target.id === "sidebar-thumbs" || (e.target.closest && e.target.closest("#sidebar-thumbs"))) {
    $$(".sl-thumb", e.target).forEach(function (thumb) {
      thumb.addEventListener("click", function () {
        var slideId = thumb.dataset.slide;
        var view = document.getElementById("slides-content");
        if (view && slideId) {
          htmx.ajax("GET", "/suite/slides/fragments/presentation-view", {
            target: "#slides-content",
            swap: "innerHTML",
            values: { id: "current", slide: slideId }
          });
        }
      });
    });
  }
});
