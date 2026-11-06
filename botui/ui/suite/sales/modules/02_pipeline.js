/* Sales app — module 2/5: pipeline board (split from sales/sales-app.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
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
    }
});
})(window.SalesApp);
