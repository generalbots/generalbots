/* #2933 — parse the colour instead of appending a probe node and reading its
 * computed style: the old probe forced a synchronous style/layout pass twice
 * inside the boot-time theme response handler. */
function parseColor(color) {
  if (!color) return null;
  var value = String(color).trim().toLowerCase();
  if (value === "black") return [0, 0, 0];
  if (value === "white") return [255, 255, 255];
  var hex = value.match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/);
  if (hex) {
    var raw = hex[1].length === 3
      ? hex[1].split("").map(function (c) { return c + c; }).join("")
      : hex[1];
    return [
      parseInt(raw.slice(0, 2), 16),
      parseInt(raw.slice(2, 4), 16),
      parseInt(raw.slice(4, 6), 16),
    ];
  }
  var rgb = value.match(/^rgba?\(\s*([0-9.]+)\s*[, ]\s*([0-9.]+)\s*[, ]\s*([0-9.]+)/);
  if (rgb) return [parseFloat(rgb[1]), parseFloat(rgb[2]), parseFloat(rgb[3])];
  return null;
}

function getContrastYIQ(color) {
  var rgb = parseColor(color);
  if (!rgb) return "#ffffff";
  var yiq = (rgb[0] * 299 + rgb[1] * 587 + rgb[2] * 114) / 1000;
  return yiq >= 128 ? "#000000" : "#ffffff";
}

function applyThemeData(themeData) {
  var color1 = themeData.color1 || themeData.data && themeData.data.color1 || "black";
  var color2 = themeData.color2 || themeData.data && themeData.data.color2 || "white";
  var logo = themeData.logo_url || themeData.data && themeData.data.logo_url || "";
  var title = themeData.title || themeData.data && themeData.data.title || window.__INITIAL_BOT_NAME__ || "Chat";

  document.documentElement.style.setProperty("--chat-color1", color1);
  document.documentElement.style.setProperty("--chat-color2", color2);
  document.documentElement.style.setProperty("--suggestion-color", color1);
  document.documentElement.style.setProperty("--suggestion-bg", color2);
  document.documentElement.style.setProperty("--color1", color1);
  document.documentElement.style.setProperty("--color2", color2);
  document.documentElement.style.setProperty("--primary", color1);
  document.documentElement.style.setProperty("--accent", color1);
  document.documentElement.style.setProperty("--chat-fg1", getContrastYIQ(color1));
  document.documentElement.style.setProperty("--chat-fg2", getContrastYIQ(color2));
}

function loadBotConfig() {
  var botName = window.__INITIAL_BOT_NAME__ || "default";

  fetch("/api/bot/public?bot_name=" + encodeURIComponent(botName) + "&_=" + Date.now())
    .then(function (response) { return response.json(); })
    .then(function (config) {
      if (!config) return;

      var color1 = config.theme_color1 || config["theme-color1"] || config["Theme Color"] || "#2563eb";
      var color2 = config.theme_color2 || config["theme-color2"] || config["theme-color2"] || "#f5deb3";
      var title = config.theme_title || config["theme-title"] || botName;
      var logo = config.theme_logo || config["theme-logo"] || "";

      document.documentElement.setAttribute("data-has-bot-colors", "true");
      applyThemeData({ color1: color1, color2: color2, title: title, logo_url: logo });

      if (logo) {
        var logoImg = document.querySelector(".logo-icon-img");
        if (logoImg) { logoImg.src = logo; logoImg.alt = title || botName; logoImg.style.display = "block"; }
        var logoSvg = document.querySelector(".logo-icon-svg");
        if (logoSvg) logoSvg.style.display = "none";
      }
    })
    .catch(function (err) {
      // Best-effort: bot theme config is optional; defaults remain usable.
      console.debug("[chat-theme] could not load bot config", err);
    });
}
