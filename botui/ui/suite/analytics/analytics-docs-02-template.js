"use strict";

/**
 * Documents tab markup (#2507). Only static, authored strings are injected —
 * API values never enter this template. Every visible label is a catalogue
 * key; the renderer fills the table bodies with text nodes.
 */
(function () {
    var ns = window.AnalyticsDocs;
    var slot = document.getElementById("analyticsDocsSlot");
    if (!ns || !slot) return;

    var TRANSLATABLE =
        "[data-i18n],[data-i18n-placeholder],[data-i18n-title],[data-i18n-aria-label]";

    function field(definition) {
        return (
            '<div class="docs-field">' +
            '<label for="' + definition.id + '" data-i18n="' + definition.labelKey + '"></label>' +
            '<select id="' + definition.id + '" data-docs-scope="' + definition.name + '">' +
            '<option value="" data-i18n="analytics-docs-all"></option>' +
            "</select>" +
            "</div>"
        );
    }

    function tableMarkup(table) {
        var headers = table.columns
            .map(function (column) {
                return '<th scope="col" data-i18n="' + column.labelKey + '"></th>';
            })
            .join("");
        var colspan = table.columns.length;
        return (
            '<article class="docs-card">' +
            '<table class="docs-table" data-docs-table="' + table.id + '">' +
            '<caption data-i18n="' + table.captionKey + '"></caption>' +
            "<thead><tr>" + headers + "</tr></thead>" +
            '<tbody data-docs-body="' + table.id + '">' +
            '<tr class="docs-message"><td colspan="' + colspan + '" role="status">' +
            '<span class="docs-loading" aria-hidden="true"></span>' +
            '<span data-i18n="analytics-docs-status-loading"></span>' +
            "</td></tr>" +
            "</tbody></table></article>"
        );
    }

    function groupMarkup(section) {
        return (
            '<section class="docs-group" aria-labelledby="docs-group-' + section.id + '">' +
            '<h3 id="docs-group-' + section.id + '" data-i18n="' + section.labelKey + '"></h3>' +
            '<div class="docs-grid">' +
            section.tables.map(tableMarkup).join("") +
            "</div></section>"
        );
    }

    var html = [
        '<section class="analytics-docs" id="analytics-documents">',
        '<div class="docs-toolbar" role="group" data-i18n-aria-label="analytics-docs-scope">',
        ns.SCOPE_FIELDS.map(field).join(""),
        '<p class="docs-privacy-note" data-i18n="analytics-docs-date-range-note"></p>',
        '<div class="docs-actions">',
        '<button type="button" id="docsRefresh" data-i18n="analytics-docs-refresh" ' +
            'data-i18n-aria-label="analytics-docs-refresh"></button>',
        '<button type="button" id="docsExport" data-i18n="analytics-docs-export" ' +
            'data-i18n-aria-label="analytics-docs-export"></button>',
        "</div>",
        "</div>",
        '<div id="docsStatus" class="docs-status" role="status" aria-live="polite" aria-atomic="true"></div>',
        '<span id="docsFreshness" class="docs-freshness" role="status" aria-live="polite" aria-atomic="true"></span>',
        ns.SECTIONS.map(groupMarkup).join(""),
        '<p class="docs-privacy-note" data-i18n="analytics-docs-privacy-note"></p>',
        '<p class="docs-privacy-note" data-i18n-params=\'' +
            JSON.stringify({ threshold: ns.SUPPRESSION_THRESHOLD }) +
            '\' data-i18n="analytics-docs-suppressed-note"></p>',
        "</section>",
    ];

    slot.outerHTML = html.join("");

    // The app is an HTMX fragment and may be injected after i18n has already
    // translated the original page, so translate the new subtree explicitly.
    var root = document.getElementById("analytics-documents");
    if (root && window.i18n && typeof window.i18n.translateElement === "function") {
        if (root.matches(TRANSLATABLE)) window.i18n.translateElement(root);
        Array.prototype.forEach.call(root.querySelectorAll(TRANSLATABLE), function (element) {
            window.i18n.translateElement(element);
        });
    }
})();
