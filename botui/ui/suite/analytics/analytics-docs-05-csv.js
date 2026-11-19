"use strict";

/**
 * CSV export (#2507). The export is generated from `ns.lastDisplay`, the very
 * same display model the tables render, so suppressed cells stay suppressed.
 * Labels are neutralised before quoting to prevent spreadsheet formula
 * injection (=, +, -, @, tab and carriage return as the first character).
 */
(function () {
    var ns = window.AnalyticsDocs;
    if (!ns) return;

    ns.csvCell = function (value) {
        var text = String(value == null ? "" : value);
        if (/^[\s]*[=+\-@]/.test(text) || /^[\t\r]/.test(text)) {
            text = "'" + text;
        }
        return '"' + text.replace(/"/g, '""') + '"';
    };

    ns.csvText = function () {
        var display = ns.lastDisplay || {};
        var lines = [];
        ns.SECTIONS.forEach(function (section) {
            section.tables.forEach(function (table) {
                var data = display[table.id];
                if (!data || !data.rows.length) return;
                lines.push(ns.csvCell(data.caption));
                lines.push(data.columns.map(ns.csvCell).join(","));
                data.rows.forEach(function (row) {
                    lines.push(row.map(ns.csvCell).join(","));
                });
                lines.push("");
            });
        });
        return lines.length ? lines.join("\r\n") : "";
    };

    ns.exportCsv = function () {
        var csv = ns.csvText();
        if (!csv) {
            ns.setStatus("empty");
            return;
        }
        var blob = new Blob(["\ufeff" + csv], { type: "text/csv;charset=utf-8" });
        var url = URL.createObjectURL(blob);
        var link = document.createElement("a");
        link.href = url;
        link.download = "document-analytics.csv";
        document.body.appendChild(link);
        link.click();
        link.remove();
        window.setTimeout(function () {
            URL.revokeObjectURL(url);
        }, 0);
    };
})();
