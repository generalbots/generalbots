/* Banking app — module 4/5: PIX and statements (split from banking/banking.js for #1731) */

(function (app) {
'use strict';
Object.assign(app, {
    // ---------------- PIX ----------------
    async loadPix() {
        try { this.state.pix = await this.api('/api/banking/pix'); this.renderPix(); }
        catch (e) {
            const b = document.getElementById('pix-body');
            if (b) b.innerHTML = '<tr><td colspan="7" class="loading-row">Failed to load PIX</td></tr>';
        }
    },

    renderPix() {
        const b = document.getElementById('pix-body');
        if (!b) return;
        if (!this.state.pix.length) { b.innerHTML = '<tr><td colspan="7" class="loading-row">No PIX transactions.</td></tr>'; return; }
        b.innerHTML = this.state.pix.map(t => `
            <tr>
                <td>${this.esc(this.fmtDate(t.created_at))}</td>
                <td><span class="badge ${t.direction==='in'?'matched':'pending'}">${t.direction==='in'?'Received':'Sent'}</span></td>
                <td>${this.esc(t.counterparty || t.key_value)}</td>
                <td>${this.esc(t.key_type)}</td>
                <td>${this.fmtBRL(parseFloat(t.amount)||0)}</td>
                <td><span class="badge ${t.status==='completed'?'matched':'unmatched'}">${this.esc(t.status||'completed')}</span></td>
                <td>${this.esc(t.end_to_end_id || '-')}</td>
            </tr>
        `).join('');
    },

    showPixTransfer() {
        this.state.transferDirection = 'out';
        const title = document.getElementById('bankingTransferTitle');
        if (title) title.textContent = 'New PIX transfer';
        this.openModal('bankingTransferModal');
    },

    showPixReceive() {
        this.state.transferDirection = 'in';
        const title = document.getElementById('bankingTransferTitle');
        if (title) title.textContent = 'Receive via PIX';
        // In receive mode the description becomes optional memo and amount is what was received.
        this.openModal('bankingTransferModal');
    },

    confirmTransfer() {
        const keyType = document.getElementById('transferKeyType').value;
        const key = document.getElementById('transferKey').value.trim();
        const amount = document.getElementById('transferAmount').value;
        const description = document.getElementById('transferDescription').value.trim();
        if (!key) { this.toast('Recipient key is required'); return; }
        if (!amount || parseFloat(amount) <= 0) { this.toast('Amount must be positive'); return; }
        const payload = { key_type: keyType, key_value: key, amount: amount, description: description };
        const url = this.state.transferDirection === 'in' ? '/api/banking/pix/receive' : '/api/banking/pix/transfer';
        const token = localStorage.getItem('gb_token');
        fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
            body: JSON.stringify(payload)
        })
        .then(r => { if (!r.ok) throw new Error('HTTP ' + r.status); return r.json(); })
        .then(() => { this.closeTransferModal(); this.toast('PIX ' + (this.state.transferDirection === 'in' ? 'received' : 'sent')); this.loadPix(); this.loadStats(); })
        .catch(err => this.toast('Failed: ' + err.message));
    },

    closeTransferModal() { this.closeModal('bankingTransferModal'); },

    async exportPixHistory() {
        try {
            const token = localStorage.getItem('gb_token');
            const res = await fetch('/api/banking/pix/export', { headers: { 'Authorization': 'Bearer ' + token } });
            if (!res.ok) throw new Error('HTTP ' + res.status);
            const blob = await res.blob();
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'pix-history.csv';
            document.body.appendChild(a); a.click(); a.remove();
            URL.revokeObjectURL(url);
        } catch (e) { this.toast('Export failed: ' + e.message); }
    },

    // ---------------- Statements ----------------
    async loadStatements() {
        try { this.state.statements = await this.api('/api/banking/statements'); this.renderStatements(); }
        catch (e) {
            const b = document.getElementById('statements-body');
            if (b) b.innerHTML = '<tr><td colspan="7" class="loading-row">Failed to load statements</td></tr>';
        }
    },

    renderStatements() {
        const b = document.getElementById('statements-body');
        if (!b) return;
        if (!this.state.statements.length) { b.innerHTML = '<tr><td colspan="7" class="loading-row">No statements generated.</td></tr>'; return; }
        b.innerHTML = this.state.statements.map(s => `
            <tr>
                <td>${this.esc(s.period)}</td>
                <td>${this.esc(s.account_label)}</td>
                <td>${this.fmtBRL(parseFloat(s.opening)||0)}</td>
                <td>${this.fmtBRL(parseFloat(s.closing)||0)}</td>
                <td>${this.esc(this.fmtDate(s.generated_at))}</td>
                <td>${this.esc(s.format || 'pdf')}</td>
                <td><button class="btn-secondary" style="padding:4px 10px;font-size:12px" onclick="BankingApp.downloadStatement('${s.id}')">Download</button></td>
            </tr>
        `).join('');
    },

    async downloadStatement(id) {
        try {
            const token = localStorage.getItem('gb_token');
            const res = await fetch('/api/banking/statements/' + id + '/download', { headers: { 'Authorization': 'Bearer ' + token } });
            if (!res.ok) throw new Error('HTTP ' + res.status);
            const blob = await res.blob();
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url; a.download = 'statement-' + id + '.pdf';
            document.body.appendChild(a); a.click(); a.remove();
            URL.revokeObjectURL(url);
        } catch (e) { this.toast('Download failed: ' + e.message); }
    },

    showNewStatement() {
        const period = prompt('Statement period (e.g. 2026-08):');
        if (!period) return;
        const accountId = (this.state.accounts[0] || {}).id;
        if (!accountId) { this.toast('Add an account first'); return; }
        const token = localStorage.getItem('gb_token');
        fetch('/api/banking/statements', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
            body: JSON.stringify({ account_id: accountId, period: period, format: 'pdf' })
        })
        .then(r => { if (!r.ok) throw new Error('HTTP ' + r.status); return r.json(); })
        .then(() => { this.toast('Statement generated'); this.loadStatements(); })
        .catch(err => this.toast('Failed: ' + err.message));
    }
});
})(window.BankingApp);
