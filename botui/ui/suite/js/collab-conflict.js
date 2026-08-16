"use strict";
/* GBCollabConflict — shared conflict-resolution client for collaboration apps.
 *
 * Backed by the server-authoritative op-log at /api/collab/ops. It polls the
 * unresolved-conflict endpoint and surfaces a banner when two editors (or a
 * person and a concurrent AI agent) diverged. Each conflict shows who edited
 * (👤 human / 🤖 llm) and offers accept-server (keep the newer state) or
 * accept-client (rebase your change on top).
 *
 * Public API (window.GBCollabConflict):
 *   start({ resourceType, resourceId, pollMs }) — begin polling + banner
 *   stop()                                      — stop polling
 *   refresh()                                   — one-shot check
 *   submitOp(opType, baseVersion, payload, actorType) — append an op
 *      (returns a promise resolving to { op_id, lamport, current_version,
 *        conflict })
 *   resolve(id, resolution)                     — resolve a conflict
 *   getState()                                  — { current_version, vector }
 *
 * Auth: Bearer JWT from localStorage/sessionStorage (gb-access-token).
 */
(function (window) {
  var CSS_ID = "gb-conflict-css";
  var POLL_MS = 15000;
  var state = { resourceType: null, resourceId: null, timer: null, currentVersion: 0 };
  var banner = null;
  var listEl = null;

  function ensureCss() {
    if (document.getElementById(CSS_ID)) return;
    var css = [
      "#gb-conflict-banner{position:fixed;top:0;left:50%;transform:translateX(-50%);",
      "width:560px;max-width:94vw;z-index:100001;margin-top:12px;display:none;",
      "background:#7f1d1d;border:1px solid #b91c1c;border-radius:10px;",
      "box-shadow:0 10px 30px rgba(0,0,0,.4);font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;}",
      "#gb-conflict-banner.gbc-open{display:block;}",
      "#gb-conflict-banner .gbc-head{display:flex;align-items:center;gap:8px;",
      "padding:10px 14px;color:#fecaca;font-size:13px;font-weight:600;border-bottom:1px solid #b91c1c;}",
      "#gb-conflict-banner .gbc-head .gbc-count{background:#dc2626;color:#fff;border-radius:999px;",
      "padding:1px 8px;font-size:12px;}",
      "#gb-conflict-banner .gbc-close{margin-left:auto;background:none;border:none;color:#fca5a5;",
      "font-size:18px;line-height:1;cursor:pointer;padding:0 4px;}",
      "#gb-conflict-banner .gbc-list{max-height:280px;overflow-y:auto;padding:6px 10px 10px;}",
      "#gb-conflict-banner .gbc-item{background:#991b1b;border:1px solid #b91c1c;border-radius:8px;",
      "padding:10px 12px;margin-top:8px;color:#fee2e2;font-size:12.5px;}",
      "#gb-conflict-banner .gbc-who{color:#fff;font-weight:600;}",
      "#gb-conflict-banner .gbc-meta{color:#fca5a5;font-size:11.5px;margin-top:2px;}",
      "#gb-conflict-banner .gbc-actions{margin-top:8px;display:flex;gap:8px;}",
      "#gb-conflict-banner .gbc-actions button{background:#dc2626;border:1px solid #ef4444;color:#fff;",
      "border-radius:6px;padding:6px 10px;font-size:12px;cursor:pointer;}",
      "#gb-conflict-banner .gbc-actions button:hover{background:#ef4444;}",
      "#gb-conflict-banner .gbc-actions button.gbc-keep{background:#14532d;border-color:#16a34a;}",
      "#gb-conflict-banner .gbc-actions button.gbc-keep:hover{background:#166534;}",
      "#gb-conflict-banner .gbc-empty{color:#fca5a5;text-align:center;padding:10px;font-size:12.5px;}"
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

  function actorBadge(actorType) {
    return actorType === "llm" ? "🤖" : "👤";
  }

  function build() {
    ensureCss();
    if (banner && banner.parentNode) return banner;
    banner = document.createElement("div");
    banner.id = "gb-conflict-banner";
    banner.setAttribute("role", "alert");
    banner.setAttribute("aria-live", "polite");
    banner.innerHTML =
      '<div class="gbc-head">' +
      '<span>Conflicting edits</span>' +
      '<span class="gbc-count"></span>' +
      '<button class="gbc-close" title="Dismiss">×</button>' +
      '</div>' +
      '<div class="gbc-list"></div>';
    document.body.appendChild(banner);
    listEl = banner.querySelector(".gbc-list");
    banner.querySelector(".gbc-close").addEventListener("click", function () {
      banner.classList.remove("gbc-open");
    });
    return banner;
  }

  function render(conflicts) {
    var b = build();
    var count = b.querySelector(".gbc-count");
    count.textContent = conflicts.length;
    if (!conflicts.length) {
      b.classList.remove("gbc-open");
      return;
    }
    listEl.innerHTML = conflicts.map(function (c) {
      var payload = c.payload && typeof c.payload === "object"
        ? JSON.stringify(c.payload) : String(c.payload || "");
      return '<div class="gbc-item" data-id="' + esc(c.id) + '">' +
        '<div><span class="gbc-who">' + actorBadge(c.actor_type) + ' ' + esc(c.actor_name || c.actor_id) + '</span>' +
        ' edited concurrently (' + esc(c.op_type) + ')</div>' +
        '<div class="gbc-meta">' + esc(payload.slice(0, 120)) + '</div>' +
        '<div class="gbc-actions">' +
        '<button class="gbc-keep" data-res="accept-server">Keep server version</button>' +
        '<button data-res="accept-client">Keep my change</button>' +
        '</div>' +
        '</div>';
    }).join("");
    b.classList.add("gbc-open");
  }

  function refresh() {
    if (!state.resourceType || !state.resourceId) return Promise.resolve([]);
    return api("/api/collab/conflicts?resource_type=" +
      encodeURIComponent(state.resourceType) + "&resource_id=" +
      encodeURIComponent(state.resourceId))
      .then(function (conflicts) {
        render(conflicts || []);
        return conflicts || [];
      })
      .catch(function () { return []; });
  }

  function start(opts) {
    stop();
    state.resourceType = opts.resourceType;
    state.resourceId = opts.resourceId;
    state.pollMs = opts.pollMs || POLL_MS;
    refresh();
    state.timer = setInterval(refresh, state.pollMs);
  }

  function stop() {
    if (state.timer) {
      clearInterval(state.timer);
      state.timer = null;
    }
    if (banner) banner.classList.remove("gbc-open");
  }

  function submitOp(opType, baseVersion, payload, actorType) {
    if (!state.resourceType || !state.resourceId) return Promise.reject(new Error("Conflict watch not started"));
    return api("/api/collab/ops", {
      method: "POST",
      body: JSON.stringify({
        resource_type: state.resourceType,
        resource_id: state.resourceId,
        op_type: opType,
        base_version: baseVersion,
        actor_type: actorType || "human",
        payload: payload || {}
      })
    }).then(function (r) {
      state.currentVersion = r.current_version;
      if (r.conflict) refresh();
      return r;
    });
  }

  function resolve(id, resolution) {
    return api("/api/collab/conflicts/" + encodeURIComponent(id) + "/resolve", {
      method: "POST",
      body: JSON.stringify({ resolution: resolution })
    }).then(function (r) {
      refresh();
      return r;
    });
  }

  function getState() {
    return api("/api/collab/ops/state?resource_type=" +
      encodeURIComponent(state.resourceType) + "&resource_id=" +
      encodeURIComponent(state.resourceId))
      .then(function (s) {
        state.currentVersion = s.current_version;
        return s;
      });
  }

  // Delegate resolve clicks from the banner.
  document.addEventListener("click", function (e) {
    var btn = e.target;
    if (!btn || !btn.getAttribute || !btn.hasAttribute("data-res")) return;
    var item = btn.closest(".gbc-item");
    if (!item) return;
    resolve(item.getAttribute("data-id"), btn.getAttribute("data-res")).catch(function () {});
  });

  window.GBCollabConflict = {
    start: start, stop: stop, refresh: refresh,
    submitOp: submitOp, resolve: resolve, getState: getState
  };
})(window);
