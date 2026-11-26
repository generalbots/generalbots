/* Assistant parity — Virtual Machine consent inside classic chat.
 *
 * The classic chat already renders consent cards (modules/40_consent_cards.js)
 * and already resolves them at /api/consent/resolve. This module never adds a
 * second transport: it observes the cards chat renders for requests whose
 * detail.vm is set and turns them into the assistant's VM card (agent name,
 * command, where it runs, offers, protected-path notice). Decisions are posted
 * through the same endpoint; a blocking VM decision never sends a chat
 * continuation message — the server wakes the waiting tool call through the
 * native_tools consent gate.
 */
"use strict";
(function () {
  var P = (window.GBChatAssistantParity = window.GBChatAssistantParity || {});
  var requests = Object.create(null);
  var upgraded = Object.create(null);
  var mounted = false;
  var timer = 0;
  var lastTyping = 0;

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function esc(value) {
    return String(value == null ? "" : value).replace(/&/g, "&amp;")
      .replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }
  function headers(extra) {
    var h = extra || {};
    try {
      if (window.GBShared && window.GBShared.auth && typeof window.GBShared.auth.headers === "function") {
        var all = window.GBShared.auth.headers();
        Object.keys(all).forEach(function (key) { h[key] = all[key]; });
      }
    } catch (error) { /* fall through to the chat token */ }
    try {
      var token = (typeof window.getChatAccessToken === "function" && window.getChatAccessToken()) ||
        (window.GBSecurity && window.GBSecurity.getToken && window.GBSecurity.getToken()) ||
        sessionStorage.getItem("gb-access-token") || "";
      if (token) h.Authorization = "Bearer " + token;
    } catch (error) { /* anonymous chat */ }
    return h;
  }
  function thread() {
    return String((window.ChatState && ChatState.currentSessionId) || "");
  }
  function isVm(request) {
    return !!(request && request.detail && request.detail.vm && request.request_id);
  }
  function remember(request) {
    if (!isVm(request)) return false;
    if (!request.thread_id) request.thread_id = thread();
    requests[String(request.request_id)] = request;
    return true;
  }
  function announce(text) {
    var live = document.getElementById("gbParityVmAnnouncer");
    if (!live) {
      live = document.createElement("div");
      live.id = "gbParityVmAnnouncer";
      live.className = "gb-parity-vm-announcer";
      live.setAttribute("role", "status");
      live.setAttribute("aria-live", "polite");
      (document.getElementById("chatForm") || document.body).appendChild(live);
    }
    live.textContent = text || "";
  }
  function focusComposer() {
    var input = document.getElementById("messageInput");
    if (!input) return;
    try { input.focus({ preventScroll: true }); } catch (error) { input.focus(); }
  }

  /* The assistant names who asks. Chat has no A._agentMeta, so the agent id is
   * resolved from the options chat's own model picker installed. */
  function agentName(agent) {
    if (!agent || agent === "chat") return t("chat-vm-consent-assistant", "Assistant");
    var select = document.getElementById("modelPicker");
    if (select) {
      for (var i = 0; i < select.options.length; i++) {
        if (select.options[i].value === "sub:" + agent) {
          return String(select.options[i].textContent || agent).trim();
        }
      }
    }
    var linked = window.gbLinkedAgents || [];
    for (var j = 0; j < linked.length; j++) {
      if (linked[j] && linked[j].provider === agent) return linked[j].name || agent;
    }
    return agent;
  }
  function subject(agent) {
    return agentName(agent) + " \u00B7 " + t("chat-vm-consent-subject", "Agent");
  }
  function title(detail) {
    var wants = detail.kind === "command" || detail.kind === "shell"
      ? t("chat-vm-consent-wants-command", "The agent wants to run a command")
      : t("chat-vm-consent-wants", "The agent wants to perform an action");
    return wants + " \u2014 " + subject(detail.agent);
  }
  function fact(label, value) {
    if (value === undefined || value === null || value === "" ||
        (Array.isArray(value) && !value.length)) return "";
    var text = Array.isArray(value) ? value.join(", ") : String(value);
    return '<div class="gb-parity-vm-meta"><span>' + esc(label) +
      "</span> " + esc(text) + "</div>";
  }
  function where(detail) {
    if (detail.kind === "drive_save") {
      return fact(t("chat-vm-consent-destination", "Destination"),
        t("chat-vm-consent-where-drive", "your Drive") + (detail.folder ? " \u00B7 " + detail.folder : ""));
    }
    if (detail.kind === "external_save") {
      return fact(t("chat-vm-consent-destination", "Destination"),
        (detail.provider || "") + (detail.folder ? " \u00B7 " + detail.folder : ""));
    }
    return fact(t("chat-vm-consent-where", "Where"),
      t("chat-vm-consent-where-vm", "Virtual Machine") + (detail.cwd ? " \u00B7 " + detail.cwd : ""));
  }
  function offersFor(detail) {
    var raw = Array.isArray(detail.offers) && detail.offers.length
      ? detail.offers : ["allow_once", "deny"];
    var allowed = ["allow_once", "allow_conversation", "allow_all", "deny"].filter(function (name) {
      return raw.indexOf(name) !== -1;
    });
    /* Protected paths and actions that leave the VM never get a broad grant. */
    if (detail.protected) {
      allowed = allowed.filter(function (name) {
        return name === "allow_once" || name === "deny";
      });
    }
    if (detail.outside_vm) {
      allowed = allowed.filter(function (name) { return name !== "allow_all"; });
    }
    if (!allowed.length) allowed = ["deny"];
    return allowed;
  }
  function noteFor(detail, offers) {
    if (detail.protected) {
      return '<p class="gb-parity-vm-note gb-parity-vm-note--warn" role="status" aria-live="polite">' +
        esc(t("chat-vm-consent-protected",
          "This request involves the agent credentials; it can only be allowed once.")) + "</p>";
    }
    if (offers.indexOf("allow_all") !== -1) {
      return '<p class="gb-parity-vm-note" role="status" aria-live="polite">' +
        esc(t("chat-vm-consent-all-note",
          "Allow all covers every action by this agent in this conversation, only inside the Virtual Machine.")) + "</p>";
    }
    if (detail.outside_vm) {
      return '<p class="gb-parity-vm-note" role="status" aria-live="polite">' +
        esc(t("chat-vm-consent-outside-note",
          "This action leaves the Virtual Machine: allow all does not cover it.")) + "</p>";
    }
    return "";
  }
  function decisionLabel(name) {
    if (name === "allow_once") return t("chat-vm-consent-allow-once", "Allow once");
    if (name === "allow_conversation") return t("chat-vm-consent-allow-conversation", "Allow in this conversation");
    if (name === "allow_all") return t("chat-vm-consent-allow-all", "Allow all");
    return t("chat-vm-consent-deny", "Deny");
  }
  function buttonClass(name) {
    if (name === "allow_once") return "gb-parity-vm-btn--primary";
    if (name === "deny") return "gb-parity-vm-btn--deny";
    return "";
  }

  function decorate(request, card) {
    var detail = request.detail || {};
    var offers = offersFor(detail);
    var titleId = "gbChatVmConsentTitle-" + String(request.request_id).replace(/[^A-Za-z0-9_-]/g, "");
    var owner = detail.owner_id || request.user_id || request.owner_user_id;
    var me = window.ChatState && ChatState.currentUserId;
    var foreign = !!(owner && me && String(owner) !== String(me));
    var blocking = request.blocking === false ? "0" : "1";

    card.className = "gb-consent-card gb-parity-vm-consent";
    card.dataset.requestId = String(request.request_id);
    card.dataset.threadId = String(request.thread_id || thread());
    card.dataset.blocking = blocking;
    card.dataset.agent = detail.agent || "chat";
    card.setAttribute("role", "group");
    card.setAttribute("aria-labelledby", titleId);
    card.tabIndex = -1;

    var actions = foreign ? "" : offers.map(function (name) {
      return '<button type="button" class="gb-parity-vm-btn ' + buttonClass(name) +
        '" data-decision="' + name + '">' + esc(decisionLabel(name)) + "</button>";
    }).join("");
    var note = foreign
      ? '<p class="gb-parity-vm-note gb-parity-vm-note--warn" role="alert">' +
        esc(t("chat-vm-consent-forbidden", "This decision cannot be recorded here. The request stays open.")) + "</p>"
      : noteFor(detail, offers);

    card.innerHTML =
      '<div class="gb-parity-vm-head">' +
        '<span class="gb-parity-vm-shield" aria-hidden="true">\u{1F6E1}</span>' +
        '<div class="gb-parity-vm-headings">' +
          '<div class="gb-parity-vm-kicker">' + esc(t("chat-vm-consent-kicker", "Needs your permission")) + "</div>" +
          '<div class="gb-parity-vm-title" id="' + titleId + '">' + esc(title(detail)) + "</div>" +
          (detail.repair ? '<div class="gb-parity-vm-subtitle">' +
            esc(t("chat-vm-consent-repair", "Corrected version of the program")) + "</div>" : "") +
        "</div></div>" +
      '<div class="gb-parity-vm-body">' +
        '<pre class="gb-parity-vm-code" tabindex="0" aria-label="' +
          esc(t("chat-vm-consent-command-label", "Command")) + '">' + esc(detail.text || "") + "</pre>" +
        where(detail) +
        fact(t("chat-vm-consent-language", "Language"), detail.language) +
        fact(t("chat-vm-consent-packages", "Installs from the internet"), detail.packages) +
        fact(t("chat-vm-consent-from", "Source"), detail.from) +
        (detail.background ? '<div class="gb-parity-vm-meta">' +
          esc(t("chat-vm-consent-background", "Runs in the background")) + "</div>" : "") +
      "</div>" +
      (actions ? '<div class="gb-parity-vm-actions">' + actions + "</div>" : "") + note;

    card.addEventListener("click", function (event) {
      var btn = event.target && event.target.closest ? event.target.closest(".gb-parity-vm-btn") : null;
      if (btn && !btn.disabled && !card.classList.contains("resolved")) {
        resolve(card, request, btn.getAttribute("data-decision"));
      }
    });
    card.addEventListener("keydown", function (event) {
      if (event.key !== "Escape" || card.classList.contains("resolved")) return;
      event.preventDefault();
      event.stopPropagation();
      focusComposer();
      announce(t("chat-vm-consent-still-open",
        "A permission request is still open. Choose an option; Deny runs nothing."));
    });
    upgraded[String(request.request_id)] = true;
    return card;
  }

  function outcome(decision) {
    if (decision === "allow_all") return t("chat-vm-consent-outcome-all", "Full permission in this conversation");
    if (decision === "allow_conversation") return t("chat-vm-consent-outcome-conversation", "Allowed in this conversation");
    if (decision === "allow_once") return t("chat-vm-consent-outcome-once", "Allowed once");
    if (decision === "deny") return t("chat-vm-consent-outcome-denied", "Denied by you: nothing was run");
    return null;
  }
  function showResolved(card, text, failed) {
    card.classList.add("resolved", "gb-parity-vm-resolved");
    if (failed) card.classList.add("gb-parity-vm-failed");
    card.innerHTML = '<div class="gb-parity-vm-outcome" role="status" aria-live="polite">' +
      esc(text) + "</div>";
  }
  function decisionEvent(card, request, decision, ok) {
    /* Server-side gap (cannot be closed in the UI): chat's WebSocket has no
     * frame for a VM decision and the browser cannot call native_tools.
     * Botserver must wake the waiting tool call in botconsent::resolve and
     * feed the decision back to the model via native_tools::tell_model. This
     * UI only records the choice and emits the structured hand-off below. */
    try {
      window.dispatchEvent(new CustomEvent("gb-vm-consent-decision", {
        detail: {
          requestId: String(request.request_id),
          decision: decision,
          blocking: card.dataset.blocking === "1",
          agent: card.dataset.agent || "chat",
          threadId: card.dataset.threadId || "",
          ok: !!ok,
          contract: "native_tools"
        }
      }));
    } catch (error) { /* custom events are optional in older browsers */ }
  }
  function resolve(card, request, decision) {
    var hadFocus = card.contains(document.activeElement);
    var buttons = card.querySelectorAll(".gb-parity-vm-btn");
    Array.prototype.forEach.call(buttons, function (button) { button.disabled = true; });
    var body = { request_id: request.request_id, decision: decision };
    if (card.dataset.threadId) body.thread_id = card.dataset.threadId;
    fetch("/api/consent/resolve", {
      method: "POST",
      headers: headers({ "Content-Type": "application/json" }),
      body: JSON.stringify(body)
    }).then(function (response) {
      return response.json().catch(function () { return null; })
        .then(function (data) { return { ok: response.ok, status: response.status, data: data }; });
    }).then(function (result) {
      if (result.status === 403 || (result.data && result.data.error === "forbidden")) {
        Array.prototype.forEach.call(buttons, function (button) { button.disabled = false; });
        var warn = document.createElement("p");
        warn.className = "gb-parity-vm-note gb-parity-vm-note--warn";
        warn.setAttribute("role", "alert");
        warn.textContent = t("chat-vm-consent-forbidden", "This decision cannot be recorded here. The request stays open.");
        card.appendChild(warn);
        return;
      }
      var ok = result.ok && !(result.data && result.data.error);
      showResolved(card, ok ? (outcome(decision) ||
        t("chat-vm-consent-outcome-once", "Allowed once")) :
        t("chat-vm-consent-outcome-failed", "Could not record your decision"), !ok);
      decisionEvent(card, request, decision, ok);
      if (hadFocus) focusComposer();
      window.setTimeout(function () { if (card.parentNode) card.parentNode.removeChild(card); }, 3000);
    }).catch(function () {
      Array.prototype.forEach.call(buttons, function (button) { button.disabled = false; });
      showResolved(card, t("chat-vm-consent-outcome-failed", "Could not record your decision"), true);
    });
  }

  function findCard(id) {
    var all = document.querySelectorAll('.gb-consent-card[data-request-id="' + CSS.escape(id) + '"]');
    for (var i = 0; i < all.length; i++) {
      if (!all[i].classList.contains("resolved")) return all[i];
    }
    return null;
  }
  function upgradeAll() {
    var cards = document.querySelectorAll(".gb-consent-card[data-request-id]");
    Array.prototype.forEach.call(cards, function (card) {
      var id = String(card.getAttribute("data-request-id") || "");
      if (!id || upgraded[id] || card.classList.contains("resolved")) return;
      var request = requests[id];
      if (isVm(request)) decorate(request, card);
    });
  }
  function scheduleUpgrade() {
    if (timer) return;
    timer = window.setTimeout(function () { timer = 0; upgradeAll(); }, 80);
  }
  function loadPending() {
    try {
      var cfg = window.productConfig;
      if (cfg && Array.isArray(cfg.compiled_features) &&
          cfg.compiled_features.indexOf("consent") === -1) return;
      fetch("/api/consent/pending", { cache: "no-store", headers: headers() })
        .then(function (response) { return response.ok ? response.json() : null; })
        .then(function (data) {
          if (!data) return;
          var items = Array.isArray(data) ? data :
            (data.items || data.requests || data.pending || []);
          items.forEach(remember);
          upgradeAll();
        })
        .catch(function () { /* revival is best effort */ });
    } catch (error) { /* no consent API in this build */ }
  }
  function onFrame(event) {
    var detail = (event && event.detail) || {};
    var request = detail.consent_request || (detail.data && detail.data.consent_request);
    if (remember(request)) scheduleUpgrade();
  }
  function onTyping(event) {
    var tag = event.target && event.target.tagName;
    if (tag === "TEXTAREA" || tag === "INPUT") lastTyping = Date.now();
  }
  function openCard() {
    var cards = document.querySelectorAll(".gb-parity-vm-consent");
    for (var i = 0; i < cards.length; i++) {
      var card = cards[i];
      if (card.isConnected && !card.classList.contains("resolved") &&
          (!card.dataset.threadId || card.dataset.threadId === thread())) return card;
    }
    return null;
  }
  function waitingCard() {
    var card = openCard();
    return card && card.dataset.blocking === "1" ? card : null;
  }
  function onKey(event) {
    if (!event.altKey || event.ctrlKey || event.metaKey || event.code !== "KeyP") return;
    var card = openCard();
    if (!card) return;
    event.preventDefault();
    (card.lastElementChild || card).scrollIntoView({ block: "nearest" });
    var first = card.querySelector(".gb-parity-vm-btn:not(:disabled)");
    (first || card).focus();
  }

  P.consentVm = {
    mount: function () {
      if (mounted) return P;
      mounted = true;
      document.addEventListener("gb-ws-frame", onFrame);
      document.addEventListener("input", onTyping, true);
      window.addEventListener("keydown", onKey, true);
      window.addEventListener("gb-vm-consent-refresh", scheduleUpgrade);
      if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", function () {
          loadPending();
          scheduleUpgrade();
        }, { once: true });
      } else {
        loadPending();
        scheduleUpgrade();
      }
      var host = document.getElementById("messages");
      if (host && typeof MutationObserver === "function") {
        new MutationObserver(scheduleUpgrade).observe(host, { childList: true, subtree: true });
      }
      return P;
    },
    remember: remember,
    upgradeAll: upgradeAll,
    findCard: findCard,
    openCard: openCard,
    waitingCard: waitingCard,
    outcomeText: outcome,
    agentName: agentName,
    announce: announce,
    typing: function () {
      var active = document.activeElement;
      var busy = active && (active.tagName === "TEXTAREA" || active.tagName === "INPUT") &&
        String(active.value || "").trim() !== "";
      return busy || Date.now() - lastTyping < 2000;
    }
  };
})();
