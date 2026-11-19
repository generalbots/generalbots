/* #1441 P2/#1456 — campaigns (create, send, metrics) and the audit drawer,
   split out of crm-bulk.js so every module stays under the 450-line limit.
   Behaviour is unchanged; the small helpers below mirror crm-bulk.js. */
"use strict";
(function() {
    function headers(json) {
        return window.authHeaders(json ? { 'Content-Type': 'application/json' } : {});
    }

    function t(key, fallback, params) {
        return window.gbT ? window.gbT(key, fallback, params) : fallback;
    }

    var GRIDS = {
        opportunities: { body: 'opportunities-table-body', all: 'opps-check-all', count: 'opps-bulk-count', result: 'opps-bulk-result' },
        deals: { body: 'deals-table-body', all: 'deals-check-all', count: 'deals-bulk-count', result: 'deals-bulk-result' },
        contacts: { body: 'contacts-table-body', all: 'contacts-check-all', count: 'contacts-bulk-count', result: 'contacts-bulk-result' }
    };

    function config(grid) { return GRIDS[grid]; }

    function notice(grid, text) {
        var cfg = config(grid);
        var elId = cfg && cfg.result ? cfg.result : (grid === 'campaigns' ? 'campaign-result' : null);
        var el = elId ? document.getElementById(elId) : null;
        if (el) {
            var lower = String(text || '').toLower();
            var isAlert = /fail|error|unavailable|blocked|required|first|denied/.test(lower);
            el.setAttribute('role', isAlert ? 'alert' : 'status');
            el.setAttribute('aria-live', isAlert ? 'assertive' : 'polite');
            el.setAttribute('aria-atomic', 'true');
            el.textContent = text;
        }
        setTimeout(function() { if (el) el.textContent = ''; }, 8000);
    }

    function onClick(id, handler) {
        var el = document.getElementById(id);
        if (el) el.addEventListener('click', handler);
    }

    function gbConfirm(message, onYes) {
        if (window.GBModal && typeof window.GBModal.confirm === 'function') {
            window.GBModal.confirm(message, 'Confirm', onYes);
        }
    }

    function refreshCampaigns() {
        var list = document.getElementById('crmCampaignsList');
        if (list && window.htmx) htmx.trigger(list, 'load');
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
        })
          .catch(function (fetchError) {
            window.gbLogError ? gbLogError(fetchError, "onClick")
                                : console.error("onClick", fetchError);
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
            gbConfirm('Send this campaign to its recipients now?', function() {
                fetch('/api/crm/campaigns/' + id + '/send', {
                    method: 'POST', headers: headers(true), body: '{}'
                }).then(function(resp) {
                    notice('campaigns', resp.ok ? 'Send dispatched' : 'Send failed (' + resp.status + ')');
                    refreshCampaigns();
                })
                  .catch(function (fetchError) {
                    window.gbLogError ? gbLogError(fetchError, "gbConfirm")
                                        : console.error("gbConfirm", fetchError);
                  });
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
                        var metrics = {
                            sent: data.total_sent || 0,
                            delivered: data.total_delivered || 0,
                            opened: data.total_opened || 0,
                            clicked: data.total_clicked || 0
                        };
                        box.textContent = t(
                            'crm-campaign-metrics-sent',
                            'Sent ' + metrics.sent + ' · delivered ' + metrics.delivered +
                                ' · opened ' + metrics.opened + ' · clicked ' + metrics.clicked,
                            metrics
                        );
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
            body.setAttribute('role', 'status');
            body.setAttribute('aria-live', 'polite');
            body.innerHTML = '<div class="crm-audit-empty">' + t('crm-loading', 'Loading…') + '</div>';
            auditDrawer.classList.add('open');
            fetch('/api/crm/audit?limit=60', { headers: headers() })
                .then(function(r) { return r.ok ? r.json() : Promise.reject(r.status); })
                .then(function(rows) {
                    body.setAttribute('role', 'status');
                    body.setAttribute('aria-live', 'polite');
                    if (!rows.length) { body.innerHTML = '<div class="crm-audit-empty">' + t('crm-audit-empty', 'No audit entries yet') + '</div>'; return; }
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
                    body.setAttribute('role', 'alert');
                    body.setAttribute('aria-live', 'assertive');
                    body.innerHTML = '<div class="crm-audit-empty">' +
                        t('crm-audit-unavailable', 'Audit unavailable (' + status + ')', { status: status }) +
                        '</div>';
                });
        });
        document.getElementById('crm-audit-close').addEventListener('click', function() {
            auditDrawer.classList.remove('open');
        });
    }
})();
