"use strict";
/* Power menu (#2444): the appliance had no power button at all — the only
   "power" affordance in the start menu was a sign-out redirect, and the
   physical power key was logind's instant poweroff with no confirmation.
   This module is the UI half; `POST /api/system/power` in botsettings is the
   audited server half, and the image's `/usr/local/libexec/gb-power` helper
   is the only process that actually suspends/reboots/powers off.

   Nothing here is destructive without an explicit confirmation, and the menu
   also opens from the Tauri `gb-power-menu` event, which is what a kiosk
   close request becomes (#2450). */

const PowerMenu = (() => {
  const PANEL_ID = "gb-power-menu";
  let initialized = false;

  function t(key, fallback) {
    return window.gbT ? window.gbT(key, fallback) : fallback;
  }

  const ITEMS = [
    { id: "lock", label: "power-menu-lock", fallback: "Lock", icon: "M12 17v5M7 11V7a5 5 0 0 1 10 0v4M5 11h14a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-8a1 1 0 0 1 1-1z" },
    { id: "signout", label: "power-menu-signout", fallback: "Sign out", icon: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4M16 17l5-5-5-5M21 12H9" },
    { id: "suspend", label: "power-menu-suspend", fallback: "Suspend", icon: "M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z" },
    { id: "restart", label: "power-menu-restart", fallback: "Restart", icon: "M21 12a9 9 0 1 1-3-6.7M21 3v6h-6" },
    { id: "shutdown", label: "power-menu-shutdown", fallback: "Shut down", icon: "M12 2v10M18.4 6.6a9 9 0 1 1-12.8 0" }
  ];

  /** Tauri events are the shell bridge already used by the power menu (#2450).
   *  Emitting a named event is a no-op when no native listener is attached. */
  function emitShell(name) {
    try {
      var tauri = window.__TAURI__;
      if (!tauri || !tauri.event || typeof tauri.event.emit !== "function") return;
      var result = tauri.event.emit(name, {});
      if (result && typeof result.catch === "function") {
        result.catch(function (error) { console.warn("power menu: shell event", error); });
      }
    } catch (error) {
      console.warn("power menu: shell event", error);
    }
  }

  function isOpen() {
    return document.getElementById(PANEL_ID) !== null;
  }

  function close() {
    const panel = document.getElementById(PANEL_ID);
    if (panel) panel.remove();
  }

  function open() {
    if (isOpen()) return;
    const panel = document.createElement("div");
    panel.id = PANEL_ID;
    panel.className = "gb-power-menu";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-modal", "true");
    panel.setAttribute("aria-label", t("power-menu-title", "Power"));
    panel.innerHTML =
      '<div class="gb-power-box"><div class="gb-power-title">' +
      t("power-menu-title", "Power") +
      "</div>" +
      ITEMS.map(function (item) {
        return (
          '<button type="button" class="gb-power-item" data-power="' +
          item.id +
          '"><svg viewBox="0 0 24 24" width="18" height="18" fill="none" ' +
          'stroke="currentColor" stroke-width="2" stroke-linecap="round" ' +
          'stroke-linejoin="round" aria-hidden="true"><path d="' +
          item.icon +
          '"/></svg><span>' +
          t(item.label, item.fallback) +
          "</span></button>"
        );
      }).join("") +
      '<button type="button" class="gb-power-item gb-power-cancel" data-power="cancel">' +
      t("power-menu-cancel", "Cancel") +
      "</button></div>";
    document.body.appendChild(panel);

    panel.addEventListener("click", function (event) {
      if (event.target === panel) close();
    });
    panel.querySelectorAll("[data-power]").forEach(function (button) {
      button.addEventListener("click", function () {
        const action = button.getAttribute("data-power");
        close();
        if (action && action !== "cancel") run(action);
      });
    });
    const first = panel.querySelector("[data-power]");
    if (first) first.focus();
    document.addEventListener("keydown", function onKey(event) {
      if (event.key === "Escape") {
        close();
        document.removeEventListener("keydown", onKey, true);
      }
    }, true);
  }

  /** Asks before anything irreversible, then calls the audited endpoint. */
  async function run(action) {
    if (action === "lock") {
      // Ask logind to lock the session as well; the webview overlay always
      // requires the credential, even if no native listener exists (#2446).
      emitShell("gb-lock-request");
      if (window.LockScreen) window.LockScreen.lock("power-menu");
      return;
    }

    const confirmKey = "power-menu-confirm-" + action;
    const fallbackConfirm = {
      signout: "Sign out of this session? Unsaved work in open windows is lost.",
      suspend: "Suspend this computer?",
      restart: "Restart this computer?",
      shutdown: "Shut down this computer?"
    }[action] || "Continue?";
    const ok = await ask(t(confirmKey, fallbackConfirm));
    if (!ok) return;

    // Suspend/resume, lid-open and session-switch must never reveal an
    // unlocked desktop: lock first, then hand the action to /api/system/power.
    if (action === "suspend" || action === "restart" || action === "shutdown") {
      emitShell("gb-lock-request");
      if (window.LockScreen) window.LockScreen.lock("power-menu");
    }

    try {
      const headers = window.GBShared && window.GBShared.auth
        ? window.GBShared.auth.headers({ "Content-Type": "application/json" })
        : { "Content-Type": "application/json" };
      const response = await fetch("/api/system/power", {
        method: "POST",
        headers: headers,
        body: JSON.stringify({ action: action })
      });
      if (!response.ok) {
        const detail = await response.json().catch(function () { return null; });
        const message = detail && detail.error
          ? detail.error
          : t("power-menu-failed", "The power action failed.");
        await notify(message);
        return;
      }
      if (action === "signout") {
        try {
          localStorage.removeItem("gb-access-token");
          sessionStorage.removeItem("gb-access-token");
        } catch (e) {
          console.warn("power menu: could not clear the local session", e);
        }
        window.location.href = window.GB_LOGIN_URL || "/login";
        return;
      }
      await notify(t("power-menu-done", "Request accepted. The computer is shutting down now."));
    } catch (error) {
      await notify(
        t("power-menu-failed", "The power action failed.") + " " + (error && error.message ? error.message : "")
      );
    }
  }

  function ask(message) {
    if (window.GBModal && typeof window.GBModal.confirm === "function") {
      return new Promise(function (resolve) {
        window.GBModal.confirm(message, function () { resolve(true); }, function () { resolve(false); });
      });
    }
    return Promise.resolve(window.confirm(message));
  }

  function notify(message) {
    if (window.GBModal && typeof window.GBModal.alert === "function") {
      return window.GBModal.alert(message);
    }
    if (window.GBToasts && typeof window.GBToasts.show === "function") {
      window.GBToasts.show(t("power-menu-title", "Power"), message, "info");
      return Promise.resolve();
    }
    return Promise.resolve(window.alert(message));
  }

  /** Listens for the kiosk close request (#2450) once per page. */
  function init() {
    if (initialized || typeof window === "undefined") return;
    initialized = true;
    const tauri = window.__TAURI__;
    if (tauri && tauri.event && typeof tauri.event.listen === "function") {
      tauri.event.listen("gb-power-menu", function () { open(); });
    }
  }

  if (typeof document !== "undefined") {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", init);
    } else {
      init();
    }
  }

  return { init: init, open: open, close: close, run: run, isOpen: isOpen };
})();

window.PowerMenu = PowerMenu;
