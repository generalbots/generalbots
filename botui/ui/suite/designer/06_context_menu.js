/* Dialog Designer — split from suite/designer.js for #1731 */

// Context Menu
function initContextMenu() {
    const canvas = document.getElementById('canvas');
    const contextMenu = document.getElementById('context-menu');

    if (canvas && contextMenu) {
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
}

function hideContextMenu() {
    const contextMenu = document.getElementById('context-menu');
    if (contextMenu) {
        contextMenu.classList.remove('visible');
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
    const nodeCount = document.getElementById('node-count');
    const connCount = document.getElementById('connection-count');
    if (nodeCount) nodeCount.textContent = state.nodes.size + ' nodes';
    if (connCount) connCount.textContent = state.connections.length + ' connections';
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
        if (id === 'open-modal' && typeof htmx !== 'undefined') {
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
