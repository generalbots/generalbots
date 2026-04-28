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
