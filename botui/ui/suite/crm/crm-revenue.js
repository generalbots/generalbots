/* #2548 — enhance the server-rendered revenue summary with the full typed
   breakdown. All strings go through gbT with an authored English fallback;
   new catalog keys are reported to the driver (locales are out of scope).
   "use strict" first, plain script, no ES modules — matches the CRM app. */
"use strict";
(function () {
    var container = document.getElementById('crm-revenue-summary');
    if (!container) return;
    var rendered = false;

    function localized(key, fallback, params) {
        return typeof window.gbT === 'function' ? window.gbT(key, fallback, params) : fallback;
    }

    /* #2548 — the API reports machine-readable `data_quality.notes`. Map each
       code to an authored English fallback; each needs a catalog key in every
       locale (reported to the driver, locales are out of scope here). */
    var NOTE_TEXT = {
        unattributed_touches: {
            key: 'crm-revenue-note-unattributed',
            fallback: 'Unattributed recipients: { $count } were not matched to a CRM contact and are counted separately.'
        },
        unknown_event_types: {
            key: 'crm-revenue-note-unknown',
            fallback: 'Unknown event types: { $count } touches were not recognized.'
        },
        incomplete_deal_money: {
            key: 'crm-revenue-note-incomplete-money',
            fallback: 'Incomplete deal money: { $count } won deal(s) lack a value or currency.'
        },
        no_deal_cohort: {
            key: 'crm-revenue-note-no-cohort',
            fallback: 'No deal cohort in scope: a conversion rate cannot be computed.'
        },
        roi_not_reconciled: {
            key: 'crm-revenue-note-roi',
            fallback: 'ROI is not available: no reconciled budget with a matching currency exists.'
        }
    };

    function noteParams(note, quality) {
        if (note === 'unattributed_touches') return { count: quality.unattributed_recipients };
        if (note === 'unknown_event_types') return { count: quality.unknown_event_types };
        if (note === 'incomplete_deal_money') return { count: quality.deals_without_money };
        return {};
    }

    function appendLine(text) {
        var node = document.createElement('p');
        node.className = 'crm-revenue-detail';
        node.textContent = text;
        container.appendChild(node);
    }

    function render() {
        var source = document.getElementById('crm-revenue-json');
        if (!source || rendered) return;
        var report;
        try {
            report = JSON.parse(source.textContent);
        } catch (error) {
            return;
        }
        if (!report || !report.touches) return;
        rendered = true;
        var counts = report.touches;
        appendLine(localized('crm-revenue-detail',
            'Replies { $reply } · unsubscribes { $unsubscribe } · bounces { $bounce } · complaints { $complaint } · skipped { $skip }',
            {
                reply: counts.reply || 0,
                unsubscribe: counts.unsubscribe || 0,
                bounce: counts.bounce || 0,
                complaint: counts.complaint || 0,
                skip: counts.skip || 0
            }));
        appendLine(localized('crm-revenue-influenced',
            'Influenced deals { $deals } · won { $won }',
            { deals: report.deals_influenced, won: report.won_deals_influenced }));
        appendLine(localized('crm-revenue-unattributed',
            'Unattributed recipients { $count }',
            { count: report.unattributed_recipients }));
        if (report.conversion_rate === null || report.revenue_influenced === null || report.roi === null) {
            appendLine(localized('crm-revenue-unavailable',
                'Not available: a metric has no complete denominator or currency'));
        }
        var quality = report.data_quality;
        if (quality && Array.isArray(quality.notes)) {
            quality.notes.forEach(function (note) {
                var text = NOTE_TEXT[note];
                if (!text) return;
                appendLine(localized(text.key, text.fallback, noteParams(note, quality)));
            });
        }
    }

    document.addEventListener('DOMContentLoaded', render);
    document.body.addEventListener('htmx:afterSwap', function (event) {
        if (event.detail && event.detail.target && event.detail.target.id === 'crm-revenue-summary') {
            rendered = false;
            render();
        }
    });
})();
