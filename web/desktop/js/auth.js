// Handle authentication state
let currentUser = null;
let currentSession = null;

// Initialize auth with mock data
function initializeAuth() {
  if (window.location.pathname.includes('auth')) {
    return; // Don't initialize on auth pages
  }

  // Check for existing session
  const sessionId = localStorage.getItem('sessionId');
  if (sessionId) {
    currentSession = mockSessions.find(s => s.id === sessionId) || mockSessions[0];
  } else {
    currentSession = mockSessions[0];
    localStorage.setItem('sessionId', currentSession.id);
  }

  // Set current user
  currentUser = mockUsers[0];
  updateUserUI();
}

// Update UI based on auth state
function updateUserUI() {
  const userAvatar = document.getElementById('userAvatar');
  if (userAvatar && currentUser) {
    userAvatar.textContent = currentUser.avatar;
  }
}

// Handle login
function handleLogin(email, password) {
  // In a real app, this would call an API
  currentUser = mockUsers.find(u => u.email === email) || mockUsers[0];
  currentSession = mockSessions[0];
  localStorage.setItem('sessionId', currentSession.id);
  updateUserUI();
  window.location.href = '/desktop/index.html';
}

// Handle logout
function handleLogout() {
  localStorage.removeItem('sessionId');
  window.location.href = '/desktop/auth/login.html';
}

// Check auth state for protected routes
function checkAuth() {
  if (!currentUser && !window.location.pathname.includes('auth')) {
    window.location.href = '/desktop/auth/login.html';
  }
}

// Initialize on page load
if (document.readyState === 'complete') {
  initializeAuth();
} else {
  window.addEventListener('load', initializeAuth);
}
