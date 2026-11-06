function openModal(modalId) {
    document.getElementById(modalId).classList.add('active');
    document.body.style.overflow = 'hidden';
}

function closeModal(modalId) {
    document.getElementById(modalId).classList.remove('active');
    document.body.style.overflow = 'auto';
}

// Close modal on overlay click
document.querySelectorAll('.modal-overlay').forEach(overlay => {
    overlay.addEventListener('click', (e) => {
        if (e.target === overlay) {
            overlay.classList.remove('active');
            document.body.style.overflow = 'auto';
        }
    });
});

// Close modal on Escape key
document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
        document.querySelectorAll('.modal-overlay.active').forEach(modal => {
            modal.classList.remove('active');
        });
        document.body.style.overflow = 'auto';
    }
});

async function submitRequest(event, type) {
    event.preventDefault();

    const form = event.target;
    const formData = new FormData(form);
    const data = Object.fromEntries(formData.entries());
    data.request_type = type;

    try {
        const response = await fetch('/api/privacy/request', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(data)
        });

        if (response.ok) {
            alert('✅ Your request has been submitted successfully! Check your email for confirmation.');
            form.reset();
            closeModal(form.closest('.modal-overlay').id);
            refreshStatus();
        } else {
            throw new Error('Request failed');
        }
    } catch (error) {
        alert('❌ There was an error submitting your request. Please try again or contact support.');
    }
}

async function refreshStatus() {
    try {
        const response = await fetch('/api/privacy/requests');
        if (response.ok) {
            const requests = await response.json();
            updateStatusList(requests);
        }
    } catch (error) {
        console.error('Failed to refresh status:', error);
    }
}

function updateStatusList(requests) {
    const statusList = document.getElementById('statusList');
    if (requests.length === 0) {
        statusList.innerHTML = '<p style="padding: 1rem; color: var(--text-secondary);">No requests found.</p>';
        return;
    }

    statusList.innerHTML = requests.map(req => `
        <div class="status-item">
            <span class="status-badge status-${req.status.toLowerCase()}">${req.status}</span>
            <span>${req.type}</span>
            <span>${new Date(req.created_at).toLocaleDateString()}</span>
            ${req.download_url ?
                `<a href="${req.download_url}" class="btn btn-secondary" style="padding: 0.5rem 1rem; font-size: 0.875rem;">Download</a>` :
                `<span>${req.status}</span>`
            }
        </div>
    `).join('');
}

// Load status on page load
document.addEventListener('DOMContentLoaded', refreshStatus);
