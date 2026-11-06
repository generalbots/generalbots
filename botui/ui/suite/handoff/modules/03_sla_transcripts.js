/* Handoff app — module 3/5: SLA, transcripts and CSAT detail tables (split from handoff/handoff.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
    async loadSLA() {
        try {
            const data = await this.api('/api/handoff/sla');
            this.renderSLA(data);
        } catch (e) {
            document.getElementById('sla-grid').innerHTML = '<div class="loading-row">Failed to load SLA</div>';
        }
    },

    renderSLA(sla) {
        const grid = document.getElementById('sla-grid');
        if (!grid) return;
        const fr = sla.first_response_pct || 0;
        const res = sla.resolution_pct || 0;
        const csat = sla.csat_pct || 0;
        grid.innerHTML = `
            <article class="handoff-sla-card ${fr >= 95 ? 'handoff-sla-card--ok' : 'handoff-sla-card--warn'}">
                <h3>First response &lt; 30s</h3>
                <p class="handoff-sla-percentage">${fr}%</p>
                <small>Target 95%</small>
            </article>
            <article class="handoff-sla-card ${res >= 90 ? 'handoff-sla-card--ok' : 'handoff-sla-card--warn'}">
                <h3>Resolution &lt; 4h</h3>
                <p class="handoff-sla-percentage">${res}%</p>
                <small>Target 90%</small>
            </article>
            <article class="handoff-sla-card ${csat >= 85 ? 'handoff-sla-card--ok' : 'handoff-sla-card--warn'}">
                <h3>CSAT &gt; 4.0</h3>
                <p class="handoff-sla-percentage">${csat}%</p>
                <small>Target 85%</small>
            </article>`;
        const breachBody = document.getElementById('sla-breach-body');
        if (breachBody) {
            const breaches = sla.breaches || [];
            breachBody.innerHTML = breaches.length ? breaches.map(b => `
                <tr>
                    <td>${b.id}</td>
                    <td>${this.esc(b.user_name || '')}</td>
                    <td>—</td>
                    <td>${this.esc(b.channel || '')}</td>
                    <td>${this.fmtTime(b.elapsed_seconds || 0)}</td>
                    <td>First response</td>
                    <td><span class="badge disconnected">Breached</span></td>
                </tr>`).join('') : '<tr><td colspan="7" class="empty-state"><p>No active SLA breaches</p></td></tr>';
        }
    },

    async loadTranscripts() {
        try {
            const data = await this.api('/api/handoff/transcripts');
            const items = Array.isArray(data) ? data : (data.items || []);
            const tbody = document.getElementById('transcripts-body');
            if (!tbody) return;
            tbody.innerHTML = items.length ? items.map(t => `
                <tr>
                    <td>${this.esc((t.created_at || '').slice(0, 10))}</td>
                    <td>${this.esc(t.customer || '')}</td>
                    <td>${this.esc(t.agent || '')}</td>
                    <td>${this.esc(t.channel || '')}</td>
                    <td>${this.fmtTime(t.duration_seconds)}</td>
                    <td>${t.messages || 0}</td>
                    <td>${this.esc(t.outcome || '')}</td>
                    <td><button class="btn-sm" onclick="HandoffApp.showFeedback('Transcript view not available','error')">View</button></td>
                </tr>`).join('') : '<tr><td colspan="8" class="empty-state"><p>No transcripts yet</p></td></tr>';
        } catch (e) {
            const tbody = document.getElementById('transcripts-body');
            if (tbody) tbody.innerHTML = '<tr><td colspan="8" class="loading-row">Failed to load transcripts</td></tr>';
        }
    },

    async loadDeflection() {
        try {
            const data = await this.api('/api/handoff/deflection');
            const items = Array.isArray(data) ? data : (data.items || []);
            const tbody = document.getElementById('deflection-body');
            if (!tbody) return;
            tbody.innerHTML = items.length ? items.map(d => `
                <tr>
                    <td>${this.esc(d.reason)}</td>
                    <td>${d.count || 0}</td>
                    <td>—</td>
                </tr>`).join('') : '<tr><td colspan="3" class="empty-state"><p>No deflection data</p></td></tr>';
        } catch (e) {
            const tbody = document.getElementById('deflection-body');
            if (tbody) tbody.innerHTML = '<tr><td colspan="3" class="loading-row">Failed to load</td></tr>';
        }
    },

    async loadCsatDetail() {
        try {
            const data = await this.api('/api/handoff/csat');
            const items = Array.isArray(data) ? data : (data.items || []);
            const tbody = document.getElementById('csat-body');
            if (!tbody) return;
            tbody.innerHTML = items.length ? items.map(r => `
                <tr>
                    <td>${this.esc((r.submitted_at || '').slice(0, 10))}</td>
                    <td>—</td>
                    <td>—</td>
                    <td>—</td>
                    <td>${r.rating || 0} / 5</td>
                    <td>${this.esc(r.comment || 'No comment')}</td>
                </tr>`).join('') : '<tr><td colspan="6" class="empty-state"><p>No CSAT responses</p></td></tr>';
        } catch (e) {
            const tbody = document.getElementById('csat-body');
            if (tbody) tbody.innerHTML = '<tr><td colspan="6" class="loading-row">Failed to load</td></tr>';
        }
    },

    async loadAnalyticsCharts() {
        try {
            const data = await this.api('/api/handoff/analytics');
            const analytics = Array.isArray(data) ? data : (data.items || []);
            const el = document.getElementById('analytics-charts');
            if (!el) return;
            const total = analytics.reduce((s, a) => s + (a.total_transfers || 0), 0);
            const avgWait = analytics.length ? analytics.reduce((s, a) => s + (a.avg_wait_seconds || 0), 0) / analytics.length : 0;
            el.innerHTML = `
                <div class="analytics-card">
                    <span class="analytics-card-title">Total transfers (tracked)</span>
                    <span class="analytics-card-value accent">${total}</span>
                </div>
                <div class="analytics-card">
                    <span class="analytics-card-title">Avg wait (tracked)</span>
                    <span class="analytics-card-value">${this.fmtTime(avgWait)}</span>
                </div>`;
        } catch (e) {
            const el = document.getElementById('analytics-charts');
            if (el) el.innerHTML = '<div class="loading-row">Failed to load charts</div>';
        }
    }
});
})(window.HandoffApp);
