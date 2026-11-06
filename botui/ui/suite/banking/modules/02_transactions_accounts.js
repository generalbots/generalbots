/* Banking app — module 2/5: transactions, stats and accounts (split from banking/banking.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
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
    }
});
})(window.BankingApp);
