/* Handoff app — module 2/5: queue, transfers and agents (split from handoff/handoff.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
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
    }
});
})(window.HandoffApp);
