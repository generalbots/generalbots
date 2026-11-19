(function () {
  "use strict";

  var INDICATORS = [
    { id: "trayBattery", key: "battery", labelKey: "taskbar-battery", defaultLabel: "Battery", statusKey: "taskbar-battery-none", defaultStatus: "No battery", priority: "high" },
    { id: "trayNetworkStatus", key: "network", labelKey: "taskbar-network", defaultLabel: "Network", statusKey: "taskbar-network-wired", defaultStatus: "Wired only", priority: "high" },
    { id: "trayVolume", key: "volume", labelKey: "taskbar-volume", defaultLabel: "Volume", statusKey: "taskbar-volume-unknown", defaultStatus: "Volume unavailable", priority: "med" },
    { id: "trayBrightness", key: "brightness", labelKey: "taskbar-brightness", defaultLabel: "Brightness", statusKey: "taskbar-brightness-unknown", defaultStatus: "Brightness unavailable", priority: "low" },
    { id: "trayUpdate", key: "update", labelKey: "taskbar-updates", defaultLabel: "Updates", statusKey: "taskbar-updates-unknown", defaultStatus: "Update status unavailable", priority: "med" },
    { id: "trayPower", key: "power", labelKey: "taskbar-power", defaultLabel: "Power", statusKey: "taskbar-power-unknown", defaultStatus: "Power controls unavailable", priority: "low" }
  ];

  function t(key, fallback) {
    if (typeof window._t === "function") {
      var v = window._t(key, fallback);
      if (v && v !== key) return v;
    }
    return fallback;
  }

  function edition() {
    var cfg = window.productConfig || {};
    var declared = String(cfg.edition || "").toLowerCase();
    if (declared === "cloud" || declared === "desktop" || declared === "os") return declared;
    if (cfg.shell_mode === true || cfg.no_security === true) return "os";
    if (window.__TAURI__ || window.__TAURI_INTERNALS__) return "desktop";
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

  function openPopover(ind, btn) {
    var pop = byId("osTrayPopover");
    if (!pop) return;
    closeOverflow();
    var title = byId("osTrayPopoverTitle");
    var status = byId("osTrayPopoverStatus");
    var settings = byId("osTrayPopoverSettings");
    if (title) title.textContent = t(ind.labelKey, ind.defaultLabel);
    if (status) status.textContent = t(ind.statusKey, ind.defaultStatus);
    if (settings) {
      settings.onclick = function () { closePopover(); openSettings(btn.getAttribute("data-settings-target") || "os"); };
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
        item.textContent = t(ind.labelKey, ind.defaultLabel) + ": " + t(ind.statusKey, ind.defaultStatus);
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
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", wire);
  } else {
    wire();
  }
  window.addEventListener("load", wire);
})();