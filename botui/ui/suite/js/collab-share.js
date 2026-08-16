"use strict";
/* GBCollabShare — shared sharing dialog for Docs, Slides, Drive, …
 *
 * Backed by the generic REST API at /api/collab/permissions and
 * /api/collab/links. Replaces the previous per-app Share stubs with a real
 * access-control dialog: add people (viewer/commenter/editor), remove access,
 * share a public link with a role, and transfer ownership.
 *
 * Public API (window.GBCollabShare):
 *   open({ resourceType, resourceId, title })  — open the dialog
 *   close()                                     — close it
 */
(function (window) {
  var CSS_ID = "gb-collab-share-css";
  var overlay = null;
  var state = { resourceType: null, resourceId: null, title: "Share" };

  function ensureCss() {
    if (document.getElementById(CSS_ID)) return;
    var css = [
      "#gb-share-overlay{position:fixed;inset:0;background:rgba(0,0,0,.55);z-index:100001;",
      "display:none;align-items:center;justify-content:center;}",
      "#gb-share-overlay.gbs-open{display:flex;}",
      "#gb-share-dialog{width:440px;max-width:94vw;background:#0f172a;border:1px solid #334155;",
      "border-radius:10px;box-shadow:0 12px 40px rgba(0,0,0,.5);display:flex;flex-direction:column;",
      "max-height:88vh;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;}",
      "#gb-share-dialog .gbs-header{display:flex;align-items:center;gap:8px;padding:14px 16px;",
      "border-bottom:1px solid #334155;background:#1e293b;border-radius:10px 10px 0 0;}",
      "#gb-share-dialog .gbs-title{flex:1;color:#f8fafc;font-size:15px;font-weight:600;}",
      "#gb-share-dialog .gbs-close{background:none;border:none;color:#94a3b8;font-size:20px;",
      "line-height:1;cursor:pointer;padding:0 4px;}",
      "#gb-share-dialog .gbs-body{flex:1;overflow-y:auto;padding:14px 16px;display:flex;",
      "flex-direction:column;gap:12px;}",
      "#gb-share-dialog .gbs-row{display:flex;gap:6px;}",
      "#gb-share-dialog input,#gb-share-dialog select{flex:1;background:#0f172a;",
      "border:1px solid #334155;border-radius:6px;color:#f8fafc;padding:7px 10px;font-size:13px;}",
      "#gb-share-dialog .gbs-btn{background:#3b82f6;color:#fff;border:none;border-radius:6px;",
      "padding:7px 12px;font-size:13px;font-weight:600;cursor:pointer;white-space:nowrap;}",
      "#gb-share-dialog .gbs-btn:hover{background:#2563eb;}",
      "#gb-share-dialog .gbs-btn.gbs-ghost{background:#1e293b;border:1px solid #334155;color:#e2e8f0;}",
      "#gb-share-dialog .gbs-btn.gbs-ghost:hover{background:#334155;}",
      "#gb-share-dialog .gbs-section{color:#94a3b8;font-size:11px;font-weight:600;",
      "text-transform:uppercase;letter-spacing:.04em;}",
      "#gb-share-dialog .gbs-item{display:flex;align-items:center;gap:8px;padding:6px 0;",
      "border-bottom:1px solid #1e293b;}",
      "#gb-share-dialog .gbs-item-name{flex:1;color:#f8fafc;font-size:13px;overflow:hidden;",
      "text-overflow:ellipsis;white-space:nowrap;}",
      "#gb-share-dialog .gbs-item-role{color:#93c5fd;font-size:12px;}",
      "#gb-share-dialog .gbs-item-remove{background:none;border:none;color:#94a3b8;font-size:16px;",
      "cursor:pointer;padding:0 4px;}",
      "#gb-share-dialog .gbs-item-remove:hover{color:#f87171;}",
      "#gb-share-dialog .gbs-link{display:flex;align-items:center;gap:8px;background:#1e293b;",
      "border:1px solid #334155;border-radius:6px;padding:6px 8px;}",
      "#gb-share-dialog .gbs-link-text{flex:1;color:#e2e8f0;font-size:12px;font-family:monospace;",
      "overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}",
      "#gb-share-dialog .gbs-error{color:#f87171;font-size:12px;}",
      "#gb-share-dialog .gbs-hint{color:#64748b;font-size:12px;}"
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

  function build() {
    ensureCss();
    if (overlay && overlay.parentNode) return overlay;
    overlay = document.createElement("div");
    overlay.id = "gb-share-overlay";
    overlay.innerHTML =
      '<div id="gb-share-dialog" role="dialog" aria-modal="true" aria-label="Share">' +
        '<div class="gbs-header">' +
          '<span class="gbs-title">Share</span>' +
          '<button class="gbs-close" title="Close">×</button>' +
        '</div>' +
        '<div class="gbs-body">' +
          '<div class="gbs-section">Invite people</div>' +
          '<div class="gbs-row">' +
            '<input id="gbs-email" type="email" placeholder="name@example.com" />' +
            '<select id="gbs-role">' +
              '<option value="editor">Editor</option>' +
              '<option value="commenter">Commenter</option>' +
              '<option value="viewer">Viewer</option>' +
            '</select>' +
            '<button class="gbs-btn" id="gbs-add">Add</button>' +
          '</div>' +
          '<div id="gbs-list"></div>' +
          '<div class="gbs-section">Anyone with the link</div>' +
          '<div class="gbs-row">' +
            '<select id="gbs-link-role">' +
              '<option value="viewer">Viewer</option>' +
              '<option value="commenter">Commenter</option>' +
              '<option value="editor">Editor</option>' +
            '</select>' +
            '<button class="gbs-btn gbs-ghost" id="gbs-create-link">Create link</button>' +
          '</div>' +
          '<div id="gbs-links"></div>' +
          '<div class="gbs-section">Transfer ownership</div>' +
          '<div class="gbs-row">' +
            '<input id="gbs-new-owner" type="email" placeholder="new owner email" />' +
            '<button class="gbs-btn gbs-ghost" id="gbs-transfer">Transfer</button>' +
          '</div>' +
          '<div class="gbs-error" id="gbs-error" style="display:none;"></div>' +
        '</div>' +
      '</div>';
    document.body.appendChild(overlay);
    overlay.querySelector(".gbs-close").addEventListener("click", close);
    overlay.querySelector("#gbs-add").addEventListener("click", addGrant);
    overlay.querySelector("#gbs-create-link").addEventListener("click", createLink);
    overlay.querySelector("#gbs-transfer").addEventListener("click", transfer);
    overlay.addEventListener("click", function (e) { if (e.target === overlay) close(); });
    return overlay;
  }

  function err(msg) {
    var el = document.getElementById("gbs-error");
    if (!el) return;
    el.textContent = msg;
    el.style.display = msg ? "block" : "none";
  }

  function refresh() {
    err("");
    return api("/api/collab/permissions?resource_type=" + encodeURIComponent(state.resourceType) +
      "&resource_id=" + encodeURIComponent(state.resourceId))
      .then(function (grants) {
        var list = document.getElementById("gbs-list");
        list.innerHTML = grants.map(function (g) {
          var isOwner = g.role === "owner";
          return '<div class="gbs-item">' +
            '<span class="gbs-item-name">' + esc(g.grantee_id) + '</span>' +
            '<span class="gbs-item-role">' + esc(g.role) + (isOwner ? " · owner" : "") + '</span>' +
            (isOwner ? "" : '<button class="gbs-item-remove" data-gtype="' + esc(g.grantee_type) +
              '" data-gid="' + esc(g.grantee_id) + '" title="Remove">×</button>') +
            '</div>';
        }).join("") || '<div class="gbs-hint">No people added yet.</div>';
        list.querySelectorAll(".gbs-item-remove").forEach(function (btn) {
          btn.addEventListener("click", function () {
            revoke(btn.getAttribute("data-gtype"), btn.getAttribute("data-gid"));
          });
        });
        return api("/api/collab/links?resource_type=" + encodeURIComponent(state.resourceType) +
          "&resource_id=" + encodeURIComponent(state.resourceId));
      })
      .then(function (links) {
        var list = document.getElementById("gbs-links");
        list.innerHTML = links.map(function (l) {
          var url = window.location.origin + "/suite/docs/" + encodeURIComponent(state.resourceId) +
            "?link=" + l.token;
          return '<div class="gbs-link">' +
            '<span class="gbs-link-text">' + esc(url) + '</span>' +
            '<button class="gbs-btn gbs-ghost" data-copy="' + esc(url) + '">Copy</button>' +
            '<button class="gbs-item-remove" data-revoke="' + esc(l.token) + '" title="Revoke">×</button>' +
            '</div>';
        }).join("") || '<div class="gbs-hint">No public link yet.</div>';
        list.querySelectorAll("[data-copy]").forEach(function (btn) {
          btn.addEventListener("click", function () {
            try {
              navigator.clipboard.writeText(btn.getAttribute("data-copy"));
              btn.textContent = "Copied";
            } catch (_) {}
          });
        });
        list.querySelectorAll("[data-revoke]").forEach(function (btn) {
          btn.addEventListener("click", function () { revokeLink(btn.getAttribute("data-revoke")); });
        });
      })
      .catch(function (e) { err(e.message); });
  }

  function addGrant() {
    var email = document.getElementById("gbs-email").value.trim();
    var role = document.getElementById("gbs-role").value;
    if (!email) { err("Enter an email"); return; }
    api("/api/collab/permissions", {
      method: "POST",
      body: JSON.stringify({
        resource_type: state.resourceType,
        resource_id: state.resourceId,
        grantee_type: "user",
        grantee_id: email,
        role: role
      })
    })
      .then(function () {
        document.getElementById("gbs-email").value = "";
        refresh();
      })
      .catch(function (e) { err(e.message); });
  }

  function revoke(gtype, gid) {
    api("/api/collab/permissions", {
      method: "DELETE",
      body: JSON.stringify({
        resource_type: state.resourceType,
        resource_id: state.resourceId,
        grantee_type: gtype,
        grantee_id: gid
      })
    })
      .then(refresh)
      .catch(function (e) { err(e.message); });
  }

  function createLink() {
    var role = document.getElementById("gbs-link-role").value;
    api("/api/collab/links", {
      method: "POST",
      body: JSON.stringify({
        resource_type: state.resourceType,
        resource_id: state.resourceId,
        role: role
      })
    })
      .then(refresh)
      .catch(function (e) { err(e.message); });
  }

  function revokeLink(tok) {
    api("/api/collab/links/" + encodeURIComponent(tok), { method: "DELETE" })
      .then(refresh)
      .catch(function (e) { err(e.message); });
  }

  function transfer() {
    var email = document.getElementById("gbs-new-owner").value.trim();
    if (!email) { err("Enter the new owner's email"); return; }
    api("/api/collab/permissions/transfer", {
      method: "POST",
      body: JSON.stringify({
        resource_type: state.resourceType,
        resource_id: state.resourceId,
        new_owner_id: email
      })
    })
      .then(function () {
        document.getElementById("gbs-new-owner").value = "";
        refresh();
      })
      .catch(function (e) { err(e.message); });
  }

  function open(opts) {
    state.resourceType = opts.resourceType;
    state.resourceId = opts.resourceId;
    state.title = opts.title || "Share";
    var ov = build();
    ov.querySelector(".gbs-title").textContent = state.title;
    ov.classList.add("gbs-open");
    refresh();
  }

  function close() {
    if (overlay) overlay.classList.remove("gbs-open");
  }

  window.GBCollabShare = { open: open, close: close };
})(window);
