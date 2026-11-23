/* Local catalogue loader for the static bulletin pages (#2716). */
"use strict";
(function () {
  var supported = ["pt-BR", "en", "es", "fr", "de", "ja", "ko", "zh-CN"];
  var requested = new URLSearchParams(window.location.search).get("lang");
  var locale = supported.indexOf(requested) !== -1 ? requested : "pt-BR";
  document.documentElement.lang = locale;
  fetch("../lang/" + locale + ".json")
    .then(function (response) { return response.json(); })
    .then(function (catalog) {
      Array.prototype.forEach.call(document.querySelectorAll("[data-i18n]"), function (node) {
        var key = node.getAttribute("data-i18n");
        if (Object.prototype.hasOwnProperty.call(catalog, key)) node.textContent = catalog[key];
      });
    })
    .catch(function () { /* Keep the PT-BR text already in the markup. */ });
})();
