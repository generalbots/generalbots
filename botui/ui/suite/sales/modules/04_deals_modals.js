/* Sales app — module 4/5: deal modal actions (split from sales/sales-app.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
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
    }
});
})(window.SalesApp);
