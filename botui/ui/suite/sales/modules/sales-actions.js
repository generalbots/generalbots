(function () {
'use strict';
    const SalesApp = window.SalesApp;
    Object.assign(SalesApp, {
    showNewLead() {
        this.showNewDeal();
        const stage = document.getElementById('dealStage');
        if (stage) stage.value = 'new';
    },

    async importLeads() {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.csv';
        input.onchange = async () => {
            const file = input.files && input.files[0];
            if (!file) return;
            const text = await file.text();
            const rows = text.split(/\r?\n/).filter(Boolean);
            if (rows.length < 2) { this.toast('CSV must contain a header row and at least one deal'); return; }
            const headers = rows[0].split(',').map(h => h.trim().replace(/^"|"$/g, ''));
            let created = 0;
            for (let i = 1; i < rows.length; i++) {
                const cells = this.parseCsvRow(rows[i]);
                const rec = {};
                headers.forEach((h, idx) => rec[h] = cells[idx]);
                try {
                    await this.api('/api/sales/deals', {
                        method: 'POST',
                        body: JSON.stringify({
                            title: rec.title || rec.name || rec['Deal'] || ('Imported deal ' + (i + 1)),
                            value: parseFloat(rec.value || rec.amount) || null,
                            currency: rec.currency || 'USD',
                            stage: rec.stage || 'new',
                            probability: parseInt(rec.probability || '20', 10),
                            expected_close_date: rec['close date'] || rec.close_date || null,
                            notes: rec.notes || null,
                        }),
                    });
                    created++;
                } catch (err) { console.error('Import row failed:', err); }
            }
            this.toast('Imported ' + created + ' of ' + (rows.length - 1) + ' deals');
            this.loadPipeline();
            this.loadForecast();
        };
        input.click();
    },

    parseCsvRow(row) {
        const out = [];
        let cur = '', inQuotes = false;
        for (let i = 0; i < row.length; i++) {
            const ch = row[i];
            if (inQuotes) {
                if (ch === '"') { if (row[i + 1] === '"') { cur += '"'; i++; } else { inQuotes = false; } }
                else cur += ch;
            } else if (ch === '"') { inQuotes = true; }
            else if (ch === ',') { out.push(cur); cur = ''; }
            else cur += ch;
        }
        out.push(cur);
        return out;
    },

    exportLeads() {
        const rows = [['title', 'value', 'currency', 'stage', 'probability', 'expected_close_date', 'notes']];
        this.state.deals.forEach(d => rows.push([
            d.title || d.name || '',
            d.value != null ? d.value : '',
            d.currency || 'USD',
            d.stage || '',
            d.probability != null ? d.probability : '',
            d.expected_close_date || '',
            d.notes || '',
        ]));
        const csv = rows.map(r => r.map(c => '"' + String(c).replace(/"/g, '""') + '"').join(',')).join('\n');
        const blob = new Blob([csv], { type: 'text/csv' });
        const a = document.createElement('a');
        a.href = URL.createObjectURL(blob);
        a.download = 'sales-deals.csv';
        a.click();
        URL.revokeObjectURL(a.href);
    },

    async showNewContact() {
        const name = prompt('Contact name');
        if (!name) return;
        const email = prompt('Contact email (optional)');
        const parts = name.trim().split(/\s+/);
        try {
            await this.api('/api/crm/contacts', {
                method: 'POST',
                body: JSON.stringify({ first_name: parts[0], last_name: parts.slice(1).join(' '), email: email || null }),
            });
            this.loadContacts();
        } catch (err) { console.error('Create contact failed:', err); this.toast('Failed to create contact'); }
    },

    async showLogActivity() {
        const type = prompt('Activity type (call, email, meeting, task)');
        if (!type) return;
        const subject = prompt('Subject');
        if (!subject) return;
        try {
            await this.api('/api/crm/activities', {
                method: 'POST',
                body: JSON.stringify({ activity_type: type, subject: subject }),
            });
            this.loadActivities();
        } catch (err) { console.error('Log activity failed:', err); this.toast('Failed to log activity'); }
    },

    async generateReport(kind) {
        const grid = document.getElementById('report-grid');
        if (!grid) return;
        const deals = this.state.deals;
        if (kind === 'pipeline') {
            grid.innerHTML = this.stages.map(stage => {
                const list = deals.filter(d => d.stage === stage);
                const value = list.reduce((s, d) => s + (d.value || 0), 0);
                return `<div class="report-card"><h3>${this.esc(this.stageLabels[stage] || stage)}</h3><div class="report-value">${list.length} deals</div><div class="report-value">${this.fmt(value)}</div></div>`;
            }).join('');
        } else if (kind === 'rep') {
            const byOwner = {};
            deals.forEach(d => { const k = d.owner_id || 'unassigned'; byOwner[k] = byOwner[k] || []; byOwner[k].push(d); });
            grid.innerHTML = Object.keys(byOwner).map(k => {
                const list = byOwner[k];
                const value = list.reduce((s, d) => s + (d.value || 0), 0);
                return `<div class="report-card"><h3>${this.esc(k)}</h3><div class="report-value">${list.length} deals</div><div class="report-value">${this.fmt(value)}</div></div>`;
            }).join('') || '<div class="loading-row">No deals to report</div>';
        } else {
            grid.innerHTML = `<div class="report-card"><h3>Deal Velocity</h3><div class="report-value">${deals.length} deals</div><div class="report-value">${this.fmt(deals.reduce((s, d) => s + (d.value || 0), 0))} total</div></div>`;
        }
    },

    toast(msg) {
        let t = document.getElementById('sales-toast');
        if (!t) {
            t = document.createElement('div');
            t.id = 'sales-toast';
            t.style.cssText = 'position:fixed;top:20px;left:50%;transform:translateX(-50%);background:var(--accent,#d4f505);color:#0a0a0a;padding:10px 20px;border-radius:8px;z-index:99999;font-weight:500;box-shadow:0 4px 12px rgba(0,0,0,.3)';
            document.body.appendChild(t);
        }
        t.textContent = msg;
        t.style.display = 'block';
        clearTimeout(t._timer);
        t._timer = setTimeout(() => { t.style.display = 'none'; }, 3000);
    },

    fmt(v) { return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(v || 0); },
    esc(s) { const d = document.createElement('div'); d.textContent = s == null ? '' : s; return d.innerHTML; }

    });
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", function () { SalesApp.init(); });
    } else {
        SalesApp.init();
    }
})();
