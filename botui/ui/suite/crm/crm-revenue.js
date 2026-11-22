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
    }

    document.addEventListener('DOMContentLoaded', render);
    document.body.addEventListener('htmx:afterSwap', function (event) {
        if (event.detail && event.detail.target && event.detail.target.id === 'crm-revenue-summary') {
            rendered = false;
            render();
        }
    });
})();
