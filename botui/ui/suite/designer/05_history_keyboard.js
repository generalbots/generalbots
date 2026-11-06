/* Dialog Designer — split from suite/designer.js for #1731 */

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

function redo() {
    if (state.historyIndex < state.history.length - 1) {
        state.historyIndex++;
        restoreSnapshot(state.history[state.historyIndex]);
    }
}

function restoreSnapshot(snapshot) {
    // Clear canvas
    const canvasInner = document.getElementById('canvas-inner');
    if (canvasInner) canvasInner.innerHTML = '';
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
