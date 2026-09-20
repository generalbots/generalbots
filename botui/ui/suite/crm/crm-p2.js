"use strict";
(function() {
    // ── #1451 — contacts bulk bar: select / delete / CSV ────────────────
    function selectedContactIds() {
        return Array.from(document.querySelectorAll('#contacts-table-body input.opp-select:checked'))
            .map(cb => cb.closest('tr').dataset.id)
            .filter(Boolean);
    }

    function refreshContactBulkCount() {
        const el = document.getElementById('contacts-bulk-count');
        if (el) el.textContent = selectedContactIds().length + ' selected';
    }

    document.addEventListener('change', function(e) {
        if (e.target.id === 'contacts-check-all') {
            const checked = e.target.checked;
            document.querySelectorAll('#contacts-table-body input.opp-select')
                .forEach(cb => { cb.checked = checked; });
        }
        refreshContactBulkCount();
    });

    function showContactResult(text) {
        const el = document.getElementById('contacts-bulk-result');
        if (el) el.textContent = text;
        setTimeout(() => { if (el) el.textContent = ''; }, 8000);
    }

    function reloadContactRows() {
        const body = document.getElementById('contacts-table-body');
        if (body && window.htmx) window.htmx.trigger(body, 'load');
    }

    const contactDeleteBtn = document.getElementById('contacts-bulk-delete-btn');
    if (contactDeleteBtn) contactDeleteBtn.addEventListener('click', async function() {
        const ids = selectedContactIds();
        if (!ids.length) { showContactResult('Select rows first'); return; }
        if (!confirm('Delete the selected contacts? This is recorded in the audit trail.')) return;
        const resp = await fetch('/api/crm/contacts/bulk', {
            method: 'POST',
            headers: authHeaders({ 'Content-Type': 'application/json' }),
            body: JSON.stringify({ ids: ids, action: 'delete' })
        });
        if (resp.ok) {
            const r = await resp.json();
            showContactResult('Deleted ' + (r.deleted || 0) +
                (r.failed && r.failed.length ? ', failed ' + r.failed.length : ''));
            reloadContactRows();
        } else {
            showContactResult('Delete failed (' + resp.status + ')');
        }
    });

    const contactExportBtn = document.getElementById('contacts-export-btn');
    if (contactExportBtn) contactExportBtn.addEventListener('click', async function() {
        const resp = await fetch('/api/crm/contacts/export', { headers: authHeaders() });
        if (!resp.ok) { showContactResult('Export failed (' + resp.status + ')'); return; }
        const blob = await resp.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'contacts-export.csv';
        a.click();
        URL.revokeObjectURL(url);
        showContactResult('Export downloaded');
    });

    const contactImportBtn = document.getElementById('contacts-import-btn');
    const contactImportFile = document.getElementById('contacts-import-file');
    if (contactImportBtn && contactImportFile) {
        contactImportBtn.addEventListener('click', () => contactImportFile.click());
        contactImportFile.addEventListener('change', async function() {
            const file = contactImportFile.files && contactImportFile.files[0];
            if (!file) return;
            const text = await file.text();
            const resp = await fetch('/api/crm/contacts/import', {
                method: 'POST',
                headers: authHeaders({ 'Content-Type': 'text/csv' }),
                body: text
            });
            if (resp.ok) {
                const r = await resp.json();
                showContactResult('Imported ' + r.imported +
                    (r.skipped_duplicates ? ', duplicates ' + r.skipped_duplicates : '') +
                    (r.errors && r.errors.length ? ', errors ' + r.errors.length : ''));
                reloadContactRows();
            } else {
                showContactResult('Import failed (' + resp.status + ')');
            }
            contactImportFile.value = '';
        });
    }

    // ── #1453 — campaigns: create + send ────────────────────────────────
    const campaignNewBtn = document.getElementById('campaign-new-btn');
    if (campaignNewBtn) campaignNewBtn.addEventListener('click', function() {
        const tpl = document.getElementById('campaign-form-template');
        const content = document.getElementById('crm-modal-content');
        content.innerHTML = '';
        content.appendChild(tpl.content.cloneNode(true));
        openCrmModal();
        document.getElementById('campaignForm').addEventListener('submit', async function(ev) {
            ev.preventDefault();
            const fd = new FormData(ev.target);
            const data = { campaign_type: 'outreach' };
            for (const kv of fd.entries()) {
                if (!kv[1]) continue;
                if (kv[0] === 'budget') data[kv[0]] = parseFloat(kv[1]);
                else data[kv[0]] = kv[1];
            }
            const resp = await fetch('/api/crm/campaigns', {
                method: 'POST',
                headers: authHeaders({ 'Content-Type': 'application/json' }),
                body: JSON.stringify(data)
            });
            const banner = document.getElementById('campaign-result');
            if (resp.ok) {
                closeCrmModal();
                if (banner) banner.textContent = 'Campaign created';
                if (window.htmx) window.htmx.trigger(document.getElementById('crmCampaignsList'), 'load');
            } else {
                if (banner) banner.textContent = 'Create failed (' + resp.status + ')';
            }
            setTimeout(() => { if (banner) banner.textContent = ''; }, 8000);
        });
    });

    // Send button — delegated because campaign cards come from HTMX.
    document.addEventListener('click', async function(e) {
        const btn = e.target.closest('.campaign-send-btn');
        if (!btn) return;
        const id = btn.dataset.id;
        if (!id || !confirm('Send this campaign to its recipients now?')) return;
        const resp = await fetch('/api/crm/campaigns/' + id + '/send', {
            method: 'POST',
            headers: authHeaders({ 'Content-Type': 'application/json' }),
            body: '{}'
        });
        const banner = document.getElementById('campaign-result');
        if (banner) {
            banner.textContent = resp.ok
                ? 'Send dispatched'
                : 'Send failed (' + resp.status + ')';
            setTimeout(() => { banner.textContent = ''; }, 8000);
        }
        if (window.htmx) window.htmx.trigger(document.getElementById('crmCampaignsList'), 'load');
    });

    // ── #1449 — lead/opportunity detail drawer with audit history ───────
    function escDetail(s) {
        return String(s == null ? '' : s).replace(/[&<>"']/g, c =>
            ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
    }

    async function openLeadDetail(id) {
        const panel = document.getElementById('detail-panel');
        panel.innerHTML = '<div class="crm-audit-empty">Loading…</div>';
        panel.classList.add('open');
        const resp = await fetch('/api/crm/leads/' + id, { headers: authHeaders() });
        if (!resp.ok) {
            panel.innerHTML = '<div class="crm-audit-empty">Lead unavailable (' + resp.status + ')</div>';
            return;
        }
        const lead = await resp.json();
        const contact = lead.contact_id
            ? await fetch('/api/crm/contacts/' + lead.contact_id, { headers: authHeaders() })
                .then(r => r.ok ? r.json() : null).catch(() => null)
            : null;
        const stageOptions = ['new', 'qualified', 'proposal', 'negotiation', 'won', 'lost']
            .map(s => '<option' + (s === (lead.stage || '') ? ' selected' : '') + '>' + s + '</option>').join('');
        const contactLabel = contact
            ? (((contact.first_name || '') + ' ' + (contact.last_name || '')).trim() +
               (contact.email ? ' · ' + contact.email : ''))
            : '—';
        panel.innerHTML =
            '<div class="crm-detail-head">' +
                '<strong>' + escDetail(lead.title || lead.name || 'Lead') + '</strong>' +
                '<button id="crm-detail-close" title="Close">×</button>' +
            '</div>' +
            '<div class="crm-detail-body">' +
                '<div class="crm-detail-grid">' +
                    '<div><span class="crm-detail-label">Value</span> ' + escDetail(lead.value != null ? (lead.currency || '') + ' ' + lead.value : '—') + '</div>' +
                    '<div><span class="crm-detail-label">Stage</span> ' + escDetail(lead.stage || '—') + ' (' + (lead.probability != null ? lead.probability + '%' : '—') + ')</div>' +
                    '<div><span class="crm-detail-label">Contact</span> ' + escDetail(contactLabel) + '</div>' +
                    '<div><span class="crm-detail-label">Created</span> ' + escDetail((lead.created_at || '').slice(0, 10)) + '</div>' +
                '</div>' +
                '<div class="crm-detail-actions">' +
                    '<select id="crm-detail-stage">' + stageOptions + '</select>' +
                    '<button class="btn-secondary" id="crm-detail-stage-btn">Set stage</button>' +
                    '<button class="btn-secondary" id="crm-detail-convert-btn">Convert</button>' +
                    '<button class="btn-danger" id="crm-detail-delete-btn">Delete</button>' +
                '</div>' +
                '<div class="crm-detail-label" style="margin-top:12px">Audit history</div>' +
                '<div id="crm-detail-audit" class="crm-audit-body"><div class="crm-audit-empty">Loading…</div></div>' +
            '</div>';

        document.getElementById('crm-detail-close').addEventListener('click', () => panel.classList.remove('open'));
        document.getElementById('crm-detail-stage-btn').addEventListener('click', async function() {
            const stage = document.getElementById('crm-detail-stage').value;
            await fetch('/api/crm/leads/' + id + '/stage?stage=' + encodeURIComponent(stage), {
                method: 'PUT', headers: authHeaders()
            });
            reloadOppRows();
            openLeadDetail(id);
        });
        document.getElementById('crm-detail-convert-btn').addEventListener('click', async function() {
            await fetch('/api/crm/leads/' + id + '/convert', { method: 'POST', headers: authHeaders() });
            reloadOppRows();
            panel.classList.remove('open');
        });
        document.getElementById('crm-detail-delete-btn').addEventListener('click', async function() {
            if (!confirm('Delete this lead? This is recorded in the audit trail.')) return;
            await fetch('/api/crm/leads/' + id, { method: 'DELETE', headers: authHeaders() });
            reloadOppRows();
            panel.classList.remove('open');
        });

        const auditBox = document.getElementById('crm-detail-audit');
        const auditResp = await fetch('/api/crm/audit?entity=lead&entity_id=' + id + '&limit=30', { headers: authHeaders() });
        if (auditResp.ok) {
            const rows = await auditResp.json();
            auditBox.innerHTML = rows.length
                ? rows.map(r =>
                    '<div class="crm-audit-row">' +
                        '<span class="crm-audit-when">' + escDetail((r.created_at || '').replace('T', ' ').slice(0, 16)) + '</span>' +
                        '<span class="crm-audit-action">' + escDetail(r.action) +
                            (r.before && r.before.stage ? ' · ' + escDetail(r.before.stage) + ' → ' + escDetail((r.after && r.after.stage) || '?') : '') +
                        '</span>' +
                        '<span class="crm-audit-actor">' + escDetail(r.actor_email || 'anonymous') + '</span>' +
                    '</div>').join('')
                : '<div class="crm-audit-empty">No audit entries yet</div>';
        } else {
            auditBox.innerHTML = '<div class="crm-audit-empty">Audit unavailable</div>';
        }
    }

    // Row click → drawer (delegated; checkbox/button clicks are ignored).
    document.addEventListener('click', function(e) {
        if (e.target.closest('input, button, a, select')) return;
        const row = e.target.closest('#opportunities-table-body tr[data-id], #deals-table-body tr[data-id]');
        if (row && row.dataset.id) openLeadDetail(row.dataset.id);
    });
})();
