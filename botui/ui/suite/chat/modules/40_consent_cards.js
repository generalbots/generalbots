"use strict";

/**
 * Consent card renderer (#1176-fe).
 * Renders a .gb-consent-card for bot frames carrying
 * consent_request:{request_id,app_id,action_class,detail}. The three
 * actions (Allow once / Always allow / Deny) POST /api/consent/resolve
 * {request_id,decision} with the suite Bearer token and swap the card to an
 * outcome summary. At boot, GET /api/consent/pending revives unresolved
 * requests so they survive a page reload.
 */

(function () {
  var DECISIONS = {
    allow_once: { label: "Allow once", outcome: "Allowed once" },
    allow_run: { label: "Allow for this run", outcome: "Allowed for this run" },
    allow_workspace: { label: "Allow for 1 hour", outcome: "Allowed for 1 hour" },
    always: { label: "Always allow", outcome: "Always allowed" },
    deny: { label: "Deny", outcome: "Denied" },
  };

  var lastConsentRequest = null;

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

  // The payload's detail is a JSON object (e.g. {command:"api.exec",
  // params:{method:"GET",path:"/api/..."}}). Rendering it directly printed
  // "[object Object]" and left the user unable to see WHAT was being approved
  // (UAT billing/products runs). Summarise it into one readable line.
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

  function buildCard(req) {
    var card = document.createElement("div");
    card.className = "gb-consent-card";
    card.setAttribute("data-request-id", String(req.request_id));
    card.innerHTML =
      '<div class="gb-consent-head">' +
      '<span class="gb-consent-glyph">\u{1F6E1}\uFE0F</span>' +
      '<div class="gb-consent-headings">' +
      '<div class="gb-consent-title">Permission request</div>' +
      '<div class="gb-consent-app"></div>' +
      "</div></div>" +
      '<div class="gb-consent-detail"></div>' +
      '<div class="gb-consent-risk"></div>' +
      '<div class="gb-consent-actions">' +
      '<button type="button" class="gb-consent-btn primary" data-decision="allow_once">Allow once</button>' +
      '<button type="button" class="gb-consent-btn" data-decision="allow_run">This run</button>' +
      '<button type="button" class="gb-consent-btn" data-decision="allow_workspace">1 hour</button>' +
      '<button type="button" class="gb-consent-btn" data-decision="always">Always allow</button>' +
      '<button type="button" class="gb-consent-btn danger" data-decision="deny">Deny</button>' +
      "</div>";
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
    card.classList.add("resolved", ok ? "granted" : "failed");
    card.innerHTML =
      '<div class="gb-consent-outcome">' +
      '<span class="gb-consent-outcome-icon">' + (ok ? "\u2705" : "\u26A0\uFE0F") + "</span>" +
      '<span class="gb-consent-outcome-text">' +
      escapeHtml(ok ? meta.outcome : "Could not record your decision") +
      "</span>" +
      '<span class="gb-consent-outcome-time">' +
      gbFormatTime(new Date()) +
      "</span></div>";
  }

  /**
   * A granted decision must RESUME the turn. The server ends the turn with the
   * consent marker, so the assistant never retries the gated call by itself:
   * "Allow once" used to grant the capability and then sit there, and UAT runs
   * for billing/tax/products stalled on a card that had already been approved.
   * Sending the continuation through `window.sendMessage` routes to the tab
   * that owns the card (multichat wraps that global), so the retry lands in the
   * same conversation and session the approval is bound to.
   */
  function resumeAfterGrant(decision) {
    if (decision === "deny") return;
    if (typeof window.sendMessage !== "function") return;
    setTimeout(function () {
      try {
        window.sendMessage("Approved \u2014 continue with my previous request.");
      } catch (e) { /* the grant still applies to the next message the user sends */ }
    }, 200);
  }

  function resolve(card, requestId, decision) {
    var buttons = card.querySelectorAll(".gb-consent-btn");
    buttons.forEach(function (b) { b.disabled = true; });
    fetch("/api/consent/resolve", {
      method: "POST",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify({ request_id: requestId, decision: decision }),
    })
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        swapToOutcome(card, decision, true);
        resumeAfterGrant(decision);
      })
      .catch(function () {
        swapToOutcome(card, decision, false);
      });
  }

  /**
   * Renders one consent request into a message element.
   * Idempotent per request id within the conversation.
   */
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
    // Builds without the consent route would answer 404; the manifest says
    // so up front, and the browser logs every failed request (#1693).
    if (featureCompiled("consent") === false) {
      window.__gbConsentApiAvailable = false;
      return;
    }
    fetch("/api/consent/pending", { headers: authHeaders() })
      .then(function (r) {
        // Builds without the optional `consent` feature have no such route;
        // do not retry or log a 404 on every boot.
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

  // Hook: wrap the global addMessage (assignment preservation). This module
  // loads after 30_citations.js, chaining on top of any earlier wrapper.
  var MARKER = "GB-CONSENT-REQUEST:";

  function decodeMarker(content) {
    if (typeof content !== "string") return null;
    var idx = content.indexOf(MARKER);
    if (idx === -1) return null;
    try {
      var b64 = content.slice(idx + MARKER.length).trim();
      // base64url (no padding) -> standard base64; restore padding so atob
      // accepts payloads whose length is not a multiple of 4.
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

  // Shared by every render path (stock addMessage, parallel-tab panes,
  // streaming preview, history replay). `GBConsentStrip` guarantees the raw
  // `GB-CONSENT-REQUEST:<base64>` payload never reaches the user even when a
  // path cannot render the card (truncated chunk, legacy pane).
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
          // #2229 — during history replay, addMessage appends into a
          // DocumentFragment (ChatState.appendTarget), not #messages. Look in
          // the same container the message actually went into, otherwise the
          // card is attached to the wrong bubble and the marker text leaks.
          var pane = (window.ChatState && ChatState.appendTarget) || document.getElementById("messages");
          var lastMsgEl = pane ? pane.lastElementChild : null;
          if (lastMsgEl && lastMsgEl.classList && lastMsgEl.classList.contains("bot")) {
            var body = lastMsgEl.querySelector(".message-content");
            if (body) {
              // Replace the marker payload with the interactive card; the
              // leading sentence stays for screen readers and copy-out.
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

  // chat-send.js defines `addMessage` after this module loads, so a single
  // install at parse time found nothing and every consent prompt leaked its
  // raw `GB-CONSENT-REQUEST:<base64>` marker into the bubble (#2194).
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

  // Render-path independent sweep. Wrapping `window.addMessage` is fragile:
  // the chat reassigns that global from several modules (citations,
  // multichat) and history replay batches through a DocumentFragment, so a
  // marker could still reach the bubble as raw text (#2194). The observer
  // watches the transcript itself, so every path ends up with a real card.
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
