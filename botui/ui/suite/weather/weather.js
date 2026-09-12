"use strict";
/* Weather (#1154): current conditions + 5-day forecast via the free
   Open-Meteo API (no key). Falls back to local estimates when offline. */

(function () {
  if (window.GBWeather) return;

  const CACHE_KEY = "gb-weather-cache";
  // #1311 — the chosen city is persisted for the user: locally for instant
  // restore and in the user profile (settings → Profile, `location` field) so
  // it follows the account across browsers/devices.
  const CITY_KEY = "gb-weather-city";
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

  function readCity() {
    try {
      return localStorage.getItem(CITY_KEY) || "";
    } catch (e) {
      return "";
    }
  }

  function writeCity(city) {
    try {
      localStorage.setItem(CITY_KEY, city);
    } catch (e) {}
    persistCityToProfile(city);
  }

  // Best-effort: mirrors the city into the user profile so other devices see
  // it. The endpoint expects form data and renders HTML; failures are ignored.
  function persistCityToProfile(city) {
    try {
      if (!window.GBSecurity || typeof window.GBSecurity.getToken !== "function") return;
      var token = window.GBSecurity.getToken();
      if (!token) return;
      fetch("/api/user/profile", {
        method: "PUT",
        credentials: "same-origin",
        headers: {
          "Content-Type": "application/x-www-form-urlencoded",
          Authorization: "Bearer " + token,
        },
        body: "location=" + encodeURIComponent(city),
      }).catch(function () {});
    } catch (e) {}
  }

  // Restores a previously saved city from the user profile when this browser
  // has no local copy yet.
  function restoreCityFromProfile(done) {
    try {
      if (!window.GBSecurity || typeof window.GBSecurity.getToken !== "function") {
        done("");
        return;
      }
      var token = window.GBSecurity.getToken();
      if (!token) {
        done("");
        return;
      }
      fetch("/api/user/profile", {
        credentials: "same-origin",
        headers: { Authorization: "Bearer " + token },
      })
        .then(function (r) { return r.ok ? r.text() : ""; })
        .then(function (html) {
          var m = /name="location"[^>]*value="([^"]*)"/.exec(html || "");
          done(m && m[1] ? m[1] : "");
        })
        .catch(function () { done(""); });
    } catch (e) {
      done("");
    }
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

  function fetchForecast(lat, lon, cityLabel) {
    const wxUrl =
      "https://api.open-meteo.com/v1/forecast?latitude=" + lat +
      "&longitude=" + lon +
      "&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m" +
      "&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=5";
    return fetch(wxUrl)
      .then(function (r) { return r.json(); })
      .then(function (wx) {
        if (!wx || !wx.current) throw new Error("no data");
        return {
          city: cityLabel,
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
      });
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
        return fetchForecast(hit.latitude, hit.longitude, city);
      })
      .then(function (data) {
        writeCache(data);
        writeCity(city);
        render(data);
      })
      .catch(function () {
        renderFallback(city);
      });
  }

  // #1341 — a user with no saved city previously saw an empty panel. Offer a
  // default derived from the device location, falling back to the built-in city
  // when geolocation is unavailable, denied or too slow. The auto-detected
  // result is cached but never written as the user's explicit choice.
  function searchDefaultLocation() {
    const fallbackCity = FALLBACK[0].city;
    if (!navigator.geolocation) {
      search(fallbackCity);
      return;
    }
    let settled = false;
    const fallback = function () {
      if (settled) return;
      settled = true;
      search(fallbackCity);
    };
    const timer = setTimeout(fallback, 4000);
    navigator.geolocation.getCurrentPosition(
      function (pos) {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        searchByCoords(pos.coords.latitude, pos.coords.longitude);
      },
      fallback,
      { timeout: 4000, maximumAge: 600000 }
    );
  }

  function searchByCoords(lat, lon) {
    const current = document.getElementById("weatherCurrent");
    const forecast = document.getElementById("weatherForecast");
    if (!current) return;
    current.innerHTML = '<div class="weather-empty">Locating…</div>';
    forecast.innerHTML = "";
    fetchForecast(lat, lon, "My location")
      .then(function (data) {
        writeCache(data);
        render(data);
      })
      .catch(function () {
        renderFallback(FALLBACK[0].city);
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
      '<div class="weather-city">' + escapeHtml(data.city || "") + "</div>" +
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

  // #1311 — the desktop injects this file into a window body, so the script
  // runs AFTER the page's own DOMContentLoaded: listening for it left the
  // search box permanently dead. Initialise immediately, and fall back to the
  // event only when opened as a standalone document.
  function boot() {
    const input = document.getElementById("weatherCity");
    if (!input || input.dataset.gbWeatherBound === "1") return;
    input.dataset.gbWeatherBound = "1";

    const btn = document.getElementById("weatherGo");
    const doSearch = function () { if (input.value.trim()) search(input.value.trim()); };
    if (btn) btn.addEventListener("click", doSearch);
    input.addEventListener("keydown", function (e) { if (e.key === "Enter") doSearch(); });

    const cached = readCache();
    const saved = readCity();
    if (saved) input.value = saved;
    if (cached) {
      render(cached);
      return;
    }
    if (saved) {
      search(saved);
      return;
    }
    restoreCityFromProfile(function (profileCity) {
      if (profileCity) {
        input.value = profileCity;
        search(profileCity);
      } else {
        searchDefaultLocation();
      }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }

  // `savedCity` is exposed so the desktop weather widget renders the user's
  // chosen city instead of its own hardcoded default (#1341).
  window.GBWeather = { search: search, boot: boot, savedCity: readCity };
})();