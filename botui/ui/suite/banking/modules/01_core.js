/* Banking app — module 1/5: state, tabs and API core (split from banking/banking.js for #1731) */

if (window.GBAppLifecycle) GBAppLifecycle.begin("banking");
(function (app) {
'use strict';
Object.assign(app, {
    state: {
        transactions: [], platforms: [], reconcilePairs: [],
        accounts: [], pix: [], statements: [], settings: null,
        selectedBank: null, selectedPlat: null, transferDirection: 'out'
    },

    init() {
        document.querySelectorAll('.banking-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.banking-tab').forEach(t => t.classList.remove('active'));
                document.querySelectorAll('.banking-view').forEach(v => v.classList.remove('active'));
                tab.classList.add('active');
                const view = document.getElementById(tab.dataset.view + '-view');
                if (view) view.classList.add('active');
                this.onTabShown(tab.dataset.view);
            });
        });
        const search = document.getElementById('bankingSearch');
        if (search) search.addEventListener('input', e => this.filterTransactions(e.target.value));
        this.refresh();
        const params = window.__gbAppParams__ || {};
        if (params.transaction_id) {
            const target = params.transaction_id;
            let tries = 0;
            const tryOpen = () => {
                if (this.state.transactions.length && this.viewTransaction) {
                    this.viewTransaction(target);
                } else if (tries++ < 20) {
                    setTimeout(tryOpen, 500);
                }
            };
            setTimeout(tryOpen, 600);
        }
    },

    onTabShown(view) {
        if (view === 'accounts') this.loadAccounts();
        else if (view === 'pix') this.loadPix();
        else if (view === 'statements') this.loadStatements();
        else if (view === 'settings') this.loadSettings();
        else if (view === 'transactions') this.loadTransactions();
        else if (view === 'platforms') this.loadPlatforms();
        else if (view === 'reconcile') this.loadReconcile();
        else if (view === 'reports') this.loadReports();
    },

    async api(path, opts) {
        const token = localStorage.getItem('gb_token');
        const headers = { 'Authorization': 'Bearer ' + token };
        if (opts && opts.body) headers['Content-Type'] = 'application/json';
        const res = await fetch(path, { headers, ...opts });
        if (!res.ok) throw new Error('API error: ' + res.status);
        const ct = res.headers.get('content-type') || '';
        if (ct.includes('application/json')) return res.json();
        return res.text();
    },

    refresh() {
        this.loadTransactions();
        this.loadPlatforms();
        this.loadReconcile();
        this.loadReports();
        this.loadStats();
        const active = document.querySelector('.banking-tab.active');
        if (active) this.onTabShown(active.dataset.view);
    }
});
})(window.BankingApp = window.BankingApp || {});
