        function createNode(type, x, y) {
            const template = nodeTemplates[type];
            if (!template) {
                console.warn('No template found for node type:', type);
                return null;
            }

            const id = 'node-' + state.nextNodeId++;
            const node = {
                id,
                type,
                x,
                y,
                fields: {}
            };

            // Initialize field values
            template.fields.forEach(field => {
                node.fields[field.name] = field.default;
            });

            state.nodes.set(id, node);
            renderNode(node);
            saveToHistory();
            updateStatusBar();

            return node;
        }

        // Render Node
        function renderNode(node) {
            const template = nodeTemplates[node.type];
            const typeClass = node.type.toLowerCase().replace(/\s+/g, '-');

            let fieldsHtml = '';
            template.fields.forEach(field => {
                const value = node.fields[field.name] || '';
                if (field.type === 'textarea') {
                    fieldsHtml += `
                        <div class="node-field">
                            <label class="node-field-label">${field.label}</label>
                            <textarea class="node-field-input" data-field="${field.name}" rows="2">${value}</textarea>
                        </div>
                    `;
                } else if (field.type === 'select') {
                    const options = field.options.map(opt =>
                        `<option value="${opt}" ${opt === value ? 'selected' : ''}>${opt}</option>`
                    ).join('');
                    fieldsHtml += `
                        <div class="node-field">
                            <label class="node-field-label">${field.label}</label>
                            <select class="node-field-select" data-field="${field.name}">${options}</select>
                        </div>
                    `;
                } else {
                    fieldsHtml += `
                        <div class="node-field">
                            <label class="node-field-label">${field.label}</label>
                            <input type="text" class="node-field-input" data-field="${field.name}" value="${value}">
                        </div>
                    `;
                }
            });

            let portsHtml = '';
            if (template.hasInput) {
                portsHtml += `<div class="node-port input" data-port="input"></div>`;
            }
            if (template.hasOutput) {
                portsHtml += `<div class="node-port output" data-port="output"></div>`;
            }
            if (template.hasOutputTrue) {
                portsHtml += `<div class="node-port output-true" data-port="true" title="True"></div>`;
            }
            if (template.hasOutputFalse) {
                portsHtml += `<div class="node-port output-false" data-port="false" title="False"></div>`;
            }

            const nodeEl = document.createElement('div');
            nodeEl.className = 'node';
            nodeEl.id = node.id;
            nodeEl.style.left = node.x + 'px';
            nodeEl.style.top = node.y + 'px';
            nodeEl.innerHTML = `
                <div class="node-header ${typeClass}">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
                        ${getNodeIcon(node.type)}
                    </svg>
                    ${node.type}
                </div>
                <div class="node-body">
                    ${fieldsHtml}
                </div>
                ${portsHtml}
            `;

            // Make draggable
            nodeEl.addEventListener('mousedown', (e) => {
                if (e.target.classList.contains('node-port')) return;
                selectNode(node.id);
                startNodeDrag(e, node);
            });

            // Field change handlers
            nodeEl.querySelectorAll('.node-field-input, .node-field-select').forEach(input => {
                input.addEventListener('change', (e) => {
                    node.fields[e.target.dataset.field] = e.target.value;
                    saveToHistory();
                });
            });

            // Port handlers for connections
            nodeEl.querySelectorAll('.node-port').forEach(port => {
                port.addEventListener('mousedown', (e) => {
                    e.stopPropagation();
                    startConnection(node.id, port.dataset.port);
                });
                port.addEventListener('mouseup', (e) => {
                    e.stopPropagation();
                    endConnection(node.id, port.dataset.port);
                });
            });

            document.getElementById('canvas-inner').appendChild(nodeEl);
        }

        function getNodeIcon(type) {
            const icons = {
                'TALK': '<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>',
                'HEAR': '<path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><line x1="12" y1="19" x2="12" y2="23"/>',
                'SET': '<polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>',
                'IF': '<path d="M16 3h5v5"/><path d="M8 3H3v5"/><path d="M12 22v-8.3a4 4 0 0 0-1.172-2.872L3 3"/><path d="m15 9 6-6"/>',
                'FOR': '<polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>',
                'SWITCH': '<path d="M18 20V10"/><path d="M12 20V4"/><path d="M6 20v-6"/>',
                'CALL': '<path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.362 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.338 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"/>',
                'SEND MAIL': '<path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/>',
                'GET': '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>',
                'POST': '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/>',
                'SAVE': '<path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/>',
                'WAIT': '<circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>',
                'SET BOT MEMORY': '<path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><line x1="12" y1="2" x2="12" y2="6"/><line x1="12" y1="18" x2="12" y2="22"/>',
                'GET BOT MEMORY': '<path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="7 10 12 15 17 10"/>',
                'SET USER MEMORY': '<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>',
                'GET USER MEMORY': '<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>'
            };
            return icons[type] || '<circle cx="12" cy="12" r="10"/>';
        }

        // Node dragging
        function startNodeDrag(e, node) {
            state.isDragging = true;
            const nodeEl = document.getElementById(node.id);
            const startX = e.clientX;
            const startY = e.clientY;
            const origX = node.x;
            const origY = node.y;

            function onMove(e) {
                const dx = (e.clientX - startX) / state.zoom;
                const dy = (e.clientY - startY) / state.zoom;
                node.x = snapToGrid(origX + dx);
                node.y = snapToGrid(origY + dy);
                nodeEl.style.left = node.x + 'px';
                nodeEl.style.top = node.y + 'px';
                updateConnections();
            }

            function onUp() {
                state.isDragging = false;
                document.removeEventListener('mousemove', onMove);
                document.removeEventListener('mouseup', onUp);
                saveToHistory();
            }

            document.addEventListener('mousemove', onMove);
            document.addEventListener('mouseup', onUp);
        }

        // Node selection
        function selectNode(id) {
            if (state.selectedNode) {
                const prevEl = document.getElementById(state.selectedNode);
                if (prevEl) prevEl.classList.remove('selected');
            }
            state.selectedNode = id;
            const nodeEl = document.getElementById(id);
            if (nodeEl) nodeEl.classList.add('selected');
            updatePropertiesPanel();
        }

        function deselectAll() {
            if (state.selectedNode) {
                const el = document.getElementById(state.selectedNode);
                if (el) el.classList.remove('selected');
            }
            state.selectedNode = null;
            state.selectedConnection = null;
            updatePropertiesPanel();
        }

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
            const empty = document.getElementById('properties-empty');

            if (!state.selectedNode) {
                content.style.display = 'none';
                empty.style.display = 'block';
                return;
            }

            const node = state.nodes.get(state.selectedNode);
            if (!node) return;

            const template = nodeTemplates[node.type];
            empty.style.display = 'none';
            content.style.display = 'block';

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
                            <textarea class="property-textarea" data-node="${node.id}" data-field="${field.name}">${value}</textarea>
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
            content.innerHTML = html;

            // Add change handlers
            content.querySelectorAll('.property-input:not([readonly]), .property-textarea').forEach(input => {
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

        // History (Undo/Redo)
        function saveToHistory() {
            const snapshot = {
                nodes: Array.from(state.nodes.entries()).map(([id, node]) => ({...node})),
                connections: [...state.connections]
            };
            state.history = state.history.slice(0, state.historyIndex + 1);
            state.history.push(snapshot);
            state.historyIndex = state.history.length - 1;
        }

        function undo() {
            if (state.historyIndex > 0) {
                state.historyIndex--;
                restoreSnapshot(state.history[state.historyIndex]);
            }
        }
        window.undo = undo;

        function redo() {
            if (state.historyIndex < state.history.length - 1) {
                state.historyIndex++;
                restoreSnapshot(state.history[state.historyIndex]);
            }
        }
        window.redo = redo;

        function restoreSnapshot(snapshot) {
            // Clear canvas
            document.getElementById('canvas-inner').innerHTML = '';
            state.nodes.clear();
            state.connections = [];

            // Restore nodes
            snapshot.nodes.forEach(node => {
                state.nodes.set(node.id, {...node});
                renderNode(node);
            });

            // Restore connections
            state.connections = [...snapshot.connections];
            updateConnections();
            updateStatusBar();
        }

        // Keyboard Shortcuts
        function initKeyboardShortcuts() {
            document.addEventListener('keydown', (e) => {
                if (e.ctrlKey || e.metaKey) {
                    switch (e.key) {
                        case 's':
                            e.preventDefault();
                            saveDesign();
                            break;
                        case 'o':
                            e.preventDefault();
                            showModal('open-modal');
                            break;
                        case 'z':
                            e.preventDefault();
                            if (e.shiftKey) redo();
                            else undo();
                            break;
                        case 'y':
                            e.preventDefault();
                            redo();
                            break;
                        case 'c':
                            if (state.selectedNode) {
                                e.preventDefault();
                                state.clipboard = {...state.nodes.get(state.selectedNode)};
                            }
                            break;
                        case 'v':
                            if (state.clipboard) {
                                e.preventDefault();
                                const newNode = {...state.clipboard};
                                newNode.id = 'node-' + state.nextNodeId++;
                                newNode.x += 40;
                                newNode.y += 40;
                                state.nodes.set(newNode.id, newNode);
                                renderNode(newNode);
                                selectNode(newNode.id);
                                saveToHistory();
                            }
                            break;
                    }
                }

                if (e.key === 'Delete' && state.selectedNode) {
                    deleteSelectedNode();
                }

                if (e.key === 'Escape') {
                    deselectAll();
                    hideContextMenu();
                }
            });
        }
