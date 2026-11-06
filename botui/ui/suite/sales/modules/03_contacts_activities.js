/* Sales app — module 3/5: contacts, activities and forecast (split from sales/sales-app.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
    async loadContacts() {
        try {
            const data = await this.api('/api/sales/contacts');
            this.state.contacts = Array.isArray(data) ? data : (data.items || []);
            this.renderContacts();
        } catch (e) {
            const tbody = document.getElementById('contacts-body');
            if (tbody) tbody.innerHTML = '<tr><td colspan="6" class="loading-row">Failed to load contacts</td></tr>';
        }
    },

    contactName(c) {
        return [c.first_name, c.last_name].filter(Boolean).join(' ') || c.email || 'Unnamed contact';
    },

    renderContacts() {
        const tbody = document.getElementById('contacts-body');
        if (!tbody) return;
        const contacts = this.state.contacts;
        if (!contacts.length) {
            tbody.innerHTML = '<tr><td colspan="6" class="loading-row">No contacts</td></tr>';
            return;
        }
        tbody.innerHTML = contacts.map(c => `
            <tr>
                <td style="font-weight:500">${this.esc(this.contactName(c))}</td>
                <td>${this.esc(c.company || '-')}</td>
                <td>${this.esc(c.email || '-')}</td>
                <td>${this.esc(c.phone || '-')}</td>
                <td>${this.esc((c.updated_at || '').slice(0, 10))}</td>
                <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px" onclick="SalesApp.newDealForContact('${c.id}')">New deal</button></td>
            </tr>
        `).join('');
    },

    filterContacts(q) {
        const query = (q || '').toLowerCase();
        const filtered = this.state.contacts.filter(c =>
            this.contactName(c).toLowerCase().includes(query) ||
            (c.company || '').toLowerCase().includes(query) ||
            (c.email || '').toLowerCase().includes(query)
        );
        const tbody = document.getElementById('contacts-body');
        if (!tbody) return;
        if (!filtered.length) { tbody.innerHTML = '<tr><td colspan="6" class="loading-row">No matches</td></tr>'; return; }
        tbody.innerHTML = filtered.map(c => `
            <tr>
                <td style="font-weight:500">${this.esc(this.contactName(c))}</td>
                <td>${this.esc(c.company || '-')}</td>
                <td>${this.esc(c.email || '-')}</td>
                <td>${this.esc(c.phone || '-')}</td>
                <td>${this.esc((c.updated_at || '').slice(0, 10))}</td>
                <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px" onclick="SalesApp.newDealForContact('${c.id}')">New deal</button></td>
            </tr>
        `).join('');
    },

    async loadActivities() {
        try {
            const data = await this.api('/api/sales/activities');
            this.state.activities = Array.isArray(data) ? data : (data.items || []);
            this.renderActivities();
        } catch (e) {
            const list = document.getElementById('activities-list');
            if (list) list.innerHTML = '<div class="loading-row">Failed to load activities</div>';
        }
    },

    renderActivities() {
        const list = document.getElementById('activities-list');
        if (!list) return;
        const acts = this.state.activities;
        if (!acts.length) { list.innerHTML = '<div class="loading-row">No upcoming activities</div>'; return; }
        list.innerHTML = acts.map(a => `
            <div class="activity-item">
                <div class="activity-icon ${a.activity_type || 'task'}">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">${a.activity_type === 'call' ? '<path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72"/>' : a.activity_type === 'meeting' ? '<rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>' : '<path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>'}</svg>
                </div>
                <div class="activity-body">
                    <span class="activity-title">${this.esc(a.subject)}</span>
                    <span class="activity-meta">${this.esc(a.activity_type || '')} ${a.due_date ? '- ' + this.esc(a.due_date.slice(0, 10)) : ''}</span>
                </div>
                <div class="activity-actions">
                    <span class="badge ${a.activity_type || 'task'}">${this.esc(a.activity_type || 'task')}</span>
                </div>
            </div>
        `).join('');
    },

    async loadForecast() {
        try {
            const data = await this.api('/api/sales/forecast');
            this.state.forecast = data || {};
            this.renderForecast();
        } catch (e) {
            const grid = document.getElementById('forecast-grid');
            if (grid) grid.innerHTML = '<div class="loading-row">Failed to load forecast</div>';
        }
    },

    renderForecast() {
        const grid = document.getElementById('forecast-grid');
        if (!grid) return;
        const f = this.state.forecast;
        const deals = this.state.deals;
        const weighted = deals.reduce((s, d) => s + (d.value || 0) * (d.probability || 0) / 100, 0);
        const won = deals.filter(d => d.stage === 'won');
        const winRate = deals.length ? Math.round(won.length / deals.length * 100) : 0;
        const avg = deals.length ? deals.reduce((s, d) => s + (d.value || 0), 0) / deals.length : 0;
        const overall = f.overall_weighted || weighted;
        grid.innerHTML = `
            <div class="forecast-card">
                <span class="forecast-card-title">Weighted Pipeline Value</span>
                <span class="forecast-card-value accent">${this.fmt(overall)}</span>
                <span class="forecast-card-detail">Based on stage probabilities</span>
                <div class="forecast-bar"><div class="forecast-bar-fill accent" style="width:${overall ? 60 : 0}%"></div></div>
            </div>
            <div class="forecast-card">
                <span class="forecast-card-title">Win Rate</span>
                <span class="forecast-card-value success">${winRate}%</span>
                <span class="forecast-card-detail">${won.length} won of ${deals.length} total</span>
                <div class="forecast-bar"><div class="forecast-bar-fill success" style="width:${winRate}%"></div></div>
            </div>
            <div class="forecast-card">
                <span class="forecast-card-title">Average Deal Size</span>
                <span class="forecast-card-value">${this.fmt(avg)}</span>
                <span class="forecast-card-detail">${deals.length} deals in pipeline</span>
            </div>
            <div class="forecast-card">
                <span class="forecast-card-title">Predicted Revenue</span>
                <span class="forecast-card-value">${this.fmt(f.overall_predicted)}</span>
                <span class="forecast-card-detail">Confidence ${Math.round(f.overall_confidence || 0)}%</span>
            </div>
            <div class="forecast-card">
                <span class="forecast-card-title">Open Pipeline</span>
                <span class="forecast-card-value">${this.fmt(deals.filter(d => !['won', 'lost'].includes(d.stage)).reduce((s, d) => s + (d.value || 0), 0))}</span>
                <span class="forecast-card-detail">Active deals</span>
            </div>
            <div class="forecast-card">
                <span class="forecast-card-title">Pipeline Velocity</span>
                <span class="forecast-card-value">${f.velocity || deals.length || 0}</span>
                <span class="forecast-card-detail">Deals in pipeline</span>
            </div>
        `;
    }
});
})(window.SalesApp);
