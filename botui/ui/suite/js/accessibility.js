/* Accessibility and keyboard-first use (GB6-27).
 * Adds ARIA landmarks, skip-nav, keyboard shortcuts, focus trapping
 * in modals, high-contrast mode, and reduced-motion support. */
"use strict";

(function () {
  function addSkipNav() {
    if (document.getElementById("gb-skip-nav")) return;
    var skip = document.createElement("a");
    skip.id = "gb-skip-nav";
    skip.href = "#messages";
    skip.textContent = "Skip to main content";
    skip.style.cssText = "position:fixed;top:-100px;left:50%;transform:translateX(-50%);background:#3b82f6;color:#fff;padding:10px 20px;border-radius:0 0 8px 8px;z-index:999999;font-size:14px;font-weight:600;transition:top 0.2s;";
    skip.addEventListener("focus", function () { skip.style.top = "0"; });
    skip.addEventListener("blur", function () { skip.style.top = "-100px"; });
    document.body.insertBefore(skip, document.body.firstChild);
  }

  function addAriaLandmarks() {
    var chat = document.getElementById("chat-app");
    if (chat && !chat.getAttribute("role")) {
      chat.setAttribute("role", "main");
      chat.setAttribute("aria-label", "Chat conversation");
    }
    var messages = document.getElementById("messages");
    if (messages) {
      messages.setAttribute("role", "log");
      messages.setAttribute("aria-live", "polite");
      messages.setAttribute("aria-label", "Chat messages");
    }
    var chatForm = document.getElementById("chatForm");
    if (chatForm) {
      chatForm.setAttribute("role", "form");
      chatForm.setAttribute("aria-label", "Send message");
    }
    var input = document.getElementById("messageInput");
    if (input) {
      input.setAttribute("aria-label", "Type your message");
    }
    var sendBtn = document.getElementById("sendBtn");
    if (sendBtn) {
      sendBtn.setAttribute("aria-label", "Send message");
    }
  }

  function setupGlobalShortcuts() {
    document.addEventListener("keydown", function (e) {
      if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA" || e.target.isContentEditable) return;

      if (e.key === "/" && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        var input = document.getElementById("messageInput");
        if (input) input.focus();
      }

      if (e.key === "?" && e.shiftKey) {
        e.preventDefault();
        showKeyboardHelp();
      }

      if (e.key === "Escape") {
        var modals = document.querySelectorAll('[id$="-modal"], .gb-artifact-panel, #gb-sandbox-panel');
        if (modals.length > 0) {
          var last = modals[modals.length - 1];
          if (last.remove) last.remove();
        }
      }

      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        if (window.CommandPalette && typeof CommandPalette.toggle === "function") {
          CommandPalette.toggle();
        }
      }

      if ((e.ctrlKey || e.metaKey) && e.key === ",") {
        e.preventDefault();
        if (window.GBPrivacy) GBPrivacy.showPanel();
      }
    });
  }

  function showKeyboardHelp() {
    var modal = document.getElementById("keyboard-help-modal");
    if (modal) { modal.remove(); return; }
    modal = document.createElement("div");
    modal.id = "keyboard-help-modal";
    modal.style.cssText = "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:99999;backdrop-filter:blur(8px);";

    var shortcuts = [
      ["/", "Focus chat input"],
      ["Shift + ?", "Show this help"],
      ["Escape", "Close modal / clear selection"],
      ["Ctrl + K", "Command palette"],
      ["Ctrl + ,", "Privacy settings"],
      ["PageUp / PageDown", "Scroll chat history"],
      ["Tab", "Navigate between elements"],
      ["Enter", "Activate focused element"],
      ["Delete", "Delete selected files (Drive)"],
      ["Ctrl + A", "Select all files (Drive)"],
      ["Ctrl + C / X / V", "Copy / Cut / Paste files"],
      ["Backspace", "Navigate up in Drive"],
      ["F2", "Rename selected file"],
    ];

    var rows = shortcuts.map(function (s) {
      return '<tr><td style="padding:6px 12px;"><kbd style="background:#0f172a;border:1px solid #334155;border-radius:4px;padding:2px 8px;font-family:monospace;font-size:12px;color:#e2e8f0;">' + s[0] + '</kbd></td><td style="padding:6px 12px;color:#94a3b8;font-size:13px;">' + s[1] + '</td></tr>';
    }).join("");

    modal.innerHTML = '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:480px;max-height:80vh;overflow-y:auto;padding:24px;">'
      + '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:16px;"><h3 style="margin:0;color:#f8fafc;">Keyboard Shortcuts</h3><button onclick="document.getElementById(\'keyboard-help-modal\').remove()" style="background:none;border:none;color:#94a3b8;font-size:24px;cursor:pointer;">×</button></div>'
      + '<table style="width:100%;border-collapse:collapse;">' + rows + '</table>'
      + '</div>';
    modal.addEventListener("click", function (e) { if (e.target === modal) modal.remove(); });
    document.body.appendChild(modal);
  }

  function trapFocusInModal(modal) {
    if (!modal) return;
    var focusable = modal.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])');
    if (focusable.length === 0) return;
    var first = focusable[0];
    var last = focusable[focusable.length - 1];
    modal.addEventListener("keydown", function (e) {
      if (e.key !== "Tab") return;
      if (e.shiftKey) {
        if (document.activeElement === first) { e.preventDefault(); last.focus(); }
      } else {
        if (document.activeElement === last) { e.preventDefault(); first.focus(); }
      }
    });
    first.focus();
  }

  function applyReducedMotion() {
    if (window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      var style = document.createElement("style");
      style.id = "gb-reduced-motion";
      style.textContent = "*, *::before, *::after { animation-duration: 0.01ms !important; animation-iteration-count: 1 !important; transition-duration: 0.01ms !important; }";
      document.head.appendChild(style);
    }
  }

  function applyHighContrast() {
    if (window.matchMedia && window.matchMedia("(prefers-contrast: more)").matches) {
      var style = document.createElement("style");
      style.id = "gb-high-contrast";
      style.textContent = ":root { --border: #888 !important; --text: #fff !important; --text-secondary: #ccc !important; } .message-content { border: 1px solid #555; }";
      document.head.appendChild(style);
    }
  }

  function observeNewModals() {
    var observer = new MutationObserver(function (mutations) {
      mutations.forEach(function (m) {
        m.addedNodes.forEach(function (node) {
          if (node.nodeType === 1 && node.id && node.id.indexOf("-modal") !== -1) {
            trapFocusInModal(node);
          }
        });
      });
    });
    observer.observe(document.body, { childList: true });
  }

  function init() {
    addSkipNav();
    addAriaLandmarks();
    setupGlobalShortcuts();
    applyReducedMotion();
    applyHighContrast();
    observeNewModals();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }

  window.GBAccessibility = {
    showKeyboardHelp: showKeyboardHelp,
    trapFocus: trapFocusInModal,
  };
})();
