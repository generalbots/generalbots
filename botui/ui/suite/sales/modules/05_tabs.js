/* Sales app — module 5/5: leads, quotes and orders tabs (split from sales/sales-app.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
    async loadLeads() {
        const tbody = document.getElementById('leads-body');
        if (!tbody) return;
        try {
            const data = await this.api('/api/sales/leads');
            const items = Array.isArray(data) ? data : (data.items || []);
            if (!items.length) { tbody.innerHTML = '<tr><td colspan="8" class="loading-row">No leads yet</td></tr>'; return; }
            tbody.innerHTML = items.map(l => `
                <tr>
                    <td style="font-weight:500">${this.esc(l.name || '-')}</td>
                    <td>${this.esc(l.company || '-')}</td>
                    <td>${this.esc(l.source || '-')}</td>
                    <td>${this.esc(l.score != null ? l.score : '-')}</td>
                    <td><span class="badge ${l.status === 'won' ? 'matched' : 'unmatched'}">${this.esc(l.status || 'new')}</span></td>
                    <td>${this.esc(l.owner || '-')}</td>
                    <td>${this.esc((l.created_at || '').slice(0, 10))}</td>
                    <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px" onclick="SalesApp.convertLead('${l.id}')">New deal</button></td>
                </tr>
            `).join('');
        } catch (e) {
            tbody.innerHTML = '<tr><td colspan="8" class="loading-row">Failed to load leads</td></tr>';
        }
    },

    async loadQuotes() {
        const tbody = document.getElementById('quotes-body');
        if (!tbody) return;
        try {
            const data = await this.api('/api/sales/quotes');
            const items = Array.isArray(data) ? data : (data.items || []);
            if (!items.length) { tbody.innerHTML = '<tr><td colspan="7" class="loading-row">No quotes yet</td></tr>'; return; }
            tbody.innerHTML = items.map(q => `
                <tr>
                    <td>${this.esc(q.quote_number || q.id)}</td>
                    <td>${this.esc(q.title || '-')}</td>
                    <td>${this.esc(q.customer || '-')}</td>
                    <td>${this.fmt(parseFloat(q.amount) || 0)}</td>
                    <td>${this.esc((q.valid_until || '').toString().slice(0, 10) || '-')}</td>
                    <td><span class="badge ${q.status === 'accepted' ? 'matched' : 'unmatched'}">${this.esc(q.status || 'draft')}</span></td>
                    <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px" onclick="SalesApp.viewQuote('${q.id}')">View</button></td>
                </tr>
            `).join('');
        } catch (e) {
            tbody.innerHTML = '<tr><td colspan="7" class="loading-row">Failed to load quotes</td></tr>';
        }
    },

    async loadOrders() {
        const tbody = document.getElementById('orders-body');
        if (!tbody) return;
        try {
            const data = await this.api('/api/sales/orders');
            const items = Array.isArray(data) ? data : (data.items || []);
            if (!items.length) { tbody.innerHTML = '<tr><td colspan="7" class="loading-row">No orders yet</td></tr>'; return; }
            tbody.innerHTML = items.map(o => `
                <tr>
                    <td>${this.esc(o.order_number || o.id)}</td>
                    <td>${this.esc(o.customer || '-')}</td>
                    <td>${this.esc(o.items != null ? o.items : '-')}</td>
                    <td>${this.fmt(parseFloat(o.total) || 0)}</td>
                    <td><span class="badge ${o.status === 'delivered' ? 'matched' : 'unmatched'}">${this.esc(o.status || 'pending')}</span></td>
                    <td>${this.esc(o.delivery || '-')}</td>
                    <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px" onclick="SalesApp.viewOrder('${o.id}')">View</button></td>
                </tr>
            `).join('');
        } catch (e) {
            tbody.innerHTML = '<tr><td colspan="7" class="loading-row">Failed to load orders</td></tr>';
        }
    },

    convertLead(id) {
        this.showNewDeal();
        const el = document.getElementById('dealAccount');
        if (el && id) el.value = id;
    },

    viewQuote(id) { this.toast('Quote ' + id); },
    viewOrder(id) { this.toast('Order ' + id); }
});
})(window.SalesApp);
