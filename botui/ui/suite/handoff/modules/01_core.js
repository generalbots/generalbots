/* Handoff app — module 1/5: state, tabs and API core (split from handoff/handoff.js for #1731) */

(function (app) {
'use strict';
if (window.GBAppLifecycle) GBAppLifecycle.begin("handoff");
Object.assign(app, {
    state: { queue: [], analytics: {}, channels: [], csat: {} },

    init() {
        document.querySelectorAll('.handoff-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.handoff-tab').forEach(t => t.classList.remove('active'));
                document.querySelectorAll('.handoff-view').forEach(v => v.classList.remove('active'));
                tab.classList.add('active');
                document.getElementById(tab.dataset.view + '-view').classList.add('active');
            });
        });
        const search = document.getElementById('handoffSearch');
        if (search) search.addEventListener('input', () => this.applyQueueFilter());
        const channelFilter = document.getElementById('handoffChannelFilter');
        if (channelFilter) channelFilter.addEventListener('change', () => this.applyQueueFilter());
        this.loadAll();
    },

    async api(path, opts) {
        const token = localStorage.getItem('gb_token');
        const res = await fetch(path, { headers: { 'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json' }, ...opts });
        if (!res.ok) throw new Error('API error: ' + res.status);
        return res.json();
    },

    async loadAll() {
        await Promise.allSettled([
            this.loadQueue(), this.loadAnalytics(), this.loadChannels(), this.loadCSAT(),
            this.loadAgents(), this.loadSLA(), this.loadTranscripts(), this.loadDeflection(),
            this.loadCsatDetail(), this.loadAnalyticsCharts()
        ]);
    }
});
})(window.HandoffApp = window.HandoffApp || {});
