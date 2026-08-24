"use strict";
/* Weather (#1154): current conditions + 5-day forecast via the free
   Open-Meteo API (no key). Falls back to local estimates when offline. */

(function () {
  if (window.GBWeather) return;

  const CACHE_KEY = "gb-weather-cache";
  const FALLBACK = [
    { city: "São Paulo", temp: 24, desc: "Partly cloudy", emoji: "⛅", humidity: 65, wind: 12 },
    { city: "Lisbon", temp: 22, desc: "Sunny", emoji: "☀️", humidity: 55, wind: 14 },
  ];

  function readCache() {
    try {
      return JSON.parse(localStorage.getItem(CACHE_KEY) || "null");
    } catch (e) {
      return null;
    }
  }

  function writeCache(data) {
    try {
      localStorage.setItem(CACHE_KEY, JSON.stringify(data));
    } catch (e) {}
  }

  function emojiFor(code) {
    if (code === 0) return "☀️";
    if (code === 1 || code === 2) return "🌤";
    if (code === 3) return "☁️";
    if (code >= 45 && code <= 48) return "🌫";
    if (code >= 51 && code <= 67) return "🌧";
    if (code >= 71 && code <= 77) return "🌨";
    if (code >= 80 && code <= 82) return "🌦";
    if (code >= 95) return "⛈";
    return "🌡";
  }

  function search(city) {
    const current = document.getElementById("weatherCurrent");
    const forecast = document.getElementById("weatherForecast");
    if (!current) return;
    current.innerHTML = '<div class="weather-empty">Searching…</div>';
    forecast.innerHTML = "";

    const geoUrl = "https://geocoding-api.open-meteo.com/v1/search?name=" + encodeURIComponent(city) + "&count=1&language=en&format=json";
    fetch(geoUrl)
      .then(function (r) { return r.json(); })
      .then(function (geo) {
        const hit = geo && geo.results && geo.results[0];
        if (!hit) throw new Error("not found");
        const wxUrl =
          "https://api.open-meteo.com/v1/forecast?latitude=" + hit.latitude +
          "&longitude=" + hit.longitude +
          "&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m" +
          "&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=5";
        return fetch(wxUrl).then(function (r) { return r.json(); });
      })
      .then(function (wx) {
        if (!wx || !wx.current) throw new Error("no data");
        const data = {
          city: city,
          temp: Math.round(wx.current.temperature_2m),
          desc: "Current conditions",
          emoji: emojiFor(wx.current.weather_code),
          humidity: wx.current.relative_humidity_2m,
          wind: Math.round(wx.current.wind_speed_10m),
          daily: (wx.daily && wx.daily.time || []).map(function (day, i) {
            return {
              day: new Date(day).toLocaleDateString([], { weekday: "short" }),
              emoji: emojiFor((wx.daily.weather_code || [])[i]),
              max: Math.round((wx.daily.temperature_2m_max || [])[i]),
              min: Math.round((wx.daily.temperature_2m_min || [])[i]),
            };
          }),
        };
        writeCache(data);
        render(data);
      })
      .catch(function () {
        renderFallback(city);
      });
  }

  function render(data) {
    const current = document.getElementById("weatherCurrent");
    const forecast = document.getElementById("weatherForecast");
    if (!current) return;
    current.innerHTML =
      '<div class="weather-emoji" style="font-size:44px">' + data.emoji + "</div>" +
      '<div class="weather-temp">' + data.temp + "°C</div>" +
      '<div class="weather-desc">' + escapeHtml(data.desc) + "</div>" +
      '<div class="weather-city">' + escapeHtml(data.city) + "</div>" +
      '<div class="weather-meta"><span>💧 ' + data.humidity + "%</span><span>🌬 " + data.wind + " km/h</span></div>";
    forecast.innerHTML = (data.daily || [])
      .map(function (d) {
        return '<div class="weather-day"><div class="wd-emoji">' + d.emoji + '</div><div class="wd-day">' + d.day + '</div><div class="wd-temp">' + d.max + "° / " + d.min + "°</div></div>";
      })
      .join("");
  }

  function renderFallback(city) {
    const current = document.getElementById("weatherCurrent");
    if (!current) return;
    const cached = readCache();
    const base = cached || FALLBACK[0];
    current.innerHTML =
      '<div class="weather-emoji" style="font-size:44px">' + base.emoji + "</div>" +
      '<div class="weather-temp">' + base.temp + "°C</div>" +
      '<div class="weather-desc">' + escapeHtml(base.desc) + " (offline estimate)</div>" +
      '<div class="weather-city">' + escapeHtml(city) + "</div>";
  }

  function escapeHtml(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  document.addEventListener("DOMContentLoaded", function () {
    const input = document.getElementById("weatherCity");
    const btn = document.getElementById("weatherGo");
    const doSearch = function () { if (input && input.value.trim()) search(input.value.trim()); };
    if (btn) btn.addEventListener("click", doSearch);
    if (input) {
      input.addEventListener("keydown", function (e) { if (e.key === "Enter") doSearch(); });
      const cached = readCache();
      if (cached) {
        render(cached);
      } else {
        search("São Paulo");
      }
    }
  });

  window.GBWeather = { search: search };
})();