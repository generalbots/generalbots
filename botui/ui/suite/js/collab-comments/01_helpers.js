/* GBCollabComments — shared threaded-comments panel for every collaboration app
 * (drive, sheet, docs, tasks, calendar, slides, plan...). Backed by the generic
 * REST API at /api/collab/* (threaded comments, @mentions, emoji reactions,
 * presence). This file is app-agnostic: it ships its own styles and only needs
 * a resource_type + resource_id to address a document, cell, task or event.
 *
 * Public API (window.GBCollabComments):
 *   open({ resourceType, resourceId, title, notify })  — open the panel
 *   close()                                            — close + stop heartbeat
 *   post()                                             — submit the composer
 *   mountButton(container, { resourceType, resourceId, label, notify })
 *                                                      — inject a button that opens
 *                                                        the panel for a resource
 *
 * Auth: reads the JWT from localStorage/sessionStorage (gb-access-token) and
 * sends it as a Bearer header. All endpoints are authenticated server-side.
 */
(function (window, document) {
  "use strict";
  // #2095 — user-visible strings resolve through the shared catalogue
  // (window.gbT) with the authored English text as fallback.
  function T(key, fallback, params) {
    return window.gbT ? window.gbT(key, fallback, params) : fallback;
  }

  var N = (document.__gb_mod_collab_comments = {});
  N.s = { panel: null, listEl: null, inputEl: null, presenceEl: null,
    heartbeatTimer: null, typingSent: false, badgeRefreshFn: null, mentionBox: null,
    mentionCandidates: [], presenceUsers: [], commentAuthors: [], mentionItems: [],
    mentionIndex: -1, activeMention: null };
  var CSS_ID = "gb-collab-comments-css";
  var HEARTBEAT_MS = 30000;
  var state = { resourceType: null, resourceId: null, title: "Comments", notify: null, includeChildren: false, filter: "open" };

  function ensureCss() {
    if (document.getElementById(CSS_ID)) return;
    var css = [
      "#gb-comments-panel{position:fixed;top:0;right:0;bottom:0;width:340px;max-width:92vw;",
      "background:#0f172a;border-left:1px solid #334155;z-index:100000;display:flex;flex-direction:column;",
      "box-shadow:-8px 0 24px rgba(0,0,0,.4);transform:translateX(100%);transition:transform .2s ease;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;}",
      "#gb-comments-panel.gbc-open{transform:translateX(0);}",
      "#gb-comments-panel .gbc-header{display:flex;align-items:center;gap:8px;padding:12px 14px;border-bottom:1px solid #334155;background:#1e293b;}",
      "#gb-comments-panel .gbc-title{flex:1;color:#f8fafc;font-size:14px;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}",
      "#gb-comments-panel .gbc-presence{display:none;font-size:11px;color:#94a3b8;max-width:120px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}",
      "#gb-comments-panel .gbc-close{background:none;border:none;color:#94a3b8;font-size:20px;line-height:1;cursor:pointer;padding:0 4px;}",
      "#gb-comments-panel .gbc-close:hover{color:#f8fafc;}",
      "#gb-comments-panel .gbc-list{flex:1;overflow-y:auto;padding:12px 14px;display:flex;flex-direction:column;gap:12px;}",
      "#gb-comments-panel .gbc-empty,#gb-comments-panel .gbc-loading,#gb-comments-panel .gbc-error{color:#94a3b8;font-size:13px;text-align:center;padding:24px 8px;}",
      "#gb-comments-panel .gbc-error{color:#f87171;}",
      "#gb-comments-panel .gbc-input-row{display:flex;gap:8px;padding:10px 14px;border-top:1px solid #334155;background:#1e293b;}",
      "#gb-comments-panel .gbc-input{flex:1;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;padding:8px 10px;font-size:13px;font-family:inherit;}",
      "#gb-comments-panel .gbc-send{background:#3b82f6;color:#fff;border:none;border-radius:6px;padding:8px 14px;font-size:13px;font-weight:600;cursor:pointer;}",
      "#gb-comments-panel .gbc-send:hover{background:#2563eb;}",
      "#gb-comments-panel .gbc-item{background:#1e293b;border:1px solid #334155;border-radius:8px;padding:10px 12px;}",
      "#gb-comments-panel .gbc-item.gbc-reply{margin-left:18px;background:#16202f;}",
      "#gb-comments-panel .gbc-meta{display:flex;align-items:baseline;gap:8px;margin-bottom:4px;}",
      "#gb-comments-panel .gbc-author{color:#f8fafc;font-weight:600;font-size:12.5px;}",
      "#gb-comments-panel .gbc-time{color:#64748b;font-size:11px;}",
      "#gb-comments-panel .gbc-body{color:#e2e8f0;font-size:13px;line-height:1.5;white-space:pre-wrap;word-break:break-word;}",
      "#gb-comments-panel .gbc-body .gbc-mention{color:#60a5fa;font-weight:600;background:rgba(59,130,246,.15);border-radius:3px;padding:0 2px;}",
      "#gb-comments-panel .gbc-actions{display:flex;align-items:center;gap:4px;flex-wrap:wrap;margin-top:8px;}",
      "#gb-comments-panel .gbc-react{font-size:14px;padding:2px 6px;border-radius:4px;cursor:pointer;user-select:none;}",
      "#gb-comments-panel .gbc-react:hover{background:#334155;}",
      "#gb-comments-panel .gbc-chip{display:inline-flex;align-items:center;gap:2px;font-size:12px;background:#334155;border-radius:999px;padding:2px 8px;cursor:pointer;}",
      "#gb-comments-panel .gbc-chip:hover{background:#475569;}",
      "#gb-comments-panel .gbc-reply-btn,#gb-comments-panel .gbc-del{font-size:12px;color:#94a3b8;cursor:pointer;padding:2px 4px;border-radius:4px;}",
      "#gb-comments-panel .gbc-reply-btn:hover{color:#60a5fa;background:#1e293b;}",
      "#gb-comments-panel .gbc-del:hover{color:#f87171;background:#1e293b;}",
      "#gb-comments-panel .gbc-replies{display:flex;flex-direction:column;gap:8px;margin-top:10px;}",
      "#gb-comments-panel .gbc-filter{display:flex;gap:4px;padding:8px 14px;border-bottom:1px solid #334155;background:#0f172a;}",
      "#gb-comments-panel .gbc-filter-btn{background:none;border:1px solid #334155;color:#94a3b8;border-radius:999px;padding:3px 10px;font-size:11px;cursor:pointer;}",
      "#gb-comments-panel .gbc-filter-btn.active{background:#3b82f6;border-color:#3b82f6;color:#fff;}",
      "#gb-comments-panel .gbc-resolved-chip{font-size:10px;color:#10b981;background:rgba(16,185,129,.15);border-radius:3px;padding:0 5px;line-height:16px;font-weight:600;}",
      "#gb-comments-panel .gbc-item.gbc-resolved .gbc-body{opacity:.55;}",
      "#gb-comments-panel .gbc-resolve-btn{font-size:12px;color:#94a3b8;cursor:pointer;padding:2px 4px;border-radius:4px;}",
      "#gb-comments-panel .gbc-resolve-btn:hover{color:#10b981;background:#1e293b;}"
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

  function req(endpoint, options) {
    var headers = { "Content-Type": "application/json" };
    var t = token();
    if (t) headers["Authorization"] = "Bearer " + t;
    return fetch("/api/collab" + endpoint, Object.assign({ headers: headers }, options || {}))
      .then(function (res) {
        return res.json().catch(function () { return { error: res.statusText }; })
          .then(function (body) {
            if (!res.ok) throw new Error(body.error ||
              T("comments-request-failed", "Request failed (" + res.status + ")", { status: res.status }));
            return body;
          });
      });
  }

  function renderBody(body) {
    return esc(body).replace(/@([\w.@-]+)/g, '<span class="gbc-mention">@$1</span>');
  }

  function timeFmt(iso) {
    try { return gbFormatDateTime(new Date(iso)); } catch (_) { return iso; }
  }

  function chips(comment) {
    if (!comment.reactions || !comment.reactions.length) return "";
    var counts = {};
    comment.reactions.forEach(function (r) { counts[r.emoji] = (counts[r.emoji] || 0) + 1; });
    return Object.keys(counts).map(function (e) {
      return '<span class="gbc-chip" data-comment="' + esc(comment.id) + '" data-emoji="' + esc(e) + '">' + esc(e) + " " + counts[e] + "</span>";
    }).join("");
  }

  // When the panel aggregates child resources (includeChildren), surface the
  // anchor (e.g. cell A1) of a comment whose resource is nested under the
  // panel's resource, so users can tell where each comment belongs.
  function anchorBadge(comment) {
    if (!state.includeChildren || !comment.resource_type || comment.resource_type === state.resourceType) return "";
    var suffix = comment.resource_id || "";
    if (state.resourceId && suffix.indexOf(state.resourceId + ":") === 0) {
      suffix = suffix.slice(state.resourceId.length + 1);
    }
    var parts = suffix.split(":");
    var cell = parts[parts.length - 1];
    if (!cell) return "";
    return '<span class="gbc-anchor" style="font-size:10px;color:#60a5fa;background:rgba(59,130,246,.12);border-radius:3px;padding:0 5px;line-height:16px;font-weight:600;">' + esc(cell) + "</span>";
  }

  function node(comment, isReply) {
    var n = document.createElement("div");
    n.className = "gbc-item" + (isReply ? " gbc-reply" : "") + (comment.resolved ? " gbc-resolved" : "");
    var resolvedChip = comment.resolved
      ? '<span class="gbc-resolved-chip">✓ ' +
        esc(T("comments-resolved-chip", "Resolved")) + '</span>'
      : "";
    var resolveBtn = isReply ? "" :
      '<span class="gbc-resolve-btn" data-comment="' + esc(comment.id) + '" data-resolved="' + (comment.resolved ? "1" : "0") + '">' +
        (comment.resolved
          ? T("comments-reopen", "Reopen")
          : T("comments-resolve", "Resolve")) + "</span>";
    n.innerHTML =
      '<div class="gbc-meta"><span class="gbc-author">' + esc(comment.author_name || comment.author_id) + "</span>" +
      anchorBadge(comment) + resolvedChip +
      '<span class="gbc-time">' + timeFmt(comment.created_at) + "</span></div>" +
      '<div class="gbc-body">' + renderBody(comment.body) + "</div>" +
      '<div class="gbc-actions">' +
        ["👍", "❤️", "😄", "🎉"].map(function (e) {
          return '<span class="gbc-react" data-comment="' + esc(comment.id) + '" data-emoji="' + e + '">' + e + "</span>";
        }).join("") +
        chips(comment) +
        '<span class="gbc-reply-btn" data-comment="' + esc(comment.id) + '">' +
          esc(T("action-reply", "Reply")) + '</span>' +
        resolveBtn +
        '<span class="gbc-del" data-comment="' + esc(comment.id) + '">' +
          esc(T("action-delete", "Delete")) + '</span>' +
      "</div>";
    if (comment.replies && comment.replies.length) {
      var wrap = document.createElement("div");
      wrap.className = "gbc-replies";
      comment.replies.forEach(function (r) { wrap.appendChild(node(r, true)); });
      n.appendChild(wrap);
    }
    return n;
  }


  N.T = T;
  N.CSS_ID = CSS_ID;
  N.HEARTBEAT_MS = HEARTBEAT_MS;
  N.state = state;
  N.ensureCss = ensureCss;
  N.esc = esc;
  N.token = token;
  N.req = req;
  N.renderBody = renderBody;
  N.timeFmt = timeFmt;
  N.chips = chips;
  N.anchorBadge = anchorBadge;
  N.node = node;
})(window, document);
