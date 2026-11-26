(function(){var root=(document.currentScript&&document.currentScript.closest('[id^="window-body-"]'))||null,b=root?root.querySelector('#btn-edit-text'):document.getElementById('btn-edit-text'),ds=(root&&root.dataset)||{},wid=ds.windowId||(root&&root.id?root.id.replace('window-body-',''):''),m=(window.__GB_WINDOW_FILE_CTX&&wid&&window.__GB_WINDOW_FILE_CTX[wid])||{};if(b&&(ds.gbFilePath||m.path||window.__EDITOR_FILE_PATH))b.style.display='';})();
// Designer State — #2956: the window manager re-evaluates these classic app
        // scripts on every Designer open. A top-level `const state` threw
        // "Identifier 'state' has already been declared" on the second evaluation
        // and also collided with other apps that own a global `state`. Keep the
        // shared objects on a uniquely named, guarded namespace: re-opening the
        // window reuses the live state instead of resetting or redeclaring it.
        if (!window.GBDesignerState) {
            window.GBDesignerState = {
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
        }
        var GBDesignerState = window.GBDesignerState;

        // Node Templates
        // One line per node type: kept compact so this state module stays
        // below the 450-line limit (the interaction code below needs room).
        if (!window.GBDesignerTemplates) {
            window.GBDesignerTemplates = {
            'TALK': { fields: [{ name: 'message', label: 'Message', type: 'textarea', default: 'Hello!' }], hasInput: true, hasOutput: true },
            'HEAR': { fields: [{ name: 'variable', label: 'Variable', type: 'text', default: 'response' }, { name: 'type', label: 'Type', type: 'select', options: ['STRING', 'NUMBER', 'DATE', 'EMAIL', 'PHONE'], default: 'STRING' }], hasInput: true, hasOutput: true },
            'SET': { fields: [{ name: 'variable', label: 'Variable', type: 'text', default: 'value' }, { name: 'expression', label: 'Expression', type: 'text', default: '' }], hasInput: true, hasOutput: true },
            'IF': { fields: [{ name: 'condition', label: 'Condition', type: 'text', default: 'value = 1' }], hasInput: true, hasOutput: false, hasOutputTrue: true, hasOutputFalse: true },
            'FOR': { fields: [{ name: 'variable', label: 'Item Variable', type: 'text', default: 'item' }, { name: 'collection', label: 'Collection', type: 'text', default: 'items' }], hasInput: true, hasOutput: true, hasLoopOutput: true },
            'SWITCH': { fields: [{ name: 'expression', label: 'Expression', type: 'text', default: 'value' }], hasInput: true, hasOutput: true },
            'CALL': { fields: [{ name: 'procedure', label: 'Procedure', type: 'text', default: '' }, { name: 'arguments', label: 'Arguments', type: 'text', default: '' }], hasInput: true, hasOutput: true },
            'SEND MAIL': { fields: [{ name: 'to', label: 'To', type: 'text', default: '' }, { name: 'subject', label: 'Subject', type: 'text', default: '' }, { name: 'body', label: 'Body', type: 'textarea', default: '' }], hasInput: true, hasOutput: true },
            'GET': { fields: [{ name: 'url', label: 'URL', type: 'text', default: '' }, { name: 'variable', label: 'Result Variable', type: 'text', default: 'result' }], hasInput: true, hasOutput: true },
            'POST': { fields: [{ name: 'url', label: 'URL', type: 'text', default: '' }, { name: 'body', label: 'Body', type: 'textarea', default: '' }, { name: 'variable', label: 'Result Variable', type: 'text', default: 'result' }], hasInput: true, hasOutput: true },
            'SAVE': { fields: [{ name: 'filename', label: 'Filename', type: 'text', default: 'data.csv' }, { name: 'data', label: 'Data', type: 'text', default: '' }], hasInput: true, hasOutput: true },
            'WAIT': { fields: [{ name: 'duration', label: 'Duration (seconds)', type: 'text', default: '5' }], hasInput: true, hasOutput: true },
            'SET BOT MEMORY': { fields: [{ name: 'key', label: 'Key', type: 'text', default: '' }, { name: 'value', label: 'Value', type: 'text', default: '' }], hasInput: true, hasOutput: true },
            'GET BOT MEMORY': { fields: [{ name: 'key', label: 'Key', type: 'text', default: '' }, { name: 'variable', label: 'Result Variable', type: 'text', default: 'value' }], hasInput: true, hasOutput: true },
            'SET USER MEMORY': { fields: [{ name: 'key', label: 'Key', type: 'text', default: '' }, { name: 'value', label: 'Value', type: 'text', default: '' }], hasInput: true, hasOutput: true },
            'GET USER MEMORY': { fields: [{ name: 'key', label: 'Key', type: 'text', default: '' }, { name: 'variable', label: 'Result Variable', type: 'text', default: 'value' }], hasInput: true, hasOutput: true }
            };
        }
        var GBDesignerTemplates = window.GBDesignerTemplates;

        // Pointer capture is shared by the canvas and the node/port drag code
        // in designer-inline-nodes.js (loaded after this file).
        function gbCapturePointer(el, pointerId) {
            if (!el || !el.setPointerCapture) return;
            try { el.setPointerCapture(pointerId); } catch (error) {
                if (window.gbLogError) window.gbLogError(error, "designer.capture");
            }
        }
        function gbReleasePointer(el, pointerId) {
            if (!el || !el.releasePointerCapture) return;
            try { el.releasePointerCapture(pointerId); } catch (error) {
                if (window.gbLogError) window.gbLogError(error, "designer.release");
            }
        }

        // Initialize
        function initDesigner() {
            gbLog.debug('initDesigner called');
            initDragAndDrop();
            initCanvasInteraction();
            initKeyboardShortcuts();
            initContextMenu();
            updateStatusBar();
            loadFromUrlParams();
        }

        // #2956 — publish the initializer from the script that owns it. The
        // bootstrap in designer-inline-search.js calls the published function,
        // so an evaluation-order change can no longer produce
        // "initDesigner is not defined".
        window.initDesigner = initDesigner;

        // #2956 — node counts must tolerate a missing/partially built state
        // (e.g. an older app still holding a foreign global). Never dereference
        // `.size` on an undefined nodes collection.
        function gbDesignerNodeCount() {
            return (window.GBDesignerState && window.GBDesignerState.nodes)
                ? window.GBDesignerState.nodes.size : 0;
        }
        window.gbDesignerNodeCount = gbDesignerNodeCount;

        // The auto-run bootstrap lives at the end of the last part
        // (designer-inline-search.js). This file is evaluated before p2/p3 define
        // initKeyboardShortcuts and initContextMenu, so initializing here threw
        // "initKeyboardShortcuts is not defined" whenever the designer was
        // injected via HTMX (#1342).

        // Also run when HTMX swaps content — bind once per document so a
        // re-open cannot stack duplicate initializers (idempotent init guard).
        if (!window.__gbDesignerSwapBound) {
            window.__gbDesignerSwapBound = true;
            document.body.addEventListener('htmx:afterSwap', (e) => {
                if (e.detail.target.id === 'main-content') {
                    gbLog.debug('htmx:afterSwap detected for main-content');
                    window.initDesigner();
                }
            });
        }

        // #2282 — Drive passes file context on the window body; accept URL/hash for direct opens.
        async function loadFromUrlParams() {
            const ctx = (typeof window.gbDesignerFileCtx === 'function')
                ? window.gbDesignerFileCtx() : { bucket: null, path: null };
            let bucket = ctx.bucket || null;
            let path = ctx.path || null;

            // Direct app opens carry the context in the query string OR in the
            // hash fragment (e.g. /#designer?bucket=x&path=y).
            if (!bucket || !path) {
                const queryParams = new URLSearchParams(window.location.search);
                bucket = bucket || queryParams.get('bucket');
                path = path || queryParams.get('path');
            }
            if (!bucket || !path) {
                const hash = window.location.hash;
                const hashQueryIndex = hash.indexOf('?');
                if (hashQueryIndex !== -1) {
                    const hashParams = new URLSearchParams(hash.substring(hashQueryIndex + 1));
                    bucket = bucket || hashParams.get('bucket');
                    path = path || hashParams.get('path');
                }
            }

            gbLog.debug('loadFromUrlParams called:', { bucket, path, hash: window.location.hash, search: window.location.search });

            if (bucket && path) {
                const fileName = path.split('/').pop() || 'dialog.bas';
                document.getElementById('current-filename').value = path;
                document.getElementById('selected-file').value = path;

                GBDesignerState.driveSource = { bucket, path };

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
                    gbLog.debug('Loaded .bas content:', content.substring(0, 200) + '...');

                    // Parse BASIC code and create nodes
                    parseBasicCodeToNodes(content);
                    updateStatusBar();

                    const statusEl = document.querySelector('.status-item span');
                    if (statusEl) {
                        statusEl.textContent = `Loaded: ${fileName}`;
                    }
                } catch (err) {
                    console.error('Failed to load .bas file:', err);
                    GBModal.alert(`Failed to load file: ${err.message}`);
                }
            }
        }

        // Parse BASIC code and create visual nodes
        function parseBasicCodeToNodes(content) {
            gbLog.debug('parseBasicCodeToNodes called');
            GBDesignerState.nodes.clear();
            GBDesignerState.connections = [];
            GBDesignerState.nextNodeId = 1;

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

                if (nodeType && GBDesignerTemplates[nodeType]) {
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
                        gbLog.debug('Created node:', nodeType, fields);
                    }
                }
            }

            gbLog.debug(`Parsed ${nodeCount} nodes from BASIC code`);
            updateStatusBar();
            saveToHistory();
        }

        // Initialize canvas with loaded nodes from server (called by HTMX response)
        function initializeCanvas() {
            gbLog.debug('initializeCanvas called');
            const canvasLoaded = document.querySelector('.canvas-loaded');
            if (!canvasLoaded) {
                gbLog.debug('No canvas-loaded element found');
                return;
            }

            const content = canvasLoaded.dataset.content || '';
            gbLog.debug('Canvas content from server:', content.substring(0, 100));

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
                    const x = (e.clientX - rect.left) / GBDesignerState.zoom;
                    const y = (e.clientY - rect.top) / GBDesignerState.zoom;
                    createNode(nodeType, snapToGrid(x), snapToGrid(y));
                }
            });
        }

        // Canvas Interaction (#2419): one finger on empty canvas pans, two
        // fingers pinch-zoom, nodes own their own long-press/select gesture.
        function initCanvasInteraction() {
            const canvas = document.getElementById('canvas');
            const container = document.getElementById('canvas-container');

            if (!canvas || !container) {
                console.warn('initCanvasInteraction: canvas or canvas-container not found');
                return;
            }
            canvas.style.touchAction = 'none';
            const pointers = new Map();
            let panStart = null;
            let pinchStart = null;

            function isBackground(target) {
                return target === canvas || (target && (target.id === 'canvas-inner' || target.id === 'connections-layer'));
            }

            function points() {
                return Array.from(pointers.values());
            }

            canvas.addEventListener('pointerdown', (e) => {
                if (e.target.closest && e.target.closest('.node')) return;
                const middleMouse = e.pointerType === 'mouse' && e.button === 1;
                if (!middleMouse && e.button !== 0) return;
                if (!isBackground(e.target) && !middleMouse) return;
                pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
                gbCapturePointer(canvas, e.pointerId);
                if (pointers.size === 1) {
                    panStart = { x: e.clientX - GBDesignerState.pan.x, y: e.clientY - GBDesignerState.pan.y };
                    canvas.style.cursor = 'grabbing';
                } else if (pointers.size === 2) {
                    panStart = null;
                    const p = points();
                    const gap = Math.hypot(p[0].x - p[1].x, p[0].y - p[1].y) || 1;
                    pinchStart = { gap: gap, zoom: GBDesignerState.zoom };
                }
            });

            canvas.addEventListener('pointermove', (e) => {
                if (!pointers.has(e.pointerId)) return;
                pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
                if (pointers.size >= 2 && pinchStart) {
                    const p = points();
                    const gap = Math.hypot(p[0].x - p[1].x, p[0].y - p[1].y) || 1;
                    GBDesignerState.zoom = Math.min(Math.max(pinchStart.zoom * gap / pinchStart.gap, 0.25), 2);
                    updateCanvasTransform();
                    updateZoomDisplay();
                    return;
                }
                if (panStart) {
                    GBDesignerState.pan.x = e.clientX - panStart.x;
                    GBDesignerState.pan.y = e.clientY - panStart.y;
                    updateCanvasTransform();
                }
            });

            function endPointer(e) {
                if (!pointers.has(e.pointerId)) return;
                pointers.delete(e.pointerId);
                gbReleasePointer(canvas, e.pointerId);
                if (pointers.size < 2) pinchStart = null;
                if (pointers.size === 1) {
                    const remaining = points()[0];
                    panStart = { x: remaining.x - GBDesignerState.pan.x, y: remaining.y - GBDesignerState.pan.y };
                } else if (pointers.size === 0) {
                    panStart = null;
                    canvas.style.cursor = 'default';
                }
            }
            canvas.addEventListener('pointerup', endPointer);
            canvas.addEventListener('pointercancel', endPointer);

            // Zoom with scroll (desktop behaviour unchanged)
            container.addEventListener('wheel', (e) => {
                e.preventDefault();
                const delta = e.deltaY > 0 ? -0.1 : 0.1;
                const newZoom = Math.min(Math.max(GBDesignerState.zoom + delta, 0.25), 2);
                GBDesignerState.zoom = newZoom;
                updateCanvasTransform();
                updateZoomDisplay();
            });
        }

        function updateCanvasTransform() {
            const inner = document.getElementById('canvas-inner');
            inner.style.transform = `translate(${GBDesignerState.pan.x}px, ${GBDesignerState.pan.y}px) scale(${GBDesignerState.zoom})`;
        }

        function updateZoomDisplay() {
            document.getElementById('zoom-value').textContent = Math.round(GBDesignerState.zoom * 100) + '%';
        }

        // Grid snapping
        function snapToGrid(value, gridSize = 20) {
            return Math.round(value / gridSize) * gridSize;
        }

        // Create Node
