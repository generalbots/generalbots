"use strict";
/* Weather (#1154): current conditions + 7-day week forecast via the free
   Open-Meteo API (no key). Falls back to local estimates when offline.
   #1438 — every render path is guarded against undefined values (the city
   label once rendered the literal "undefined"), the saved city is restored
   before first paint, and the forecast is an old-school cellphone week list. */

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
  const FORECAST_DAYS = 7;

  function safeNum(v) {
    const n = Number(v);
    return Number.isFinite(n) ? Math.round(n) : null;
  }

  function safeStr(v, fallback) {
    return typeof v === "string" && v ? v : (fallback || "");
  }

  // Coerces any cache/API payload into a well-formed record so no render path
  // can ever format `undefined` (old cache schemas, partial API responses).
  function normalize(data) {
    if (!data || typeof data !== "object") data = {};
    const daily = Array.isArray(data.daily)
      ? data.daily.map(function (dd) {
          return {
            day: safeStr(dd && dd.day),
            emoji: safeStr(dd && dd.emoji, "🌡"),
            min: safeNum(dd && dd.min),
            max: safeNum(dd && dd.max),
          };
        })
      : [];
    return {
      city: safeStr(data.city),
      temp: safeNum(data.temp),
      desc: safeStr(data.desc),
      emoji: safeStr(data.emoji, "🌡"),
      humidity: safeNum(data.humidity),
      wind: safeNum(data.wind),
      updated: safeNum(data.updated),
      daily: daily,
    };
  }

  function readCache() {
    try {
      return normalize(gbStoreGet(CACHE_KEY, null));
    } catch (e) {
      return normalize(null);
    }
  }

  function writeCache(data) {
    try {
      var record = normalize(data);
      record.updated = Date.now();
      localStorage.setItem(CACHE_KEY, JSON.stringify(record));
    } catch (e) {
      // Deliberate best-effort: a full/blocked localStorage only costs the
      // offline fallback, so this never becomes a user-facing error.
      debugLog("could not cache the forecast", e);
    }
  }

  function debugLog(message, error) {
    if (window.console && window.console.debug) {
      window.console.debug("[weather] " + message, error || "");
    }
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
      localStorage.setItem(CITY_KEY, safeStr(city));
    } catch (e) {
      // Best-effort: the profile copy below still remembers the city.
      debugLog("could not store the chosen city", e);
    }
    persistCityToProfile(safeStr(city));
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
        body: "location=" + encodeURIComponent(safeStr(city)),
      }).catch(function (error) {
        debugLog("profile city mirror failed", error);
      });
    } catch (e) {
      debugLog("profile city mirror unavailable", e);
    }
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
          done(m && m[1] ? m[1].trim() : "");
        })
        .catch(function (error) {
          debugLog("profile city restore failed", error);
          done("");
        });
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

  function mapForecast(wx, cityLabel) {
    if (!wx || !wx.current) throw new Error("no data");
    return normalize({
      city: cityLabel,
      temp: wx.current.temperature_2m,
      desc: "Current conditions",
      emoji: emojiFor(wx.current.weather_code),
      humidity: wx.current.relative_humidity_2m,
      wind: wx.current.wind_speed_10m,
      daily: (wx.daily && wx.daily.time || []).map(function (day, i) {
        return {
          day: new Date(day).toLocaleDateString([], { weekday: "short" }),
          emoji: emojiFor((wx.daily.weather_code || [])[i]),
          max: (wx.daily.temperature_2m_max || [])[i],
          min: (wx.daily.temperature_2m_min || [])[i],
        };
      }),
    });
  }

  function fetchForecast(lat, lon, cityLabel) {
    const wxUrl =
      "https://api.open-meteo.com/v1/forecast?latitude=" + lat +
      "&longitude=" + lon +
      "&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m" +
      "&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=" + FORECAST_DAYS;
    return fetch(wxUrl)
      .then(function (r) { return r.json(); })
      .then(function (wx) { return mapForecast(wx, cityLabel); });
  }

  // #1691 — prefer the botserver proxy: it reaches Open-Meteo on networks
  // where the browser picks an unreachable IPv6 geocoding host, and it keeps
  // the app working behind CORS/CSP restrictions. Direct Open-Meteo remains a
  // fallback when the API is unavailable.
  function fetchForecastProxy(params) {
    var query = Object.keys(params).map(function (key) {
      return encodeURIComponent(key) + "=" + encodeURIComponent(params[key]);
    }).join("&");
    return fetch("/api/weather?" + query)
      .then(function (r) {
        if (!r.ok) throw new Error("weather proxy " + r.status);
        return r.json();
      })
      .then(function (wx) { return mapForecast(wx, wx.city || params.city || "My location"); });
  }

  // A slow geolocation failure used to overwrite a city the user had just
  // searched. Every explicit search invalidates the pending auto-discovery.
  let discoverGeneration = 0;

  function search(city) {
    discoverGeneration++;
    const current = document.getElementById("weatherCurrent");
    const forecast = document.getElementById("weatherForecast");
    if (!current) return;
    current.innerHTML = '<div class="weather-empty">Searching…</div>';
    forecast.innerHTML = "";

    fetchForecastProxy({ city: safeStr(city) })
      .catch(function () {
        const geoUrl = "https://geocoding-api.open-meteo.com/v1/search?name=" + encodeURIComponent(city) + "&count=1&language=en&format=json";
        return fetch(geoUrl)
          .then(function (r) { return r.json(); })
          .then(function (geo) {
            const hit = geo && geo.results && geo.results[0];
            if (!hit) throw new Error("not found");
            return fetchForecast(hit.latitude, hit.longitude, safeStr(city));
          });
      })
      .then(function (data) {
        writeCache(data);
        writeCity(safeStr(city));
        render(data);
      })
      .catch(function (error) {
        debugLog("forecast search failed for " + city, error);
        renderStale(city, function () { search(city); });
      });
  }

  // #1341 — a user with no saved city previously saw an empty panel. Use the
  // device location first, otherwise ask for a city — never render "undefined".
  function requestDefaultCity() {
    // An explicit search or an automation already chose a city; a late
    // geolocation failure must never replace it with the empty prompt.
    if (discoverGeneration > 0) return;
    const current = document.getElementById("weatherCurrent");
    if (!navigator.geolocation) {
      showCityPrompt();
      return;
    }
    current.innerHTML = '<div class="weather-empty">Locating…</div>';
    const generation = ++discoverGeneration;
    const onFail = function (reason) {
      if (generation !== discoverGeneration) return;
      const cached = readCache();
      if (cached.temp !== null || cached.daily.length) {
        renderStale(cached.city, function () { search(cached.city); });
        return;
      }
      showCityPrompt("Location unavailable — " + (reason || "type a city name") + ".");
    };
    const timer = setTimeout(function () { onFail("location lookup timed out"); }, 4000);
    navigator.geolocation.getCurrentPosition(
      function (pos) {
        clearTimeout(timer);
        if (generation !== discoverGeneration) return;
        searchByCoords(pos.coords.latitude, pos.coords.longitude);
      },
      function () {
        clearTimeout(timer);
        onFail("the browser denied or could not find your location");
      },
      { timeout: 4000, maximumAge: 600000 }
    );
  }

  function showCityPrompt(reason) {
    const current = document.getElementById("weatherCurrent");
    if (!current) return;
    current.innerHTML =
      (reason ? '<div class="weather-empty weather-reason">' + escapeHtml(reason) + "</div>" : "") +
      '<div class="weather-empty">Type a city name to see its weather.<br/>' +
      'Example: <em>São Paulo</em></div>';
    const input = document.getElementById("weatherCity");
    if (input) {
      input.placeholder = "Search city… (e.g. São Paulo)";
      try {
        input.focus();
      } catch (e) {
        // The input may be detached while the window closes; focusing is
        // cosmetic, so the prompt stays usable either way.
        debugLog("could not focus the city input", e);
      }
    }
  }

  function searchByCoords(lat, lon) {
    discoverGeneration++;
    const current = document.getElementById("weatherCurrent");
    const forecast = document.getElementById("weatherForecast");
    if (!current) return;
    current.innerHTML = '<div class="weather-empty">Locating…</div>';
    forecast.innerHTML = "";
    fetchForecastProxy({ lat: lat, lon: lon })
      .catch(function () { return fetchForecast(lat, lon, "My location"); })
      .then(function (data) {
        writeCache(data);
        render(data);
      })
      .catch(function (error) {
        debugLog("coordinate forecast failed", error);
        renderStale("", function () { searchByCoords(lat, lon); });
      });
  }

  // View layer lives in weather-render.js (both files stay well under 450
  // lines). These delegates keep the call sites below readable.
  function render(data) {
    GBWeatherRender.render(normalize(data));
  }

  function escapeHtml(s) {
    return GBWeatherRender.escapeHtml(s);
  }

  function renderStale(city, retry) {
    const cached = readCache();
    if (city) cached.city = city;
    if (cached.temp === null && !cached.daily.length) {
      GBWeatherRender.renderOffline(cached.city || city);
      return;
    }
    GBWeatherRender.renderStale(cached, retry);
  }

  function renderFallback(city) {
    const cached = readCache();
    if (cached.temp === null && !cached.daily.length) {
      GBWeatherRender.renderOffline(city);
      return;
    }
    GBWeatherRender.render(cached);
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
    const doSearch = function () {
      const c = input.value.trim();
      if (c) search(c);
    };
    if (btn) btn.addEventListener("click", doSearch);
    input.addEventListener("keydown", function (e) { if (e.key === "Enter") doSearch(); });

    // Restore before first paint: an existing local city always wins for the
    // label even when the cached forecast is rendered instantly.
    const cached = readCache();
    const saved = readCity();
    if (saved) {
      input.value = saved;
      cached.city = saved;
    }
    if (cached.temp !== null || cached.daily.length) {
      render(cached);
      return;
    }
    if (saved) {
      search(saved);
      return;
    }
    restoreCityFromProfile(function (profileCity) {
      if (discoverGeneration > 0) return;
      if (profileCity) {
        input.value = profileCity;
        search(profileCity);
      } else {
        requestDefaultCity();
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