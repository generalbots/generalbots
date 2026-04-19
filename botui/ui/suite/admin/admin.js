/* Admin Module JavaScript */

/**
 * Set active navigation item in admin sidebar
 * @param {HTMLElement} el - The clicked navigation element
 */
function setActiveNav(el) {
    document.querySelectorAll('.admin-nav .nav-item').forEach(item => {
        item.classList.remove('active');
    });
    el.classList.add('active');
}

/**
 * Toggle mobile sidebar visibility
 */
function toggleAdminSidebar() {
    const sidebar = document.querySelector('.admin-sidebar');
    if (sidebar) {
        sidebar.classList.toggle('open');
    }
}

/**
 * Load dashboard template as fallback when HTMX fails
 */
document.addEventListener('htmx:responseError', function(e) {
    if (e.detail.target.id === 'admin-content') {
        const template = document.getElementById('dashboard-template');
        if (template) {
            e.detail.target.innerHTML = template.innerHTML;
        }
    }
});

/**
 * Close mobile sidebar when clicking outside
 */
document.addEventListener('click', function(e) {
    const sidebar = document.querySelector('.admin-sidebar');
    const menuToggle = document.querySelector('.admin-menu-toggle');
    
    if (sidebar && sidebar.classList.contains('open')) {
        if (!sidebar.contains(e.target) && !menuToggle?.contains(e.target)) {
            sidebar.classList.remove('open');
        }
    }
});

/**
 * Handle keyboard navigation for admin panel
 */
document.addEventListener('keydown', function(e) {
    // Close sidebar on Escape
    if (e.key === 'Escape') {
        const sidebar = document.querySelector('.admin-sidebar');
        if (sidebar && sidebar.classList.contains('open')) {
            sidebar.classList.remove('open');
        }
        
        // Also close any open modals
        document.querySelectorAll('.modal[open]').forEach(modal => {
            modal.close();
        });
    }
});

/**
 * Confirm dangerous actions
 * @param {string} message - Confirmation message
 * @returns {boolean} - User's confirmation choice
 */
function confirmAction(message) {
    return confirm(message || 'Are you sure you want to perform this action?');
}

/**
 * Format bytes to human readable string
 * @param {number} bytes - Number of bytes
 * @returns {string} - Formatted string (e.g., "1.5 GB")
 */
function formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * Format date to relative time
 * @param {string|Date} date - Date to format
 * @returns {string} - Relative time string
 */
function formatRelativeTime(date) {
    const now = new Date();
    const past = new Date(date);
    const diffMs = now - past;
    const diffSec = Math.floor(diffMs / 1000);
    const diffMin = Math.floor(diffSec / 60);
    const diffHour = Math.floor(diffMin / 60);
    const diffDay = Math.floor(diffHour / 24);
    
    if (diffSec < 60) return 'just now';
    if (diffMin < 60) return `${diffMin}m ago`;
    if (diffHour < 24) return `${diffHour}h ago`;
    if (diffDay < 7) return `${diffDay}d ago`;
    
    return past.toLocaleDateString();
}
