"use strict";

/**
 * Consent card renderer (#1176-fe). Cards POST {request_id,decision} to
 * /api/consent/resolve; GET /api/consent/pending revives them on reload.
 */
(function () {
  function escapeHtml(value) {
    if (typeof window.escapeHtml === "function") return window.escapeHtml(value);
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  var DECISIONS = {
    allow_once: { label: "Allow once", outcome: "Allowed once" },
    allow_run: { label: "Allow for this run", outcome: "Allowed for this run" },
    allow_workspace: { label: "Allow for 1 hour", outcome: "Allowed for 1 hour" },
    always: { label: "Always allow", outcome: "Always allowed" },
    deny: { label: "Deny", outcome: "Denied" },
  };
  var CONFIRM_WIRE_DECISION = "allow_once", CANCEL_WIRE_DECISION = "deny";
  function wireDecision(d) {
    return d === "confirm" ? CONFIRM_WIRE_DECISION : (d === "cancel" ? CANCEL_WIRE_DECISION : d);
  }
  function isConfirmGate(r) {
    return !!(r && (r.confirm_required === true || r.action_class === "delete"));
  }

  var lastConsentRequest = null;
  // #2448: explicit batch id only; never a request id.
  function batchIdFrom(req) {
    if (!req || !isConfirmGate(req)) return "";
    var d = req.detail || {}, id = req.batch_id || (d.params || d).batch_id;
    return typeof id === "string" && id.trim() ? id.trim() : "";
  }
  window.GBConsentBatchId = batchIdFrom;

  document.addEventListener("gb-ws-frame", function (e) {
    var d = e.detail || {};
    if (d.message_type === 2 && d.consent_request && d.consent_request.request_id) {
      lastConsentRequest = d.consent_request;
    }
  });
  // #2347 — bearer tokens must not be read from localStorage.
  function authHeaders(extra) {
    var h = extra || {};
    try {
      var t = (typeof window.getChatAccessToken === "function" && window.getChatAccessToken())
        || (window.GBSecurity && window.GBSecurity.getToken && window.GBSecurity.getToken())
        || sessionStorage.getItem("gb-access-token")
        || "";
      if (t) h["Authorization"] = "Bearer " + t;
    } catch (e) { /* anonymous */ }
    return h;
  }
  function describeDetail(detail) {
    if (detail === null || detail === undefined) return "";
    if (typeof detail === "string") return detail;
    if (typeof detail !== "object") return String(detail);
    try {
      var parts = [];
      var command = detail.command || detail.tool || detail.name;
      if (command && typeof command === "string") parts.push(command);
      var params = detail.params;
      if (params && typeof params === "object") {
        var method = params.method ? String(params.method).toUpperCase() + " " : "";
        var path = params.path || params.url || params.endpoint || params.route || "";
        if (method || path) parts.push((parts.length ? " → " : "") + method + path);
        var inner = params.params;
        if (inner && typeof inner === "object") {
          var keys = Object.keys(inner);
          if (keys.length) {
            parts.push((parts.length ? " " : "") + JSON.stringify(inner).slice(0, 200));
          }
        } else if (typeof inner === "string" && inner) {
          parts.push((parts.length ? " " : "") + inner.slice(0, 200));
        }
      }
      var text = parts.join("").replace(/\s+/g, " ").trim();
      if (text) return text;
      return JSON.stringify(detail).slice(0, 300).replace(/\s+/g, " ");
    } catch (e) {
      return "";
    }
  }
  var PREVIEW_MAX_ROWS = 10, PREVIEW_VALUE_CHARS = 80;
  function t(k, fb) { return typeof gbT === "function" ? gbT(k, fb) : fb; }
  function pv(v) {
    var s = v === null || v === undefined ? "" : (typeof v === "object" ? JSON.stringify(v) : String(v));
    return s.length > PREVIEW_VALUE_CHARS ? s.slice(0, PREVIEW_VALUE_CHARS - 1) + "\u2026" : s;
  }
  function buildPreview(preview) {
    if (!preview || typeof preview !== "object") return null;
    var rows = Array.isArray(preview.entries) ? preview.entries.slice(0, PREVIEW_MAX_ROWS) : [];
    var count = preview.affected_count;
    if (!rows.length && (count === undefined || count === null)) return null;
    var region = document.createElement("div");
    region.className = "gb-consent-preview";
    region.setAttribute("role", "status");
    region.setAttribute("aria-live", "polite");
    var title = document.createElement("div");
    title.className = "gb-consent-preview-title";
    title.textContent = t("chat-preview", "Preview");
    region.appendChild(title);
    if (count !== undefined && count !== null) {
      var line = document.createElement("div");
      line.className = "gb-consent-preview-count";
      line.textContent = t("chat-consent-affected-count", "Affected records") + ": " + count;
      region.appendChild(line);
    }
    rows.forEach(function (e) {
      if (!e || typeof e !== "object") return;
      var row = document.createElement("div");
      row.className = "gb-consent-preview-row";
      row.textContent = pv(e.label) + ": " + pv(e.before) + " \u2192 " + pv(e.after);
      region.appendChild(row);
    });
    return region;
  }
  function buildCard(req) {
    var confirmGate = isConfirmGate(req);
    var actions = confirmGate
      ? '<button type="button" class="gb-consent-btn primary" data-decision="confirm">' + escapeHtml(t("action-confirm", "Confirm")) + '</button><button type="button" class="gb-consent-btn danger" data-decision="cancel">' + escapeHtml(t("action-cancel", "Cancel")) + '</button>'
      : '<button type="button" class="gb-consent-btn primary" data-decision="allow_once">Allow once</button><button type="button" class="gb-consent-btn" data-decision="allow_run">This run</button><button type="button" class="gb-consent-btn" data-decision="allow_workspace">1 hour</button><button type="button" class="gb-consent-btn" data-decision="always">Always allow</button><button type="button" class="gb-consent-btn danger" data-decision="deny">Deny</button>';
    var card = document.createElement("div");
    card.className = "gb-consent-card" + (confirmGate ? " gb-consent-confirm-gate" : "");
    card.setAttribute("data-request-id", String(req.request_id));
    var b = batchIdFrom(req);
    if (b) card.setAttribute("data-batch-id", b);
    card.innerHTML =
      '<div class="gb-consent-head">' +
      '<span class="gb-consent-glyph">\u{1F6E1}\uFE0F</span>' +
      '<div class="gb-consent-headings">' +
      '<div class="gb-consent-title">Permission request</div>' +
      '<div class="gb-consent-app"></div>' +
      "</div></div>" +
      '<div class="gb-consent-detail"></div>' +
      '<div class="gb-consent-risk"></div>' +
      '<div class="gb-consent-actions">' + actions + "</div>";
    var detail = req.detail && typeof req.detail === "object" ? req.detail : {};
    var appName = typeof req.app_id === "string" && req.app_id
      ? req.app_id
      : (typeof detail.app_id === "string" && detail.app_id ? detail.app_id : "workspace");
    var action = typeof req.action_class === "string" && req.action_class
      ? req.action_class
      : (typeof detail.action_class === "string" && detail.action_class ? detail.action_class : "action");
    card.querySelector(".gb-consent-app").textContent = appName + " \u00B7 " + action;
    card.querySelector(".gb-consent-detail").textContent = describeDetail(req.detail);
    var risk = card.querySelector(".gb-consent-risk");
    risk.textContent = req.risk_tier ? "Risk: " + req.risk_tier : "";
    risk.className = "gb-consent-risk risk-" + (req.risk_tier || "unknown");
    var previewRegion = buildPreview(req.preview);
    if (previewRegion) card.insertBefore(previewRegion, risk);
    card.addEventListener("click", function (e) {
      var btn = e.target.closest(".gb-consent-btn");
      if (!btn) return;
      resolve(card, req.request_id, btn.getAttribute("data-decision"));
    });
    return card;
  }
  window.GBBuildConsentCard = buildCard;
  function swapToOutcome(card, decision, ok) {
    var meta = DECISIONS[decision] || { outcome: decision };
    var text = decision === "confirm" ? t("action-confirm", "Confirm")
      : decision === "cancel" ? t("status-cancelled", "Cancelled") : meta.outcome;
    card.classList.add("resolved", ok ? "granted" : "failed");
    card.innerHTML =
      '<div class="gb-consent-outcome">' +
      '<span class="gb-consent-outcome-icon">' + (ok ? "\u2705" : "\u26A0\uFE0F") + "</span>" +
      '<span class="gb-consent-outcome-text">' +
      escapeHtml(ok ? text : "Could not record your decision") +
      "</span>" +
      '<span class="gb-consent-outcome-time">' +
      gbFormatTime(new Date()) +
      "</span></div>";
    // #2448 criterion 4 — one-click undo of the batch carried by this card;
    // user/bot/session are derived from the verified session, not from the UI.
    if (ok && decision === "confirm") appendUndoButton(card);
  }
  function appendUndoButton(card) {
    var id = card && card.getAttribute ? card.getAttribute("data-batch-id") : "";
    if (!id) return;
    var b = document.createElement("button");
    b.type = "button"; b.className = "gb-consent-btn gb-consent-undo";
    b.setAttribute("data-undo-batch", id); b.textContent = t("action-undo", "Undo");
    b.addEventListener("click", function () { undoBatch(id, b, card); });
    card.appendChild(b);
  }
  function undoBatch(id, button, card) {
    if (!id || !button || button.disabled) return;
    button.disabled = true;
    fetch("/api/collab/undo", { method: "POST",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify({ batch_id: id }) })
      .then(function (r) { if (!r.ok) throw new Error("HTTP " + r.status); return r.json(); })
      .then(function () { if (card) card.classList.add("gb-consent-undone"); })
      .catch(function () { button.disabled = false; });
  }
  window.GBUndoBatch = undoBatch;
  function resumeAfterGrant(decision) {
    if (decision === "deny") return;
    if (typeof window.sendMessage !== "function") return;
    setTimeout(function () {
      try {
        window.sendMessage("Approved. Continue and run the previously requested command now with the same parameters.");
      } catch (e) { /* the grant still applies to the next message the user sends */ }
    }, 200);
  }
  function resolve(card, requestId, decision) {
    var buttons = card.querySelectorAll(".gb-consent-btn");
    buttons.forEach(function (b) { b.disabled = true; });
    fetch("/api/consent/resolve", {
      method: "POST",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify({ request_id: requestId, decision: wireDecision(decision) }),
    })
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        swapToOutcome(card, decision, true);
        if (decision === "cancel") {
          lastConsentRequest = null;
          try { if (window.addMessage) window.addMessage("bot", t("status-cancelled", "Cancelled")); } catch (e) {}
        } else if (decision === "confirm") {
          if (typeof window.sendMessage === "function") setTimeout(function () {
            try { window.sendMessage("Confirmed. Continue and run the previously requested command now with the same parameters."); } catch (e) {}
          }, 200);
        } else {
          resumeAfterGrant(decision);
        }
      })
      .catch(function () {
        swapToOutcome(card, decision, false);
      });
  }
  // Renders one request into a message; idempotent per request id.
  window.GBRenderConsentCard = function (msgEl, req) {
    if (!msgEl || !req || !req.request_id) return;
    var pane = document.getElementById("messages");
    if (pane && pane.querySelector('.gb-consent-card[data-request-id="' + req.request_id + '"]')) {
      return;
    }
    if (msgEl.querySelector(".gb-consent-card")) return;
    msgEl.appendChild(buildCard(req));
  };
  function appendStandaloneCard(req) {
    var pane = document.getElementById("messages");
    if (!pane) return;
    if (pane.querySelector('.gb-consent-card[data-request-id="' + req.request_id + '"]')) {
      return;
    }
    var wrap = document.createElement("div");
    wrap.className = "message bot";
    wrap.setAttribute("data-gb-consent-revived", "1");
    wrap.innerHTML = '<div class="message-content bot-message"></div>';
    wrap.querySelector(".message-content").appendChild(buildCard(req));
    pane.appendChild(wrap);
  }
  function featureCompiled(name) {
    var cfg = window.productConfig;
    if (!cfg || !Array.isArray(cfg.compiled_features)) return null;
    return cfg.compiled_features.indexOf(name) !== -1;
  }
  function revivePending() {
    // A missing consent route answers 404 (#1693); check the manifest first.
    if (featureCompiled("consent") === false) {
      window.__gbConsentApiAvailable = false;
      return;
    }
    fetch("/api/consent/pending", { headers: authHeaders() })
      .then(function (r) {
        // No route in builds without `consent`; do not retry the 404.
        if (r.status === 404) {
          window.__gbConsentApiAvailable = false;
          return null;
        }
        return r.ok ? r.json() : null;
      })
      .then(function (d) {
        if (!d) return;
        var items = Array.isArray(d) ? d : (d.items || d.requests || d.pending || []);
        items.forEach(function (req) {
          if (req && req.request_id) appendStandaloneCard(req);
        });
      })
      .catch(function () { /* revival is best-effort */ });
  }
  var MARKER = "GB-CONSENT-REQUEST:";

  function decodeMarker(content) {
    if (typeof content !== "string") return null;
    var idx = content.indexOf(MARKER);
    if (idx === -1) return null;
    try {
      var b64 = content.slice(idx + MARKER.length).trim();
      var std = b64.replace(/-/g, "+").replace(/_/g, "/");
      while (std.length % 4 !== 0) std += "=";
      var bin = atob(std);
      var json = decodeURIComponent(
        Array.prototype.map.call(bin, function (c) {
          return "%" + ("00" + c.charCodeAt(0).toString(16)).slice(-2);
        }).join("")
      );
      var req = JSON.parse(json);
      return req && req.request_id ? req : null;
    } catch (e) {
      return null;
    }
  }

  // Every render path uses GBConsentStrip so the raw marker never reaches
  // the user, even when no card can be built (truncated chunk, legacy pane).
  var MARKER_RE = /GB-CONSENT-REQUEST:[A-Za-z0-9+/=_-]+/g;

  window.GBConsentExtract = decodeMarker;
  window.GBConsentStrip = function (content) {
    if (typeof content !== "string") return content;
    return content
      .replace(MARKER_RE, "")
      .replace(/[ \t]+$/gm, "")
      .replace(/\n{3,}/g, "\n\n")
      .trim();
  };

  function installHook() {
    if (typeof window.addMessage !== "function" || window.addMessage.__gbConsentHooked) {
      return;
    }
    var orig = window.addMessage;
    var wrapped = function (sender) {
      var result = orig.apply(this, arguments);
      try {
        var req = sender === "bot" ? lastConsentRequest : null;
        if (!req && arguments.length > 1) {
          req = decodeMarker(arguments[1]);
        }
        if (sender === "bot" && !req && typeof arguments[1] === "string") {
          // No decodable request (truncated chunk): still hide the payload.
          var cleaned = window.GBConsentStrip(arguments[1]);
          if (cleaned !== arguments[1]) arguments[1] = cleaned;
        }
        if (sender === "bot" && req) {
          // #2229 — history replay appends into ChatState.appendTarget, not
          // #messages; use that container or the card lands on the wrong bubble.
          var pane = (window.ChatState && ChatState.appendTarget) || document.getElementById("messages");
          var lastMsgEl = pane ? pane.lastElementChild : null;
          if (lastMsgEl && lastMsgEl.classList && lastMsgEl.classList.contains("bot")) {
            var body = lastMsgEl.querySelector(".message-content");
            if (body) {
              // Keep the leading sentence for screen readers / copy-out.
              var lead = typeof arguments[1] === "string"
                ? window.GBConsentStrip(arguments[1])
                : "";
              body.innerHTML = lead
                ? '<div class="gb-message-text">' + escapeHtml(lead) + "</div>"
                : "";
              body.appendChild(buildCard(req));
            } else {
              window.GBRenderConsentCard(lastMsgEl, req);
            }
          }
          lastConsentRequest = null;
        }
      } catch (e) { /* rendering must never break message flow */ }
      return result;
    };
    wrapped.__gbConsentHooked = true;
    window.addMessage = wrapped;
  }

  // chat-send.js defines addMessage later (#2194); retry the install.
  function installHookWhenReady(attempt) {
    var tries = attempt || 0;
    if (typeof window.addMessage === "function") {
      installHook();
      return;
    }
    if (tries >= 60) {
      return;
    }
    setTimeout(function () { installHookWhenReady(tries + 1); }, 250);
  }

  installHook();
  installHookWhenReady(0);
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () { installHookWhenReady(0); });
  }

  // Sweep the transcript itself: addMessage is reassigned by other modules and
  // history replay batches via a DocumentFragment (#2194).
  function renderMessageMarker(msgEl) {
    if (!msgEl || msgEl.dataset.gbConsentDone === "1") return;
    var body = msgEl.querySelector(".message-content") || msgEl;
    var text = body.textContent || "";
    if (text.indexOf(MARKER) === -1) return;
    msgEl.dataset.gbConsentDone = "1";
    var req = decodeMarker(text);
    var lead = window.GBConsentStrip(text);
    body.innerHTML = lead ? '<div class="gb-message-text">' + escapeHtml(lead) + "</div>" : "";
    if (req) body.appendChild(buildCard(req));
  }

  function sweepConsentMarkers() {
    var nodes = document.querySelectorAll("#messages .message, .chat-messages .message");
    for (var i = 0; i < nodes.length; i++) renderMessageMarker(nodes[i]);
  }

  function observeConsentMarkers() {
    sweepConsentMarkers();
    if (typeof window.MutationObserver !== "function") return;
    var scheduled = null;
    var observer = new MutationObserver(function () {
      if (scheduled) return;
      scheduled = setTimeout(function () {
        scheduled = null;
        sweepConsentMarkers();
      }, 120);
    });
    observer.observe(document.body, { childList: true, subtree: true });
  }

  // Revive unresolved cards once the messages pane exists.
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      observeConsentMarkers();
      setTimeout(revivePending, 600);
    });
  } else {
    observeConsentMarkers();
    setTimeout(revivePending, 600);
  }
})();
