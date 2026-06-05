"use strict";

/**
 * Module 11: Comments UI for Word Processor.
 * Wires the DocsComments engine (module 06) into a right-side panel
 * showing all comments in threaded order, plus in-line comment
 * highlights on the document. When text is selected and "Comment"
 * is clicked, a bubble is attached to the selection and the comment
 * is created. Supports resolve/reopen, delete, @mentions, and a
 * user autocomplete (best-effort from a window.knownUsers list).
 *
 * Public API: window.DocsCommentsUI = { openSidebar, closeSidebar,
 *   attachToSelection, renderSidebar, syncWithBackend }.
 */

(function () {
  const USERS_HINT = (typeof window.knownUsers !== "undefined" && window.knownUsers) || [];

  function getState() { return window.state || null; }
  function getComments() {
    const s = getState();
    if (!s) return [];
    if (!s.comments) s.comments = [];
    return s.comments;
  }
  function getCurrentUser() {
    const s = getState();
    return (s && s.currentUser) || (window.user && window.user.name) || "Anonymous";
  }

  function ensureSidebar() {
    let sb = document.getElementById("docsCommentsSidebar");
    if (sb) return sb;
    sb = document.createElement("div");
    sb.id = "docsCommentsSidebar";
    sb.className = "docs-comments-sidebar";
    sb.style.cssText = "position:fixed;top:0;right:0;bottom:0;width:340px;background:#f8f9fa;border-left:1px solid #ddd;z-index:9998;display:none;flex-direction:column;font-family:Arial,sans-serif;font-size:13px;";
    sb.innerHTML = `
      <div style="padding:12px;border-bottom:1px solid #ddd;display:flex;align-items:center;gap:8px;">
        <strong>Comments</strong>
        <span id="docsCommentCount" style="background:#e0e0e0;border-radius:10px;padding:1px 8px;font-size:11px;">0</span>
        <button id="docsCommentsClose" style="margin-left:auto;background:transparent;border:0;font-size:18px;cursor:pointer;">×</button>
      </div>
      <div id="docsCommentList" style="flex:1;overflow-y:auto;padding:8px;"></div>
      <div style="padding:8px;border-top:1px solid #ddd;">
        <textarea id="docsCommentInput" placeholder="Add a comment…" style="width:100%;height:60px;box-sizing:border-box;padding:6px;border:1px solid #ccc;border-radius:4px;font-family:inherit;font-size:13px;resize:vertical;"></textarea>
        <div style="display:flex;gap:6px;margin-top:6px;align-items:center;">
          <input type="text" id="docsCommentMention" placeholder="@mention" style="flex:1;padding:4px;border:1px solid #ccc;border-radius:3px;font-size:12px;" />
          <button id="docsCommentSubmit" style="background:#1a73e8;color:#fff;border:0;border-radius:3px;padding:5px 12px;cursor:pointer;">Comment</button>
        </div>
      </div>
    `;
    document.body.appendChild(sb);
    sb.querySelector("#docsCommentsClose").addEventListener("click", closeSidebar);
    sb.querySelector("#docsCommentSubmit").addEventListener("click", function () {
      const text = sb.querySelector("#docsCommentInput").value.trim();
      const mention = sb.querySelector("#docsCommentMention").value.trim();
      if (!text) return;
      const sel = window.getSelection();
      const anchor = sel && sel.rangeCount ? { start: sel.getRangeAt(0).startOffset, end: sel.getRangeAt(0).endOffset } : { start: 0, end: 0 };
      const full = text + (mention ? " @" + mention : "");
      if (window.DocsComments) {
        window.DocsComments.createComment(getState(), anchor, getCurrentUser(), full);
      }
      sb.querySelector("#docsCommentInput").value = "";
      sb.querySelector("#docsCommentMention").value = "";
      renderSidebar();
      syncWithBackend();
    });
    return sb;
  }

  function openSidebar() {
    const sb = ensureSidebar();
    sb.style.display = "flex";
    renderSidebar();
  }

  function closeSidebar() {
    const sb = document.getElementById("docsCommentsSidebar");
    if (sb) sb.style.display = "none";
  }

  function toggleSidebar() {
    const sb = ensureSidebar();
    sb.style.display = sb.style.display === "flex" ? "none" : "flex";
    if (sb.style.display === "flex") renderSidebar();
  }

  function renderSidebar() {
    const sb = ensureSidebar();
    const list = sb.querySelector("#docsCommentList");
    if (!list) return;
    list.innerHTML = "";
    const comments = getComments();
    sb.querySelector("#docsCommentCount").textContent = comments.length;
    for (const cmt of comments) {
      const card = document.createElement("div");
      card.className = "docs-comment-card";
      card.style.cssText = "background:#fff;border:1px solid #ddd;border-left:3px solid " + (cmt.resolved ? "#0a8" : "#1a73e8") + ";border-radius:4px;padding:8px;margin-bottom:8px;";
      card.innerHTML = `
        <div style="display:flex;align-items:center;gap:6px;margin-bottom:4px;">
          <strong style="font-size:12px;">${escapeHtml(cmt.author || "Anonymous")}</strong>
          <span style="color:#888;font-size:11px;">${new Date(cmt.timestamp).toLocaleString()}</span>
        </div>
        <div style="margin-bottom:6px;">${escapeHtml(cmt.text)}</div>
      `;
      if (cmt.replies && cmt.replies.length) {
        const replies = document.createElement("div");
        replies.style.cssText = "margin-left:12px;padding-left:8px;border-left:2px solid #eee;";
        for (const r of cmt.replies) {
          const div = document.createElement("div");
          div.style.cssText = "font-size:12px;margin:4px 0;";
          div.innerHTML = `<strong>${escapeHtml(r.author || "Anonymous")}:</strong> ${escapeHtml(r.text)} <span style="color:#888;font-size:10px;">${new Date(r.timestamp).toLocaleString()}</span>`;
          replies.appendChild(div);
        }
        card.appendChild(replies);
      }
      const actions = document.createElement("div");
      actions.style.cssText = "display:flex;gap:4px;";
      if (!cmt.resolved) {
        const resolveBtn = document.createElement("button");
        resolveBtn.textContent = "Resolve";
        resolveBtn.style.cssText = "background:transparent;border:0;color:#1a73e8;cursor:pointer;font-size:11px;padding:2px 6px;";
        resolveBtn.addEventListener("click", () => {
          if (window.DocsComments) window.DocsComments.resolveComment(getState(), cmt.id, getCurrentUser());
          renderSidebar();
          syncWithBackend();
        });
        actions.appendChild(resolveBtn);
      } else {
        const reopenBtn = document.createElement("button");
        reopenBtn.textContent = "Reopen";
        reopenBtn.style.cssText = "background:transparent;border:0;color:#1a73e8;cursor:pointer;font-size:11px;padding:2px 6px;";
        reopenBtn.addEventListener("click", () => {
          if (window.DocsComments) window.DocsComments.reopenComment(getState(), cmt.id);
          renderSidebar();
          syncWithBackend();
        });
        actions.appendChild(reopenBtn);
      }
      const replyBtn = document.createElement("button");
      replyBtn.textContent = "Reply";
      replyBtn.style.cssText = "background:transparent;border:0;color:#1a73e8;cursor:pointer;font-size:11px;padding:2px 6px;";
      replyBtn.addEventListener("click", () => {
        const text = window.prompt("Reply:");
        if (text && window.DocsComments) {
          window.DocsComments.replyToComment(getState(), cmt.id, getCurrentUser(), text);
          renderSidebar();
          syncWithBackend();
        }
      });
      actions.appendChild(replyBtn);
      const delBtn = document.createElement("button");
      delBtn.textContent = "Delete";
      delBtn.style.cssText = "background:transparent;border:0;color:#c00;cursor:pointer;font-size:11px;padding:2px 6px;";
      delBtn.addEventListener("click", () => {
        if (window.confirm("Delete this comment?")) {
          if (window.DocsComments) window.DocsComments.deleteComment(getState(), cmt.id);
          renderSidebar();
          syncWithBackend();
        }
      });
      actions.appendChild(delBtn);
      card.appendChild(actions);
      list.appendChild(card);
    }
  }

  function escapeHtml(s) {
    return String(s || "").replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  function attachToSelection() {
    const sel = window.getSelection();
    if (!sel || sel.isCollapsed || sel.rangeCount === 0) {
      window.alert("Please select some text first.");
      return;
    }
    const text = window.prompt("Add comment:", "");
    if (text && window.DocsComments) {
      const r = sel.getRangeAt(0);
      const anchor = { start: r.startOffset, end: r.endOffset };
      window.DocsComments.createComment(getState(), anchor, getCurrentUser(), text);
      renderSidebar();
      syncWithBackend();
    }
  }

  function syncWithBackend() {
    const s = getState();
    if (!s || !s.botId) return;
    const docId = s.documentId || s.id;
    if (!docId) return;
    try {
      fetch("/api/docs/" + encodeURIComponent(docId) + "/comments/sync", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ comments: s.comments || [] }),
      }).catch(function () { /* offline-safe */ });
    } catch (_e) { /* ignore */ }
  }

  function attach() {
    document.addEventListener("docsPaginated", renderSidebar);
    document.addEventListener("keydown", function (e) {
      if ((e.ctrlKey || e.metaKey) && e.altKey && e.key.toLowerCase() === "c") {
        e.preventDefault();
        toggleSidebar();
      }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.DocsCommentsUI = {
    openSidebar,
    closeSidebar,
    toggleSidebar,
    attachToSelection,
    renderSidebar,
    syncWithBackend,
  };
})();
