"use strict";

/* #2339 — shared, localized copy affordance. Copy targets never contain
   secrets: callers pass connection ids, provider slugs, redirect URIs or a
   diagnostics summary assembled from sanitized metadata. */
(function (namespace) {
    function t(key, fallback, params) {
        return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
    }

    namespace.copyButton = function (text, label) {
        return '<button type="button" class="integrations-copy-button integrations-button integrations-button-subtle" ' +
            'data-copy-text="' + namespace.escapeHtml(text) + '" aria-label="' + namespace.escapeHtml(label) + '" ' +
            'title="' + namespace.escapeHtml(label) + '">' + namespace.escapeHtml(label) + '</button>';
    };

    namespace.copyText = function (root, text, label) {
        var target = String(text === null || text === undefined ? "" : text);
        var what = label || t("action-copy", "Copy");
        if (!navigator.clipboard || typeof navigator.clipboard.writeText !== "function") {
            namespace.announce(root, t("chat-copy-failed", "Copy failed"));
            return Promise.resolve(false);
        }
        return navigator.clipboard.writeText(target).then(function () {
            namespace.announce(root, t("chat-copied", "Copied") + ": " + what);
            return true;
        }).catch(function () {
            namespace.announce(root, t("chat-copy-failed", "Copy failed"));
            return false;
        });
    };

    document.addEventListener("click", function (event) {
        var button = event.target && event.target.closest ? event.target.closest("[data-copy-text]") : null;
        if (!button) {
            return;
        }
        var root = button.closest("[data-integrations-app]");
        event.preventDefault();
        void namespace.copyText(root, button.dataset.copyText, button.getAttribute("aria-label"));
    });
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
