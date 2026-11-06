// Modal functions
function openModal(id) {
    document.getElementById(id).classList.add('active');
}

function closeModal(id) {
    document.getElementById(id).classList.remove('active');
}

// Toast notifications
function showToast(message, type = 'success') {
    const container = document.getElementById('toast-container');
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    container.appendChild(toast);
    setTimeout(() => toast.remove(), 3000);
}

// HTMX event handlers
document.body.addEventListener('htmx:responseError', function(e) {
    showToast('An error occurred: ' + (e.detail.xhr.statusText || 'Unknown error'), 'error');
});

document.body.addEventListener('htmx:afterSwap', function(e) {
    // Re-initialize any dynamic elements after swap
});

// Edit item
function editItem(id) {
    // The server substitutes ${table} in layout.html; the data-table attribute on
    // this script tag carries that value into the external file.
    const tag = document.querySelector('script[data-table]');
    const table = tag ? tag.dataset.table : '';
    htmx.ajax('GET', `/api/data/${table}/${id}`, {
        target: '#edit-form',
        swap: 'innerHTML'
    }).then(() => openModal('edit-modal'));
}

// Delete item
function deleteItem(id) {
    if (confirm('Are you sure you want to delete this item?')) {
        const tag = document.querySelector('script[data-table]');
        const table = tag ? tag.dataset.table : '';
        htmx.ajax('DELETE', `/api/data/${table}/${id}`, {
            target: '#data-list',
            swap: 'innerHTML'
        }).then(() => showToast('Item deleted'));
    }
}
