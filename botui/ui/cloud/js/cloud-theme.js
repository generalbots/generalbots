(function() {
  var STORAGE_KEY = 'gb-cloud-theme';

  function getStoredTheme() {
    return localStorage.getItem(STORAGE_KEY);
  }

  function applyTheme(theme) {
    var root = document.documentElement;
    if (theme === 'system' || !theme) {
      root.removeAttribute('data-theme');
    } else {
      root.setAttribute('data-theme', theme);
    }
    document.querySelectorAll('.theme-toggle button').forEach(function(btn) {
      btn.classList.toggle('active', btn.dataset.theme === (theme || 'system'));
    });
  }

  function setTheme(theme) {
    localStorage.setItem(STORAGE_KEY, theme);
    applyTheme(theme);
  }

  // Apply stored theme immediately to prevent flash
  applyTheme(getStoredTheme());

  document.addEventListener('DOMContentLoaded', function() {
    // Re-apply to sync toggle buttons after DOM is ready
    applyTheme(getStoredTheme());

    document.querySelectorAll('.theme-toggle button').forEach(function(btn) {
      btn.addEventListener('click', function() {
        setTheme(btn.dataset.theme);
      });
    });
  });

  // Public API
  window.gbTheme = { set: setTheme, get: getStoredTheme };
})();
