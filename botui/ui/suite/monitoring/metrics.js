/* Metrics page JavaScript */

function updateMetricsRange(range) {
    localStorage.setItem('metrics-time-range', range);
    refreshMetrics();
}

function refreshMetrics() {
    // Refresh all metric components
    htmx.trigger('.key-metrics', 'refresh');
    htmx.trigger('#requests-chart', 'refresh');
    htmx.trigger('#latency-chart', 'refresh');
    htmx.trigger('#errors-chart', 'refresh');
    htmx.trigger('#throughput-chart', 'refresh');
}

function updateChartType(chart, type) {
    // This would update the chart visualization type
    console.log(`Updating ${chart} chart to ${type} type`);
    // Implementation depends on charting library
}

function filterMetrics(query) {
    const rows = document.querySelectorAll('.metrics-table tbody tr');
    const lowerQuery = query.toLowerCase();

    rows.forEach(row => {
        const text = row.textContent.toLowerCase();
        row.style.display = text.includes(lowerQuery) ? '' : 'none';
    });
}

function filterByCategory(category) {
    const rows = document.querySelectorAll('.metrics-table tbody tr');

    rows.forEach(row => {
        if (category === 'all') {
            row.style.display = '';
        } else {
            const rowCategory = row.dataset.category || '';
            row.style.display = rowCategory === category ? '' : 'none';
        }
    });
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', function() {
    // Restore time range preference
    const savedRange = localStorage.getItem('metrics-time-range');
    if (savedRange) {
        const select = document.getElementById('metrics-time-range');
        if (select) select.value = savedRange;
    }
});
