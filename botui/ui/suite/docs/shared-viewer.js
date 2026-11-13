"use strict";
/* Shared document viewer (#2253).
 *
 * Opened by the URL the Share dialog copies:
 *   /suite/docs/shared.html?link=<token>
 *
 * The page is standalone on purpose: a reader on a share link has no session,
 * so it must not boot the desktop shell (login redirect) nor the editor (writes
 * that the server rejects). The document is resolved by the server from the
 * token alone — the URL never carries a document id a reader could swap.
 */
(function (window) {
  var TOKEN_PARAM = "link";
  var ENDPOINT = "/api/docs/shared/";

  function byId(id) {
    return document.getElementById(id);
  }

  function text(id, value) {
    var el = byId(id);
    if (el) el.textContent = value;
  }

  function showError(message) {
    var state = byId("sharedState");
    var pages = byId("sharedPages");
    var badge = byId("sharedBadge");
    if (pages) pages.hidden = true;
    if (badge) badge.hidden = true;
    if (state) {
      state.hidden = false;
      state.classList.add("docs-shared-state-error");
      state.textContent = message;
    }
    text("sharedNotice", "This link grants no access.");
  }

  /**
   * Renders authored document HTML with the tags an anonymous reader must
   * never execute removed. Document bodies are authored by the owner, but a
   * link reaches readers outside that trust boundary, so active content is
   * dropped rather than rendered.
   */
  function sanitize(html) {
    var parser = new DOMParser();
    var doc = parser.parseFromString(String(html || ""), "text/html");
    var blocked = ["script", "iframe", "object", "embed", "form", "meta", "link", "base"];
    blocked.forEach(function (tag) {
      Array.prototype.forEach.call(doc.querySelectorAll(tag), function (node) {
        if (node.parentNode) node.parentNode.removeChild(node);
      });
    });
    Array.prototype.forEach.call(doc.querySelectorAll("*"), function (node) {
      Array.prototype.slice.call(node.attributes).forEach(function (attr) {
        var name = attr.name.toLowerCase();
        var value = String(attr.value || "").replace(/\s/g, "").toLowerCase();
        var isHandler = name.indexOf("on") === 0;
        var isScriptUrl = (name === "href" || name === "src" || name === "xlink:href") &&
          value.indexOf("javascript:") === 0;
        if (isHandler || isScriptUrl) node.removeAttribute(attr.name);
      });
      if (node.tagName === "A") {
        node.setAttribute("rel", "noopener noreferrer");
        node.setAttribute("target", "_blank");
      }
    });
    return doc.body.innerHTML;
  }

  function roleLabel(role) {
    if (role === "commenter") return "shared view — comments only in the editor";
    if (role === "editor") return "shared view — editing needs a sign-in";
    return "read-only";
  }

  function render(doc) {
    var pages = byId("sharedPages");
    var state = byId("sharedState");
    var badge = byId("sharedBadge");
    var article = byId("sharedDoc");
    var title = doc.title && String(doc.title).trim() ? String(doc.title) : "Shared document";

    text("sharedTitle", title);
    document.title = title + " — Shared (read-only)";
    if (article) {
      article.innerHTML = sanitize(doc.content);
      article.setAttribute("contenteditable", "false");
      if (doc.id) article.setAttribute("data-doc-id", String(doc.id));
    }
    if (state) {
      state.hidden = true;
      state.textContent = "";
    }
    if (pages) pages.hidden = false;
    if (badge) badge.hidden = false;
    text(
      "sharedNotice",
      "This document was shared as a read-only link (" + roleLabel(doc.role) + "). " +
        "Reopen the link to see the latest version."
    );
  }

  function boot() {
    var params = new URLSearchParams(window.location.search);
    var token = (params.get(TOKEN_PARAM) || "").trim();
    if (!token) {
      showError("This link is missing its share token.");
      return;
    }
    fetch(ENDPOINT + encodeURIComponent(token), {
      headers: { Accept: "application/json" },
      credentials: "omit",
    })
      .then(function (response) {
        if (response.status === 404) throw new Error("gone");
        if (!response.ok) throw new Error("http " + response.status);
        return response.json();
      })
      .then(function (doc) {
        if (!doc || typeof doc.content !== "string") throw new Error("shape");
        render(doc);
      })
      .catch(function (error) {
        if (error && error.message === "gone") {
          showError("This shared link is no longer available.");
          return;
        }
        showError("The shared document could not be opened. Try again later.");
      });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})(window);
