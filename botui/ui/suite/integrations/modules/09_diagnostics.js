"use strict";

/* #2339 — persistent, non-secret connection diagnostics. Every recorded value
   is sanitized metadata (outcome, reason, time, latency, endpoint); no
   credential material is ever read or stored here. */
(function (namespace) {
    var STORAGE_KEY = "gb-integrations-test-diagnostics";

    function t(key, fallback, params) {
        return typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
    }

    function readStore() {
        try {
            var parsed = JSON.parse(sessionStorage.getItem(STORAGE_KEY) || "{}");
            return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed : {};
        } catch (error) {
            return {};
        }
    }

    function writeStore(store) {
        try {
            sessionStorage.setItem(STORAGE_KEY, JSON.stringify(store));
        } catch (error) {
            /* Diagnostics are best-effort; sessionStorage may be unavailable. */
        }
    }

    namespace.saveTestDiagnostic = function (connectionId, detail) {
        if (!connectionId) {
            return;
        }
        var store = readStore();
        store[connectionId] = Object.assign({}, detail, { at: Date.now() });
        writeStore(store);
        namespace.testDetails[connectionId] = Object.assign(
            {}, namespace.testDetails[connectionId], store[connectionId]
        );
    };

    /* Rehydrate the in-memory overlay before the first connected render so a
       reload still shows the last test outcome, reason and latency. */
    var store = readStore();
    Object.keys(store).forEach(function (id) {
        namespace.testDetails[id] = Object.assign({}, namespace.testDetails[id], store[id]);
    });

    var originalNormalize = namespace.normalizeConnections;
    if (typeof originalNormalize === "function") {
        namespace.normalizeConnections = function (payload) {
            var rows = originalNormalize(payload);
            var saved = readStore();
            return rows.map(function (row) {
                var detail = saved[row.id];
                if (!detail) {
                    return row;
                }
                return Object.assign({}, row, {
                    last_test_status: detail.outcome || row.last_test_status,
                    last_tested_at: detail.testedAt || row.last_tested_at,
                    last_test_detail: detail.detail || row.last_test_detail,
                    latency_ms: detail.latencyMs
                });
            });
        };
    }

    namespace.diagnosticsText = function (row) {
        var lines = [t("integrations-diagnostics-title", "Connection diagnostics")];
        lines.push(t("integrations-diagnostics-provider", "Provider: { $slug }", {
            slug: namespace.text(row.provider_slug, "unknown")
        }));
        if (row.last_tested_at) {
            lines.push(t("integrations-last-tested", "Last tested") + ": " + row.last_tested_at);
        }
        lines.push(t("integrations-diagnostics-outcome", "Outcome: { $outcome }", {
            outcome: namespace.text(row.last_test_status, t("integrations-not-tested", "Not tested yet"))
        }));
        if (row.last_test_detail) {
            lines.push(t("integrations-diagnostics-detail", "Detail: { $detail }", {
                detail: row.last_test_detail
            }));
        }
        if (row.latency_ms !== undefined && row.latency_ms !== null) {
            lines.push(t("integrations-diagnostics-latency", "Latency: { $ms } ms", {
                ms: row.latency_ms
            }));
        }
        return lines.join("\n");
    };
})(window.GBIntegrationsCatalog = window.GBIntegrationsCatalog || {});
