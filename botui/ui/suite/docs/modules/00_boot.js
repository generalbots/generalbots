"use strict";
/* docs boot — binds the JSON document API to the server-rendered fragments.
 *
 * The editor shell in docs.html is an HTMX surface, but /api/docs/* speaks
 * JSON. Without this bridge the sidebar dumped the raw list JSON and the
 * document view requested a fragment route that no longer exists.
 *
 * Flow:
 *   GET  /api/docs/list              -> document metadata[]
 *   POST /suite/docs/fragments/doc-list  -> sidebar HTML
 *   GET  /api/docs/load?id=<id>      -> full document
 *   POST /suite/docs/fragments/view  -> editor HTML (article[contenteditable])
 *
 * The app is normally injected into the desktop shell, where DOMContentLoaded
 * has already fired, so start() also runs immediately when the document is
 * ready.
 */
(function () {
  function status(text, isError) {
    if (typeof setSaveStatus === "function") setSaveStatus(text, isError);
  }

  function setTitle(title) {
    var el = document.getElementById("docTitle");
    if (el && title) el.value = title;
  }

  function activeArticle() {
    return document.querySelector("#docs-content article[contenteditable]");
  }

  function activeDocId() {
    var article = activeArticle();
    return article && article.dataset ? (article.dataset.docId || "current") : "current";
  }

  function processHtmx(host) {
    if (window.htmx && typeof htmx.process === "function") htmx.process(host);
  }

  function json(url, options) {
    return fetch(url, options).then(function (response) {
      if (!response.ok) throw new Error(url + " -> " + response.status);
      return response.json();
    });
  }

  function renderView(doc) {
    var host = document.getElementById("docs-content");
    if (!host) return Promise.resolve();
    return fetch("/suite/docs/fragments/view", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        id: doc.id,
        title: doc.title,
        content: doc.content
      })
    })
      .then(function (response) {
        if (!response.ok) throw new Error("view -> " + response.status);
        return response.text();
      })
      .then(function (html) {
        host.innerHTML = html;
        processHtmx(host);
        if (typeof attachEditorHandlers === "function") attachEditorHandlers(host);
        setTitle(doc.title || "Untitled Document");
        var stats = document.getElementById("doc-stats");
        if (stats) {
          var text = String(doc.content || "").replace(/<[^>]*>/g, " ");
          var words = text.trim().split(/\s+/).filter(Boolean).length;
          stats.textContent = words + (words === 1 ? " word" : " words");
        }
        try {
          history.replaceState(null, "", "?id=" + encodeURIComponent(doc.id));
        } catch (ignored) {
          /* file:// or a host that forbids history updates */
        }
      });
  }

  function renderSidebar(items) {
    var host = document.getElementById("sidebar-files");
    if (!host) return Promise.resolve();
    if (!items.length) {
      host.innerHTML =
        '<div style="padding:14px;color:#94a3b8;font-size:12px;">No documents yet. Click + Novo.</div>';
      return Promise.resolve();
    }
    return fetch("/suite/docs/fragments/doc-list", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(items)
    })
      .then(function (response) {
        if (!response.ok) throw new Error("list -> " + response.status);
        return response.text();
      })
      .then(function (html) {
        host.innerHTML = html;
        processHtmx(host);
      });
  }

  function saveTitle() {
    var input = document.getElementById("docTitle");
    var article = activeArticle();
    if (!input || !article) return;
    fetch("/api/docs/save", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        id: activeDocId(),
        title: input.value,
        content: article.innerHTML
      })
    })
      .then(function (response) {
        if (!response.ok) throw new Error("http " + response.status);
        status("All changes saved", false);
      })
      .catch(function () {
        status("Title not saved", true);
      });
  }

  function start() {
    if (!document.getElementById("docs-content")) return;
    if (new URLSearchParams(window.location.search).get("file")) return;
    var wanted = new URLSearchParams(window.location.search).get("id");
    json("/api/docs/list")
      .then(function (items) {
        return renderSidebar(items).then(function () {
          if (wanted && items.some(function (item) { return item.id === wanted; })) {
            return json("/api/docs/load?id=" + encodeURIComponent(wanted)).then(renderView);
          }
          if (items.length) {
            return json("/api/docs/load?id=" + encodeURIComponent(items[0].id)).then(renderView);
          }
          return json("/api/docs/new").then(renderView);
        });
      })
      .catch(function (error) {
        status("Docs failed to load: " + error.message, true);
      });
  }

  document.addEventListener(
    "blur",
    function (event) {
      if (event.target && event.target.id === "docTitle") saveTitle();
    },
    true
  );

  document.addEventListener("htmx:afterSwap", function (event) {
    if (event.target && event.target.id === "docs-content") {
      var view = document.getElementById("doc-view");
      if (view && view.dataset && view.dataset.title) setTitle(view.dataset.title);
    }
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
})();
