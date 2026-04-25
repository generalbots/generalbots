function getContrastYIQ(hexcolor) {
  if (!hexcolor) return "#ffffff";
  var temp = document.createElement("div");
  temp.style.color = hexcolor;
  temp.style.display = "none";
  document.body.appendChild(temp);
  var style = window.getComputedStyle(temp).color;
  document.body.removeChild(temp);

  var rgb = style.match(/\d+/g);
  if (!rgb || rgb.length < 3) return "#ffffff";
  var r = parseInt(rgb[0]);
  var g = parseInt(rgb[1]);
  var b = parseInt(rgb[2]);
  var yiq = (r * 299 + g * 587 + b * 114) / 1000;
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

  fetch("/api/bot/config?bot_name=" + encodeURIComponent(botName))
    .then(function (response) { return response.json(); })
    .then(function (config) {
      if (!config) return;

      var botId = botName.toLowerCase();
      var botThemeKey = "gb-theme-" + botId;
      var localStorageTheme = localStorage.getItem(botThemeKey);
      var configThemeBase = config.theme_base || config["theme-base"] || "light";
      var useBotConfigColors = !localStorageTheme || localStorageTheme === "default" || localStorageTheme === configThemeBase;

      var color1 = config.theme_color1 || config["theme-color1"] || config["Theme Color"] || "#3b82f6";
      var color2 = config.theme_color2 || config["theme-color2"] || config["theme-color2"] || "#f5deb3";
      var title = config.theme_title || config["theme-title"] || botName;
      var logo = config.theme_logo || config["theme-logo"] || "";

      if (useBotConfigColors) {
        document.documentElement.setAttribute("data-has-bot-colors", "true");
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

      if (logo) {
        var logoImg = document.querySelector(".logo-icon-img");
        if (logoImg) { logoImg.src = logo; logoImg.alt = title || botName; logoImg.style.display = "block"; }
        var logoSvg = document.querySelector(".logo-icon-svg");
        if (logoSvg) logoSvg.style.display = "none";
      }
    })
    .catch(function () {});
}
