/* Chat memory — persists user facts across conversations (GB6-16).
 * Stores key-value pairs in localStorage keyed by bot name.
 * The server can send a "memory_update" message to store facts, and
 * the client injects remembered facts into outgoing payloads. */
"use strict";

(function () {
  var STORAGE_PREFIX = "gb_chat_memory_";
  var MAX_ENTRIES = 100;
  var MAX_VALUE_LEN = 500;

  function storageKey() {
    var bot = (ChatState && ChatState.currentBotName) || window.__INITIAL_BOT_NAME__ || "default";
    return STORAGE_PREFIX + bot;
  }

  function loadMemory() {
    try { return JSON.parse(localStorage.getItem(storageKey()) || "{}"); } catch (e) { return {}; }
  }

  function saveMemory(mem) {
    try { localStorage.setItem(storageKey(), JSON.stringify(mem)); } catch (e) {}
  }

  // GB6-16 — the server is the durable copy: localStorage keeps the fast path,
  // but every write is also persisted so a new session (or another device)
  // starts with the same preferences.
  function authToken() {
    return localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
  }

  function persistToServer(key, value) {
    var token = authToken();
    if (!token) return;
    fetch("/api/user/memory", {
      method: "PUT",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ key: key, value: value }),
    }).catch(function () { /* offline: the local copy stays authoritative */ });
  }

  function deleteOnServer(key) {
    var token = authToken();
    if (!token) return;
    fetch("/api/user/memory/" + encodeURIComponent(key), {
      method: "DELETE",
      headers: { Authorization: "Bearer " + token },
    }).catch(function () { /* offline */ });
  }

  async function syncFromServer() {
    var token = authToken();
    if (!token) return;
    try {
      var response = await fetch("/api/user/memory", { headers: { Authorization: "Bearer " + token } });
      if (!response.ok) return;
      var payload = await response.json();
      var memories = Array.isArray(payload.memories) ? payload.memories : [];
      if (!memories.length) return;
      var mem = loadMemory();
      memories.forEach(function (entry) {
        if (!entry || !entry.key) return;
        mem[entry.key] = { value: String(entry.value || "").substring(0, MAX_VALUE_LEN), _ts: Date.parse(entry.updated_at) || Date.now() };
      });
      saveMemory(mem);
      document.dispatchEvent(new CustomEvent("gb-memory-synced"));
    } catch (e) { /* offline: local memory still works */ }
  }

  function remember(key, value) {
    if (!key || typeof key !== "string") return;
    var mem = loadMemory();
    if (Object.keys(mem).length >= MAX_ENTRIES && !(key in mem)) {
      var oldest = Object.keys(mem).sort(function (a, b) {
        return (mem[a]._ts || 0) - (mem[b]._ts || 0);
      })[0];
      if (oldest) delete mem[oldest];
    }
    mem[key] = { value: String(value).substring(0, MAX_VALUE_LEN), _ts: Date.now() };
    saveMemory(mem);
    persistToServer(key, String(value));
  }

  function forget(key) {
    var mem = loadMemory();
    delete mem[key];
    saveMemory(mem);
    deleteOnServer(key);
  }

  function getMemorySummary() {
    var mem = loadMemory();
    var keys = Object.keys(mem);
    if (keys.length === 0) return null;
    var lines = keys.map(function (k) { return k + ": " + mem[k].value; });
    return lines.join("; ");
  }

  function getMemoryEntries() {
    var mem = loadMemory();
    var result = [];
    Object.keys(mem).forEach(function (k) { result.push({ key: k, value: mem[k].value }); });
    return result;
  }

  function clearMemory() {
    try { localStorage.removeItem(storageKey()); } catch (e) {}
  }

  document.addEventListener("gb-ws-frame", function (e) {
    var data = e.detail;
    if (!data) return;
    if (data.type === "memory_update" || data.message_type === "memory_update") {
      var updates = data.memories || data.facts || [];
      if (Array.isArray(updates)) {
        updates.forEach(function (entry) {
          if (entry.forget) forget(entry.key);
          else remember(entry.key, entry.value);
        });
      }
    }
  });

  syncFromServer();
  document.addEventListener("gb-chat-ready", syncFromServer);

  window.ChatMemory = {
    syncFromServer: syncFromServer,
    remember: remember,
    forget: forget,
    getMemorySummary: getMemorySummary,
    getMemoryEntries: getMemoryEntries,
    clearMemory: clearMemory,
    loadMemory: loadMemory,
  };
})();
