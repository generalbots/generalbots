"use strict";
/* #2342 (partial) — persistent engineer status strip: connection, bot/session, model, turn state. */
(function () {
  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function text(el, value) {
    if (!el) return;
    var v = value == null ? "" : String(value);
    if (el.textContent !== v) el.textContent = v;
  }

  function currentBot() {
    return (window.ChatState && (window.ChatState.currentBotName || window.ChatState.currentBotId)) ||
      window.__INITIAL_BOT_NAME__ || "default";
  }

  function currentSession() {
    return (window.ChatState && window.ChatState.currentSessionId) || "";
  }

  function currentModel() {
    var picker = document.getElementById("modelPicker");
    if (!picker) return "";
    var opt = picker.options[picker.selectedIndex];
    return (opt && (opt.textContent || opt.value)) || picker.value || "";
  }

  function connectionInfo() {
    var el = document.getElementById("connectionStatus");
    var cls = (el && el.className) || "";
    var txt = el ? (el.querySelector(".connection-text") || el).textContent || "" : "";
    txt = txt.trim();
    if (/connected|online/i.test(cls) || /online/i.test(txt)) {
      return { key: "connected", label: txt || t("chat-online", "Online") };
    }
    if (/connecting|reconnect/i.test(cls) || /connect/i.test(txt)) {
      return { key: "connecting", label: txt || t("chat-connecting", "Connecting…") };
    }
    return { key: "offline", label: txt || t("chat-offline", "Offline") };
  }

  function turnInfo() {
    try {
      if (window.GBTurn && typeof window.GBTurn.state === "function") {
        var st = window.GBTurn.state();
        if (st && st !== "idle") {
          if (st === "streaming") return t("chat-status-streaming", "Streaming");
          if (st === "thinking" || st === "queued") return t("chat-status-thinking", "Thinking");
          if (st === "error") return t("chat-status-error", "Error");
          if (st === "cancelled" || st === "stopped") return t("chat-status-stopped", "Stopped");
          return st;
        }
      }
    } catch (e) {}
    return t("chat-status-idle", "Idle");
  }

  async function copySession() {
    var sid = currentSession();
    if (!sid) return;
    try {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(sid);
      } else {
        var ta = document.createElement("textarea");
        ta.value = sid;
        ta.style.position = "fixed";
        ta.style.opacity = "0";
        document.body.appendChild(ta);
        ta.select();
        document.execCommand("copy");
        document.body.removeChild(ta);
      }
      var btn = document.getElementById("chatStatusCopySession");
      if (btn) {
        btn.dataset.copied = "1";
        btn.title = t("chat-status-copied", "Copied");
        setTimeout(function () {
          if (btn) {
            btn.dataset.copied = "";
            btn.title = t("chat-status-copy-session", "Copy session id");
          }
        }, 1200);
      }
      if (typeof window.showToast === "function") {
        window.showToast(t("chat-status-copied", "Copied"), "success");
      }
    } catch (e) {}
  }

  function ensureStrip() {
    var wrapper = document.getElementById("chatContentWrapper");
    if (!wrapper || document.getElementById("chatEngineerStatus")) return document.getElementById("chatEngineerStatus");
    var strip = document.createElement("div");
    strip.id = "chatEngineerStatus";
    strip.className = "gb-chat-engineer-status";
    strip.setAttribute("role", "status");
    strip.setAttribute("aria-live", "polite");
    strip.innerHTML =
      '<span class="gb-chat-es-item gb-chat-es-conn" data-state="connecting">' +
        '<span class="gb-chat-es-dot" aria-hidden="true"></span>' +
        '<span id="chatStatusConnText" class="gb-chat-es-text"></span>' +
      '</span>' +
      '<span class="gb-chat-es-sep" aria-hidden="true">·</span>' +
      '<span class="gb-chat-es-item"><span class="gb-chat-es-label" data-i18n="chat-status-bot">Bot</span> <span id="chatStatusBot" class="gb-chat-es-value"></span></span>' +
      '<span class="gb-chat-es-sep" aria-hidden="true">·</span>' +
      '<span class="gb-chat-es-item"><span class="gb-chat-es-label" data-i18n="chat-status-session">Session</span> <span id="chatStatusSession" class="gb-chat-es-value gb-chat-es-mono"></span>' +
        '<button type="button" id="chatStatusCopySession" class="gb-chat-es-copy" aria-label="' + t("chat-status-copy-session", "Copy session id") + '" title="' + t("chat-status-copy-session", "Copy session id") + '">⧉</button>' +
      '</span>' +
      '<span class="gb-chat-es-sep" aria-hidden="true">·</span>' +
      '<span class="gb-chat-es-item"><span class="gb-chat-es-label" data-i18n="chat-status-model">Model</span> <span id="chatStatusModel" class="gb-chat-es-value"></span></span>' +
      '<span class="gb-chat-es-sep" aria-hidden="true">·</span>' +
      '<span class="gb-chat-es-item"><span class="gb-chat-es-label" data-i18n="chat-status-state">State</span> <span id="chatStatusTurn" class="gb-chat-es-value"></span></span>';

    var header = document.getElementById("chatConversationHeader") || document.getElementById("chatHeaderActions");
    if (header && header.parentNode === wrapper) {
      wrapper.insertBefore(strip, header.nextSibling);
    } else {
      wrapper.insertBefore(strip, wrapper.firstChild);
    }
    var copyBtn = document.getElementById("chatStatusCopySession");
    if (copyBtn) copyBtn.addEventListener("click", copySession);
    return strip;
  }

  function update() {
    var strip = ensureStrip();
    if (!strip) return;
    var conn = connectionInfo();
    var connItem = strip.querySelector(".gb-chat-es-conn");
    if (connItem) connItem.dataset.state = conn.key;
    text(document.getElementById("chatStatusConnText"), conn.label);
    text(document.getElementById("chatStatusBot"), currentBot());
    var sid = currentSession();
    var shortSid = sid ? (sid.length > 12 ? sid.slice(0, 8) + "…" + sid.slice(-4) : sid) : t("chat-status-no-session", "—");
    var sessEl = document.getElementById("chatStatusSession");
    text(sessEl, shortSid);
    if (sessEl) sessEl.title = sid || "";
    var copyBtn = document.getElementById("chatStatusCopySession");
    if (copyBtn) copyBtn.disabled = !sid;
    text(document.getElementById("chatStatusModel"), currentModel() || "—");
    text(document.getElementById("chatStatusTurn"), turnInfo());
  }

  var scheduled = false;
  function scheduleUpdate() {
    if (scheduled) return;
    scheduled = true;
    setTimeout(function () {
      scheduled = false;
      update();
    }, 50);
  }

  function bind() {
    ensureStrip();
    update();
    var conn = document.getElementById("connectionStatus");
    if (conn && typeof MutationObserver !== "undefined") {
      try {
        new MutationObserver(scheduleUpdate).observe(conn, { attributes: true, childList: true, subtree: true, characterData: true });
      } catch (e) {}
    }
    var picker = document.getElementById("modelPicker");
    if (picker) picker.addEventListener("change", scheduleUpdate);
    document.addEventListener("gb-chat-session-changed", scheduleUpdate);
    document.addEventListener("gb-chat-turn-state", scheduleUpdate);
    window.addEventListener("focus", scheduleUpdate);
    setInterval(update, 2000);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bind);
  } else {
    bind();
  }
  window.gbUpdateEngineerStatus = update;
})();
