const API = '/api/cloud';
let currentOrgId = null;
let currentWsId = null;
let orgs = [];

const RES_ITEMS = {
  compute: [
    { id: 'vps-small', label: 'VPS Small — $9.99/mo' },
    { id: 'vps-medium', label: 'VPS Medium — $19.99/mo' },
    { id: 'vps-large', label: 'VPS Large — $39.99/mo' },
    { id: 'vps-xl', label: 'VPS XL — $79.99/mo' },
    { id: 'gpu-basic', label: 'GPU Basic — $29.99/mo' },
    { id: 'gpu-pro', label: 'GPU Pro — $99.99/mo' },
  ],
  storage: [
    { id: 'storage-50', label: '50 GB — $9.99/mo' },
    { id: 'storage-250', label: '250 GB — $29.99/mo' },
    { id: 'storage-1tb', label: '1 TB — $59.99/mo' },
    { id: 'storage-10tb', label: '10 TB — $199.99/mo' },
  ],
  phone: [
    { id: 'number-local', label: 'Local Number — $5.99/mo' },
    { id: 'number-global', label: 'Global Bundle (3) — $19.99/mo' },
    { id: 'number-business', label: 'Business Pack (10) — $49.99/mo' },
  ],
};


// ── Org list ──
async function loadOrgs() {
  const token = requireAuth();
  try {
    const res = await fetch(API + '/organizations', { headers: { 'Authorization': 'Bearer ' + token } });
    if (!res.ok) { return renderOrgs(); }
    const data = await res.json();
    orgs = data.organizations || [];
    // Fetch workspace counts per org
    for (const o of orgs) {
      try {
        const wr = await fetch(API + '/organizations/' + o.id + '/workspaces', { headers: { 'Authorization': 'Bearer ' + token } });
        if (!wr.ok) continue;
        const wd = await wr.json();
        o.wsCount = (wd.workspaces || []).length;
        o.resCount = 0;
        for (const ws of (wd.workspaces || [])) {
          try {
            const rr = await fetch(API + '/organizations/' + o.id + '/workspaces/' + ws.id + '/resources', { headers: { 'Authorization': 'Bearer ' + token } });
            if (!rr.ok) continue;
            const rd = await rr.json();
            o.resCount += (rd.resources || []).length;
          } catch (_) {}
        }
      } catch (_) { o.wsCount = 0; o.resCount = 0; }
    }
  } catch (_) { /* API unavailable */ }
  renderOrgs();
}

function renderOrgs() {
  const el = document.getElementById('org-list');
  if (!orgs.length) {
    el.innerHTML = '<div class="mgmt-empty"><div class="mgmt-empty-icon">🏢</div><div>No organizations yet.<br><button class="btn btn-primary btn-sm" onclick="showCreateOrg()" style="margin-top:.75rem">Create your first organization</button></div></div>';
    return;
  }
  el.innerHTML = orgs.map(o => `
    <div class="org-card" onclick="showOrgDetail('${o.id}')">
      <div class="org-card-icon">${(o.name||'O')[0].toUpperCase()}</div>
      <div class="org-card-name">${esc(o.name)}</div>
      <div class="org-card-plan">${o.plan || 'free'} plan</div>
      <div class="org-card-stats">
        <div class="org-card-stat"><div class="org-card-stat-num">${o.wsCount || 0}</div><div class="org-card-stat-label">Workspaces</div></div>
        <div class="org-card-stat"><div class="org-card-stat-num">${o.resCount || 0}</div><div class="org-card-stat-label">Resources</div></div>
      </div>
    </div>
  `).join('');
}

// ── Create org ──
function showCreateOrg() { document.getElementById('create-org-modal').classList.add('open'); document.getElementById('org-name-input').focus(); }
function hideCreateOrg() { document.getElementById('create-org-modal').classList.remove('open'); }

async function createOrg() {
  const name = document.getElementById('org-name-input').value.trim();
  const plan = document.getElementById('org-plan-select').value;
  if (!name) return alert('Enter a name');

  const token = requireAuth();
  const res = await fetch(API + '/organizations', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
    body: JSON.stringify({ name, plan }),
  });
  const data = await res.json();
  if (!res.ok) return alert(data.detail || 'Failed to create');
  hideCreateOrg();
  document.getElementById('org-name-input').value = '';
  await loadOrgs();
}

// ── Edit org ──
function showEditOrg(orgId, name) {
  document.getElementById('edit-org-modal').classList.add('open');
  document.getElementById('edit-org-name-input').value = name;
  document.getElementById('edit-org-name-input').focus();
  document.getElementById('edit-org-modal').setAttribute('data-org-id', orgId);
}
function hideEditOrg() { document.getElementById('edit-org-modal').classList.remove('open'); }

async function updateOrg() {
  const orgId = document.getElementById('edit-org-modal').getAttribute('data-org-id');
  const name = document.getElementById('edit-org-name-input').value.trim();
  if (!name) return alert('Enter a name');
  if (!orgId) return alert('No organization selected');

  const token = requireAuth();
  const res = await fetch(API + '/organizations/' + orgId, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
    body: JSON.stringify({ name }),
  });
  if (!res.ok) { const d = await res.json(); return alert(d.detail || 'Failed to update'); }
  hideEditOrg();
  await loadOrgs();
  // Re-show detail with updated data
  if (currentOrgId === orgId) showOrgDetail(orgId);
}

async function deleteOrg(orgId, name) {
  if (!confirm(`Delete organization "${name}"?\n\nThis will also remove all workspaces and resources. This action cannot be undone.`)) return;
  const token = requireAuth();
  const res = await fetch(API + '/organizations/' + orgId, {
    method: 'DELETE', headers: { 'Authorization': 'Bearer ' + token },
  });
  if (!res.ok) { const d = await res.json(); return alert(d.detail || 'Failed to delete'); }
  if (currentOrgId === orgId) showOrgList();
  await loadOrgs();
}

// ── Org detail ──
async function showOrgDetail(orgId) {
  currentOrgId = orgId;
  const org = orgs.find(o => o.id === orgId) || { name: 'Organization', plan: 'free' };
  const safeName = jesc(org.name);
  document.getElementById('org-detail-header').innerHTML = `
    <div class="org-detail-icon">${(org.name||'O')[0].toUpperCase()}</div>
    <div><div class="org-detail-name">${esc(org.name)}</div><div class="org-detail-plan">${org.plan || 'free'} plan</div>
    <div class="org-detail-meta">ID: ${orgId}</div></div>
    <div class="org-detail-actions">
      <button class="btn btn-secondary btn-sm" onclick="showEditOrg('${orgId}','${safeName}')">Edit</button>
      <button class="btn btn-danger btn-sm" onclick="deleteOrg('${orgId}','${safeName}')">Delete</button>
    </div>
  `;
  document.querySelector('#app-root > .mgmt-topbar').style.display = 'none';
  document.getElementById('org-list').parentElement.style.display = 'none';
  document.getElementById('org-detail-view').style.display = 'block';
  document.getElementById('page-content').style.display = 'none';

  // store org_id for create workspace
  document.getElementById('ws-list').setAttribute('data-org-id', orgId);
  await loadWorkspaces(orgId);
}

function showOrgList() {
  currentOrgId = null;
  document.querySelector('#app-root > .mgmt-topbar').style.display = '';
  document.getElementById('org-list').parentElement.style.display = '';
  document.getElementById('org-detail-view').style.display = 'none';
  document.getElementById('page-content').style.display = '';
}

async function loadWorkspaces(orgId) {
  const token = requireAuth();
  const el = document.getElementById('ws-list');
  if (!el) return;
  try {
    const res = await fetch(API + '/organizations/' + orgId + '/workspaces', { headers: { 'Authorization': 'Bearer ' + token } });
    if (!res.ok) { el.innerHTML = '<div style="color:var(--muted);padding:1rem">Failed to load workspaces.</div>'; return; }
    const data = await res.json();
    const wss = data.workspaces || [];
    if (!wss.length) {
      el.innerHTML = '<div class="mgmt-empty"><div class="mgmt-empty-icon">📂</div><div>No workspaces in this organization.<br><button class="btn btn-primary btn-sm" onclick="showCreateWorkspace()" style="margin-top:.75rem">Create your first workspace</button></div></div>';
      return;
    }
    // Fetch resource counts per workspace
    for (const ws of wss) {
      try {
        const rr = await fetch(API + '/organizations/' + orgId + '/workspaces/' + ws.id + '/resources', { headers: { 'Authorization': 'Bearer ' + token } });
        if (!rr.ok) continue;
        const rd = await rr.json();
        ws.resCount = (rd.resources || []).length;
      } catch (_) { ws.resCount = 0; }
    }
    el.innerHTML = wss.map(ws => {
      const safeName = jesc(ws.name);
      return `<div class="ws-card">
        <div class="ws-card-header">
          <div class="ws-card-icon">${ws.icon || '⊞'}</div>
          <div class="ws-card-name">${esc(ws.name)}</div>
        </div>
        <div class="ws-card-desc">${ws.description ? esc(ws.description) : 'No description'}</div>
        <div class="ws-card-footer">
          <span class="ws-card-resource">${ws.resCount || 0} resources</span>
          <button class="btn btn-ghost btn-sm" onclick="showWsDetail('${ws.id}','${safeName}')">Manage</button>
        </div>
      </div>`;
    }).join('');
  } catch (_) {
    el.innerHTML = '<div style="color:var(--muted);padding:1rem">Could not load workspaces.</div>';
  }
}

// ── Create workspace ──
function showCreateWorkspace() { document.getElementById('create-ws-modal').classList.add('open'); document.getElementById('ws-name-input').focus(); }
function hideCreateWorkspace() { document.getElementById('create-ws-modal').classList.remove('open'); }

async function createWorkspace() {
  const name = document.getElementById('ws-name-input').value.trim();
  const desc = document.getElementById('ws-desc-input').value.trim();
  if (!name) return alert('Enter a name');
  if (!currentOrgId) return alert('No organization selected');

  const token = requireAuth();
  const res = await fetch(API + '/organizations/' + currentOrgId + '/workspaces', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
    body: JSON.stringify({ name, description: desc || null }),
  });
  const data = await res.json();
  if (!res.ok) return alert(data.detail || 'Failed');
  hideCreateWorkspace();
  document.getElementById('ws-name-input').value = '';
  document.getElementById('ws-desc-input').value = '';
  await loadWorkspaces(currentOrgId);
}

// ── Workspace detail / resources ──
async function showWsDetail(wsId, wsName) {
  currentWsId = wsId;
  document.getElementById('ws-detail-title').textContent = wsName;
  document.getElementById('delete-ws-btn').onclick = () => deleteWorkspace(wsId);
  await loadResources(wsId);
  document.getElementById('ws-detail-modal').classList.add('open');
}

function hideWsDetail() { document.getElementById('ws-detail-modal').classList.remove('open'); currentWsId = null; }

async function loadResources(wsId) {
  const token = requireAuth();
  const res = await fetch(API + '/organizations/' + currentOrgId + '/workspaces/' + wsId + '/resources', { headers: { 'Authorization': 'Bearer ' + token } });
  const data = await res.json();
  const resources = data.resources || [];
  const el = document.getElementById('ws-resources');
  if (!resources.length) {
    el.innerHTML = '<div style="color:var(--muted);font-size:.85rem;padding:.5rem 0">No resources assigned yet.</div>';
    return;
  }
  el.innerHTML = resources.map(r => {
    const typeClass = r.resource_type === 'compute' ? 'compute' : r.resource_type === 'storage' ? 'storage' : r.resource_type === 'phone' ? 'phone' : 'comms';
    const statusClass = r.status === 'active' ? 'active' : 'provisioning';
    const typeLabel = r.resource_type === 'compute' ? '⚡' : r.resource_type === 'storage' ? '💾' : r.resource_type === 'phone' ? '📞' : '🌐';
    return `<div class="res-row">
      <div class="res-icon ${typeClass}">${typeLabel}</div>
      <div class="res-info"><div class="res-name">${esc(r.name)}</div><div class="res-meta">${r.resource_type} · ${r.store_item_id}</div></div>
      <span class="res-status ${statusClass}">${r.status}</span>
      <button class="btn btn-ghost btn-sm" onclick="removeResource('${r.id}')">✕</button>
    </div>`;
  }).join('');
  document.getElementById('assign-res-modal').setAttribute('data-ws-id', wsId);
}

async function removeResource(resId) {
  if (!confirm('Remove this resource?')) return;
  const token = requireAuth();
  await fetch(API + '/organizations/' + currentOrgId + '/workspaces/' + currentWsId + '/resources/' + resId, {
    method: 'DELETE', headers: { 'Authorization': 'Bearer ' + token },
  });
  await loadResources(currentWsId);
}

async function deleteWorkspace(wsId) {
  if (!confirm('Delete this workspace and all its resources?')) return;
  const token = requireAuth();
  await fetch(API + '/organizations/' + currentOrgId + '/workspaces/' + wsId, {
    method: 'DELETE', headers: { 'Authorization': 'Bearer ' + token },
  });
  hideWsDetail();
  await loadWorkspaces(currentOrgId);
}

// ── Assign resource ──
function showAssignResource() {
  document.getElementById('assign-res-modal').classList.add('open');
  updateResourceItems();
}
function hideAssignResource() { document.getElementById('assign-res-modal').classList.remove('open'); }

function updateResourceItems() {
  const type = document.getElementById('assign-res-type').value;
  const sel = document.getElementById('assign-res-item');
  sel.innerHTML = (RES_ITEMS[type] || []).map(i => `<option value="${i.id}">${i.label}</option>`).join('');
}

async function confirmAssignResource() {
  if (!currentWsId) return;
  const storeItemId = document.getElementById('assign-res-item').value;
  const name = document.getElementById('assign-res-name').value.trim() || null;
  const token = requireAuth();
  const res = await fetch(API + '/organizations/' + currentOrgId + '/workspaces/' + currentWsId + '/resources', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
    body: JSON.stringify({ store_item_id: storeItemId, name }),
  });
  if (!res.ok) { const d = await res.json(); return alert(d.detail || 'Failed'); }
  hideAssignResource();
  document.getElementById('assign-res-name').value = '';
  await loadResources(currentWsId);
}

function esc(s) { if (!s) return ''; const d = document.createElement('div'); d.textContent = s; return d.innerHTML; }
function jesc(s) { if (!s) return ''; return esc(s).replace(/'/g,"\\'").replace(/"/g,'&quot;'); }

// ── Init ──
document.addEventListener('DOMContentLoaded', () => {
  const email = localStorage.getItem('management_email') || '';
  const emailEl = document.getElementById('sidebar-email');
  const avatarEl = document.getElementById('sidebar-avatar');
  if (emailEl) emailEl.textContent = email;
  if (avatarEl && email) avatarEl.textContent = email[0].toUpperCase();
  loadOrgs();
});
