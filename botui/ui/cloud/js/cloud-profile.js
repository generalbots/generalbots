const API_BASE = '/api/cloud';

document.addEventListener('DOMContentLoaded', async () => {
  const token = requireAuth();
  document.getElementById('user-email').textContent = localStorage.getItem('management_email') || '';
  await loadProfile(token);
});

async function loadProfile(token) {
  try {
    const res = await fetch(`${API_BASE}/profile`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) return;
    const data = await res.json();
    document.getElementById('profile-name').value = data.name || '';
    document.getElementById('profile-email').value = data.email || '';
    document.getElementById('profile-company').value = data.company || '';
  } catch (_) {}
}

document.getElementById('profile-form').addEventListener('submit', async (e) => {
  e.preventDefault();
  const token = requireAuth();
  const name = document.getElementById('profile-name').value;
  const email = document.getElementById('profile-email').value;
  const company = document.getElementById('profile-company').value;

  try {
    const res = await fetch(`${API_BASE}/profile`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
      body: JSON.stringify({ name, email, company }),
    });
    if (!res.ok) { const err = await res.json(); alert(err.detail || 'Save failed'); return; }
    alert('Profile saved!');
  } catch (err) {
    alert('Error: ' + err.message);
  }
});

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
