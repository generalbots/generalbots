const vibeMcp = {
    init: function() {
        this.refreshServers();
    },

    refreshServers: async function() {
        const listContainer = document.getElementById('mcpServerList');
        if (listContainer) {
            listContainer.innerHTML = '<div class="mcp-loading">Loading servers...</div>';
        }

        try {
            const headers = { 'Content-Type': 'application/json' };
            const token = localStorage.getItem('gb-access-token') || sessionStorage.getItem('gb-access-token') || '';
            if (token) headers['Authorization'] = 'Bearer ' + token;
            const response = await fetch('/api/sources/mcp', { headers });
            if (!response.ok) throw new Error('Failed to fetch MCP servers (HTTP ' + response.status + ')');
            const contentType = response.headers.get('content-type') || '';
            if (!contentType.includes('application/json')) {
                throw new Error('MCP endpoint returned non-JSON response');
            }

            const data = await response.json();
            if (!data || !data.success) {
                throw new Error((data && data.error) || 'MCP endpoint reported failure');
            }

            this.renderServers((data.data || []).map(s => ({
                name: s.name,
                command: (s.server_type || '') + (s.description ? ' — ' + s.description : ''),
                enabled: s.enabled === true,
                active: s.status === 'enabled' || s.status === 'ready',
                tools: (s.tools_count !== undefined ? [{ name: s.tools_count + ' tool(s)' }] : [])
            })));
        } catch (err) {
            console.error('MCP servers failed to load:', err);
            if (listContainer) {
                listContainer.innerHTML = `<div class="mcp-error">Error loading servers: ${this._esc(err.message)}</div>`;
            }
        }
    },

    toggleServer: async function(name, enable) {
        try {
            const response = await fetch(`/api/sources/mcp/${encodeURIComponent(name)}/${enable ? 'enable' : 'disable'}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' }
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
    },

    _esc: function (str) {
        if (str === null || str === undefined) return '';
        const div = document.createElement('div');
        div.textContent = String(str);
        return div.innerHTML;
    }
};

(function(){ var __cb = () => {
    vibeMcp.init();
}; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
