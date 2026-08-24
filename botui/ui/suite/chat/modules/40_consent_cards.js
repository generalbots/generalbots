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

  function authHeaders(extra) {
    var h = extra || {};
    try {
      var t = window.getGBAccessToken ? window.getGBAccessToken()
        : (localStorage.getItem("gb-access-token") ||
           sessionStorage.getItem("gb-access-token") ||
           localStorage.getItem("management_token"));
      if (t) h["Authorization"] = "Bearer " + t;
    } catch (e) { /* anonymous */ }
    return h;
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
      '<div class="gb-consent-actions">' +
      '<button type="button" class="gb-consent-btn primary" data-decision="allow_once">Allow once</button>' +
      '<button type="button" class="gb-consent-btn" data-decision="always">Always allow</button>' +
      '<button type="button" class="gb-consent-btn danger" data-decision="deny">Deny</button>' +
      "</div>";
    card.querySelector(".gb-consent-app").textContent =
      req.app_id + " \u00B7 " + (req.action_class || "action");
    card.querySelector(".gb-consent-detail").textContent = req.detail || "";
    card.addEventListener("click", function (e) {
      var btn = e.target.closest(".gb-consent-btn");
      if (!btn) return;
      resolve(card, req.request_id, btn.getAttribute("data-decision"));
    });
    return card;
  }

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
      new Date().toLocaleTimeString() +
      "</span></div>";
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

  function revivePending() {
    fetch("/api/consent/pending", { headers: authHeaders() })
      .then(function (r) { return r.ok ? r.json() : null; })
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
      var bin = atob(b64.replace(/-/g, "+").replace(/_/g, "/"));
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
        if (sender === "bot" && req) {
          var pane = document.getElementById("messages");
          var lastMsgEl = pane ? pane.lastElementChild : null;
          if (lastMsgEl && lastMsgEl.classList.contains("bot")) {
            var body = lastMsgEl.querySelector(".message-content");
            if (body) {
              // Replace the marker payload with the interactive card; the
              // leading sentence stays for screen readers and copy-out.
              body.innerHTML = "";
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

  installHook();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", installHook);
  } else {
    installHook();
  }

  // Revive unresolved cards once the messages pane exists.
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      setTimeout(revivePending, 600);
    });
  } else {
    setTimeout(revivePending, 600);
  }
})();
