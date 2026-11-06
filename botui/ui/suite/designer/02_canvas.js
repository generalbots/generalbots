/* Dialog Designer — split from suite/designer.js for #1731 */

// Drag and Drop from Toolbox
function initDragAndDrop() {
    const toolboxItems = document.querySelectorAll('.toolbox-item');
    const canvas = document.getElementById('canvas-inner');

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
function createNode(type, x, y) {
    const template = nodeTemplates[type];
    if (!template) return;

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
}
