(function () {
  "use strict";

  var INDICATORS = [
    { id: "trayBattery", key: "battery", labelKey: "taskbar-battery", defaultLabel: "Battery", statusKey: "taskbar-battery-none", defaultStatus: "No battery", priority: "high" },
    { id: "trayNetworkStatus", key: "network", labelKey: "taskbar-network", defaultLabel: "Network", statusKey: "taskbar-network-wired", defaultStatus: "Wired only", priority: "high" },
    { id: "trayVolume", key: "volume", labelKey: "taskbar-volume", defaultLabel: "Volume", statusKey: "taskbar-volume-unknown", defaultStatus: "Volume unavailable", priority: "med" },
    { id: "trayBrightness", key: "brightness", labelKey: "taskbar-brightness", defaultLabel: "Brightness", statusKey: "taskbar-brightness-unknown", defaultStatus: "Brightness unavailable", priority: "low" },
    { id: "trayStorage", key: "storage", labelKey: "taskbar-storage", defaultLabel: "Removable devices", statusKey: "taskbar-storage-none", defaultStatus: "No removable devices", priority: "low" },
    { id: "trayUpdate", key: "update", labelKey: "taskbar-updates", defaultLabel: "Updates", statusKey: "taskbar-updates-unknown", defaultStatus: "Update status unavailable", priority: "med" },
    { id: "trayPower", key: "power", labelKey: "taskbar-power", defaultLabel: "Power", statusKey: "taskbar-power-unknown", defaultStatus: "Power controls unavailable", priority: "low" }
  ];

  var powerState = {
    data: null,
    lastFetchOk: false,
    notified: {},
    pollTimer: null,
    pollMs: 30000
  };

  function t(key, fallback, params) {
    var v = fallback;
    if (typeof window._t === "function") {
      var r = window._t(key, fallback);
      if (r && r !== key) v = r;
    }
    if (params && v) {
      Object.keys(params).forEach(function (k) {
        v = String(v).split("{" + k + "}").join(String(params[k]));
      });
    }
    return v;
  }

  function edition() {
    var cfg = window.productConfig || {};
    var declared = String(cfg.edition || "").toLowerCase();
    if (declared === "cloud" || declared === "desktop" || declared === "os") return declared;
    if (cfg.shell_mode === true || cfg.no_security === true) return "os";
    if (window.GBTAURI || window.__TAURI_INTERNALS__) return "desktop";
    return "cloud";
  }

  function byId(id) { return document.getElementById(id); }

  function openSettings(target) {
    var section = target || "os";
    if (window.openDeepLink) {
      window.openDeepLink("settings", { section: section });
      return;
    }
    window.location.hash = "#" + section;
  }

  function closePopover() {
    var pop = byId("osTrayPopover");
    if (pop) pop.hidden = true;
    document.querySelectorAll(".os-tray-btn[aria-expanded='true']").forEach(function (b) {
      b.setAttribute("aria-expanded", "false");
    });
  }

  function indicatorByKey(key) {
    for (var i = 0; i < INDICATORS.length; i++) {
      if (INDICATORS[i].key === key) return INDICATORS[i];
    }
    return null;
  }

  function batteryStatusText(data) {
    if (!data || !data.present) return t("taskbar-battery-none", "No battery");
    var pct = typeof data.percent === "number" ? data.percent : null;
    var state = String(data.state || "").toLowerCase();
    var charging = state === "charging" || (state === "unknown" && data.ac_online && pct !== null && pct < 100);
    var full = state === "full" || (pct === 100 && data.ac_online);
    var discharging = state === "discharging";
    if (pct === null) {
      if (charging) return t("taskbar-battery-charging-unknown", "Charging");
      if (full) return t("taskbar-battery-full", "Fully charged");
      return t("taskbar-battery-present", "Battery present");
    }
    if (full) return t("taskbar-battery-full-pct", "{percent}% — fully charged", { percent: pct });
    if (charging) return t("taskbar-battery-charging-pct", "{percent}% — charging", { percent: pct });
    if (discharging) return t("taskbar-battery-discharging-pct", "{percent}% — on battery", { percent: pct });
    return t("taskbar-battery-pct", "{percent}% — {state}", { percent: pct, state: data.state || "unknown" });
  }

  function batteryLevelClass(data) {
    if (!data || !data.present || typeof data.percent !== "number") return "none";
    var state = String(data.state || "").toLowerCase();
    if (state === "charging") return "charging";
    if (state === "full" || data.percent === 100) return "full";
    if (data.percent <= 5) return "critical";
    if (data.percent <= 20) return "low";
    return "ok";
  }

  function powerControlStatusText(data) {
    if (!data) return t("taskbar-power-unknown", "Power controls unavailable");
    if (!data.present && data.ac_online) return t("taskbar-power-ac-only", "AC power");
    if (data.present && typeof data.percent === "number") {
      return t("taskbar-power-summary", "{percent}% battery", { percent: data.percent });
    }
    return t("taskbar-power-unknown", "Power controls unavailable");
  }

  function updateBatteryUi() {
    var btn = byId("trayBattery");
    var powerBtn = byId("trayPower");
    var data = powerState.data;
    var status = batteryStatusText(data);
    if (btn) {
      btn.title = status;
      btn.setAttribute("aria-label", t("taskbar-battery", "Battery") + ": " + status);
      btn.setAttribute("data-power-level", batteryLevelClass(data));
      var dot = btn.querySelector(".os-tray-dot");
      if (dot) {
        var warn = data && data.present && typeof data.percent === "number" && data.percent <= 20 && String(data.state || "").toLowerCase() !== "charging";
        dot.hidden = !warn;
      }
    }
    if (powerBtn) {
      var pstatus = powerControlStatusText(data);
      powerBtn.title = pstatus;
      powerBtn.setAttribute("aria-label", t("taskbar-power", "Power") + ": " + pstatus);
    }
    // keep overflow menu labels fresh on next rebuild
  }

  function triggerCriticalAction() {
    var action = "none";
    try { action = localStorage.getItem("gb-os-critical-action") || "none"; } catch (e) { /* ignore */ }
    if (!action || action === "none") return;
    if (action === "hibernate") action = "suspend"; // API exposes suspend; hibernate is logind-owned
    if (["suspend", "shutdown"].indexOf(action) < 0) return;
    var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    var headers = { "Content-Type": "application/json" };
    if (token) headers.Authorization = "Bearer " + token;
    fetch("/api/system/power", {
      method: "POST", headers: headers, credentials: "same-origin",
      body: JSON.stringify({ action: action })
    }).catch(function () { /* the notification already fired */ });
  }

  function maybeNotifyThresholds(data) {
    if (!data || !data.present || typeof data.percent !== "number") return;
    var state = String(data.state || "").toLowerCase();
    if (state === "charging" || state === "full") {
      // reset lower thresholds so they can fire again on next discharge
      if (data.percent > 20) powerState.notified = {};
      return;
    }
    [20, 10, 5].forEach(function (threshold) {
      if (data.percent <= threshold && !powerState.notified[threshold]) {
        powerState.notified[threshold] = true;
        var msg = t("taskbar-battery-low-notify", "Battery low: {percent}%", { percent: data.percent });
        if (threshold <= 5) msg = t("taskbar-battery-critical-notify", "Battery critical: {percent}%", { percent: data.percent });
        announce(msg);
        if (typeof window.Notification === "function" && window.Notification.permission === "granted") {
          try { new window.Notification(t("taskbar-battery", "Battery"), { body: msg }); } catch (e) { /* ignore */ }
        }
        // #2445 — at the critical level, run the user's configured action once
        // per discharge cycle through the audited power API.
        if (threshold === 5) triggerCriticalAction();
      }
    });
  }

  function announce(msg) {
    var live = byId("osTrayLiveStatus");
    if (live) {
      live.textContent = "";
      setTimeout(function () { live.textContent = msg; }, 20);
    }
  }

  function openPopover(ind, btn) {
    var pop = byId("osTrayPopover");
    if (!pop) return;
    closeOverflow();
    var title = byId("osTrayPopoverTitle");
    var status = byId("osTrayPopoverStatus");
    var settings = byId("osTrayPopoverSettings");
    if (title) title.textContent = t(ind.labelKey, ind.defaultLabel);
    if (status) {
      if (ind.key === "battery") status.textContent = batteryStatusText(powerState.data);
      else if (ind.key === "power") status.textContent = powerControlStatusText(powerState.data);
      else status.textContent = t(ind.statusKey, ind.defaultStatus);
    }
    if (settings) {
      settings.onclick = function () { closePopover(); openSettings(btn.getAttribute("data-settings-target") || "os"); };
    }
    // #2465 — the primary click also offers a real quick action where the
    // image has one (power controls, update check). Others keep the settings
    // link as the one-click action; the secondary click opens settings too.
    var action = byId("osTrayPopoverAction");
    if (action) {
      action.hidden = true;
      action.onclick = null;
      if (ind.key === "power" || ind.key === "battery") {
        if (window.PowerMenu && typeof window.PowerMenu.open === "function") {
          action.textContent = t("taskbar-power-controls", "Power controls");
          action.hidden = false;
          action.onclick = function () { closePopover(); window.PowerMenu.open(); };
        }
      } else if (ind.key === "storage") {
        action.textContent = t("taskbar-eject-removable", "Eject removable device");
        action.hidden = false;
        action.onclick = function () {
          closePopover();
          var dev = powerState.removable && powerState.removable[0];
          if (window.DriveRemovable && dev) window.DriveRemovable.eject(dev.name || dev.label);
          else if (window.openDeepLink) window.openDeepLink("drive", {});
        };
      } else if (ind.key === "update" && window.GBUpdate && typeof window.GBUpdate.checkNow === "function") {
        action.textContent = t("taskbar-check-updates", "Check now");
        action.hidden = false;
        action.onclick = function () { closePopover(); window.GBUpdate.checkNow(); };
      } else {
        // No OS-level toggle exists for network/volume/brightness on this
        // image; the one-click action is the matching settings section.
        action.textContent = t("taskbar-open-settings", "Open settings");
        action.hidden = false;
        action.onclick = function () { closePopover(); openSettings(btn.getAttribute("data-settings-target") || "os"); };
      }
    }
    pop.hidden = false;
    btn.setAttribute("aria-expanded", "true");
    var closeBtn = byId("osTrayPopoverClose");
    if (closeBtn) closeBtn.focus();
  }

  function togglePopover(ind, btn) {
    var pop = byId("osTrayPopover");
    if (pop && !pop.hidden && btn.getAttribute("aria-expanded") === "true") {
      closePopover();
    } else {
      openPopover(ind, btn);
    }
  }

  function closeOverflow() {
    var menu = byId("osTrayOverflowMenu");
    if (menu) menu.hidden = true;
    var ob = byId("trayOsOverflow");
    if (ob) ob.setAttribute("aria-expanded", "false");
  }

  function currentStatusKey(ind) {
    if (ind.key === "battery") return null;
    if (ind.key === "power") return null;
    return ind.statusKey;
  }

  function currentStatusText(ind) {
    if (ind.key === "battery") return batteryStatusText(powerState.data);
    if (ind.key === "power") return powerControlStatusText(powerState.data);
    return t(ind.statusKey, ind.defaultStatus);
  }

  function rebuildOverflow() {
    var tray = byId("taskbarTray");
    var group = byId("osTrayGroup");
    var overflowBtn = byId("trayOsOverflow");
    var menu = byId("osTrayOverflowMenu");
    if (!tray || !group || !overflowBtn || !menu || group.hidden) return;

    var compact = window.innerWidth <= 1280;
    tray.classList.toggle("is-tray-compact", compact);
    menu.innerHTML = "";

    var hidden = [];
    INDICATORS.forEach(function (ind) {
      var btn = byId(ind.id);
      if (!btn) return;
      btn.dataset.priority = ind.priority;
      var hide = false;
      if (window.innerWidth <= 1024 && (ind.priority === "low" || ind.priority === "med")) hide = true;
      else if (window.innerWidth <= 1280 && ind.priority === "low") hide = true;
      btn.classList.toggle("is-overflow-hidden", hide);
      if (hide) hidden.push(ind);
    });

    if (hidden.length > 0) {
      overflowBtn.hidden = false;
      hidden.forEach(function (ind) {
        var item = document.createElement("button");
        item.type = "button";
        item.className = "os-tray-overflow-item";
        item.setAttribute("role", "menuitem");
        item.textContent = t(ind.labelKey, ind.defaultLabel) + ": " + currentStatusText(ind);
        item.addEventListener("click", function () {
          closeOverflow();
          openSettings((byId(ind.id) && byId(ind.id).getAttribute("data-settings-target")) || "os");
        });
        menu.appendChild(item);
      });
    } else {
      overflowBtn.hidden = true;
      closeOverflow();
    }
  }

  function fetchPowerStatus() {
    if (edition() !== "os") return Promise.resolve();
    var urls = ["/api/system/power/status", "/api/system/os/power/status"];
    function tryUrl(i) {
      if (i >= urls.length) {
        powerState.lastFetchOk = false;
        updateBatteryUi();
        rebuildOverflow();
        return Promise.resolve();
      }
      return fetch(urls[i], { headers: { Accept: "application/json" }, credentials: "same-origin" })
        .then(function (r) {
          if (r.status === 404 && i + 1 < urls.length) return tryUrl(i + 1);
          if (!r.ok) throw new Error(String(r.status));
          return r.json();
        })
        .then(function (data) {
          if (data && typeof data === "object") {
            powerState.data = data;
            powerState.lastFetchOk = true;
            updateBatteryUi();
            maybeNotifyThresholds(data);
            fetch("/api/os/removable", { headers: { Accept: "application/json" }, credentials: "same-origin" })
              .then(function (r) { return r.ok ? r.json() : {}; })
              .then(function (d) {
                powerState.removable = (d.devices && d.devices.blockdevices) || [];
                rebuildOverflow();
              })
              .catch(function () { /* removable list is optional */ });
            rebuildOverflow();
            // refresh open popover if showing battery/power
            var pop = byId("osTrayPopover");
            if (pop && !pop.hidden) {
              var expanded = document.querySelector(".os-tray-btn[aria-expanded='true']");
              if (expanded) {
                var ind = indicatorByKey(expanded.getAttribute("data-os-key"));
                if (ind && (ind.key === "battery" || ind.key === "power")) {
                  var status = byId("osTrayPopoverStatus");
                  if (status) status.textContent = ind.key === "battery" ? batteryStatusText(powerState.data) : powerControlStatusText(powerState.data);
                }
              }
            }
          }
        })
        .catch(function () {
          if (i + 1 < urls.length) return tryUrl(i + 1);
          powerState.lastFetchOk = false;
          updateBatteryUi();
          rebuildOverflow();
        });
    }
    return tryUrl(0);
  }

  function startPolling() {
    if (powerState.pollTimer) return;
    fetchPowerStatus();
    powerState.pollTimer = setInterval(fetchPowerStatus, powerState.pollMs);
    document.addEventListener("visibilitychange", function () {
      if (!document.hidden) fetchPowerStatus();
    });
  }

  function wire() {
    var group = byId("osTrayGroup");
    if (!group) return;
    var show = edition() === "os";
    group.hidden = !show;
    if (!show) return;

    INDICATORS.forEach(function (ind) {
      var btn = byId(ind.id);
      if (!btn || btn.dataset.osWired === "1") return;
      btn.dataset.osWired = "1";
      btn.dataset.priority = ind.priority;
      btn.dataset.osKey = ind.key;
      btn.addEventListener("click", function (e) {
        e.preventDefault();
        togglePopover(ind, btn);
      });
      btn.addEventListener("contextmenu", function (e) {
        e.preventDefault();
        closePopover();
        openSettings(btn.getAttribute("data-settings-target") || "os");
      });
      btn.addEventListener("keydown", function (e) {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          togglePopover(ind, btn);
        } else if (e.key === "Escape") {
          closePopover();
          btn.focus();
        }
      });
    });

    var closeBtn = byId("osTrayPopoverClose");
    if (closeBtn && closeBtn.dataset.osWired !== "1") {
      closeBtn.dataset.osWired = "1";
      closeBtn.addEventListener("click", function () { closePopover(); });
    }
    var overflowBtn = byId("trayOsOverflow");
    if (overflowBtn && overflowBtn.dataset.osWired !== "1") {
      overflowBtn.dataset.osWired = "1";
      overflowBtn.addEventListener("click", function () {
        var menu = byId("osTrayOverflowMenu");
        if (!menu) return;
        var open = menu.hidden;
        closePopover();
        menu.hidden = !open;
        overflowBtn.setAttribute("aria-expanded", open ? "true" : "false");
        if (open) {
          var first = menu.querySelector(".os-tray-overflow-item");
          if (first) first.focus();
        }
      });
    }

    document.addEventListener("click", function (e) {
      var pop = byId("osTrayPopover");
      var menu = byId("osTrayOverflowMenu");
      var inTray = e.target && e.target.closest && e.target.closest(".os-tray-group");
      if (!inTray && pop && !pop.hidden && !(e.target.closest && e.target.closest("#osTrayPopover"))) closePopover();
      if (!inTray && menu && !menu.hidden && !(e.target.closest && e.target.closest("#osTrayOverflowMenu"))) closeOverflow();
    });
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape") { closePopover(); closeOverflow(); }
    });
    window.addEventListener("resize", rebuildOverflow);
    rebuildOverflow();
    startPolling();
  }

  // The media indicator is installed by 08b_os_tray_media.js, which defines
  // installMediaIndicator() and loads after this file. Calling it here threw
  // a ReferenceError that aborted the rest of this IIFE (overflow/polling).

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", wire);
  } else {
    wire();
  }
  window.addEventListener("load", wire);
})();
