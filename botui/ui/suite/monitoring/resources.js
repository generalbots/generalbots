/* Resources page JavaScript */

function refreshDiskInfo() {
    htmx.trigger('#disk-partitions', 'refresh');
}

function sortProcesses(sortBy) {
    const sortParam = `?sort=${sortBy}`;
    htmx.ajax('GET', `/api/monitoring/resources/processes${sortParam}`, {
        target: '#process-list',
        swap: 'innerHTML'
    });
}

// Color-code usage based on percentage
function updateUsageColors() {
    document.querySelectorAll('.card-value').forEach(el => {
        const value = parseInt(el.textContent);
        if (value >= 90) {
            el.classList.add('error');
            el.classList.remove('warning');
        } else if (value >= 75) {
            el.classList.add('warning');
            el.classList.remove('error');
        } else {
            el.classList.remove('warning', 'error');
        }
    });

    document.querySelectorAll('.usage-fill, .progress-fill').forEach(el => {
        const width = parseInt(el.style.width);
        if (width >= 90) {
            el.classList.add('error');
            el.classList.remove('warning');
        } else if (width >= 75) {
            el.classList.add('warning');
            el.classList.remove('error');
        } else {
            el.classList.remove('warning', 'error');
        }
    });
}

// Run on HTMX swap for resources
document.body.addEventListener('htmx:afterSwap', function(evt) {
    if (evt.target.closest('.resources-container')) {
        updateUsageColors();
    }
});

// Initialize on page load
document.addEventListener('DOMContentLoaded', function() {
    updateUsageColors();
});
