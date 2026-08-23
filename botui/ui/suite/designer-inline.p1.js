(function(){var b=document.getElementById('btn-edit-text');if(b&&window.__EDITOR_FILE_PATH)b.style.display='';})();
// Designer State
        const state = {
            nodes: new Map(),
            connections: [],
            selectedNode: null,
            selectedConnection: null,
            isDragging: false,
            isConnecting: false,
            connectionStart: null,
            zoom: 1,
            pan: { x: 0, y: 0 },
            history: [],
            historyIndex: -1,
            clipboard: null,
            nextNodeId: 1,
            driveSource: null
        };

        // Node Templates
        const nodeTemplates = {
            'TALK': {
                fields: [
                    { name: 'message', label: 'Message', type: 'textarea', default: 'Hello!' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'HEAR': {
                fields: [
                    { name: 'variable', label: 'Variable', type: 'text', default: 'response' },
                    { name: 'type', label: 'Type', type: 'select', options: ['STRING', 'NUMBER', 'DATE', 'EMAIL', 'PHONE'], default: 'STRING' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'SET': {
                fields: [
                    { name: 'variable', label: 'Variable', type: 'text', default: 'value' },
                    { name: 'expression', label: 'Expression', type: 'text', default: '' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'IF': {
                fields: [
                    { name: 'condition', label: 'Condition', type: 'text', default: 'value = 1' }
                ],
                hasInput: true,
                hasOutput: false,
                hasOutputTrue: true,
                hasOutputFalse: true
            },
            'FOR': {
                fields: [
                    { name: 'variable', label: 'Item Variable', type: 'text', default: 'item' },
                    { name: 'collection', label: 'Collection', type: 'text', default: 'items' }
                ],
                hasInput: true,
                hasOutput: true,
                hasLoopOutput: true
            },
            'SWITCH': {
                fields: [
                    { name: 'expression', label: 'Expression', type: 'text', default: 'value' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'CALL': {
                fields: [
                    { name: 'procedure', label: 'Procedure', type: 'text', default: '' },
                    { name: 'arguments', label: 'Arguments', type: 'text', default: '' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'SEND MAIL': {
                fields: [
                    { name: 'to', label: 'To', type: 'text', default: '' },
                    { name: 'subject', label: 'Subject', type: 'text', default: '' },
                    { name: 'body', label: 'Body', type: 'textarea', default: '' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'GET': {
                fields: [
                    { name: 'url', label: 'URL', type: 'text', default: '' },
                    { name: 'variable', label: 'Result Variable', type: 'text', default: 'result' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'POST': {
                fields: [
                    { name: 'url', label: 'URL', type: 'text', default: '' },
                    { name: 'body', label: 'Body', type: 'textarea', default: '' },
                    { name: 'variable', label: 'Result Variable', type: 'text', default: 'result' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'SAVE': {
                fields: [
                    { name: 'filename', label: 'Filename', type: 'text', default: 'data.csv' },
                    { name: 'data', label: 'Data', type: 'text', default: '' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'WAIT': {
                fields: [
                    { name: 'duration', label: 'Duration (seconds)', type: 'text', default: '5' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'SET BOT MEMORY': {
                fields: [
                    { name: 'key', label: 'Key', type: 'text', default: '' },
                    { name: 'value', label: 'Value', type: 'text', default: '' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'GET BOT MEMORY': {
                fields: [
                    { name: 'key', label: 'Key', type: 'text', default: '' },
                    { name: 'variable', label: 'Variable', type: 'text', default: 'value' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'SET USER MEMORY': {
                fields: [
                    { name: 'key', label: 'Key', type: 'text', default: '' },
                    { name: 'value', label: 'Value', type: 'text', default: '' }
                ],
                hasInput: true,
                hasOutput: true
            },
            'GET USER MEMORY': {
                fields: [
                    { name: 'key', label: 'Key', type: 'text', default: '' },
                    { name: 'variable', label: 'Variable', type: 'text', default: 'value' }
                ],
                hasInput: true,
                hasOutput: true
            }
        };

        // Initialize
        function initDesigner() {
            console.log('initDesigner called');
            initDragAndDrop();
            initCanvasInteraction();
            initKeyboardShortcuts();
            initContextMenu();
            updateStatusBar();
            loadFromUrlParams();
        }

        // Run on DOMContentLoaded (for direct page load)
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', initDesigner);
        } else {
            // DOM already loaded (HTMX injection case)
            initDesigner();
        }

        // Also run when HTMX swaps content
        document.body.addEventListener('htmx:afterSwap', (e) => {
            if (e.detail.target.id === 'main-content') {
                console.log('htmx:afterSwap detected for main-content');
                initDesigner();
            }
        });

        // Load file from URL parameters (when opening .bas from drive)
        async function loadFromUrlParams() {
            // Parameters can be in query string OR in hash fragment (after #designer?)
            let bucket = null;
            let path = null;

            // First try query string
            const queryParams = new URLSearchParams(window.location.search);
            bucket = queryParams.get('bucket');
            path = queryParams.get('path');

            // If not found, try hash fragment (e.g., /#designer?bucket=x&path=y)
            if (!bucket || !path) {
                const hash = window.location.hash;
                const hashQueryIndex = hash.indexOf('?');
                if (hashQueryIndex !== -1) {
                    const hashParams = new URLSearchParams(hash.substring(hashQueryIndex + 1));
                    bucket = bucket || hashParams.get('bucket');
                    path = path || hashParams.get('path');
                }
            }

            console.log('loadFromUrlParams called:', { bucket, path, hash: window.location.hash, search: window.location.search });

            if (bucket && path) {
                const fileName = path.split('/').pop() || 'dialog.bas';
                document.getElementById('current-filename').value = path;
                document.getElementById('selected-file').value = path;

                state.driveSource = { bucket, path };

                try {
                    // Fetch file content directly from drive API
                    const response = await fetch('/api/files/read', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ bucket, path })
                    });

                    if (!response.ok) {
                        throw new Error(`Failed to load file: ${response.statusText}`);
                    }

                    const data = await response.json();
                    const content = data.content || '';
                    console.log('Loaded .bas content:', content.substring(0, 200) + '...');

                    // Parse BASIC code and create nodes
                    parseBasicCodeToNodes(content);
                    updateStatusBar();

                    const statusEl = document.querySelector('.status-item span');
                    if (statusEl) {
                        statusEl.textContent = `Loaded: ${fileName}`;
                    }
                } catch (err) {
                    console.error('Failed to load .bas file:', err);
                    alert(`Failed to load file: ${err.message}`);
                }
            }
        }

        // Parse BASIC code and create visual nodes
        function parseBasicCodeToNodes(content) {
            console.log('parseBasicCodeToNodes called');
            state.nodes.clear();
            state.connections = [];
            state.nextNodeId = 1;

            const lines = content.split('\n');
            let yPos = 100;
            let nodeCount = 0;

            for (const line of lines) {
                const trimmed = line.trim();
                if (!trimmed || trimmed.startsWith("'")) continue;

                const upper = trimmed.toUpperCase();
                let nodeType = null;
                let fields = {};

                if (upper.startsWith('TALK ')) {
                    nodeType = 'TALK';
                    const match = trimmed.match(/TALK\s+"([^"]*)"/i) || trimmed.match(/TALK\s+(.+)/i);
                    fields.message = match ? match[1] : '';
                } else if (upper.startsWith('HEAR ')) {
                    nodeType = 'HEAR';
                    const match = trimmed.match(/HEAR\s+(\w+)(?:\s+AS\s+(\w+))?/i);
                    fields.variable = match ? match[1] : 'input';
                    fields.type = match && match[2] ? match[2] : 'string';
                } else if (upper.startsWith('SET ') || upper.includes(' = ')) {
                    nodeType = 'SET';
                    const match = trimmed.match(/(?:SET\s+)?(\w+)\s*=\s*(.+)/i);
                    fields.variable = match ? match[1] : 'x';
                    fields.expression = match ? match[2] : '0';
                } else if (upper.startsWith('IF ')) {
                    nodeType = 'IF';
                    const match = trimmed.match(/IF\s+(.+?)\s+THEN/i);
                    fields.condition = match ? match[1] : 'true';
                } else if (upper.startsWith('FOR ')) {
                    nodeType = 'FOR';
                    const match = trimmed.match(/FOR\s+(?:EACH\s+)?(\w+)\s+IN\s+(.+)/i);
                    fields.variable = match ? match[1] : 'item';
                    fields.collection = match ? match[2] : 'items';
                } else if (upper.startsWith('CALL ')) {
                    nodeType = 'CALL';
                    const match = trimmed.match(/CALL\s+(\w+)\s*\(([^)]*)\)/i);
                    fields.procedure = match ? match[1] : 'sub';
                    fields.arguments = match ? match[2] : '';
                } else if (upper.startsWith('WAIT ')) {
                    nodeType = 'WAIT';
                    const match = trimmed.match(/WAIT\s+(\d+)/i);
                    fields.duration = match ? match[1] : '1000';
                } else if (upper.startsWith('GET ')) {
                    nodeType = 'GET';
                    const match = trimmed.match(/GET\s+(.+?)\s+TO\s+(\w+)/i);
                    fields.url = match ? match[1] : '';
                    fields.variable = match ? match[2] : 'result';
                } else if (upper.startsWith('PARAM ')) {
                    nodeType = 'HEAR';
                    const match = trimmed.match(/PARAM\s+(\w+)\s+AS\s+(\w+)/i);
                    fields.variable = match ? match[1] : 'param';
                    fields.type = match ? match[2] : 'string';
                }

                if (nodeType && nodeTemplates[nodeType]) {
                    const node = createNode(nodeType, 400, yPos);
                    if (node) {
                        Object.assign(node.fields, fields);

                        // Update the rendered node with field values
                        const nodeEl = document.getElementById(node.id);
                        if (nodeEl) {
                            nodeEl.querySelectorAll('.node-field-input, .node-field-select, textarea').forEach(input => {
                                const fieldName = input.dataset.field || input.name;
                                if (fields[fieldName] !== undefined) {
                                    input.value = fields[fieldName];
                                }
                            });
                        }

                        yPos += 100;
                        nodeCount++;
                        console.log('Created node:', nodeType, fields);
                    }
                }
            }

            console.log(`Parsed ${nodeCount} nodes from BASIC code`);
            updateStatusBar();
            saveToHistory();
        }

        // Initialize canvas with loaded nodes from server (called by HTMX response)
        function initializeCanvas() {
            console.log('initializeCanvas called');
            const canvasLoaded = document.querySelector('.canvas-loaded');
            if (!canvasLoaded) {
                console.log('No canvas-loaded element found');
                return;
            }

            const content = canvasLoaded.dataset.content || '';
            console.log('Canvas content from server:', content.substring(0, 100));

            // Remove the server-rendered container and parse content client-side
            canvasLoaded.remove();
            parseBasicCodeToNodes(content);
        }

        // Drag and Drop from Toolbox
        function initDragAndDrop() {
            const toolboxItems = document.querySelectorAll('.toolbox-item');
            const canvas = document.getElementById('canvas-inner');

            if (!canvas) {
                console.warn('initDragAndDrop: canvas-inner not found');
                return;
            }

            toolboxItems.forEach(item => {
                item.addEventListener('dragstart', (e) => {
                    e.dataTransfer.setData('nodeType', item.dataset.nodeType);
                    item.classList.add('dragging');
                });

                item.addEventListener('dragend', () => {
                    item.classList.remove('dragging');
                });
            });

            canvas.addEventListener('dragover', (e) => {
                e.preventDefault();
            });

            canvas.addEventListener('drop', (e) => {
                e.preventDefault();
                const nodeType = e.dataTransfer.getData('nodeType');
                if (nodeType) {
                    const rect = canvas.getBoundingClientRect();
                    const x = (e.clientX - rect.left) / state.zoom;
                    const y = (e.clientY - rect.top) / state.zoom;
                    createNode(nodeType, snapToGrid(x), snapToGrid(y));
                }
            });
        }

        // Canvas Interaction
        function initCanvasInteraction() {
            const canvas = document.getElementById('canvas');
            const container = document.getElementById('canvas-container');

            if (!canvas || !container) {
                console.warn('initCanvasInteraction: canvas or canvas-container not found');
                return;
            }

            // Pan with middle mouse or space+drag
            let isPanning = false;
            let panStart = { x: 0, y: 0 };

            canvas.addEventListener('mousedown', (e) => {
                if (e.button === 1 || (e.button === 0 && e.target === canvas)) {
                    isPanning = true;
                    panStart = { x: e.clientX - state.pan.x, y: e.clientY - state.pan.y };
                    canvas.style.cursor = 'grabbing';
                }
            });

            document.addEventListener('mousemove', (e) => {
                if (isPanning) {
                    state.pan.x = e.clientX - panStart.x;
                    state.pan.y = e.clientY - panStart.y;
                    updateCanvasTransform();
                }
            });

            document.addEventListener('mouseup', () => {
                isPanning = false;
                canvas.style.cursor = 'default';
            });

            // Zoom with scroll
            container.addEventListener('wheel', (e) => {
                e.preventDefault();
                const delta = e.deltaY > 0 ? -0.1 : 0.1;
                const newZoom = Math.min(Math.max(state.zoom + delta, 0.25), 2);
                state.zoom = newZoom;
                updateCanvasTransform();
                updateZoomDisplay();
            });
        }

        function updateCanvasTransform() {
            const inner = document.getElementById('canvas-inner');
            inner.style.transform = `translate(${state.pan.x}px, ${state.pan.y}px) scale(${state.zoom})`;
        }

        function updateZoomDisplay() {
            document.getElementById('zoom-value').textContent = Math.round(state.zoom * 100) + '%';
        }

        // Grid snapping
        function snapToGrid(value, gridSize = 20) {
            return Math.round(value / gridSize) * gridSize;
        }

        // Create Node
