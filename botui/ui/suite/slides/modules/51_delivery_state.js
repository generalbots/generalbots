"use strict";
/* Slides delivery mode (#2432) — local-only state, helpers and accessible
 * dialog shell. Everything this layer owns lives in deck-scoped localStorage
 * and is labelled "Local only — not saved to the server". No server writes,
 * no media capture, no audience sync is attempted here. */
(function (window) {
  var D = (window.GBSlidesDelivery = window.GBSlidesDelivery || {});
  var dialogSeq = 0;

  D.tr = function (key, fallback, params) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
  };
  D.escapeHtml = function (value) {
    return String(value === null || value === undefined ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  };
  D.uid = function (prefix) {
    return (prefix || "delivery") + "_" + Date.now().toString(36) + "_" +
      Math.random().toString(36).slice(2, 7);
  };

  D.deck = function () {
    return (typeof window.getSlidesPresentationId === "function" &&
      window.getSlidesPresentationId()) || window.__SLIDES_PRESENTATION_ID || "current";
  };
  D.storeKey = function () { return "slides-delivery:" + D.deck(); };

  function blank() {
    return {
      shows: [],
      sections: [],
      notes: {},
      captions: { on: false, style: { color: "#f8fafc", background: "#111827", size: 20 } },
      rehearsal: { targets: {}, actual: {}, appliedAt: 0 },
      handouts: { perPage: 4 },
      recording: { active: false, startedAt: 0, duration: 0 }
    };
  }
  D.blank = blank;

  D.load = function () {
    var fresh = blank();
    try {
      var raw = window.localStorage.getItem(D.storeKey());
      var stored = raw ? JSON.parse(raw) : null;
      if (!stored || typeof stored !== "object") return fresh;
      ["shows", "sections"].forEach(function (field) {
        if (Array.isArray(stored[field])) fresh[field] = stored[field];
      });
      if (stored.notes && typeof stored.notes === "object") fresh.notes = stored.notes;
      if (stored.captions && typeof stored.captions === "object") {
        fresh.captions.on = !!stored.captions.on;
        if (stored.captions.style && typeof stored.captions.style === "object") {
          fresh.captions.style = Object.assign(fresh.captions.style, stored.captions.style);
        }
      }
      if (stored.rehearsal && typeof stored.rehearsal === "object") {
        if (stored.rehearsal.targets && typeof stored.rehearsal.targets === "object") {
          fresh.rehearsal.targets = stored.rehearsal.targets;
        }
        if (stored.rehearsal.actual && typeof stored.rehearsal.actual === "object") {
          fresh.rehearsal.actual = stored.rehearsal.actual;
        }
        if (Object.prototype.hasOwnProperty.call(stored.rehearsal, "appliedAt")) {
          fresh.rehearsal.appliedAt = Number(stored.rehearsal.appliedAt) || 0;
        }
      }
      if (stored.handouts && typeof stored.handouts === "object") {
        fresh.handouts.perPage = Number(stored.handouts.perPage) || 4;
      }
      if (stored.recording && typeof stored.recording === "object") {
        fresh.recording.active = !!stored.recording.active;
        fresh.recording.startedAt = Number(stored.recording.startedAt) || 0;
        if (Object.prototype.hasOwnProperty.call(stored.recording, "duration")) {
          fresh.recording.duration = Number(stored.recording.duration) || 0;
        }
      }
      return fresh;
    } catch (error) {
      return fresh;
    }
  };
  D.save = function (data) {
    try {
      window.localStorage.setItem(D.storeKey(), JSON.stringify(data || blank()));
      return true;
    } catch (error) {
      return false;
    }
  };
  D.patch = function (mutator) {
    var data = D.load();
    var result = mutator(data);
    D.save(data);
    return result;
  };

  D.status = function (key, fallback, params) {
    var text = D.tr(key, fallback, params);
    var host = document.getElementById("slides-tools-status");
    if (host) host.textContent = text;
    return text;
  };

  D.slideCount = function () {
    var thumbs = document.querySelectorAll(".sl-thumb");
    return thumbs.length || 1;
  };
  D.currentSlide = function () {
    var canvas = document.getElementById("slide-canvas");
    var raw = canvas && (canvas.getAttribute("data-slide-id") || canvas.dataset.slideId);
    var index = parseInt(raw, 10);
    return isFinite(index) ? index : 0;
  };
  D.jump = function (index) {
    var count = D.slideCount();
    var target = Math.max(0, Math.min(count - 1, Number(index) || 0));
    var thumbs = document.querySelectorAll(".sl-thumb");
    if (thumbs[target] && typeof thumbs[target].click === "function") {
      thumbs[target].click();
      return true;
    }
    if (window.GBSlidesOutline && typeof window.GBSlidesOutline.jump === "function") {
      window.GBSlidesOutline.jump(target);
      return true;
    }
    if (typeof window.refreshSlideView === "function") {
      window.refreshSlideView(target);
      return true;
    }
    return false;
  };

  D.labelled = function (el, attr, key, fallback) {
    el.setAttribute(attr, key);
    var value = D.tr(key, fallback);
    if (attr === "data-i18n") el.textContent = value;
    else if (attr === "data-i18n-placeholder") el.setAttribute("placeholder", value);
    else if (attr === "data-i18n-title") el.setAttribute("title", value);
    else if (attr === "data-i18n-aria-label") el.setAttribute("aria-label", value);
    return el;
  };
  D.el = function (tag, cls, key, fallback) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    if (key) D.labelled(node, "data-i18n", key, fallback);
    return node;
  };
  D.button = function (key, fallback, onClick, variant) {
    var node = D.el("button", "sd-btn" + (variant ? " sd-btn-" + variant : ""), key, fallback);
    node.type = "button";
    if (typeof onClick === "function") node.addEventListener("click", onClick);
    return node;
  };
  D.field = function (key, fallback, value, placeholderKey, placeholderFallback) {
    var label = D.el("label", "sd-field");
    D.labelled(label, "data-i18n", key, fallback);
    var input = document.createElement("input");
    input.type = "text";
    input.value = value === undefined || value === null ? "" : String(value);
    if (placeholderKey) D.labelled(input, "data-i18n-placeholder", placeholderKey, placeholderFallback);
    label.appendChild(input);
    label.control = input;
    return label;
  };
  D.localNote = function () {
    return D.el("p", "sd-local-note", "slides-delivery-local-only", "Local only — not saved to the server");
  };

  D.formatTime = function (ms) {
    var total = Math.max(0, Math.floor((Number(ms) || 0) / 1000));
    var minutes = Math.floor(total / 60);
    var seconds = total % 60;
    return minutes + ":" + (seconds < 10 ? "0" : "") + seconds;
  };

  function focusables(root) {
    return Array.prototype.slice.call(root.querySelectorAll(
      "button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex=\"-1\"])"
    ));
  }

  /* Accessible modal: labelled dialog, focus trap, Esc close, focus restore,
   * and a polite status region. No native alert/confirm/prompt anywhere. */
  D.openDialog = function (options) {
    options = options || {};
    var previous = document.activeElement;
    var overlay = document.createElement("div");
    overlay.className = "sd-overlay";
    var dialog = document.createElement("div");
    dialog.className = "sd-dialog";
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = "sd-title-" + (++dialogSeq);
    dialog.setAttribute("aria-labelledby", titleId);

    var header = document.createElement("div");
    header.className = "sd-dialog-head";
    var title = D.el("h2", "sd-dialog-title", options.titleKey || "slides-delivery-tools-title",
      options.titleFallback || "Delivery tools");
    title.id = titleId;
    header.appendChild(title);
    var close = document.createElement("button");
    close.type = "button";
    close.className = "sd-dialog-close";
    close.textContent = "\u00d7";
    D.labelled(close, "data-i18n-aria-label", "slides-delivery-close", "Close");
    close.addEventListener("click", function () { api.close(); });
    header.appendChild(close);
    dialog.appendChild(header);

    var body = document.createElement("div");
    body.className = "sd-dialog-body";
    dialog.appendChild(body);
    var status = document.createElement("div");
    status.className = "sd-dialog-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    dialog.appendChild(status);
    var footer = document.createElement("div");
    footer.className = "sd-dialog-foot";
    dialog.appendChild(footer);

    var closed = false;
    var api = {
      root: dialog,
      body: body,
      footer: footer,
      setStatus: function (key, fallback, params) {
        status.textContent = D.tr(key, fallback, params);
        return api;
      },
      addButton: function (key, fallback, onClick, variant) {
        var button = D.button(key, fallback, onClick, variant);
        footer.appendChild(button);
        return button;
      },
      close: function () {
        if (closed) return;
        closed = true;
        document.removeEventListener("keydown", onKeyDown, true);
        if (overlay.parentNode) overlay.parentNode.removeChild(overlay);
        if (previous && typeof previous.focus === "function") previous.focus();
      }
    };

    function onKeyDown(event) {
      if (closed) return;
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        api.close();
      } else if (event.key === "Tab") {
        var items = focusables(dialog);
        if (!items.length) { event.preventDefault(); return; }
        var first = items[0];
        var last = items[items.length - 1];
        if (event.shiftKey && document.activeElement === first) {
          event.preventDefault();
          last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first.focus();
        }
      }
    }

    overlay.appendChild(dialog);
    document.body.appendChild(overlay);
    document.addEventListener("keydown", onKeyDown, true);
    if (typeof options.onBuild === "function") options.onBuild(api);
    var initial = focusables(dialog);
    if (initial.length) initial[0].focus();
    else { dialog.setAttribute("tabindex", "-1"); dialog.focus(); }
    return api;
  };
})(window);
