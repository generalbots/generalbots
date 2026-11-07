/* Settings module 05: navigation, toggles and saved preferences. */
(function (window) {
  "use strict";
  var S = window.__GBSettingsMain;
  var STORAGE_KEYS = S.STORAGE_KEYS;

function bindNavigation() {
  document.querySelectorAll(".settings-nav-item").forEach(function (item) {
    item.addEventListener("click", function (e) {
      e.preventDefault();
      document.querySelectorAll(".settings-nav-item").forEach(function (i) {
        i.classList.remove("active");
      });
      this.classList.add("active");
    });
  });
}

function bindToggles() {
  document
    .querySelectorAll(".toggle-switch input")
    .forEach(function (toggle) {
      toggle.addEventListener("change", function () {
        saveSetting(this.dataset.setting, this.checked);
      });
    });
}

function bindThemeSelector() {
  document.querySelectorAll(".theme-option input").forEach(function (option) {
    option.addEventListener("change", function () {
      document.body.setAttribute("data-theme", this.value);
      saveSetting("theme", this.value);
    });
  });
}

function bindAvatarUpload() {
  var avatarInput = document.getElementById("avatar-input");
  if (avatarInput) {
    avatarInput.addEventListener("change", function (e) {
      var file = e.target.files[0];
      if (file) {
        var reader = new FileReader();
        reader.onload = function (ev) {
          var preview = document.querySelector(".avatar-preview img");
          if (preview) preview.src = ev.target.result;
        };
        reader.readAsDataURL(file);
      }
    });
  }
}

function bindFormValidation() {
  document.querySelectorAll(".settings-form").forEach(function (form) {
    form.addEventListener("submit", function (e) {
      var inputs = form.querySelectorAll("[required]");
      var valid = true;
      inputs.forEach(function (input) {
        if (!input.value.trim()) {
          valid = false;
          input.classList.add("error");
        } else {
          input.classList.remove("error");
        }
      });
      if (!valid) e.preventDefault();
    });
  });
}

function initLanguageSettings() {
  var languageSelect = document.getElementById("language-select");
  var savedLocale = localStorage.getItem(STORAGE_KEYS.LOCALE) || "en";
  if (languageSelect) languageSelect.value = savedLocale;
  document.querySelectorAll(".language-option").forEach(function (opt) {
    opt.classList.toggle("active", opt.dataset.locale === savedLocale);
  });
}

function loadSavedSettings() {
  var theme = localStorage.getItem(STORAGE_KEYS.THEME);
  if (theme) {
    document.body.setAttribute("data-theme", theme);
    var themeOption = document.querySelector(
      '.theme-option[data-theme="' + theme + '"]',
    );
    if (themeOption) themeOption.classList.add("active");
  }
  var compactMode =
    localStorage.getItem(STORAGE_KEYS.COMPACT_MODE) === "true";
  var compactToggle = document.querySelector('[name="compact_mode"]');
  if (compactToggle) {
    compactToggle.checked = compactMode;
    // #1806 — apply to both <html> and <body> for global density.
    if (compactMode) {
      document.documentElement.classList.add("compact-mode");
      document.body.classList.add("compact-mode");
    }
  }
  var animations = localStorage.getItem(STORAGE_KEYS.ANIMATIONS) !== "false";
  var animationsToggle = document.querySelector('[name="animations"]');
  if (animationsToggle) {
    animationsToggle.checked = animations;
    if (!animations) document.body.classList.add("no-animations");
  }
  var savedTz = localStorage.getItem("gb-timezone");
  var tzSelect = document.getElementById("timezone-select");
  if (savedTz && tzSelect) tzSelect.value = savedTz;
  var savedFirstDay = localStorage.getItem("gb-first-day");
  var firstDaySelect = document.getElementById("first-day-select");
  if (savedFirstDay && firstDaySelect) firstDaySelect.value = savedFirstDay;
  var showSidebar =
    localStorage.getItem("gb-show-sidebar") !== "false";
  var showSidebarToggle = document.querySelector('[name="show_sidebar"]');
  if (showSidebarToggle) {
    showSidebarToggle.checked = showSidebar;
    if (!showSidebar) document.body.classList.add("sidebar-hidden");
  }
}

function saveSetting(key, value) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch (e) {
    console.warn("Failed to save setting:", key, e);
  }
}

  S.bindNavigation = bindNavigation;
  S.bindToggles = bindToggles;
  S.bindThemeSelector = bindThemeSelector;
  S.bindAvatarUpload = bindAvatarUpload;
  S.bindFormValidation = bindFormValidation;
  S.initLanguageSettings = initLanguageSettings;
  S.loadSavedSettings = loadSavedSettings;
  S.saveSetting = saveSetting;
})(window);
