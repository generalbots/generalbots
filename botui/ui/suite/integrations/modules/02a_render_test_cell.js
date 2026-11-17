"use strict";
/* #2345 — actionable, accessible test-result cell. Kept separate so
   02_render.js stays under the 450-line limit; the date/status formatters are
   injected because they are module-local to the render file. */
(function (namespace) {
    function t(key, fallback) {
        return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
    }

    namespace.buildTestCell = function (row, helpers) {
        var statusBadge = helpers.statusBadge;
        var relativeDate = helpers.relativeDate;
        var formatDate = helpers.formatDate;
        var expiryState = helpers.expiryState;
        var tested = !!row.last_test_status;
        var status = tested
            ? statusBadge(row.last_test_status)
            : '<span class="integrations-faint-text">' + namespace.escapeHtml(t("integrations-not-tested", "Not tested yet")) + '</span>';
        var lines = [];
        if (tested) {
            var when = relativeDate(row.last_tested_at) || formatDate(row.last_tested_at);
            lines.push('<span class="integrations-cell-subtext">' +
                namespace.escapeHtml(t("integrations-last-tested", "Last tested") + " " + when) + '</span>');
        } else {
            lines.push('<span class="integrations-cell-subtext">' +
                namespace.escapeHtml(t("integrations-not-tested-hint", "Run a test to verify stored credentials.")) + '</span>');
        }
        var detail = namespace.text(
            row.last_test_detail || (namespace.testDetails && namespace.testDetails[row.id] && namespace.testDetails[row.id].detail),
            ""
        );
        if (detail) {
            lines.push('<span class="integrations-cell-subtext">' + namespace.escapeHtml(detail) + '</span>');
        }
        var expState = expiryState(row.expires_at);
        if (expState === "soon" || expState === "expired") {
            var expLabel = expState === "expired"
                ? t("integrations-credential-expired", "Credential expired")
                : t("integrations-credential-expires-soon", "Credential expires soon");
            lines.push('<span class="integrations-cell-subtext integrations-expiry-warning">' +
                namespace.escapeHtml(expLabel + " · " + (relativeDate(row.expires_at) || formatDate(row.expires_at))) + '</span>');
        }

        var retestLabel = t(tested ? "integrations-retest" : "integrations-test-now", tested ? "Retest" : "Test");
        var rotateLabel = t("integrations-rotate-renew", "Rotate / renew");
        var id = namespace.escapeHtml(row.id);
        var slug = namespace.escapeHtml(row.provider_slug);
        // The rotation endpoint exists (POST .../rotate). When the dialog helper
        // is unavailable, expose an explicit disabled state with an explanatory
        // title instead of a dead button — never invent a new endpoint.
        var rotateButton = typeof namespace.openRotateConnection === "function"
            ? '<button class="integrations-button integrations-button-subtle integrations-test-action" type="button" data-action="rotate-connection" data-connection-id="' + id + '" data-provider-slug="' + slug + '">' + namespace.escapeHtml(rotateLabel) + '</button>'
            : '<button class="integrations-button integrations-button-subtle integrations-test-action" type="button" disabled title="' + namespace.escapeHtml(t("integrations-rotate-unavailable", "Credential rotation is unavailable in this build.")) + '">' + namespace.escapeHtml(rotateLabel) + '</button>';
        var actions = '<span class="integrations-test-actions">' +
            '<button class="integrations-button integrations-button-subtle integrations-test-action" type="button" data-action="test-connection" data-connection-id="' + id + '" data-provider-slug="' + slug + '">' + namespace.escapeHtml(retestLabel) + '</button>' +
            rotateButton + '</span>';

        return '<div class="integrations-test-result">' +
            '<div class="integrations-test-status" role="status" aria-live="polite">' + status + lines.join("") + '</div>' +
            actions + '</div>';
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
