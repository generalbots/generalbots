// Load product config on page load
loadProductConfig();

if (appsBtn) {
  appsBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    const isOpen = appsDropdown.classList.toggle("show");
    appsBtn.setAttribute("aria-expanded", isOpen);
    settingsPanel.classList.remove("show");
  });
}

if (settingsBtn) {
  settingsBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    const isOpen = settingsPanel.classList.toggle("show");
    settingsBtn.setAttribute("aria-expanded", isOpen);
    appsDropdown.classList.remove("show");
    if (userMenu) {
      userMenu.style.display = "none";
      userAvatarBtn.setAttribute("aria-expanded", "false");
    }
  });
}

if (userAvatarBtn) {
  userAvatarBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    const isOpen = userMenu.style.display === "none" || userMenu.style.display === "";
    userMenu.style.display = isOpen ? "block" : "none";
    userAvatarBtn.setAttribute("aria-expanded", isOpen);
    if (appsDropdown) appsDropdown.classList.remove("show");
    if (settingsPanel) settingsPanel.classList.remove("show");
    if (appsBtn) appsBtn.setAttribute("aria-expanded", "false");
    if (settingsBtn) settingsBtn.setAttribute("aria-expanded", "false");
  });
}

document.addEventListener("click", (e) => {
  if (appsDropdown && !appsDropdown.contains(e.target) && !appsBtn.contains(e.target)) {
    appsDropdown.classList.remove("show");
    appsBtn.setAttribute("aria-expanded", "false");
  }
  if (settingsPanel && !settingsPanel.contains(e.target) && !settingsBtn.contains(e.target)) {
    settingsPanel.classList.remove("show");
    settingsBtn.setAttribute("aria-expanded", "false");
  }
  if (userMenu && !userMenu.contains(e.target) && !userAvatarBtn.contains(e.target)) {
    userMenu.style.display = "none";
    userAvatarBtn.setAttribute("aria-expanded", "false");
  }
});

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    if (appsDropdown) appsDropdown.classList.remove("show");
    if (settingsPanel) settingsPanel.classList.remove("show");
    if (userMenu) userMenu.style.display = "none";
    if (appsBtn) appsBtn.setAttribute("aria-expanded", "false");
    if (settingsBtn) settingsBtn.setAttribute("aria-expanded", "false");
    if (userAvatarBtn) userAvatarBtn.setAttribute("aria-expanded", "false");
  }
});

document.addEventListener("keydown", (e) => {
  if (e.altKey && !e.ctrlKey && !e.shiftKey) {
    const shortcuts = {
      1: "chat", 2: "drive", 3: "tasks", 4: "mail", 5: "calendar", 6: "meet",
      7: "paper", 8: "research", 9: "sources", 0: "analytics",
      a: "admin", m: "monitoring",
    };
    if (shortcuts[e.key]) {
      e.preventDefault();
      const link = document.querySelector(`a[href="#${shortcuts[e.key]}"]`);
      if (link) link.click();
      if (appsDropdown) appsDropdown.classList.remove("show");
    }
    if (e.key === ",") {
      e.preventDefault();
      if (settingsPanel) settingsPanel.classList.toggle("show");
    }
    if (e.key === "s") {
      e.preventDefault();
      const settingsLink = document.querySelector(`a[href="#settings"]`);
      if (settingsLink) settingsLink.click();
    }
  }
});

document.body.addEventListener("htmx:afterSwap", (e) => {
  if (e.detail.target.id === "main-content") {
    if (settingsPanel) settingsPanel.classList.remove("show");
  }
});

const themeOptions = document.querySelectorAll(".theme-option");
// #2930: ThemeManager is the single data-theme owner; mirror only meta + highlight.
function activeThemeId() {
  return document.documentElement.getAttribute("data-theme") || "sentient";
}

// The browser chrome colour comes from the active theme's own accent token,
// not from a second, hardcoded palette that drifts from the theme catalogue.
function themeMetaColor() {
  const style = getComputedStyle(document.documentElement);
  const primary = style.getPropertyValue("--primary").trim()
    || style.getPropertyValue("--id-accent").trim();
  if (!primary) return style.getPropertyValue("--palette-84d669").trim();
  return /^(#|hsl\(|rgb\()/i.test(primary) ? primary : `hsl(${primary})`;
}

function updateThemeColor() {
  const metaTheme = document.querySelector('meta[name="theme-color"]');
  if (metaTheme) metaTheme.setAttribute("content", themeMetaColor());
}
function reflectTheme(theme) {
  themeOptions.forEach((o) => o.classList.toggle("active", o.getAttribute("data-theme") === theme));
  document.querySelectorAll(".theme-option").forEach((opt) => {
    opt.classList.toggle("active", opt.getAttribute("data-theme") === theme);
  });
  updateThemeColor();
}
updateThemeColor();
reflectTheme(activeThemeId());

themeOptions.forEach((option) => {
  option.addEventListener("click", () => {
    const theme = option.getAttribute("data-theme");
    if (window.GBTheme && typeof window.GBTheme.set === "function") window.GBTheme.set(theme);
    else if (window.ThemeManager && typeof window.ThemeManager.loadTheme === "function") window.ThemeManager.loadTheme(theme);
  });
});

// Subscribe instead of writing: the owner announces every change.
["gb:themechange", "gb-theme-changed"].forEach((eventName) => {
  window.addEventListener(eventName, (e) => {
    const id = e && e.detail && (e.detail.themeId || e.detail.theme) || activeThemeId();
    reflectTheme(id);
  });
});

window.setTheme = function (theme, element) {
  if (window.GBTheme && typeof window.GBTheme.set === "function") window.GBTheme.set(theme);
  else if (window.ThemeManager && typeof window.ThemeManager.loadTheme === "function") window.ThemeManager.loadTheme(theme);
  reflectTheme(theme);
  if (element) element.classList.add("active");
  if (window.showNotification) window.showNotification("Theme updated", "success");
};

function toggleQuickSetting(el) {
  el.classList.toggle("active");
  const setting = el.id.replace("toggle-", "");
  localStorage.setItem(`gb-${setting}`, el.classList.contains("active"));
}

["notifications", "sound", "compact"].forEach((setting) => {
  const saved = localStorage.getItem(`gb-${setting}`);
  const toggle = document.getElementById(`toggle-${setting}`);
  if (toggle && saved !== null) toggle.classList.toggle("active", saved === "true");
});
