if (window.GBAppLifecycle) GBAppLifecycle.begin("banking");
(function() {
'use strict';
const BankingApp = {
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
    },

    // ---------------- Transactions ----------------
    async loadTransactions() {
        try {
            const data = await this.api('/api/banking/transactions');
            this.state.transactions = Array.isArray(data) ? data : (data.items || []);
            this.renderSummary();
            this.renderTransactions();
        } catch (e) {
            const body = document.getElementById('tx-body');
            if (body) body.innerHTML = '<tr><td colspan="8" class="loading-row">Failed to load transactions</td></tr>';
        }
    },

    renderSummary() {
        const txs = this.state.transactions;
        const total = txs.reduce((s, t) => s + (parseFloat(t.amount) || 0), 0);
        const matched = txs.filter(t => t.matched).length;
        const unmatched = txs.length - matched;
        const platformTotal = txs.reduce((s, t) => s + (parseFloat(t.platform_amount) || 0), 0);
        const el = document.getElementById('tx-summary');
        if (!el) return;
        el.innerHTML = `
            <div class="summary-card"><span class="label">Total Transactions</span><span class="value">${txs.length}</span></div>
            <div class="summary-card"><span class="label">Total Amount</span><span class="value">${this.fmtBRL(total)}</span></div>
            <div class="summary-card"><span class="label">Matched</span><span class="value positive">${matched}</span></div>
            <div class="summary-card"><span class="label">Unmatched</span><span class="value negative">${unmatched}</span></div>
            <div class="summary-card"><span class="label">Platform Total</span><span class="value accent">${this.fmtBRL(platformTotal)}</span></div>
        `;
    },

    renderTransactions() {
        const txs = this.state.transactions;
        const body = document.getElementById('tx-body');
        if (!body) return;
        if (!txs.length) { body.innerHTML = '<tr><td colspan="8" class="loading-row">No transactions found</td></tr>'; return; }
        body.innerHTML = txs.map(t => `
            <tr>
                <td><input type="checkbox" class="tx-check" value="${this.esc(t.id)}"></td>
                <td>${this.esc(t.date || '-')}</td>
                <td>${this.esc(t.description || '-')}</td>
                <td style="font-weight:600;color:${(parseFloat(t.amount)||0)<0?'#ef4444':'#22c55e'}">${this.fmtBRL(parseFloat(t.amount)||0)}</td>
                <td>${this.esc(t.platform || '-')}</td>
                <td><span class="badge ${t.matched?'matched':'unmatched'}">${t.matched?'Matched':'Unmatched'}</span></td>
                <td><span class="badge ${(t.status||'').toLowerCase()==='reconciled'?'matched':'pending'}">${this.esc(t.status||'pending')}</span></td>
                <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px" onclick="BankingApp.viewTransaction('${t.id}')">View</button></td>
            </tr>
        `).join('');
    },

    filterTransactions(query) {
        const q = (query || '').toLowerCase();
        const filtered = this.state.transactions.filter(t =>
            (t.description || '').toLowerCase().includes(q) ||
            (t.platform || '').toLowerCase().includes(q) ||
            (t.date || '').toLowerCase().includes(q)
        );
        const tbody = document.getElementById('tx-body');
        if (!tbody) return;
        if (!filtered.length) { tbody.innerHTML = '<tr><td colspan="8" class="loading-row">No matches</td></tr>'; return; }
        tbody.innerHTML = filtered.map(t => `
            <tr>
                <td><input type="checkbox" class="tx-check" value="${this.esc(t.id)}"></td>
                <td>${this.esc(t.date || '-')}</td>
                <td>${this.esc(t.description || '-')}</td>
                <td style="font-weight:600;color:${(parseFloat(t.amount)||0)<0?'#ef4444':'#22c55e'}">${this.fmtBRL(parseFloat(t.amount)||0)}</td>
                <td>${this.esc(t.platform || '-')}</td>
                <td><span class="badge ${t.matched?'matched':'unmatched'}">${t.matched?'Matched':'Unmatched'}</span></td>
                <td><span class="badge ${(t.status||'').toLowerCase()==='reconciled'?'matched':'pending'}">${this.esc(t.status||'pending')}</span></td>
                <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px" onclick="BankingApp.viewTransaction('${t.id}')">View</button></td>
            </tr>
        `).join('');
    },

    // ---------------- Stats (header cards) ----------------
    async loadStats() {
        try {
            const s = await this.api('/api/banking/stats');
            const set = (id, v) => { const el = document.getElementById(id); if (el) el.textContent = v; };
            set('stat-total-in', this.fmtBRL(s.total_in));
            set('stat-total-out', this.fmtBRL(s.total_out));
            set('stat-net', this.fmtBRL(s.net));
            set('stat-pending', s.pending);
            set('stat-match-rate', (s.match_rate || 0).toFixed(1) + '%');
        } catch (e) { /* stats non-critical */ }
    },

    // ---------------- Accounts ----------------
    async loadAccounts() {
        try {
            this.state.accounts = await this.api('/api/banking/accounts');
            this.renderAccounts();
        } catch (e) {
            const b = document.getElementById('accounts-body');
            if (b) b.innerHTML = '<tr><td colspan="8" class="loading-row">Failed to load accounts</td></tr>';
        }
    },

    renderAccounts() {
        const b = document.getElementById('accounts-body');
        if (!b) return;
        if (!this.state.accounts.length) { b.innerHTML = '<tr><td colspan="8" class="loading-row">No accounts. Click "+ Add account".</td></tr>'; return; }
        b.innerHTML = this.state.accounts.map(a => `
            <tr>
                <td>${this.esc(a.bank)}</td>
                <td>${this.esc(a.agency)}</td>
                <td>${this.esc(a.account_number)}</td>
                <td>${this.esc(a.account_type)}</td>
                <td>${this.fmtBRL(parseFloat(a.balance)||0)}</td>
                <td>${this.esc(a.last_sync ? this.fmtDate(a.last_sync) : '-')}</td>
                <td><span class="badge ${a.status==='active'?'matched':'unmatched'}">${this.esc(a.status||'active')}</span></td>
                <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px" onclick="BankingApp.syncAccount('${a.id}')">Sync</button></td>
            </tr>
        `).join('');
    },

    async syncAllAccounts() {
        try {
            await this.api('/api/banking/accounts/sync', { method: 'POST' });
            this.toast('All accounts synced');
            this.loadAccounts();
        } catch (e) { this.toast('Sync failed: ' + e.message); }
    },

    async syncAccount(id) {
        try {
            await this.api('/api/banking/accounts/' + id + '/sync', { method: 'PUT' });
            this.loadAccounts();
        } catch (e) { this.toast('Sync failed: ' + e.message); }
    },

    showAddAccount() {
        this.openModal('bankingAccountModal');
    },

    async submitAccount(e) {
        e.preventDefault();
        const payload = {
            bank: document.getElementById('accountBank').value.trim(),
            agency: document.getElementById('accountAgency').value.trim(),
            account_number: document.getElementById('accountNumber').value.trim(),
            account_type: document.getElementById('accountType').value,
            balance: document.getElementById('accountBalance').value || '0',
            currency: document.getElementById('accountCurrency').value || 'BRL'
        };
        if (!payload.bank) { this.toast('Bank name is required'); return; }
        try {
            await this.api('/api/banking/accounts', { method: 'POST', body: JSON.stringify(payload) });
            this.closeModal('bankingAccountModal');
            this.toast('Account added');
            this.loadAccounts();
        } catch (err) { this.toast('Failed: ' + err.message); }
    },

    // ---------------- Platforms ----------------
    async loadPlatforms() {
        try {
            const data = await this.api('/api/banking/platforms');
            this.state.platforms = Array.isArray(data) ? data : (data.items || []);
            this.renderPlatforms();
        } catch (e) {
            const el = document.getElementById('platforms-grid');
            if (el) el.innerHTML = '<div class="loading-row">Failed to load platforms</div>';
        }
    },

    renderPlatforms() {
        const el = document.getElementById('platforms-grid');
        if (!el) return;
        const plats = this.state.platforms;
        if (!plats.length) { el.innerHTML = '<div class="empty-state"><p>No platforms configured</p></div>'; return; }
        el.innerHTML = plats.map(p => `
            <div class="platform-card">
                <div class="platform-card-header">
                    <span class="platform-card-name">${this.esc(p.name)}</span>
                    <span class="badge ${p.status || 'connected'}">${this.esc(p.status || 'Connected')}</span>
                </div>
                <div class="platform-card-body">
                    <div class="platform-row"><span class="label">Last Sync</span><span class="val">${this.esc(p.last_sync || '-')}</span></div>
                    <div class="platform-row"><span class="label">Order Count</span><span class="val">${p.order_count || 0}</span></div>
                    <div class="platform-row"><span class="label">Revenue</span><span class="val">${this.fmtBRL(parseFloat(p.revenue)||0)}</span></div>
                    <div class="platform-row"><span class="label">Pending Orders</span><span class="val">${p.pending_orders || 0}</span></div>
                </div>
                <div class="platform-actions">
                    <button onclick="BankingApp.syncPlatform('${p.id}')">Sync Now</button>
                    <button onclick="BankingApp.viewPlatformOrders('${p.id}')">View Orders</button>
                </div>
            </div>
        `).join('');
    },

    async syncPlatform(id) {
        try { await this.api('/api/banking/platforms/' + id + '/sync', { method: 'PUT' }); this.loadPlatforms(); }
        catch (e) { this.toast('Sync failed: ' + e.message); }
    },

    viewPlatformOrders(id) { this.toast('Orders view for platform ' + id + ' (coming soon)'); },

    // ---------------- Reconcile ----------------
    async loadReconcile() {
        try {
            const data = await this.api('/api/banking/reconcile/pairs');
            const items = data.items || [];
            this.state.reconcilePairs = items;
            this.renderReconcile();
        } catch (e) {
            const b = document.getElementById('bank-list'); const p = document.getElementById('plat-list');
            if (b) b.innerHTML = '<div class="loading-row">Failed to load</div>';
            if (p) p.innerHTML = '<div class="loading-row">Failed to load</div>';
        }
    },

    renderReconcile() {
        const items = this.state.reconcilePairs;
        const bEl = document.getElementById('bank-unmatched-count');
        const pEl = document.getElementById('plat-unmatched-count');
        if (bEl) bEl.textContent = items.length + ' unmatched';
        if (pEl) pEl.textContent = items.length + ' unmatched';
        const renderList = (containerId) => {
            const el = document.getElementById(containerId);
            if (!el) return;
            if (!items.length) { el.innerHTML = '<div class="empty-state"><p>All matched!</p></div>'; return; }
            el.innerHTML = items.map(i => `
                <div class="reconcile-item" data-id="${i.id}" onclick="BankingApp.selectItem('bank','${i.id}',this)">
                    <div class="reconcile-item-top">
                        <span class="reconcile-item-desc">${this.esc(i.description || i.order_id || '-')}</span>
                        <span class="reconcile-item-amount">${this.fmtBRL(parseFloat(i.amount)||0)}</span>
                    </div>
                    <div class="reconcile-item-bottom">
                        <span>${this.esc(i.date || '-')}</span>
                        <span>${this.esc(i.platform || i.source || '-')}</span>
                    </div>
                </div>
            `).join('');
        };
        renderList('bank-list');
        renderList('plat-list');
    },

    selectItem(type, id, el) {
        const parent = el.closest('.reconcile-list');
        parent.querySelectorAll('.reconcile-item').forEach(i => i.classList.remove('selected'));
        el.classList.add('selected');
        if (type === 'bank') this.state.selectedBank = id;
        else this.state.selectedPlat = id;
    },

    async autoMatch() {
        try { await this.api('/api/banking/reconcile', { method: 'POST' }); this.loadReconcile(); this.loadStats(); }
        catch (e) { this.toast('Auto-match failed: ' + e.message); }
    },

    async manualMatch() {
        const { selectedBank, selectedPlat } = this.state;
        if (!selectedBank || !selectedPlat) { this.toast('Select one item from each side'); return; }
        try {
            const token = localStorage.getItem('gb_token');
            await fetch('/api/banking/reconcile/match', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
                body: JSON.stringify({ bank_id: selectedBank, platform_id: selectedPlat })
            });
            this.state.selectedBank = null; this.state.selectedPlat = null;
            this.loadReconcile();
        } catch (e) { this.toast('Match failed: ' + e.message); }
    },

    // ---------------- PIX ----------------
    async loadPix() {
        try { this.state.pix = await this.api('/api/banking/pix'); this.renderPix(); }
        catch (e) {
            const b = document.getElementById('pix-body');
            if (b) b.innerHTML = '<tr><td colspan="7" class="loading-row">Failed to load PIX</td></tr>';
        }
    },

    renderPix() {
        const b = document.getElementById('pix-body');
        if (!b) return;
        if (!this.state.pix.length) { b.innerHTML = '<tr><td colspan="7" class="loading-row">No PIX transactions.</td></tr>'; return; }
        b.innerHTML = this.state.pix.map(t => `
            <tr>
                <td>${this.esc(this.fmtDate(t.created_at))}</td>
                <td><span class="badge ${t.direction==='in'?'matched':'pending'}">${t.direction==='in'?'Received':'Sent'}</span></td>
                <td>${this.esc(t.counterparty || t.key_value)}</td>
                <td>${this.esc(t.key_type)}</td>
                <td>${this.fmtBRL(parseFloat(t.amount)||0)}</td>
                <td><span class="badge ${t.status==='completed'?'matched':'unmatched'}">${this.esc(t.status||'completed')}</span></td>
                <td>${this.esc(t.end_to_end_id || '-')}</td>
            </tr>
        `).join('');
    },

    showPixTransfer() {
        this.state.transferDirection = 'out';
        const title = document.getElementById('bankingTransferTitle');
        if (title) title.textContent = 'New PIX transfer';
        this.openModal('bankingTransferModal');
    },

    showPixReceive() {
        this.state.transferDirection = 'in';
        const title = document.getElementById('bankingTransferTitle');
        if (title) title.textContent = 'Receive via PIX';
        // In receive mode the description becomes optional memo and amount is what was received.
        this.openModal('bankingTransferModal');
    },

    confirmTransfer() {
        const keyType = document.getElementById('transferKeyType').value;
        const key = document.getElementById('transferKey').value.trim();
        const amount = document.getElementById('transferAmount').value;
        const description = document.getElementById('transferDescription').value.trim();
        if (!key) { this.toast('Recipient key is required'); return; }
        if (!amount || parseFloat(amount) <= 0) { this.toast('Amount must be positive'); return; }
        const payload = { key_type: keyType, key_value: key, amount: amount, description: description };
        const url = this.state.transferDirection === 'in' ? '/api/banking/pix/receive' : '/api/banking/pix/transfer';
        const token = localStorage.getItem('gb_token');
        fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
            body: JSON.stringify(payload)
        })
        .then(r => { if (!r.ok) throw new Error('HTTP ' + r.status); return r.json(); })
        .then(() => { this.closeTransferModal(); this.toast('PIX ' + (this.state.transferDirection === 'in' ? 'received' : 'sent')); this.loadPix(); this.loadStats(); })
        .catch(err => this.toast('Failed: ' + err.message));
    },

    closeTransferModal() { this.closeModal('bankingTransferModal'); },

    async exportPixHistory() {
        try {
            const token = localStorage.getItem('gb_token');
            const res = await fetch('/api/banking/pix/export', { headers: { 'Authorization': 'Bearer ' + token } });
            if (!res.ok) throw new Error('HTTP ' + res.status);
            const blob = await res.blob();
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'pix-history.csv';
            document.body.appendChild(a); a.click(); a.remove();
            URL.revokeObjectURL(url);
        } catch (e) { this.toast('Export failed: ' + e.message); }
    },

    // ---------------- Statements ----------------
    async loadStatements() {
        try { this.state.statements = await this.api('/api/banking/statements'); this.renderStatements(); }
        catch (e) {
            const b = document.getElementById('statements-body');
            if (b) b.innerHTML = '<tr><td colspan="7" class="loading-row">Failed to load statements</td></tr>';
        }
    },

    renderStatements() {
        const b = document.getElementById('statements-body');
        if (!b) return;
        if (!this.state.statements.length) { b.innerHTML = '<tr><td colspan="7" class="loading-row">No statements generated.</td></tr>'; return; }
        b.innerHTML = this.state.statements.map(s => `
            <tr>
                <td>${this.esc(s.period)}</td>
                <td>${this.esc(s.account_label)}</td>
                <td>${this.fmtBRL(parseFloat(s.opening)||0)}</td>
                <td>${this.fmtBRL(parseFloat(s.closing)||0)}</td>
                <td>${this.esc(this.fmtDate(s.generated_at))}</td>
                <td>${this.esc(s.format || 'pdf')}</td>
                <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px" onclick="BankingApp.downloadStatement('${s.id}')">Download</button></td>
            </tr>
        `).join('');
    },

    async downloadStatement(id) {
        try {
            const token = localStorage.getItem('gb_token');
            const res = await fetch('/api/banking/statements/' + id + '/download', { headers: { 'Authorization': 'Bearer ' + token } });
            if (!res.ok) throw new Error('HTTP ' + res.status);
            const blob = await res.blob();
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url; a.download = 'statement-' + id + '.pdf';
            document.body.appendChild(a); a.click(); a.remove();
            URL.revokeObjectURL(url);
        } catch (e) { this.toast('Download failed: ' + e.message); }
    },

    showNewStatement() {
        const period = prompt('Statement period (e.g. 2026-08):');
        if (!period) return;
        const accountId = (this.state.accounts[0] || {}).id;
        if (!accountId) { this.toast('Add an account first'); return; }
        const token = localStorage.getItem('gb_token');
        fetch('/api/banking/statements', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
            body: JSON.stringify({ account_id: accountId, period: period, format: 'pdf' })
        })
        .then(r => { if (!r.ok) throw new Error('HTTP ' + r.status); return r.json(); })
        .then(() => { this.toast('Statement generated'); this.loadStatements(); })
        .catch(err => this.toast('Failed: ' + err.message));
    },

    // ---------------- Reports ----------------
    async loadReports() {
        try {
            const data = await this.api('/api/banking/reports');
            const reports = Array.isArray(data) ? data : (data.items || []);
            this.renderReports(reports);
        } catch (e) {
            const el = document.getElementById('reports-grid');
            if (el) el.innerHTML = '<div class="loading-row">Failed to load reports</div>';
        }
    },

    renderReports(reports) {
        const el = document.getElementById('reports-grid');
        if (!el) return;
        if (!reports.length) { el.innerHTML = '<div class="empty-state"><p>No reports available</p></div>'; return; }
        el.innerHTML = reports.map(r => `
            <div class="report-card">
                <div class="report-card-header">
                    <span class="report-card-title">${this.esc(r.name || r.title || r.period)}</span>
                    <span class="report-card-period">${this.esc(r.period || '')}</span>
                </div>
                <div class="report-card-stats">
                    <div class="report-stat"><span class="label">Kind</span><span class="val">${this.esc(r.kind || '-')}</span></div>
                    <div class="report-stat"><span class="label">Generated</span><span class="val">${this.esc(this.fmtDate(r.created_at))}</span></div>
                </div>
            </div>
        `).join('');
    },

    async generateReport(kind) {
        try {
            await this.api('/api/banking/reports/generate', { method: 'POST', body: JSON.stringify({ kind }) });
            this.toast('Report generated: ' + kind);
            this.loadReports();
        } catch (e) { this.toast('Failed: ' + e.message); }
    },

    // ---------------- Settings ----------------
    async loadSettings() {
        try {
            this.state.settings = await this.api('/api/banking/settings');
            const s = this.state.settings;
            const set = (id, v) => { const el = document.getElementById(id); if (el) el.value = v; };
            set('setting-tolerance', s.tolerance_cents);
            set('setting-window', s.date_window_days);
            set('setting-auto-approve', s.auto_approve_under);
            set('setting-webhook', s.webhook);
            const notify = document.getElementById('setting-notify');
            if (notify) notify.checked = !!s.notify_on_unmatched;
        } catch (e) { this.toast('Failed to load settings: ' + e.message); }
    },

    async saveSettings(e) {
        if (e) e.preventDefault();
        const payload = {
            tolerance_cents: parseInt(document.getElementById('setting-tolerance').value, 10),
            date_window_days: parseInt(document.getElementById('setting-window').value, 10),
            auto_approve_under: document.getElementById('setting-auto-approve').value,
            notify_on_unmatched: document.getElementById('setting-notify').checked,
            webhook: document.getElementById('setting-webhook').value
        };
        try {
            this.state.settings = await this.api('/api/banking/settings', { method: 'PUT', body: JSON.stringify(payload) });
            this.toast('Settings saved');
        } catch (err) { this.toast('Save failed: ' + err.message); }
    },

    async resetSettings() {
        try {
            this.state.settings = await this.api('/api/banking/settings/reset', { method: 'POST' });
            this.loadSettings();
            this.toast('Settings reset to defaults');
        } catch (e) { this.toast('Reset failed: ' + e.message); }
    },

    // ---------------- Helpers ----------------
    viewTransaction(id) {
        const tx = this.state.transactions.find(t => t.id === id);
        if (tx) this.toast('Transaction ' + id + ': ' + (tx.description || ''));
    },

    openModal(id) { const el = document.getElementById(id); if (el) el.style.display = 'flex'; },
    closeModal(id) { const el = document.getElementById(id); if (el) el.style.display = 'none'; },

    toast(msg) {
        let t = document.getElementById('banking-toast');
        if (!t) {
            t = document.createElement('div');
            t.id = 'banking-toast';
            t.style.cssText = 'position:fixed;bottom:24px;right:24px;background:#1f2937;color:#fff;padding:10px 16px;border-radius:8px;z-index:9999;font-size:13px;box-shadow:0 4px 12px rgba(0,0,0,.3)';
            document.body.appendChild(t);
        }
        t.textContent = msg;
        t.style.display = 'block';
        clearTimeout(this._toastTimer);
        this._toastTimer = setTimeout(() => { t.style.display = 'none'; }, 2600);
    },

    fmtBRL(v) { return new Intl.NumberFormat('pt-BR', { style: 'currency', currency: 'BRL' }).format(v || 0); },
    fmtDate(s) { try { return new Date(s).toLocaleString('pt-BR'); } catch { return s; } },
    esc(s) { const d = document.createElement('div'); d.textContent = s == null ? '' : String(s); return d.innerHTML; }
};

window.BankingApp = BankingApp;

(function() {
    const cb = () => BankingApp.init();
    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', cb);
    else cb();
})();
})();
