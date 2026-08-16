let currentView = 'gantt';
let currentZoom = 'week';
let currentProjectId = null;
let currentProjectData = null;
const taskNameCache = {};

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

    try {
        const resp = await fetch(`/api/projects/${projectId}`);
        if (resp.ok) {
            currentProjectData = await resp.json();
        }

        const tasksResp = await fetch(`/api/projects/${projectId}/tasks`);
        let tasks = [];
        if (tasksResp.ok) tasks = await tasksResp.json();

        tasks.forEach((t) => { taskNameCache[t.id] = t.name || t.title || t.id; });

        renderProjectHeader(currentProjectData, tasks);
        renderGanttTable(tasks);
        renderGanttChart(tasks);
        renderListView(tasks);
        renderBoardView(tasks);
        bindProjectCommentsBadge();
    } catch (e) {
        console.error('Failed to load project data:', e);
    }
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
                <div class="col-assignee">${task.assignee || '-'} <button class="row-comment-btn" onclick="event.stopPropagation(); openProjectTaskComments('${task.id}')" title="Comments">💬</button></div>
            </div>
        `;
    }
    body.innerHTML = html;
}

function renderGanttChart(tasks) {
    const body = document.getElementById('gantt-chart-body');
    if (!body) return;

    if (!tasks || tasks.length === 0) {
        body.innerHTML = '<div class="empty-state-inline"><p>No tasks to display</p></div>';
        return;
    }

    generateTimelineHeaders();

    let html = '';
    for (const task of tasks) {
        const pct = task.percent_complete || task.percentComplete || 0;
        const isCritical = task.is_critical ? 'critical' : '';
        html += `
            <div class="gantt-bar-row">
                <div class="gantt-bar ${isCritical}" style="width:${Math.max(pct, 20)}%; min-width:60px;">
                    <span>${task.name || 'Task'}</span>
                </div>
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
                <td>${task.assignee || '-'}</td>
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
                <div class="card-meta">${task.assignee ? '👤 ' + task.assignee : ''}</div>
                <div class="card-progress">${task.percent_complete || task.percentComplete || 0}%</div>
            </div>
        `;
    }
    container.innerHTML = html;
}

function showTaskDetail(taskId) {
    const panel = document.getElementById('task-details');
    if (!panel) return;

    const tasks = document.querySelectorAll('.gantt-table-row[onclick]');
    panel.innerHTML = `<p class="empty-message">Task details would appear here (ID: ${taskId})</p>`;
    document.getElementById('details-panel').classList.remove('collapsed');
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
                    <label>Assignee</label>
                    <input type="text" id="new-task-assignee" class="form-input" placeholder="Assignee name" />
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

async function createTask() {
    const name = document.getElementById('new-task-name').value.trim();
    if (!name) { alert('Please enter a task name'); return; }

    try {
        const resp = await fetch(`/api/projects/${currentProjectId}/tasks`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                name: name,
                assignee: document.getElementById('new-task-assignee').value.trim() || null,
                start_date: document.getElementById('new-task-start').value || null,
                end_date: document.getElementById('new-task-end').value || null
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
    generateTimelineHeaders();
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

function generateTimelineHeaders() {
    const header = document.getElementById('gantt-timeline-header');
    if (!header) return;

    const today = new Date();
    let html = '';

    for (let i = 0; i < 30; i++) {
        const date = new Date(today);
        date.setDate(date.getDate() + i);
        const day = date.getDate();
        const dayName = date.toLocaleDateString('en-US', { weekday: 'short' });
        const isWeekend = date.getDay() === 0 || date.getDay() === 6;

        html += `
            <div class="timeline-day ${isWeekend ? 'weekend' : ''}" style="width: 40px; text-align: center; padding: 0.5rem 0; border-right: 1px solid var(--border-color);">
                <div style="font-size: 0.625rem; color: var(--text-muted);">${dayName}</div>
                <div style="font-size: 0.75rem; font-weight: 600;">${day}</div>
            </div>
        `;
    }

    header.innerHTML = html;
}
