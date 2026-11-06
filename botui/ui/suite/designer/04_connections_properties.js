/* Dialog Designer — split from suite/designer.js for #1731 */

// Connections
function startConnection(nodeId, portType) {
    state.isConnecting = true;
    state.connectionStart = { nodeId, portType };
}

function endConnection(nodeId, portType) {
    if (!state.isConnecting || !state.connectionStart) return;
    if (state.connectionStart.nodeId === nodeId) {
        state.isConnecting = false;
        state.connectionStart = null;
        return;
    }

    // Only allow output to input connections
    if (state.connectionStart.portType === 'input' || portType !== 'input') {
        state.isConnecting = false;
        state.connectionStart = null;
        return;
    }

    const connection = {
        from: state.connectionStart.nodeId,
        fromPort: state.connectionStart.portType,
        to: nodeId,
        toPort: portType
    };

    state.connections.push(connection);
    state.isConnecting = false;
    state.connectionStart = null;
    updateConnections();
    saveToHistory();
}

function updateConnections() {
    const svg = document.getElementById('connections-svg');
    if (!svg) return;
    
    let paths = '';

    state.connections.forEach((conn, index) => {
        const fromEl = document.getElementById(conn.from);
        const toEl = document.getElementById(conn.to);
        if (!fromEl || !toEl) return;

        const fromPort = fromEl.querySelector(`[data-port="${conn.fromPort}"]`);
        const toPort = toEl.querySelector(`[data-port="${conn.toPort}"]`);
        if (!fromPort || !toPort) return;

        const fromRect = fromPort.getBoundingClientRect();
        const toRect = toPort.getBoundingClientRect();
        const canvasRect = document.getElementById('canvas-inner').getBoundingClientRect();

        const x1 = (fromRect.left + fromRect.width / 2 - canvasRect.left) / state.zoom;
        const y1 = (fromRect.top + fromRect.height / 2 - canvasRect.top) / state.zoom;
        const x2 = (toRect.left + toRect.width / 2 - canvasRect.left) / state.zoom;
        const y2 = (toRect.top + toRect.height / 2 - canvasRect.top) / state.zoom;

        const midX = (x1 + x2) / 2;
        const d = `M ${x1} ${y1} C ${midX} ${y1}, ${midX} ${y2}, ${x2} ${y2}`;

        paths += `<path class="connection" d="${d}" data-index="${index}"/>`;
    });

    svg.innerHTML = `
        <defs>
            <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                <polygon points="0 0, 10 3.5, 0 7" fill="var(--primary)"/>
            </marker>
        </defs>
        ${paths}
    `;
}

// Properties Panel
function updatePropertiesPanel() {
    const content = document.getElementById('properties-content');
    const empty = document.querySelector('.properties-empty');

    if (!state.selectedNode) {
        if (content) content.style.display = 'none';
        if (empty) empty.style.display = 'block';
        return;
    }

    const node = state.nodes.get(state.selectedNode);
    if (!node) return;

    const template = nodeTemplates[node.type];
    if (empty) empty.style.display = 'none';
    if (content) content.style.display = 'block';

    let html = `
        <div class="property-group">
            <div class="property-group-title">Node Info</div>
            <div class="property-field">
                <label class="property-label">Type</label>
                <input type="text" class="property-input" value="${node.type}" readonly>
            </div>
            <div class="property-field">
                <label class="property-label">ID</label>
                <input type="text" class="property-input" value="${node.id}" readonly>
            </div>
        </div>
        <div class="property-group">
            <div class="property-group-title">Properties</div>
    `;

    template.fields.forEach(field => {
        const value = node.fields[field.name] || '';
        if (field.type === 'textarea') {
            html += `
                <div class="property-field">
                    <label class="property-label">${field.label}</label>
                    <textarea class="property-textarea property-input" data-node="${node.id}" data-field="${field.name}">${value}</textarea>
                </div>
            `;
        } else {
            html += `
                <div class="property-field">
                    <label class="property-label">${field.label}</label>
                    <input type="text" class="property-input" data-node="${node.id}" data-field="${field.name}" value="${value}">
                </div>
            `;
        }
    });

    html += '</div>';
    if (content) content.innerHTML = html;

    // Add change handlers
    if (content) {
        content.querySelectorAll('.property-input:not([readonly])').forEach(input => {
            input.addEventListener('change', (e) => {
                const n = state.nodes.get(e.target.dataset.node);
                if (n) {
                    n.fields[e.target.dataset.field] = e.target.value;
                    // Update node view
                    const nodeInput = document.querySelector(`#${n.id} [data-field="${e.target.dataset.field}"]`);
                    if (nodeInput) nodeInput.value = e.target.value;
                    saveToHistory();
                }
            });
        });
    }
}
