"use strict";
/* Bookmarks (#2485): create/name/delete/go-to plus cross-reference insertion.
 * Bookmarks are stored per document in localStorage; the in-document marker is
 * a real anchor so TOC/export and cross-reference links resolve to it. */

(function () {
  var NS = window.DocsNav;
  if (!NS) return;
  var tr = NS.tr;

  function mintId() {
    if (typeof crypto !== "undefined" && crypto && typeof crypto.randomUUID === "function") {
      try { return "bm-" + crypto.randomUUID(); } catch (error) { /* fall through */ }
    }
    return "bm-" + Date.now().toString(36) + "-" + Math.random().toString(36).slice(2, 8);
  }
  NS.mintBookmarkId = mintId;

  function all() {
    return NS.readStore("bookmarks", []);
  }
  function store(items) {
    NS.writeStore("bookmarks", items);
  }

  function markerNode(id) {
    var article = NS.getArticle();
    if (!article) return null;
    return article.querySelector('[id="' + id + '"]');
  }

  function insertAtCaret(node) {
    var article = NS.getArticle();
    if (!article || !node) return false;
    article.focus();
    var selection = window.getSelection();
    var range = selection && selection.rangeCount > 0 ? selection.getRangeAt(0) : null;
    if (range && article.contains(range.startContainer)) {
      range.deleteContents();
      range.insertNode(node);
      var after = document.createRange();
      after.setStartAfter(node);
      after.collapse(true);
      selection.removeAllRanges();
      selection.addRange(after);
    } else {
      article.appendChild(node);
    }
    return true;
  }

  NS.addBookmark = function (name, offset) {
    var article = NS.getArticle();
    if (!article) return null;
    var items = all();
    var id = mintId();
    var at = typeof offset === "number"
      ? offset
      : (typeof getCaretCharacterOffsetWithin === "function" ? getCaretCharacterOffsetWithin(article) : 0);
    var marker = document.createElement("span");
    marker.id = id;
    marker.className = "gb-bookmark-marker";
    marker.setAttribute("data-gb-bookmark", name || "");
    marker.contentEditable = "false";
    if (!insertAtCaret(marker)) return null;
    items.push({ id: id, name: name || id, offset: at });
    store(items);
    NS.save();
    NS.status(tr("docs-bm-created", "Bookmark created"));
    if (NS.refreshPane) NS.refreshPane();
    return id;
  };

  NS.deleteBookmark = function (id) {
    var items = all().filter(function (item) { return item.id !== id; });
    store(items);
    var marker = markerNode(id);
    if (marker && marker.parentNode) marker.parentNode.removeChild(marker);
    NS.save();
    NS.status(tr("docs-bm-deleted", "Bookmark deleted"));
    if (NS.refreshPane) NS.refreshPane();
  };

  NS.gotoBookmark = function (id) {
    var marker = markerNode(id);
    if (!marker) {
      NS.status(tr("docs-bm-missing", "Bookmark not found in this document"));
      return false;
    }
    NS.jumpTo(marker);
    return true;
  };

  NS.insertBookmarkRef = function (id) {
    var item = all().filter(function (entry) { return entry.id === id; })[0];
    if (!item || !markerNode(id)) {
      NS.status(tr("docs-bm-missing", "Bookmark not found in this document"));
      return false;
    }
    var link = document.createElement("a");
    link.setAttribute("href", "#" + id);
    link.setAttribute("data-gb-xref", id);
    link.textContent = item.name || id;
    if (!insertAtCaret(link)) return false;
    NS.save();
    NS.status(tr("docs-bm-xref-inserted", "Cross-reference inserted"));
    return true;
  };

  function close(host, restore) {
    host.innerHTML = "";
    if (restore && typeof restore.focus === "function") restore.focus();
  }

  NS.openBookmarkDialog = function () {
    var host = document.getElementById("docs-modal-container");
    if (!host) return;
    var restore = document.activeElement;
    host.innerHTML = [
      '<div class="docs-modal" style="position:fixed;inset:0;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;">',
      '<div class="gb-bm-dialog" role="dialog" aria-modal="true" aria-labelledby="gb-bm-title" style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:360px;max-width:90vw;padding:20px;color:#f8fafc;">',
      '<h3 id="gb-bm-title" style="margin:0 0 12px;font-size:16px;">' + NS.escapeHtml(tr("docs-bm-add", "Add bookmark")) + "</h3>",
      '<label style="font-size:13px;display:block;">' + NS.escapeHtml(tr("docs-bm-name", "Bookmark name")) +
      '<input id="gb-bm-name" type="text" style="display:block;width:100%;margin-top:6px;padding:8px;background:#0f172a;border:1px solid #334155;color:#f8fafc;border-radius:4px;" /></label>',
      '<div style="display:flex;gap:8px;justify-content:flex-end;margin-top:14px;">' +
      '<button type="button" id="gb-bm-cancel" class="gb-fr-btn">' + NS.escapeHtml(tr("docs-find-close", "Close")) + "</button>" +
      '<button type="button" id="gb-bm-create" class="gb-fr-btn gb-fr-primary">' + NS.escapeHtml(tr("docs-bm-create", "Create")) + "</button></div>",
      "</div></div>"
    ].join("");
    var name = host.querySelector("#gb-bm-name");
    var onKey = function (event) {
      if (event.key === "Escape") {
        document.removeEventListener("keydown", onKey, true);
        close(host, restore);
      }
    };
    document.addEventListener("keydown", onKey, true);
    host.querySelector("#gb-bm-cancel").addEventListener("click", function () {
      document.removeEventListener("keydown", onKey, true);
      close(host, restore);
    });
    host.querySelector("#gb-bm-create").addEventListener("click", function () {
      document.removeEventListener("keydown", onKey, true);
      NS.addBookmark((name && name.value || "").trim());
      close(host, restore);
    });
    if (name) name.focus();
  };

  document.addEventListener("click", function (event) {
    var goto = event.target.closest("[data-bm-goto]");
    if (goto) {
      event.preventDefault();
      NS.gotoBookmark(goto.getAttribute("data-bm-goto"));
      return;
    }
    var del = event.target.closest("[data-bm-delete]");
    if (del) {
      event.preventDefault();
      NS.deleteBookmark(del.getAttribute("data-bm-delete"));
    }
  });
})();
