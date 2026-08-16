"use strict";
/* GBCollabVersions — shared version-history panel for every collaboration app.
 *
 * Backed by the generic REST API at /api/collab/versions. Apps snapshot their
 * serialized content on save (deduped by content hash server-side) and open
 * this panel to browse/restore/name prior states. Restore is append-only:
 * the backend inserts a NEW current version rather than mutating history.
 *
 * Public API (window.GBCollabVersions):
 *   snapshot({ resourceType, resourceId, content, name })  — POST a snapshot
 *   open({ resourceType, resourceId, title, onRestore })    — open the panel
 *   close()                                                 — close it
 *
 * `onRestore(content)` is called with the restored document content so the
 * app can replace its editor and re-save (never called automatically).
 */
(function (window) {
  var CSS_ID = "gb-collab-versions-css";
  var panel = null;
  var listEl = null;
  var state = { resourceType: null, resourceId: null, title: "Version history", onRestore: null };

  function ensureCss() {
    if (document.getElementById(CSS_ID)) return;
    var css = [
      "#gb-versions-panel{position:fixed;top:0;right:0;bottom:0;width:360px;max-width:92vw;",
      "background:#0f172a;border-left:1px solid #334155;z-index:100000;display:flex;flex-direction:column;",
      "box-shadow:-8px 0 24px rgba(0,0,0,.4);transform:translateX(100%);transition:transform .2s ease;",
      "font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;}",
      "#gb-versions-panel.gbv-open{transform:translateX(0);}",
      "#gb-versions-panel .gbv-header{display:flex;align-items:center;gap:8px;padding:12px 14px;",
      "border-bottom:1px solid #334155;background:#1e293b;}",
      "#gb-versions-panel .gbv-title{flex:1;color:#f8fafc;font-size:14px;font-weight:600;",
      "overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}",
      "#gb-versions-panel .gbv-close{background:none;border:none;color:#94a3b8;font-size:20px;",
      "line-height:1;cursor:pointer;padding:0 4px;}",
      "#gb-versions-panel .gbv-close:hover{color:#f8fafc;}",
      "#gb-versions-panel .gbv-list{flex:1;overflow-y:auto;padding:12px 14px;display:flex;",
      "flex-direction:column;gap:8px;}",
      "#gb-versions-panel .gbv-empty,#gb-versions-panel .gbv-loading,#gb-versions-panel .gbv-error{",
      "color:#94a3b8;font-size:13px;text-align:center;padding:24px 8px;}",
      "#gb-versions-panel .gbv-error{color:#f87171;}",
      "#gb-versions-panel .gbv-item{background:#1e293b;border:1px solid #334155;border-radius:8px;padding:10px 12px;}",
      "#gb-versions-panel .gbv-item.gbv-current{border-color:#3b82f6;}",
      "#gb-versions-panel .gbv-name{color:#f8fafc;font-weight:600;font-size:13px;margin-bottom:2px;}",
      "#gb-versions-panel .gbv-meta{color:#94a3b8;font-size:11.5px;line-height:1.5;}",
      "#gb-versions-panel .gbv-who{color:#e2e8f0;}",
      "#gb-versions-panel .gbv-hash{color:#64748b;font-size:10px;font-family:monospace;display:block;margin-top:2px;}",
      "#gb-versions-panel .gbv-actions{display:flex;gap:6px;margin-top:8px;}",
      "#gb-versions-panel .gbv-btn{background:#334155;border:none;color:#e2e8f0;border-radius:5px;",
      "padding:4px 10px;font-size:12px;cursor:pointer;}",
      "#gb-versions-panel .gbv-btn:hover{background:#475569;}",
      "#gb-versions-panel .gbv-btn.gbv-primary{background:#3b82f6;color:#fff;}",
      "#gb-versions-panel .gbv-btn.gbv-primary:hover{background:#2563eb;}"
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

  function sizeFmt(bytes) {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  }

  function build() {
    ensureCss();
    if (panel && panel.parentNode) return panel;
    panel = document.createElement("div");
    panel.id = "gb-versions-panel";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-label", "Version history");
    panel.innerHTML =
      '<div class="gbv-header">' +
      '<span class="gbv-title">' + esc(state.title) + '</span>' +
      '<button class="gbv-close" title="Close">×</button>' +
      '</div>' +
      '<div class="gbv-list"></div>';
    document.body.appendChild(panel);
    listEl = panel.querySelector(".gbv-list");
    panel.querySelector(".gbv-close").addEventListener("click", close);
    return panel;
  }

  function load() {
    listEl.innerHTML = '<div class="gbv-loading">Loading…</div>';
    api("/api/collab/versions?resource_type=" + encodeURIComponent(state.resourceType) +
        "&resource_id=" + encodeURIComponent(state.resourceId) + "&limit=100")
      .then(function (items) {
        if (!items.length) {
          listEl.innerHTML = '<div class="gbv-empty">No versions yet — the first save creates one.</div>';
          return;
        }
        var html = items.map(function (v, i) {
          var isCurrent = i === 0;
          var label = v.name ? esc(v.name) : ("Version " + (items.length - i));
          var currentTag = isCurrent ? '<span style="color:#3b82f6;font-size:11px;font-weight:600;"> (current)</span>' : "";
          return '<div class="gbv-item' + (isCurrent ? " gbv-current" : "") + '">' +
            '<div class="gbv-name">' + label + currentTag + '</div>' +
            '<div class="gbv-meta"><span class="gbv-who">' + esc(v.actor_name || v.actor_id) + '</span> · ' +
            esc(timeFmt(v.created_at)) + ' · ' + sizeFmt(v.size) + '</div>' +
            '<span class="gbv-hash">#' + esc(v.content_hash.slice(0, 12)) + '</span>' +
            '<div class="gbv-actions">' +
            '<button class="gbv-btn" data-act="name" data-id="' + esc(v.id) + '">Name…</button>' +
            (isCurrent ? "" : '<button class="gbv-btn gbv-primary" data-act="restore" data-id="' + esc(v.id) + '">Restore</button>') +
            '</div>' +
            '</div>';
        }).join("");
        listEl.innerHTML = html;
        listEl.querySelectorAll("[data-act]").forEach(function (btn) {
          btn.addEventListener("click", function () {
            var act = btn.getAttribute("data-act");
            var id = btn.getAttribute("data-id");
            if (act === "restore") doRestore(id);
            else if (act === "name") doName(id, btn);
          });
        });
      })
      .catch(function (e) {
        listEl.innerHTML = '<div class="gbv-error">' + esc(e.message) + '</div>';
      });
  }

  function doRestore(id) {
    if (!window.confirm("Restore this version? A new current version will be created — nothing is lost.")) return;
    api("/api/collab/versions/" + encodeURIComponent(id))
      .then(function (detail) {
        if (typeof state.onRestore === "function") state.onRestore(detail.content);
        load();
      })
      .catch(function (e) { window.alert("Restore failed: " + e.message); });
  }

  function doName(id, btn) {
    var name = window.prompt("Name this version (e.g. \"v2 — approved\"):", "");
    if (name === null) return;
    name = name.trim();
    if (!name) return;
    api("/api/collab/versions/" + encodeURIComponent(id) + "/name", {
      method: "POST",
      body: JSON.stringify({ name: name })
    })
      .then(function () { load(); })
      .catch(function (e) { window.alert("Rename failed: " + e.message); });
  }

  function open(opts) {
    state.resourceType = opts.resourceType;
    state.resourceId = opts.resourceId;
    state.title = opts.title || "Version history";
    state.onRestore = opts.onRestore || null;
    var p = build();
    p.querySelector(".gbv-title").textContent = state.title;
    p.classList.add("gbv-open");
    load();
  }

  function close() {
    if (panel) panel.classList.remove("gbv-open");
  }

  function snapshot(opts) {
    return api("/api/collab/versions", {
      method: "POST",
      body: JSON.stringify({
        resource_type: opts.resourceType,
        resource_id: opts.resourceId,
        content: opts.content,
        name: opts.name || ""
      })
    });
  }

  window.GBCollabVersions = { open: open, close: close, snapshot: snapshot };
})(window);
