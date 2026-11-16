/* Split from project-state.js for #2305 (450-line limit). */
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
            projectAlert(projectT('project-dependency-add-failed', 'Failed to add dependency'));
        }
    } catch (e) {
        projectAlert(projectT('project-error', 'Error: ' + e.message, { error: e.message }));
    }
}
