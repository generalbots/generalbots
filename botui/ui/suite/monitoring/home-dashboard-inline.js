        // Update service status dots based on API response
        document.body.addEventListener('htmx:afterSwap', function(event) {
            if (event.detail.target.id === 'service-status-container') {
                try {
                    const data = JSON.parse(event.detail.target.textContent);
                    Object.entries(data).forEach(([service, status]) => {
                        const dot = document.querySelector(`[data-status="${service}"]`);
                        if (dot) {
                            dot.classList.remove('running', 'warning', 'stopped');
                            dot.classList.add(status);
                        }
                    });
                } catch (e) {
                    console.log('Service status update:', e);
                }
            }
        });

        // Add click handlers for service nodes
        document.querySelectorAll('.service-node').forEach(node => {
            node.addEventListener('click', function() {
                const service = this.dataset.service;
                if (service) {
                    // Navigate to detailed service view
                    htmx.ajax('GET', `/monitoring/service/${service}`, {target: '#main-content'});
                }
            });
        });

        // Keyboard shortcut: R to refresh
        document.addEventListener('keydown', function(e) {
            if (e.key === 'r' && !e.ctrlKey && !e.metaKey) {
                htmx.trigger(document.body, 'htmx:load');
            }
        });
    
