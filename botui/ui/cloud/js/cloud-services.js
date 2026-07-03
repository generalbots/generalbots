document.addEventListener('DOMContentLoaded', async () => {
  const token = requireAuth();
  const emailEl = document.getElementById('user-email');
  if (emailEl) emailEl.textContent = localStorage.getItem('management_email') || '';
  await loadServices(token);
});

async function loadServices(token) {
  try {
    const res = await fetch(`${API_BASE}/services`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) { showError('Failed to load services'); return; }
    const data = await res.json();
    renderServices(data.services || []);
  } catch (err) {
    showError('Error: ' + err.message);
  }
}

function renderServices(services) {
  const container = document.getElementById('services-list');
  if (services.length === 0) {
    container.innerHTML = '<div class="saas-loading">No active services. Visit the Store to purchase one.</div>';
    return;
  }
  container.innerHTML = services.map(s => `
    <div class="service-card">
      <div class="service-header">
        <h3>${escapeHtml(s.name)}</h3>
        <span class="service-badge ${s.status}">${s.status}</span>
      </div>
      <div class="service-body">
        <p>${escapeHtml(s.description || '')}</p>
        <div class="service-meta">
          <span>Created: ${new Date(s.created_at).toLocaleDateString()}</span>
          ${s.expires_at ? `<span>Expires: ${new Date(s.expires_at).toLocaleDateString()}</span>` : ''}
        </div>
      </div>
      <div class="service-actions">
        ${s.dashboard_url ? `<a href="${s.dashboard_url}" class="btn-secondary" target="_blank">Open</a>` : ''}
        ${s.status === 'active' ? `<button class="btn-text" onclick="cancelService('${s.id}')">Cancel</button>` : ''}
      </div>
    </div>
  `).join('');
}

async function cancelService(id) {
  if (!confirm('Are you sure you want to cancel this service?')) return;
  const token = requireAuth();
  try {
    const res = await fetch(`${API_BASE}/services/${id}/cancel`, {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) { alert('Cancel failed'); return; }
    await loadServices(token);
  } catch (err) {
    alert('Error: ' + err.message);
  }
}

function requireAuth() {
  const token = localStorage.getItem('management_token');
  if (!token) window.location.href = (window.GB_LOGIN_URL || '/login');
  return token;
}

function doLogout() {
  localStorage.removeItem('management_token');
  localStorage.removeItem('management_email');
  window.location.href = '/';
}

function escapeHtml(str) {
  if (!str) return '';
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function showError(msg) {
  document.getElementById('services-list').innerHTML = '<div class="saas-loading" style="color:red">' + escapeHtml(msg) + '</div>';
}
