/*
 * weather-render.js — the Weather widget's view layer (#2316).
 *
 * Kept separate from weather.js so both files stay well under the repo's
 * 450-line limit. The contract these functions implement: show current data,
 * or say honestly that there is none — never present cached data as fresh.
 */
(function () {
  "use strict";

  function escapeHtml(s) {
    return String(s == null ? "" : s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  // Old-school cellphone week list: one row per day with weekday, small icon
  // and min/max temperatures. Compact and monochrome-friendly.
  function weekRows(daily) {
    var days = daily || [];
    if (!days.length) {
      return '<div class="weather-empty">No week forecast available.</div>';
    }
    return (
      '<div class="weather-week">' +
      days
        .map(function (d, i) {
          var min = d.min === null || d.min === undefined ? "\u2013" : d.min + "\u00b0";
          var max = d.max === null || d.max === undefined ? "\u2013" : d.max + "\u00b0";
          return (
            '<div class="weather-week-row">' +
            '<span class="ww-day">' + escapeHtml(d.day || "Day " + (i + 1)) + "</span>" +
            '<span class="ww-icon">' + d.emoji + "</span>" +
            '<span class="ww-min">' + min + "</span>" +
            '<span class="ww-max">' + max + "</span>" +
            "</div>"
          );
        })
        .join("") +
      "</div>"
    );
  }

  function currentBlock(d) {
    return (
      '<div class="weather-emoji" style="font-size:44px">' + d.emoji + "</div>" +
      '<div class="weather-temp">' + (d.temp === null ? "\u2013" : d.temp + "\u00b0C") + "</div>" +
      '<div class="weather-desc">' + escapeHtml(d.desc || "--") + "</div>" +
      '<div class="weather-city">' + escapeHtml(d.city || "Current location") + "</div>" +
      '<div class="weather-meta">' +
      "<span>\ud83d\udca7 " + (d.humidity === null ? "\u2013" : d.humidity + "%") + "</span>" +
      "<span>\ud83c\udf2c " + (d.wind === null ? "\u2013" : d.wind + " km/h") + "</span>" +
      "</div>"
    );
  }

  function render(d) {
    var current = document.getElementById("weatherCurrent");
    var forecast = document.getElementById("weatherForecast");
    if (!current) return;
    current.innerHTML = currentBlock(d);
    if (forecast) forecast.innerHTML = weekRows(d.daily);
  }

  function clock(updatedAt) {
    if (!updatedAt) return "an earlier visit";
    return new Date(updatedAt).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }

  // Cached data stays on screen only with its age and a retry control.
  function renderStale(d, retry) {
    render(d);
    var current = document.getElementById("weatherCurrent");
    if (!current) return;
    var notice = document.createElement("div");
    notice.className = "weather-stale";
    notice.setAttribute("role", "status");
    notice.textContent = "As of " + clock(d.updated) + " \u2014 couldn't refresh. ";
    var button = document.createElement("button");
    button.type = "button";
    button.className = "weather-retry";
    button.textContent = "Retry";
    button.addEventListener("click", retry);
    notice.appendChild(button);
    current.appendChild(notice);
  }

  // Nothing cached: say so, and never invent a reading.
  function renderOffline(city) {
    var current = document.getElementById("weatherCurrent");
    if (!current) return;
    current.innerHTML =
      '<div class="weather-empty">Weather is offline and no saved data exists. ' +
      "Try again later or search another city.</div>";
    var forecast = document.getElementById("weatherForecast");
    if (forecast) forecast.innerHTML = "";
    if (city) current.setAttribute("data-weather-city", city);
  }

  window.GBWeatherRender = {
    render: render,
    renderStale: renderStale,
    renderOffline: renderOffline,
    escapeHtml: escapeHtml
  };
})();
