// Unified Theme Manager - Dropdown only, no light/dark toggle
const ThemeManager = (() => {
  let currentThemeId = "mindfulness";
  let subscribers = [];

  // Bot ID to theme mapping (configured via config.csv theme-base field)
  const botThemeMap = {
    // Default bot uses the recommended calm workspace theme
    default: "mindfulness",
    // Cristo bot uses typewriter theme (classic typewriter style)
    cristo: "typewriter",
    // Salesianos bot uses light theme with blue accents
    salesianos: "light",
  };

  // Detect current bot from URL path
  function getCurrentBotId() {
    const path = window.location.pathname;
    // Match patterns like /bot/cristo, /cristo, etc.
    const match = path.match(/(?:\/bot\/)?([a-z0-9-]+)/i);
    if (match && match[1]) {
      return match[1].toLowerCase();
    }
    return "default";
  }

  const themes = [
    {
      id: "mindfulness",
      name: "Mindfulness",
      file: "mindfulness.css",
      category: "recommended",
    },
    { id: "aurora", name: "🌌 Aurora", file: "aurora.css", category: "fluent" },
    { id: "mica", name: "🪟 Mica", file: "mica.css", category: "fluent" },
    { id: "sonoma", name: "🍏 Sonoma", file: "sonoma.css", category: "fluent" },
    { id: "nord", name: "🧊 Nord", file: "nord.css", category: "fluent" },
    { id: "carbon", name: "🖤 Carbon", file: "carbon.css", category: "fluent" },
    { id: "paper", name: "📄 Paper", file: "paper.css", category: "fluent" },
    { id: "default", name: "🎨 Default", file: "light.css" },
    { id: "light", name: "☀️ Light", file: "light.css" },
    { id: "minimal-organico", name: "🌿 Minimal Organic", file: "minimal-organico.css", category: "v15" },
    { id: "glass-premium", name: "🪟 Glass Premium", file: "glass-premium.css", category: "v15" },
    { id: "dark-glass", name: "🌑 Dark+Glass", file: "dark-glass.css", category: "v15" },
    { id: "neon-futurista", name: "🌃 Neon Futurist", file: "neon-futurista.css", category: "v15" },
    { id: "orange", name: "🍊 Orange", file: "orange.css" },
    { id: "cyberpunk", name: "🌉 Cyberpunk", file: "cyberpunk.css" },
    { id: "retrowave", name: "🪄 Retrowave", file: "retrowave.css" },
    { id: "vapordream", name: "💭 Vapor Dream", file: "vapordream.css" },
    { id: "y2kglow", name: "✨ Y2K", file: "y2kglow.css" },
    { id: "3dbevel", name: "🔲 3D Bevel", file: "3dbevel.css" },
    { id: "arcadeflash", name: "🕹️ Arcade", file: "arcadeflash.css" },
    { id: "discofever", name: "🪩 Disco", file: "discofever.css" },
    { id: "grungeera", name: "🎸 Grunge", file: "grungeera.css" },
    { id: "jazzage", name: "🎺 Jazz", file: "jazzage.css" },
    { id: "mellowgold", name: "🌻 Mellow", file: "mellowgold.css" },
    { id: "midcenturymod", name: "🏠 Mid Century", file: "midcenturymod.css" },
    {
      id: "polaroidmemories",
      name: "📷 Polaroid",
      file: "polaroidmemories.css",
    },
    {
      id: "saturdaycartoons",
      name: "📺 Cartoons",
      file: "saturdaycartoons.css",
    },
    { id: "seasidepostcard", name: "🏖️ Seaside", file: "seasidepostcard.css" },
    { id: "typewriter", name: "⌨️ Typewriter", file: "typewriter.css" },
    { id: "xeroxui", name: "📠 Xerox", file: "xeroxui.css" },
    { id: "xtreegold", name: "📁 XTree", file: "xtreegold.css" },
  ];

  function loadTheme(id) {
    const theme = themes.find((t) => t.id === id);
    if (!theme) {
      console.warn("Theme not found:", id);
      return;
    }

    document.documentElement.setAttribute("data-theme", id);

    const old = document.getElementById("theme-css");
    if (old) old.remove();

    if (!theme.file) {
      currentThemeId = theme.id;
      const botId = getCurrentBotId();
      localStorage.setItem(`gb-theme-${botId}`, theme.id);
      document.documentElement.setAttribute("data-theme", theme.id);
      updateDropdown();
      return;
    }

    const link = document.createElement("link");
    link.id = "theme-css";
    link.rel = "stylesheet";
    link.href = `/suite/public/themes/${theme.file}`;
    link.onload = () => {
      if (document.getElementById("theme-css") !== link) return;
      console.log("✓ Theme loaded:", theme.name);
      currentThemeId = id;
      const botId = getCurrentBotId();
      localStorage.setItem(`gb-theme-${botId}`, id);

      document.documentElement.setAttribute("data-theme", id);

      // Remove data-theme from body to prevent base.css theme rules from overriding
      document.body.removeAttribute("data-theme");

      // Small delay to ensure CSS variables are applied
      setTimeout(() => {
        // Get the theme's colors from CSS variables
        const rootStyle = getComputedStyle(document.documentElement);
        const primary =
          rootStyle.getPropertyValue("--primary")?.trim() || "#3b82f6";
        const background =
          rootStyle.getPropertyValue("--background")?.trim() || "0 0% 100%";
        const foreground =
          rootStyle.getPropertyValue("--foreground")?.trim() || "222 47% 11%";
        const mutedForeground =
          rootStyle.getPropertyValue("--muted-foreground")?.trim() || foreground;
        const card = rootStyle.getPropertyValue("--card")?.trim() || "0 0% 98%";
        const border =
          rootStyle.getPropertyValue("--border")?.trim() || "214 32% 91%";

        // Convert HSL values to hex format for app compatibility
        const hslToHex = (h, s, l) => {
          l /= 100;
          const a = (s * Math.min(l, 1 - l)) / 100;
          const f = (n) => {
            const k = (n + h / 30) % 12;
            const color = l - a * Math.max(Math.min(k - 3, 9 - k, 1), -1);
            return Math.round(255 * color)
              .toString(16)
              .padStart(2, "0");
          };
          return `#${f(0)}${f(8)}${f(4)}`;
        };

        const parseHsl = (hslStr) => {
          if (!hslStr) return null;
          const match = hslStr
            .trim()
            .match(
              /^(-?[0-9.]+)\s+([0-9.]+)%\s+([0-9.]+)%(?:\s*\/\s*[0-9.]+%?)?$/,
            );
          if (match) {
            return [
              parseFloat(match[1]),
              parseFloat(match[2]),
              parseFloat(match[3]),
            ];
          }
          return null;
        };

        const toCssColor = (value, fallback) => {
          const normalized = value?.trim();
          if (!normalized) return fallback;
          return parseHsl(normalized) ? `hsl(${normalized})` : normalized;
        };

        const getContrastYIQ = (hexcolor) => {
          if (!hexcolor) return "#ffffff";
          hexcolor = hexcolor.replace("#", "");
          if (hexcolor.length === 3) {
            hexcolor = hexcolor
              .split("")
              .map((c) => c + c)
              .join("");
          }
          if (hexcolor.length !== 6) return "#ffffff";
          var r = parseInt(hexcolor.substr(0, 2), 16);
          var g = parseInt(hexcolor.substr(2, 2), 16);
          var b = parseInt(hexcolor.substr(4, 2), 16);
          var yiq = (r * 299 + g * 587 + b * 114) / 1000;
          return yiq >= 128 ? "#000000" : "#ffffff";
        };

        const bgHsl = parseHsl(background);
        const fgHsl = parseHsl(foreground);
        const bgColor = toCssColor(background, "#ffffff");
        const primaryColor = toCssColor(primary, "#3b82f6");
        const cardColor = toCssColor(card, "#fafafa");
        const secondaryBgColor = rootStyle.getPropertyValue("--gb-bg-secondary")?.trim()
          || cardColor;
        const surfaceColor = rootStyle.getPropertyValue("--gb-surface")?.trim()
          || cardColor;
        const borderColor = toCssColor(border, "#e2e8f0");
        const borderLightColor = rootStyle.getPropertyValue("--gb-border-light")?.trim()
          || borderColor;
        const hoverColor = rootStyle.getPropertyValue("--gb-surface-hover")?.trim()
          || `color-mix(in srgb, ${surfaceColor} 94%, ${primaryColor})`;
        const activeColor = rootStyle.getPropertyValue("--gb-surface-active")?.trim()
          || `color-mix(in srgb, ${surfaceColor} 86%, ${primaryColor})`;
        const calculatedText = bgHsl
          ? getContrastYIQ(hslToHex(...bgHsl))
          : getContrastYIQ(bgColor);
        const textColor = fgHsl
          ? hslToHex(...fgHsl)
          : toCssColor(foreground, calculatedText);
        const secondaryTextColor = toCssColor(mutedForeground, textColor);
        const mutedTextColor = rootStyle.getPropertyValue("--gb-text-muted")?.trim()
          || secondaryTextColor;

        document.documentElement.style.setProperty("--bg", bgColor);
        document.documentElement.style.setProperty("--bg-primary", bgColor);
        document.documentElement.style.setProperty("--bg-secondary", secondaryBgColor);
        document.documentElement.style.setProperty("--bg-card", cardColor);
        document.documentElement.style.setProperty("--primary-bg", bgColor);
        document.documentElement.style.setProperty("--primary-fg", textColor);
        document.documentElement.style.setProperty("--text", textColor);
        document.documentElement.style.setProperty("--text-primary", textColor);
        document.documentElement.style.setProperty("--text-secondary", secondaryTextColor);
        document.documentElement.style.setProperty("--text-muted", mutedTextColor);
        document.documentElement.style.setProperty("--surface", surfaceColor);
        document.documentElement.style.setProperty("--surface-hover", hoverColor);
        document.documentElement.style.setProperty("--surface-active", activeColor);
        document.documentElement.style.setProperty("--card-bg", cardColor);
        document.documentElement.style.setProperty("--border-color", borderColor);
        document.documentElement.style.setProperty("--border-light", borderLightColor);
        // Check if config.csv already set the primary color, we shouldn't wipe it
        // Only update color and suggestion variables if they aren't marked as bot-config
        if (
          document.documentElement.getAttribute("data-has-bot-colors") !==
          "true"
        ) {
          document.documentElement.style.setProperty(
            "--chat-color1",
            primaryColor,
          );
          document.documentElement.style.setProperty(
            "--chat-color2",
            surfaceColor,
          );
          document.documentElement.style.setProperty(
            "--suggestion-color",
            primaryColor,
          );
          document.documentElement.style.setProperty(
            "--suggestion-bg",
            surfaceColor,
          );
          document.documentElement.style.setProperty(
            "--color1",
            primaryColor,
          );
          document.documentElement.style.setProperty(
            "--color2",
            surfaceColor,
          );
        }

        console.log("✓ Theme colors applied:", {
          bg: background,
          primary: primary,
        });
        updateDropdown();

        // Fix theme dropdown background to use surface color
        const themeDropdown = document.getElementById("themeDropdown");
        if (themeDropdown) {
          const surfaceColor = getComputedStyle(document.documentElement)
            .getPropertyValue("--surface")
            .trim();
          if (surfaceColor) {
            themeDropdown.style.setProperty(
              "background",
              surfaceColor,
              "important",
            );
          }
        }

        const detail = { themeId: id, themeName: theme.name };
        window.dispatchEvent(new CustomEvent("gb:themechange", { detail }));
        subscribers.forEach((cb) => cb(detail));
      }, 50);
    };
    link.onerror = () => console.error("✗ Failed:", theme.name);
    document.head.appendChild(link);
  }

  function updateDropdown() {
    const select = document.getElementById("themeDropdown");
    if (select) select.value = currentThemeId;
  }

  function createDropdown() {
    const select = document.createElement("select");
    select.id = "themeDropdown";
    select.className = "theme-dropdown";
    themes.forEach((t) => {
      const opt = document.createElement("option");
      opt.value = t.id;
      opt.textContent = t.name;
      select.appendChild(opt);
    });
    select.value = currentThemeId;
    select.onchange = (e) => loadTheme(e.target.value);
    return select;
  }

  function init() {
    // Ensure data-theme is set on html element so CSS selectors work
    if (!document.documentElement.getAttribute("data-theme")) {
      document.documentElement.setAttribute("data-theme", currentThemeId);
    }

    // First, load saved bot theme from config.csv (if available)
    loadSavedTheme();

    // Then load the UI theme (CSS theme)
    // Priority: 1) localStorage user preference, 2) bot-specific theme, 3) default
    const botId = getCurrentBotId();
    let saved = localStorage.getItem(`gb-theme-${botId}`);
    if (!saved || !themes.find((t) => t.id === saved)) {
      // No user preference, try bot-specific theme
      saved = botThemeMap[botId] || "mindfulness";
      // Save to localStorage so it persists
      localStorage.setItem(`gb-theme-${botId}`, saved);
    }
    if (!themes.find((t) => t.id === saved)) saved = "mindfulness";
    currentThemeId = saved;
    loadTheme(saved);

    // Dropdown injection restored for the window manager
    const container = document.getElementById("themeSelectorContainer");
    if (container) {
      container.innerHTML = "";
      container.appendChild(createDropdown());
    }

    console.log("✓ Theme Manager initialized");
  }

  function setThemeFromServer(data) {
    // Save theme to localStorage for persistence across page loads
    const botId = getCurrentBotId();
    localStorage.setItem(`gb-theme-data-${botId}`, JSON.stringify(data));

    // Load base theme if specified
    if (data.theme_base) {
      loadTheme(data.theme_base);
    }

    if (data.logo_url) {
      // For img elements - set src and show, hide SVG
      const logoImg = document.querySelector(".logo-icon-img");
      const logoSvg = document.querySelector(".logo-icon-svg");
      if (logoImg && logoSvg) {
        logoImg.src = data.logo_url;
        logoImg.alt = data.title || "Logo";
        logoImg.style.display = "block";
        logoSvg.style.display = "none";
      }

      // For elements that use background image
      document.querySelectorAll(".assistant-avatar").forEach((el) => {
        el.style.backgroundImage = `url("${data.logo_url}")`;
        el.style.backgroundSize = "contain";
        el.style.backgroundRepeat = "no-repeat";
        el.style.backgroundPosition = "center";
      });
    }
    if (data.color1) {
      document.documentElement.style.setProperty("--color1", data.color1);
    }
    if (data.color2) {
      document.documentElement.style.setProperty("--color2", data.color2);
    }
    if (data.title) document.title = data.title;
    if (data.logo_text) {
      document.querySelectorAll(".logo span, .logo-text").forEach((el) => {
        el.textContent = data.logo_text;
      });
    }
  }

  // Load saved theme from localStorage on page load
  function loadSavedTheme() {
    const botId = getCurrentBotId();
    const savedTheme = localStorage.getItem(`gb-theme-data-${botId}`);
    if (savedTheme) {
      try {
        const data = JSON.parse(savedTheme);
        setThemeFromServer(data);
        console.log(`✓ Theme loaded from localStorage for ${botId}`);
      } catch (e) {
        console.warn("Failed to load saved theme:", e);
      }
    }
  }

  function applyCustomizations() {
    // Called by modules if needed
  }

  function subscribe(cb) {
    subscribers.push(cb);
  }

  return {
    init,
    loadTheme,
    setThemeFromServer,
    loadSavedTheme,
    applyCustomizations,
    subscribe,
    getAvailableThemes: () => themes,
  };
})();

window.ThemeManager = ThemeManager;
