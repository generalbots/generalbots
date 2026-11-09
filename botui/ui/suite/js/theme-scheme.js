/* Theme manager — background-luminance colour scheme.
 *
 * Split out of `theme-manager.js` for the 450-line structural rule. Reads the
 * active theme's background from computed styles and publishes
 * `data-color-scheme` ("dark"/"light") on <html>, watching both theme
 * attributes so any switcher keeps the scheme in sync. State-free by design.
 */
(function (window) {
  "use strict";

function backgroundLuminanceIsDark() {
  const rootStyle = getComputedStyle(document.documentElement);
  const background = rootStyle.getPropertyValue("--background").trim();
  const hsl = background.match(/^(-?[\d.]+)(?:deg)?\s+([\d.]+)%\s+([\d.]+)%/);
  if (hsl) {
    return parseFloat(hsl[3]) < 50;
  }
  // Themes without an HSL triplet still set a solid `--bg`.
  const solid =
    rootStyle.getPropertyValue("--bg").trim() ||
    rootStyle.getPropertyValue("--bg-primary").trim();
  const hex = solid.match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/i);
  if (!hex) {
    return false;
  }
  const value =
    hex[1].length === 3
      ? hex[1]
          .split("")
          .map((c) => c + c)
          .join("")
      : hex[1];
  const r = parseInt(value.slice(0, 2), 16);
  const g = parseInt(value.slice(2, 4), 16);
  const b = parseInt(value.slice(4, 6), 16);
  return (r * 299 + g * 587 + b * 114) / 1000 < 128;
}

function syncColorScheme() {
  document.documentElement.setAttribute(
    "data-color-scheme",
    backgroundLuminanceIsDark() ? "dark" : "light",
  );
}

// Any theme switcher (this manager, Settings, the legacy body attribute)
// must update the scheme, so watch both elements instead of chasing callers.
function watchColorScheme() {
  const observe = () => {
    if (typeof MutationObserver !== "function") {
      return;
    }
    const observer = new MutationObserver(syncColorScheme);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    if (document.body) {
      observer.observe(document.body, {
        attributes: true,
        attributeFilter: ["data-theme"],
      });
    }
  };
  if (document.body) {
    observe();
  } else {
    document.addEventListener("DOMContentLoaded", observe, { once: true });
  }
  window.addEventListener("load", syncColorScheme);
  syncColorScheme();
}

  window.GBThemeScheme = {
    isDarkBackground: backgroundLuminanceIsDark,
    sync: syncColorScheme,
    watch: watchColorScheme,
  };
})(window);
