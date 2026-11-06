"use strict";
/* slides events — sidebar, collab, auth, canvas styles, HTMX listeners */

// Toolbar forms send the deck id as `id`/`presentation_id` = "current". Once a
// deck is open, point those parameters at the real id so save/rename/export
// and element actions reach the open deck.
document.addEventListener("htmx:configRequest", function (event) {
  var real = window.__SLIDES_PRESENTATION_ID;
  if (!real || real === "current") return;
  var parameters = (event.detail && event.detail.parameters) || {};
  ["id", "presentation_id"].forEach(function (key) {
    if (parameters[key] === "current") parameters[key] = real;
  });
  // Toolbar forms address the visible slide as "current"; send its 0-based
  // index, since the handlers deserialize slide_index as a number.
  var canvas = document.getElementById("slide-canvas");
  var visibleSlide = canvas && canvas.dataset.slideId ? canvas.dataset.slideId : "0";
  ["slide", "slide_index"].forEach(function (key) {
    if (parameters[key] === "current") {
      parameters[key] = parseInt(visibleSlide, 10) || 0;
    }
  });
});

// Re-render the current slide after a toolbar insert. The element API answers
// JSON, so those forms save with hx-swap="none" and ask for a fresh view
// instead of replacing the canvas with the response body.
// A successful insert saves and refreshes the slide; close its modal too.
// Export is a JSON POST returning a file body; fetch it with the suite token
// and save a blob, because htmx cannot stream a download.
window.GBSlidesExport = function (event) {
  event.preventDefault();
  var form = event.target;
  var formatSelect = form.querySelector('[name="format"]');
  var format = formatSelect ? formatSelect.value : "pptx";
  var deck = (form.dataset && form.dataset.presentationId) ||
    window.__SLIDES_PRESENTATION_ID || "current";
  var token = (typeof getGBAccessToken === "function" && getGBAccessToken()) ||
    window.localStorage.getItem("gb-access-token") ||
    window.sessionStorage.getItem("gb-access-token") || "";
  var headers = { "Content-Type": "application/json" };
  if (token) headers.Authorization = "Bearer " + token;
  fetch("/api/slides/export", {
    method: "POST",
    headers: headers,
    body: JSON.stringify({ presentation_id: deck, format: format }),
  })
    .then(function (response) {
      if (!response.ok) throw new Error("HTTP " + response.status);
      return response.blob();
    })
    .then(function (blob) {
      var link = document.createElement("a");
      link.href = URL.createObjectURL(blob);
      link.download = "presentation." + format;
      document.body.appendChild(link);
      link.click();
      link.remove();
      setTimeout(function () { URL.revokeObjectURL(link.href); }, 4000);
      var modal = form.closest ? form.closest(".sl-modal") : null;
      if (modal) modal.remove();
      else {
        var container = form.closest ? form.closest("#modal-container") : null;
        if (container) container.innerHTML = "";
      }
    })
    .catch(function (error) {
      if (window.GBToasts && GBToasts.show) GBToasts.show("Export", error.message, "error");
    });
  return false;
};

window.afterSlideInsert = function (element) {
  var modal = element && element.closest ? element.closest(".sl-modal") : null;
  if (modal) {
    modal.remove();
  } else {
    // Toolbar forms render as a bare form inside #modal-container.
    var container = element && element.closest ? element.closest("#modal-container") : null;
    if (container) container.innerHTML = "";
    else if (element && element.remove) element.remove();
  }
  if (window.refreshSlideView) window.refreshSlideView();
};

window.refreshSlideView = function () {
  var canvas = document.getElementById("slide-canvas");
  var index = (canvas && canvas.dataset.slideId) || "0";
  var deck = window.__SLIDES_PRESENTATION_ID || "current";
  if (!window.htmx) return;
  htmx.ajax(
    "GET",
    "/suite/slides/fragments/presentation-view?id=" + encodeURIComponent(deck) +
      "&slide=" + encodeURIComponent(index),
    { target: "#slides-content", swap: "innerHTML" }
  );
};

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

// #1715 — the delete API existed with no caller; the presentation card's
// Delete button resolves through here. The list fragment refreshes in place
// (scoped to this app window — #sidebar-files also exists in the docs app).
function deletePresentation(id) {
  if (!id || !window.confirm("Delete this presentation?")) return;
  fetch("/api/slides/delete", {
    method: "POST",
    headers: Object.assign(
      { "Content-Type": "application/json" },
      (window.GBSidebarConvos && window.GBSidebarConvos.authHeaders)
        ? window.GBSidebarConvos.authHeaders()
        : {}
    ),
    body: JSON.stringify({ id: id }),
  })
    .then(function (r) {
      if (!r.ok) throw new Error("HTTP " + r.status);
      return r.json();
    })
    .then(function () {
      var scope = document.querySelector('[id^="window-body-slides"]') || document;
      var list = scope.querySelector("#sidebar-files");
      if (window.htmx && list) {
        htmx.ajax("GET", "/suite/slides/fragments/presentation-list", {
          target: list,
          swap: "innerHTML",
        });
      }
      if (window.__SLIDES_PRESENTATION_ID === id) {
        window.__SLIDES_PRESENTATION_ID = null;
      }
      var status = scope.querySelector("#saveStatus");
      if (status) status.textContent = "Presentation deleted";
    })
    .catch(function (err) {
      var scope = document.querySelector('[id^="window-body-slides"]') || document;
      var status = scope.querySelector("#saveStatus");
      if (status) status.textContent = "Delete failed: " + err.message;
    });
}
window.deletePresentation = deletePresentation;

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
      if (!msg) return;
      var t = msg.msg_type;
      if (t === "cursor") { if (window.SlidesPresence) window.SlidesPresence.cursor(msg); }
      else if (t === "laser") { if (window.SlidesPresenter) window.SlidesPresenter.laser(msg); }
      else if (t === "question") { if (window.SlidesPresenter) window.SlidesPresenter.question(msg); }
      else if (t === "question_upvote" || t === "question_answered") {
        if (window.SlidesPresenter && window.SlidesPresenter.refreshQa) window.SlidesPresenter.refreshQa();
      }
      else if (t === "remote_nav") { if (window.SlidesPresenter) window.SlidesPresenter.remoteNav(msg); }
      else if (t === "presenter_take" || t === "presenter_assign" || t === "presenter_release") {
        if (window.SlidesPresenter) window.SlidesPresenter.presenter(msg);
      }
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
    // Any rendered deck (import or metadata card) tells the toolbar which id
    // to address; without this the next save/rename created a second deck.
    var view = document.getElementById("slides-view");
    if (view && view.dataset.presentationId) {
      window.__SLIDES_PRESENTATION_ID = view.dataset.presentationId;
      var titleInput = document.getElementById("presTitle");
      if (titleInput) {
        titleInput.setAttribute(
          "hx-vals",
          JSON.stringify({ id: view.dataset.presentationId, action: "rename" })
        );
        if (window.htmx) htmx.process(titleInput);
      }
    }
    SlideCanvas.attach(document.getElementById("slides-content"));
    if (typeof checkSlidesDraftOnLoad === "function") checkSlidesDraftOnLoad();
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
        // Presenter drives every follower's slide (remote control).
        if (window.SlidesPresenter && window.SlidesPresenter.isPresenter && window.SlidesPresenter.isPresenter()) {
          var idx = $$(".sl-thumb").indexOf(thumb);
          if (idx >= 0 && window.GBCollab) window.GBCollab.send("remote_nav", { slide_index: idx });
        }
      });
    });
  }
});
