/* #1441 P2/#1450/#1456 — bulk selection, CSV import/export and the audit
   drawer across the Leads/Opportunities, Deals and Contacts grids. */
"use strict";
(function() {
    function headers(json) {
        return window.authHeaders(json ? { 'Content-Type': 'application/json' } : {});
    }

    var GRIDS = {
        opportunities: { body: 'opportunities-table-body', all: 'opps-check-all', count: 'opps-bulk-count', result: 'opps-bulk-result' },
        deals: { body: 'deals-table-body', all: 'deals-check-all', count: 'deals-bulk-count', result: 'deals-bulk-result' },
        contacts: { body: 'contacts-table-body', all: 'contacts-check-all', count: 'contacts-bulk-count', result: 'contacts-bulk-result' }
    };

    function config(grid) { return GRIDS[grid]; }

    function selectedIds(grid) {
        var cfg = config(grid);
        var body = document.getElementById(cfg.body);
        if (!body) return [];
        return Array.prototype.slice
            .call(body.querySelectorAll('input.opp-select:checked'))
            .map(function(cb) { return cb.closest('tr').dataset.id; })
            .filter(Boolean);
    }

    function refreshCount(grid) {
        var cfg = config(grid);
        var el = document.getElementById(cfg.count);
        if (el) el.textContent = selectedIds(grid).length + ' selected';
    }

    function notice(grid, text) {
        var cfg = config(grid);
        var el = document.getElementById(cfg.result);
        if (el) el.textContent = text;
        setTimeout(function() { if (el) el.textContent = ''; }, 8000);
    }

    document.addEventListener('change', function(e) {
        var grid = gridForCheckbox(e.target);
        if (!grid) return;
        var cfg = config(grid);
        if (e.target.getAttribute('id') === cfg.all) {
            var body = document.getElementById(cfg.body);
            if (body) {
                Array.prototype.slice.call(body.querySelectorAll('input.opp-select')).forEach(function(cb) {
                    cb.checked = e.target.checked;
                });
            }
        }
        refreshCount(grid);
    });

    function gridForCheckbox(el) {
        if (!el || el.type !== 'checkbox') return null;
        // The "select all" control lives in the header, outside any tbody.
        // Compare the `id` attribute: element wrappers in the desktop shell
        // make the `id` property unreliable.
        var elId = el.getAttribute ? el.getAttribute('id') : null;
        var headerMatch = Object.keys(GRIDS).filter(function(g) { return GRIDS[g].all === elId; });
        if (headerMatch[0]) return headerMatch[0];
        var body = el.closest('tbody');
        if (!body) return null;
        var match = Object.keys(GRIDS).filter(function(g) { return GRIDS[g].body === body.id; });
        return match[0] || null;
    }

    function bulk(grid, action, extra) {
        var ids = selectedIds(grid);
        if (!ids.length) { notice(grid, 'Select rows first'); return; }
        fetch('/api/crm/leads/bulk', {
            method: 'POST',
            headers: headers(true),
            body: JSON.stringify(Object.assign({ ids: ids, action: action }, extra || {}))
        }).then(function(resp) {
            if (!resp.ok) { notice(grid, 'Bulk action failed (' + resp.status + ')'); return; }
            return resp.json().then(function(r) {
                notice(grid, 'Updated ' + (r.updated || 0) + ', deleted ' + (r.deleted || 0) +
                    (r.failed && r.failed.length ? ', failed ' + r.failed.length : ''));
                window.reloadCrmGrid(grid);
            });
        });
    }

    // #2313 — the loss reason is CRM gold: churn analysis, win-back campaigns
    // and pricing decisions all read it. It is recorded only when a human picks
    // it explicitly; the old flow pre-filled the native prompt with "price", so
    // every user who pressed Enter wrote a reason nobody chose.
    var LOST_REASONS = [
        { value: 'price', label: 'Price / budget' },
        { value: 'competitor', label: 'Lost to a competitor' },
        { value: 'timing', label: 'Bad timing' },
        { value: 'no_decision', label: 'No decision' },
        { value: 'feature_gap', label: 'Missing feature' },
        { value: 'other', label: 'Other' }
    ];

    function lostReasonLabel(value) {
        for (var i = 0; i < LOST_REASONS.length; i++) {
            if (LOST_REASONS[i].value === value) return LOST_REASONS[i].label;
        }
        return value;
    }

    // Reuses the app's own #crm-modal, so this works in the desktop shell where
    // native prompt()/confirm() are blocked.
    function askLostReason(onPick) {
        var modal = document.getElementById('crm-modal');
        var content = document.getElementById('crm-modal-content');
        if (!modal || !content) {
            notice('opportunities', 'Lost-reason dialog unavailable — open CRM in the suite');
            return;
        }
        var options = LOST_REASONS.map(function(r) {
            return '<option value="' + r.value + '">' + r.label + '</option>';
        }).join('');
        content.innerHTML =
            '<div class="crm-form">' +
            '<h3 data-i18n="crm-lost-reason-title">Why was the deal lost?</h3>' +
            '<p class="crm-form-hint">The reason is saved with every selected opportunity ' +
            'and drives loss reporting.</p>' +
            '<label class="crm-form-label" for="crm-lost-reason-pick">Lost reason</label>' +
            '<select class="crm-form-input" id="crm-lost-reason-pick">' +
            '<option value="">Choose a reason…</option>' + options + '</select>' +
            '<div class="crm-form-actions">' +
            '<button type="button" class="btn-secondary" id="crm-lost-cancel">Cancel</button>' +
            '<button type="button" class="btn-danger" id="crm-lost-confirm" disabled>Mark lost</button>' +
            '</div></div>';
        var pick = document.getElementById('crm-lost-reason-pick');
        var confirmBtn = document.getElementById('crm-lost-confirm');
        var cancelBtn = document.getElementById('crm-lost-cancel');
        function close() { window.closeCrmModal(); }
        if (pick) {
            pick.addEventListener('change', function () {
                confirmBtn.disabled = !pick.value;
            });
            pick.focus();
        }
        if (cancelBtn) cancelBtn.addEventListener('click', close);
        if (confirmBtn) {
            confirmBtn.addEventListener('click', function () {
                if (!pick || !pick.value) return;
                var reason = pick.value;
                close();
                onPick(reason);
            });
        }
        if (typeof window.openCrmModal === 'function') window.openCrmModal();
        else modal.classList.add('open');
    }

    function closeOpportunities(won, lostReason) {
        var ids = selectedIds('opportunities');
        if (!ids.length) { notice('opportunities', 'Select rows first'); return; }
        if (won === false && !lostReason) {
            askLostReason(function (reason) {
                if (!reason) return; // cancelled — nothing is written
                closeOpportunities(false, reason);
            });
            return;
        }
        var done = 0;
        var failed = 0;
        ids.forEach(function(id) {
            fetch('/api/crm/opportunities/' + id + '/close', {
                method: 'POST',
                headers: headers(true),
                body: JSON.stringify({ won: won, lost_reason: won ? null : lostReason })
            }).then(function(resp) {
                if (resp.ok) done++; else failed++;
                if (done + failed === ids.length) {
                    notice('opportunities', 'Closed ' + done + (failed ? ', failed ' + failed : ''));
                    window.reloadCrmGrid('opportunities');
                }
            });
        });
    }

    function refreshCampaigns() {
        var list = document.getElementById('crmCampaignsList');
        if (list && window.htmx) htmx.trigger(list, 'load');
    }

    function onClick(id, handler) {
        var el = document.getElementById(id);
        if (el) el.addEventListener('click', handler);
    }

    // Opportunities grid
    onClick('opps-bulk-stage-btn', function() {
        var stage = document.getElementById('opps-bulk-stage').value;
        bulk('opportunities', 'stage', { stage: stage });
    });
    onClick('opps-close-won-btn', function() { closeOpportunities(true); });
    onClick('opps-close-lost-btn', function() { closeOpportunities(false); });
    onClick('opps-bulk-delete-btn', function() {
        if (confirm('Delete the selected records? This is recorded in the audit trail.')) {
            bulk('opportunities', 'delete');
        }
    });

    // Deals grid
    onClick('deals-bulk-stage-btn', function() { bulk('deals', 'stage', { stage: 'won' }); });
    onClick('deals-bulk-delete-btn', function() {
        if (confirm('Delete the selected deals? This is recorded in the audit trail.')) {
            bulk('deals', 'delete');
        }
    });

    // Contacts grid
    onClick('contacts-bulk-delete-btn', function() {
        var ids = selectedIds('contacts');
        if (!ids.length) { notice('contacts', 'Select rows first'); return; }
        if (!confirm('Delete the selected contacts? This is recorded in the audit trail.')) return;
        fetch('/api/crm/contacts/bulk', {
            method: 'POST',
            headers: headers(true),
            body: JSON.stringify({ ids: ids, action: 'delete' })
        }).then(function(resp) {
            if (!resp.ok) { notice('contacts', 'Delete failed (' + resp.status + ')'); return; }
            return resp.json().then(function(r) {
                notice('contacts', 'Deleted ' + (r.deleted || 0) +
                    (r.failed && r.failed.length ? ', failed ' + r.failed.length : ''));
                window.reloadCrmGrid('contacts');
            });
        });
    });

    // ── CSV export / import (dry-run first, #1456) ─────────────────────────
    function exportCsv(grid, endpoint, filename) {
        fetch(endpoint, { headers: headers() })
            .then(function(resp) {
                if (!resp.ok) { notice(grid, 'Export failed (' + resp.status + ')'); return; }
                var total = resp.headers.get('X-Total-Count');
                return resp.blob().then(function(blob) {
                    var url = URL.createObjectURL(blob);
                    var a = document.createElement('a');
                    a.href = url;
                    a.download = filename;
                    a.click();
                    URL.revokeObjectURL(url);
                    notice(grid, total ? ('Exported ' + total + ' rows') : 'Export downloaded');
                });
            });
    }

    function importCsv(grid, endpoint, input) {
        var file = input.files && input.files[0];
        if (!file) return;
        file.text().then(function(text) {
            fetch(endpoint + '?dry_run=true', {
                method: 'POST', headers: headers({ 'Content-Type': 'text/csv' }), body: text
            }).then(function(dry) {
                if (!dry.ok) { notice(grid, 'Validation failed (' + dry.status + ')'); return; }
                return dry.json().then(function(report) {
                    var summary = 'Validated: ' + report.imported + ' rows, ' +
                        report.skipped_duplicates + ' duplicates, ' + report.errors.length + ' errors';
                    if (!confirm(summary + '. Commit this import?')) { notice(grid, summary + ' — not committed'); return; }
                    fetch(endpoint, {
                        method: 'POST', headers: headers({ 'Content-Type': 'text/csv' }), body: text
                    }).then(function(resp) {
                        if (!resp.ok) { notice(grid, 'Import failed (' + resp.status + ')'); return; }
                        return resp.json().then(function(result) {
                            notice(grid, 'Imported ' + result.imported +
                                (result.skipped_duplicates ? ', duplicates ' + result.skipped_duplicates : '') +
                                (result.errors && result.errors.length ? ', errors ' + result.errors.length : ''));
                            window.reloadCrmGrid(grid);
                        });
                    });
                });
            });
        });
        input.value = '';
    }

    onClick('opps-export-btn', function() { exportCsv('opportunities', '/api/crm/leads/export', 'leads-export.csv'); });
    onClick('contacts-export-btn', function() { exportCsv('contacts', '/api/crm/contacts/export', 'contacts-export.csv'); });

    var oppsImport = document.getElementById('opps-import-file');
    onClick('opps-import-btn', function() { if (oppsImport) oppsImport.click(); });
    if (oppsImport) {
        oppsImport.addEventListener('change', function() { importCsv('opportunities', '/api/crm/leads/import', oppsImport); });
    }
    var contactsImport = document.getElementById('contacts-import-file');
    onClick('contacts-import-btn', function() { if (contactsImport) contactsImport.click(); });
    if (contactsImport) {
        contactsImport.addEventListener('change', function() { importCsv('contacts', '/api/crm/contacts/import', contactsImport); });
    }

    // ── Campaigns: create, send, metrics ────────────────────────────────────
    // The form lives inside the shared modal and is cloned from a <template>, so
    // the submit is handled by delegation: elements returned by
    // getElementById/querySelector in the desktop shell are wrapped and do not
    // reliably expose their methods.
    var campaignDraft = { id: null, campaign_type: 'outreach' };

    function campaignPayload(form) {
        var data = { campaign_type: campaignDraft.campaign_type };
        var fd = new FormData(form);
        fd.forEach(function (value, key) {
            if (!value) return;
            data[key] = key === 'budget' ? parseFloat(value) : value;
        });
        return data;
    }

    onClick('campaign-new-btn', function() {
        var tpl = document.getElementById('campaign-form-template');
        var content = document.getElementById('crm-modal-content');
        content.innerHTML = '';
        content.appendChild(tpl.content.cloneNode(true));
        campaignDraft = { id: null, campaign_type: 'outreach' };
        window.openCrmModal();
    });

    document.addEventListener('submit', function(ev) {
        var form = ev.target;
        if (!form || form.getAttribute('id') !== 'campaignForm') return;
        ev.preventDefault();
        var data = campaignPayload(form);
        var editing = campaignDraft.id;
        var url = editing ? '/api/crm/campaigns/' + editing : '/api/crm/campaigns';
        fetch(url, {
            method: editing ? 'PUT' : 'POST',
            headers: window.authHeaders({ 'Content-Type': 'application/json' }),
            body: JSON.stringify(data)
        }).then(function(resp) {
            if (!resp.ok) { notice('campaigns', (editing ? 'Update' : 'Create') + ' failed (' + resp.status + ')'); return; }
            window.closeCrmModal();
            notice('campaigns', editing ? 'Campaign updated' : 'Campaign created');
            refreshCampaigns();
        });
    });

    // Campaign edit — same modal and same delegated submit, with the id seeded.
    document.addEventListener('click', function(e) {
        var edit = e.target.closest('[data-action="edit"][data-entity="campaign"]');
        if (!edit) return;
        var id = edit.dataset.id;
        fetch('/api/crm/campaigns/' + id, { headers: window.authHeaders() })
            .then(function(r) { return r.ok ? r.json() : Promise.reject(r.status); })
            .then(function(campaign) {
                var tpl = document.getElementById('campaign-form-template');
                var content = document.getElementById('crm-modal-content');
                content.innerHTML = '';
                content.appendChild(tpl.content.cloneNode(true));
                var form = content.querySelector('#campaignForm');
                form.querySelector('[name="name"]').value = campaign.name || '';
                form.querySelector('[name="channel"]').value = campaign.channel || 'email';
                if (campaign.budget != null) form.querySelector('[name="budget"]').value = campaign.budget;
                campaignDraft = { id: id, campaign_type: campaign.campaign_type || 'outreach' };
                window.openCrmModal();
            })
            .catch(function(status) { notice('campaigns', 'Campaign unavailable (' + status + ')'); });
    });

    document.addEventListener('click', function(e) {
        var send = e.target.closest('.campaign-send-btn');
        if (send) {
            var id = send.dataset.id;
            if (!confirm('Send this campaign to its recipients now?')) return;
            fetch('/api/crm/campaigns/' + id + '/send', {
                method: 'POST', headers: headers(true), body: '{}'
            }).then(function(resp) {
                notice('campaigns', resp.ok ? 'Send dispatched' : 'Send failed (' + resp.status + ')');
                refreshCampaigns();
            });
            return;
        }
        var metrics = e.target.closest('.campaign-metrics-btn');
        if (metrics) {
            var campaignId = metrics.dataset.id;
            fetch('/api/crm/metrics/campaign/' + campaignId, { headers: headers() })
                .then(function(r) { return r.ok ? r.json() : Promise.reject(r.status); })
                .then(function(data) {
                    var box = document.getElementById('campaign-result');
                    if (box) {
                        box.textContent = 'Sent ' + (data.total_sent || 0) + ' · delivered ' +
                            (data.total_delivered || 0) + ' · opened ' + (data.total_opened || 0) +
                            ' · clicked ' + (data.total_clicked || 0);
                    }
                })
                .catch(function(status) { notice('campaigns', 'Metrics unavailable (' + status + ')'); });
        }
    });

    // ── Audit drawer ───────────────────────────────────────────────────────
    var auditDrawer = document.getElementById('crm-audit-drawer');
    var auditBtn = document.getElementById('crm-audit-btn');
    function escAudit(value) {
        return String(value == null ? '' : value).replace(/[&<>"']/g, function(c) {
            return ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c];
        });
    }
    if (auditBtn && auditDrawer) {
        auditBtn.addEventListener('click', function() {
            var body = document.getElementById('crm-audit-body');
            body.innerHTML = '<div class="crm-audit-empty">Loading…</div>';
            auditDrawer.classList.add('open');
            fetch('/api/crm/audit?limit=60', { headers: headers() })
                .then(function(r) { return r.ok ? r.json() : Promise.reject(r.status); })
                .then(function(rows) {
                    if (!rows.length) { body.innerHTML = '<div class="crm-audit-empty">No audit entries yet</div>'; return; }
                    body.innerHTML = rows.map(function(r) {
                        var when = (r.created_at || '').replace('T', ' ').slice(0, 16);
                        var target = r.entity_id ? ' · ' + r.entity_id.slice(0, 8) : '';
                        return '<div class="crm-audit-row">' +
                            '<span class="crm-audit-when">' + escAudit(when) + '</span>' +
                            '<span class="crm-audit-action">' + escAudit(r.entity + '/' + r.action + target) + '</span>' +
                            '<span class="crm-audit-actor">' + escAudit(r.actor_email || 'anonymous') + '</span>' +
                            '</div>';
                    }).join('');
                })
                .catch(function(status) {
                    body.innerHTML = '<div class="crm-audit-empty">Audit unavailable (' + status + ')</div>';
                });
        });
        document.getElementById('crm-audit-close').addEventListener('click', function() {
            auditDrawer.classList.remove('open');
        });
    }
})();
