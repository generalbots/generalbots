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

// Listen for successful auth inside the embedded iframe (port 5000) and
// finish the flow here: persist the token and navigate to the dashboard.
window.addEventListener('message', function (event) {
  var data = event.data;
  if (!data || data.type !== 'GB_AUTH_SUCCESS') return;
  if (data.token) {
    localStorage.setItem('management_token', data.token);
  }
  if (data.email) localStorage.setItem('management_email', data.email);
  if (data.name) localStorage.setItem('management_name', data.name);
  closeEmbeddedSignup();
  window.location.href = '/dashboard';
});
