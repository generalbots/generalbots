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

  // The fragment routes (/suite/docs/fragments/*) are proxied to botserver,
  // which rejects a token-less call with 401/403 — so the docs app could never
  // list or open a document. Same header the other docs calls already send.
  function docsAuthHeaders(extra) {
    var headers = Object.assign({}, extra || {});
    var token = (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) || "";
    if (token) headers.Authorization = "Bearer " + token;
    return headers;
  }

  function json(url, options) {
    var opts = Object.assign({}, options || {});
    opts.headers = docsAuthHeaders(opts.headers);
    return fetch(url, opts).then(function (response) {
      if (!response.ok) throw new Error(url + " -> " + response.status);
      return response.json();
    });
  }

  function renderView(doc) {
    var host = document.getElementById("docs-content");
    if (!host) return Promise.resolve();
    return fetch("/suite/docs/fragments/view", {
      method: "POST",
      headers: docsAuthHeaders({ "Content-Type": "application/json" }),
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
      .catch(function () {
        // The fragment route is server-rendered; when it is unavailable (403,
        // offline desktop build) the app still has to be editable instead of
        // showing an empty window.
        host.innerHTML = localViewHtml(doc);
        bindLocalView(host);
        return null;
      })
      .then(function (html) {
        if (html !== null && html !== undefined) {
          host.innerHTML = html;
          processHtmx(host);
        }
        if (typeof attachEditorHandlers === "function") attachEditorHandlers(host);
        // Tag the rendered article with the semantic fill target so a UI-hint
        // plan can write into an already-open document too (#2184).
        if (typeof ensureDocArticle === "function") ensureDocArticle();
        setTitle(doc.title || "Untitled Document");
        var stats = document.getElementById("doc-stats");
        if (stats) {
          var text = String(doc.content || "").replace(/<[^>]*>/g, " ");
          var words = text.trim().split(/\s+/).filter(Boolean).length;
          stats.textContent = docsT(
            "docs-word-count",
            words + (words === 1 ? " word" : " words"),
            { count: words },
          );
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
      headers: docsAuthHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify(items)
    })
      .then(function (response) {
        if (!response.ok) throw new Error("list -> " + response.status);
        return response.text();
      })
      .catch(function () {
        host.innerHTML = localSidebarHtml(items);
        bindLocalSidebar(host);
        return null;
      })
      .then(function (html) {
        if (html !== null && html !== undefined) {
          host.innerHTML = html;
          processHtmx(host);
        }
      });
  }

  /* Local renderers — the same shapes the fragments produce, built in the
   * browser so Docs works when /suite/docs/fragments/* is unavailable. */
  function escapeHtml(value) {
    return String(value === null || value === undefined ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function localSidebarHtml(items) {
    var rows = items.map(function (item) {
      var title = item.title || "Untitled Document";
      return (
        '<button type="button" class="dc-file-row" data-doc-id="' +
        escapeHtml(item.id) +
        '" style="display:block;width:100%;text-align:left;padding:10px 12px;margin:0 0 4px;border:0;border-radius:6px;background:transparent;color:#e2e8f0;font-size:13px;cursor:pointer;">' +
        escapeHtml(title) +
        "</button>"
      );
    });
    return rows.join("");
  }

  function bindLocalSidebar(host) {
    host.addEventListener("click", function (event) {
      var row = event.target.closest("[data-doc-id]");
      if (!row) return;
      json("/api/docs/load?id=" + encodeURIComponent(row.dataset.docId))
        .then(renderView)
        .catch(function (error) {
          status("Could not open the document: " + error.message, true);
        });
    });
  }

  function localViewHtml(doc) {
    var pages = doc && doc.pageCount ? doc.pageCount : 1;
    return (
      '<div class="docs-pages" data-pages="' + pages + '">' +
      '<article class="docs-doc-view" contenteditable="true" spellcheck="false" ' +
      'data-doc-id="' + escapeHtml(doc.id) + '" ' +
      'data-page-h="1123" style="width:794px;min-height:1123px;padding:96px;">' +
      (doc.content || "<p><br></p>") +
      "</article></div>"
    );
  }

  function bindLocalView(host) {
    var article = host.querySelector("article.docs-doc-view");
    if (!article) return;
    if (typeof window.fitPageToWidth === "function") window.fitPageToWidth(article);
    else if (typeof window.renderPageGuides === "function") window.renderPageGuides(article);
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

  /* #2184 — empty editor surface used when a plan drives this window. */
  function startBlank() {
    var host = document.getElementById("docs-content");
    if (!host) return;
    host.innerHTML = "";
    if (typeof ensureDocArticle === "function") ensureDocArticle();
    setTitle(window.docsT ? docsT("docs-untitled", "Untitled Document") : "Untitled Document");
  }

  function start() {
    if (!document.getElementById("docs-content")) return;
    // A deep link has no query string of its own (the shell URL stays
    // /suite/desktop.html) and passes the file through __gbAppParams__. Without
    // this guard start() loaded/created a blank document in parallel with the
    // Drive open and the last fetch won, so a chat attachment opened a blank
    // Docs (#1711).
    var deepLink = window.__gbAppParams__ || {};
    var params = new URLSearchParams(window.location.search);
    // #2184 — a UI-hint plan opens Docs expecting an EMPTY surface to draw in
    // (blank: true). Loading the most recent document here would fill the
    // wrong one and the plan's fill would land on a stale doc.
    if (params.get("blank") === "true" || deepLink.blank === true) {
      startBlank();
      return;
    }
    if (params.get("file") || deepLink.file) return;
    // #2184 — the same race applies to a VM/agent-workspace file
    // (?vmConvId=&vmPath=): 99_init renders it, then this start() loaded the
    // most recent Drive document on top and the last write won, so "Open in
    // app" showed an unrelated document instead of the generated file.
    if (params.get("vmConvId") || params.get("vmPath") || deepLink.vmConvId || deepLink.vmPath) {
      return;
    }
    var wanted = params.get("id");
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

  // A window that is already open (re-targeted by a plan) must also clear
  // itself when the caller asks for a blank surface.
  document.addEventListener("gb:deep-link", function (event) {
    var params = (event && event.detail && event.detail.params) || {};
    if (params.blank === true) startBlank();
  });

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
