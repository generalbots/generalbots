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
  document.getElementById('canvas-inner').innerHTML = '';
  state.nodes.clear();
  state.connections = [];

  snapshot.nodes.forEach(node => {
    state.nodes.set(node.id, {...node});
    renderNode(node);
  });

  state.connections = [...snapshot.connections];
  updateConnections();
  updateStatusBar();
}
