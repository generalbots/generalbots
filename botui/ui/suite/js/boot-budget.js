"use strict";

/**
 * boot-budget.js — deterministic post-first-paint loader for the suite shell
 * (issue #2495). One public entry point: GBBootBudget.init(scope).
 *
 * BUDGET (target, not a measured claim): <= 350 KB compressed JavaScript and
 * <= 120 KB CSS before first interaction at 390px. This file does not assert
 * Lighthouse numbers — the driver records before/after measurements in the
 * verification issue. What this file guarantees structurally:
 *   - no new parser-blocking <script src> tags;
 *   - optional surfaces are skipped entirely when the measured viewport is
 *     compact, so they cost zero bytes and zero parse time there;
 *   - on desktop the exact same modules still load, in their original order,
 *     immediately after first paint and before any realistic first input.
 *
 * FORM FACTOR: exclusively matchMedia('(max-width: 767px)').matches — measured
 * width only. No user-agent / device sniffing, no vendor strings. Desktop
 * (>= 768px) loads every deferred module; compact skips all of them.
 *
 * CORE — stays synchronous, original order preserved, never deferred:
 *   desktop.html: 01_boot_flags.js (AUTH GATE — first script in <head>),
 *     01_auth_gate.css, app/base/theme/config/identity/save-menu CSS,
 *     desktop/*.css + screensaver.css, htmx.min.js, gb-shared_a/b, gb-icons,
 *     i18n.js, app-i18n.js, i18n-format.js, tour.js, security-bootstrap/*,
 *     marked.min.js, window-manager/01..99 (incl. 99_init.js), css-loader,
 *     vibe-bridge, theme-scheme/manager, command-palette, sidebar and launcher
 *     modules, app-lifecycle, pinned-launcher, desktop-shortcuts, sidebar-enhance,
 *     gb-toasts, gb-alerts, gb-modal, gb-datetime, agent-executor/control,
 *     proactivity, workspace-tabs, desktop-inline/02..07, setup-wizard*,
 *     ui-orchestrator/*, external-save, gb-search, simple-context-menu,
 *     widget-renderer.js + widget-registry.js (GBWidgets.init() is called by
 *     desktop-inline/04_auth_fullscreen_a.js at boot and the context menu
 *     offers "Add widget" — both would become dead on a phone).
 *   chat.html: marked.min.js, katex.min.js, chat-math.js, chat-state,
 *     chat-switchers, chat-mentions, external-save, artifact-download,
 *     chat-sanitize, chat-render, chat-turn-indicator/state, messages/10..70
 *     (transcript + composer), chat-output-polish, chat-suggestions,
 *     chat-export, conversation-header, edit-last, feedback, staged-files,
 *     chat-theme, chat-websocket_a/b, chat-session, chat-send_a/b,
 *     chat-init_a/b, chat-model-picker, chat-agent-mode, gb-search, tour,
 *     45_web_search_toggle / 46_chat_controls (composer payload defaults).
 *
 * DEFERRED — injected in ORIGINAL ORDER after first paint, skipped in compact:
 *   desktop: screensavers.js, snap-assist.js, virtual-desktops.js,
 *            control-center.js, lock-screen.js, spotlight.js.
 *   chat: chart.umd.min.js, chat-artifacts.js, chat-live-artifact-svg.js,
 *         chat-artifact-overlay.js, chat-live-artifact.js, 20_tabs*, 21_tabs*,
 *         multichat/01..07, vm_attachments/01..06_hydrate, 71_run_timeline*,
 *         72_surface_*, 74_viz, 73_app_frame, 74_subagents,
 *         76_voice*, 77_camera*, 78_voice_mode.
 *
 * DELIBERATELY NOT DEFERRED (regression risk, see report):
 *   - mission-control.js, alt-tab.js, taskbar-previews.js bind their keyboard
 *     handlers ONLY from a DOMContentLoaded listener registered when the file
 *     executes. A post-first-paint injector runs after that event, so the
 *     shortcuts would silently die. They stay inline on desktop and compact.
 *   - window-manager, security-bootstrap, i18n, auth gate: core boot path.
 *
 * No document.write, no eval, no CDN, no async attribute on inline tags.
 */
(function (global) {
  var COMPACT_QUERY = "(max-width: 767px)";
  var doc = global.document;

  var DEFERRED = {
    desktop: [
      "/suite/js/screensavers.js?v=2",
      "/suite/js/snap-assist.js?v=1",
      "/suite/js/virtual-desktops.js?v=1",
      "/suite/js/control-center.js?v=1",
      "/suite/js/lock-screen.js?v=2",
      // #2444 — the power menu depends on the lock screen and on a Tauri event.
      "/suite/js/power-menu.js?v=3",
      // #2450 — branded restart/recovery surface for kiosk webview restarts.
      "/suite/js/restarting-screen.js?v=1",
      "/suite/js/spotlight.js?v=1"
    ],
    chat: [
      "/suite/js/vendor/chart.umd.min.js",
      "/suite/chat/chat-artifacts.js?v=5",
      "/suite/chat/chat-live-artifact-svg.js?v=10",
      "/suite/chat/chat-artifact-overlay.js?v=1",
      "/suite/chat/chat-live-artifact.js?v=13",
      "/suite/chat/modules/20_tabs.js?v=10",
      "/suite/chat/modules/20_tabs_render.js?v=10",
      "/suite/chat/modules/21_tabs_events.js?v=10",
      "/suite/chat/modules/21_tabs_events_boot.js?v=10",
      "/suite/chat/modules/multichat/01_state.js?v=64",
      "/suite/chat/modules/multichat/02_dom.js?v=68",
      "/suite/chat/modules/multichat/03_tabs.js?v=64",
      "/suite/chat/modules/multichat/04_socket.js?v=65",
      "/suite/chat/modules/multichat/05_send.js?v=65",
      "/suite/chat/modules/multichat/06_events.js?v=65",
      "/suite/chat/modules/multichat/07_boot.js?v=66",
      "/suite/chat/modules/vm_attachments/01_state_mime.js?v=23",
      "/suite/chat/modules/vm_attachments/02_links.js?v=23",
      "/suite/chat/modules/vm_attachments/03_preview.js?v=24",
      "/suite/chat/modules/vm_attachments/04_bind.js?v=24",
      "/suite/chat/modules/vm_attachments/05_folder.js?v=25",
      "/suite/chat/modules/vm_attachments/05a_external_save.js?v=1",
      "/suite/chat/modules/vm_attachments/06_hydrate.js?v=24",
      "/suite/chat/modules/71_run_timeline.js?v=11",
      "/suite/chat/modules/71_run_timeline_rows.js?v=11",
      "/suite/chat/modules/71_run_timeline_frames.js?v=12",
      "/suite/chat/modules/72_surface_renderer.js?v=4",
      "/suite/chat/modules/72_surface_forms.js?v=5",
      "/suite/chat/modules/72_surface_cards.js?v=6",
      "/suite/chat/modules/74_viz.js?v=1",
      "/suite/chat/modules/73_app_frame.js?v=8",
      "/suite/chat/modules/74_subagents.js?v=1",
      "/suite/chat/modules/76_voice.js?v=5",
      "/suite/chat/modules/76_voice_duplex_audio.js?v=1",
      "/suite/chat/modules/76_voice_duplex_a.js?v=3",
      "/suite/chat/modules/76_voice_duplex_b.js?v=3",
      "/suite/chat/modules/77_camera_preview.js?v=1",
      "/suite/chat/modules/77_camera.js?v=3",
      "/suite/chat/modules/78_voice_mode.js?v=4"
    ]
  };

  /*
   * The desktop modules below are normalised by core code at DOMContentLoaded
   * via guarded `if (window.X) X.init()` calls. When they load later, those
   * guarded calls find no global; calling init() here re-establishes exactly
   * the same state. Every one of these init() functions is idempotent, so the
   * two orders (core first, helper second — or helper only) converge.
   * Chat modules need no post-init: they self-bind via readyState checks and
   * the `gb-chat-ready` event.
   */
  var POST_INIT = {
    desktop: [
      function () {
        if (global.GBSnapAssist && typeof global.GBSnapAssist.init === "function") {
          global.GBSnapAssist.init();
        }
      },
      function () {
        if (global.VirtualDesktops && typeof global.VirtualDesktops.init === "function") {
          global.VirtualDesktops.init(doc.getElementById("desktop-content"));
        }
      },
      function () {
        if (global.ControlCenter && typeof global.ControlCenter.init === "function") {
          global.ControlCenter.init();
        }
      },
      function () {
        if (global.LockScreen && typeof global.LockScreen.init === "function") {
          global.LockScreen.init();
        }
      },
      function () {
        if (global.Spotlight && typeof global.Spotlight.init === "function") {
          global.Spotlight.init();
        }
      }
    ],
    chat: []
  };

  /* Gated controls. The elements keep their existing translated aria-label /
   * data-i18n-aria-label, so a disabled affordance still has an accessible
   * name. No new user-facing string is introduced (no new i18n key needed).
   */
  var COMPACT_TARGETS = {
    desktop: ["#trayLock"],
    chat: ["#cameraBtn", "#voiceBtn", "#voiceDuplexBtn"]
  };

  var state = {
    desktop: { injected: false, ready: false },
    chat: { injected: false, ready: false }
  };

  function isCompact() {
    return !!(global.matchMedia && global.matchMedia(COMPACT_QUERY).matches);
  }

  function afterFirstPaint(callback) {
    if (typeof global.requestAnimationFrame === "function") {
      global.requestAnimationFrame(function () {
        global.requestAnimationFrame(callback);
      });
    } else {
      global.setTimeout(callback, 0);
    }
  }

  function gate(selector) {
    var nodes = doc.querySelectorAll(selector);
    for (var i = 0; i < nodes.length; i++) {
      var el = nodes[i];
      if (el.getAttribute("data-gb-boot-gated") === "1") continue;
      el.setAttribute("data-gb-boot-gated", "1");
      el.setAttribute("aria-disabled", "true");
      if ("disabled" in el) el.disabled = true;
      el.style.opacity = "0.45";
      el.style.cursor = "not-allowed";
    }
  }

  function applyCompactAffordances(scope) {
    if (!isCompact()) return;
    var selectors = COMPACT_TARGETS[scope] || [];
    for (var i = 0; i < selectors.length; i++) gate(selectors[i]);
  }

  function announce(scope, mode) {
    try {
      doc.documentElement.setAttribute("data-gb-boot-budget", scope + ":" + mode);
      global.dispatchEvent(new global.CustomEvent("gb-boot-budget-ready", {
        detail: { scope: scope, mode: mode }
      }));
    } catch (e) {
      /* older hosts without CustomEvent still get the data attribute above */
    }
  }

  function runPostInit(scope) {
    var actions = POST_INIT[scope] || [];
    for (var i = 0; i < actions.length; i++) {
      try {
        actions[i]();
      } catch (e) {
        if (global.console && global.console.warn) {
          global.console.warn("[boot-budget] post-init failed for " + scope, e);
        }
      }
    }
    state[scope].ready = true;
    announce(scope, "deferred");
  }

  function injectInOrder(urls, scope) {
    var index = 0;

    function next() {
      if (index >= urls.length) {
        runPostInit(scope);
        return;
      }
      var url = urls[index++];
      var el = doc.createElement("script");
      el.src = url;
      el.async = false;
      el.onload = next;
      el.onerror = function () {
        if (global.console && global.console.warn) {
          global.console.warn("[boot-budget] optional module failed to load: " + url);
        }
        next();
      };
      (doc.head || doc.body).appendChild(el);
    }

    next();
  }

  function onReady(scope) {
    applyCompactAffordances(scope);
    if (isCompact()) {
      state[scope].ready = true;
      announce(scope, "compact");
      return;
    }
    if (state[scope].injected) return;
    state[scope].injected = true;
    afterFirstPaint(function () {
      injectInOrder(DEFERRED[scope] || [], scope);
    });
  }

  /** Single public entry point. Called by the shell for 'desktop' and by the
   * chat fragment (and the htmx:afterSwap hook below) for 'chat'. Safe to call
   * repeatedly: injection happens once per scope, gating is re-applied so a
   * freshly swapped chat window is gated too. */
  function init(scope) {
    if (!state[scope] || !doc) return;
    if (doc.readyState === "loading") {
      doc.addEventListener("DOMContentLoaded", function () {
        onReady(scope);
      }, { once: true });
    } else {
      onReady(scope);
    }
  }

  /* htmx swaps chat.html in after the shell has booted. Registering the hook
   * here (rather than relying on the fragment's inline script being executed
   * by htmx) makes chat deferral deterministic. */
  function hookChatSwap() {
    if (!doc.body || !global.htmx) {
      if (doc.readyState !== "loading") return;
      doc.addEventListener("DOMContentLoaded", hookChatSwap, { once: true });
      return;
    }
    doc.body.addEventListener("htmx:afterSwap", function (event) {
      var target = event && event.detail && event.detail.target;
      var fragment = target && target.querySelector
        ? target.querySelector("#chat-app")
        : null;
      if (fragment || (target && target.id === "chat-app")) init("chat");
    });
  }

  global.GBBootBudget = {
    init: init,
    isCompact: isCompact,
    deferred: DEFERRED
  };

  hookChatSwap();
})(window);
