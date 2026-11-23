function onActivateClick() {
  if (isAuthenticated()) {
    submitCalculator();
  } else {
    openEmbeddedSignup();
  }
}

function openEmbeddedSignup() {
  switchEmbeddedAuth('signup');
}

function switchEmbeddedAuth(mode) {
  var loginBase = window.GB_LOGIN_URL || '/login';
  var path = mode === 'signup' ? '/signup?embedded=1' : '/login?embedded=1';
  var iframe = document.getElementById('signup-iframe');
  if (!iframe) return;
  var title = document.getElementById('signup-modal-title');
  if (title) title.textContent = mode === 'signup' ? 'Create Free Account' : 'Sign In';
  iframe.src = loginBase + path + '&redirect=' + encodeURIComponent(window.location.origin + '/dashboard');
  document.getElementById('signup-modal').classList.add('open');
}

function closeEmbeddedSignup() {
  var modal = document.getElementById('signup-modal');
  if (modal) modal.classList.remove('open');
  var iframe = document.getElementById('signup-iframe');
  if (iframe) iframe.src = 'about:blank';
}

// #2812 — only accept the handoff from the configured login origin, and only
// once: the short-lived state cookie is consumed on receipt. A wildcard
// postMessage target on the sender is useless if this origin check is absent.
function gbTrustedLoginOrigin(origin) {
  var allowed = [window.location.origin];
  try { if (window.GB_LOGIN_URL) allowed.push(new URL(window.GB_LOGIN_URL, window.location.origin).origin); } catch (_) {}
  return allowed.indexOf(origin) !== -1;
}
function gbConsumeSso(state, ts) {
  var raw = null, h = location.hostname;
  try { document.cookie.split(';').forEach(function (c) { c = c.trim(); if (c.indexOf('gb_sso_state=') === 0) raw = c.slice(13); }); } catch (_) {}
  var d = h === 'localhost' || h === '127.0.0.1' ? '' : (h.split('.').length > 2 ? '.' + h.split('.').slice(1).join('.') : '');
  document.cookie = 'gb_sso_state=; Path=/; SameSite=Lax; Max-Age=-1' + (d ? '; Domain=' + d : '') + (location.protocol === 'https:' ? '; Secure' : '');
  if (!raw || !state || !/^\d{10,15}$/.test(String(ts))) return false;
  var t = Number(ts), now = Date.now();
  if (now - t > 60000 || t - now > 5000) return false;
  try { var p = decodeURIComponent(raw).split('.'); return p.length === 2 && p[0] === state && p[1] === String(ts); } catch (_) { return false; }
}
window.addEventListener('message', function (event) {
  var data = event.data;
  if (!data || data.type !== 'GB_AUTH_SUCCESS') return;
  if (!gbTrustedLoginOrigin(event.origin)) return;
  if (data.state && !gbConsumeSso(data.state, data.ts)) return;
  if (data.token) {
    localStorage.setItem('management_token', data.token);
  }
  if (data.email) localStorage.setItem('management_email', data.email);
  if (data.name) localStorage.setItem('management_name', data.name);
  closeEmbeddedSignup();
  window.location.href = '/dashboard';
});
