// Single owner of document.documentElement's data-theme; other modules subscribe.
const ThemeManager = (() => {
  const STORAGE_KEY = "gb-theme";
  const LIGHT_ID = "light";
  const DARK_ID = "carbon";
  const SYSTEM = "system";
  const VALID_PREF = { light: 1, dark: 1, system: 1 };
  let currentThemeId = "default";
  let subscribers = [];
  let mediaQuery = null;

  // Bot ID to theme mapping (config.csv theme-base)
  const botThemeMap = { default: "default", cristo: "typewriter", salesianos: "light" };

  function getCurrentBotId() {
    const path = window.location.pathname;
    const match = path.match(/(?:\/bot\/)?([a-z0-9-]+)/i);
    return match && match[1] ? match[1].toLowerCase() : "default";
  }

  const themes = [
    { id: "default", name: "Default", file: "default.css", category: "recommended" },
    { id: "mindfulness", name: "Mindfulness", file: "mindfulness.css", category: "recommended" },
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
    { id: "polaroidmemories", name: "📷 Polaroid", file: "polaroidmemories.css" },
    { id: "saturdaycartoons", name: "📺 Cartoons", file: "saturdaycartoons.css" },
    { id: "seasidepostcard", name: "🏖️ Seaside", file: "seasidepostcard.css" },
    { id: "typewriter", name: "⌨️ Typewriter", file: "typewriter.css" },
    { id: "xeroxui", name: "📠 Xerox", file: "xeroxui.css" },
    { id: "xtreegold", name: "📁 XTree", file: "xtreegold.css" },
  ];

  // scheme helpers are state-free; only this module writes data-theme.
  const scheme = window.GBThemeScheme || {
    sync: () => {}, watch: () => {}, clearInline: () => {}, bridge: () => {},
  };
  const syncColorScheme = scheme.sync;

  function known(id) { return themes.some((t) => t.id === id); }

  function applyThemeId(id) {
    scheme.clearInline();
    document.documentElement.setAttribute("data-theme", id);
    if (document.body) document.body.removeAttribute("data-theme");
    syncColorScheme();
  }

  /* Read one key. */
  function storedPreference() {
    let value = null;
    try {
      value = localStorage.getItem(STORAGE_KEY);
      if (!value) {
        const botId = getCurrentBotId();
        value = localStorage.getItem(`gb-theme-${botId}`) || localStorage.getItem(`gb-theme-data-${botId}`);
        if (value && value.charAt(0) === "{") {
          try {
            const data = JSON.parse(value);
            value = data.theme_base || null;
          } catch (e) { value = null; }
        }
        if (value) localStorage.setItem(STORAGE_KEY, value);
      }
    } catch (e) { /* storage unavailable */ }
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
    // A named theme id is already a concrete choice — never collapse it to
    // SYSTEM just because it is not one of the three preference keywords.
    if (known(pref)) return pref;
    if (!VALID_PREF[pref]) return SYSTEM;
    if (pref === SYSTEM) return osDark() ? DARK_ID : LIGHT_ID;
    if (pref === "dark") return DARK_ID;
    return LIGHT_ID;
  }

  /* The preference is one key. */
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
    // #2933 — repeated init() calls must not remove and re-fetch the stylesheet
    // (each load re-ran the bridge after a full-document style resolution).
    if (theme.file && old && old.getAttribute("data-gb-theme-id") === theme.id) {
      currentThemeId = theme.id;
      updateDropdown();
      return;
    }
    if (!theme.file) {
      if (old) old.remove();
      currentThemeId = theme.id;
      scheme.bridge(theme.id, theme.name, announce);
      updateDropdown();
      return;
    }
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = `/suite/public/themes/${theme.file}?v=2`;
    link.setAttribute("data-gb-theme-id", theme.id);
    // Keep the outgoing stylesheet applied until the new one has loaded, then
    // hand over its id so the onload guard below targets this link.
    if (old) old.removeAttribute("id");
    // #2933 — the stylesheet's load event already guarantees style resolution;
    // the old 50 ms timer only delayed the batched read/compute/write bridge
    // onto a later task and left the newly loaded theme unbridged for 50 ms.
    link.id = "theme-css";
    link.onload = () => {
      if (document.getElementById("theme-css") !== link) return;
      gbLog.debug("✓ Theme loaded:", theme.name);
      currentThemeId = id;
      scheme.bridge(id, theme.name, announce);
      updateDropdown();
      if (old && old !== link) old.remove();
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
    if (data.theme_base) setPreference(data.theme_base);
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
    if (data.color1) document.documentElement.style.setProperty("--color1", data.color1);
    if (data.color2) document.documentElement.style.setProperty("--color2", data.color2);
    if (data.title) document.title = data.title;
    if (data.logo_text) {
      document.querySelectorAll(".logo span, .logo-text").forEach((el) => {
        el.textContent = data.logo_text;
      });
    }
  }

  // Read-only cache of server customizations; never supplies a theme choice.
  function loadSavedTheme() {
    const botId = getCurrentBotId();
    let saved = null;
    try {
      saved = localStorage.getItem(`gb-theme-data-${botId}`);
    } catch (e) { return; }
    if (!saved) return;
    try {
      setThemeFromServer(JSON.parse(saved));
      gbLog.debug(`✓ Theme customizations loaded from localStorage for ${botId}`);
    } catch (e) {
      console.warn("Failed to load saved theme:", e);
    }
  }

  function applyCustomizations() {}
  function subscribe(cb) { subscribers.push(cb); }

  function watchSystem() {
    try {
      if (!mediaQuery) mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      mediaQuery.addEventListener("change", () => {
        if (storedPreference() === SYSTEM) loadTheme(resolve(SYSTEM));
      });
    } catch (e) { /* matchMedia or addEventListener not supported */ }
  }

  scheme.watch();
  watchSystem();

  const api = {
    init, loadTheme, setThemeFromServer, loadSavedTheme,
    applyCustomizations, subscribe,
    getAvailableThemes: () => themes,
    set: setPreference,
    get: storedPreference,
    current: () => (currentThemeId === LIGHT_ID ? "light" : "dark"),
    resolve,
    currentThemeId: () => currentThemeId,
  };

  window.ThemeManager = api;
  window.GBTheme = api;
  return api;
})();
