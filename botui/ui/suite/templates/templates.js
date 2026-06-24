(function() {
'use strict';
let allTemplates = [];
let currentFilter = 'all';

function filterTemplates(view, btn) {
    currentFilter = view;
    document.querySelectorAll('.crm-tab').forEach(function(t) { t.classList.remove('active'); });
    if (btn) btn.classList.add('active');
    renderTemplates();
}

function searchTemplates(q) { renderTemplates(q.toLowerCase()); }

function renderTemplates(searchQuery) {
    var grid = document.getElementById('templatesList');
    var filtered = allTemplates;
    if (currentFilter !== 'all') {
        filtered = filtered.filter(function(t) { return t.channel === currentFilter; });
    }
    if (searchQuery) {
        filtered = filtered.filter(function(t) { return (t.name || '').toLowerCase().includes(searchQuery) || (t.body || '').toLowerCase().includes(searchQuery); });
    }
    if (filtered.length === 0) {
        grid.innerHTML = '<div style="grid-column:1/-1;padding:40px;text-align:center;color:var(--text-secondary)">No templates found.</div>';
        return;
    }
    grid.innerHTML = filtered.map(function(tpl) {
        return '<div class="template-card" onclick="previewTemplateFull(\'' + tpl.id + '\')">'
            + '<div class="template-card-header">'
            + '<h3 class="template-card-title">' + (tpl.name || 'Unnamed') + '</h3>'
            + '<span class="template-channel ' + (tpl.channel || 'email') + '">' + (tpl.channel || 'email') + '</span>'
            + '</div>'
            + '<div class="template-preview">' + (tpl.body || '').substring(0, 120) + '</div>'
            + '<div class="template-card-actions" onclick="event.stopPropagation()">'
            + '<button onclick="showTemplateModal(\'' + tpl.id + '\')">Edit</button>'
            + '<button onclick="duplicateTemplate(\'' + tpl.id + '\')">Duplicate</button>'
            + '<button onclick="previewTemplateFull(\'' + tpl.id + '\')">Preview</button>'
            + '<button style="color:#ef4444" onclick="deleteTemplate(\'' + tpl.id + '\')">Delete</button>'
            + '</div>'
            + '</div>';
    }).join('');
}

function showTemplateModal(id) {
    var modal = document.getElementById('template-modal');
    var title = document.getElementById('template-modal-title');
    var submit = document.getElementById('tpl-form-submit');
    if (id) {
        var tpl = allTemplates.find(function(t) { return t.id === id; });
        if (tpl) {
            document.getElementById('tpl-form-id').value = tpl.id;
            document.getElementById('tpl-form-name').value = tpl.name || '';
            document.getElementById('tpl-form-channel').value = tpl.channel || 'email';
            document.getElementById('tpl-form-subject').value = tpl.subject || '';
            document.getElementById('tpl-form-body').value = tpl.body || '';
            document.getElementById('tpl-form-ai').value = tpl.ai_prompt || '';
            title.textContent = 'Edit Template';
            submit.textContent = 'Save Changes';
        }
    } else {
        document.getElementById('template-form').reset();
        document.getElementById('tpl-form-id').value = '';
        title.textContent = 'Create Template';
        submit.textContent = 'Create Template';
    }
    document.getElementById('tpl-preview-box').style.display = 'none';
    modal.style.display = 'flex';
}

function hideTemplateModal() {
    document.getElementById('template-modal').style.display = 'none';
}

function hidePreviewModal() {
    document.getElementById('preview-modal').style.display = 'none';
}

function insertVariable(varName) {
    var textarea = document.getElementById('tpl-form-body');
    var pos = textarea.selectionStart;
    var text = textarea.value;
    textarea.value = text.substring(0, pos) + '{{' + varName + '}}' + text.substring(pos);
    textarea.focus();
    textarea.selectionStart = textarea.selectionEnd = pos + varName.length + 4;
    previewTemplate();
}

function previewTemplate() {
    var body = document.getElementById('tpl-form-body').value;
    if (!body.trim()) {
        document.getElementById('tpl-preview-box').style.display = 'none';
        return;
    }
    var rendered = body
        .replace(/\{\{contact\.name\}\}/g, 'John Doe')
        .replace(/\{\{contact\.email\}\}/g, 'john@example.com')
        .replace(/\{\{company\}\}/g, 'Acme Corp')
        .replace(/\{\{date\}\}/g, new Date().toLocaleDateString())
        .replace(/\{\{bot\.name\}\}/g, 'Support Bot')
        .replace(/\{\{user\.name\}\}/g, 'Admin User')
        .replace(/\{\{[^}]+\}\}/g, '[variable]');
    document.getElementById('tpl-preview-content').textContent = rendered;
    document.getElementById('tpl-preview-box').style.display = 'block';
}

function previewTemplateFull(id) {
    var tpl = allTemplates.find(function(t) { return t.id === id; });
    if (!tpl) return;
    var rendered = (tpl.body || '')
        .replace(/\{\{contact\.name\}\}/g, 'John Doe')
        .replace(/\{\{contact\.email\}\}/g, 'john@example.com')
        .replace(/\{\{company\}\}/g, 'Acme Corp')
        .replace(/\{\{date\}\}/g, new Date().toLocaleDateString())
        .replace(/\{\{bot\.name\}\}/g, 'Support Bot')
        .replace(/\{\{user\.name\}\}/g, 'Admin User')
        .replace(/\{\{[^}]+\}\}/g, '[variable]');
    document.getElementById('preview-modal-body').textContent = rendered;
    document.getElementById('preview-modal').style.display = 'flex';
}

async function saveTemplate(e) {
    e.preventDefault();
    var id = document.getElementById('tpl-form-id').value;
    var data = {
        name: document.getElementById('tpl-form-name').value,
        channel: document.getElementById('tpl-form-channel').value,
        subject: document.getElementById('tpl-form-subject').value,
        body: document.getElementById('tpl-form-body').value,
        ai_prompt: document.getElementById('tpl-form-ai').value
    };
    var url = id ? '/api/crm/templates/' + id : '/api/crm/templates';
    var method = id ? 'PUT' : 'POST';
    try {
        var resp = await fetch(url, {
            method: method,
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });
        if (resp.ok) {
            hideTemplateModal();
            loadTemplates();
        }
    } catch (err) { console.error('saveTemplate:', err); }
}

async function deleteTemplate(id) {
    if (!confirm('Delete this template?')) return;
    try {
        await fetch('/api/crm/templates/' + id, { method: 'DELETE' });
        allTemplates = allTemplates.filter(function(t) { return t.id !== id; });
        renderTemplates();
    } catch (e) { console.error('deleteTemplate:', e); }
}

async function duplicateTemplate(id) {
    var tpl = allTemplates.find(function(t) { return t.id === id; });
    if (!tpl) return;
    try {
        await fetch('/api/crm/templates', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name: tpl.name + ' (Copy)', channel: tpl.channel, subject: tpl.subject, body: tpl.body, ai_prompt: tpl.ai_prompt })
        });
        loadTemplates();
    } catch (e) { console.error('duplicateTemplate:', e); }
}

function exportTemplates() {
    var data = JSON.stringify(allTemplates, null, 2);
    var blob = new Blob([data], { type: 'application/json' });
    var a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = 'templates-export.json';
    a.click();
}

function importTemplates() {
    var input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = function(e) {
        var file = e.target.files[0];
        var reader = new FileReader();
        reader.onload = async function(ev) {
            try {
                var templates = JSON.parse(ev.target.result);
                for (var i = 0; i < templates.length; i++) {
                    await fetch('/api/crm/templates', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(templates[i])
                    });
                }
                loadTemplates();
            } catch (err) { alert('Invalid JSON file'); }
        };
        reader.readAsText(file);
    };
    input.click();
}

async function loadTemplates() {
    try {
        var resp = await fetch('/api/crm/templates');
        if (resp.ok) {
            allTemplates = await resp.json();
            if (!Array.isArray(allTemplates)) allTemplates = [];
            renderTemplates();
        }
    } catch (e) { console.error('loadTemplates:', e); }
}

document.getElementById('tpl-form-channel').addEventListener('change', function() {
    document.getElementById('tpl-subject-group').style.display = this.value === 'email' ? 'block' : 'none';
});

loadTemplates();
})();
