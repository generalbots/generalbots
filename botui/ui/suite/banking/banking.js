const BankingApp = {
    state: { transactions: [], platforms: [], reconcilePairs: [], selectedBank: null, selectedPlat: null },

    init() {
        document.querySelectorAll('.banking-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.banking-tab').forEach(t => t.classList.remove('active'));
                document.querySelectorAll('.banking-view').forEach(v => v.classList.remove('active'));
                tab.classList.add('active');
                document.getElementById(tab.dataset.view + '-view').classList.add('active');
            });
        });
        document.getElementById('bankingSearch').addEventListener('input', e => this.filterTransactions(e.target.value));
        this.loadTransactions();
    },

    async api(path) {
        const token = localStorage.getItem('gb_token');
        const res = await fetch(path, { headers: { 'Authorization': 'Bearer ' + token } });
        if (!res.ok) throw new Error('API error: ' + res.status);
        return res.json();
    },

    async loadTransactions() {
        try {
            const data = await this.api('/api/banking/transactions');
            this.state.transactions = Array.isArray(data) ? data : (data.items || []);
            this.renderSummary();
            this.renderTransactions();
        } catch (e) {
            document.getElementById('tx-body').innerHTML = '<tr><td colspan="6" class="loading-row">Failed to load transactions</td></tr>';
        }
    },

    renderSummary() {
        const txs = this.state.transactions;
        const total = txs.reduce((s, t) => s + (t.amount || 0), 0);
        const matched = txs.filter(t => t.matched).length;
        const unmatched = txs.length - matched;
        const platformTotal = txs.reduce((s, t) => s + (t.platform_amount || 0), 0);
        document.getElementById('tx-summary').innerHTML = `
            <div class="summary-card"><span class="label">Total Transactions</span><span class="value">${txs.length}</span></div>
            <div class="summary-card"><span class="label">Total Amount</span><span class="value">${this.fmt(total)}</span></div>
            <div class="summary-card"><span class="label">Matched</span><span class="value positive">${matched}</span></div>
            <div class="summary-card"><span class="label">Unmatched</span><span class="value negative">${unmatched}</span></div>
            <div class="summary-card"><span class="label">Platform Total</span><span class="value accent">${this.fmt(platformTotal)}</span></div>
        `;
    },

    renderTransactions() {
        const txs = this.state.transactions;
        if (!txs.length) {
            document.getElementById('tx-body').innerHTML = '<tr><td colspan="6" class="loading-row">No transactions found</td></tr>';
            return;
        }
        document.getElementById('tx-body').innerHTML = txs.map(t => `
            <tr>
                <td>${this.esc(t.date || '-')}</td>
                <td>${this.esc(t.description || '-')}</td>
                <td style="font-weight:600;color:${(t.amount||0)<0?'#ef4444':'#22c55e'}">${this.fmt(t.amount)}</td>
                <td>${this.esc(t.platform || '-')}</td>
                <td><span class="badge ${t.matched?'matched':'unmatched'}">${t.matched?'Matched':'Unmatched'}</span></td>
                <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px" onclick="BankingApp.viewTransaction('${t.id}')">View</button></td>
            </tr>
        `).join('');
    },

    filterTransactions(query) {
        const q = query.toLowerCase();
        const filtered = this.state.transactions.filter(t =>
            (t.description || '').toLowerCase().includes(q) ||
            (t.platform || '').toLowerCase().includes(q) ||
            (t.date || '').toLowerCase().includes(q)
        );
        const tbody = document.getElementById('tx-body');
        if (!filtered.length) { tbody.innerHTML = '<tr><td colspan="6" class="loading-row">No matches</td></tr>'; return; }
        tbody.innerHTML = filtered.map(t => `
            <tr>
                <td>${this.esc(t.date || '-')}</td>
                <td>${this.esc(t.description || '-')}</td>
                <td style="font-weight:600;color:${(t.amount||0)<0?'#ef4444':'#22c55e'}">${this.fmt(t.amount)}</td>
                <td>${this.esc(t.platform || '-')}</td>
                <td><span class="badge ${t.matched?'matched':'unmatched'}">${t.matched?'Matched':'Unmatched'}</span></td>
                <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px" onclick="BankingApp.viewTransaction('${t.id}')">View</button></td>
            </tr>
        `).join('');
    },

    async loadPlatforms() {
        try {
            const data = await this.api('/api/banking/platforms');
            this.state.platforms = Array.isArray(data) ? data : (data.items || []);
            this.renderPlatforms();
        } catch (e) {
            document.getElementById('platforms-grid').innerHTML = '<div class="loading-row">Failed to load platforms</div>';
        }
    },

    renderPlatforms() {
        const plats = this.state.platforms;
        if (!plats.length) { document.getElementById('platforms-grid').innerHTML = '<div class="empty-state"><p>No platforms configured</p></div>'; return; }
        document.getElementById('platforms-grid').innerHTML = plats.map(p => `
            <div class="platform-card">
                <div class="platform-card-header">
                    <span class="platform-card-name">${this.esc(p.name)}</span>
                    <span class="badge ${p.status || 'connected'}">${this.esc(p.status || 'Connected')}</span>
                </div>
                <div class="platform-card-body">
                    <div class="platform-row"><span class="label">Last Sync</span><span class="val">${this.esc(p.last_sync || '-')}</span></div>
                    <div class="platform-row"><span class="label">Order Count</span><span class="val">${p.order_count || 0}</span></div>
                    <div class="platform-row"><span class="label">Revenue</span><span class="val">${this.fmt(p.revenue)}</span></div>
                    <div class="platform-row"><span class="label">Pending Orders</span><span class="val">${p.pending_orders || 0}</span></div>
                </div>
                <div class="platform-actions">
                    <button onclick="BankingApp.syncPlatform('${p.id}')">Sync Now</button>
                    <button onclick="BankingApp.viewPlatformOrders('${p.id}')">View Orders</button>
                </div>
            </div>
        `).join('');
    },

    async loadReconcile() {
        try {
            const data = await this.api('/api/banking/reconcile');
            const pairs = Array.isArray(data) ? data : (data.items || []);
            this.state.reconcilePairs = pairs;
            this.renderReconcile();
        } catch (e) {
            document.getElementById('bank-list').innerHTML = '<div class="loading-row">Failed to load</div>';
            document.getElementById('plat-list').innerHTML = '<div class="loading-row">Failed to load</div>';
        }
    },

    renderReconcile() {
        const bankItems = this.state.reconcilePairs.filter(p => !p.matched);
        const platItems = this.state.reconcilePairs.filter(p => !p.matched);
        document.getElementById('bank-unmatched-count').textContent = bankItems.length + ' unmatched';
        document.getElementById('plat-unmatched-count').textContent = platItems.length + ' unmatched';
        const renderList = (items, containerId, type) => {
            const el = document.getElementById(containerId);
            if (!items.length) { el.innerHTML = '<div class="empty-state"><p>All matched!</p></div>'; return; }
            el.innerHTML = items.map(i => `
                <div class="reconcile-item" data-id="${i.id}" onclick="BankingApp.selectItem('${type}','${i.id}',this)">
                    <div class="reconcile-item-top">
                        <span class="reconcile-item-desc">${this.esc(i.description || i.order_id || '-')}</span>
                        <span class="reconcile-item-amount">${this.fmt(i.amount)}</span>
                    </div>
                    <div class="reconcile-item-bottom">
                        <span>${this.esc(i.date || '-')}</span>
                        <span>${this.esc(i.platform || i.source || '-')}</span>
                    </div>
                </div>
            `).join('');
        };
        renderList(bankItems, 'bank-list', 'bank');
        renderList(platItems, 'plat-list', 'plat');
    },

    selectItem(type, id, el) {
        const parent = el.closest('.reconcile-list');
        parent.querySelectorAll('.reconcile-item').forEach(i => i.classList.remove('selected'));
        el.classList.add('selected');
        if (type === 'bank') this.state.selectedBank = id;
        else this.state.selectedPlat = id;
    },

    async autoMatch() {
        try {
            await this.api('/api/banking/reconcile/auto');
            this.loadReconcile();
        } catch (e) {
            alert('Auto-match failed: ' + e.message);
        }
    },

    async manualMatch() {
        const { selectedBank, selectedPlat } = this.state;
        if (!selectedBank || !selectedPlat) { alert('Select one item from each side'); return; }
        try {
            const token = localStorage.getItem('gb_token');
            await fetch('/api/banking/reconcile/match', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
                body: JSON.stringify({ bank_id: selectedBank, platform_id: selectedPlat })
            });
            this.state.selectedBank = null;
            this.state.selectedPlat = null;
            this.loadReconcile();
        } catch (e) {
            alert('Match failed: ' + e.message);
        }
    },

    async loadReports() {
        try {
            const data = await this.api('/api/banking/reports');
            const reports = Array.isArray(data) ? data : (data.items || []);
            this.renderReports(reports);
        } catch (e) {
            document.getElementById('reports-grid').innerHTML = '<div class="loading-row">Failed to load reports</div>';
        }
    },

    renderReports(reports) {
        if (!reports.length) { document.getElementById('reports-grid').innerHTML = '<div class="empty-state"><p>No reports available</p></div>'; return; }
        document.getElementById('reports-grid').innerHTML = reports.map(r => `
            <div class="report-card">
                <div class="report-card-header">
                    <span class="report-card-title">${this.esc(r.title || r.period)}</span>
                    <span class="report-card-period">${this.esc(r.period || '')}</span>
                </div>
                <div class="report-card-stats">
                    <div class="report-stat"><span class="label">Total In</span><span class="val positive">${this.fmt(r.total_in)}</span></div>
                    <div class="report-stat"><span class="label">Total Out</span><span class="val negative">${this.fmt(r.total_out)}</span></div>
                    <div class="report-stat"><span class="label">Net</span><span class="val">${this.fmt(r.net)}</span></div>
                    <div class="report-stat"><span class="label">Transactions</span><span class="val">${r.transaction_count || 0}</span></div>
                    <div class="report-stat"><span class="label">Matched %</span><span class="val positive">${r.match_rate || 0}%</span></div>
                    <div class="report-stat"><span class="label">Discrepancies</span><span class="val negative">${r.discrepancies || 0}</span></div>
                </div>
            </div>
        `).join('');
    },

    viewTransaction(id) {
        const tx = this.state.transactions.find(t => t.id === id);
        if (tx) alert('Transaction: ' + JSON.stringify(tx, null, 2));
    },

    async syncPlatform(id) {
        try {
            await this.api('/api/banking/platforms/' + id + '/sync');
            this.loadPlatforms();
        } catch (e) {
            alert('Sync failed: ' + e.message);
        }
    },

    viewPlatformOrders(id) {
        alert('View orders for platform: ' + id);
    },

    refresh() {
        this.loadTransactions();
        this.loadPlatforms();
        this.loadReconcile();
        this.loadReports();
    },

    fmt(v) { return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(v || 0); },
    esc(s) { const d = document.createElement('div'); d.textContent = s; return d.innerHTML; }
};

document.addEventListener('DOMContentLoaded', () => BankingApp.init());
