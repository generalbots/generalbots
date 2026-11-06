/* Banking app — module 5/5: reports, settings, helpers and boot (split from banking/banking.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
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
});

(function() {
    const cb = () => app.init();
    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', cb);
    else cb();
})();
})(window.BankingApp);
