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

    // #2959 — the suite keeps the live bearer in GBSecurity's closure /
    // sessionStorage and chat-init deliberately purges the legacy `gb_token`
    // localStorage key. Reading only that stale key made every handoff call
    // carry `Authorization: Bearer null`, which auth rejected with 401 before
    // the tenant-scoped handler ran. Prefer the canonical accessor; the legacy
    // key stays as a fallback for standalone / older shell contexts.
    authToken() {
        if (typeof window.getGBAccessToken === 'function') {
            return window.getGBAccessToken() || null;
        }
        if (window.GBAppKit && typeof window.GBAppKit.getAuthToken === 'function') {
            return window.GBAppKit.getAuthToken();
        }
        try { return localStorage.getItem('gb_token') || null; } catch (e) { return null; }
    },

    async api(path, opts) {
        const options = Object.assign({}, opts || {});
        const headers = Object.assign({ 'Content-Type': 'application/json' }, options.headers || {});
        const token = this.authToken();
        if (token) headers.Authorization = 'Bearer ' + token;
        options.headers = headers;
        const res = await fetch(path, options);
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
