let currentGoalsView = 'dashboard';

function switchGoalsView(view) {
    currentGoalsView = view;

    document.querySelectorAll('.goals-app .view-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.view === view);
    });

    document.querySelectorAll('.goals-app .view-container').forEach(container => {
        container.classList.toggle('active', container.id === `${view}-view`);
    });
}

function toggleGoalsPanel() {
    const panel = document.getElementById('details-panel');
    panel.classList.toggle('collapsed');
}

function selectObjective(objectiveId) {
    htmx.ajax('GET', `/api/goals/objectives/${objectiveId}`, {
        target: '#objective-details',
        swap: 'innerHTML'
    });

    const panel = document.getElementById('details-panel');
    panel.classList.remove('collapsed');
}

function updateProgressRing(percentage) {
    const ring = document.querySelector('.progress-bar');
    if (ring) {
        ring.setAttribute('stroke-dasharray', `${percentage}, 100`);
    }
}

function checkIn(krId) {
    htmx.ajax('GET', `/api/goals/key-results/${krId}/check-in`, {
        target: '#goals-modal',
        swap: 'innerHTML'
    });
}

document.addEventListener('DOMContentLoaded', function() {
    document.body.addEventListener('htmx:afterSwap', function(event) {
        if (event.detail.target.id === 'objectives-list' ||
            event.detail.target.classList.contains('objectives-grid')) {
            const hasObjectives = event.detail.target.children.length > 0;
            document.querySelector('.goals-app').classList.toggle('has-objectives', hasObjectives);
        }
    });
});
