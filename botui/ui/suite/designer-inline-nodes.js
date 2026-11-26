        function createNode(type, x, y) {
            const template = GBDesignerTemplates[type];
            if (!template) { console.warn('No template found for node type:', type); return null; }
            const id = 'node-' + GBDesignerState.nextNodeId++;
            const node = { id, type, x, y, fields: {} };
            template.fields.forEach(field => { node.fields[field.name] = field.default; });
            GBDesignerState.nodes.set(id, node);
            renderNode(node);
            saveToHistory();
            updateStatusBar();
            return node;
        }

        // Render Node
        function renderNode(node) {
            const template = GBDesignerTemplates[node.type];
            const typeClass = node.type.toLowerCase().replace(/\s+/g, '-');

            let fieldsHtml = '';
            template.fields.forEach(field => {
                const value = node.fields[field.name] || '';
                let control;
                if (field.type === 'textarea') control = `<textarea class="node-field-input" data-field="${field.name}" rows="2">${value}</textarea>`;
                else if (field.type === 'select') control = `<select class="node-field-select" data-field="${field.name}">${field.options.map(opt => `<option value="${opt}" ${opt === value ? 'selected' : ''}>${opt}</option>`).join('')}</select>`;
                else control = `<input type="text" class="node-field-input" data-field="${field.name}" value="${value}">`;
                fieldsHtml += `<div class="node-field"><label class="node-field-label">${field.label}</label>${control}</div>`;
            });

            let portsHtml = '';
            if (template.hasInput) portsHtml += '<div class="node-port input" data-port="input"></div>';
            if (template.hasOutput) portsHtml += '<div class="node-port output" data-port="output"></div>';
            if (template.hasOutputTrue) portsHtml += '<div class="node-port output-true" data-port="true" title="True"></div>';
            if (template.hasOutputFalse) portsHtml += '<div class="node-port output-false" data-port="false" title="False"></div>';

            const coarse = window.gbGestures && window.gbGestures.isCoarsePointer();
            const overflowBtn = coarse
                ? '<button type="button" class="node-more" aria-label="' + (window.gbT ? window.gbT('label-more', 'More') : 'More') + '" style="margin-left:auto;border:0;background:transparent;color:inherit;cursor:pointer;min-width:44px;min-height:32px;">&#8943;</button>'
                : '';
            const nodeEl = document.createElement('div');
            nodeEl.className = 'node';
            nodeEl.id = node.id;
            nodeEl.style.left = node.x + 'px';
            nodeEl.style.top = node.y + 'px';
            nodeEl.innerHTML = `<div class="node-header ${typeClass}"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">${getNodeIcon(node.type)}</svg>${node.type}${overflowBtn}</div><div class="node-body">${fieldsHtml}</div>${portsHtml}`;

            // Pointer events with capture. Mouse drags immediately (desktop
            // behaviour unchanged); touch nodes wait for a long press so a
            // finger moving over the canvas pans instead of moving a node.
            nodeEl.style.touchAction = 'none';
            nodeEl.addEventListener('pointerdown', (e) => {
                if (e.target.classList.contains('node-port') || !e.isPrimary) return;
                if (e.pointerType !== 'mouse') return;
                selectNode(node.id);
                startNodeDrag(e, node, nodeEl);
            });
            if (window.gbGestures) {
                window.gbGestures.onLongPress(nodeEl, function (e, start) {
                    selectNode(node.id);
                    startNodeDrag({ pointerId: start.id, clientX: start.x, clientY: start.y }, node, nodeEl);
                }, { includeMouse: false, tolerance: 12 });
                const moreBtn = nodeEl.querySelector('.node-more');
                if (moreBtn) {
                    moreBtn.addEventListener('click', function (e) {
                        e.stopPropagation();
                        selectNode(node.id);
                        const menu = document.getElementById('context-menu');
                        if (menu) {
                            const rect = moreBtn.getBoundingClientRect();
                            menu.style.left = rect.left + 'px';
                            menu.style.top = (rect.bottom + 4) + 'px';
                            menu.classList.add('visible');
                            if (window.gbGestures) window.gbGestures.clampToViewport(menu, rect.left, rect.bottom + 4);
                        }
                    });
                }
            }

            // Field change handlers
            nodeEl.querySelectorAll('.node-field-input, .node-field-select').forEach(input => {
                input.addEventListener('change', (e) => {
                    node.fields[e.target.dataset.field] = e.target.value;
                    saveToHistory();
                });
            });

            // Port handlers for connections. Capture keeps pointerup on the
            // source port; elementFromPoint resolves the actual target port.
            nodeEl.querySelectorAll('.node-port').forEach(port => {
                port.style.touchAction = 'none';
                let activePortPointer = null;
                port.addEventListener('pointerdown', (e) => {
                    if (!e.isPrimary) return;
                    e.stopPropagation();
                    activePortPointer = e.pointerId;
                    gbCapturePointer(port, e.pointerId);
                    startConnection(node.id, port.dataset.port);
                });
                port.addEventListener('pointerup', (e) => {
                    if (activePortPointer !== e.pointerId) return;
                    e.stopPropagation();
                    activePortPointer = null;
                    gbReleasePointer(port, e.pointerId);
                    const under = document.elementFromPoint(e.clientX, e.clientY);
                    const target = under && under.closest ? under.closest('.node-port') : null;
                    if (target) endConnection(target.closest('.node').id, target.dataset.port);
                    else { GBDesignerState.isConnecting = false; GBDesignerState.connectionStart = null; }
                });
                port.addEventListener('pointercancel', (e) => {
                    if (activePortPointer !== e.pointerId) return;
                    activePortPointer = null;
                    GBDesignerState.isConnecting = false;
                    GBDesignerState.connectionStart = null;
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
                'SAVE': '<path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/>', 'WAIT': '<circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>', 'SET BOT MEMORY': '<path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><line x1="12" y1="2" x2="12" y2="6"/><line x1="12" y1="18" x2="12" y2="22"/>',
                'GET BOT MEMORY': '<path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="7 10 12 15 17 10"/>',
                'SET USER MEMORY': '<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>',
                'GET USER MEMORY': '<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>'
            };
            return icons[type] || '<circle cx="12" cy="12" r="10"/>';
        }

        // Node dragging — pointer capture keeps the gesture alive outside the
        // node and works with both mouse and touch (#2419).
        function startNodeDrag(e, node, el) {
            const nodeEl = el || document.getElementById(node.id);
            if (!nodeEl) return;
            GBDesignerState.isDragging = true;
            const pointerId = e.pointerId;
            const startX = e.clientX;
            const startY = e.clientY;
            const origX = node.x;
            const origY = node.y;
            gbCapturePointer(nodeEl, pointerId);

            function onMove(ev) {
                if (ev.pointerId !== pointerId) return;
                node.x = snapToGrid(origX + (ev.clientX - startX) / GBDesignerState.zoom);
                node.y = snapToGrid(origY + (ev.clientY - startY) / GBDesignerState.zoom);
                nodeEl.style.left = node.x + 'px';
                nodeEl.style.top = node.y + 'px';
                updateConnections();
            }

            function onUp(ev) {
                if (ev.pointerId !== pointerId) return;
                GBDesignerState.isDragging = false;
                nodeEl.removeEventListener('pointermove', onMove);
                nodeEl.removeEventListener('pointerup', onUp);
                nodeEl.removeEventListener('pointercancel', onUp);
                gbReleasePointer(nodeEl, pointerId);
                saveToHistory();
            }

            nodeEl.addEventListener('pointermove', onMove);
            nodeEl.addEventListener('pointerup', onUp);
            nodeEl.addEventListener('pointercancel', onUp);
        }

        // Node selection
        function selectNode(id) {
            if (GBDesignerState.selectedNode) {
                const prevEl = document.getElementById(GBDesignerState.selectedNode);
                if (prevEl) prevEl.classList.remove('selected');
            }
            GBDesignerState.selectedNode = id;
            const nodeEl = document.getElementById(id);
            if (nodeEl) nodeEl.classList.add('selected');
            updatePropertiesPanel();
        }

        function deselectAll() {
            if (GBDesignerState.selectedNode) {
                const el = document.getElementById(GBDesignerState.selectedNode);
                if (el) el.classList.remove('selected');
            }
            GBDesignerState.selectedNode = null;
            GBDesignerState.selectedConnection = null;
            updatePropertiesPanel();
        }

        // Connections
        function startConnection(nodeId, portType) {
            GBDesignerState.isConnecting = true;
            GBDesignerState.connectionStart = { nodeId, portType };
        }

        function endConnection(nodeId, portType) {
            if (!GBDesignerState.isConnecting || !GBDesignerState.connectionStart) return;
            if (GBDesignerState.connectionStart.nodeId === nodeId) {
                GBDesignerState.isConnecting = false;
                GBDesignerState.connectionStart = null;
                return;
            }

            // Only allow output to input connections
            if (GBDesignerState.connectionStart.portType === 'input' || portType !== 'input') {
                GBDesignerState.isConnecting = false;
                GBDesignerState.connectionStart = null;
                return;
            }

            const connection = {
                from: GBDesignerState.connectionStart.nodeId,
                fromPort: GBDesignerState.connectionStart.portType,
                to: nodeId,
                toPort: portType
            };

            GBDesignerState.connections.push(connection);
            GBDesignerState.isConnecting = false;
            GBDesignerState.connectionStart = null;
            updateConnections();
            saveToHistory();
        }

        function updateConnections() {
            const svg = document.getElementById('connections-svg');
            let paths = '';

            GBDesignerState.connections.forEach((conn, index) => {
                const fromEl = document.getElementById(conn.from);
                const toEl = document.getElementById(conn.to);
                if (!fromEl || !toEl) return;

                const fromPort = fromEl.querySelector(`[data-port="${conn.fromPort}"]`);
                const toPort = toEl.querySelector(`[data-port="${conn.toPort}"]`);
                if (!fromPort || !toPort) return;

                const fromRect = fromPort.getBoundingClientRect();
                const toRect = toPort.getBoundingClientRect();
                const canvasRect = document.getElementById('canvas-inner').getBoundingClientRect();

                const x1 = (fromRect.left + fromRect.width / 2 - canvasRect.left) / GBDesignerState.zoom;
                const y1 = (fromRect.top + fromRect.height / 2 - canvasRect.top) / GBDesignerState.zoom;
                const x2 = (toRect.left + toRect.width / 2 - canvasRect.left) / GBDesignerState.zoom;
                const y2 = (toRect.top + toRect.height / 2 - canvasRect.top) / GBDesignerState.zoom;

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

            if (!GBDesignerState.selectedNode) {
                content.style.display = 'none';
                empty.style.display = 'block';
                return;
            }

            const node = GBDesignerState.nodes.get(GBDesignerState.selectedNode);
            if (!node) return;

            const template = GBDesignerTemplates[node.type];
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
                    const n = GBDesignerState.nodes.get(e.target.dataset.node);
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
                nodes: Array.from(GBDesignerState.nodes.entries()).map(([id, node]) => ({...node})),
                connections: [...GBDesignerState.connections]
            };
            GBDesignerState.history = GBDesignerState.history.slice(0, GBDesignerState.historyIndex + 1);
            GBDesignerState.history.push(snapshot);
            GBDesignerState.historyIndex = GBDesignerState.history.length - 1;
        }

        function undo() {
            if (GBDesignerState.historyIndex > 0) {
                GBDesignerState.historyIndex--;
                restoreSnapshot(GBDesignerState.history[GBDesignerState.historyIndex]);
            }
        }
        window.undo = undo;

        function redo() {
            if (GBDesignerState.historyIndex < GBDesignerState.history.length - 1) {
                GBDesignerState.historyIndex++;
                restoreSnapshot(GBDesignerState.history[GBDesignerState.historyIndex]);
            }
        }
        window.redo = redo;

        function restoreSnapshot(snapshot) {
            // Clear canvas
            document.getElementById('canvas-inner').innerHTML = '';
            GBDesignerState.nodes.clear();
            GBDesignerState.connections = [];

            // Restore nodes
            snapshot.nodes.forEach(node => {
                GBDesignerState.nodes.set(node.id, {...node});
                renderNode(node);
            });

            // Restore connections
            GBDesignerState.connections = [...snapshot.connections];
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
                            if (GBDesignerState.selectedNode) {
                                e.preventDefault();
                                GBDesignerState.clipboard = {...GBDesignerState.nodes.get(GBDesignerState.selectedNode)};
                            }
                            break;
                        case 'v':
                            if (GBDesignerState.clipboard) {
                                e.preventDefault();
                                const newNode = {...GBDesignerState.clipboard};
                                newNode.id = 'node-' + GBDesignerState.nextNodeId++;
                                newNode.x += 40;
                                newNode.y += 40;
                                GBDesignerState.nodes.set(newNode.id, newNode);
                                renderNode(newNode);
                                selectNode(newNode.id);
                                saveToHistory();
                            }
                            break;
                    }
                }

                if (e.key === 'Delete' && GBDesignerState.selectedNode) {
                    deleteSelectedNode();
                }

                if (e.key === 'Escape') {
                    deselectAll();
                    hideContextMenu();
                }
            });
        }
