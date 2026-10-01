/* #1441 — record CRUD for the CRM app: the New-record menu, create/edit
   modals, the row action delegation (View / Edit / Delete) and the detail
   drawer with its activity timeline. */
"use strict";
(function() {
    var FORM_PARTIALS = {
        lead: '/suite/crm/partials/lead-form.html',
        contact: '/suite/crm/partials/contact-form.html',
        account: '/suite/crm/partials/account-form.html',
        deal: '/suite/crm/partials/deal-form.html',
        opportunity: '/suite/crm/partials/deal-form.html'
    };

    // REST base per entity (campaigns are owned by botmarketing).
    var ENDPOINTS = {
        lead: '/api/crm/leads',
        deal: '/api/crm/deals',
        opportunity: '/api/crm/opportunities',
        contact: '/api/crm/contacts',
        account: '/api/crm/accounts',
        campaign: '/api/crm/campaigns'
    };

    // Which grid refreshes after a mutation.
    var REFRESH = {
        lead: 'deals',
        deal: 'deals',
        opportunity: 'opportunities',
        contact: 'contacts',
        account: 'accounts',
        campaign: 'campaigns'
    };

    function headers(json) {
        return window.authHeaders(json ? { 'Content-Type': 'application/json' } : {});
    }

    function esc(value) {
        return String(value == null ? '' : value).replace(/[&<>"']/g, function(c) {
            return ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c];
        });
    }

    function notice(text) {
        var box = document.getElementById('crm-bulk-result');
        if (!box) box = document.getElementById('campaign-result');
        if (!box) return;
        box.textContent = text;
        setTimeout(function() { box.textContent = ''; }, 8000);
    }

    // ── Grid refresh + sorting (#1441 C2) ──────────────────────────────────
    var gridState = {};

    function gridBody(grid) {
        var map = {
            deals: 'deals-table-body',
            opportunities: 'opportunities-table-body',
            contacts: 'contacts-table-body',
            accounts: 'accounts-table-body'
        };
        return document.getElementById(map[grid]);
    }

    window.reloadCrmGrid = function(grid) {
        var body = gridBody(grid);
        if (!body || !window.htmx) return;
        var state = gridState[grid] || {};
        var qs = new URLSearchParams();
        if (state.stage) qs.set('stage', state.stage);
        if (state.search) qs.set('search', state.search);
        if (state.sort) qs.set('sort', state.sort);
        if (state.dir) qs.set('dir', state.dir);
        var url = '/api/ui/crm/' + grid + (qs.toString() ? '?' + qs.toString() : '');
        htmx.ajax('GET', url, { target: '#' + body.id, swap: 'innerHTML' });
    };

    function refreshEntity(entity) {
        var grid = REFRESH[entity];
        if (grid === 'campaigns') {
            var list = document.getElementById('crmCampaignsList');
            if (list && window.htmx) htmx.trigger(list, 'load');
        } else if (grid) {
            window.reloadCrmGrid(grid);
        }
        document.body.dispatchEvent(new CustomEvent('crm:stage-changed'));
    }

    var searchTimer = null;
    document.addEventListener('input', function(e) {
        if (!e.target.classList || !e.target.classList.contains('grid-search')) return;
        var grid = e.target.dataset.grid;
        var value = e.target.value.trim();
        clearTimeout(searchTimer);
        searchTimer = setTimeout(function() {
            gridState[grid] = gridState[grid] || {};
            gridState[grid].search = value;
            window.reloadCrmGrid(grid);
        }, 300);
    });

    document.addEventListener('change', function(e) {
        if (e.target.classList && e.target.classList.contains('grid-filter')) {
            var grid = e.target.dataset.grid;
            gridState[grid] = gridState[grid] || {};
            gridState[grid].stage = e.target.value;
            window.reloadCrmGrid(grid);
        }
    });

    document.addEventListener('click', function(e) {
        var header = e.target.closest('th.sortable');
        if (!header) return;
        var grid = header.dataset.grid;
        var sort = header.dataset.sort;
        gridState[grid] = gridState[grid] || {};
        gridState[grid].dir = (gridState[grid].sort === sort && gridState[grid].dir === 'asc') ? 'desc' : 'asc';
        gridState[grid].sort = sort;
        window.reloadCrmGrid(grid);
    });

    // ── Create / edit modals ───────────────────────────────────────────────
    function fillStageOptions(current) {
        var select = document.getElementById('crm-stage');
        if (!select || select.dataset.filled === '1') return;
        fetch('/api/crm/pipeline/stages', { headers: headers() })
            .then(function(r) { return r.ok ? r.json() : []; })
            .then(function(stages) {
                var names = (stages.length ? stages : [{ name: 'new' }, { name: 'qualified' }])
                    .map(function(s) { return s.name; });
                select.innerHTML = names.map(function(n) {
                    return '<option value="' + esc(n) + '">' + esc(n) + '</option>';
                }).join('');
                select.dataset.filled = '1';
                if (current) select.value = current;
            })
            .catch(function() { /* the static list stays */ });
    }

    function setField(id, value) {
        var el = document.getElementById(id);
        if (el && value !== null && value !== undefined) el.value = value;
    }

    function loadIntoForm(entity, record) {
        setField('crm-record-id', record.id);
        setField('crm-first-name', record.first_name);
        setField('crm-last-name', record.last_name);
        setField('crm-email', record.email);
        setField('crm-phone', record.phone);
        setField('crm-company', record.company);
        setField('crm-job-title', record.job_title);
        setField('crm-city', record.city);
        setField('crm-country', record.country);
        setField('crm-notes', record.notes);
        setField('crm-description', record.description);
        setField('crm-industry', record.industry);
        setField('crm-website', record.website);
        setField('crm-employees', record.employees_count);
        setField('crm-revenue', record.annual_revenue);
        setField('crm-name', record.title || record.name);
        setField('crm-value', record.value);
        setField('crm-currency', record.currency);
        setField('crm-source', record.source);
        setField('crm-lost-reason', record.lost_reason);
        setField('crm-close-date', record.expected_close_date);
        setField('crm-status', record.status);
        fillStageOptions(record.stage);
        if (record.contact_id) setField('crm-contact-select', record.contact_id);
        if (record.account_id) setField('crm-account-select', record.account_id);
        var title = document.querySelector('#crm-modal-content [data-form-title]');
        if (title) title.textContent = 'Edit ' + entity.charAt(0).toUpperCase() + entity.slice(1);
    }

    function openForm(entity, record) {
        var partial = FORM_PARTIALS[entity];
        if (!partial) return;
        fetch(partial, { headers: headers() })
            .then(function(r) { return r.text(); })
            .then(function(html) {
                var content = document.getElementById('crm-modal-content');
                content.innerHTML = html;
                var form = document.getElementById('crm-record-form');
                form.dataset.entity = entity;
                form.dataset.mode = record ? 'edit' : 'create';
                if (record) loadIntoForm(entity, record);
                else fillStageOptions(null);
                if (window.htmx) htmx.process(content);
                window.openCrmModal();
            });
    }

    window.openCrmRecordForm = function(entity, record) {
        openForm(entity, record);
    };

    // New-record menu + the "New …" buttons on each view header.
    document.addEventListener('click', function(e) {
        var trigger = e.target.closest('[data-create]');
        if (!trigger) return;
        e.preventDefault();
        openForm(trigger.dataset.create, null);
    });

    // ── Form submit ────────────────────────────────────────────────────────
    function formPayload(form) {
        var data = {};
        var numeric = ['value', 'employees_count', 'annual_revenue'];
        form.querySelectorAll('input[name], select[name], textarea[name]').forEach(function(el) {
            if (!el.name || el.name === 'id') return;
            var raw = (el.value || '').trim();
            if (raw === '') return;
            data[el.name] = numeric.indexOf(el.name) >= 0 ? parseFloat(raw) : raw;
        });
        return data;
    }

    document.addEventListener('submit', function(e) {
        var form = e.target;
        // The desktop shell wraps DOM elements, so the `id` *property* is not
        // reliable here (it can read back as a node reference). The attribute
        // is, and `getElementById` matches on it.
        if (!form || form.getAttribute('id') !== 'crm-record-form') return;
        e.preventDefault();
        var entity = form.dataset.entity;
        var id = (document.getElementById('crm-record-id') || {}).value;
        var base = ENDPOINTS[entity] || ENDPOINTS.lead;
        var url = id ? base + '/' + id : base;
        var method = id ? 'PUT' : 'POST';
        var payload = formPayload(form);

        if (entity === 'opportunity') {
            // The opportunity endpoint names the record `name`, the shared
            // deal form labels it `title`.
            payload.name = payload.title;
            delete payload.title;
        }
        if (entity === 'lead') {
            // The lead form keeps its own field names (title/first_name/…).
            var title = (document.getElementById('leadTitle') || {}).value || '';
            var first = (document.getElementById('leadFirstName') || {}).value || '';
            var last = (document.getElementById('leadLastName') || {}).value || '';
            if (!title || title === 'New Lead') {
                title = (first + ' ' + last).trim() || 'New Lead';
            }
            payload.title = title;
        }

        fetch(url, {
            method: method,
            headers: headers(true),
            body: JSON.stringify(payload)
        }).then(function(resp) {
            if (!resp.ok) {
                return resp.text().then(function(text) {
                    notice('Save failed (' + resp.status + '): ' + text.substring(0, 120));
                });
            }
            window.closeCrmModal();
            notice((id ? 'Updated ' : 'Created ') + entity);
            refreshEntity(entity);
        });
    });

    // ── Detail drawer (#1441 C1) ───────────────────────────────────────────
    function openRecordDetail(entity, id) {
        var panel = document.getElementById('record-detail-panel');
        if (!panel) return;
        panel.classList.add('open');
        panel.innerHTML = '<div class="crm-detail-empty">Loading…</div>';
        fetch('/api/ui/crm/records/' + entity + '/' + id, { headers: headers() })
            .then(function(r) { return r.ok ? r.text() : Promise.reject(r.status); })
            .then(function(html) {
                panel.innerHTML = html;
                if (window.i18n && window.i18n.translatePage) window.i18n.translatePage();
            })
            .catch(function(status) {
                panel.innerHTML = '<div class="crm-detail-empty">Record unavailable (' + status + ')</div>';
            });
    }
    window.openCrmRecordDetail = openRecordDetail;

    document.addEventListener('click', function(e) {
        var close = e.target.closest('.record-detail-close');
        if (close) {
            document.getElementById('record-detail-panel').classList.remove('open');
            return;
        }
    });

    // Activity timeline composer → POST /api/crm/activities.
    document.addEventListener('submit', function(e) {
        var form = e.target;
        if (!form.classList || !form.classList.contains('activity-form')) return;
        e.preventDefault();
        var entity = form.dataset.entity;
        var id = form.dataset.id;
        var payload = {
            activity_type: form.activity_type.value,
            subject: form.subject.value
        };
        // crm_activities links by record type; lead/opportunity share the column.
        if (entity === 'contact') payload.contact_id = id;
        else if (entity === 'account') payload.account_id = id;
        else if (entity === 'opportunity') payload.opportunity_id = id;
        else payload.lead_id = id;

        fetch('/api/crm/activities', {
            method: 'POST', headers: headers(true), body: JSON.stringify(payload)
        }).then(function(resp) {
            if (!resp.ok) { notice('Activity failed (' + resp.status + ')'); return; }
            form.reset();
            openRecordDetail(entity, id);
        });
    });

    // Lost reason from the detail panel (#1441 C7).
    document.addEventListener('click', function(e) {
        var btn = e.target.closest('[data-action="save-lost-reason"]');
        if (!btn) return;
        var input = document.getElementById('record-lost-reason');
        var entity = btn.dataset.entity;
        var id = btn.dataset.id;
        var base = entity === 'opportunity' ? ENDPOINTS.opportunity : ENDPOINTS.lead;
        fetch(base + '/' + id, {
            method: 'PUT',
            headers: headers(true),
            body: JSON.stringify({ lost_reason: input ? input.value.trim() : '' })
        }).then(function(resp) {
            notice(resp.ok ? 'Lost reason saved' : 'Save failed (' + resp.status + ')');
            if (resp.ok) openRecordDetail(entity, id);
        });
    });

    // ── Row actions (View / Edit / Delete) ─────────────────────────────────
    function fetchRecord(entity, id) {
        var base = ENDPOINTS[entity];
        if (!base) return Promise.resolve(null);
        return fetch(base + '/' + id, { headers: headers() })
            .then(function(r) { return r.ok ? r.json() : null; })
            .catch(function() { return null; });
    }

    document.addEventListener('click', function(e) {
        var btn = e.target.closest('[data-action]');
        if (!btn) return;
        var action = btn.dataset.action;
        var entity = btn.dataset.entity;
        var id = btn.dataset.id;
        if (!action || !entity || !id) return;

        if (action === 'view') {
            e.preventDefault();
            openRecordDetail(entity, id);
        } else if (action === 'edit') {
            e.preventDefault();
            // Campaigns have their own editor (crm-bulk.js).
            if (!FORM_PARTIALS[entity]) return;
            fetchRecord(entity, id).then(function(record) {
                if (record) openForm(entity === 'lead' ? 'deal' : entity, record);
                else notice('Record unavailable');
            });
        } else if (action === 'delete') {
            e.preventDefault();
            if (!confirm('Delete this ' + entity + '? The action is recorded in the audit trail.')) return;
            var base = ENDPOINTS[entity];
            if (!base) return;
            fetch(base + '/' + id, { method: 'DELETE', headers: headers() })
                .then(function(resp) {
                    notice(resp.ok ? entity + ' deleted' : 'Delete failed (' + resp.status + ')');
                    if (resp.ok) refreshEntity(entity);
                });
        }
    });
})();
