/* Alerts page JavaScript */

function switchTab(tab, btn) {
    // Update tab buttons
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');

    // Update panels
    document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
    document.getElementById(`${tab}-panel`).classList.add('active');
}

function filterAlerts() {
    const severity = document.getElementById('severity-filter').value;
    const status = document.getElementById('status-filter').value;
    const items = document.querySelectorAll('.alert-item');

    items.forEach(item => {
        let show = true;
        if (severity !== 'all' && !item.classList.contains(severity)) {
            show = false;
        }
        if (status !== 'all' && item.dataset.status !== status) {
            show = false;
        }
        item.style.display = show ? '' : 'none';
    });
}

function searchAlerts(query) {
    const items = document.querySelectorAll('.alert-item');
    const lowerQuery = query.toLowerCase();

    items.forEach(item => {
        const text = item.textContent.toLowerCase();
        item.style.display = text.includes(lowerQuery) ? '' : 'none';
    });
}

function filterRules() {
    const status = document.getElementById('rule-status-filter').value;
    const category = document.getElementById('rule-category-filter').value;
    const cards = document.querySelectorAll('.rule-card:not(.skeleton)');

    cards.forEach(card => {
        let show = true;
        if (status === 'enabled' && card.classList.contains('disabled')) {
            show = false;
        }
        if (status === 'disabled' && !card.classList.contains('disabled')) {
            show = false;
        }
        if (category !== 'all' && card.dataset.category !== category) {
            show = false;
        }
        card.style.display = show ? '' : 'none';
    });
}

function filterHistory() {
    htmx.trigger('#history-list', 'refresh');
}

function openCreateAlertModal() {
    document.getElementById('create-alert-modal').classList.add('open');
}

function closeCreateAlertModal() {
    document.getElementById('create-alert-modal').classList.remove('open');
    document.getElementById('create-alert-form').reset();
}

function openAlertDetailModal(alertId) {
    const modal = document.getElementById('alert-detail-modal');
    const content = document.getElementById('alert-detail-content');

    // Load alert details
    htmx.ajax('GET', `/api/monitoring/alerts/${alertId}`, {
        target: content,
        swap: 'innerHTML'
    });

    modal.classList.add('open');
}

function closeAlertDetailModal() {
    document.getElementById('alert-detail-modal').classList.remove('open');
}

function acknowledgeAlert(alertId) {
    htmx.ajax('POST', `/api/monitoring/alerts/${alertId}/acknowledge`, {
        swap: 'none'
    }).then(() => {
        htmx.trigger('#alerts-list', 'refresh');
    });
}

function acknowledgeAllAlerts() {
    if (confirm('Are you sure you want to acknowledge all active alerts?')) {
        htmx.ajax('POST', '/api/ui/monitoring/alerts/acknowledge-all', {
            swap: 'none'
        }).then(() => {
            htmx.trigger('#alerts-list', 'refresh');
        });
    }
}

function silenceAlert(alertId, duration) {
    htmx.ajax('POST', `/api/monitoring/alerts/${alertId}/silence?duration=${duration}`, {
        swap: 'none'
    }).then(() => {
        htmx.trigger('#alerts-list', 'refresh');
    });
}

function toggleRule(ruleId, enabled) {
    htmx.ajax('PATCH', `/api/monitoring/alerts/rules/${ruleId}`, {
        values: { enabled: enabled },
        swap: 'none'
    }).then(() => {
        htmx.trigger('#rules-grid', 'refresh');
    });
}

function editRule(ruleId) {
    // Load rule into form and open modal
    htmx.ajax('GET', `/api/monitoring/alerts/rules/${ruleId}`, {
        target: '#create-alert-form',
        swap: 'outerHTML'
    }).then(() => {
        openCreateAlertModal();
    });
}

function deleteRule(ruleId) {
    if (confirm('Are you sure you want to delete this alert rule?')) {
        htmx.ajax('DELETE', `/api/monitoring/alerts/rules/${ruleId}`, {
            swap: 'none'
        }).then(() => {
            htmx.trigger('#rules-grid', 'refresh');
        });
    }
}

function exportHistory() {
    const timeFilter = document.getElementById('history-time-filter').value;
    window.open(`/api/monitoring/alerts/history/export?range=${timeFilter}`, '_blank');
}

// Close modals on escape key
document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') {
        closeCreateAlertModal();
        closeAlertDetailModal();
    }
});

// Close modals when clicking overlay
document.querySelectorAll('.modal-overlay').forEach(overlay => {
    overlay.addEventListener('click', function(e) {
        if (e.target === this) {
            this.classList.remove('open');
        }
    });
});

// Update counts from HTMX responses
document.body.addEventListener('htmx:afterSwap', function(evt) {
    if (evt.target.id === 'alerts-list') {
        const count = evt.target.querySelectorAll('.alert-item').length;
        const countEl = document.getElementById('active-count');
        if (countEl) countEl.textContent = count;
    }
    if (evt.target.id === 'rules-grid') {
        const count = evt.target.querySelectorAll('.rule-card:not(.skeleton)').length;
        const countEl = document.getElementById('rules-count');
        if (countEl) countEl.textContent = count;
    }
});
