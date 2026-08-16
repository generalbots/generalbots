// Top-level `var` (not `let`/`const`) — the window manager re-injects app
// scripts every time the Projects window opens, and `let`/`const` throw
// "Identifier has already been declared" on the second injection.
var currentView = 'gantt';
var currentZoom = 'week';
var currentProjectId = null;
var currentProjectData = null;
var taskNameCache = {};
var currentTasks = [];
var resourceById = {};

function switchView(view) {
    currentView = view;

    document.querySelectorAll('.view-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.view === view);
    });

    document.querySelectorAll('.view-container').forEach(container => {
        container.classList.toggle('active', container.id === `${view}-view`);
    });
}

function zoomGantt(level) {
    currentZoom = level;

    document.querySelectorAll('.zoom-btn').forEach(btn => {
        btn.classList.toggle('active', btn.textContent.toLowerCase() === level);
    });

    renderGantt(currentTasks, currentZoom);
}

function toggleCriticalPath() {
    const show = document.getElementById('show-critical').checked;
    document.querySelectorAll('.gantt-bar.critical').forEach(bar => {
        bar.style.display = show ? '' : 'none';
    });
}

function toggleMilestones() {
    const show = document.getElementById('show-milestones').checked;
    document.querySelectorAll('.gantt-milestone').forEach(ms => {
        ms.style.display = show ? '' : 'none';
    });
}

function fitGanttToScreen() {
    const container = document.querySelector('.gantt-chart');
    if (container) {
        container.scrollLeft = 0;
    }
}

function toggleDetailsPanel() {
    const panel = document.getElementById('details-panel');
    panel.classList.toggle('collapsed');
}

async function loadProjectData(projectId) {
    currentProjectId = projectId;
    document.querySelector('.project-app').classList.add('has-project');
    document.getElementById('add-task-btn').disabled = false;
    const exportBtn = document.getElementById('project-export-btn');
    if (exportBtn) exportBtn.disabled = false;

    try {
        const resp = await fetch(`/api/projects/${projectId}`);
        if (resp.ok) {
            currentProjectData = await resp.json();
        }

        const tasksResp = await fetch(`/api/projects/${projectId}/tasks`);
        let tasks = [];
        if (tasksResp.ok) tasks = await tasksResp.json();

        const resResp = await fetch(`/api/projects/${projectId}/resources`);
        let resources = [];
        if (resResp.ok) resources = await resResp.json();
        resourceById = {};
        resources.forEach((r) => { resourceById[r.id] = r.name || r.email || r.id; });

        tasks.forEach((t) => { taskNameCache[t.id] = t.name || t.title || t.id; });
        currentTasks = tasks;

        renderProjectHeader(currentProjectData, tasks);
        renderGanttTable(tasks);
        renderGantt(tasks, currentZoom);
        renderListView(tasks);
        renderBoardView(tasks);
        renderTimelineView(tasks);
        renderResourceView();
        bindProjectCommentsBadge();
    } catch (e) {
        console.error('Failed to load project data:', e);
    }
}

/* Resolve a task's assigned_to (Vec<Uuid>) into resource names. */
function assigneeNames(task) {
    const ids = task.assigned_to || task.assignedTo || [];
    if (!ids || !ids.length) return '-';
    const names = ids.map((id) => resourceById[id] || id).filter(Boolean);
    return names.length ? names.join(', ') : '-';
}

async function updateProjectStatus(status) {
    if (!currentProjectId || !status) return;
    try {
        const resp = await fetch(`/api/projects/${currentProjectId}/status`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ status })
        });
        if (resp.ok) {
            loadProjectData(currentProjectId);
        }
    } catch (e) {
        console.error('Failed to update project status:', e);
    }
}

function openProjectComments() {
    if (!currentProjectId) return;
    const name = (currentProjectData && (currentProjectData.name || currentProjectData.title)) || 'Project';
    if (typeof GBCollabComments === 'undefined') return;
    GBCollabComments.open({
        resourceType: 'project',
        resourceId: String(currentProjectId),
        includeChildren: true,
        title: 'Comments on ' + name
    });
}

function openProjectTaskComments(taskId) {
    if (typeof GBCollabComments === 'undefined') return;
    const name = taskNameCache[taskId] || 'Task';
    GBCollabComments.open({
        resourceType: 'project:task',
        resourceId: String(taskId),
        title: 'Comments on ' + name
    });
}

function bindProjectCommentsBadge() {
    const btn = document.getElementById('projectCommentsBtn');
    if (!btn || !currentProjectId) return;
    btn.disabled = false;
    if (typeof GBCollabComments === 'undefined') return;
    GBCollabComments.bindBadge(btn, {
        resourceType: 'project',
        resourceId: String(currentProjectId),
        includeChildren: true
    });
}

function selectProject(projectId) {
    loadProjectData(projectId);
}

function renderProjectHeader(project, tasks) {
    const nameEl = document.getElementById('project-name');
    if (project) nameEl.textContent = project.name || 'Unnamed Project';

    const statusEl = document.getElementById('project-status');
    if (statusEl) {
        statusEl.querySelector('span:last-child').textContent = project ? (project.status || 'Active') : 'No project selected';
    }

    const statusSelect = document.getElementById('project-status-select');
    if (statusSelect) {
        if (project) {
            statusSelect.value = (project.status || 'active').toLowerCase();
            statusSelect.style.display = '';
        } else {
            statusSelect.style.display = 'none';
        }
    }

    const progressEl = document.getElementById('project-progress');
    if (progressEl && tasks && tasks.length > 0) {
        const completed = tasks.filter(t => t.status === 'Completed' || t.percent_complete === 100).length;
        const pct = Math.round((completed / tasks.length) * 100);
        progressEl.querySelector('.progress-value').textContent = `${pct}%`;
    } else if (progressEl) {
        progressEl.querySelector('.progress-value').textContent = '--';
    }
}

function renderGanttTable(tasks) {
    const body = document.getElementById('gantt-table-body');
    if (!body) return;

    if (!tasks || tasks.length === 0) {
        body.innerHTML = '<div class="empty-state-inline">No tasks in this project</div>';
        return;
    }

    let html = '';
    for (const task of tasks) {
        const statusClass = (task.status || '').toLowerCase().replace(' ', '-');
        html += `
            <div class="gantt-table-row" onclick="showTaskDetail('${task.id}')">
                <div class="col-name">${task.name || ''}</div>
                <div class="col-start">${task.start_date || task.startDate || ''}</div>
                <div class="col-end">${task.end_date || task.endDate || ''}</div>
                <div class="col-duration">${task.duration || '-'}</div>
                <div class="col-progress"><div class="progress-bar"><div class="progress-fill" style="width:${task.percent_complete || task.percentComplete || 0}%"></div></div></div>
                <div class="col-assignee">${assigneeNames(task)} <button class="row-comment-btn" onclick="event.stopPropagation(); openProjectTaskComments('${task.id}')" title="Comments">💬</button></div>
            </div>
        `;
    }
    body.innerHTML = html;
}



function renderListView(tasks) {
    const container = document.getElementById('list-container');
    if (!container) return;

    if (!tasks || tasks.length === 0) {
        container.innerHTML = '<div class="empty-state-inline">No tasks found</div>';
        return;
    }

    let html = '<table class="task-list-table"><thead><tr><th>Name</th><th>Status</th><th>Assignee</th><th>Progress</th></tr></thead><tbody>';
    for (const task of tasks) {
        html += `
            <tr onclick="showTaskDetail('${task.id}')">
                <td>${task.name || ''}</td>
                <td><span class="status-badge status-${(task.status || 'not-started').toLowerCase().replace(' ', '-')}">${task.status || 'Not Started'}</span></td>
                <td>${assigneeNames(task)}</td>
                <td>${task.percent_complete || task.percentComplete || 0}%</td>
                <td><button class="row-comment-btn" onclick="event.stopPropagation(); openProjectTaskComments('${task.id}')" title="Comments">💬</button></td>
            </tr>
        `;
    }
    html += '</tbody></table>';
    container.innerHTML = html;
}

function renderBoardView(tasks) {
    if (!tasks) tasks = [];

    const notStarted = tasks.filter(t => !t.status || t.status === 'Not Started' || t.status === 'not_started');
    const inProgress = tasks.filter(t => t.status === 'In Progress' || t.status === 'in_progress');
    const completed = tasks.filter(t => t.status === 'Completed' || t.percent_complete === 100 || t.percentComplete === 100);

    renderBoardColumn('board-tasks-not-started', notStarted);
    renderBoardColumn('board-tasks-in-progress', inProgress);
    renderBoardColumn('board-tasks-completed', completed);
}

function renderBoardColumn(containerId, tasks) {
    const container = document.getElementById(containerId);
    if (!container) return;

    if (!tasks || tasks.length === 0) {
        container.innerHTML = '<div class="empty-card"><p>No tasks</p></div>';
        return;
    }

    let html = '';
    for (const task of tasks) {
        html += `
            <div class="board-card" onclick="showTaskDetail('${task.id}')" draggable="true">
                <div class="card-title">${task.name || ''} <button class="row-comment-btn" onclick="event.stopPropagation(); openProjectTaskComments('${task.id}')" title="Comments">💬</button></div>
                <div class="card-meta">${assigneeNames(task) !== '-' ? '👤 ' + assigneeNames(task) : ''}</div>
                <div class="card-progress">${task.percent_complete || task.percentComplete || 0}%</div>
            </div>
        `;
    }
    container.innerHTML = html;
}

function renderTimelineView(tasks) {
    const container = document.getElementById('timeline-container');
    if (!container) return;

    if (!tasks || tasks.length === 0) {
        container.innerHTML = '<div class="empty-state-inline">No tasks in this project</div>';
        return;
    }

    const sorted = tasks.slice().sort((a, b) =>
        (a.start_date || a.startDate || '').localeCompare(b.start_date || b.startDate || '')
    );
    let html = '<div class="timeline-list">';
    for (const task of sorted) {
        const pct = task.percent_complete || task.percentComplete || 0;
        const statusLabel = pct === 100 ? 'Completed' : pct > 0 ? 'In Progress' : 'Not Started';
        html += `
            <div class="timeline-row" onclick="showTaskDetail('${task.id}')">
                <span class="timeline-date">${task.start_date || task.startDate || ''}</span>
                <span class="timeline-name">${task.name || ''}</span>
                <span class="timeline-end">&rarr; ${task.end_date || task.endDate || ''}</span>
                <span class="status-badge status-${statusLabel.toLowerCase().replace(' ', '-')}">${statusLabel}</span>
            </div>
        `;
    }
    html += '</div>';
    container.innerHTML = html;
}

async function renderResourceView() {
    const container = document.getElementById('resource-container');
    if (!container) return;
    if (!currentProjectId) {
        container.innerHTML = '<div class="empty-state-inline">Select a project to view resources</div>';
        return;
    }
    try {
        const resp = await fetch(`/api/projects/${currentProjectId}/resources`);
        if (!resp.ok) {
            container.innerHTML = '<div class="empty-state-inline">Failed to load resources</div>';
            return;
        }
        const resources = await resp.json();
        if (!resources || resources.length === 0) {
            container.innerHTML = '<div class="empty-state-inline">No resources assigned to this project</div>';
            return;
        }
        let html = '<table class="task-list-table resource-table"><thead><tr><th>Name</th><th>Type</th><th>Email</th><th>Max Units</th><th>Rate</th></tr></thead><tbody>';
        for (const r of resources) {
            const type = (r.resource_type && typeof r.resource_type === 'string') ? r.resource_type : 'work';
            const rate = r.standard_rate != null ? r.standard_rate : '-';
            const units = r.max_units != null ? `${r.max_units}%` : '100%';
            html += `
                <tr>
                    <td>${r.name || ''}</td>
                    <td>${type}</td>
                    <td>${r.email || '-'}</td>
                    <td>${units}</td>
                    <td>${rate}</td>
                </tr>
            `;
        }
        html += '</tbody></table>';
        container.innerHTML = html;
    } catch (e) {
        container.innerHTML = '<div class="empty-state-inline">Error loading resources</div>';
    }
}

function showTaskDetail(taskId) {
    const panel = document.getElementById('task-details');
    if (!panel) return;

    const task = currentTasks.find((t) => t.id === taskId);
    if (!task) {
        panel.innerHTML = '<p class="empty-message">Task not found</p>';
        return;
    }

    const pct = task.percent_complete || task.percentComplete || 0;
    const deps = task.dependencies || [];
    const otherTasks = currentTasks.filter((t) => t.id !== taskId);

    let depHtml = '';
    if (!deps.length) {
        depHtml = '<p class="empty-message">No dependencies. Link a predecessor below.</p>';
    } else {
        depHtml = '<ul class="dep-list">';
        deps.forEach((d) => {
            const predName = taskNameCache[d.predecessor_id] || d.predecessor_id;
            depHtml += `<li class="dep-item"><span>${predName}</span>` +
                `<span class="dep-type">${d.dependency_type || 'finish_to_start'}</span>` +
                (d.lag_days ? `<span class="dep-lag">lag ${d.lag_days}d</span>` : '') +
                `<button class="row-comment-btn" onclick="removeDependency('${taskId}','${d.predecessor_id}')" title="Remove">✕</button></li>`;
        });
        depHtml += '</ul>';
    }

    const predOptions = otherTasks
        .map((t) => `<option value="${t.id}">${t.name || t.id}</option>`)
        .join('');

    panel.innerHTML = `
        <h4 class="td-title">${task.name || 'Task'}</h4>
        <div class="td-grid">
            <div><label>Start</label><span>${task.start_date || task.startDate || '-'}</span></div>
            <div><label>End</label><span>${task.end_date || task.endDate || '-'}</span></div>
            <div><label>Duration</label><span>${task.duration_days != null ? task.duration_days + 'd' : '-'}</span></div>
            <div><label>Status</label><span>${task.status || '-'}</span></div>
            <div><label>Assignee</label><span>${assigneeNames(task)}</span></div>
        </div>
        <div class="form-group">
            <label>Progress: ${pct}%</label>
            <input type="range" min="0" max="100" value="${pct}"
                oninput="document.getElementById('td-progress-val').textContent = this.value + '%'"
                onchange="updateTaskProgress('${taskId}', this.value)" />
            <span id="td-progress-val" class="td-progress-val">${pct}%</span>
        </div>
        <div class="td-section">
            <h5>Dependencies</h5>
            ${depHtml}
            <div class="dep-add">
                <select id="dep-predecessor" class="form-input">${predOptions}</select>
                <select id="dep-type" class="form-input">
                    <option value="finish_to_start">Finish → Start</option>
                    <option value="start_to_start">Start → Start</option>
                    <option value="finish_to_finish">Finish → Finish</option>
                    <option value="start_to_finish">Start → Finish</option>
                </select>
                <input type="number" id="dep-lag" class="form-input" value="0" step="1" title="Lag days" />
                <button class="btn-primary" onclick="addDependency('${taskId}')" ${predOptions ? '' : 'disabled'}>Link</button>
            </div>
        </div>
    `;
    document.getElementById('details-panel').classList.remove('collapsed');
}

async function addDependency(taskId) {
    const predecessorId = document.getElementById('dep-predecessor').value;
    const dependencyType = document.getElementById('dep-type').value;
    const lagDays = parseInt(document.getElementById('dep-lag').value || '0', 10);
    if (!predecessorId) return;

    try {
        const resp = await fetch(`/api/tasks/${taskId}/dependencies`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ predecessor_id: predecessorId, dependency_type: dependencyType, lag_days: lagDays })
        });
        if (resp.ok) {
            await loadProjectData(currentProjectId);
            showTaskDetail(taskId);
        } else {
            alert('Failed to add dependency');
        }
    } catch (e) {
        alert('Error: ' + e.message);
    }
}

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
