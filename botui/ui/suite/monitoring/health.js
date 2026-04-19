/* Health page JavaScript */

function refreshAllChecks() {
    htmx.trigger('#health-checks', 'refresh');
    htmx.trigger('#dependencies', 'refresh');
    htmx.trigger('.health-overview', 'refresh');

    // Visual feedback
    const btn = event.currentTarget;
    const originalText = btn.innerHTML;
    btn.innerHTML = `
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spin">
            <polyline points="23 4 23 10 17 10"></polyline>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
        </svg>
        Refreshing...
    `;
    btn.disabled = true;

    setTimeout(() => {
        btn.innerHTML = originalText;
        btn.disabled = false;
    }, 1000);
}

// Add tooltip functionality for uptime bars
function initUptimeTooltips() {
    document.querySelectorAll('.uptime-bar').forEach(bar => {
        bar.addEventListener('mouseenter', function(e) {
            const tooltip = document.createElement('div');
            tooltip.className = 'uptime-tooltip';
            tooltip.textContent = this.title;
            tooltip.style.cssText = `
                position: fixed;
                background: var(--surface);
                border: 1px solid var(--border);
                padding: 0.375rem 0.625rem;
                border-radius: 4px;
                font-size: 0.75rem;
                color: var(--text);
                pointer-events: none;
                z-index: 1000;
                box-shadow: 0 4px 12px rgba(0,0,0,0.15);
            `;
            document.body.appendChild(tooltip);

            const rect = this.getBoundingClientRect();
            tooltip.style.left = `${rect.left + rect.width/2 - tooltip.offsetWidth/2}px`;
            tooltip.style.top = `${rect.top - tooltip.offsetHeight - 8}px`;

            this._tooltip = tooltip;
        });

        bar.addEventListener('mouseleave', function() {
            if (this._tooltip) {
                this._tooltip.remove();
                this._tooltip = null;
            }
        });
    });
}

// Initialize tooltips after HTMX content loads
document.body.addEventListener('htmx:afterSwap', function(evt) {
    if (evt.target.id === 'uptime-chart') {
        initUptimeTooltips();
    }
});

// Initialize on page load
document.addEventListener('DOMContentLoaded', function() {
    initUptimeTooltips();
});
