// Unified Theme Manager - Dropdown only, no light/dark toggle
const ThemeManager = (() => {
  let currentThemeId = "default";
  let subscribers = [];

  const themes = [
    { id: "default", name: "🎨 Default", file: null },
    { id: "orange", name: "🍊 Orange", file: "orange.css" },
    { id: "cyberpunk", name: "🌃 Cyberpunk", file: "cyberpunk.css" },
    { id: "retrowave", name: "🌴 Retrowave", file: "retrowave.css" },
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
    { id: "xtreegold", name: "📁 XTree", file: "xtreegold.css" }
  ];

  function loadTheme(id) {
    const theme = themes.find(t => t.id === id);
    if (!theme) {
      console.warn("Theme not found:", id);
      return;
    }

    const old = document.getElementById("theme-css");
    if (old) old.remove();

    if (!theme.file) {
      currentThemeId = "default";
      localStorage.setItem("gb-theme", "default");
      updateDropdown();
      return;
    }

    const link = document.createElement("link");
    link.id = "theme-css";
    link.rel = "stylesheet";
    link.href = `public/themes/${theme.file}`;
    link.onload = () => {
      console.log("✓ Theme loaded:", theme.name);
      currentThemeId = id;
      localStorage.setItem("gb-theme", id);
      updateDropdown();
      subscribers.forEach(cb => cb({ themeId: id, themeName: theme.name }));
    };
    link.onerror = () => console.error("✗ Failed:", theme.name);
    document.head.appendChild(link);
  }

  function updateDropdown() {
    const dd = document.getElementById("themeDropdown");
    if (dd) dd.value = currentThemeId;
  }

  function createDropdown() {
    const select = document.createElement("select");
    select.id = "themeDropdown";
    select.className = "theme-dropdown";
    themes.forEach(t => {
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
    let saved = localStorage.getItem("gb-theme") || "default";
    if (!themes.find(t => t.id === saved)) saved = "default";
    currentThemeId = saved;
    loadTheme(saved);
    
    const container = document.getElementById("themeSelectorContainer");
    if (container) container.appendChild(createDropdown());
    
    console.log("✓ Theme Manager initialized");
  }

  function setThemeFromServer(data) {
    if (data.logo_url) {
      document.querySelectorAll(".logo-icon, .assistant-avatar").forEach(el => {
        el.style.backgroundImage = `url("${data.logo_url}")`;
      });
    }
    if (data.title) document.title = data.title;
    if (data.logo_text) {
      document.querySelectorAll(".logo-text").forEach(el => {
        el.textContent = data.logo_text;
      });
    }
  }

  function applyCustomizations() {
    // Called by modules if needed
  }

  function subscribe(cb) {
    subscribers.push(cb);
  }

  return { init, loadTheme, setThemeFromServer, applyCustomizations, subscribe, getAvailableThemes: () => themes };
})();

window.ThemeManager = ThemeManager;
