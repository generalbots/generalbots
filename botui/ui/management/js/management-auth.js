const API_BASE = '/api/management';

async function doLogin(e) {
  e.preventDefault();
  const email = document.getElementById('email').value;
  const password = document.getElementById('password').value;
  try {
    const res = await fetch(`${API_BASE}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password }),
    });
    if (!res.ok) { const err = await res.json(); alert(err.detail || 'Login failed'); return; }
    const data = await res.json();
    localStorage.setItem('management_token', data.token);
    localStorage.setItem('management_email', data.account.email);
    window.location.href = '/management/dashboard';
  } catch (err) {
    alert('Network error: ' + err.message);
  }
}

async function doSignup(e) {
  e.preventDefault();
  const name = document.getElementById('name').value;
  const email = document.getElementById('email').value;
  const password = document.getElementById('password').value;
  try {
    const res = await fetch(`${API_BASE}/auth/signup`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, name, password }),
    });
    if (!res.ok) { const err = await res.json(); alert(err.detail || 'Signup failed'); return; }
    const data = await res.json();
    localStorage.setItem('management_token', data.token);
    localStorage.setItem('management_email', data.account.email);
    window.location.href = '/management/dashboard';
  } catch (err) {
    alert('Network error: ' + err.message);
  }
}

function getToken() {
  return localStorage.getItem('management_token');
}

function requireAuth() {
  const token = getToken();
  if (!token) {
    window.location.href = '/management/login';
  }
  return token;
}
