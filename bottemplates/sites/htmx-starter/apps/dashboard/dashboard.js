// Chart.js configuration
const chartColors = {
    primary: '#0ea5e9',
    success: '#10b981',
    warning: '#f59e0b',
    danger: '#ef4444',
    purple: '#8b5cf6',
    gray: '#64748b'
};

let revenueChart = null;
let categoryChart = null;

function renderRevenueChart(event) {
    const data = JSON.parse(event.detail.xhr.responseText);
    const ctx = document.getElementById('revenue-chart').getContext('2d');

    if (revenueChart) revenueChart.destroy();

    revenueChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: data.labels,
            datasets: [{
                label: 'Revenue',
                data: data.values,
                borderColor: chartColors.primary,
                backgroundColor: 'rgba(14, 165, 233, 0.1)',
                fill: true,
                tension: 0.4
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: { display: false }
            },
            scales: {
                y: {
                    beginAtZero: true,
                    grid: { color: 'rgba(0,0,0,0.05)' }
                },
                x: {
                    grid: { display: false }
                }
            }
        }
    });
}

function renderCategoryChart(event) {
    const data = JSON.parse(event.detail.xhr.responseText);
    const ctx = document.getElementById('category-chart').getContext('2d');

    if (categoryChart) categoryChart.destroy();

    categoryChart = new Chart(ctx, {
        type: 'doughnut',
        data: {
            labels: data.labels,
            datasets: [{
                data: data.values,
                backgroundColor: [
                    chartColors.primary,
                    chartColors.success,
                    chartColors.warning,
                    chartColors.purple,
                    chartColors.gray
                ]
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    position: 'bottom'
                }
            }
        }
    });
}

// Chart period toggle
document.querySelectorAll('.chart-btn[data-period]').forEach(btn => {
    btn.addEventListener('click', function() {
        document.querySelectorAll('.chart-btn[data-period]').forEach(b => b.classList.remove('active'));
        this.classList.add('active');

        htmx.ajax('GET', `/api/dashboard/chart/revenue?period=${this.dataset.period}`, {
            target: '#revenue-chart',
            swap: 'none'
        });
    });
});

// Update last updated timestamp
function updateTimestamp() {
    document.getElementById('last-update').textContent = new Date().toLocaleString();
}

document.body.addEventListener('htmx:afterRequest', updateTimestamp);
