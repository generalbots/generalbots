const API_BASE = '/api/cloud';

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
    // Persist session
    localStorage.setItem('management_token', data.token);
    localStorage.setItem('management_email', data.email || email);
    localStorage.setItem('management_name',  data.name  || '');
    var dest = (new URLSearchParams(window.location.search)).get('redirect') || (CLOUD_CONFIG.baseUrl + '/dashboard');
    window.location.href = dest + '?token=' + encodeURIComponent(data.token) + '&email=' + encodeURIComponent(data.email || email) + '&name=' + encodeURIComponent(data.name  || '');
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
  const botName = document.getElementById('signup-botname').value.trim();
  const plan = document.querySelector('input[name="plan"]:checked')?.value || 'free';
  const templateSelect = document.getElementById('bot-template');
  const template = templateSelect ? templateSelect.value : '';

  btn.textContent = 'Creating account…';
  btn.disabled = true;
  errEl.style.display = 'none';

  try {
    const res = await fetch(`${API_BASE}/auth/signup`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, name, bot_name: botName, password, plan, template }),
    });
    const data = await res.json();
    if (!res.ok) {
      errEl.textContent = data.detail || data || 'Signup failed.';
      errEl.style.display = 'block';
      return;
    }
    // Handler returns: { status, account: { email, name }, org_id, branch_id, bot_id,
    //                    bucket, subscription_id, plan, trial_days, token }
    const token = data.token;
    const emailOut = data.account?.email || email;
    localStorage.setItem('management_token', token);
    localStorage.setItem('management_email', emailOut);
    localStorage.setItem('management_name',  data.account?.name || name);
    localStorage.setItem('management_org_id', data.org_id || '');
    localStorage.setItem('management_bot_id', data.bot_id || '');
    localStorage.setItem('management_bot_name', botName);
    localStorage.setItem('management_plan', data.plan || 'free');
    // Private Server: redirect to Store VPS calculator instead of Dashboard
    var redir = (new URLSearchParams(window.location.search)).get('redirect');
    var dest = redir || (plan === 'private-cloud' ? CLOUD_CONFIG.baseUrl + '/store' : CLOUD_CONFIG.baseUrl + '/dashboard');
    window.location.href = dest + '?token=' + encodeURIComponent(token) + '&email=' + encodeURIComponent(emailOut) + '&name=' + encodeURIComponent(data.account?.name || name);
  } catch (err) {
    errEl.textContent = 'Network error: ' + err.message;
    errEl.style.display = 'block';
  } finally {
    btn.textContent = 'Create Account';
    btn.disabled = false;
  }
}

async function doSignup(e) { return handleSignup(e); }

// ── Plan selector visual highlight + private server toggle ──
document.addEventListener('DOMContentLoaded', function() {
  // Use ?plan= query param to pre-select a plan
  var params = new URLSearchParams(window.location.search);
  var presetPlan = params.get('plan');
  if (presetPlan) {
    var presetInput = document.querySelector('input[name="plan"][value="' + presetPlan + '"]');
    if (presetInput) {
      document.querySelectorAll('.plan-option').forEach(function(o) { o.classList.remove('selected'); });
      var presetLabel = presetInput.closest('.plan-option');
      if (presetLabel) presetLabel.classList.add('selected');
      presetInput.checked = true;
    }
  }

  var infoEl = document.getElementById('private-server-info');
  var btn = document.getElementById('signup-btn');
  document.querySelectorAll('.plan-option').forEach(function(el) {
    el.addEventListener('click', function() {
      document.querySelectorAll('.plan-option').forEach(function(o) { o.classList.remove('selected'); });
      el.classList.add('selected');
      var isPrivate = el.querySelector('input[name="plan"]')?.value === 'private-cloud';
      if (infoEl) infoEl.style.display = isPrivate ? 'block' : 'none';
      // All plans use "Create Account" — private redirects to store after signup
      if (btn) btn.textContent = 'Create Account';
    });
  });
});

// ── Token helpers ──
function getToken() {
  return localStorage.getItem('management_token');
}

function requireAuth() {
  const token = getToken();
  if (!token) window.location.href = (window.GB_LOGIN_URL || '/login');
  return token;
}

function doLogout() {
  localStorage.removeItem('management_token');
  localStorage.removeItem('management_email');
  localStorage.removeItem('management_name');
  sessionStorage.setItem('gb-signed-out', 'true');
  window.location.href = '/';
}
