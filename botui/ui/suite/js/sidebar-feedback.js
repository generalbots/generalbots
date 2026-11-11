"use strict";

(function () {
  if (window.GBSidebarFeedback) return;

  function qs(sel) { return document.querySelector(sel); }

  function getBotName() {
    return (window.__SELECTED_BOT_NAME__ || window.__INITIAL_BOT_NAME__ || "default");
  }

  function truncateTitle(text) {
    var t = (text || "").replace(/\s+/g, " ").trim();
    if (!t) return "Feedback";
    if (t.length > 80) return t.slice(0, 77) + "...";
    return t;
  }

  function buildBody(text, product) {
    var lines = [];
    lines.push((text || "").trim());
    lines.push("");
    lines.push("---");
    lines.push("**Context**");
    if (product) {
      if (product.version) lines.push("- App version: " + product.version);
      if (product.commit) lines.push("- Commit: " + product.commit);
      if (product.edition) lines.push("- Edition: " + product.edition);
    }
    lines.push("- Bot: " + getBotName());
    try { lines.push("- URL: " + window.location.href); } catch (e) {}
    try { lines.push("- User agent: " + navigator.userAgent); } catch (e) {}
    return lines.join("\n");
  }

  function openIssue(text) {
    var url = "https://github.com/generalbots/generalbots/issues/new";
    var title = truncateTitle(text.split("\n")[0] || text);
    function go(product) {
      var body = buildBody(text, product);
      var params = new URLSearchParams();
      params.set("title", title);
      params.set("body", body);
      params.set("labels", "feedback");
      var full = url + "?" + params.toString();
      try {
        window.open(full, "_blank", "noopener");
      } catch (e) {
        window.location.href = full;
      }
    }
    // try to fetch product context
    try {
      if (window.__GB_PRODUCT__) { go(window.__GB_PRODUCT__); return; }
      fetch("/api/product", { credentials: "same-origin" })
        .then(function (r) { return r.ok ? r.json() : null; })
        .then(function (d) { go(d || null); })
        .catch(function () { go(null); });
    } catch (e) {
      go(null);
    }
  }

  function init() {
    var textarea = qs("#sidebarFeedbackText");
    var btn = qs("#sidebarFeedbackBtn");
    var collapsed = qs("#sidebarFeedbackCollapsed");
    if (!textarea || !btn) return;

    function updateState() {
      var hasText = (textarea.value || "").trim().length > 0;
      btn.disabled = !hasText;
    }

    textarea.addEventListener("input", updateState);
    updateState();

    btn.addEventListener("click", function () {
      var text = (textarea.value || "").trim();
      if (!text) return;
      openIssue(text);
      // keep text (per UX: pick one and document - we keep it so user can retry/edit)
    });

    if (collapsed) {
      collapsed.addEventListener("click", function (e) {
        e.stopPropagation();
        // expand sidebar if collapsed to show the box
        var sidebar = qs("#chatSidebar") || qs(".chat-sidebar");
        if (sidebar && sidebar.classList.contains("collapsed") && typeof window.toggleChatSidebar === "function") {
          window.toggleChatSidebar();
        }
        setTimeout(function () {
          var ta = qs("#sidebarFeedbackText");
          if (ta) { try { ta.focus(); } catch (err) {} }
        }, 150);
      });
    }

    window.GBSidebarFeedback = { openIssue: openIssue, init: init };
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
