"use strict";

/**
 * Module 12: Track Changes UI for Word Processor.
 * Wires the DocsTrackChanges engine (module 05) into a right-side
 * panel listing all pending revisions with Accept/Reject buttons per
 * row. Wraps the editor's input event to capture insertions,
 * deletions, replacements, and formatting changes when track-changes
 * is enabled. Accept All / Reject All toolbar buttons. Renders
 * insertions as underlined green and deletions as strikethrough red.
 *
 * Public API: window.DocsTrackChangesUI = { toggle, setEnabled,
 *   isEnabled, openSidebar, closeSidebar, renderSidebar, acceptAll,
 *   rejectAll }.
 */

(function () {
  function getState() { return window.state || null; }
  function isEnabled() {
    const s = getState();
    return !!(s && s.trackingEnabled);
  }
  function setEnabled(on) {
    const s = getState();
    if (!s) return false;
    s.trackingEnabled = !!on;
    document.dispatchEvent(new CustomEvent("docsTrackChangesToggle", { detail: { enabled: !!on } }));
    renderSidebar();
    applyEditorMarkers();
    return s.trackingEnabled;
  }
  function toggle() { return setEnabled(!isEnabled()); }

  function ensureSidebar() {
    let sb = document.getElementById("docsTrackChangesSidebar");
    if (sb) return sb;
    sb = document.createElement("div");
    sb.id = "docsTrackChangesSidebar";
    sb.className = "docs-track-changes-sidebar";
    sb.style.cssText = "position:fixed;top:0;right:340px;bottom:0;width:320px;background:#fefefe;border-left:1px solid #ddd;z-index:9997;display:none;flex-direction:column;font-family:Arial,sans-serif;font-size:13px;";
    sb.innerHTML = `
      <div style="padding:12px;border-bottom:1px solid #ddd;display:flex;align-items:center;gap:8px;">
        <strong>Track Changes</strong>
        <span id="docsTrackCount" style="background:#e0e0e0;border-radius:10px;padding:1px 8px;font-size:11px;">0</span>
        <label style="margin-left:auto;display:flex;align-items:center;gap:4px;font-size:12px;">
          <input type="checkbox" id="docsTrackToggle" /> On
        </label>
        <button id="docsTrackClose" style="background:transparent;border:0;font-size:18px;cursor:pointer;">×</button>
      </div>
      <div style="padding:8px;border-bottom:1px solid #ddd;display:flex;gap:6px;">
        <button id="docsTrackAcceptAll" style="flex:1;background:#0a8;color:#fff;border:0;border-radius:3px;padding:5px;cursor:pointer;">Accept All</button>
        <button id="docsTrackRejectAll" style="flex:1;background:#c00;color:#fff;border:0;border-radius:3px;padding:5px;cursor:pointer;">Reject All</button>
      </div>
      <div id="docsTrackList" style="flex:1;overflow-y:auto;padding:8px;"></div>
    `;
    document.body.appendChild(sb);
    sb.querySelector("#docsTrackClose").addEventListener("click", closeSidebar);
    sb.querySelector("#docsTrackToggle").addEventListener("change", (e) => { setEnabled(e.target.checked); });
    sb.querySelector("#docsTrackAcceptAll").addEventListener("click", () => { acceptAll(); });
    sb.querySelector("#docsTrackRejectAll").addEventListener("click", () => { rejectAll(); });
    return sb;
  }

  function openSidebar() {
    const sb = ensureSidebar();
    sb.style.display = "flex";
    sb.querySelector("#docsTrackToggle").checked = isEnabled();
    renderSidebar();
  }

  function closeSidebar() {
    const sb = document.getElementById("docsTrackChangesSidebar");
    if (sb) sb.style.display = "none";
  }

  function toggleSidebar() {
    const sb = ensureSidebar();
    sb.style.display = sb.style.display === "flex" ? "none" : "flex";
    if (sb.style.display === "flex") renderSidebar();
  }

  function renderSidebar() {
    const sb = ensureSidebar();
    const list = sb.querySelector("#docsTrackList");
    if (!list) return;
    list.innerHTML = "";
    const s = getState();
    if (!s) return;
    const revisions = s.revisions || [];
    const pending = revisions.filter((r) => r.accepted == null && r.rejected == null);
    sb.querySelector("#docsTrackCount").textContent = pending.length;
    for (const rev of revisions) {
      const row = document.createElement("div");
      row.style.cssText = "background:#fff;border:1px solid #ddd;border-left:3px solid " + (rev.accepted ? "#0a8" : rev.rejected ? "#c00" : "#fa0") + ";border-radius:4px;padding:6px;margin-bottom:6px;";
      const head = document.createElement("div");
      head.style.cssText = "font-size:11px;color:#666;margin-bottom:4px;";
      head.textContent = rev.author + " • " + rev.type + " • " + new Date(rev.timestamp).toLocaleTimeString();
      row.appendChild(head);
      const body = document.createElement("div");
      body.style.cssText = "font-size:12px;margin-bottom:4px;";
      if (rev.type === "insert") body.innerHTML = '<span style="color:#0a8;text-decoration:underline;">+' + escapeHtml(rev.after) + '</span>';
      else if (rev.type === "delete") body.innerHTML = '<span style="color:#c00;text-decoration:line-through;">-' + escapeHtml(rev.before) + '</span>';
      else if (rev.type === "replace") body.innerHTML = '<span style="color:#c00;text-decoration:line-through;">' + escapeHtml(rev.before) + '</span> <span style="color:#0a8;text-decoration:underline;">' + escapeHtml(rev.after) + '</span>';
      else body.textContent = "(" + rev.type + ")";
      row.appendChild(body);
      if (rev.accepted == null && rev.rejected == null) {
        const actions = document.createElement("div");
        actions.style.cssText = "display:flex;gap:4px;";
        const a = document.createElement("button");
        a.textContent = "Accept";
        a.style.cssText = "background:transparent;border:0;color:#0a8;cursor:pointer;font-size:11px;padding:2px 6px;";
        a.addEventListener("click", () => { if (window.DocsTrackChanges) window.DocsTrackChanges.acceptRevision(getState(), rev.id); renderSidebar(); });
        const r = document.createElement("button");
        r.textContent = "Reject";
        r.style.cssText = "background:transparent;border:0;color:#c00;cursor:pointer;font-size:11px;padding:2px 6px;";
        r.addEventListener("click", () => { if (window.DocsTrackChanges) window.DocsTrackChanges.rejectRevision(getState(), rev.id); renderSidebar(); });
        actions.appendChild(a);
        actions.appendChild(r);
        row.appendChild(actions);
      }
      list.appendChild(row);
    }
  }

  function acceptAll() {
    if (window.DocsTrackChanges) window.DocsTrackChanges.acceptAll(getState());
    renderSidebar();
  }
  function rejectAll() {
    if (window.DocsTrackChanges) window.DocsTrackChanges.rejectAll(getState());
    renderSidebar();
  }

  function escapeHtml(s) {
    return String(s || "").replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  function applyEditorMarkers() {
    const s = getState();
    if (!s || !s.revisions) return;
    if (!isEnabled()) {
      document.querySelectorAll(".track-change-insert, .track-change-delete").forEach((el) => {
        const txt = el.dataset.originalText || el.textContent;
        el.replaceWith(document.createTextNode(txt));
      });
      return;
    }
    const editor = document.querySelector(".editor") || document.querySelector("[contenteditable]");
    if (!editor) return;
    document.querySelectorAll(".track-change-insert, .track-change-delete").forEach((el) => {
      const txt = el.dataset.originalText || el.textContent;
      el.replaceWith(document.createTextNode(txt));
    });
    const pending = (s.revisions || []).filter((r) => r.accepted == null && r.rejected == null);
    for (const rev of pending) {
      if (rev.type === "insert" && rev.after) wrapOccurrence(editor, rev.after, "track-change-insert");
      if (rev.type === "delete" && rev.before) wrapOccurrence(editor, rev.before, "track-change-delete");
    }
  }

  function wrapOccurrence(root, text, className) {
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, null, false);
    let node;
    while ((node = walker.nextNode())) {
      const idx = node.textContent.indexOf(text);
      if (idx === -1) continue;
      const before = node.textContent.slice(0, idx);
      const match = node.textContent.slice(idx, idx + text.length);
      const after = node.textContent.slice(idx + text.length);
      const span = document.createElement("span");
      span.className = className;
      span.dataset.originalText = match;
      span.textContent = match;
      const beforeNode = document.createTextNode(before);
      const afterNode = document.createTextNode(after);
      const parent = node.parentNode;
      parent.insertBefore(beforeNode, node);
      parent.insertBefore(span, node);
      parent.insertBefore(afterNode, node);
      parent.removeChild(node);
      return;
    }
  }

  function attach() {
    document.addEventListener("docsPaginated", applyEditorMarkers);
    document.addEventListener("docsTrackChangesToggle", applyEditorMarkers);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.DocsTrackChangesUI = {
    toggle, setEnabled, isEnabled, openSidebar, closeSidebar, toggleSidebar,
    renderSidebar, acceptAll, rejectAll, applyEditorMarkers,
  };
})();
