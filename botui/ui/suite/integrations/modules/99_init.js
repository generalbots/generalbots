if (window.GBAppLifecycle) GBAppLifecycle.begin("integrations");
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
        roots.forEach(function (root) {
            // #2346 — the catalog-source tab owns its own delegated events so it
            // keeps working even when the main initializer hits a bad payload.
            if (typeof namespace.installProviderSourceEvents === "function") {
                namespace.installProviderSourceEvents(root);
            }
            if (typeof namespace.installConnectedControls === "function") {
                namespace.installConnectedControls(root);
            }
            namespace.initializeRoot(root);
        });
    }

    namespace.initializeAll = initializeAll;
    if (typeof namespace.installActionRunner === "function") {
        namespace.installActionRunner();
    }
    if (typeof namespace.installAutomations === "function") {
        namespace.installAutomations();
    }
    if (typeof namespace.installAutomationSchema === "function") {
        namespace.installAutomationSchema();
    }
    initializeAll(document);

    if (!namespace.htmxInitializerBound) {
        namespace.htmxInitializerBound = true;
        document.addEventListener("htmx:afterSwap", function (event) {
            initializeAll(event.target);
        });
    }
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
