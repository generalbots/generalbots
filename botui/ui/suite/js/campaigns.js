/**
 * Campaigns JS Controller (Issue #531)
 * Gerencia a interface de campanhas de marketing multicanal.
 * Comunica-se com a API REST /api/crm/campaigns e /api/instagram/campaigns.
 */

(function() {
'use strict';

const CampaignsAPI = {
    baseUrl: '/api/crm/campaigns',
    instagramUrl: '/api/instagram/campaigns',

    async list() {
        const resp = await fetch(this.baseUrl);
        if (!resp.ok) throw new Error(`Failed to list campaigns: ${resp.status}`);
        return resp.json();
    },

    async create(data) {
        const resp = await fetch(this.baseUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data),
        });
        if (!resp.ok) throw new Error(`Failed to create campaign: ${resp.status}`);
        return resp.json();
    },

    async get(id) {
        const resp = await fetch(`${this.baseUrl}/${id}`);
        if (!resp.ok) throw new Error(`Failed to get campaign: ${resp.status}`);
        return resp.json();
    },

    async update(id, data) {
        const resp = await fetch(`${this.baseUrl}/${id}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data),
        });
        if (!resp.ok) throw new Error(`Failed to update campaign: ${resp.status}`);
        return resp.json();
    },

    async delete(id) {
        const resp = await fetch(`${this.baseUrl}/${id}`, { method: 'DELETE' });
        if (!resp.ok) throw new Error(`Failed to delete campaign: ${resp.status}`);
    },

    async createInstagramCampaign(prompt, numImages, schedule) {
        const resp = await fetch(`${this.instagramUrl}/create`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ prompt, num_images: numImages, scheduled_at: schedule }),
        });
        if (!resp.ok) throw new Error(`Failed to create Instagram campaign: ${resp.status}`);
        return resp.json();
    },
};

async function loadCampaigns() {
    const container = document.getElementById('campaignsList');
    if (!container) return;

    try {
        container.innerHTML = '<div class="pipeline-column" style="grid-column:1/-1;"><div style="padding:40px;text-align:center;color:var(--text-secondary);">Loading campaigns...</div></div>';
        const campaigns = await CampaignsAPI.list();
        renderCampaignsList(campaigns);
    } catch (err) {
        console.error('loadCampaigns error:', err);
        container.innerHTML = `<div class="pipeline-column" style="grid-column:1/-1;"><div style="padding:40px;text-align:center;color:var(--danger,#ef4444);">Failed to load campaigns: ${err.message}</div></div>`;
    }
}

function renderCampaignsList(campaigns) {
    const container = document.getElementById('campaignsList');
    if (!container) return;

    if (!campaigns || campaigns.length === 0) {
        container.innerHTML = `
            <div class="pipeline-column" style="grid-column:1/-1;">
                <div style="padding:60px;text-align:center;color:var(--text-secondary,#888);">
                    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="margin-bottom:16px;opacity:0.5;">
                        <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
                    </svg>
                    <h3 style="margin:0 0 8px;font-size:18px;font-weight:600;">No campaigns yet</h3>
                    <p style="margin:0 0 20px;font-size:14px;">Create your first marketing campaign to get started.</p>
                    <button class="btn-primary" onclick="showCampaignModal()">Create Campaign</button>
                </div>
            </div>`;
        return;
    }

    let html = '';
    for (const c of campaigns) {
        const statusClass = (c.status || 'draft').toLowerCase();
        const channels = c.channels || [c.channel || 'email'];
        const metrics = c.metrics || { sent: 0, opened: 0, clicked: 0 };

        html += `
            <div class="campaign-card" data-campaign-id="${c.id || ''}">
                <div class="campaign-card-header">
                    <h3 class="campaign-card-title">${escapeHtml(c.name || 'Untitled')}</h3>
                    <span class="campaign-status ${statusClass}">${statusClass}</span>
                </div>
                <div class="campaign-channels">
                    ${channels.map(ch => `<span class="campaign-channel-tag">${escapeHtml(ch)}</span>`).join('')}
                </div>
                ${c.description ? `<p style="font-size:13px;color:var(--text-secondary,#888);margin:0 0 12px;">${escapeHtml(c.description)}</p>` : ''}
                <div class="campaign-metrics">
                    <div class="campaign-metric">
                        <span class="campaign-metric-value">${metrics.sent || 0}</span>
                        <span class="campaign-metric-label">Sent</span>
                    </div>
                    <div class="campaign-metric">
                        <span class="campaign-metric-value">${metrics.opened || 0}</span>
                        <span class="campaign-metric-label">Opened</span>
                    </div>
                    <div class="campaign-metric">
                        <span class="campaign-metric-value">${metrics.clicked || 0}</span>
                        <span class="campaign-metric-label">Clicked</span>
                    </div>
                </div>
                <div class="campaign-actions">
                    <button class="campaign-action-btn" onclick="showCampaignDetail('${c.id || ''}')">View</button>
                    <button class="campaign-action-btn" onclick="editCampaign('${c.id || ''}')">Edit</button>
                    <button class="campaign-action-btn primary" onclick="runCampaign('${c.id || ''}')">Run</button>
                    <button class="campaign-action-btn" onclick="deleteCampaign('${c.id || ''}')" style="color:var(--danger,#ef4444);">Delete</button>
                </div>
            </div>`;
    }
    container.innerHTML = html;
}

async function showCampaignDetail(campaignId) {
    if (!campaignId) return;
    try {
        const campaign = await CampaignsAPI.get(campaignId);
        alert(`Campaign: ${campaign.name}\nStatus: ${campaign.status}\nChannel: ${campaign.channel}\nCreated: ${campaign.created_at || 'N/A'}`);
    } catch (err) {
        console.error('showCampaignDetail error:', err);
        alert('Failed to load campaign details.');
    }
}

async function createCampaign(prompt, numImages, schedule) {
    try {
        const result = await CampaignsAPI.createInstagramCampaign(prompt, numImages, schedule);
        alert(`Instagram campaign created! ID: ${result.id}`);
        loadCampaigns();
    } catch (err) {
        console.error('createCampaign error:', err);
        alert(`Failed to create campaign: ${err.message}`);
    }
}

function showCampaignModal(campaignId) {
    const modal = document.getElementById('campaign-modal');
    if (!modal) return;
    const title = document.getElementById('campaign-modal-title');
    if (title) title.textContent = campaignId ? 'Edit Campaign' : 'Create Campaign';
    modal.style.display = 'flex';
    document.body.style.overflow = 'hidden';

    if (campaignId) {
        CampaignsAPI.get(campaignId).then(c => {
            const nameInput = document.getElementById('campaign-name');
            const channelSelect = document.getElementById('campaign-channel');
            const budgetInput = document.getElementById('campaign-budget');
            const scheduleInput = document.getElementById('campaign-schedule');
            if (nameInput) nameInput.value = c.name || '';
            if (channelSelect) channelSelect.value = c.channel || 'email';
            if (budgetInput) budgetInput.value = c.budget || '';
            if (scheduleInput) scheduleInput.value = c.scheduled_at ? c.scheduled_at.slice(0, 16) : '';
        }).catch(err => console.error('Failed to load campaign for edit:', err));
    }
}

function hideCampaignModal() {
    const modal = document.getElementById('campaign-modal');
    if (!modal) return;
    modal.style.display = 'none';
    document.body.style.overflow = '';
    const form = document.getElementById('campaign-form');
    if (form) form.reset();
}

async function editCampaign(campaignId) {
    showCampaignModal(campaignId);
}

async function runCampaign(campaignId) {
    if (!confirm('Run this campaign now?')) return;
    try {
        await CampaignsAPI.update(campaignId, { status: 'running' });
        alert('Campaign is now running!');
        loadCampaigns();
    } catch (err) {
        alert(`Failed to run campaign: ${err.message}`);
    }
}

async function deleteCampaign(campaignId) {
    if (!confirm('Delete this campaign permanently?')) return;
    try {
        await CampaignsAPI.delete(campaignId);
        loadCampaigns();
    } catch (err) {
        alert(`Failed to delete campaign: ${err.message}`);
    }
}

function filterCampaigns(view) {
    const cards = document.querySelectorAll('.campaign-card');
    cards.forEach(card => {
        if (view === 'all') {
            card.style.display = '';
        } else {
            const channels = card.querySelector('.campaign-channels');
            if (channels && channels.textContent.toLowerCase().includes(view)) {
                card.style.display = '';
            } else {
                card.style.display = 'none';
            }
        }
    });
}

function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

// Register HTMX event handlers
document.addEventListener('DOMContentLoaded', function() {
    // Auto-load campaigns when campaigns view becomes active
    const campaignsView = document.getElementById('campaigns-view');
    if (campaignsView) {
        const observer = new MutationObserver(() => {
            if (campaignsView.classList.contains('active')) {
                loadCampaigns();
            }
        });
        observer.observe(campaignsView, { attributes: true, attributeFilter: ['class'] });
    }

    // Handle campaign form submission
    const form = document.getElementById('campaign-form');
    if (form) {
        form.addEventListener('submit', async function(e) {
            e.preventDefault();
            const formData = new FormData(form);
            const data = Object.fromEntries(formData.entries());
            try {
                await CampaignsAPI.create(data);
                hideCampaignModal();
                loadCampaigns();
            } catch (err) {
                alert(`Failed to create campaign: ${err.message}`);
            }
        });
    }
});

// Make functions globally accessible for HTML onclick handlers
window.CampaignsAPI = CampaignsAPI;
window.loadCampaigns = loadCampaigns;
window.renderCampaignsList = renderCampaignsList;
window.showCampaignDetail = showCampaignDetail;
window.createCampaign = createCampaign;
window.showCampaignModal = showCampaignModal;
window.hideCampaignModal = hideCampaignModal;
window.editCampaign = editCampaign;
window.runCampaign = runCampaign;
window.deleteCampaign = deleteCampaign;
window.filterCampaigns = filterCampaigns;

})();
