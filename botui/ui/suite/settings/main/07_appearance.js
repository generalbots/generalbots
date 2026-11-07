/* Settings module 07: appearance, wallpaper, push and section handlers. */
(function (window) {
  "use strict";
  var S = window.__GBSettingsMain;
  var STORAGE_KEYS = S.STORAGE_KEYS;
  var getAuthToken = S.getAuthToken;
  var showToast = S.showToast;

function hasUnsavedSettingsChanges() {
  try {
    if (window.__gbSettingsDirty) return true;
    if (typeof window.gbHasUnsavedChanges === "function" && window.gbHasUnsavedChanges()) {
      return true;
    }
    if (document.querySelector('[data-dirty="true"], .is-dirty, .settings-dirty, .unsaved')) {
      return true;
    }
  } catch (e) {}
  return false;
}

function confirmLanguageReload(locale) {
  if (!hasUnsavedSettingsChanges()) return true;
  var languageName = locale || "the selected language";
  return window.confirm(
    "Changing the language to " +
      languageName +
      " will reload the app. Unsaved changes may be lost. Continue?"
  );
}

window.changeLanguage = function (locale) {
  if (!confirmLanguageReload(locale)) return;
  localStorage.setItem(STORAGE_KEYS.LOCALE, locale);
  document.documentElement.lang = locale;
  showToast("Language changed");
  setTimeout(function () {
    location.reload();
  }, 500);
};
window.selectLanguage = function (locale, element) {
  if (!confirmLanguageReload(locale)) return;
  document.querySelectorAll(".language-option").forEach(function (o) {
    o.classList.remove("active");
  });
  if (element) element.classList.add("active");
  localStorage.setItem(STORAGE_KEYS.LOCALE, locale);
  document.documentElement.lang = locale;
  showToast("Language changed");
  setTimeout(function () {
    location.reload();
  }, 500);
};
window.changeDateFormat = function (format) {
  localStorage.setItem(STORAGE_KEYS.DATE_FORMAT, format);
  showToast("Date format updated");
};
window.changeTimeFormat = function (format) {
  localStorage.setItem(STORAGE_KEYS.TIME_FORMAT, format);
  showToast("Time format updated");
};
function currentBotId() {
  var path = window.location.pathname || "";
  var match = path.match(/(?:\/bot\/)?([a-z0-9-]+)/i);
  return match && match[1] ? match[1].toLowerCase() : "default";
}

window.setTheme = function (theme, element) {
  // Apply on <html> so the desktop shell CSS variables react, not just the
  // settings fragment body.
  document.documentElement.setAttribute("data-theme", theme);
  document.body.setAttribute("data-theme", theme);
  localStorage.setItem(STORAGE_KEYS.THEME, theme);
  // Mirror into the per-bot key the desktop theme-manager reads, so the
  // choice survives navigation and applies to the shell.
  localStorage.setItem("gb-theme-" + currentBotId(), theme);
  document.querySelectorAll(".theme-option").forEach(function (o) {
    o.classList.remove("active");
  });
  if (element) element.classList.add("active");
  showToast("Theme updated");
  window.dispatchEvent(
    new CustomEvent("gb-theme-changed", { detail: { theme: theme } }),
  );
};

var WALLPAPERS = [
  { id: "default", name: "Default", css: "linear-gradient(135deg, #0a0a0a 0%, #1a2338 50%, #0a0a0a 100%)" },
  { id: "mountains", name: "Mountains", css: "linear-gradient(180deg, #7ea8d8 0%, #b8d4ef 35%, #6b96c9 70%, #3b5a8c 100%)" },
  { id: "clean", name: "Clean", css: "linear-gradient(160deg, #f5f7fa 0%, #e8ecf1 55%, #d6dde6 100%)" },
  { id: "ocean", name: "Ocean", css: "linear-gradient(180deg, #0a2e4d 0%, #0d4470 45%, #061c30 100%)" },
  { id: "forest", name: "Forest", css: "linear-gradient(180deg, #102b1a 0%, #1d4d2e 40%, #081a0f 100%)" },
  { id: "sunset", name: "Sunset", css: "radial-gradient(circle at 50% 85%, #ffd166 0%, #ef476f 48%, #2d2440 100%)" },
  { id: "night", name: "Night Sky", css: "linear-gradient(180deg, #0b1026 0%, #131a3a 50%, #0a0e22 100%)" },
  { id: "aurora", name: "Aurora", css: "linear-gradient(180deg, #060a1a 0%, #0d1530 55%, #0a0e22 100%)" },
  { id: "retro", name: "Retro", css: "linear-gradient(180deg, #f72585 0%, #7209b7 45%, #190c38 100%)" },
  { id: "paper", name: "Paper", css: "linear-gradient(180deg, #fbfaf6 0%, #f4f1e8 60%, #eae5d8 100%)" },
];

function getWallpaper() {
  return localStorage.getItem(STORAGE_KEYS.WALLPAPER) || "default";
}

function applyWallpaper(name) {
  document.body.setAttribute("data-wallpaper", name);
  var grid = document.getElementById("wallpaper-grid");
  if (grid) {
    Array.prototype.forEach.call(
      grid.querySelectorAll(".wallpaper-option"),
      function (opt) {
        opt.classList.toggle("active", opt.dataset.wallpaper === name);
      },
    );
  }
}

window.setWallpaper = function (name, element) {
  localStorage.setItem(STORAGE_KEYS.WALLPAPER, name);
  applyWallpaper(name);
  // Apply to the desktop shell too (same-tab windows don't receive the
  // `storage` event).
  document.body.setAttribute("data-wallpaper", name);
  showToast("Wallpaper updated");
  window.dispatchEvent(
    new CustomEvent("gb-wallpaper-changed", { detail: { name: name } }),
  );
};

function initWallpapers() {
  var grid = document.getElementById("wallpaper-grid");
  if (!grid) return;
  var current = getWallpaper();
  grid.innerHTML = WALLPAPERS.map(function (w) {
    var check =
      '<span class="wallpaper-check"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"></polyline></svg></span>';
    return (
      '<button type="button" class="wallpaper-option' +
      (w.id === current ? " active" : "") +
      '" data-wallpaper="' +
      w.id +
      '" onclick="setWallpaper(\'' +
      w.id +
      "', this)\" title=\"" +
      w.name +
      '"><span class="wallpaper-thumb" style="--wall:' +
      w.css +
      ";--wall-vignette:transparent\"></span>" +
      check +
      '<span class="wallpaper-name">' +
      w.name +
      "</span></button>"
    );
  }).join("");
}
window.toggleCompactMode = function (checkbox) {
  var enabled = checkbox.checked;
  localStorage.setItem(STORAGE_KEYS.COMPACT_MODE, enabled);
  document.body.classList.toggle("compact-mode", enabled);
  showToast(enabled ? "Compact mode enabled" : "Compact mode disabled");
};
window.toggleAnimations = function (checkbox) {
  var enabled = checkbox.checked;
  localStorage.setItem(STORAGE_KEYS.ANIMATIONS, enabled);
  document.body.classList.toggle("no-animations", !enabled);
  showToast(enabled ? "Animations enabled" : "Animations disabled");
};
window.changeTimezone = function (tz) {
  if (!tz) return;
  localStorage.setItem("gb-timezone", tz);
  showToast("Timezone updated");
};
window.changeFirstDay = function (day) {
  if (!day) return;
  localStorage.setItem("gb-first-day", day);
  showToast("First day of week updated");
};
window.toggleShowSidebar = function (checkbox) {
  var enabled = checkbox.checked;
  localStorage.setItem("gb-show-sidebar", enabled);
  document.body.classList.toggle("sidebar-hidden", !enabled);
  showToast(enabled ? "Sidebar shown" : "Sidebar hidden");
  window.dispatchEvent(new CustomEvent("gb-sidebar-visibility-changed", {
    detail: { show: enabled },
  }));
};
window.togglePushNotifications = function (checkbox) {
  var enabled = checkbox.checked;
  localStorage.setItem("gb-push-enabled", enabled);
  if (enabled) {
    requestPushPermissionAndSubscribe().catch(function () {});
  } else {
    unregisterPushSubscription().catch(function () {});
  }
  // Persist the whole preferences form to the server (best effort).
  var form = checkbox.closest("form");
  if (form) {
    var params = new URLSearchParams(new FormData(form));
    fetch("/api/user/notifications/preferences", {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + getAuthToken(),
      },
      body: JSON.stringify(Object.fromEntries(params.entries())),
    }).catch(function () {});
  }
  showToast(enabled ? "Push notifications enabled" : "Push notifications disabled");
};
function pushManagerAvailable() {
  return (
    "serviceWorker" in navigator &&
    "PushManager" in window &&
    "Notification" in window
  );
}
function fetchVapidKey() {
  return fetch("/api/notifications/push/vapid-key")
    .then(function (r) { return r.ok ? r.json() : null; })
    .then(function (data) { return data && data.key ? data.key : null; })
    .catch(function () { return null; });
}
function requestPushPermissionAndSubscribe() {
  if (!pushManagerAvailable()) {
    showToast("Push notifications are not supported in this browser");
    return Promise.resolve();
  }
  // #1816: fetch the VAPID public key from the server first.  When no key is
  // configured (common in the desktop-local build) we skip the subscription
  // with a friendly message instead of crashing on null applicationServerKey.
  return fetchVapidKey().then(function (vapidKey) {
    if (!vapidKey) {
      showToast("Push notifications are not yet available in the desktop edition");
      return Promise.resolve();
    }
    return Notification.requestPermission().then(function (perm) {
      if (perm !== "granted") {
        showToast("Push notifications blocked by the browser");
        return Promise.resolve();
      }
      return navigator.serviceWorker.ready.then(function (reg) {
        return reg.pushManager
          .subscribe({ userVisibleOnly: true, applicationServerKey: vapidKey })
          .then(function (sub) {
            return fetch("/api/notifications/push/register", {
              method: "POST",
              headers: {
                "Content-Type": "application/json",
                Authorization: "Bearer " + getAuthToken(),
              },
              body: JSON.stringify(sub),
            });
          });
      });
    });
  }).catch(function (err) {
    showToast("Push subscription failed: " + (err.message || "unknown error"));
    return Promise.resolve();
  });
}
function unregisterPushSubscription() {
  if (!pushManagerAvailable()) return Promise.resolve();
  return navigator.serviceWorker.ready.then(function (reg) {
    return reg.pushManager.getSubscription().then(function (sub) {
      if (!sub) return Promise.resolve();
      return fetch("/api/notifications/push/unregister", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: "Bearer " + getAuthToken(),
        },
        body: JSON.stringify(sub),
      })
        .then(function () {
          return sub.unsubscribe();
        })
        .catch(function () {
          return sub.unsubscribe();
        });
    });
  });
}
  window.addEventListener("gb-sidebar-visibility-changed", function (e) {
    var show = !!(e && e.detail && e.detail.show);
    var input = document.querySelector('[name="show_sidebar"]');
    if (input) input.checked = show;
  });

  S.getWallpaper = getWallpaper;
  S.applyWallpaper = applyWallpaper;
  S.initWallpapers = initWallpapers;
})(window);
