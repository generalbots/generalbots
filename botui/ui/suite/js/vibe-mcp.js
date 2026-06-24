const vibeMcp = {
    init: function() {
        this.refreshServers();
    },

    refreshServers: async function() {
        try {
            const listContainer = document.getElementById('mcpServerList');
            if (listContainer) {
                listContainer.innerHTML = '<div class="mcp-loading">Loading servers...</div>';
            }

            const response = await fetch('/api/ui/sources/mcp');
            if (!response.ok) throw new Error('Failed to fetch MCP servers');
            
            const data = await response.json();
            // Expected data structure: { servers: [...] }
            this.renderServers(data.servers || []);
        } catch (err) {
            console.error(err);
            const listContainer = document.getElementById('mcpServerList');
            if (listContainer) {
                listContainer.innerHTML = `<div class="mcp-error">Error loading servers: ${err.message}</div>`;
            }
        }
    },

    toggleServer: async function(name, enable) {
        try {
            // Assume an endpoint to toggle enable/disable
            const response = await fetch(`/api/ui/sources/mcp/${name}/enable`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ enable: enable })
            });
            if (response.ok) {
                this.refreshServers();
            } else {
                alert('Failed to toggle server');
            }
        } catch (e) {
            alert('Error toggling server');
        }
    },

    renderServers: function(servers) {
        const listContainer = document.getElementById('mcpServerList');
        if (!listContainer) return;

        if (servers.length === 0) {
            listContainer.innerHTML = '<div class="mcp-empty">No MCP servers connected.</div>';
            return;
        }

        let html = '';
        servers.forEach(server => {
            const isActive = server.enabled || server.active;
            const statusClass = isActive ? 'status-active' : 'status-inactive';
            
            html += `
                <div class="mcp-server-card">
                    <div class="mcp-server-header">
                        <span class="mcp-status-dot ${statusClass}"></span>
                        <div class="mcp-server-info">
                            <h4>${server.name}</h4>
                            <span class="mcp-server-cmd">${server.command}</span>
                        </div>
                        <label class="mcp-toggle-switch">
                            <input type="checkbox" ${isActive ? 'checked' : ''} 
                                onchange="vibeMcp.toggleServer('${server.name}', this.checked)">
                            <span class="mcp-slider round"></span>
                        </label>
                    </div>
                    
                    <div class="mcp-tools-list">
                        ${this.renderTools(server.tools || [])}
                    </div>
                </div>
            `;
        });
        
        listContainer.innerHTML = html;
    },

    renderTools: function(tools) {
        if (!tools || tools.length === 0) return '<div class="mcp-no-tools">No tools available</div>';
        
        // Show up to 3 tools
        const displayTools = tools.slice(0, 3);
        let toolsHtml = displayTools.map(t => `<div class="mcp-tool-pill">${t.name}</div>`).join('');
        
        if (tools.length > 3) {
            toolsHtml += `<div class="mcp-tool-more">+${tools.length - 3} more</div>`;
        }
        return toolsHtml;
    }
};

// Initialize when the script loads
document.addEventListener('DOMContentLoaded', () => {
    vibeMcp.init();
});

// Since HTMX loads this dynamically, also invoke init right away if DOM is already ready
if (document.readyState === 'complete' || document.readyState === 'interactive') {
    vibeMcp.init();
}
