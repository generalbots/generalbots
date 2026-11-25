"use strict";
/* docs collab — WebSocket collaboration
 *
 * Collaboration is bound to the *rendered document*, not to the window. The
 * desktop shell opens Docs before any article exists and re-targets the same
 * window when another document is opened, so a single connect() at startup
 * either sent the placeholder title as the doc id (rejected by the server) or
 * stayed attached to the first document forever (#doc-collab-doc-id).
 *
 * Lifecycle (all in this module — no other app/module is touched):
 *   - initCollab() only installs a watcher plus one initial sync; it never
 *     opens a socket while there is no article with a valid data-doc-id.
 *   - a blank doc's minted id is flagged unsaved until its first successful
 *     save; renderedDocId() then stays empty so no refused socket (and no
 *     reconnect storm) is ever opened for a document Drive does not have.
 *   - every render path (server fragment, local view, Drive/VM deep link,
 *     blank start) mutates the DOM, which re-syncs against the article id.
 *   - switching documents updates the GBCollab singleton's doc id/url in place
 *     and uses its guarded native reconnect. Calling connect() again would
 *     re-run _init(), orphan the old socket and drop the reconnect-timer handle,
 *     leaving two sockets open.
 */
var docsCollabReady = false;

function initCollab() {
  if (!window.GBCollab || docsCollabReady) return;
  docsCollabReady = true;

  var gb = window.GBCollab;
  var typingEl = document.getElementById("typing-indicator");
  var currentDocId = "";
  var syncScheduled = false;

  // Only the real document id may go on the wire. #docTitle holds the
  // human-readable name ("Untitled Document") and must never be used: the
  // server validates a UUID/slug and closed the socket ~100ms after a hello
  // carrying a title (#GB-UAT 27893d4e). "current" is a placeholder that
  // docsEnsureArticleDocId() replaces with a real id as soon as it can.
  function renderedDocId() {
    var article = document.querySelector("article[contenteditable]");
    var id = article && article.dataset ? article.dataset.docId : "";
    // A blank doc mints its id before it exists in Drive. The server refuses
    // that socket and the close handler reopens it in a loop, so while the
    // article is flagged unsaved there is no id to put on the wire at all.
    if (id && typeof docsIsBlankDocUnsaved === "function" && docsIsBlankDocUnsaved(article)) return "";
    if (!id || id === "current" || !/^[A-Za-z0-9_.-]+$/.test(id)) return "";
    return id;
  }

  // Same address GBCollab._init() builds; kept in step when we swap the id.
  function collabUrl(id) {
    var loc = window.location;
    var proto = loc.protocol === "https:" ? "wss:" : "ws:";
    return proto + "//" + loc.host + "/ws/docs/" + encodeURIComponent(id);
  }

  var opts = {
    app: "docs",
    collaboratorsEl: document.getElementById("collaborators"),
    typingEl: typingEl,
    onConnect: function () {},
    onDisconnect: function () {
      if (window.DocsPresence) window.DocsPresence.clearAll();
    },
    onMessage: function (msg) {
      if (!msg || !window.DocsPresence) return;
      if (msg.msg_type === "cursor") window.DocsPresence.cursor(msg);
    },
    onSelection: function (msg) {
      if (window.DocsPresence) window.DocsPresence.selection(msg);
    },
    onPresence: function (users) {
      if (window.DocsPresence) window.DocsPresence.sync(users);
    },
    onTyping: function (msg) {
      var map = (window.__gbTypingUsers = window.__gbTypingUsers || new Map());
      if (msg.msg_type === "typing_start") map.set(msg.user_id, msg);
      else map.delete(msg.user_id);
      var arr = Array.from(map.values()).filter(function (m) { return Date.now() - (m.timestamp || 0) < 5000; });
      if (window.GBCollab && window.GBCollab.helpers) {
        window.GBCollab.helpers.renderTypingIndicator(typingEl, arr);
      }
    },
    onEdit: function (msg) {
      if (!msg) return;
      var article = document.querySelector("article[contenteditable]");
      if (!article || article.dataset.suppressRemote) return;
      var hasDelta = typeof msg.position === "number" && (typeof msg.removeLength === "number" || typeof msg.length === "number");
      article.dataset.suppressRemote = "1";
      if (hasDelta) {
        var pos = Math.max(0, msg.position | 0);
        var removeLength = typeof msg.removeLength === "number" ? msg.removeLength : (msg.length && msg.length > 0 && msg.content === "" ? msg.length : 0);
        applyDeltaEdit(article, pos, msg.content, removeLength);
      } else if (typeof msg.content === "string") {
        // #2840: a remote edit is untrusted input — sanitize before render.
        var offset = getCaretCharacterOffsetWithin(article);
        article.innerHTML = typeof docsSanitizeHtml === "function" ? docsSanitizeHtml(msg.content) : "";
        setCaretPosition(article, offset);
      }
      article.dataset.suppressRemote = "";
      updatePageCount();
    }
  };

  // Idempotent: a second sync with the same id is a no-op, so re-renders,
  // presence avatars and keystrokes cannot create a reconnect storm.
  function sync() {
    var id = renderedDocId();
    if (!id || id === currentDocId) return;
    var hadConnection = !!currentDocId;
    currentDocId = id;
    if (!hadConnection) {
      opts.docId = id;
      gb.connect(opts);
      return;
    }
    // Re-target without a second connect(): close the old socket and let the
    // singleton's _scheduleReconnect() open exactly one socket for the new id.
    if (gb.opts) gb.opts.docId = id;
    gb.docId = id;
    gb.url = collabUrl(id);
    gb.disconnect();
    gb._scheduleReconnect();
  }

  function scheduleSync() {
    if (syncScheduled) return;
    syncScheduled = true;
    setTimeout(function () { syncScheduled = false; sync(); }, 0);
  }

  // Only react to the document container being replaced (innerHTML swaps for
  // server/local views, append for blank start) or to data-doc-id changing.
  // Ignore text-node churn from typing and presence avatar re-renders.
  function onMutations(records) {
    for (var i = 0; i < records.length; i++) {
      var record = records[i];
      if (record.type === "attributes") { scheduleSync(); return; }
      var nodes = record.addedNodes.length ? record.addedNodes : record.removedNodes;
      for (var j = 0; j < nodes.length; j++) {
        var node = nodes[j];
        if (node.nodeType === 1 &&
            (node.tagName === "ARTICLE" ||
             (node.querySelector && node.querySelector("article[contenteditable]")))) {
          scheduleSync();
          return;
        }
      }
    }
  }

  sync();

  if (typeof MutationObserver === "function") {
    // Watch the document element, not a single #docs-content node: the shell
    // replaces the window body on every open but this page is never reloaded.
    new MutationObserver(onMutations).observe(document.documentElement, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ["data-doc-id"]
    });
  }

  // Lets a deep-link handler force a check after re-targeting a window.
  window.docsSyncCollab = sync;
}
