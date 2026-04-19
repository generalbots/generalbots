/* Editor page JavaScript */

// Show save notification
function showSaveNotification(event) {
    const notification = document.getElementById('notification');
    if (event.detail.successful) {
        notification.textContent = '✓ File saved';
        notification.className = 'notification success show';
        document.getElementById('dirty-indicator').style.display = 'none';
    } else {
        notification.textContent = '✗ Save failed';
        notification.className = 'notification error show';
    }
    setTimeout(() => notification.classList.remove('show'), 3000);
}

// Mark as dirty on edit
document.getElementById('text-editor')?.addEventListener('input', function() {
    document.getElementById('dirty-indicator').style.display = 'inline-block';
});

// Keyboard shortcuts using htmx triggers
document.addEventListener('keydown', function(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        htmx.trigger(document.querySelector('[hx-post="/api/editor/save"]'), 'click');
    }
});
