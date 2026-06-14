const API_BASE = '/api/management';

// ── Login ──
async function handleLogin(e) {
  e.preventDefault();
  const btn   = document.getElementById('login-btn');
  const errEl = document.getElementById('login-error');
  const email = document.getElementById('login-email').value.trim();
  const password = document.getElementById('login-password').value;

  btn.textContent = 'Signing in…';
  btn.disabled = true;
  errEl.style.display = 'none';

  try {
    const res = await fetch(`${API_BASE}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password }),
    });
    const data = await res.json();
    if (!res.ok) {
      errEl.textContent = data.detail || data || 'Login failed. Check your credentials.';
      errEl.style.display = 'block';
      return;
    }
    // Persist session (token + email from new handler shape)
    localStorage.setItem('management_token', data.token);
    localStorage.setItem('management_email', data.email || email);
    localStorage.setItem('management_name',  data.name  || '');
    window.location.href = '/management/dashboard';
  } catch (err) {
    errEl.textContent = 'Network error: ' + err.message;
    errEl.style.display = 'block';
  } finally {
    btn.textContent = 'Sign In';
    btn.disabled = false;
  }
}

// Legacy alias kept for signup.html compatibility
async function doLogin(e) { return handleLogin(e); }

// ── Signup ──
async function handleSignup(e) {
  e.preventDefault();
  const btn   = document.getElementById('signup-btn');
  const errEl = document.getElementById('signup-error');
  const name  = document.getElementById('signup-name').value.trim();
  const email = document.getElementById('signup-email').value.trim();
  const password = document.getElementById('signup-password').value;

  btn.textContent = 'Creating account…';
  btn.disabled = true;
  errEl.style.display = 'none';

  try {
    const res = await fetch(`${API_BASE}/auth/signup`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, name, password }),
    });
    const data = await res.json();
    if (!res.ok) {
      errEl.textContent = data.detail || data || 'Signup failed.';
      errEl.style.display = 'block';
      return;
    }
    // Handler returns: { status, account: { email, name }, token, org_id }
    const token = data.token;
    const emailOut = data.account?.email || email;
    localStorage.setItem('management_token', token);
    localStorage.setItem('management_email', emailOut);
    localStorage.setItem('management_name',  data.account?.name || name);
    window.location.href = '/management/dashboard';
  } catch (err) {
    errEl.textContent = 'Network error: ' + err.message;
    errEl.style.display = 'block';
  } finally {
    btn.textContent = 'Create Account';
    btn.disabled = false;
  }
}

async function doSignup(e) { return handleSignup(e); }

// ── Token helpers ──
function getToken() {
  return localStorage.getItem('management_token');
}

function requireAuth() {
  const token = getToken();
  if (!token) window.location.href = '/management/login';
  return token;
}

function doLogout() {
  localStorage.removeItem('management_token');
  localStorage.removeItem('management_email');
  localStorage.removeItem('management_name');
  window.location.href = '/management';
}
