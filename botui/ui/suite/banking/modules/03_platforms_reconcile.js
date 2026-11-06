/* Banking app — module 3/5: platforms and reconciliation (split from banking/banking.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
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
    }
});
})(window.BankingApp);
