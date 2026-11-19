"use strict";

/**
 * Payload validation and privacy-safe display model (#2507).
 *
 * Nothing here creates numbers: a section with malformed or missing rows is
 * simply empty and renders the explicit "no data available" state. Counts
 * below the threshold are replaced by a marker in the display model itself,
 * so the renderer and the CSV export can only ever see suppressed values.
 */
(function () {
    var ns = window.AnalyticsDocs;
    if (!ns) return;

    function number(value) {
        var parsed = Number(value);
        return Number.isFinite(parsed) ? parsed : NaN;
    }

    function scopesFrom(payload) {
        var source = payload.scopes && typeof payload.scopes === "object" ? payload.scopes : {};
        var scopes = {};
        ns.SCOPE_FIELDS.forEach(function (field) {
            var values = source[field.name];
            scopes[field.name] = Array.isArray(values)
                ? values.filter(function (value) {
                      return typeof value === "string" || typeof value === "number";
                  })
                : [];
        });
        return scopes;
    }

    ns.normalize = function (payload) {
        if (!payload || typeof payload !== "object") return null;
        var source = payload.sections;
        if (!source || typeof source !== "object") return null;

        var generatedAt = typeof payload.generatedAt === "string"
            ? Date.parse(payload.generatedAt)
            : NaN;
        var tables = {};
        var hasData = false;

        ns.SECTIONS.forEach(function (section) {
            var group = source[section.id];
            section.tables.forEach(function (table) {
                var raw = group && group[table.id];
                var rows = [];
                if (Array.isArray(raw)) {
                    raw.slice(0, ns.MAX_ROWS).forEach(function (item) {
                        if (!item || typeof item !== "object") return;
                        var valid = table.columns.every(function (column) {
                            if (!column.sensitive) return true;
                            var value = number(item[column.key]);
                            return Number.isFinite(value) && value >= 0;
                        });
                        if (valid) rows.push(item);
                    });
                }
                tables[table.id] = rows;
                if (rows.length) hasData = true;
            });
        });

        if (!hasData) return null;
        return {
            generatedAt: Number.isFinite(generatedAt) ? generatedAt : null,
            scopes: scopesFrom(payload),
            tables: tables,
        };
    };

    ns.suppressedMarker = function () {
        return ns.tr("analytics-docs-suppressed", "Suppressed");
    };

    ns.suppressionNote = function () {
        return ns.tr(
            "analytics-docs-suppressed-note",
            "Values below { $threshold } are hidden to protect privacy.",
            { threshold: ns.SUPPRESSION_THRESHOLD },
        );
    };

    function formatter() {
        var locale = window.i18n && typeof window.i18n.getCurrentLocale === "function"
            ? window.i18n.getCurrentLocale()
            : "en";
        var integer = new Intl.NumberFormat(locale);
        var decimal = new Intl.NumberFormat(locale, { maximumFractionDigits: 1 });
        return function (type, value) {
            if (type === "count") return integer.format(Math.round(value));
            if (type === "percent") {
                return decimal.format(value) + "%";
            }
            if (type === "hours") {
                return decimal.format(value) + " " + ns.tr("analytics-docs-unit-hours", "h");
            }
            if (type === "bytes") {
                return integer.format(Math.round(value / 1048576)) +
                    " " + ns.tr("analytics-docs-unit-mb", "MB");
            }
            return ns.valueLabel(value);
        };
    }

    /**
     * Builds the exact strings shown in the DOM. A low cell suppresses every
     * sensitive cell in its row and column, so no hidden number can be
     * recovered by subtracting visible values from a total.
     */
    ns.buildDisplay = function (model) {
        var format = formatter();
        var marker = ns.suppressedMarker();
        var display = {};

        ns.SECTIONS.forEach(function (section) {
            section.tables.forEach(function (table) {
                var rows = model.tables[table.id] || [];
                var columnSuppressed = table.columns.map(function (column) {
                    return column.sensitive && rows.some(function (row) {
                        return number(row[column.key]) < ns.SUPPRESSION_THRESHOLD;
                    });
                });

                var displayRows = rows.map(function (row) {
                    var rowLow = table.columns.some(function (column) {
                        return column.sensitive &&
                            number(row[column.key]) < ns.SUPPRESSION_THRESHOLD;
                    });
                    return table.columns.map(function (column, index) {
                        if (column.sensitive && (rowLow || columnSuppressed[index])) {
                            return marker;
                        }
                        if (column.type === "text") {
                            return ns.valueLabel(row[column.key]);
                        }
                        return format(column.type, number(row[column.key]));
                    });
                });

                display[table.id] = {
                    caption: ns.tr(table.captionKey, table.id),
                    columns: table.columns.map(function (column) {
                        return ns.tr(column.labelKey, column.key);
                    }),
                    rows: displayRows,
                };
            });
        });

        return display;
    };
})();
