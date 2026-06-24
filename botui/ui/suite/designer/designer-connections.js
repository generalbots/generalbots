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
