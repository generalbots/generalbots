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
});
