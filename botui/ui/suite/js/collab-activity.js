"use strict";
/* GBCollabActivity — shared audit-trail panel for every collaboration app.
 *
 * Backed by the generic REST API at /api/activity. It renders a timeline of
 * who did what (edit/comment/share/resolve/…) on a resource, newest first,
 * with "Load more" cursor pagination. Apps only need a resource_type +
 * resource_id to address a document, deck, sheet or task.
 *
 * Public API (window.GBCollabActivity):
 *   open({ resourceType, resourceId, title })  — open the panel
 *   close()                                    — close it
 *   record({ resourceType, resourceId, action, payload })  — POST an event
 *      (used by autosave/share/restore hooks to write the audit trail)
 *
 * Auth: reads the JWT from localStorage/sessionStorage (gb-access-token) and
 * sends it as a Bearer header; every endpoint is authenticated server-side.
 */
(function (window) {
  var CSS_ID = "gb-collab-activity-css";
  var PAGE_SIZE = 50;
  var panel = null;
  var listEl = null;
  var moreBtn = null;
  var state = { resourceType: null, resourceId: null, title: "Activity", before: null, done: false };

  function ensureCss() {
    if (document.getElementById(CSS_ID)) return;
    var css = [
      "#gb-activity-panel{position:fixed;top:0;right:0;bottom:0;width:360px;max-width:92vw;",
      "background:#0f172a;border-left:1px solid #334155;z-index:100000;display:flex;flex-direction:column;",
      "box-shadow:-8px 0 24px rgba(0,0,0,.4);transform:translateX(100%);transition:transform .2s ease;",
      "font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;}",
      "#gb-activity-panel.gba-open{transform:translateX(0);}",
      "#gb-activity-panel .gba-header{display:flex;align-items:center;gap:8px;padding:12px 14px;",
      "border-bottom:1px solid #334155;background:#1e293b;}",
      "#gb-activity-panel .gba-title{flex:1;color:#f8fafc;font-size:14px;font-weight:600;",
      "overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}",
      "#gb-activity-panel .gba-close{background:none;border:none;color:#94a3b8;font-size:20px;",
      "line-height:1;cursor:pointer;padding:0 4px;}",
      "#gb-activity-panel .gba-close:hover{color:#f8fafc;}",
      "#gb-activity-panel .gba-list{flex:1;overflow-y:auto;padding:12px 14px;display:flex;",
      "flex-direction:column;gap:2px;}",
      "#gb-activity-panel .gba-empty,#gb-activity-panel .gba-loading,#gb-activity-panel .gba-error{",
      "color:#94a3b8;font-size:13px;text-align:center;padding:24px 8px;}",
      "#gb-activity-panel .gba-error{color:#f87171;}",
      "#gb-activity-panel .gba-row{display:flex;align-items:flex-start;gap:10px;padding:8px 6px;",
      "border-radius:6px;}",
      "#gb-activity-panel .gba-row:hover{background:#1e293b;}",
      "#gb-activity-panel .gba-dot{width:8px;height:8px;border-radius:50%;margin-top:5px;flex-shrink:0;}",
      "#gb-activity-panel .gba-line{color:#94a3b8;font-size:12.5px;line-height:1.5;min-width:0;flex:1;}",
      "#gb-activity-panel .gba-who{color:#f8fafc;font-weight:600;}",
      "#gb-activity-panel .gba-action{color:#93c5fd;}",
      "#gb-activity-panel .gba-when{color:#64748b;font-size:11px;display:block;margin-top:1px;}",
      "#gb-activity-panel .gba-more{width:100%;background:#1e293b;border:1px solid #334155;",
      "color:#93c5fd;border-radius:6px;padding:8px;margin:8px 0;font-size:12.5px;cursor:pointer;}",
      "#gb-activity-panel .gba-more:hover{background:#334155;}"
    ].join("");
    var style = document.createElement("style");
    style.id = CSS_ID;
    style.textContent = css;
    document.head.appendChild(style);
  }

  function esc(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }

  function token() {
    try {
      return localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
    } catch (_) { return ""; }
  }

  function api(endpoint, options) {
    var headers = { "Content-Type": "application/json" };
    var t = token();
    if (t) headers["Authorization"] = "Bearer " + t;
    return fetch(endpoint, Object.assign({ headers: headers }, options || {}))
      .then(function (res) {
        return res.json().catch(function () { return { error: res.statusText }; })
          .then(function (body) {
            if (!res.ok) throw new Error(body.error || "Request failed (" + res.status + ")");
            return body;
          });
      });
  }

  function timeFmt(iso) {
    try { return new Date(iso).toLocaleString(); } catch (_) { return iso; }
  }

  var ACTION_COLORS = {
    create: "#3b82f6", edit: "#3b82f6", comment: "#8b5cf6", delete: "#ef4444",
    resolve: "#10b981", reopen: "#f59e0b", reaction: "#ec4899", share: "#06b6d4",
    restore: "#84cc16", transfer: "#f97316"
  };

  function actionLabel(action) {
    var labels = {
      create: "created", edit: "edited", comment: "commented on", delete: "deleted",
      resolve: "resolved a comment on", reopen: "reopened a comment on",
      reaction: "reacted to", share: "shared", restore: "restored", transfer: "transferred"
    };
    return labels[action] || action;
  }

  function rowHtml(item) {
    var color = ACTION_COLORS[item.action] || "#64748b";
    return '<div class="gba-row">' +
      '<span class="gba-dot" style="background:' + color + '"></span>' +
      '<div class="gba-line"><span class="gba-who">' + esc(item.actor_name || item.actor_id) + '</span> ' +
      '<span class="gba-action">' + esc(actionLabel(item.action)) + '</span>' +
      '<span class="gba-when">' + esc(timeFmt(item.created_at)) + '</span></div>' +
      '</div>';
  }

  function build() {
    ensureCss();
    if (panel && panel.parentNode) return panel;
    panel = document.createElement("div");
    panel.id = "gb-activity-panel";
    panel.innerHTML =
      '<div class="gba-header">' +
      '<span class="gba-title">' + esc(state.title) + '</span>' +
      '<button class="gba-close" title="Close">×</button>' +
      '</div>' +
      '<div class="gba-list"></div>' +
      '<div style="padding:0 14px 14px;"><button class="gba-more" style="display:none;">Load more</button></div>';
    document.body.appendChild(panel);
    listEl = panel.querySelector(".gba-list");
    moreBtn = panel.querySelector(".gba-more");
    panel.querySelector(".gba-close").addEventListener("click", close);
    moreBtn.addEventListener("click", function () { load(true); });
    return panel;
  }

  function load(append) {
    var q = "resource_type=" + encodeURIComponent(state.resourceType) +
      "&resource_id=" + encodeURIComponent(state.resourceId) +
      "&limit=" + PAGE_SIZE;
    if (state.before) q += "&before=" + encodeURIComponent(state.before);
    if (!append) {
      listEl.innerHTML = '<div class="gba-loading">Loading…</div>';
      moreBtn.style.display = "none";
    }
    api("/api/activity?" + q)
      .then(function (items) {
        if (!append) listEl.innerHTML = "";
        if (!items.length) {
          state.done = true;
          if (!append) listEl.innerHTML = '<div class="gba-empty">No activity yet</div>';
          moreBtn.style.display = "none";
          return;
        }
        var html = items.map(rowHtml).join("");
        listEl.insertAdjacentHTML(append ? "beforeend" : "afterbegin", html);
        state.before = items[items.length - 1].created_at;
        state.done = items.length < PAGE_SIZE;
        moreBtn.style.display = state.done ? "none" : "block";
      })
      .catch(function (e) {
        listEl.innerHTML = '<div class="gba-error">' + esc(e.message) + '</div>';
      });
  }

  function open(opts) {
    state.resourceType = opts.resourceType;
    state.resourceId = opts.resourceId;
    state.title = opts.title || "Activity";
    state.before = null;
    state.done = false;
    var p = build();
    p.querySelector(".gba-title").textContent = state.title;
    p.classList.add("gba-open");
    load(false);
  }

  function close() {
    if (panel) panel.classList.remove("gba-open");
  }

  function record(opts) {
    return api("/api/activity", {
      method: "POST",
      body: JSON.stringify({
        resource_type: opts.resourceType,
        resource_id: opts.resourceId,
        action: opts.action,
        payload: opts.payload || {}
      })
    });
  }

  window.GBCollabActivity = { open: open, close: close, record: record };
})(window);
