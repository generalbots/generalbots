const CAT_MAP = { compute: 'vps-small', gpu: 'gpu-basic', storage: 'storage-50', comms: 'number-local', apps: 'domain-search' };
document.addEventListener('DOMContentLoaded', () => {
  const email = localStorage.getItem('management_email') || '';
  const emailEl = document.getElementById('sidebar-email');
  const avatarEl = document.getElementById('sidebar-avatar');
  if (emailEl) emailEl.textContent = email;
  if (avatarEl && email) avatarEl.textContent = email[0].toUpperCase();

  const cat = new URLSearchParams(location.search).get('cat');
  if (cat && CAT_MAP[cat]) {
    renderRight(CAT_MAP[cat]);
  } else {
    renderStoreLanding();
  }

  if (new URLSearchParams(location.search).get('calc') === '1') {
    var right = document.getElementById('store-right');
    if (right) right.style.display = 'none';
    setTimeout(function() {
      var shell = document.getElementById('calc-shell');
      if (shell) shell.style.display = 'block';
      currentProfile = null;
      document.querySelectorAll('.calc-prof-btn').forEach(function(b) { b.classList.remove('active'); });
      var rec = document.getElementById('calc-recommend');
      if (rec) rec.textContent = 'Select a profile for personalized recommendations';
      calcUpdate();
      var anchor = location.hash === '#calc-llm-grid' ? document.getElementById('calc-llm-grid') : null;
      if (anchor) {
        anchor.scrollIntoView({ behavior: 'smooth', block: 'center' });
      } else {
        shell.scrollIntoView({ behavior: 'smooth', block: 'start' });
      }
    }, 300);
  }
});
