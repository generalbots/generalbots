(function() {
'use strict';
if (window.GBAppLifecycle) GBAppLifecycle.begin("handoff");
const HandoffApp = {
    state: { queue: [], analytics: {}, channels: [], csat: {} },

    init() {
        document.querySelectorAll('.handoff-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.handoff-tab').forEach(t => t.classList.remove('active'));
                document.querySelectorAll('.handoff-view').forEach(v => v.classList.remove('active'));
                tab.classList.add('active');
                document.getElementById(tab.dataset.view + '-view').classList.add('active');
            });
        });
        const search = document.getElementById('handoffSearch');
        if (search) search.addEventListener('input', () => this.applyQueueFilter());
        const channelFilter = document.getElementById('handoffChannelFilter');
        if (channelFilter) channelFilter.addEventListener('change', () => this.applyQueueFilter());
        this.loadAll();
    },

    async api(path, opts) {
        const token = localStorage.getItem('gb_token');
        const res = await fetch(path, { headers: { 'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json' }, ...opts });
        if (!res.ok) throw new Error('API error: ' + res.status);
        return res.json();
    },

    async loadAll() {
        await Promise.allSettled([
            this.loadQueue(), this.loadAnalytics(), this.loadChannels(), this.loadCSAT(),
            this.loadAgents(), this.loadSLA(), this.loadTranscripts(), this.loadDeflection(),
            this.loadCsatDetail(), this.loadAnalyticsCharts()
        ]);
    },

    async loadQueue() {
        try {
            const data = await this.api('/api/handoff/queue');
            this.state.queue = Array.isArray(data) ? data : (data.items || []);
            this.renderQueue();
        } catch (e) {
            document.getElementById('queue-list').innerHTML = '<div class="loading-row">Failed to load queue</div>';
        }
    },

    renderQueue() {
        const queue = this.state.queue;
        const pending = queue.filter(q => q.status === 'pending');
        const avgWait = queue.length ? queue.reduce((s, q) => s + (q.wait_seconds || 0), 0) / queue.length : 0;
        document.getElementById('queue-summary').innerHTML = `
            <div class="summary-card"><span class="label">Pending Transfers</span><span class="value accent">${pending.length}</span></div>
            <div class="summary-card"><span class="label">Total in Queue</span><span class="value">${queue.length}</span></div>
            <div class="summary-card"><span class="label">Avg Wait Time</span><span class="value">${this.fmtTime(avgWait)}</span></div>
            <div class="summary-card"><span class="label">Longest Wait</span><span class="value warning">${this.fmtTime(queue.reduce((m, q) => Math.max(m, q.wait_seconds || 0), 0))}</span></div>
        `;
        const container = document.getElementById('queue-list');
        if (!queue.length) { container.innerHTML = '<div class="empty-state"><p>No pending handoffs</p></div>'; return; }
        container.innerHTML = queue.map(q => `
            <div class="handoff-item">
                <div class="handoff-item-body">
                    <div class="handoff-item-header">
                        <span class="handoff-item-title">${this.esc(q.user_name || q.user_id || 'Unknown')}</span>
                        <span class="handoff-item-channel">${this.esc(q.channel || 'web')}</span>
                    </div>
                    <div class="handoff-item-preview">${this.esc(q.last_message || 'No message preview')}</div>
                    <div class="handoff-item-meta">
                        <span>Wait: ${this.fmtTime(q.wait_seconds || 0)}</span>
                        <span>Bot: ${this.esc(q.bot_name || '-')}</span>
                        <span>${this.esc(q.created_at || '')}</span>
                    </div>
                </div>
                <div class="handoff-item-actions">
                    <select class="handoff-agent-select" id="agent-${q.id}">
                        <option value="">Select agent...</option>
                    </select>
                    <button class="handoff-transfer-btn" onclick="HandoffApp.openTransfer('${q.id}')">Transfer</button>
                </div>
            </div>
        `).join('');
        this.loadAgentOptions();
    },

    async loadAgentOptions() {
        try {
            const data = await this.api('/api/handoff/agents');
            const agents = Array.isArray(data) ? data : (data.items || []);
            document.querySelectorAll('.handoff-agent-select').forEach(sel => {
                const current = sel.value;
                sel.innerHTML = '<option value="">Select agent...</option>' +
                    agents.map(a => `<option value="${a.id}">${this.esc(a.name)}</option>`).join('');
                sel.value = current;
            });
        } catch (e) {
            document.querySelectorAll('.handoff-agent-select').forEach(sel => {
                sel.innerHTML = '<option value="">Agents unavailable</option>';
            });
        }
    },

    async transfer(id) {
        const agentSelect = document.getElementById('agent-' + id);
        if (!agentSelect || !agentSelect.value) { alert('Select an agent first'); return; }
        try {
            await this.api('/api/handoff/transfer/' + id, { method: 'POST', body: JSON.stringify({ agent_id: agentSelect.value }) });
            this.loadQueue();
        } catch (e) {
            alert('Transfer failed: ' + e.message);
        }
    },

    openTransfer(id) {
        this._currentTransferId = id;
        const sel = document.getElementById('transferAgentSelect');
        if (sel) {
            sel.innerHTML = '<option value="">Select agent...</option>' +
                (this._agentsCache || []).map(a => `<option value="${a.id}">${this.esc(a.name)}</option>`).join('');
        }
        const modal = document.getElementById('handoffTransferModal');
        if (modal) modal.style.display = 'flex';
    },

    confirmTransfer() {
        const id = this._currentTransferId;
        if (!id) { alert('No conversation selected'); return; }
        const sel = document.getElementById('transferAgentSelect');
        const reason = document.getElementById('transferReasonSelect');
        const note = document.getElementById('transferNote');
        if (!sel || !sel.value) { alert('Select a target agent'); return; }
        this.api('/api/handoff/transfer/' + id, {
            method: 'POST',
            body: JSON.stringify({
                agent_id: sel.value,
                reason: reason ? reason.value : '',
                notes: note ? note.value : ''
            })
        }).then(() => {
            this.closeTransferModal();
            this.loadQueue();
        }).catch(e => alert('Transfer failed: ' + e.message));
    },

    closeTransferModal() {
        const modal = document.getElementById('handoffTransferModal');
        if (modal) modal.style.display = 'none';
        this._currentTransferId = null;
    },

    inviteAgent() {
        const name = prompt('Agent name:');
        if (!name) return;
        const email = prompt('Agent email:', '') || '';
        const skills = prompt('Skills (comma separated):', '') || '';
        this.api('/api/handoff/agents', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name, email, skills })
        }).then(() => { this.loadAgents(); }).catch(e => alert('Invite failed: ' + e.message));
    },

    exportAgents() {
        const agents = this._agentsCache || [];
        const header = ['name', 'email', 'status', 'active_chats', 'handled_today', 'avg_handling_seconds', 'csat', 'skills'];
        const rows = agents.map(a => [a.name, a.email, a.status, a.active_chats, a.handled_today, a.avg_handling_seconds, a.csat, a.skills]);
        const csv = [header, ...rows].map(r => r.map(v => '"' + String(v == null ? '' : v).replace(/"/g, '""') + '"').join(',')).join('\n');
        const blob = new Blob([csv], { type: 'text/csv;charset=utf-8' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url; a.download = 'agents.csv'; a.click();
        URL.revokeObjectURL(url);
    },

    applyQueueFilter() {
        const q = (document.getElementById('handoffSearch').value || '').toLowerCase();
        const ch = document.getElementById('handoffChannelFilter') ? document.getElementById('handoffChannelFilter').value : '';
        const full = this.state.queue;
        let list = full;
        if (q) list = list.filter(x => (x.user_name || '').toLowerCase().includes(q) || (x.last_message || '').toLowerCase().includes(q));
        if (ch) list = list.filter(x => (x.channel || 'web') === ch || (x.channel || '').toLowerCase() === ch.toLowerCase());
        const saved = this.state.queue;
        this.state.queue = list;
        this.renderQueue();
        this.state.queue = saved;
    },

    async loadAgents() {
        try {
            const data = await this.api('/api/handoff/agents');
            const agents = Array.isArray(data) ? data : (data.items || []);
            this._agentsCache = agents;
            this.renderAgents(agents);
        } catch (e) {
            document.getElementById('agents-body').innerHTML = '<tr><td colspan="8" class="loading-row">Failed to load agents</td></tr>';
        }
    },

    renderAgents(agents) {
        const tbody = document.getElementById('agents-body');
        if (!tbody) return;
        if (!agents.length) { tbody.innerHTML = '<tr><td colspan="8" class="empty-state"><p>No agents yet</p></td></tr>'; return; }
        tbody.innerHTML = agents.map(a => `
            <tr>
                <td>${this.esc(a.name)}</td>
                <td><span class="badge ${a.status === 'available' ? 'connected' : 'disconnected'}">${this.esc(a.status)}</span></td>
                <td>${a.active_chats || 0}</td>
                <td>${a.handled_today || 0}</td>
                <td>${this.fmtTime(a.avg_handling_seconds)}</td>
                <td>${this.esc(a.csat || '0.0')}</td>
                <td>${this.esc(a.skills || '')}</td>
                <td><button class="btn-sm" onclick="HandoffApp.inviteAgent()">Edit</button></td>
            </tr>`).join('');
    },

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
    },

    async loadAnalytics() {
        try {
            const data = await this.api('/api/handoff/analytics');
            this.state.analytics = data || {};
            this.renderAnalytics();
        } catch (e) {
            document.getElementById('analytics-cards').innerHTML = '<div class="loading-row">Failed to load analytics</div>';
        }
    },

    renderAnalytics() {
        const a = this.state.analytics;
        const hourly = a.hourly || [];
        const maxH = hourly.length ? Math.max(...hourly.map(h => h.count || 0), 1) : 1;
        document.getElementById('analytics-cards').innerHTML = `
            <div class="analytics-card">
                <span class="analytics-card-title">Conversations Today</span>
                <span class="analytics-card-value accent">${a.conversations_today || 0}</span>
                <span class="analytics-card-detail">${a.conversations_change || ''} vs yesterday</span>
            </div>
            <div class="analytics-card">
                <span class="analytics-card-title">Avg Response Time</span>
                <span class="analytics-card-value">${this.fmtTime(a.avg_response_seconds)}</span>
                <span class="analytics-card-detail">${a.response_change || ''} vs yesterday</span>
            </div>
            <div class="analytics-card">
                <span class="analytics-card-title">Resolution Rate</span>
                <span class="analytics-card-value success">${a.resolution_rate || 0}%</span>
                <span class="analytics-card-detail">${a.resolved || 0} resolved</span>
            </div>
            <div class="analytics-card">
                <span class="analytics-card-title">Bot Containment</span>
                <span class="analytics-card-value">${a.bot_containment || 0}%</span>
                <span class="analytics-card-detail">${a.contained || 0} contained by bot</span>
            </div>
            <div class="analytics-chart">
                <h3>Hourly Conversations</h3>
                <div class="chart-bars">
                    ${hourly.map(h => {
                        const pct = maxH ? ((h.count || 0) / maxH * 100) : 0;
                        return `<div class="chart-bar" style="height:${Math.max(pct, 2)}%"><span class="chart-bar-value">${h.count || 0}</span><span class="chart-bar-label">${this.esc(h.hour || '')}</span></div>`;
                    }).join('')}
                </div>
            </div>
        `;
    },

    async loadChannels() {
        try {
            const data = await this.api('/api/handoff/channels');
            this.state.channels = Array.isArray(data) ? data : (data.items || []);
            this.renderChannels();
        } catch (e) {
            document.getElementById('channels-grid').innerHTML = '<div class="loading-row">Failed to load channels</div>';
        }
    },

    renderChannels() {
        const channels = this.state.channels;
        if (!channels.length) { document.getElementById('channels-grid').innerHTML = '<div class="empty-state"><p>No channels configured</p></div>'; return; }
        const iconMap = {
            whatsapp: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/></svg>',
            telegram: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z"/></svg>',
            teams: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="7" width="18" height="13" rx="2"/><path d="M16 3v4M8 3v4"/></svg>',
            web: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>'
        };
        document.getElementById('channels-grid').innerHTML = channels.map(ch => `
            <div class="channel-card">
                <div class="channel-card-header">
                    <span class="channel-card-name">${iconMap[ch.type] || ''} ${this.esc(ch.name)}</span>
                    <span class="badge ${ch.connected ? 'connected' : 'disconnected'}">${ch.connected ? 'Connected' : 'Disconnected'}</span>
                </div>
                <div class="channel-card-body">
                    <div class="channel-row"><span class="ch-label">Messages Today</span><span class="ch-val">${ch.messages_today || 0}</span></div>
                    <div class="channel-row"><span class="ch-label">Active Conversations</span><span class="ch-val">${ch.active_conversations || 0}</span></div>
                    <div class="channel-row"><span class="ch-label">Avg Response</span><span class="ch-val">${this.fmtTime(ch.avg_response_seconds)}</span></div>
                    <div class="channel-row"><span class="ch-label">Error Rate</span><span class="ch-val">${ch.error_rate || 0}%</span></div>
                </div>
            </div>
        `).join('');
    },

    async loadCSAT() {
        try {
            const data = await this.api('/api/handoff/csat');
            this.state.csat = data || {};
            this.renderCSAT();
        } catch (e) {
            document.getElementById('csat-grid').innerHTML = '<div class="loading-row">Failed to load CSAT</div>';
        }
    },

    renderCSAT() {
        const c = this.state.csat;
        const ratings = c.recent_ratings || [];
        const distribution = c.distribution || [];
        const maxDist = distribution.length ? Math.max(...distribution.map(d => d.count || 0), 1) : 1;
        document.getElementById('csat-grid').innerHTML = `
            <div class="csat-score-card">
                <span class="csat-score-label">Overall Score</span>
                <div class="csat-score-value">${(c.avg_score || 0).toFixed(1)}</div>
                <span class="csat-score-label">${c.total_ratings || 0} ratings</span>
            </div>
            <div class="csat-score-card">
                <span class="csat-score-label">Response Score</span>
                <div class="csat-score-value" style="color:#22c55e">${(c.response_score || 0).toFixed(1)}</div>
                <span class="csat-score-label">Helpfulness</span>
            </div>
            <div class="csat-score-card">
                <span class="csat-score-label">Resolution Score</span>
                <div class="csat-score-value" style="color:#60a5fa">${(c.resolution_score || 0).toFixed(1)}</div>
                <span class="csat-score-label">Problem solved</span>
            </div>
            <div class="csat-ratings">
                <h3 style="margin:0;font-size:14px;font-weight:600">Recent Ratings</h3>
                ${!ratings.length ? '<div class="empty-state"><p>No ratings yet</p></div>' : ratings.map(r => `
                    <div class="csat-rating-item">
                        <div class="csat-rating-stars">${this.renderStars(r.score)}</div>
                        <div class="csat-rating-body">
                            <span class="csat-rating-comment">${this.esc(r.comment || 'No comment')}</span>
                            <span class="csat-rating-meta">${this.esc(r.user_name || 'Anonymous')} - ${this.esc(r.created_at || '')}</span>
                        </div>
                    </div>
                `).join('')}
            </div>
        `;
    },

    renderStars(score) {
        return Array.from({length: 5}, (_, i) =>
            `<svg class="csat-star ${i < score ? '' : 'empty'}" width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>`
        ).join('');
    },

    refresh() { this.loadAll(); },

    fmtTime(s) {
        if (!s && s !== 0) return '-';
        s = Math.round(s);
        if (s < 60) return s + 's';
        return Math.floor(s / 60) + 'm ' + (s % 60) + 's';
    },
    esc(s) { const d = document.createElement('div'); d.textContent = s; return d.innerHTML; },
    showFeedback(msg, type) {
        let el = document.getElementById('handoff-feedback');
        if (!el) { el = document.createElement('div'); el.id = 'handoff-feedback'; el.style.cssText = 'position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;color:#fff;transition:opacity .3s'; document.body.appendChild(el); }
        el.textContent = msg;
        el.style.background = type === 'error' ? '#ef4444' : '#22c55e';
        el.style.opacity = '1';
        setTimeout(() => { el.style.opacity = '0'; }, 3000);
    }
};

(function(){ var __cb = () => HandoffApp.init(); if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
})();
