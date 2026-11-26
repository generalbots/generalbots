/* Names the model that wrote each answer (adapted from gbgov 03e_model_tag.js).
 * Wraps addMessage instead of editing the 326-line renderer. */
"use strict";
(function () {
  var KEY = "gb-answer-models";
  var MAX = 400;
  var turnModel = "";

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function hash(text) {
    var s = String(text || ""), h = 5381;
    for (var i = 0; i < s.length; i++) h = ((h << 5) + h + s.charCodeAt(i)) | 0;
    return (h >>> 0).toString(36) + ":" + s.length;
  }
  function load() {
    var map = typeof gbStoreGet === "function" ? gbStoreGet(KEY, {}) : {};
    return map && typeof map === "object" && !Array.isArray(map) ? map : {};
  }
  function save(map) {
    var keys = Object.keys(map);
    if (keys.length > MAX) {
      keys.sort(function (a, b) { return (map[a].t || 0) - (map[b].t || 0); });
      keys.slice(0, keys.length - MAX).forEach(function (k) { delete map[k]; });
    }
    try { localStorage.setItem(KEY, JSON.stringify(map)); }
    catch (e) { /* storage blocked */ }
  }
  function shortName(model) {
    var value = String(model || "");
    if (value.indexOf("sub:") === 0) {
      var id = value.slice(4);
      var hit = (window.gbLinkedAgents || []).filter(function (a) { return a.provider === id; })[0];
      return (hit && (hit.name || hit.provider)) || id;
    }
    return value.replace(/^acct:/, "").replace(/\.gguf$/i, "");
  }
  function pickerModel() {
    try { return localStorage.getItem("gb-model-preference") || "auto"; }
    catch (e) { return "auto"; }
  }
  function modelFor(content) {
    var rec = load()[hash(content)];
    if (rec && rec.m) return rec.m;
    return turnModel && turnModel !== "auto" ? turnModel : pickerModel();
  }
  function remember(content, model) {
    if (!content || !model) return;
    var map = load();
    map[hash(content)] = { m: model, t: Date.now() };
    save(map);
  }
  function attach(el, model) {
    if (!el || !model) return;
    var old = el.querySelector(".gb-chat-model-tag");
    if (old) old.remove();
    var chip = document.createElement("span");
    chip.className = "gb-chat-model-tag";
    chip.title = t("chat-model-written-by", "Answer written by") + " " + model;
    chip.textContent = shortName(model);
    el.appendChild(chip);
  }
  function containers() {
    var result = [], live = document.getElementById("messages"),
      target = ChatState && ChatState.appendTarget;
    if (live) result.push(live);
    if (target && result.indexOf(target) === -1) result.push(target);
    return result;
  }
  function wrapAddMessage() {
    var original = window.addMessage;
    if (typeof original !== "function" || original.__gbModelTagWrapped) return;
    var wrapped = function () {
      var snapshots = containers().map(function (container) {
        return { container: container, nodes: Array.prototype.slice.call(container.children) };
      });
      var result = original.apply(this, arguments);
      if (arguments[0] !== "bot") return result;
      var added = null;
      snapshots.forEach(function (snapshot) {
        if (added) return;
        var now = Array.prototype.slice.call(snapshot.container.children);
        for (var i = now.length - 1; i >= 0; i--) {
          if (snapshot.nodes.indexOf(now[i]) === -1 &&
              now[i].classList && now[i].classList.contains("bot")) {
            added = now[i];
            break;
          }
        }
      });
      if (!added) {
        var live = document.getElementById("messages");
        var bots = live && live.querySelectorAll(".message.bot");
        added = bots && bots.length ? bots[bots.length - 1] : null;
      }
      var content = arguments[1];
      if (added && content && String(content).trim()) {
        var model = modelFor(content);
        remember(content, model);
        attach(added, model);
      }
      return result;
    };
    wrapped.__gbModelTagWrapped = true;
    window.addMessage = wrapped;
  }
  function wrapProcessMessage() {
    var original = window.processMessage;
    if (typeof original !== "function" || original.__gbModelTagWrapped) return;
    var wrapped = function (data) {
      if (data && typeof data.model === "string" && data.model) turnModel = data.model;
      return original.apply(this, arguments);
    };
    wrapped.__gbModelTagWrapped = true;
    window.processMessage = wrapped;
  }
  window.addEventListener("gb-chat-message-sent", function (event) {
    var model = event && event.detail && event.detail.model;
    if (model) turnModel = model === "auto" ? "" : model;
  });
  window.GBChatModelTag = { remember: remember, attach: attach, modelFor: modelFor };
  wrapAddMessage();
  wrapProcessMessage();
})();

/* gbgov assistant parity (#2563): actionable engine refusal cards.
 * The server sends `error_code` + `error_params` when an engine cannot fulfil
 * a turn. Classic chat only marked the bubble retryable, leaving a person with
 * a bare "Retry" and no way to clear the cause. This renders the sentence they
 * can act on plus Retry / Choose another model. Co-located with the model tag
 * because both wrap processMessage and both describe the answer's engine. */
(function () {
  "use strict";

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  var CODES = {
    agent_not_linked: ["chat-engine-error-not-linked", "Your %label account is not connected. Choose another model or connect the account in Settings."],
    agent_switch_off: ["chat-engine-error-switch-off", "%label needs the Agent switch on in this conversation."],
    agent_unavailable: ["chat-engine-error-unavailable", "Subscription agents are not available on this computer. Choose another model."],
    agent_model_unavailable: ["chat-engine-error-model-unavailable", "The model %model is not available on %label. Choose another model."],
    agent_workspace_failed: ["chat-engine-error-workspace-failed", "The agent's virtual machine could not be started. Check its status and try again."],
    account_model_off: ["chat-engine-error-account-model-off", "The model %model is not enabled for %label. Choose another model."],
    free_model_unavailable: ["chat-engine-error-free-unavailable", "The free model is unavailable right now. Try again later or choose another model."],
    agent_limit_reached: ["chat-engine-error-limit-reached", "You reached your plan's usage limit on %label. %when Choose another model."],
    agent_unsafe_version: ["chat-engine-error-unsafe-version", "This version of %label (%version) was blocked because it did not ask permission before acting. Choose another model."]
  };
  function fill(text, params) {
    return String(text).replace(/%(\w+)/g, function (match, key) {
      return params[key] !== undefined && params[key] !== null ? String(params[key]) : match;
    });
  }
  function backAt(iso) {
    if (!iso) return "";
    var date = new Date(iso);
    if (isNaN(date.getTime())) return "";
    try {
      return t("chat-engine-error-back-at", "It comes back %when.")
        .replace("%when", date.toLocaleTimeString(document.documentElement.lang || undefined, {
          hour: "2-digit", minute: "2-digit",
        }));
    } catch (e) { return ""; }
  }
  function isErrorFrame(data) {
    if (!data || !data.error_code || !CODES[data.error_code]) return false;
    return !!(data.is_complete || data.is_error || data.failed || data.retryable ||
      data.status === "error" || data.status === "failed" ||
      data.turn_status === "error" || data.turn_status === "failed");
  }
  function ensureStyles() {
    if (document.getElementById("gbChatEngineErrorStyles")) return;
    var style = document.createElement("style");
    style.id = "gbChatEngineErrorStyles";
    style.textContent = [
      ".gb-engine-error .message-content{display:block}",
      ".gb-engine-error-card{display:block;padding:9px 11px;border:1px solid var(--id-danger,#dc2626);border-radius:9px;background:var(--id-danger-soft,#fee2e2);color:var(--id-danger-text,#991b1b);font-size:var(--id-text-sm,13px)}",
      ".gb-engine-error-card strong{display:block;margin-bottom:3px}",
      ".gb-engine-error-actions{display:flex;gap:8px;flex-wrap:wrap;margin-top:8px}",
      ".gb-engine-error-actions button{padding:5px 11px;border:1px solid var(--id-line,#d7dde5);border-radius:7px;background:var(--id-surface-solid,#fff);color:inherit;cursor:pointer;font:inherit}",
      ".gb-engine-error-actions button:focus-visible{outline:2px solid var(--id-focus,#2563eb);outline-offset:2px}"
    ].join("");
    (document.head || document.documentElement).appendChild(style);
  }
  function chooseModel() {
    var dockButton = document.querySelector(".gb-parity-model-btn");
    if (dockButton && typeof dockButton.click === "function") { dockButton.click(); return; }
    var select = document.getElementById("modelPicker");
    if (!select) return;
    if (typeof select.focus === "function") select.focus();
    if (typeof select.showPicker === "function") {
      try { select.showPicker(); } catch (e) { /* older browsers: focus is enough */ }
    }
  }
  function retry() {
    var cs = window.ChatState;
    if (typeof window.gbResendChatPayload === "function" && cs && cs.lastUserTurnPayload) {
      window.gbResendChatPayload(cs.lastUserTurnPayload);
      return;
    }
    var input = document.getElementById("messageInput");
    var text = input && input.value;
    if (text && typeof window.sendMessage === "function") window.sendMessage(text);
  }
  function render(data) {
    var messages = document.getElementById("messages");
    if (!messages) return;
    var last = messages.lastElementChild;
    if (last && last.classList && last.classList.contains("gb-engine-error") &&
        last.dataset.gbErrorCode === data.error_code) return;
    var params = data.error_params || {};
    params.when = backAt(params.reset);
    var entry = CODES[data.error_code];
    ensureStyles();
    var bubble = document.createElement("div");
    bubble.className = "message bot gb-engine-error";
    bubble.dataset.gbErrorCode = data.error_code;
    var content = document.createElement("div");
    content.className = "message-content";
    var card = document.createElement("div");
    card.className = "gb-engine-error-card";
    card.setAttribute("role", "alert");
    var title = document.createElement("strong");
    title.textContent = t("chat-status-error", "Error");
    var body = document.createElement("span");
    body.textContent = fill(t(entry[0], entry[1]), params);
    card.appendChild(title);
    card.appendChild(body);
    content.appendChild(card);
    var bar = document.createElement("div");
    bar.className = "gb-engine-error-actions";
    var retryButton = document.createElement("button");
    retryButton.type = "button";
    retryButton.className = "gb-retry-btn";
    retryButton.textContent = t("chat-retry", "Retry");
    retryButton.addEventListener("click", retry);
    var modelButton = document.createElement("button");
    modelButton.type = "button";
    modelButton.textContent = t("chat-engine-error-choose-model", "Choose another model");
    modelButton.addEventListener("click", chooseModel);
    bar.appendChild(retryButton);
    bar.appendChild(modelButton);
    content.appendChild(bar);
    bubble.appendChild(content);
    messages.appendChild(bubble);
    if (typeof window.scrollToBottom === "function") window.scrollToBottom(true);
  }
  var original = window.processMessage;
  if (typeof original !== "function" || original.__gbEngineErrors) return;
  var wrapped = function (data) {
    if (!isErrorFrame(data)) return original.apply(this, arguments);
    // A retry of the previous answer can carry the same error code: never let
    // it end the live turn or render a second card for it.
    if (window.GBTurn && typeof GBTurn.stale === "function" && GBTurn.stale(data)) return;
    var cs = window.ChatState;
    if (cs && cs.isStreaming && typeof window.finalizeStreaming === "function") {
      window.finalizeStreaming();
    }
    if (cs) {
      cs.isStreaming = false;
      cs.streamingMessageId = null;
      // Reuse the bounded late-frame window: the error is a forced turn end.
      cs.cancelledAt = Date.now();
    }
    if (window.GBTurn && typeof GBTurn.complete === "function") {
      GBTurn.complete(data, { content: "", reasoning: "" });
    }
    if (typeof window.clearChatPending === "function") window.clearChatPending();
    render(data);
    return true;
  };
  wrapped.__gbEngineErrors = true;
  window.processMessage = wrapped;
})();
