/* Theme scheme: luminance + token bridge. Owner writes data-theme. */
(function (window) {
  "use strict";

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

// #2933 — the bridge caches the scheme from its one pre-write read, so the
// owner's pinned post-write sync call is write-only. The cold read runs at boot.
var gbSchemeSnapshot = null;
function syncColorScheme() {
  if (gbSchemeSnapshot === null) gbSchemeSnapshot = backgroundLuminanceIsDark() ? "dark" : "light";
  document.documentElement.setAttribute("data-color-scheme", gbSchemeSnapshot);
}

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

// Foregrounds are token references, never baked-in colours: the identity
// palette owns the exact neutral and resolves per theme at paint time.
const FG_ON_LIGHT = "var(--palette-000000)";
const FG_ON_DARK = "var(--palette-ffffff)";
function getContrastYIQ(hexcolor) {
  if (!hexcolor) return FG_ON_DARK;
  hexcolor = hexcolor.replace("#", "");
  if (hexcolor.length === 3) hexcolor = hexcolor.split("").map((c) => c + c).join("");
  if (hexcolor.length !== 6) return FG_ON_DARK;
  const r = parseInt(hexcolor.slice(0, 2), 16);
  const g = parseInt(hexcolor.slice(2, 4), 16);
  const b = parseInt(hexcolor.slice(4, 6), 16);
  return (r * 299 + g * 587 + b * 114) / 1000 >= 128 ? FG_ON_LIGHT : FG_ON_DARK;
}

/* Publish both token vocabularies as inline root variables. */
function bridge(themeId, themeName, onDone) {
  const rootStyle = getComputedStyle(document.documentElement);
  const cssVar = (name) => rootStyle.getPropertyValue(name).trim();
  // Fallbacks are token references: every theme supplies these tokens, and a
  // missing one resolves from the identity palette instead of a baked colour.
  const primary = cssVar("--primary") || cssVar("--id-accent") || cssVar("--palette-3b82f6");
  const background = cssVar("--background") || cssVar("--id-paper");
  const foreground = cssVar("--foreground") || cssVar("--id-ink");
  const mutedForeground = cssVar("--muted-foreground") || foreground;
  const card = cssVar("--card") || cssVar("--id-surface-solid");
  const border = cssVar("--border") || cssVar("--id-line");

  const bgHsl = parseHsl(background);
  const fgHsl = parseHsl(foreground);
  const bgColor = toCssColor(background, cssVar("--id-surface-solid"));
  const primaryColor = toCssColor(primary, cssVar("--id-accent"));
  const cardColor = toCssColor(card, cssVar("--id-surface-solid"));
  const secondaryBgColor = cssVar("--gb-bg-secondary") || cardColor;
  const surfaceColor = cssVar("--gb-surface") || cardColor;
  const borderColor = toCssColor(border, cssVar("--id-line"));
  const borderLightColor = rootStyle.getPropertyValue("--gb-border-light")?.trim() || borderColor;
  const hoverColor = rootStyle.getPropertyValue("--gb-surface-hover")?.trim() || `color-mix(in srgb, ${surfaceColor} 94%, ${primaryColor})`;
  const activeColor = rootStyle.getPropertyValue("--gb-surface-active")?.trim() || `color-mix(in srgb, ${surfaceColor} 86%, ${primaryColor})`;
  const textColor = fgHsl ? hslToHex(...fgHsl) : toCssColor(foreground, getContrastYIQ(bgHsl ? hslToHex(...bgHsl) : bgColor));
  const secondaryTextColor = toCssColor(mutedForeground, textColor);
  const mutedTextColor = rootStyle.getPropertyValue("--gb-text-muted")?.trim() || secondaryTextColor;
  const surface2Color = rootStyle.getPropertyValue("--gb-surface-2")?.trim() || `color-mix(in srgb, ${surfaceColor} 92%, ${textColor})`;
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
  // #2933 — one write phase; the cached scheme came from the pre-write reads.
  gbSchemeSnapshot = isDarkFromValues(background, bgColor) ? "dark" : "light";
  document.documentElement.setAttribute("data-color-scheme", gbSchemeSnapshot);
  Object.keys(vars).forEach((key) => document.documentElement.style.setProperty(key, vars[key]));

  if (!hasBotColors) {
    const chatVars = {
      "--chat-color1": primaryColor, "--chat-color2": surfaceColor,
      "--suggestion-color": primaryColor, "--suggestion-bg": surfaceColor,
      "--color1": primaryColor, "--color2": surfaceColor,
    };
    Object.keys(chatVars).forEach((key) => document.documentElement.style.setProperty(key, chatVars[key]));
  }

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
