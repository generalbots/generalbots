"use strict";

(function (namespace) {
    function initializeAll(scope) {
        var roots = [];
        if (scope && scope.matches && scope.matches(".integrations-app[data-integrations-app]")) {
            roots.push(scope);
        }
        if (scope && scope.querySelectorAll) {
            roots = roots.concat(Array.from(scope.querySelectorAll(".integrations-app[data-integrations-app]")));
        }
        roots.forEach(function (root) { namespace.initializeRoot(root); });
    }

    namespace.initializeAll = initializeAll;
    if (typeof namespace.installActionRunner === "function") {
        namespace.installActionRunner();
    }
    initializeAll(document);

    if (!namespace.htmxInitializerBound) {
        namespace.htmxInitializerBound = true;
        document.addEventListener("htmx:afterSwap", function (event) {
            initializeAll(event.target);
        });
    }
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
