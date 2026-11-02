
async function removeDependency(taskId, predecessorId) {
    try {
        const resp = await fetch(`/api/tasks/${taskId}/dependencies/remove`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ predecessor_id: predecessorId })
        });
        if (resp.ok) {
            await loadProjectData(currentProjectId);
            showTaskDetail(taskId);
        } else {
            alert('Failed to remove dependency');
        }
    } catch (e) {
        alert('Error: ' + e.message);
    }
}

async function updateTaskProgress(taskId, pct) {
    try {
        const resp = await fetch(`/api/tasks/${taskId}/progress`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ percent_complete: parseInt(pct, 10) })
        });
        if (resp.ok) {
            loadProjectData(currentProjectId);
        }
    } catch (e) {
        console.error('Failed to update progress:', e);
    }
}

/* ---- Import (open MPP/XML/CSV/JSON) + Export ---- */

function triggerImportPicker() {
    const input = document.getElementById('project-import-input');
    if (input) input.click();
}

async function handleImportFile(input) {
    const file = input.files && input.files[0];
    if (!file) return;

    const form = new FormData();
    form.append('file', file);

    try {
        const resp = await fetch('/api/projects/import', { method: 'POST', body: form });
        if (resp.ok) {
            const result = await resp.json();
            input.value = '';
            if (result.warnings && result.warnings.length) {
                alert(result.warnings.map((w) => w.message).join('\n'));
            }
            loadProjectList();
            if (result.project && result.project.id) {
                selectProject(result.project.id);
            }
        } else {
            const err = await resp.json().catch(() => ({}));
            alert('Import failed: ' + (err.error || resp.status));
        }
    } catch (e) {
        alert('Error: ' + e.message);
    }
}

async function exportProject(format) {
    if (!currentProjectId) { alert('Select a project first'); return; }
    const fmt = format || 'xml';
    try {
        const resp = await fetch(`/api/projects/${currentProjectId}/export?format=${fmt}`);
        if (!resp.ok) {
            alert('Export failed: ' + resp.status);
            return;
        }
        const blob = await resp.blob();
        const name = (currentProjectData && (currentProjectData.name || 'project')) || 'project';
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = name + '.' + (fmt === 'mpp' ? 'xml' : fmt);
        document.body.appendChild(a);
        a.click();
        a.remove();
        URL.revokeObjectURL(url);
    } catch (e) {
        alert('Error: ' + e.message);
    }
}

function showNewTaskForm() {
    if (!currentProjectId) return;

    const modal = document.getElementById('project-modal');
    modal.innerHTML = `
        <div class="modal" style="display:flex">
            <div class="modal-backdrop" onclick="closeNewTaskForm()"></div>
            <div class="modal-content" style="max-width:480px;padding:1.5rem">
                <h3>New Task</h3>
                <div class="form-group">
                    <label>Task Name</label>
                    <input type="text" id="new-task-name" class="form-input" placeholder="Enter task name" />
                </div>
                <div class="form-group">
                    <label>Start Date</label>
                    <input type="date" id="new-task-start" class="form-input" />
                </div>
                <div class="form-group">
                    <label>End Date</label>
                    <input type="date" id="new-task-end" class="form-input" />
                </div>
                <div class="form-actions" style="display:flex;gap:0.5rem;margin-top:1rem">
                    <button class="btn-cancel" onclick="closeNewTaskForm()">Cancel</button>
                    <button class="btn-primary" onclick="createTask()">Create Task</button>
                </div>
            </div>
        </div>
    `;
}

function closeNewTaskForm() {
    document.getElementById('project-modal').innerHTML = '';
}

function closeProjectModal() {
    document.getElementById('project-modal').innerHTML = '';
}

async function createProjectFromForm() {
    const name = document.getElementById('new-project-name').value.trim();
    if (!name) { alert('Please enter a project name'); return; }

    const start = document.getElementById('new-project-start').value
        || new Date().toISOString().slice(0, 10);
    const end = document.getElementById('new-project-end').value || null;

    try {
        const resp = await fetch('/api/projects', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                name: name,
                description: document.getElementById('new-project-description').value.trim() || null,
                start_date: start,
                end_date: end
            })
        });
        if (resp.ok) {
            closeProjectModal();
            loadProjectList();
        } else {
            alert('Failed to create project: ' + resp.status);
        }
    } catch (e) {
        alert('Error: ' + e.message);
    }
}

async function createTask() {
    const name = document.getElementById('new-task-name').value.trim();
    if (!name) { alert('Please enter a task name'); return; }

    const start = document.getElementById('new-task-start').value
        || new Date().toISOString().slice(0, 10);
    const end = document.getElementById('new-task-end').value;
    let duration_days = 1;
    if (end) {
        const ms = new Date(end) - new Date(start);
        if (ms > 0) duration_days = Math.max(1, Math.round(ms / 86400000));
    }

    try {
        // CreateTaskRequest expects { name, start_date, duration_days } —
        // end_date is derived server-side (start_date + duration_days) and
        // resource assignment is a separate endpoint.
        const resp = await fetch(`/api/projects/${currentProjectId}/tasks`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                name: name,
                start_date: start,
                duration_days: duration_days
            })
        });
        if (resp.ok) {
            closeNewTaskForm();
            loadProjectData(currentProjectId);
        } else {
            alert('Failed to create task');
        }
    } catch (e) {
        alert('Error: ' + e.message);
    }
}

async function loadProjectList(query) {
    const container = document.getElementById('project-list');
    if (!container) return;

    try {
        const url = query ? `/api/projects?q=${encodeURIComponent(query)}` : '/api/projects';
        const resp = await fetch(url);
        if (!resp.ok) {
            container.innerHTML = '<div class="empty-state-inline">Failed to load projects</div>';
            return;
        }
        const projects = await resp.json();
        if (!projects || projects.length === 0) {
            container.innerHTML = '<div class="empty-state-inline">No projects found</div>';
            return;
        }
        let html = '';
        for (const p of projects) {
            html += `
                <div class="project-list-item" onclick="selectProject('${p.id}')">
                    <div class="project-item-name">${p.name || 'Unnamed'}</div>
                    <div class="project-item-meta">${p.status || 'Active'}${p.organization_name ? ' · ' + p.organization_name : ''}</div>
                </div>
            `;
        }
        container.innerHTML = html;
    } catch (e) {
        container.innerHTML = '<div class="empty-state-inline">Error loading projects</div>';
    }
}

(function(){ var __cb = function() {
    renderGantt(currentTasks, currentZoom);
    loadProjectList();
    // Deep-link support: open contextualized via app://project?project_id=...
    const params = window.__gbAppParams__ || {};
    if (params.project_id) {
        const target = params.project_id;
        let tries = 0;
        const tryOpen = () => {
            if (typeof selectProject === "function") { selectProject(target); }
            if (tries++ < 15) { setTimeout(tryOpen, 500); }
        };
        setTimeout(tryOpen, 400);
    }
}; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
