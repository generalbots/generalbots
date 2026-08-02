(function() {
'use strict';
const SalesApp = {
    stages: ['lead', 'qualified', 'proposal', 'negotiation', 'won', 'lost'],
    state: { deals: [], contacts: [], activities: [], forecast: {} },

    init() {
        document.querySelectorAll('.sales-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.sales-tab').forEach(t => t.classList.remove('active'));
                document.querySelectorAll('.sales-view').forEach(v => v.classList.remove('active'));
                tab.classList.add('active');
                document.getElementById(tab.dataset.view + '-view').classList.add('active');
            });
        });
        document.getElementById('salesSearch').addEventListener('input', e => this.filterContacts(e.target.value));
        this.loadPipeline();
        this.loadContacts();
        this.loadActivities();
        this.loadForecast();
    },

    async api(path, opts) {
        const token = localStorage.getItem('gb_token');
        const res = await fetch(path, { headers: { 'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json' }, ...opts });
        if (!res.ok) throw new Error('API error: ' + res.status);
        return res.json();
    },

    async loadPipeline() {
        try {
            const data = await this.api('/api/sales/deals');
            this.state.deals = Array.isArray(data) ? data : (data.items || []);
            this.renderPipeline();
        } catch (e) {
            document.getElementById('pipeline-container').innerHTML = '<div class="loading-row">Failed to load pipeline</div>';
        }
    },

    renderPipeline() {
        const container = document.getElementById('pipeline-container');
        const deals = this.state.deals;
        const grouped = {};
        this.stages.forEach(s => grouped[s] = deals.filter(d => d.stage === s));
        container.innerHTML = this.stages.map(stage => `
            <div class="pipeline-column ${stage}" data-stage="${stage}">
                <div class="pipeline-header">
                    <span class="pipeline-title">${stage.charAt(0).toUpperCase() + stage.slice(1)}</span>
                    <span class="pipeline-count">${grouped[stage].length}</span>
                </div>
                <div class="pipeline-cards" data-stage="${stage}">
                    ${grouped[stage].map(d => this.renderDealCard(d)).join('')}
                </div>
            </div>
        `).join('');
        this.renderPipelineSummary();
        this.setupDragDrop();
    },

    renderDealCard(d) {
        const initial = (d.contact_name || d.owner || '?').charAt(0).toUpperCase();
        return `
            <div class="deal-card" draggable="true" data-id="${d.id}">
                <div class="deal-card-header">
                    <span class="deal-card-title">${this.esc(d.title || d.name)}</span>
                    <span class="deal-card-value">${this.fmt(d.value)}</span>
                </div>
                <div class="deal-card-company">${this.esc(d.company || d.account || '')}</div>
                <div class="deal-card-meta">
                    <span class="deal-card-owner"><span class="deal-card-avatar">${initial}</span>${this.esc(d.contact_name || d.owner || '')}</span>
                    <span>${this.esc(d.close_date || '')}</span>
                </div>
            </div>
        `;
    },

    renderPipelineSummary() {
        const deals = this.state.deals;
        const total = deals.reduce((s, d) => s + (d.value || 0), 0);
        const won = deals.filter(d => d.stage === 'won');
        const active = deals.filter(d => !['won', 'lost'].includes(d.stage));
        const avg = deals.length ? total / deals.length : 0;
        document.getElementById('pipeline-summary').innerHTML = `
            <div class="summary-card"><span class="summary-label">Total Pipeline</span><span class="summary-value">${this.fmt(total)}</span></div>
            <div class="summary-card"><span class="summary-label">Active Deals</span><span class="summary-value">${active.length}</span></div>
            <div class="summary-card"><span class="summary-label">Won Deals</span><span class="summary-value success">${won.length}</span></div>
            <div class="summary-card"><span class="summary-label">Avg Deal Size</span><span class="summary-value">${this.fmt(avg)}</span></div>
            <div class="summary-card"><span class="summary-label">Won Value</span><span class="summary-value success">${this.fmt(won.reduce((s, d) => s + (d.value || 0), 0))}</span></div>
        `;
    },

    setupDragDrop() {
        document.querySelectorAll('.pipeline-cards').forEach(col => {
            col.addEventListener('dragover', e => { e.preventDefault(); col.classList.add('drag-over'); });
            col.addEventListener('dragleave', () => col.classList.remove('drag-over'));
            col.addEventListener('drop', async e => {
                e.preventDefault();
                col.classList.remove('drag-over');
                const id = e.dataTransfer.getData('text/plain');
                const newStage = col.closest('.pipeline-column').dataset.stage;
                try {
                    await this.api('/api/sales/deals/' + id, { method: 'PATCH', body: JSON.stringify({ stage: newStage }) });
                    this.loadPipeline();
                } catch (err) { console.error(err); }
            });
        });
        document.querySelectorAll('.deal-card').forEach(card => {
            card.addEventListener('dragstart', e => e.dataTransfer.setData('text/plain', card.dataset.id));
        });
    },

    async loadContacts() {
        try {
            const data = await this.api('/api/sales/contacts');
            this.state.contacts = Array.isArray(data) ? data : (data.items || []);
            this.renderContacts();
        } catch (e) {
            document.getElementById('contacts-body').innerHTML = '<tr><td colspan="6" class="loading-row">Failed to load contacts</td></tr>';
        }
    },

    renderContacts() {
        const contacts = this.state.contacts;
        if (!contacts.length) {
            document.getElementById('contacts-body').innerHTML = '<tr><td colspan="6" class="loading-row">No contacts</td></tr>';
            return;
        }
        document.getElementById('contacts-body').innerHTML = contacts.map(c => `
            <tr>
                <td style="font-weight:500">${this.esc(c.name)}</td>
                <td>${this.esc(c.company || '-')}</td>
                <td>${this.esc(c.email || '-')}</td>
                <td>${this.esc(c.last_contact || '-')}</td>
                <td style="font-weight:600;color:var(--accent,#d4f505)">${this.fmt(c.deal_value)}</td>
                <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px">View</button></td>
            </tr>
        `).join('');
    },

    filterContacts(q) {
        const query = q.toLowerCase();
        const filtered = this.state.contacts.filter(c =>
            (c.name || '').toLowerCase().includes(query) ||
            (c.company || '').toLowerCase().includes(query) ||
            (c.email || '').toLowerCase().includes(query)
        );
        const tbody = document.getElementById('contacts-body');
        if (!filtered.length) { tbody.innerHTML = '<tr><td colspan="6" class="loading-row">No matches</td></tr>'; return; }
        tbody.innerHTML = filtered.map(c => `
            <tr>
                <td style="font-weight:500">${this.esc(c.name)}</td>
                <td>${this.esc(c.company || '-')}</td>
                <td>${this.esc(c.email || '-')}</td>
                <td>${this.esc(c.last_contact || '-')}</td>
                <td style="font-weight:600;color:var(--accent,#d4f505)">${this.fmt(c.deal_value)}</td>
                <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px">View</button></td>
            </tr>
        `).join('');
    },

    async loadActivities() {
        try {
            const data = await this.api('/api/sales/activities');
            this.state.activities = Array.isArray(data) ? data : (data.items || []);
            this.renderActivities();
        } catch (e) {
            document.getElementById('activities-list').innerHTML = '<div class="loading-row">Failed to load activities</div>';
        }
    },

    renderActivities() {
        const acts = this.state.activities;
        if (!acts.length) { document.getElementById('activities-list').innerHTML = '<div class="loading-row">No upcoming activities</div>'; return; }
        document.getElementById('activities-list').innerHTML = acts.map(a => `
            <div class="activity-item">
                <div class="activity-icon ${a.type || 'task'}">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">${a.type === 'call' ? '<path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72"/>' : a.type === 'meeting' ? '<rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>' : '<path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>'}</svg>
                </div>
                <div class="activity-body">
                    <span class="activity-title">${this.esc(a.title || a.subject)}</span>
                    <span class="activity-meta">${this.esc(a.contact_name || '')} ${a.due_date ? '- ' + this.esc(a.due_date) : ''}</span>
                </div>
                <div class="activity-actions">
                    <span class="badge ${a.type || 'task'}">${this.esc(a.type || 'task')}</span>
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
            document.getElementById('forecast-grid').innerHTML = '<div class="loading-row">Failed to load forecast</div>';
        }
    },

    renderForecast() {
        const f = this.state.forecast;
        document.getElementById('forecast-grid').innerHTML = `
            <div class="forecast-card">
                <span class="forecast-card-title">Weighted Pipeline Value</span>
                <span class="forecast-card-value accent">${this.fmt(f.weighted_value)}</span>
                <span class="forecast-card-detail">Based on stage probabilities</span>
                <div class="forecast-bar"><div class="forecast-bar-fill accent" style="width:${f.weighted_pct || 0}%"></div></div>
            </div>
            <div class="forecast-card">
                <span class="forecast-card-title">Win Rate</span>
                <span class="forecast-card-value success">${f.win_rate || 0}%</span>
                <span class="forecast-card-detail">${f.won_count || 0} won of ${f.total_count || 0} total</span>
                <div class="forecast-bar"><div class="forecast-bar-fill success" style="width:${f.win_rate || 0}%"></div></div>
            </div>
            <div class="forecast-card">
                <span class="forecast-card-title">Average Deal Size</span>
                <span class="forecast-card-value">${this.fmt(f.avg_deal_size)}</span>
                <span class="forecast-card-detail">${f.total_count || 0} deals in pipeline</span>
            </div>
            <div class="forecast-card">
                <span class="forecast-card-title">Revenue Target</span>
                <span class="forecast-card-value">${this.fmt(f.revenue_target)}</span>
                <span class="forecast-card-detail">${f.target_pct || 0}% achieved</span>
                <div class="forecast-bar"><div class="forecast-bar-fill accent" style="width:${f.target_pct || 0}%"></div></div>
            </div>
            <div class="forecast-card">
                <span class="forecast-card-title">Pipeline Velocity</span>
                <span class="forecast-card-value">${f.velocity || 0}</span>
                <span class="forecast-card-detail">Deals per week</span>
            </div>
            <div class="forecast-card">
                <span class="forecast-card-title">Avg Sales Cycle</span>
                <span class="forecast-card-value">${f.avg_cycle_days || 0}d</span>
                <span class="forecast-card-detail">Days to close</span>
            </div>
        `;
    },

    showNewDeal() {
        alert('New deal form coming soon');
    },

    fmt(v) { return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(v || 0); },
    esc(s) { const d = document.createElement('div'); d.textContent = s; return d.innerHTML; }
};

(function(){ var __cb = () => SalesApp.init(); if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
})();
