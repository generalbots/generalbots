"use strict";
(function() {
    // Tab switching
    document.querySelectorAll('.crm-tab').forEach(tab => {
        tab.addEventListener('click', function() {
            document.querySelectorAll('.crm-tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.crm-view').forEach(v => v.classList.remove('active'));
            this.classList.add('active');
            const view = this.dataset.view;
            document.getElementById(`crm-${view}-view`).classList.add('active');
        });
    });

    // New button dropdown
    const newBtn = document.getElementById('crm-new-btn');
    newBtn.addEventListener('click', function() {
        // Default: open lead form
        htmx.ajax('GET', '/suite/crm/partials/lead-form.html', '#crm-modal-content').then(() => {
            openCrmModal();
        });
    });

    // Modal functions
    window.openCrmModal = function() {
        document.getElementById('crm-modal').classList.add('open');
    };

    window.closeCrmModal = function() {
        document.getElementById('crm-modal').classList.remove('open');
    };

    // Drag and drop for pipeline (#1298) — fully DELEGATED on the container so
    // columns rebuilt from the stages API (#1441 A4) keep working.
    const pipelineRoot = document.getElementById('crm-pipeline-view');
    pipelineRoot.addEventListener('dragstart', e => {
        const card = e.target.closest('.pipeline-card');
        if (!card) return;
        e.dataTransfer.setData('text/plain', card.dataset.id || '');
        e.dataTransfer.effectAllowed = 'move';
        card.classList.add('dragging');
    });
    pipelineRoot.addEventListener('dragend', () => {
        pipelineRoot.querySelectorAll('.pipeline-card.dragging')
            .forEach(c => c.classList.remove('dragging'));
    });

    pipelineRoot.addEventListener('dragover', e => {
        const column = e.target.closest('.pipeline-cards');
        if (!column) return;
        e.preventDefault();
        e.dataTransfer.dropEffect = 'move';
        column.classList.add('drag-over');
    });

    pipelineRoot.addEventListener('dragleave', (e) => {
        const column = e.target.closest('.pipeline-cards');
        if (column && !column.contains(e.relatedTarget)) {
            column.classList.remove('drag-over');
        }
    });

    pipelineRoot.addEventListener('drop', e => {
        const column = e.target.closest('.pipeline-cards');
        if (!column) return;
        e.preventDefault();
        column.classList.remove('drag-over');
        const cardId = e.dataTransfer.getData('text/plain');
        const card = pipelineRoot.querySelector(`.pipeline-card[data-id="${cardId}"]`);
        const newStage = column.closest('.pipeline-column').dataset.stage;
        if (!cardId || !newStage) return;
        if (card && card.closest('.pipeline-column') === column.closest('.pipeline-column')) return;

        // Move card visually, then persist via PUT /api/crm/leads/:id/stage
        if (card) column.appendChild(card);
        htmx.ajax('PUT', `/api/crm/leads/${cardId}/stage?stage=${encodeURIComponent(newStage)}`, { swap: 'none' })
            .then(() => {
                // Refresh stage counts after the move
                htmx.trigger(document.body, 'crm:stage-changed');
            });
    });

    // #1441 A4 / #1455 — pipeline stages are org-configurable: render the
    // kanban from /api/crm/pipeline/stages (order/labels from the API); the
    // static six columns above remain as fallback when the API is unavailable.
    // The renderer is global so the stage admin editor (crm-p2.js) can
    // re-render after create/rename/delete.
    window.renderPipelineStages = function(stages) {
        if (!Array.isArray(stages) || !stages.length) return;
        const container = document.querySelector('.pipeline-container');
        if (!container) return;
        const sorted = stages.slice().sort((a, b) => (a.stage_order || 0) - (b.stage_order || 0));
        container.innerHTML = `
            <div class="stage-toolbar">
                <button id="stage-add-btn" class="btn-secondary" title="Add a custom stage">+ Add stage</button>
                <span id="stage-admin-result" class="stage-admin-result"></span>
            </div>` + sorted.map((s, i) => `
                <div class="pipeline-column${s.name === 'won' ? ' won' : ''}${s.name === 'lost' ? ' lost' : ''}" data-stage="${s.name}" data-stage-id="${s.id || ''}">
                    <div class="pipeline-header">
                        <span class="pipeline-title">${s.name}</span>
                        <span class="stage-actions">
                            <button class="stage-action" data-action="rename" title="Rename stage">✎</button>
                            <button class="stage-action" data-action="probability" title="Set probability">%</button>
                            <button class="stage-action" data-action="delete" title="Delete stage">✕</button>
                        </span>
                        <span class="pipeline-count" hx-get="/api/crm/count?stage=${encodeURIComponent(s.name)}" hx-trigger="load">0</span>
                    </div>
                    <div class="pipeline-cards"
                         hx-get="/api/crm/pipeline?stage=${encodeURIComponent(s.name)}"
                         hx-trigger="load"
                         hx-swap="innerHTML"></div>
                    ${i === 0 ? `<button class="pipeline-add" hx-get="/suite/crm/partials/lead-form.html" hx-target="#crm-modal-content" hx-on::after-request="openCrmModal()">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
                        </svg>
                        <span data-i18n="crm-add-lead">Add Lead</span>
                    </button>` : ''}
                </div>`).join('');
        if (window.i18n && window.i18n.translatePage) window.i18n.translatePage();
        htmx.process(container);
    };

    fetch('/api/crm/pipeline/stages', {
        headers: { 'Authorization': 'Bearer ' + (localStorage.getItem('gb-access-token') || '') }
    })
        .then(r => r.ok ? r.json() : Promise.reject(r.status))
        .then(stages => window.renderPipelineStages(stages))
        .catch(() => { /* keep static fallback columns */ });

    // Keep counts in sync after any stage change
    document.body.addEventListener('crm:stage-changed', function() {
        document.querySelectorAll('.pipeline-count').forEach(el => htmx.trigger(el, 'load'));
    });

    // #1437 — deep-link consumer: opens on a CRM contact (person_id). The id
    // arrives either at boot via window.__gbAppParams__ (fresh window) or via
    // the gb:deep-link retarget event (already-open window). Switches to the
    // Contacts view, waits for the grid (HTMX loads it async), then highlights
    // the matching row and raises the window.
    function highlightCrmRow(row) {
        document.querySelectorAll('#contacts-table-body tr').forEach(function(tr) {
            tr.style.boxShadow = '';
            tr.style.background = '';
        });
        row.style.boxShadow = '0 0 0 2px #22c55e inset, 0 0 12px rgba(34,197,94,.5)';
        row.style.background = 'rgba(34,197,94,.12)';
        row.scrollIntoView({ block: 'center', behavior: 'smooth' });
    }

    window.applyCrmPersonDeepLink = function(personId, appId) {
        if (!personId) return;
        const contactsTab = document.querySelector('.crm-tab[data-view="contacts"]');
        if (contactsTab && !contactsTab.classList.contains('active')) contactsTab.click();
        const tableBody = document.getElementById('contacts-table-body');
        if (!tableBody) return;
        let attempts = 0;
        const poll = setInterval(function() {
            const row = tableBody.querySelector('tr[data-id="' + personId + '"]');
            if (row) {
                clearInterval(poll);
                highlightCrmRow(row);
                if (appId && window.WindowManager && window.WindowManager.focusWindow) {
                    window.WindowManager.focusWindow(appId);
                }
                return;
            }
            attempts++;
            if (attempts === 1 && tableBody.rows.length === 0 && window.htmx) {
                window.htmx.trigger(tableBody, 'load');
            }
            if (attempts > 25) clearInterval(poll);
        }, 200);
    };

    if (window.__gbAppParams__ && window.__gbAppParams__.person_id) {
        window.applyCrmPersonDeepLink(window.__gbAppParams__.person_id, 'crm');
    }
    document.addEventListener('gb:deep-link', function(e) {
        const params = e.detail && e.detail.params;
        if (params && params.person_id) {
            window.applyCrmPersonDeepLink(params.person_id, (e.detail && e.detail.appId) || 'crm');
        }
    });

    // Initialize i18n if available
    if (window.i18n && window.i18n.translatePage) {
        window.i18n.translatePage();
    }

    // Submit lead form manually to ensure JSON is sent correctly
    window.submitLeadForm = async function(event) {
        const form = document.getElementById('leadForm');
        const formData = new FormData(form);
        const data = {};
        const numericFields = ['value'];
        
        for (let [key, value] of formData.entries()) {
            if (numericFields.includes(key)) {
                // Convert to number or null
                data[key] = value ? parseFloat(value) : null;
            } else if (value) {
                // Only include non-empty strings
                data[key] = value;
            }
        }
        
        console.log('Submitting lead data:', data);
        
        const token = localStorage.getItem('gb-access-token');
        
        try {
            const response = await fetch('/api/crm/leads', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': 'Bearer ' + token
                },
                body: JSON.stringify(data)
            });
            
            if (response.ok) {
                closeCrmModal();
                htmx.ajax('GET', '/api/crm/pipeline?stage=new', {
                    target: '#crm-pipeline-view',
                    swap: 'innerHTML'
                });
            } else {
                console.error('Lead creation failed:', response.status, await response.text());
            }
        } catch (err) {
            console.error('Error creating lead:', err);
        }
    };

    // ── #1441 P2 — bulk actions, CSV import/export, audit trail ─────────
    function authHeaders(extra) {
        const h = Object.assign({ 'Authorization': 'Bearer ' + (localStorage.getItem('gb-access-token') || '') }, extra || {});
        return h;
    }
    // crm-p2.js runs in its own IIFE — expose the helper globally (crmp2
    // handlers for bulk/CSV/drawer/campaigns all resolve it at click time).
    window.authHeaders = authHeaders;

    function selectedOppIds() {
        return Array.from(document.querySelectorAll('#opportunities-table-body input.opp-select:checked'))
            .map(cb => cb.closest('tr').dataset.id)
            .filter(Boolean);
    }

    function refreshBulkCount() {
        const n = selectedOppIds().length;
        const el = document.getElementById('opps-bulk-count');
        if (el) el.textContent = n + ' selected';
    }

    // Delegated: checkboxes arrive with the HTMX rows.
    document.addEventListener('change', function(e) {
        if (e.target.id === 'opps-check-all') {
            const checked = e.target.checked;
            document.querySelectorAll('#opportunities-table-body input.opp-select')
                .forEach(cb => { cb.checked = checked; });
        }
        if (e.target.classList && e.target.classList.contains('opp-select')) {
            const all = document.getElementById('opps-check-all');
            if (all) all.checked = false;
        }
        refreshBulkCount();
    });

    function showBulkResult(text) {
        const el = document.getElementById('opps-bulk-result');
        if (el) el.textContent = text;
        setTimeout(() => { if (el) el.textContent = ''; }, 8000);
    }

    function reloadOppRows() {
        const body = document.getElementById('opportunities-table-body');
        if (body && window.htmx) window.htmx.trigger(body, 'load');
    }

    async function bulkAction(action, extra) {
        const ids = selectedOppIds();
        if (!ids.length) { showBulkResult('Select rows first'); return; }
        const resp = await fetch('/api/crm/leads/bulk', {
            method: 'POST',
            headers: authHeaders({ 'Content-Type': 'application/json' }),
            body: JSON.stringify(Object.assign({ ids: ids, action: action }, extra || {}))
        });
        if (resp.ok) {
            const r = await resp.json();
            showBulkResult('Updated ' + (r.updated || 0) + ', deleted ' + (r.deleted || 0) +
                (r.failed && r.failed.length ? ', failed ' + r.failed.length : ''));
            reloadOppRows();
        } else {
            showBulkResult('Bulk action failed (' + resp.status + ')');
        }
    }

    const bulkStageBtn = document.getElementById('opps-bulk-stage-btn');
    if (bulkStageBtn) bulkStageBtn.addEventListener('click', function() {
        const stage = document.getElementById('opps-bulk-stage').value;
        bulkAction('stage', { stage: stage });
    });

    const bulkDeleteBtn = document.getElementById('opps-bulk-delete-btn');
    if (bulkDeleteBtn) bulkDeleteBtn.addEventListener('click', function() {
        if (confirm('Delete the selected records? This is recorded in the audit trail.')) {
            bulkAction('delete');
        }
    });

    const exportBtn = document.getElementById('opps-export-btn');
    if (exportBtn) exportBtn.addEventListener('click', async function() {
        // Auth-gated download: fetch with the Bearer token, then hand the CSV
        // to the browser as a blob (window.open would drop the header).
        const resp = await fetch('/api/crm/leads/export', { headers: authHeaders() });
        if (!resp.ok) { showBulkResult('Export failed (' + resp.status + ')'); return; }
        const total = resp.headers.get('X-Total-Count');
        const blob = await resp.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'leads-export.csv';
        a.click();
        URL.revokeObjectURL(url);
        showBulkResult(total ? ('Exported ' + (total - 0) + ' rows') : 'Export downloaded');
    });

    const importBtn = document.getElementById('opps-import-btn');
    const importFile = document.getElementById('opps-import-file');
    if (importBtn && importFile) {
        importBtn.addEventListener('click', () => importFile.click());
        importFile.addEventListener('change', async function() {
            const file = importFile.files && importFile.files[0];
            if (!file) return;
            const text = await file.text();
            // #1456 — dry-run first: validate without writing, then commit on
            // explicit confirmation when the report is clean.
            const dryResp = await fetch('/api/crm/leads/import?dry_run=true', {
                method: 'POST',
                headers: authHeaders({ 'Content-Type': 'text/csv' }),
                body: text
            });
            if (!dryResp.ok) { showBulkResult('Validation failed (' + dryResp.status + ')'); return; }
            const dry = await dryResp.json();
            const summary = 'Validated: ' + dry.imported + ' rows, ' + dry.skipped_duplicates + ' duplicates, ' + dry.errors.length + ' errors';
            if (!confirm(summary + '. Commit this import?')) { showBulkResult(summary + ' — not committed'); return; }
            const resp = await fetch('/api/crm/leads/import', {
                method: 'POST',
                headers: authHeaders({ 'Content-Type': 'text/csv' }),
                body: text
            });
            if (resp.ok) {
                const r = await resp.json();
                showBulkResult('Imported ' + r.imported +
                    (r.skipped_duplicates ? ', duplicates ' + r.skipped_duplicates : '') +
                    (r.errors && r.errors.length ? ', errors ' + r.errors.length : ''));
                reloadOppRows();
            } else {
                showBulkResult('Import failed (' + resp.status + ')');
            }
            importFile.value = '';
        });
    }

    // Audit trail drawer — newest-first JSON rows rendered as simple entries.
    const auditDrawer = document.getElementById('crm-audit-drawer');
    const auditBtn = document.getElementById('crm-audit-btn');
    if (auditBtn && auditDrawer) {
        auditBtn.addEventListener('click', async function() {
            const body = document.getElementById('crm-audit-body');
            body.innerHTML = '<div class="crm-audit-empty">Loading…</div>';
            auditDrawer.classList.add('open');
            const resp = await fetch('/api/crm/audit?limit=60', { headers: authHeaders() });
            if (!resp.ok) {
                body.innerHTML = '<div class="crm-audit-empty">Audit unavailable (' + resp.status + ')</div>';
                return;
            }
            const rows = await resp.json();
            if (!rows.length) {
                body.innerHTML = '<div class="crm-audit-empty">No audit entries yet</div>';
                return;
            }
            body.innerHTML = rows.map(r => {
                const when = (r.created_at || '').replace('T', ' ').slice(0, 16);
                const who = r.actor_email || 'anonymous';
                const target = r.entity_id ? (' · ' + r.entity_id.slice(0, 8)) : '';
                return '<div class="crm-audit-row">' +
                    '<span class="crm-audit-when">' + when + '</span>' +
                    '<span class="crm-audit-action">' + r.entity + '/' + r.action + target + '</span>' +
                    '<span class="crm-audit-actor">' + who + '</span>' +
                    '</div>';
            }).join('');
        });
        document.getElementById('crm-audit-close').addEventListener('click', function() {
            auditDrawer.classList.remove('open');
        });
    }
})();
