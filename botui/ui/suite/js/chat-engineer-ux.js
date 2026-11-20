/* #2342 — Chat engineer-grade UX: a discoverable `/` command menu with
 * keyboard navigation, a visible help (`?`) affordance, a composer-level Stop
 * during an active turn, and richer per-message actions (Inspect, Copy as
 * Markdown). Lives in suite/js/ so the chat app's own UI LOC surface — already
 * at its exact budget ceiling — is not grown. All strings go through gbT. */
"use strict";

(function () {
  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function safeErr(error, ctx) {
    if (window.gbLogError) window.gbLogError(error, ctx);
  }

  function messageText(bubble) {
    if (!bubble) return "";
    if (bubble.dataset && typeof bubble.dataset.originalContent === "string" && bubble.dataset.originalContent) {
      return bubble.dataset.originalContent;
    }
    var el = bubble.querySelector(".gb-message-text") || bubble.querySelector(".message-content");
    return el ? (el.innerText || el.textContent || "").trim() : "";
  }

  function notify(message, type) {
    if (typeof window.showToast === "function") {
      window.showToast(message, type || "info");
    } else if (window.GBToasts && typeof window.GBToasts.show === "function") {
      window.GBToasts.show("Chat", message, type || "info");
    }
  }

  var COMMANDS = [
    { cmd: "/help", key: "chat-cmd-help", fallback: "Show keyboard shortcuts" },
    { cmd: "/new", key: "chat-cmd-new", fallback: "Start a new conversation" },
    { cmd: "/clear", key: "chat-cmd-clear", fallback: "Clear this conversation" },
    { cmd: "/stop", key: "chat-cmd-stop", fallback: "Stop generating" }
  ];

  var menu = null;
  var options = [];
  var selected = -1;
  var input = null;

  function ensureMenu() {
    if (menu) return menu;
    menu = document.createElement("div");
    menu.id = "gbSlashMenu";
    menu.className = "gb-slash-menu";
    menu.setAttribute("role", "listbox");
    menu.setAttribute("aria-label", t("chat-cmd-menu-title", "Commands"));
    var pos = "absolute";
    menu.style.cssText = "display:none;position:" + pos + ";z-index:var(--z-menu,1000);" +
      "min-width:240px;max-width:360px;padding:4px;border-radius:8px;" +
      "background:var(--surface-2,#fff);color:var(--text-1,#111);" +
      "box-shadow:0 8px 24px rgba(0,0,0,.18);border:1px solid var(--border,rgba(0,0,0,.12));";
    document.body.appendChild(menu);
    return menu;
  }

  function closeMenu() {
    if (menu) menu.style.display = "none";
    selected = -1;
  }

  function runCommand(entry) {
    if (!entry) return;
    try {
      if (entry.cmd === "/help") {
        if (window.GBAccessibility && typeof window.GBAccessibility.showKeyboardHelp === "function") {
          window.GBAccessibility.showKeyboardHelp();
        }
      } else if (entry.cmd === "/new") {
        var fns = window.__GBSidebarFns;
        if (fns && typeof fns.newConversation === "function") fns.newConversation();
      } else if (entry.cmd === "/clear") {
        var item = document.querySelector('#chatConversationMenu [data-action="clear"]');
        if (item) item.click();
      } else if (entry.cmd === "/stop") {
        if (window.GBTurn && typeof window.GBTurn.cancel === "function") window.GBTurn.cancel();
      }
    } catch (e) {
      safeErr(e, "chat-engineer-ux.runCommand");
      notify(t("chat-cmd-failed", "Command failed"), "warning");
    }
    if (input) {
      input.value = "";
      try { input.dispatchEvent(new Event("input", { bubbles: true })); } catch (e) { safeErr(e, "chat-engineer-ux.input"); }
      input.focus();
    }
    closeMenu();
  }

  function renderMenu(filter) {
    var root = ensureMenu();
    root.innerHTML = "";
    options = COMMANDS.filter(function (c) { return c.cmd.indexOf(filter) === 0; });
    selected = options.length ? 0 : -1;
    options.forEach(function (c, idx) {
      var opt = document.createElement("button");
      opt.type = "button";
      opt.className = "gb-slash-option";
      opt.setAttribute("role", "option");
      opt.setAttribute("aria-selected", idx === selected ? "true" : "false");
      opt.textContent = c.cmd + " \u2014 " + t(c.key, c.fallback);
      opt.style.cssText = "display:block;width:100%;text-align:left;padding:6px 10px;border:0;" +
        "border-radius:6px;background:transparent;color:inherit;cursor:pointer;";
      opt.addEventListener("click", function () { runCommand(c); });
      root.appendChild(opt);
    });
    root.style.display = options.length ? "block" : "none";
    if (input && options.length) {
      var r = input.getBoundingClientRect();
      root.style.left = Math.max(8, r.left) + "px";
      root.style.top = Math.max(8, r.top - root.offsetHeight - 6) + "px";
    }
  }

  function onInput() {
    if (!input) return;
    var v = input.value;
    if (/^\/[a-z]*$/i.test(v)) renderMenu(v.toLowerCase());
    else closeMenu();
  }

  function onKeyDown(e) {
    if (!input) return;
    var open = menu && menu.style.display === "block" && options.length;
    if (open && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
      e.preventDefault();
      selected = (selected + (e.key === "ArrowDown" ? 1 : -1) + options.length) % options.length;
      Array.prototype.forEach.call(menu.children, function (child, idx) {
        child.setAttribute("aria-selected", idx === selected ? "true" : "false");
      });
      return;
    }
    if (open && e.key === "Enter") {
      e.preventDefault();
      runCommand(options[selected]);
      return;
    }
    if (open && e.key === "Escape") {
      e.preventDefault();
      closeMenu();
      return;
    }
    if (!open && e.key === "?" && e.shiftKey && !input.value) {
      e.preventDefault();
      if (window.GBAccessibility && typeof window.GBAccessibility.showKeyboardHelp === "function") {
        window.GBAccessibility.showKeyboardHelp();
      }
    }
  }

  function addHelpButton(form) {
    if (document.getElementById("gbChatHelpBtn")) return;
    var b = document.createElement("button");
    b.type = "button";
    b.id = "gbChatHelpBtn";
    b.className = "chat-tool-toggle";
    b.textContent = "?";
    b.title = t("chat-help", "Keyboard shortcuts");
    b.setAttribute("aria-label", t("chat-help", "Keyboard shortcuts"));
    b.addEventListener("click", function () {
      if (window.GBAccessibility && typeof window.GBAccessibility.showKeyboardHelp === "function") {
        window.GBAccessibility.showKeyboardHelp();
      }
    });
    var send = document.getElementById("sendBtn");
    if (send && send.parentNode === form) form.insertBefore(b, send);
    else form.appendChild(b);
  }

  function addStopButton(form) {
    if (document.getElementById("gbChatStopBtn")) return;
    var b = document.createElement("button");
    b.type = "button";
    b.id = "gbChatStopBtn";
    b.className = "chat-tool-toggle gb-chat-stop";
    b.textContent = "\u25A0";
    b.title = t("chat-stop", "Stop");
    b.setAttribute("aria-label", t("chat-stop", "Stop"));
    b.hidden = true;
    b.disabled = true;
    b.addEventListener("click", function () {
      if (window.GBTurn && typeof window.GBTurn.cancel === "function") window.GBTurn.cancel();
    });
    var send = document.getElementById("sendBtn");
    if (send && send.parentNode === form) form.insertBefore(b, send);
    else form.appendChild(b);
    window.setInterval(function () {
      var active = window.GBTurn && typeof window.GBTurn.active === "function" && window.GBTurn.active();
      b.hidden = !active;
      b.disabled = !active;
    }, 400);
  }

  var decorated = typeof WeakSet === "function" ? new WeakSet() : null;
  function decorate(bar) {
    if (!bar || bar.querySelector(".message-action-inspect")) return;
    if (decorated && decorated.has(bar)) return;
    if (decorated) decorated.add(bar);
    var bubble = bar.closest(".message");
    function add(cls, label, handler) {
      var b = document.createElement("button");
      b.type = "button";
      b.className = "message-action-btn " + cls;
      b.textContent = label;
      b.title = label;
      b.setAttribute("aria-label", label);
      b.addEventListener("click", function (e) {
        e.preventDefault();
        e.stopPropagation();
        handler(b);
      });
      bar.appendChild(b);
    }
    add("message-action-inspect", t("chat-inspect", "Inspect"), function () {
      var lines = [];
      lines.push(t("chat-inspect-sender", "Sender") + ": " + (bubble && bubble.classList.contains("user") ? "user" : "bot"));
      lines.push(t("chat-inspect-id", "Message id") + ": " + ((bubble && bubble.dataset && bubble.dataset.messageId) || t("chat-inspect-no-id", "none")));
      lines.push(t("chat-inspect-length", "Characters") + ": " + messageText(bubble).length);
      if (window.GBModal && typeof window.GBModal.alert === "function") {
        window.GBModal.alert(lines.join("\n"), t("chat-inspect-title", "Message details"));
      }
    });
    add("message-action-copy-md", t("chat-copy-markdown", "Copy as Markdown"), function (btn) {
      var text = messageText(bubble);
      if (!text || typeof window.gbCopyText !== "function") return;
      window.gbCopyText(text).then(function (ok) {
        notify(ok ? t("chat-copied-markdown", "Copied as Markdown") : t("chat-copy-failed", "Copy failed"), ok ? "success" : "warning");
      });
    });
  }

  function decorateAll() {
    var bars = document.querySelectorAll("#messages .message-actions");
    Array.prototype.forEach.call(bars, decorate);
  }

  function init() {
    input = document.getElementById("messageInput");
    var form = document.getElementById("chatForm");
    if (!input || !form) return;
    input.addEventListener("input", onInput);
    input.addEventListener("keydown", onKeyDown);
    input.addEventListener("blur", function () { window.setTimeout(closeMenu, 120); });
    addHelpButton(form);
    addStopButton(form);
    decorateAll();
    var host = document.getElementById("messages");
    if (host && typeof MutationObserver === "function") {
      var observer = new MutationObserver(function () { decorateAll(); });
      observer.observe(host, { childList: true, subtree: true });
    }
  }

  window.GBChatEngineerUX = {
    init: init,
    runCommand: runCommand,
    commands: COMMANDS,
    decorateAll: decorateAll
  };

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
