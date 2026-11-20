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
    return String(model || "").replace(/^acct:/, "").replace(/\.gguf$/i, "");
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
