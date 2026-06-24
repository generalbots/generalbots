function updatePropertiesPanel() {
  const content = document.getElementById('properties-content');
  const empty = document.getElementById('properties-empty');

  if (!state.selectedNode) {
    content.style.display = 'none';
    empty.style.display = 'block';
    return;
  }

  const node = state.nodes.get(state.selectedNode);
  if (!node) return;

  const template = nodeTemplates[node.type];
  empty.style.display = 'none';
  content.style.display = 'block';

  let html = `
  <div class="property-group">
    <div class="property-group-title">Node Info</div>
    <div class="property-field">
      <label class="property-label">Type</label>
      <input type="text" class="property-input" value="${node.type}" readonly>
    </div>
    <div class="property-field">
      <label class="property-label">ID</label>
      <input type="text" class="property-input" value="${node.id}" readonly>
    </div>
  </div>
  <div class="property-group">
    <div class="property-group-title">Properties</div>
  `;

  template.fields.forEach(field => {
    const value = node.fields[field.name] || '';
    if (field.type === 'textarea') {
      html += `
      <div class="property-field">
        <label class="property-label">${field.label}</label>
        <textarea class="property-textarea" data-node="${node.id}" data-field="${field.name}">${value}</textarea>
      </div>
      `;
    } else {
      html += `
      <div class="property-field">
        <label class="property-label">${field.label}</label>
        <input type="text" class="property-input" data-node="${node.id}" data-field="${field.name}" value="${value}">
      </div>
      `;
    }
  });

  html += '</div>';
  content.innerHTML = html;

  content.querySelectorAll('.property-input:not([readonly]), .property-textarea').forEach(input => {
    input.addEventListener('change', (e) => {
      const n = state.nodes.get(e.target.dataset.node);
      if (n) {
        n.fields[e.target.dataset.field] = e.target.value;
        const nodeInput = document.querySelector(`#${n.id} [data-field="${e.target.dataset.field}"]`);
        if (nodeInput) nodeInput.value = e.target.value;
        saveToHistory();
      }
    });
  });
}
