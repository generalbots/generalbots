
        function deleteSelectedNode() {
            if (!state.selectedNode) return;
            const nodeEl = document.getElementById(state.selectedNode);
            if (nodeEl) nodeEl.remove();

            // Remove connections
            state.connections = state.connections.filter(
                conn => conn.from !== state.selectedNode && conn.to !== state.selectedNode
            );

            state.nodes.delete(state.selectedNode);
            state.selectedNode = null;
            updateConnections();
            updatePropertiesPanel();
            updateStatusBar();
            saveToHistory();
        }

        // Context Menu
        function initContextMenu() {
            const canvas = document.getElementById('canvas');
            const contextMenu = document.getElementById('context-menu');

            if (!canvas || !contextMenu) {
                console.warn('initContextMenu: canvas or context-menu not found');
                return;
            }

            canvas.addEventListener('contextmenu', (e) => {
                e.preventDefault();
                const nodeEl = e.target.closest('.node');
                if (nodeEl) {
                    selectNode(nodeEl.id);
                }
                contextMenu.style.left = e.clientX + 'px';
                contextMenu.style.top = e.clientY + 'px';
                contextMenu.classList.add('visible');
            });

            document.addEventListener('click', () => {
                hideContextMenu();
            });
        }

        function hideContextMenu() {
            const menu = document.getElementById('context-menu');
            if (menu) {
                menu.classList.remove('visible');
            }
        }

        // Context Menu Actions
        function duplicateNode() {
            if (!state.selectedNode) return;
            const node = state.nodes.get(state.selectedNode);
            if (!node) return;

            const newNode = {...node, fields: {...node.fields}};
            newNode.id = 'node-' + state.nextNodeId++;
            newNode.x += 40;
            newNode.y += 40;
            state.nodes.set(newNode.id, newNode);
            renderNode(newNode);
            selectNode(newNode.id);
            saveToHistory();
            hideContextMenu();
        }

        // Status Bar
        function updateStatusBar() {
            document.getElementById('node-count').textContent = state.nodes.size + ' nodes';
            document.getElementById('connection-count').textContent = state.connections.length + ' connections';
        }

        // Zoom Controls
        function zoomIn() {
            state.zoom = Math.min(state.zoom + 0.1, 2);
            updateCanvasTransform();
            updateZoomDisplay();
        }

        function zoomOut() {
            state.zoom = Math.max(state.zoom - 0.1, 0.25);
            updateCanvasTransform();
            updateZoomDisplay();
        }

        // Modal Management
        function showModal(id) {
            const modal = document.getElementById(id);
            if (modal) {
                modal.classList.add('visible');
                if (id === 'open-modal') {
                    htmx.trigger('#file-list-content', 'load');
                }
            }
        }

        function hideModal(id) {
            const modal = document.getElementById(id);
            if (modal) {
                modal.classList.remove('visible');
            }
        }

        // Save Design
        function saveDesign() {
            const nodesData = Array.from(state.nodes.values());
            document.getElementById('nodes-data').value = JSON.stringify(nodesData);
            document.getElementById('connections-data').value = JSON.stringify(state.connections);

            if (state.driveSource) {
                saveToDrive();
            } else {
                htmx.ajax('POST', '/api/designer/save', {
                    source: document.getElementById('designer-data'),
                    target: '#status-message'
                });
            }
        }

        // Save to drive (MinIO) when file was loaded from drive
        async function saveToDrive() {
            const basCode = generateBasCode();
            const { bucket, path } = state.driveSource;

            try {
                const response = await fetch('/api/files/write', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ bucket, path, content: basCode })
                });

                if (response.ok) {
                    const statusEl = document.querySelector('.status-item span');
                    if (statusEl) {
                        statusEl.textContent = `Saved: ${path.split('/').pop()}`;
                    }
                } else {
                    const err = await response.json();
                    alert(`Save failed: ${err.error || 'Unknown error'}`);
                }
            } catch (e) {
                alert(`Save failed: ${e.message}`);
            }
        }

        // Generate BASIC code from nodes
        function generateBasCode() {
            let basCode = "' Generated by General Bots Designer\n";
            basCode += "' " + new Date().toISOString() + "\n\n";

            const sortedNodes = Array.from(state.nodes.values()).sort((a, b) => {
                if (Math.abs(a.y - b.y) < 30) return a.x - b.x;
                return a.y - b.y;
            });

            sortedNodes.forEach(node => {
                switch (node.type) {
                    case 'TALK':
                        basCode += `TALK "${node.fields.message || ''}"\n`;
                        break;
                    case 'HEAR':
                        basCode += `HEAR ${node.fields.variable || 'input'} AS ${node.fields.type || 'string'}\n`;
                        break;
                    case 'SET':
                        basCode += `SET ${node.fields.variable || 'x'} = ${node.fields.expression || '0'}\n`;
                        break;
                    case 'IF':
                        basCode += `IF ${node.fields.condition || 'true'} THEN\n`;
                        break;
                    case 'FOR':
                        basCode += `FOR EACH ${node.fields.variable || 'item'} IN ${node.fields.collection || 'items'}\n`;
                        break;
                    case 'CALL':
                        basCode += `CALL ${node.fields.procedure || 'sub'}(${node.fields.arguments || ''})\n`;
                        break;
                    case 'SEND MAIL':
                        basCode += `SEND MAIL TO "${node.fields.to || ''}" SUBJECT "${node.fields.subject || ''}" BODY "${node.fields.body || ''}"\n`;
                        break;
                    case 'GET':
                        basCode += `GET ${node.fields.url || 'url'} TO ${node.fields.variable || 'result'}\n`;
                        break;
                    case 'POST':
                        basCode += `POST ${node.fields.url || 'url'} WITH ${node.fields.body || '{}'} TO ${node.fields.variable || 'result'}\n`;
                        break;
                    case 'SAVE':
                        basCode += `SAVE ${node.fields.data || 'data'} TO "${node.fields.filename || 'file.txt'}"\n`;
                        break;
                    case 'WAIT':
                        basCode += `WAIT ${node.fields.duration || '1000'}\n`;
                        break;
                    case 'SET BOT MEMORY':
                        basCode += `SET BOT MEMORY "${node.fields.key || 'key'}", ${node.fields.value || '""'}\n`;
                        break;
                    case 'GET BOT MEMORY':
                        basCode += `GET BOT MEMORY "${node.fields.key || 'key'}" TO ${node.fields.variable || 'value'}\n`;
                        break;
                    case 'SET USER MEMORY':
                        basCode += `SET USER MEMORY "${node.fields.key || 'key'}", ${node.fields.value || '""'}\n`;
                        break;
                    case 'GET USER MEMORY':
                        basCode += `GET USER MEMORY "${node.fields.key || 'key'}" TO ${node.fields.variable || 'value'}\n`;
                        break;
                    case 'SWITCH':
                        basCode += `SWITCH ${node.fields.expression || 'value'}\n`;
                        break;
                }
            });

            return basCode;
        }

        // Export to .bas
        function exportToBas() {
            let basCode = "' Generated by General Bots Designer\n";
            basCode += "' " + new Date().toISOString() + "\n\n";

            // Sort nodes by position (top to bottom, left to right)
            const sortedNodes = Array.from(state.nodes.values()).sort((a, b) => {
                if (Math.abs(a.y - b.y) < 30) return a.x - b.x;
                return a.y - b.y;
            });

            sortedNodes.forEach(node => {
                const template = nodeTemplates[node.type];
                switch (node.type) {
                    case 'TALK':
                        basCode += `TALK "${node.fields.message}"\n`;
                        break;
                    case 'HEAR':
                        basCode += `HEAR ${node.fields.variable} AS ${node.fields.type}\n`;
                        break;
                    case 'SET':
                        basCode += `SET ${node.fields.variable} = ${node.fields.expression}\n`;
                        break;
                    case 'IF':
                        basCode += `IF ${node.fields.condition} THEN\n`;
                        break;
                    case 'FOR':
                        basCode += `FOR EACH ${node.fields.variable} IN ${node.fields.collection}\n`;
                        break;
                    case 'CALL':
                        basCode += `CALL ${node.fields.procedure}(${node.fields.arguments})\n`;
                        break;
                    case 'SEND MAIL':
                        basCode += `SEND MAIL TO "${node.fields.to}" SUBJECT "${node.fields.subject}" BODY "${node.fields.body}"\n`;
                        break;
                    case 'GET':
                        basCode += `GET ${node.fields.url} TO ${node.fields.variable}\n`;
                        break;
                    case 'POST':
                        basCode += `POST ${node.fields.url} WITH ${node.fields.body} TO ${node.fields.variable}\n`;
                        break;
                    case 'SAVE':
                        basCode += `SAVE ${node.fields.data} TO "${node.fields.filename}"\n`;
                        break;
                    case 'WAIT':
                        basCode += `WAIT ${node.fields.duration}\n`;
                        break;
                    case 'SET BOT MEMORY':
                        basCode += `SET BOT MEMORY "${node.fields.key}", ${node.fields.value}\n`;
                        break;
                    case 'GET BOT MEMORY':
                        basCode += `GET BOT MEMORY "${node.fields.key}" AS ${node.fields.variable}\n`;
                        break;
                    case 'SET USER MEMORY':
                        basCode += `SET USER MEMORY "${node.fields.key}", ${node.fields.value}\n`;
                        break;
                    case 'GET USER MEMORY':
                        basCode += `GET USER MEMORY "${node.fields.key}" AS ${node.fields.variable}\n`;
                        break;
                }
            });

            // Download as file
            const blob = new Blob([basCode], { type: 'text/plain' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = (document.getElementById('current-filename').value || 'dialog') + '.bas';
            a.click();
            URL.revokeObjectURL(url);
        }

        // New Design
        function newDesign() {
            if (state.nodes.size > 0) {
                if (!confirm('Clear current design? Unsaved changes will be lost.')) return;
            }
            document.getElementById('canvas-inner').innerHTML = '';
            state.nodes.clear();
            state.connections = [];
            state.selectedNode = null;
            state.history = [];
            state.historyIndex = -1;
            state.nextNodeId = 1;
            document.getElementById('current-filename').value = '';
            document.getElementById('file-name').textContent = 'Untitled';
            updateConnections();
            updatePropertiesPanel();
            updateStatusBar();
        }

        // File selection in open modal
        document.addEventListener('click', (e) => {
            const fileItem = e.target.closest('.file-item');
            if (fileItem) {
                document.querySelectorAll('.file-item').forEach(f => f.classList.remove('selected'));
                fileItem.classList.add('selected');
                document.getElementById('selected-file').value = fileItem.dataset.path;
            }
        });

        function showMagicPanel() {
            const panel = document.getElementById('magic-panel');
            panel.classList.add('visible');
            analyzeMagicSuggestions();
        }

        function hideMagicPanel() {
            const panel = document.getElementById('magic-panel');
            if (panel) {
                panel.classList.remove('visible');
            }
        }

        async function analyzeMagicSuggestions() {
            const content = document.getElementById('magic-content');
            content.innerHTML = '<div class="magic-loading"><div class="spinner"></div><p>Analyzing your dialog...</p></div>';

            const nodes = Array.from(state.nodes.values());
            const dialogData = {
                nodes: nodes.map(n => ({ type: n.type, fields: n.fields })),
                connections: state.connections.length,
                filename: document.getElementById('current-filename').value || 'untitled'
            };

            try {
                const response = await fetch('/api/designer/magic', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(dialogData)
                });

                if (response.ok) {
                    const suggestions = await response.json();
                    renderMagicSuggestions(suggestions);
                } else {
                    renderFallbackSuggestions(dialogData);
                }
            } catch (e) {
                renderFallbackSuggestions(dialogData);
            }
        }

        function renderFallbackSuggestions(dialogData) {
            const suggestions = [];
            const nodes = dialogData.nodes;

            if (!nodes.some(n => n.type === 'HEAR')) {
                suggestions.push({
                    type: 'ux',
                    title: 'Add User Input',
                    description: 'Your dialog has no HEAR nodes. Consider adding user input to make it interactive.'
                });
            }

            if (nodes.filter(n => n.type === 'TALK').length > 5) {
                suggestions.push({
                    type: 'ux',
                    title: 'Break Up Long Responses',
                    description: 'You have many TALK nodes. Consider grouping related messages or using a menu.'
                });
            }

            if (!nodes.some(n => n.type === 'IF' || n.type === 'SWITCH')) {
                suggestions.push({
                    type: 'feature',
                    title: 'Add Decision Logic',
                    description: 'Add IF or SWITCH nodes to handle different user responses dynamically.'
                });
            }

            if (dialogData.connections < nodes.length - 1 && nodes.length > 1) {
                suggestions.push({
                    type: 'perf',
                    title: 'Check Connections',
                    description: 'Some nodes may not be connected. Ensure all nodes flow properly.'
                });
            }

            suggestions.push({
                type: 'a11y',
                title: 'Use Clear Language',
                description: 'Keep messages short and clear. Avoid jargon for better accessibility.'
            });

            renderMagicSuggestions(suggestions);
        }

        function renderMagicSuggestions(suggestions) {
            const content = document.getElementById('magic-content');
            if (!suggestions || suggestions.length === 0) {
                content.innerHTML = '<p style="text-align:center;color:var(--text-secondary, #94a3b8);padding:40px;">Your dialog looks great! No suggestions at this time.</p>';
                return;
            }

            const icons = {
                ux: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>',
                perf: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>',
                a11y: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>',
                feature: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2Z"/></svg>'
            };

            content.innerHTML = suggestions.map(s => `
                <div class="magic-suggestion">
                    <div class="magic-suggestion-header">
                        <div class="magic-suggestion-icon ${s.type}">${icons[s.type] || icons.feature}</div>
                        <span class="magic-suggestion-title">${s.title}</span>
                    </div>
                    <p class="magic-suggestion-desc">${s.description}</p>
                </div>
            `).join('');
        }

        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey && e.key === 'm') {
                e.preventDefault();
                showMagicPanel();
            }
            if (e.ctrlKey && e.key === 'f') {
                e.preventDefault();
                showSearchPanel();
            }
        });
