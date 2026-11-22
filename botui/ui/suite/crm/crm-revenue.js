/* #2548 — enhance the server-rendered revenue summary with the full typed
   breakdown and honest handling of missing numbers. The no-JS fragment in
   botcontacts/revenue/fragment.rs embeds the report as JSON; this script turns
   it into readable lines. Every visible string goes through gbT with an
   authored English fallback; only EXISTING catalog keys are used. Counts are
   rendered as their own text nodes because the five crm-revenue-note-* catalog
   entries carry no { $count } placeholder. "use strict" first, plain script,
   no ES modules — matches the CRM app. */
"use strict";
(function () {
    var container = document.getElementById('crm-revenue-summary');
    if (!container) return;
    var rendered = false;

    function localized(key, fallback, params) {
        return typeof window.gbT === 'function' ? window.gbT(key, fallback, params) : fallback;
    }

    /* #2548 — machine-readable data_quality.notes -> existing catalog key,
       authored fallback and (where the API reports one) the count field. */
    var NOTES = {
        unattributed_touches: {
            key: 'crm-revenue-note-unattributed',
            fallback: 'Some touches could not be attributed to a campaign.',
            countField: 'unattributed_recipients'
        },
        unknown_event_types: {
            key: 'crm-revenue-note-unknown',
            fallback: 'Some events use an unrecognised type.',
            countField: 'unknown_event_types'
        },
        incomplete_deal_money: {
            key: 'crm-revenue-note-incomplete-money',
            fallback: 'Revenue is omitted because deal values are incomplete or mixed.',
            countField: 'deals_without_money'
        },
        no_deal_cohort: {
            key: 'crm-revenue-note-no-cohort',
            fallback: 'Conversion rate is unavailable without a deal cohort.'
        },
        roi_not_reconciled: {
            key: 'crm-revenue-note-roi',
            fallback: 'ROI is not reported because campaign budget is not reconciled.'
        }
    };

    /* A null metric is shown as N/A together with the note that explains why,
       rather than as 0. Each nullable API field maps to its reason note. */
    var METRIC_REASON = {
        conversion_rate: 'no_deal_cohort',
        revenue_influenced: 'incomplete_deal_money',
        roi: 'roi_not_reconciled'
    };

    function appendLine(text, className) {
        var node = document.createElement('p');
        node.className = className || 'crm-revenue-detail';
        node.textContent = text;
        container.appendChild(node);
        return node;
    }

    /* Localized label + count in separate text nodes. The count is a number,
       not a translatable string, so it never depends on a catalog placeholder. */
    function appendNote(desc, quality) {
        var line = document.createElement('p');
        line.className = 'crm-revenue-detail crm-revenue-note';
        line.setAttribute('data-note', desc.key);
        var label = document.createElement('span');
        label.textContent = localized(desc.key, desc.fallback);
        line.appendChild(label);
        if (desc.countField && typeof quality[desc.countField] === 'number') {
            var chip = document.createElement('span');
            chip.className = 'crm-revenue-note-count';
            chip.textContent = String(quality[desc.countField]);
            line.appendChild(chip);
        }
        container.appendChild(line);
    }

    /* "N/A — reason": the unavailable label plus the matching note, reused
       from the same NOTES table so wording stays identical to the notes list. */
    function appendUnavailable(metric, notesPresent) {
        var reasonCode = METRIC_REASON[metric];
        var line = document.createElement('p');
        line.className = 'crm-revenue-detail crm-revenue-na';
        line.setAttribute('data-metric', metric);
        var na = document.createElement('span');
        na.textContent = localized('crm-revenue-unavailable',
            'Revenue attribution is unavailable.');
        line.appendChild(na);
        if (reasonCode && notesPresent[reasonCode]) {
            var dash = document.createElement('span');
            dash.className = 'crm-revenue-na-sep';
            dash.textContent = '\u2014';
            line.appendChild(dash);
            var reason = document.createElement('span');
            reason.textContent = localized(NOTES[reasonCode].key, NOTES[reasonCode].fallback);
            line.appendChild(reason);
        }
        container.appendChild(line);
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
        var quality = report.data_quality || {};
        var notes = Array.isArray(quality.notes) ? quality.notes : [];
        var notesPresent = {};
        notes.forEach(function (code) { notesPresent[code] = true; });

        // Event breakdown — crm-revenue-detail has { $reply } … placeholders.
        appendLine(localized('crm-revenue-detail',
            'Replies: { $reply } · Unsubscribes: { $unsubscribe } · Bounces: { $bounce } · Complaints: { $complaint } · Skips: { $skip }',
            {
                reply: counts.reply || 0,
                unsubscribe: counts.unsubscribe || 0,
                bounce: counts.bounce || 0,
                complaint: counts.complaint || 0,
                skip: counts.skip || 0
            }));

        // Influenced deals (attributed only).
        appendLine(localized('crm-revenue-influenced',
            'Influenced deals: { $deals } · won: { $won }',
            { deals: report.deals_influenced, won: report.won_deals_influenced }));

        // Unattributed bucket — distinct styling + its own count. The catalog
        // key already contains { $count }, so gbT params fill it in-place.
        var unattributed = report.unattributed_recipients || 0;
        var bucket = appendLine(localized('crm-revenue-unattributed',
            'Unattributed touches: { $count }',
            { count: unattributed }),
            'crm-revenue-detail crm-revenue-unattributed-bucket');
        bucket.setAttribute('data-unattributed', String(unattributed));

        // Nullable metrics: never print 0; print N/A plus the reason note.
        ['conversion_rate', 'revenue_influenced', 'roi'].forEach(function (metric) {
            if (report[metric] === null || report[metric] === undefined) {
                appendUnavailable(metric, notesPresent);
            }
        });

        // Data-quality notes: localized label + count for each reported note.
        notes.forEach(function (code) {
            if (NOTES[code]) appendNote(NOTES[code], quality);
        });
    }

    document.addEventListener('DOMContentLoaded', render);
    document.body.addEventListener('htmx:afterSwap', function (event) {
        if (event.detail && event.detail.target && event.detail.target.id === 'crm-revenue-summary') {
            rendered = false;
            render();
        }
    });
})();
