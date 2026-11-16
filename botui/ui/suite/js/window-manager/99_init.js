/* WindowManager — public export and boot (9/9, last). */
(function (window) {
  "use strict";
  var WM = window.__GBWindowManager;
  if (!WM) return;
  var WindowManager = WM.WindowManager;

  if (typeof window.APPS_REGISTRY === "undefined") window.APPS_REGISTRY = WM.APPS_REGISTRY;
  window.GBPreviewMode = WM.GBPreviewMode;
  window.refreshAppsCatalog = WM.refreshAppsCatalog;

window.WindowManager = new WindowManager();
window.openDeepLink = (appId, params, opts) => window.WindowManager.openDeepLink(appId, params, opts);

// #2223 — window snapping shipped without a caller: nothing invoked
// GBSnapAssist.init(), so edge-drag ghosts and the snap layouts offered by the
// expand dot were dead. sidebar scripts load after this file, hence the defer.
function initSnapAssist() {
  if (window.GBSnapAssist && typeof window.GBSnapAssist.init === "function") {
    window.GBSnapAssist.init();
  }
}
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initSnapAssist, { once: true });
} else {
  initSnapAssist();
}

// Ctrl+K is owned by the unified command palette (command-palette.js) which
// is loaded after this file. Expose the start menu for launchers but do NOT
// bind Ctrl+K here — command-palette.js handles it with one handler only.

// #2411 — keep shell chrome height and window geometry in sync with the real
// taskbar/viewport (URL bar, rotation, on-screen keyboard). This makes
// --gb-chrome-bottom authoritative and reclamps windows on viewport changes.
(function initViewportShellSync() {
  function getTaskbarHeight() {
    var tb = document.getElementById("taskbar") || document.querySelector(".toolbar-v15");
    if (!tb) return 30;
    var rect = tb.getBoundingClientRect();
    if (rect && rect.height) return Math.round(rect.height);
    return 30;
  }

  function syncChromeBottom() {
    var h = getTaskbarHeight();
    try {
      document.documentElement.style.setProperty("--gb-chrome-bottom", h + "px");
    } catch (_e) {
      /* ignore */
    }
    return h;
  }

  function reclampAllWindows() {
    var wm = window.WindowManager;
    if (!wm || !Array.isArray(wm.openWindows)) return;
    wm.openWindows.forEach(function (w) {
      if (!w || !w.id) return;
      if (wm.isFullscreen && wm.isFullscreen(w.id)) return;
      if (typeof wm._clampWindowIntoView === "function") wm._clampWindowIntoView(w.id);
    });
  }

  function keepFocusedVisible() {
    try {
      var ae = document.activeElement;
      if (!ae || ae === document.body || ae === document.documentElement) return;
      var tag = (ae.tagName || "").toLowerCase();
      if (tag !== "input" && tag !== "textarea" && tag !== "select" && !ae.isContentEditable) return;
      if (typeof ae.scrollIntoView === "function") {
        ae.scrollIntoView({ block: "nearest", inline: "nearest" });
      }
    } catch (_e) {}
  }

  function reclampSoon() {
    reclampAllWindows();
    keepFocusedVisible();
    [0, 60, 120, 220, 400, 700, 1100].forEach(function (ms) {
      setTimeout(function () {
        reclampAllWindows();
        keepFocusedVisible();
      }, ms);
    });
    if (typeof requestAnimationFrame === "function") {
      requestAnimationFrame(function () {
        reclampAllWindows();
        keepFocusedVisible();
        requestAnimationFrame(function () {
          reclampAllWindows();
          keepFocusedVisible();
        });
      });
    }
  }

  function onViewportChange() {
    syncChromeBottom();
    reclampSoon();
  }

  function bindViewportSync() {
    syncChromeBottom();
    window.addEventListener("resize", onViewportChange, { passive: true });
    window.addEventListener("orientationchange", function () {
      setTimeout(onViewportChange, 0);
      setTimeout(onViewportChange, 120);
      setTimeout(onViewportChange, 350);
    }, { passive: true });
    if (window.visualViewport) {
      window.visualViewport.addEventListener("resize", onViewportChange, { passive: true });
      window.visualViewport.addEventListener("scroll", onViewportChange, { passive: true });
    }
    var tb = document.getElementById("taskbar") || document.querySelector(".toolbar-v15");
    if (tb && typeof ResizeObserver === "function") {
      try {
        var ro = new ResizeObserver(onViewportChange);
        ro.observe(tb);
      } catch (_e) {
        /* ResizeObserver unavailable */
      }
    }
    // Initial post-layout clamp for auto-opened windows.
    setTimeout(onViewportChange, 0);
    setTimeout(onViewportChange, 250);
    setTimeout(onViewportChange, 800);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bindViewportSync, { once: true });
  } else {
    bindViewportSync();
  }
})();

// Chat sidebar collapse — the choice is remembered (localStorage) so the
// sidebar stays collapsed across reloads and app switches. The collapsed
// rail (51px) keeps the toggle button reachable.
const SIDEBAR_STATE_KEY = "gb.chatSidebar.collapsed";

window.toggleChatSidebar = function () {
  const sidebar = document.getElementById("chatSidebar");
  if (!sidebar) return;
  const collapsed = sidebar.classList.toggle("collapsed");
  try {
    localStorage.setItem(SIDEBAR_STATE_KEY, collapsed ? "1" : "0");
  } catch (e) {
    /* storage may be disabled; the toggle still works for the session */
  }
  // The rail toggle is an explicit show-sidebar interaction: keep the
  // "Show Sidebar" setting (gb-show-sidebar / body.sidebar-hidden) in sync
  // so settings and the rail never ignore each other (#1810).
  try {
    localStorage.setItem("gb-show-sidebar", "true");
  } catch (e) {}
  document.body.classList.remove("sidebar-hidden");
  window.dispatchEvent(new CustomEvent("gb-sidebar-visibility-changed", {
    detail: { show: true },
  }));
};

// Restore the remembered state on boot. window-manager.js loads in the
// <head>, before desktop.html's sidebar exists, so defer until the DOM is
// ready — and also re-apply after HTMX swaps that re-create the shell.
function restoreChatSidebar() {
  const sidebar = document.getElementById("chatSidebar");
  if (!sidebar) return;
  let collapsed = "1";
  try {
    // Default to the collapsed rail: the desktop chat is the first screen and
    // the full sidebar covered most of it on small displays. A user who
    // expands it keeps that choice ("0") across reloads.
    collapsed = localStorage.getItem(SIDEBAR_STATE_KEY) || "1";
  } catch (e) {
    /* storage unavailable — leave default */
  }
  sidebar.classList.toggle("collapsed", collapsed !== "0");
  // #1433 — the Settings "show sidebar" toggle persists to a separate key;
  // honor it here so a previously-hidden rail stays hidden across reloads.
  let sidebarVisible = "true";
  try {
    sidebarVisible = localStorage.getItem("gb-show-sidebar") || "true";
  } catch (e) {
    /* storage unavailable — default to visible */
  }
  document.body.classList.toggle("sidebar-hidden", sidebarVisible === "false");
}
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", function () {
    // Apply the remembered state on first paint AND keep it applied across
    // later HTMX swaps that re-create the shell. The sidebar lives in the
    // initial desktop.html markup (not a swapped partial), so restoring only
    // on htmx:afterSwap misses loads where no swap occurs.
    restoreChatSidebar();
    restoreCompactMode();
    hookChatSidebarAfterSwap();
    hookCompactModeAfterSwap();
  });
} else {
  restoreChatSidebar();
  restoreCompactMode();
  hookChatSidebarAfterSwap();
  hookCompactModeAfterSwap();
}
// window-manager.js loads in the <head>, before <body> exists, so attach
// the htmx listener only once the body element is present (binding it at
// parse time would throw a null addEventListener error in the console).
function hookChatSidebarAfterSwap() {
  if (window.htmx && document.body) {
    document.body.addEventListener("htmx:afterSwap", restoreChatSidebar);
    return;
  }
  if (document.readyState !== "loading") document.addEventListener("DOMContentLoaded", hookChatSidebarAfterSwap);
}

function restoreCompactMode() {
  let compact = "false";
  try {
    compact = localStorage.getItem("gb-compact-mode") || "false";
  } catch (e) {
    /* storage unavailable — default off */
  }
  document.body.classList.toggle("compact-mode", compact === "true");
}

function hookCompactModeAfterSwap() {
  if (window.htmx && document.body) {
    document.body.addEventListener("htmx:afterSwap", restoreCompactMode);
    return;
  }
  if (document.readyState !== "loading") document.addEventListener("DOMContentLoaded", hookCompactModeAfterSwap);
}
window.addEventListener("gb-compact-mode-changed", function (e) {
  var enabled = !!(e && e.detail && e.detail.compact);
  document.body.classList.toggle("compact-mode", enabled);
});

})(window);
