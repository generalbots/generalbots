(function() {
'use strict';
const SalesApp = {
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
    },

    async loadPipeline() {
        try {
            const data = await this.api('/api/sales/deals');
            this.state.deals = Array.isArray(data) ? data : (data.items || []);
            this.renderPipeline();
        } catch (e) {
            const container = document.getElementById('pipeline-container');
            if (container) container.innerHTML = '<div class="loading-row">Failed to load pipeline</div>';
        }
    },

    renderPipeline() {
        const container = document.getElementById('pipeline-container');
        if (!container) return;
        const deals = this.state.deals;
        const grouped = {};
        this.stages.forEach(s => grouped[s] = deals.filter(d => d.stage === s));
        container.innerHTML = this.stages.map(stage => `
            <div class="pipeline-column ${stage}" data-stage="${stage}">
                <div class="pipeline-header">
                    <span class="pipeline-title">${this.esc(this.stageLabels[stage] || stage)}</span>
                    <span class="pipeline-count">${grouped[stage].length}</span>
                </div>
                <div class="pipeline-cards" data-stage="${stage}">
                    ${grouped[stage].map(d => this.renderDealCard(d)).join('')}
                </div>
            </div>
        `).join('');
        this.renderPipelineSummary();
        this.setupDragDrop();
        this.renderStatCards();
    },

    renderDealCard(d) {
        const initial = (d.owner_id || d.contact_id || '?').toString().slice(0, 1).toUpperCase();
        return `
            <div class="deal-card" draggable="true" data-id="${d.id}">
                <div class="deal-card-header">
                    <span class="deal-card-title">${this.esc(d.title || d.name || 'Untitled deal')}</span>
                    <span class="deal-card-value">${this.fmt(d.value)}</span>
                </div>
                <div class="deal-card-meta">
                    <span class="deal-card-owner"><span class="deal-card-avatar">${initial}</span>${this.esc(d.currency || 'USD')}</span>
                    <span>${d.expected_close_date ? this.esc(d.expected_close_date) : ''}</span>
                </div>
                <div class="deal-card-actions">
                    <button class="btn-secondary" style="padding:2px 8px;font-size:11px" onclick="SalesApp.editDeal('${d.id}')">Edit</button>
                    <button class="btn-secondary" style="padding:2px 8px;font-size:11px;color:#ef4444" onclick="SalesApp.deleteDeal('${d.id}')">Delete</button>
                </div>
            </div>
        `;
    },

    renderPipelineSummary() {
        const el = document.getElementById('pipeline-summary');
        if (!el) return;
        const deals = this.state.deals;
        const total = deals.reduce((s, d) => s + (d.value || 0), 0);
        const won = deals.filter(d => d.stage === 'won');
        const active = deals.filter(d => !['won', 'lost'].includes(d.stage));
        const avg = deals.length ? total / deals.length : 0;
        const weighted = deals.reduce((s, d) => s + (d.value || 0) * (d.probability || 0) / 100, 0);
        el.innerHTML = `
            <div class="summary-card"><span class="summary-label">Total Pipeline</span><span class="summary-value">${this.fmt(total)}</span></div>
            <div class="summary-card"><span class="summary-label">Weighted</span><span class="summary-value">${this.fmt(weighted)}</span></div>
            <div class="summary-card"><span class="summary-label">Active Deals</span><span class="summary-value">${active.length}</span></div>
            <div class="summary-card"><span class="summary-label">Won Deals</span><span class="summary-value success">${won.length}</span></div>
            <div class="summary-card"><span class="summary-label">Avg Deal Size</span><span class="summary-value">${this.fmt(avg)}</span></div>
            <div class="summary-card"><span class="summary-label">Won Value</span><span class="summary-value success">${this.fmt(won.reduce((s, d) => s + (d.value || 0), 0))}</span></div>
        `;
    },

    renderStatCards() {
        const deals = this.state.deals;
        const total = deals.reduce((s, d) => s + (d.value || 0), 0);
        const won = deals.filter(d => d.stage === 'won');
        const wonValue = won.reduce((s, d) => s + (d.value || 0), 0);
        const winRate = deals.length ? Math.round(won.length / deals.length * 100) : 0;
        const avg = deals.length ? total / deals.length : 0;
        const weighted = deals.reduce((s, d) => s + (d.value || 0) * (d.probability || 0) / 100, 0);
        const set = (id, v) => { const el = document.getElementById(id); if (el) el.textContent = v; };
        set('sales-stat-pipeline', this.fmt(total));
        set('sales-stat-won', this.fmt(wonValue));
        set('sales-stat-winrate', winRate + '%');
        set('sales-stat-avgsize', this.fmt(avg));
        set('sales-stat-forecast', this.fmt(weighted));
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
                if (!id || id === 'null') return;
                try {
                    await this.api('/api/sales/deals/' + id, { method: 'PATCH', body: JSON.stringify({ stage: newStage }) });
                    this.loadPipeline();
                    this.loadForecast();
                } catch (err) { console.error('Stage move failed:', err); }
            });
        });
        document.querySelectorAll('.deal-card').forEach(card => {
            card.addEventListener('dragstart', e => e.dataTransfer.setData('text/plain', card.dataset.id));
        });
    },

    filterDeals(q) {
        const query = (q || '').toLowerCase();
        const deals = query ? this.state.deals.filter(d =>
            (d.title || '').toLowerCase().includes(query) ||
            (d.name || '').toLowerCase().includes(query)
        ) : this.state.deals;
        const container = document.getElementById('pipeline-container');
        if (!container) return;
        const grouped = {};
        this.stages.forEach(s => grouped[s] = deals.filter(d => d.stage === s));
        container.innerHTML = this.stages.map(stage => `
            <div class="pipeline-column ${stage}" data-stage="${stage}">
                <div class="pipeline-header">
                    <span class="pipeline-title">${this.esc(this.stageLabels[stage] || stage)}</span>
                    <span class="pipeline-count">${grouped[stage].length}</span>
                </div>
                <div class="pipeline-cards" data-stage="${stage}">
                    ${grouped[stage].map(d => this.renderDealCard(d)).join('')}
                </div>
            </div>
        `).join('');
        this.setupDragDrop();
    },

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
    },

    showNewDeal() {
        this.state.editingId = null;
        const modal = document.getElementById('salesDealModal');
        if (!modal) return;
        document.getElementById('salesDealModalTitle').textContent = 'New deal';
        ['dealTitle', 'dealAccount', 'dealContact', 'dealValue', 'dealStage', 'dealCloseDate', 'dealProbability', 'dealOwner', 'dealNotes'].forEach(id => {
            const el = document.getElementById(id);
            if (el) el.value = (id === 'dealProbability') ? '20' : '';
        });
        modal.style.display = 'flex';
    },

    newDealForContact(contactId) {
        this.showNewDeal();
        const el = document.getElementById('dealContact');
        if (el) el.value = contactId;
    },

    closeDealModal() {
        const modal = document.getElementById('salesDealModal');
        if (modal) modal.style.display = 'none';
    },

    editDeal(id) {
        const deal = this.state.deals.find(d => d.id === id);
        if (!deal) return;
        this.state.editingId = id;
        const modal = document.getElementById('salesDealModal');
        if (!modal) return;
        document.getElementById('salesDealModalTitle').textContent = 'Edit deal';
        const set = (id2, v) => { const el = document.getElementById(id2); if (el) el.value = v; };
        set('dealTitle', deal.title || deal.name || '');
        set('dealValue', deal.value != null ? deal.value : '');
        set('dealStage', deal.stage || 'new');
        set('dealCloseDate', deal.expected_close_date || '');
        set('dealProbability', deal.probability != null ? deal.probability : '20');
        set('dealOwner', deal.owner_id || '');
        set('dealNotes', deal.notes || '');
        set('dealContact', deal.contact_id || '');
        modal.style.display = 'flex';
    },

    async deleteDeal(id) {
        if (!confirm('Delete this deal?')) return;
        try {
            await this.api('/api/sales/deals/' + id, { method: 'DELETE' });
            this.state.deals = this.state.deals.filter(d => d.id !== id);
            this.renderPipeline();
            this.loadForecast();
        } catch (err) { console.error('Delete failed:', err); this.toast('Failed to delete deal'); }
    },

    async saveDeal() {
        const title = document.getElementById('dealTitle').value.trim();
        if (!title) { this.toast('Deal title is required'); return; }
        const value = parseFloat(document.getElementById('dealValue').value);
        if (document.getElementById('dealValue').value && (isNaN(value) || value < 0)) {
            this.toast('Amount must be a non-negative number');
            return;
        }
        const payload = {
            title: title,
            name: title,
            value: isNaN(value) ? null : value,
            currency: 'USD',
            stage: document.getElementById('dealStage').value || 'new',
            probability: parseInt(document.getElementById('dealProbability').value || '20', 10),
            expected_close_date: document.getElementById('dealCloseDate').value || null,
            notes: document.getElementById('dealNotes').value || null,
            contact_id: document.getElementById('dealContact').value || null,
        };
        try {
            if (this.state.editingId) {
                await this.api('/api/sales/deals/' + this.state.editingId, { method: 'PATCH', body: JSON.stringify(payload) });
            } else {
                await this.api('/api/sales/deals', { method: 'POST', body: JSON.stringify(payload) });
            }
            this.closeDealModal();
            this.loadPipeline();
            this.loadForecast();        } catch (err) { console.error('Save failed:', err); this.toast('Failed to save deal'); }
    },

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
    viewOrder(id) { this.toast('Order ' + id); },
};
window.SalesApp = SalesApp;
})();
