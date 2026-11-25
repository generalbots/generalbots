/* Theme scheme: luminance + base-token bridge. State-free; the owner writes data-theme. */
(function (window) {
  "use strict";

// #2933 — pure form: derive the scheme without re-reading style after writes.
function isDarkFromValues(background, solid) {
  const hsl = background.match(/^(-?[\d.]+)(?:deg)?\s+([\d.]+)%\s+([\d.]+)%/);
  if (hsl) return parseFloat(hsl[3]) < 50;
  const hex = (solid || "").match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/i);
  if (!hex) return false;
  const value = hex[1].length === 3 ? hex[1].split("").map((c) => c + c).join("") : hex[1];
  const r = parseInt(value.slice(0, 2), 16);
  const g = parseInt(value.slice(2, 4), 16);
  const b = parseInt(value.slice(4, 6), 16);
  return (r * 299 + g * 587 + b * 114) / 1000 < 128;
}

function backgroundLuminanceIsDark() {
  const rootStyle = getComputedStyle(document.documentElement);
  return isDarkFromValues(rootStyle.getPropertyValue("--background").trim(),
    rootStyle.getPropertyValue("--bg").trim() || rootStyle.getPropertyValue("--bg-primary").trim());
}

function syncColorScheme() {
  document.documentElement.setAttribute("data-color-scheme", backgroundLuminanceIsDark() ? "dark" : "light");
}

// Watch both theme axes.
function watchColorScheme() {
  const observe = () => {
    if (typeof MutationObserver !== "function") return;
    const observer = new MutationObserver(syncColorScheme);
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
    if (document.body) {
      observer.observe(document.body, { attributes: true, attributeFilter: ["data-theme"] });
    }
  };
  if (document.body) observe();
  else document.addEventListener("DOMContentLoaded", observe, { once: true });
  window.addEventListener("load", syncColorScheme);
  syncColorScheme();
}

// Stale inline tokens beat :root[data-theme].
const BRIDGED_TOKENS = [
  "--bg", "--bg-primary", "--bg-secondary", "--bg-card", "--primary-bg",
  "--primary-fg", "--text", "--text-primary", "--text-secondary",
  "--text-muted", "--surface", "--surface-hover", "--surface-active",
  "--surface-2", "--border", "--card-bg", "--border-color",
  "--border-light", "--background", "--foreground",
];

function clearInline() {
  BRIDGED_TOKENS.forEach((key) => document.documentElement.style.removeProperty(key));
}

function hslToHex(h, s, l) {
  l /= 100;
  const a = (s * Math.min(l, 1 - l)) / 100;
  const f = (n) => {
    const k = (n + h / 30) % 12;
    const color = l - a * Math.max(Math.min(k - 3, 9 - k, 1), -1);
    return Math.round(255 * color).toString(16).padStart(2, "0");
  };
  return `#${f(0)}${f(8)}${f(4)}`;
}

function parseHsl(hslStr) {
  if (!hslStr) return null;
  const match = hslStr.trim().match(/^(-?[0-9.]+)\s+([0-9.]+)%\s+([0-9.]+)%(?:\s*\/\s*[0-9.]+%?)?$/);
  if (!match) return null;
  return [parseFloat(match[1]), parseFloat(match[2]), parseFloat(match[3])];
}

function toCssColor(value, fallback) {
  const normalized = value?.trim();
  if (!normalized) return fallback;
  return parseHsl(normalized) ? `hsl(${normalized})` : normalized;
}

function getContrastYIQ(hexcolor) {
  if (!hexcolor) return "#ffffff";
  hexcolor = hexcolor.replace("#", "");
  if (hexcolor.length === 3) hexcolor = hexcolor.split("").map((c) => c + c).join("");
  if (hexcolor.length !== 6) return "#ffffff";
  const r = parseInt(hexcolor.slice(0, 2), 16);
  const g = parseInt(hexcolor.slice(2, 4), 16);
  const b = parseInt(hexcolor.slice(4, 6), 16);
  return (r * 299 + g * 587 + b * 114) / 1000 >= 128 ? "#000000" : "#ffffff";
}

/* Publish both token vocabularies as inline root variables; onDone announces. */
function bridge(themeId, themeName, onDone) {
  const rootStyle = getComputedStyle(document.documentElement);
  const primary = rootStyle.getPropertyValue("--primary")?.trim() || "#3b82f6";
  const background = rootStyle.getPropertyValue("--background")?.trim() || "0 0% 100%";
  const foreground = rootStyle.getPropertyValue("--foreground")?.trim() || "222 47% 11%";
  const mutedForeground = rootStyle.getPropertyValue("--muted-foreground")?.trim() || foreground;
  const card = rootStyle.getPropertyValue("--card")?.trim() || "0 0% 98%";
  const border = rootStyle.getPropertyValue("--border")?.trim() || "214 32% 91%";

  const bgHsl = parseHsl(background);
  const fgHsl = parseHsl(foreground);
  const bgColor = toCssColor(background, "#ffffff");
  const primaryColor = toCssColor(primary, "#3b82f6");
  const cardColor = toCssColor(card, "#fafafa");
  const secondaryBgColor = rootStyle.getPropertyValue("--gb-bg-secondary")?.trim() || cardColor;
  const surfaceColor = rootStyle.getPropertyValue("--gb-surface")?.trim() || cardColor;
  const borderColor = toCssColor(border, "#e2e8f0");
  const borderLightColor = rootStyle.getPropertyValue("--gb-border-light")?.trim() || borderColor;
  const hoverColor = rootStyle.getPropertyValue("--gb-surface-hover")?.trim() || `color-mix(in srgb, ${surfaceColor} 94%, ${primaryColor})`;
  const activeColor = rootStyle.getPropertyValue("--gb-surface-active")?.trim() || `color-mix(in srgb, ${surfaceColor} 86%, ${primaryColor})`;
  const textColor = fgHsl ? hslToHex(...fgHsl) : toCssColor(foreground, getContrastYIQ(bgHsl ? hslToHex(...bgHsl) : bgColor));
  const secondaryTextColor = toCssColor(mutedForeground, textColor);
  const mutedTextColor = rootStyle.getPropertyValue("--gb-text-muted")?.trim() || secondaryTextColor;
  // #2004: body-appended panels read these root tokens.
  const surface2Color = rootStyle.getPropertyValue("--gb-surface-2")?.trim() || `color-mix(in srgb, ${surfaceColor} 92%, ${textColor})`;
  // #2933 — remaining DOM reads before the first write.
  const hasBotColors = document.documentElement.getAttribute("data-has-bot-colors") === "true";
  const themeDropdown = document.getElementById("themeDropdown");

  const vars = {
    "--bg": bgColor, "--bg-primary": bgColor, "--bg-secondary": secondaryBgColor,
    "--bg-card": cardColor, "--primary-bg": bgColor, "--primary-fg": textColor,
    "--text": textColor, "--text-primary": textColor, "--text-secondary": secondaryTextColor,
    "--text-muted": mutedTextColor, "--surface": surfaceColor, "--surface-hover": hoverColor,
    "--surface-active": activeColor, "--surface-2": surface2Color, "--border": borderColor,
    "--card-bg": cardColor, "--border-color": borderColor, "--border-light": borderLightColor,
  };
  vars["--background"] = background;
  vars["--foreground"] = foreground;
  // #2933 — one write phase; no read-after-write reflow.
  document.documentElement.setAttribute("data-color-scheme", isDarkFromValues(background, bgColor) ? "dark" : "light");
  Object.keys(vars).forEach((key) => document.documentElement.style.setProperty(key, vars[key]));

  // Keep config.csv's primary colour.
  if (!hasBotColors) {
    const chatVars = {
      "--chat-color1": primaryColor, "--chat-color2": surfaceColor,
      "--suggestion-color": primaryColor, "--suggestion-bg": surfaceColor,
      "--color1": primaryColor, "--color2": surfaceColor,
    };
    Object.keys(chatVars).forEach((key) => document.documentElement.style.setProperty(key, chatVars[key]));
  }

  // #2004: dropdown surface.
  if (themeDropdown) themeDropdown.style.setProperty("background", surfaceColor, "important");

  gbLog.debug("✓ Theme colours applied:", { themeId: themeId, background: background, primary: primary });
  if (typeof onDone === "function") onDone({ themeId: themeId, themeName: themeName });
}

  window.GBThemeScheme = {
    isDarkBackground: backgroundLuminanceIsDark,
    sync: syncColorScheme,
    watch: watchColorScheme,
    clearInline: clearInline,
    bridge: bridge,
  };
})(window);
