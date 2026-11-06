/* Sales app — module 1/5: state, tabs and API core (split from sales/sales-app.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
    stages: ['new', 'qualified', 'proposal', 'negotiation', 'won', 'lost'],
    stageLabels: { new: 'New', qualified: 'Qualified', proposal: 'Proposal', negotiation: 'Negotiation', won: 'Won', lost: 'Lost' },
    state: { deals: [], contacts: [], activities: [], forecast: {}, editingId: null },

    init() {
        document.querySelectorAll('.sales-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.sales-tab').forEach(t => t.classList.remove('active'));
                document.querySelectorAll('.sales-view').forEach(v => v.classList.remove('active'));
                tab.classList.add('active');
                const view = tab.dataset.view;
                const el = document.getElementById(view + '-view');
                if (el) el.classList.add('active');
                this.onTabShown(view);
            });
        });
        const search = document.getElementById('salesSearch');
        if (search) search.addEventListener('input', e => this.filterDeals(e.target.value));
        const contactSearch = document.getElementById('contactSearch');
        if (contactSearch) contactSearch.addEventListener('input', e => this.filterContacts(e.target.value));
        this.loadPipeline();
        this.loadContacts();
        this.loadActivities();
        this.loadForecast();
    },

    onTabShown(view) {
        if (view === 'leads') this.loadLeads();
        else if (view === 'quotes') this.loadQuotes();
        else if (view === 'orders') this.loadOrders();
        else if (view === 'pipeline') this.loadPipeline();
        else if (view === 'contacts') this.loadContacts();
        else if (view === 'activities') this.loadActivities();
        else if (view === 'forecast') this.loadForecast();
    },

    async api(path, opts) {
        const token = localStorage.getItem('gb_token');
        const headers = { 'Content-Type': 'application/json' };
        if (token) headers['Authorization'] = 'Bearer ' + token;
        const res = await fetch(path, { headers, ...opts });
        if (!res.ok) throw new Error('API error: ' + res.status);
        if (res.status === 204) return null;
        return res.json();
    }
});
})(window.SalesApp = window.SalesApp || {});
