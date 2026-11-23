const API_BASE = '/api/cloud';

// ── #2809 — post-auth destination guard ──────────────────────────────────────
// The login host mints the bearer token and hands it to the cloud/suite host.
// `redirect` / `return_to` are attacker-controllable, so they are resolved and
// checked against a strict allowlist BEFORE any navigation happens. A foreign
// destination must never receive the freshly minted token.

function isAuthTarget(u) {
  return /\/(login|signup)\/?$/.test(u.pathname);
}

// Returns true only for http(s) URLs on the login origin or the configured
// cloud origin. No credentials, no other schemes, no foreign hosts.
function isAllowedOrigin(u, origin, baseUrlOverride) {
  if (!u || (u.protocol !== 'http:' && u.protocol !== 'https:')) return false;
  if (u.username || u.password) return false;
  var allowed = [origin];
  try {
    var configured = baseUrlOverride;
    if (!configured && window.CLOUD_CONFIG) configured = window.CLOUD_CONFIG.baseUrl;
    if (configured) allowed.push(new URL(configured, origin).origin);
  } catch (_) { /* malformed config: keep the login origin only */ }
  return allowed.indexOf(u.origin) !== -1;
}

// Pure validator — safe to call with hostile input, and usable from the
// self-test at the bottom of this file. Returns a URL object or null.
// `originOverride` / `baseUrlOverride` exist only for the self-test so the
// assertions do not depend on the host the page happens to be served from.
function validateRedirect(raw, originOverride, baseUrlOverride) {
  if (!raw || typeof raw !== 'string') return null;
  // Protocol-relative ("//host") is a foreign host by construction — reject
  // before the URL parser can resolve it against a different authority.
  if (/^\s*\/\//.test(raw)) return null;
  var origin = originOverride || window.location.origin;
  var u;
  try { u = new URL(raw, origin); } catch (_) { return null; }
  if (!isAllowedOrigin(u, origin, baseUrlOverride)) return null;
  // #1324 — never bounce back to an auth page (nesting ?redirect= forever).
  if (isAuthTarget(u)) return null;
  return u;
}

// Fallback destination: the cloud origin when configured, otherwise login.
function defaultDestination(path) {
  var origin = window.location.origin;
  var base = origin;
  try {
    if (window.CLOUD_CONFIG && window.CLOUD_CONFIG.baseUrl) {
      base = new URL(window.CLOUD_CONFIG.baseUrl, origin).origin;
    }
  } catch (_) { /* keep login origin */ }
  var u;
  try { u = new URL(path || '/dashboard', base); } catch (_) { u = new URL('/', origin); }
  return isAllowedOrigin(u, origin) ? u : new URL('/', origin);
}

// `redirect` is used by the server-side suite gate; `return_to` by the
// desktop shell (auth-guard.js). Both go through the same allowlist.
function resolveRedirect(fallbackPath) {
  var params = new URLSearchParams(window.location.search);
  var raw = params.get('redirect') || params.get('return_to');
  return validateRedirect(raw) || defaultDestination(fallbackPath);
}

// Build the final URL with URL.searchParams.set() instead of string
// concatenation: an injected '?' / '#' / extra token param in `redirect`
// cannot override the identity we just authenticated.
function buildAuthDestination(dest, identity) {
  var target = isAllowedOrigin(dest, window.location.origin) ? dest : defaultDestination('/dashboard');
  target.searchParams.set('token', identity.token || '');
  target.searchParams.set('email', identity.email || '');
  target.searchParams.set('name', identity.name || '');
  return target.href;
}

// ── Login ──
// When the login/signup form is embedded in another page (iframe, e.g. the
// cloud store), notify the parent frame so it can close the modal and finish
// the flow without a full-page redirect inside the iframe.
function notifyParentSuccess(token, email, name) {
  try {
    if (window.self !== window.top) {
      window.parent.postMessage({
        type: 'GB_AUTH_SUCCESS',
        token: token,
        email: email || '',
        name: name || '',
      }, '*');
    }
  } catch (_) { /* parent may be cross-origin and unreachable */ }
}

// #2090 — login/signup errors and busy labels come from the catalogue; the
// authored English stays as the fallback.
function currentLocale() {
  try {
    return localStorage.getItem('gb-locale') || navigator.language || 'en';
  } catch (e) {
    return 'en';
  }
}

function _t(key, fallback, params) {
  return window.gbT ? window.gbT(key, fallback, params) : fallback;
}

// #2595 — progress helpers. The spinner is a CSS ::before on the button, so
// replacing the label via textContent never removes the animation. aria-busy
// and the polite live region make the state visible to assistive technology.
function showBusy(btn, label) {
  if (!btn) return;
  btn.classList.add('is-busy');
  btn.setAttribute('aria-busy', 'true');
  btn.textContent = label;
}
function clearBusy(btn, label) {
  if (!btn) return;
  btn.classList.remove('is-busy');
  btn.removeAttribute('aria-busy');
  if (label) btn.textContent = label;
}
function announce(liveEl, msg) {
  if (liveEl) liveEl.textContent = msg || '';
}
function showFormError(errEl, liveEl, msg) {
  if (errEl) {
    errEl.textContent = msg;
    errEl.style.display = 'block';
  }
  announce(liveEl, msg);
}

async function handleLogin(e) {
  e.preventDefault();
  const btn    = document.getElementById('login-btn');
  const errEl  = document.getElementById('login-error');
  const liveEl = document.getElementById('login-live');
  const email = document.getElementById('login-email').value.trim();
  const password = document.getElementById('login-password').value;
  const signingIn = _t('login-signing-in', 'Signing in…');
  const progress  = _t('login-auth-progress', 'Signing in, please wait…');
  const submitLabel = _t('login-submit', 'Sign In');

  errEl.textContent = '';
  errEl.style.display = 'none';
  announce(liveEl, progress);
  btn.disabled = true;
  showBusy(btn, signingIn);

  var succeeded = false;
  try {
    // Cloud installs expose the management login under /api/cloud. The
    // desktop edition ships without the cloud router: when that endpoint is
    // absent (404) sign in through the directory route (/api/auth/login),
    // which verifies the wizard's administrator credentials. The busy state
    // above spans both attempts, including the 404 fallback hop.
    const attempt = (base) => fetch(base + '/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      // #2090 — carry the chosen locale so the first dashboard render can
      // already be localized (the server ignores unknown fields today).
      body: JSON.stringify({ email, password, locale: currentLocale() }),
    });
    let res = await attempt(API_BASE);
    if (res.status === 404) {
      res = await attempt('/api');
    }
    const text = await res.text();
    let data = {};
    try { data = JSON.parse(text); } catch (_) { data = { detail: text }; }
    if (!res.ok) {
      showFormError(errEl, liveEl, data.detail || data.error || _t('login-error-failed', 'Login failed. Check your credentials.'));
      return;
    }
    const token = data.token || data.access_token;
    if (!token) {
      showFormError(errEl, liveEl, _t('login-error-no-token', 'Login succeeded but no session token was returned.'));
      return;
    }
    // Persist session
    localStorage.setItem('management_token', token);
    localStorage.setItem('gb-access-token', token);
    localStorage.setItem('management_email', data.email || email);
    localStorage.setItem('management_name',  data.name  || '');
    notifyParentSuccess(token, data.email || email, data.name || '');
    var dest = buildAuthDestination(
      resolveRedirect('/dashboard'),
      { token: token, email: data.email || email, name: data.name || '' }
    );
    succeeded = true;
    announce(liveEl, progress);
    window.location.href = dest;
  } catch (err) {
    showFormError(errEl, liveEl, _t('login-error-network', 'Network error: {error}', { error: err.message }));
  } finally {
    // #2595 — always release the disabled state so the button can never get
    // stuck. On success keep the spinner + status label until navigation
    // begins; on failure stop the animation, restore the label and focus.
    btn.disabled = false;
    if (!succeeded) {
      clearBusy(btn, submitLabel);
      btn.focus();
    }
  }
}

// Legacy alias kept for signup.html compatibility
async function doLogin(e) { return handleLogin(e); }

// ── Signup ──
// The submit label follows the selected plan (#2090); reuse it when restoring
// the button after a failed request.
function signupSubmitLabel() {
  const checked = document.querySelector('input[name="plan"]:checked');
  const plan = checked ? checked.value : 'free';
  if (plan === 'shared') return _t('signup-submit-trial', 'Start 14-Day Trial');
  if (plan === 'private-cloud') return _t('signup-submit-quote', 'Get a Quote');
  return _t('signup-submit-free', 'Create Free Account');
}

async function handleSignup(e) {
  e.preventDefault();
  const btn    = document.getElementById('signup-btn');
  const errEl  = document.getElementById('signup-error');
  const liveEl = document.getElementById('signup-live');
  const name  = document.getElementById('signup-name').value.trim();
  const email = document.getElementById('signup-email').value.trim();
  const password = document.getElementById('signup-password').value;
  const botName = document.getElementById('signup-botname').value.trim();
  const plan = document.querySelector('input[name="plan"]:checked')?.value || 'free';
  const templateSelect = document.getElementById('bot-template');
  const template = templateSelect ? templateSelect.value : '';
  const creating = _t('signup-creating', 'Creating account…');
  const progress = _t('signup-auth-progress', 'Creating your account, please wait…');

  errEl.textContent = '';
  errEl.style.display = 'none';
  announce(liveEl, progress);
  btn.disabled = true;
  showBusy(btn, creating);

  var succeeded = false;
  try {
    const res = await fetch(`${API_BASE}/auth/signup`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email, name, bot_name: botName, password, plan, template,
        locale: currentLocale(),
      }),
    });
    const text = await res.text();
    let data = {};
    try { data = JSON.parse(text); } catch (_) { data = { detail: text }; }
    if (!res.ok) {
      showFormError(errEl, liveEl, data.detail || _t('signup-error-failed', 'Signup failed.'));
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
    notifyParentSuccess(token, emailOut, data.account?.name || name);
    // Private Server: redirect to Store VPS calculator instead of Dashboard
    var dest = buildAuthDestination(
      resolveRedirect(plan === 'private-cloud' ? '/store' : '/dashboard'),
      { token: token, email: emailOut, name: data.account?.name || name }
    );
    succeeded = true;
    announce(liveEl, progress);
    window.location.href = dest;
  } catch (err) {
    showFormError(errEl, liveEl, _t('login-error-network', 'Network error: {error}', { error: err.message }));
  } finally {
    btn.disabled = false;
    if (!succeeded) {
      clearBusy(btn, signupSubmitLabel());
      btn.focus();
    }
  }
}

async function doSignup(e) { return handleSignup(e); }

// ── Plan selector visual highlight + private server toggle ──
if (typeof document !== 'undefined') document.addEventListener('DOMContentLoaded', function() {
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
      // Keep the label in sync with the selected plan (free/trial/quote).
      if (btn && !btn.classList.contains('is-busy')) btn.textContent = signupSubmitLabel();
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

// ── #2809 self-test ───────────────────────────────────────────────────────────
// Browser console: __gbCloudAuthSelfTest(). Node: tests/cloud_auth_redirect.test.js.
// Pure: overrides remove any dependency on the host serving the page.
function runRedirectSelfTest() {
  var origin = 'https://login.example';
  var cloud = 'https://cloud.example';
  var cases = [
    ['https://evil.example/dashboard?token=STOLEN', null],
    ['//evil.example/dashboard', null],
    ['https://login.example@evil.example/dashboard', null],
    ['javascript:alert(1)', null],
    [cloud + '/dashboard', cloud + '/dashboard'],
    ['/drive', origin + '/drive'],
    [origin + '/login', null]
  ];
  var results = cases.map(function(c) {
    var u = validateRedirect(c[0], origin, cloud);
    var actual = u ? u.href : null;
    return { input: c[0], expected: c[1], actual: actual, pass: actual === c[1] };
  });
  // A token param smuggled into `redirect` must be overwritten by searchParams.set.
  var injected = validateRedirect('/dashboard?token=INJECTED', origin, cloud);
  injected.searchParams.set('token', 'REAL');
  results.push({
    input: 'searchParams.set over injected token',
    expected: origin + '/dashboard?token=REAL',
    actual: injected.href,
    pass: injected.href === origin + '/dashboard?token=REAL'
  });
  var failed = results.filter(function(r) { return !r.pass; }).length;
  return { passed: results.length - failed, failed: failed, results: results };
}

if (typeof window !== 'undefined') window.__gbCloudAuthSelfTest = runRedirectSelfTest;
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { validateRedirect: validateRedirect, runRedirectSelfTest: runRedirectSelfTest };
}
