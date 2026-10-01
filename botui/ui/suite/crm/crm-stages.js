/* #1455 — pipeline stage administration (add / rename / probability /
   delete). Split out of crm-p2.js; the list is re-read from the API so the
   kanban stays the single source of truth. */
"use strict";
(function() {
    function showStageResult(text) {
        var el = document.getElementById('stage-admin-result');
        if (el) el.textContent = text;
        setTimeout(function() { if (el) el.textContent = ''; }, 8000);
    }

    function api(url, opts) {
        return fetch(url, Object.assign({
            headers: window.authHeaders({ 'Content-Type': 'application/json' })
        }, opts || {}));
    }

    function reloadStages() {
        if (window.loadCrmStages) return window.loadCrmStages();
        fetch('/api/crm/pipeline/stages', { headers: window.authHeaders() })
            .then(function(r) { return r.ok ? r.json() : Promise.reject(r.status); })
            .then(function(stages) { window.renderPipelineStages(stages); })
            .catch(function(status) { showStageResult('Stage refresh failed (' + status + ')'); });
    }

    function openAddStage() {
        var modal = document.getElementById('crm-modal');
        var content = document.getElementById('crm-modal-content');
        if (!modal || !content) return;
        content.innerHTML =
            '<h3>Add stage</h3>' +
            '<div class="crm-form-group"><label class="crm-form-label">Name</label>' +
            '<input id="stage-name-input" class="crm-form-input" maxlength="60" placeholder="e.g. Contract review"></div>' +
            '<div class="crm-form-group"><label class="crm-form-label">Probability %</label>' +
            '<input id="stage-prob-input" class="crm-form-input" type="number" min="0" max="100" value="50"></div>' +
            '<div class="crm-form-actions">' +
            '<button class="crm-form-btn secondary" onclick="closeCrmModal()">Cancel</button>' +
            '<button class="crm-form-btn primary" id="stage-create-btn">Create stage</button>' +
            '</div>';
        modal.classList.add('open');
        document.getElementById('stage-name-input').focus();
        document.getElementById('stage-create-btn').addEventListener('click', function() {
            var name = (document.getElementById('stage-name-input').value || '').trim();
            var probability = parseInt(document.getElementById('stage-prob-input').value, 10) || 0;
            if (!name) { showStageResult('Stage name required'); return; }
            api('/api/crm/pipeline/stages', {
                method: 'POST', body: JSON.stringify({ name: name, probability: probability })
            }).then(function(resp) {
                if (!resp.ok) {
                    showStageResult('Create failed (' + resp.status + ')');
                    return;
                }
                window.closeCrmModal();
                showStageResult('Stage created');
                reloadStages();
            });
        });
    }

    // Delegated: the kanban re-renders on every stage change, so the toolbar
    // button is bound by delegation as well.
    document.addEventListener('click', function(e) {
        if (e.target.closest('#stage-add-btn')) {
            openAddStage();
            return;
        }
        var btn = e.target.closest('.stage-action');
        if (!btn) return;
        var col = btn.closest('.pipeline-column');
        var stageId = col && col.dataset.stageId;
        var stageName = (col && col.dataset.stage) || '';
        if (!stageId) { showStageResult('Static column — configure stages first'); return; }

        if (btn.dataset.action === 'rename') {
            var name = prompt('Rename stage "' + stageName + '" to:', stageName);
            if (!name || name.trim() === stageName) return;
            api('/api/crm/pipeline/stages/' + stageId, {
                method: 'PUT', body: JSON.stringify({ name: name.trim() })
            }).then(function(resp) {
                showStageResult(resp.ok ? 'Stage renamed' : 'Rename failed (' + resp.status + ')');
                if (resp.ok) reloadStages();
            });
        } else if (btn.dataset.action === 'probability') {
            var raw = prompt('Win probability % for "' + stageName + '":', '50');
            if (raw === null) return;
            var probability = parseInt(raw, 10);
            if (isNaN(probability) || probability < 0 || probability > 100) {
                showStageResult('Probability must be 0-100');
                return;
            }
            api('/api/crm/pipeline/stages/' + stageId, {
                method: 'PUT', body: JSON.stringify({ probability: probability })
            }).then(function(resp) {
                showStageResult(resp.ok ? 'Probability updated' : 'Update failed (' + resp.status + ')');
                if (resp.ok) reloadStages();
            });
        } else if (btn.dataset.action === 'delete') {
            if (!confirm('Delete stage "' + stageName + '"? Blocked while leads still use it.')) return;
            api('/api/crm/pipeline/stages/' + stageId, { method: 'DELETE' }).then(function(resp) {
                if (resp.ok) {
                    showStageResult('Stage deleted');
                    reloadStages();
                } else if (resp.status === 409) {
                    showStageResult('Stage still holds leads — move them first');
                } else {
                    showStageResult('Delete failed (' + resp.status + ')');
                }
            });
        }
    });
})();
