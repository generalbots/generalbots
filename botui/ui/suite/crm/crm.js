/* #1441 — CRM app shell: tab routing, modal, API-driven kanban and the
   deep-link consumer. Record CRUD lives in crm-records.js, bulk/CSV in
   crm-bulk.js, stage administration in crm-stages.js. */
"use strict";
(function() {
    // Tabs — each view is a fragment container loaded by its own module.
    document.querySelectorAll('.crm-tab').forEach(function(tab) {
        tab.addEventListener('click', function() {
            document.querySelectorAll('.crm-tab').forEach(function(t) { t.classList.remove('active'); });
            document.querySelectorAll('.crm-view').forEach(function(v) { v.classList.remove('active'); });
            this.classList.add('active');
            var view = this.dataset.view;
            var panel = document.getElementById('crm-' + view + '-view');
            if (panel) panel.classList.add('active');
        });
    });

    // #1441 B — the New button opens a menu instead of hardcoding the lead
    // form; the menu entries are handled in crm-records.js.
    var newBtn = document.getElementById('crm-new-btn');
    var newDropdown = document.getElementById('crm-new-dropdown');
    if (newBtn && newDropdown) {
        newBtn.addEventListener('click', function(ev) {
            ev.stopPropagation();
            newDropdown.hidden = !newDropdown.hidden;
            newBtn.setAttribute('aria-expanded', String(!newDropdown.hidden));
        });
        document.addEventListener('click', function(ev) {
            if (!newDropdown.contains(ev.target) && ev.target !== newBtn) {
                newDropdown.hidden = true;
                newBtn.setAttribute('aria-expanded', 'false');
            }
        });
    }

    // Modal helpers (shared with the partials' inline close buttons).
    window.openCrmModal = function() {
        document.getElementById('crm-modal').classList.add('open');
    };
    window.closeCrmModal = function() {
        document.getElementById('crm-modal').classList.remove('open');
        var content = document.getElementById('crm-modal-content');
        if (content) content.innerHTML = '';
    };

    function authHeaders(extra) {
        return Object.assign({
            'Authorization': 'Bearer ' + (localStorage.getItem('gb-access-token') || '')
        }, extra || {});
    }
    window.authHeaders = authHeaders;

    // ── Kanban ──────────────────────────────────────────────────────────────
    var pipelineRoot = document.getElementById('crm-pipeline-view');
    if (pipelineRoot) {
        // Delegated on the container so columns rebuilt from the stages API
        // (#1441 A4) keep drag-and-drop working.
        pipelineRoot.addEventListener('dragstart', function(e) {
            var card = e.target.closest('.pipeline-card');
            if (!card) return;
            e.dataTransfer.setData('text/plain', card.dataset.id || '');
            e.dataTransfer.effectAllowed = 'move';
            card.classList.add('dragging');
        });
        pipelineRoot.addEventListener('dragend', function() {
            pipelineRoot.querySelectorAll('.pipeline-card.dragging')
                .forEach(function(c) { c.classList.remove('dragging'); });
        });
        pipelineRoot.addEventListener('dragover', function(e) {
            var column = e.target.closest('.pipeline-cards');
            if (!column) return;
            e.preventDefault();
            e.dataTransfer.dropEffect = 'move';
            column.classList.add('drag-over');
        });
        pipelineRoot.addEventListener('dragleave', function(e) {
            var column = e.target.closest('.pipeline-cards');
            if (column && !column.contains(e.relatedTarget)) column.classList.remove('drag-over');
        });
        pipelineRoot.addEventListener('drop', function(e) {
            var column = e.target.closest('.pipeline-cards');
            if (!column) return;
            e.preventDefault();
            column.classList.remove('drag-over');
            var cardId = e.dataTransfer.getData('text/plain');
            var card = pipelineRoot.querySelector('.pipeline-card[data-id="' + cardId + '"]');
            var newStage = column.closest('.pipeline-column').dataset.stage;
            if (!cardId || !newStage) return;
            if (card && card.closest('.pipeline-column') === column.closest('.pipeline-column')) return;

            if (card) column.appendChild(card);
            fetch('/api/crm/leads/' + cardId + '/stage?stage=' + encodeURIComponent(newStage), {
                method: 'PUT', headers: authHeaders()
            }).then(function() {
                document.body.dispatchEvent(new CustomEvent('crm:stage-changed'));
            });
        });
    }

    // Stages are org-configurable (#1441 A4): the kanban renders from
    // /api/crm/pipeline/stages; the static six columns above are the fallback.
    window.renderPipelineStages = function(stages) {
        if (!Array.isArray(stages) || !stages.length) return;
        var container = document.querySelector('.pipeline-container');
        if (!container) return;
        var sorted = stages.slice().sort(function(a, b) { return (a.stage_order || 0) - (b.stage_order || 0); });
        container.innerHTML =
            '<div class="stage-toolbar">' +
            '<button id="stage-add-btn" class="btn-secondary" title="Add a custom stage">+ Add stage</button>' +
            '<span id="stage-admin-result" class="stage-admin-result"></span>' +
            '</div>' + sorted.map(function(s, i) {
                return '<div class="pipeline-column' + (s.name === 'won' ? ' won' : '') +
                    (s.name === 'lost' ? ' lost' : '') + '" data-stage="' + s.name + '" data-stage-id="' + (s.id || '') + '">' +
                    '<div class="pipeline-header">' +
                    '<span class="pipeline-title">' + s.name + '</span>' +
                    '<span class="stage-actions">' +
                    '<button class="stage-action" data-action="rename" title="Rename stage">✎</button>' +
                    '<button class="stage-action" data-action="probability" title="Set probability">%</button>' +
                    '<button class="stage-action" data-action="delete" title="Delete stage">✕</button>' +
                    '</span>' +
                    '<span class="pipeline-count" hx-get="/api/crm/count?stage=' + encodeURIComponent(s.name) + '" hx-trigger="load">0</span>' +
                    '</div>' +
                    '<div class="pipeline-cards" hx-get="/api/crm/pipeline?stage=' + encodeURIComponent(s.name) + '" hx-trigger="load" hx-swap="innerHTML"></div>' +
                    (i === 0 ? '<button class="pipeline-add" data-create="lead">+ Add Lead</button>' : '') +
                    '</div>';
            }).join('');
        window.dispatchEvent(new CustomEvent('crm:stages-rendered'));
        if (window.htmx) htmx.process(container);
    };

    function loadStages() {
        return fetch('/api/crm/pipeline/stages', { headers: authHeaders() })
            .then(function(r) { return r.ok ? r.json() : Promise.reject(r.status); })
            .then(function(stages) { window.renderPipelineStages(stages); })
            .catch(function() { /* keep the static fallback columns */ });
    }
    window.loadCrmStages = loadStages;
    loadStages();

    // Any successful stage write (kanban card button, or a drag that already
    // dispatched the event itself) refreshes the column counts and the cards.
    document.body.addEventListener('htmx:afterRequest', function(e) {
        var path = (e.detail && e.detail.pathInfo && e.detail.pathInfo.requestPath) || '';
        if (path.indexOf('/stage?stage=') >= 0) {
            document.body.dispatchEvent(new CustomEvent('crm:stage-changed'));
        }
    });

    document.body.addEventListener('crm:stage-changed', function() {
        document.querySelectorAll('.pipeline-count').forEach(function(el) {
            if (window.htmx) htmx.trigger(el, 'load');
        });
        document.querySelectorAll('.pipeline-cards').forEach(function(el) {
            if (window.htmx) htmx.trigger(el, 'load');
        });
    });

    // ── #1437 deep-link consumer ─────────────────────────────────────────────
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
        var contactsTab = document.querySelector('.crm-tab[data-view="contacts"]');
        if (contactsTab && !contactsTab.classList.contains('active')) contactsTab.click();
        var tableBody = document.getElementById('contacts-table-body');
        if (!tableBody) return;
        var attempts = 0;
        var poll = setInterval(function() {
            var row = tableBody.querySelector('tr[data-id="' + personId + '"]');
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
        var params = e.detail && e.detail.params;
        if (params && params.person_id) {
            window.applyCrmPersonDeepLink(params.person_id, (e.detail && e.detail.appId) || 'crm');
        }
    });

    if (window.i18n && window.i18n.translatePage) window.i18n.translatePage();
})();
