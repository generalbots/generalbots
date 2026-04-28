function initCanvasInteraction() {
  const canvas = document.getElementById('canvas');
  const container = document.getElementById('canvas-container');

  if (!canvas || !container) {
    console.warn('initCanvasInteraction: canvas or canvas-container not found');
    return;
  }

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

function snapToGrid(value, gridSize = 20) {
  return Math.round(value / gridSize) * gridSize;
}

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
