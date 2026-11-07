"use strict";
/* GB Suite Modal (#1844): in-app replacement for legacy alert/confirm/prompt.
   Uses the desktop WindowManager floating tool windows when available, falling
   back to native dialogs in isolated runs. */
const GBModal = (() => {
  function esc(s) {
    return String(s == null ? "" : s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function wm() {
    return (typeof window !== "undefined" && window.WindowManager) ? window.WindowManager : null;
  }

  function alert(message, title) {
    title = title || "Alert";
    const w = wm();
    if (w && w.openToolWindowBody) {
      const body = w.openToolWindowBody("gb-alert", title, {
        htmlContent:
          '<div class="gb-confirm-floating">' +
          "<p>" + esc(message) + "</p>" +
          '<div class="gb-confirm-actions">' +
          '<button data-gb-alert-ok class="gb-confirm-btn primary">OK</button>' +
          "</div></div>",
      });
      if (body) {
        const btn = body.querySelector("[data-gb-alert-ok]");
        if (btn) btn.addEventListener("click", () => w.close("gb-alert"));
      }
      return;
    }
    return globalThis.alert(message);
  }

  function confirm(message, title, onYes, onNo) {
    // Allow confirm(message, onYes) shorthand.
    if (typeof title === "function") {
      onNo = onYes;
      onYes = title;
      title = "Confirm";
    }
    title = title || "Confirm";
    const hasCallback = typeof onYes === "function";
    const w = wm();
    if (hasCallback && w && w.confirmFloating) {
      w.confirmFloating(title, message, onYes, onNo || function () {});
      return;
    }
    const ok = globalThis.confirm(message);
    if (ok && hasCallback) onYes();
    else if (!ok && typeof onNo === "function") onNo();
    return ok;
  }

  function prompt(message, defaultValue, title, onOk) {
    // Allow prompt(message, onOk) and prompt(message, defaultValue, onOk).
    if (typeof defaultValue === "function") {
      onOk = defaultValue;
      defaultValue = "";
      title = "Input";
    } else if (typeof title === "function") {
      onOk = title;
      title = "Input";
    }
    title = title || "Input";
    defaultValue = (defaultValue == null) ? "" : String(defaultValue);
    const hasCallback = typeof onOk === "function";
    const w = wm();
    if (hasCallback && w && w.promptFloating) {
      w.promptFloating(title, message, defaultValue, onOk);
      return;
    }
    const value = globalThis.prompt(message, defaultValue);
    if (hasCallback) onOk(value == null ? null : String(value).trim());
    return value;
  }

  return { alert: alert, confirm: confirm, prompt: prompt };
})();

window.GBModal = GBModal;
window.GBSuiteModal = GBModal;
window.GBModalHelper = GBModal;