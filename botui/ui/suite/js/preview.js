var previewState = {
  active: false,
  projector: false,
  params: [],
  confirmCallback: null,
};

function showPreview(title, params, onConfirm) {
  previewState.active = true;
  previewState.params = params || [];
  previewState.confirmCallback = onConfirm || null;

  var modal = document.getElementById('previewModal');
  if (!modal) return;

  document.getElementById('previewTitle').textContent = title;

  var body = document.getElementById('previewBody');
  if (!body) return;
  body.innerHTML = '';

  if (params && params.length > 0) {
    params.forEach(function(p, i) {
      var field = document.createElement('div');
      field.className = 'preview-field' + (p.nameField ? ' name-field' : '');
      var label = document.createElement('label');
      label.textContent = p.label || p.name || 'Parameter ' + (i + 1);
      var input;
      if (p.type === 'textarea') {
        input = document.createElement('textarea');
        input.rows = 3;
      } else {
        input = document.createElement('input');
        input.type = p.type || 'text';
      }
      input.name = p.name || 'param_' + i;
      input.value = p.value || '';
      input.placeholder = p.placeholder || '';
      input.dataset.index = i;
      field.appendChild(label);
      field.appendChild(input);
      body.appendChild(field);
    });
  } else {
    body.innerHTML = '<p style="color:#666;font-style:italic;">No parameters to display.</p>';
  }

  modal.classList.add('show');
  if (previewState.projector) modal.classList.add('projector');
}

function closePreviewModal() {
  var modal = document.getElementById('previewModal');
  if (modal) {
    modal.classList.remove('show');
    modal.classList.remove('projector');
  }
  previewState.active = false;
  previewState.confirmCallback = null;
}

function toggleProjector() {
  previewState.projector = !previewState.projector;
  var modal = document.getElementById('previewModal');
  if (modal) {
    if (previewState.projector) {
      modal.classList.add('projector');
    } else {
      modal.classList.remove('projector');
    }
  }
}

function confirmPreview() {
  var modal = document.getElementById('previewModal');
  if (!modal) return;

  var inputs = modal.querySelectorAll('.preview-field input, .preview-field textarea');
  var values = {};
  inputs.forEach(function(input) {
    values[input.name] = input.value;
  });

  if (typeof previewState.confirmCallback === 'function') {
    previewState.confirmCallback(values);
  }

  closePreviewModal();
}
