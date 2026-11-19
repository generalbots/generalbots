"use strict";
/* Slides table & media authoring (#2487) — shared namespace, accessible dialogs
 * and deck-scoped local authoring state. Structured table editing and poster /
 * caption metadata are not representable by the current server model, so they
 * are kept locally and clearly labelled as such. */
(function () {
  var STM = (window.SlidesTableMedia = window.SlidesTableMedia || {});
  var dialogSeq = 0;

  STM.tr = function (key, fallback, params) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
  };

  STM.deck = function () {
    return (typeof window.getSlidesPresentationId === "function" && window.getSlidesPresentationId()) || "current";
  };
  STM.slide = function () {
    var canvas = document.getElementById("slide-canvas");
    return canvas ? parseInt(canvas.dataset.slideId, 10) || 0 : 0;
  };
  STM.canvas = function () {
    return document.getElementById("slide-canvas");
  };
  STM.refresh = function () {
    if (typeof window.refreshSlideView === "function") window.refreshSlideView(STM.slide());
  };

  function blank() { return { tables: {}, media: {}, captions: {} }; }

  STM.storeKey = function () { return "slides-stm:" + STM.deck(); };
  STM.load = function () {
    try {
      var raw = window.localStorage.getItem(STM.storeKey());
      var parsed = raw ? JSON.parse(raw) : null;
      if (!parsed || typeof parsed !== "object") return blank();
      parsed.tables = parsed.tables || {};
      parsed.media = parsed.media || {};
      parsed.captions = parsed.captions || {};
      return parsed;
    } catch (e) {
      return blank();
    }
  };
  STM.save = function (data) {
    try {
      window.localStorage.setItem(STM.storeKey(), JSON.stringify(data || blank()));
      return true;
    } catch (e) {
      return false;
    }
  };
  STM.patch = function (mutator) {
    var data = STM.load();
    var result = mutator(data);
    STM.save(data);
    return result;
  };

  STM.status = function (key, fallback, params) {
    var text = STM.tr(key, fallback, params);
    var host = document.getElementById("slides-tools-status");
    if (host) host.textContent = text;
    return text;
  };

  STM.escapeHtml = function (value) {
    return String(value === null || value === undefined ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  };
  STM.uid = function (prefix) {
    return (prefix || "stm") + "_" + Date.now().toString(36) + "_" + Math.random().toString(36).slice(2, 7);
  };

  STM.authHeaders = function (json) {
    var headers = { "Content-Type": "application/json" };
    try {
      var token = window.localStorage.getItem("gb-access-token") || "";
      if (token) headers.Authorization = "Bearer " + token;
    } catch (e) { /* storage disabled */ }
    return headers;
  };
  STM.postJson = function (path, payload) {
    return fetch(path, {
      method: "POST",
      headers: STM.authHeaders(true),
      credentials: "same-origin",
      body: JSON.stringify(payload)
    }).then(function (response) {
      if (!response.ok) throw new Error("HTTP " + response.status);
      return response.json().catch(function () { return {}; });
    });
  };
  STM.getJson = function (path) {
    return fetch(path, { headers: { Accept: "application/json" }, credentials: "same-origin" })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.json();
      });
  };

  function labelled(el, attr, key, fallback) {
    el.setAttribute(attr, key);
    if (attr === "data-i18n") el.textContent = STM.tr(key, fallback);
    return el;
  }
  STM.labelled = labelled;

  STM.button = function (key, fallback, onClick, variant) {
    var b = document.createElement("button");
    b.type = "button";
    b.className = "stm-btn" + (variant ? " stm-btn-" + variant : "");
    labelled(b, "data-i18n", key, fallback);
    b.addEventListener("click", onClick);
    return b;
  };

  function focusables(root) {
    return Array.prototype.slice.call(
      root.querySelectorAll('a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])')
    );
  }

  /* Accessible replacement for native dialogs: labelled modal, focus trap,
   * Esc to close, focus restore and a polite status region. */
  STM.openDialog = function (options) {
    options = options || {};
    var previous = document.activeElement;
    var overlay = document.createElement("div");
    overlay.className = "stm-overlay";
    var dialog = document.createElement("div");
    dialog.className = "stm-dialog";
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = "stm-title-" + (++dialogSeq);
    dialog.setAttribute("aria-labelledby", titleId);

    var header = document.createElement("div");
    header.className = "stm-dialog-header";
    var title = document.createElement("h2");
    title.id = titleId;
    labelled(title, "data-i18n", options.titleKey || "slides-stm-title", options.titleFallback || "Table and media");
    header.appendChild(title);
    var close = document.createElement("button");
    close.type = "button";
    close.className = "stm-dialog-close";
    labelled(close, "data-i18n-aria-label", "slides-stm-close", "Close");
    close.textContent = "\u00d7";
    close.addEventListener("click", function () { api.close(); });
    header.appendChild(close);
    dialog.appendChild(header);

    var body = document.createElement("div");
    body.className = "stm-dialog-body";
    dialog.appendChild(body);

    var status = document.createElement("div");
    status.className = "stm-dialog-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    dialog.appendChild(status);

    var footer = document.createElement("div");
    footer.className = "stm-dialog-footer";
    dialog.appendChild(footer);

    var closed = false;
    var api = {
      root: dialog,
      body: body,
      footer: footer,
      overlay: overlay,
      setStatus: function (key, fallback, params) {
        status.textContent = STM.tr(key, fallback, params);
        return api;
      },
      localNote: function () {
        var note = document.createElement("p");
        note.className = "stm-local-note";
        labelled(note, "data-i18n", "slides-stm-local-only", "Local only — not saved to the server");
        return note;
      },
      addButton: function (key, fallback, onClick, variant) {
        var b = STM.button(key, fallback, onClick, variant);
        footer.appendChild(b);
        return b;
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
          event.preventDefault(); last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault(); first.focus();
        }
      }
    }

    overlay.appendChild(dialog);
    document.body.appendChild(overlay);
    document.addEventListener("keydown", onKeyDown, true);
    if (options.onBuild) options.onBuild(api);
    var initial = focusables(dialog);
    if (initial.length) initial[0].focus();
    else { dialog.setAttribute("tabindex", "-1"); dialog.focus(); }
    return api;
  };

  /* Accessible confirm(): no native window.confirm in the authoring UI. */
  STM.confirmDialog = function (titleKey, titleFallback, messageKey, messageFallback, onYes) {
    var dlg = STM.openDialog({
      titleKey: titleKey,
      titleFallback: titleFallback,
      onBuild: function (api) {
        var p = document.createElement("p");
        labelled(p, "data-i18n", messageKey, messageFallback);
        api.body.appendChild(p);
        api.addButton("slides-stm-cancel", "Cancel", function () { api.close(); });
        api.addButton("slides-stm-apply", "Apply", function () {
          api.close();
          onYes();
        }, "primary");
        api.footer.lastChild.focus();
      }
    });
    return dlg;
  };

  STM.tableElements = function () {
    var canvas = STM.canvas();
    if (!canvas) return [];
    return Array.prototype.slice.call(canvas.querySelectorAll('.sl-element[data-type="table"] table'));
  };
  STM.tableById = function (id) {
    var all = STM.tableElements();
    for (var i = 0; i < all.length; i++) {
      var el = all[i].closest(".sl-element");
      if (el && el.dataset.id === String(id)) return all[i];
    }
    return null;
  };
  STM.elementIdOf = function (table) {
    var el = table && table.closest ? table.closest(".sl-element") : null;
    return el ? el.dataset.id : "";
  };

  /* Media ids are stable: derive one from the slide + source when the server
   * has not returned an id yet (e.g. a freshly inserted local-only overlay). */
  STM.mediaId = function (src, id) {
    return id || "m_" + String(src || "").replace(/[^A-Za-z0-9]/g, "").slice(-40);
  };
  STM.mediaMeta = function (id) {
    var all = STM.load().media;
    return all[id] || {};
  };
  STM.saveMediaMeta = function (id, patch) {
    return STM.patch(function (data) {
      data.media[id] = Object.assign({}, data.media[id], patch);
      return data.media[id];
    });
  };
})();
