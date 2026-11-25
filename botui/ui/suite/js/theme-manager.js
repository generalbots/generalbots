// Unified Theme Manager — THE SINGLE OWNER of document.documentElement's
// data-theme. Every other theme module (identity/theme.js, base/02_theme.js,
// theme-scheme.js) subscribes to this API; none of them writes the attribute.
//
// One authoritative axis: the attribute carries a THEME ID. The light/dark/
// System preference is resolved to a pair of theme ids (LIGHT_ID / DARK_ID),
// persisted in ONE localStorage key (gb-theme), and migrated from the legacy
// gb-theme-<botId> / gb-theme-data-<botId> values without losing the choice.
const ThemeManager = (() => {
  const STORAGE_KEY = "gb-theme";
  const LIGHT_ID = "light";
  const DARK_ID = "sentient";
  const SYSTEM = "system";
  const VALID_PREF = { light: 1, dark: 1, system: 1 };
  let currentThemeId = "default";
  let subscribers = [];
  let mediaQuery = null;

  // Bot ID to theme mapping (configured via config.csv theme-base field)
  const botThemeMap = {
    // Default bot uses the suite identity theme (assistant look, #2567)
    default: "default",
    // Cristo bot uses typewriter theme (classic typewriter style)
    cristo: "typewriter",
    // Salesianos bot uses light theme with blue accents
    salesianos: "light",
  };

  function getCurrentBotId() {
    const path = window.location.pathname;
    const match = path.match(/(?:\/bot\/)?([a-z0-9-]+)/i);
    return match && match[1] ? match[1].toLowerCase() : "default";
  }

  const themes = [
    {
      // #2567 — the suite's identity theme: a faithful copy of the assistant
      // module's look (paper, ink, orange accent, Fira Sans / Inter).
      id: "default",
      name: "Default",
      file: "default.css",
      category: "recommended",
    },
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
    { id: "high-contrast", name: gbT("theme-high-contrast", "High contrast"), file: "high-contrast.css", category: "accessibility" },
    { id: "paper", name: "📄 Paper", file: "paper.css", category: "fluent" },
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

  // Colour-scheme helpers (state-free) live in theme-scheme.js, loaded before
  // this module in desktop.html. They read computed styles and bridge token
  // vocabularies; only this module writes data-theme.
  const scheme = window.GBThemeScheme || {
    sync: () => {}, watch: () => {}, clearInline: () => {}, bridge: () => {},
  };
  const syncColorScheme = scheme.sync;

  function known(id) { return themes.some((t) => t.id === id); }

  /* The ONLY write of document.documentElement's data-theme in the suite. */
  function applyThemeId(id) {
    // Drop the previous theme's inline tokens before the new :root tokens can
    // be read; otherwise a stale inline --background keeps the old scheme.
    scheme.clearInline();
    document.documentElement.setAttribute("data-theme", id);
    // Legacy body writers cannot override the shell once base CSS is scoped
    // to :root, but clear it anyway so fragments never carry a stale theme.
    if (document.body) document.body.removeAttribute("data-theme");
    syncColorScheme();
  }

  /* Read one key, migrating the three legacy stores once. */
  function storedPreference() {
    let value = null;
    try {
      value = localStorage.getItem(STORAGE_KEY);
      if (!value) {
        const botId = getCurrentBotId();
        value =
          localStorage.getItem(`gb-theme-${botId}`) ||
          localStorage.getItem(`gb-theme-data-${botId}`);
        if (value && value.charAt(0) === "{") {
          try {
            const data = JSON.parse(value);
            value = data.theme_base || null;
          } catch (e) {
            value = null;
          }
        }
        if (value) localStorage.setItem(STORAGE_KEY, value);
      }
    } catch (e) {
      /* storage unavailable */
    }
    if (VALID_PREF[value] || known(value)) return value;
    return SYSTEM;
  }

  function osDark() {
    if (!mediaQuery) {
      if (!window.matchMedia) return false;
      mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    }
    return mediaQuery.matches;
  }

  function resolve(pref) {
    const value = VALID_PREF[pref] ? pref : SYSTEM;
    if (value === SYSTEM) return osDark() ? DARK_ID : LIGHT_ID;
    if (value === "dark") return DARK_ID;
    if (value === "light") return LIGHT_ID;
    return value;
  }

  /* The preference is one key; the attribute always carries a theme id. */
  function setPreference(pref) {
    const value = VALID_PREF[pref] || known(pref) ? pref : SYSTEM;
    try { localStorage.setItem(STORAGE_KEY, value); } catch (e) { /* */ }
    loadTheme(resolve(value));
  }

  function loadTheme(id) {
    const theme = themes.find((t) => t.id === id);
    if (!theme) {
      console.warn("Theme not found:", id);
      return;
    }
    applyThemeId(theme.id);

    const old = document.getElementById("theme-css");
    if (old) old.remove();

    // A theme without a stylesheet still gets both token vocabularies from
    // the scoped base CSS (:root[data-theme=<id>]); bridge them inline so
    // late-loading apps read the new surface, never the old one's tokens.
    if (!theme.file) {
      currentThemeId = theme.id;
      scheme.bridge(theme.id, theme.name, announce);
      updateDropdown();
      return;
    }

    const link = document.createElement("link");
    link.id = "theme-css";
    link.rel = "stylesheet";
    link.href = `/suite/public/themes/${theme.file}?v=2`;
    link.onload = () => {
      if (document.getElementById("theme-css") !== link) return;
      gbLog.debug("✓ Theme loaded:", theme.name);
      currentThemeId = id;
      // Small delay to ensure the theme CSS variables are applied before the
      // bridge reads them.
      setTimeout(() => {
        scheme.bridge(id, theme.name, announce);
        updateDropdown();
      }, 50);
    };
    link.onerror = () => console.error("✗ Failed:", theme.name);
    document.head.appendChild(link);
  }

  function announce(themeId, themeName) {
    const detail = { themeId: themeId, themeName: themeName };
    window.dispatchEvent(new CustomEvent("gb:themechange", { detail: detail }));
    window.dispatchEvent(new CustomEvent("gb-theme-changed", { detail: detail }));
    subscribers.forEach((cb) => cb(detail));
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
    select.onchange = (e) => setPreference(e.target.value);
    return select;
  }

  function init() {
    const botId = getCurrentBotId();
    loadSavedTheme();
    const pref = storedPreference();
    const saved = pref || botThemeMap[botId] || "default";
    currentThemeId = resolve(saved);
    loadTheme(currentThemeId);

    const container = document.getElementById("themeSelectorContainer");
    if (container) {
      container.innerHTML = "";
      container.appendChild(createDropdown());
    }
    gbLog.debug("✓ Theme Manager initialized");
  }

  function setThemeFromServer(data) {
    if (!data) return;
    // Theme choice is never re-persisted here: gb-theme is the only theme key.
    if (data.theme_base) {
      setPreference(data.theme_base);
    }
    if (data.logo_url) {
      const logoImg = document.querySelector(".logo-icon-img");
      const logoSvg = document.querySelector(".logo-icon-svg");
      if (logoImg && logoSvg) {
        logoImg.src = data.logo_url;
        logoImg.alt = data.title || "Logo";
        logoImg.style.display = "block";
        logoSvg.style.display = "none";
      }
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

  // Legacy read-only cache of server customizations (logo/title/colours).
  // It never supplies a theme choice — storedPreference() does that.
  function loadSavedTheme() {
    const botId = getCurrentBotId();
    let saved = null;
    try {
      saved = localStorage.getItem(`gb-theme-data-${botId}`);
    } catch (e) {
      return;
    }
    if (!saved) return;
    try {
      setThemeFromServer(JSON.parse(saved));
      gbLog.debug(`✓ Theme customizations loaded from localStorage for ${botId}`);
    } catch (e) {
      console.warn("Failed to load saved theme:", e);
    }
  }

  function applyCustomizations() {
    // Called by modules if needed.
  }

  function subscribe(cb) { subscribers.push(cb); }

  /* Follow the OS live only while the persisted preference is "system". */
  function watchSystem() {
    try {
      if (!mediaQuery) mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      mediaQuery.addEventListener("change", () => {
        if (storedPreference() === SYSTEM) loadTheme(resolve(SYSTEM));
      });
    } catch (e) {
      /* matchMedia or addEventListener not supported */
    }
  }

  scheme.watch();
  watchSystem();

  const api = {
    init,
    loadTheme,
    setThemeFromServer,
    loadSavedTheme,
    applyCustomizations,
    subscribe,
    getAvailableThemes: () => themes,
    // The light/dark/system preference surface used by Settings/assistant.
    set: setPreference,
    get: storedPreference,
    current: () => (currentThemeId === LIGHT_ID ? "light" : "dark"),
    resolve,
    currentThemeId: () => currentThemeId,
  };

  window.ThemeManager = api;
  // Pages that load identity/theme.js use this handle to delegate instead of
  // opening a second write path to data-theme.
  window.GBTheme = api;
  return api;
})();
