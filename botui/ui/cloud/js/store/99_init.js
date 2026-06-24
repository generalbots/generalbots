const CAT_MAP = { compute: 'vps-small', gpu: 'gpu-basic', storage: 'storage-50', comms: 'number-local', apps: 'domain-search' };

document.addEventListener('DOMContentLoaded', () => {
  const email = localStorage.getItem('management_email') || '';
  const emailEl = document.getElementById('sidebar-email');
  const avatarEl = document.getElementById('sidebar-avatar');
  if (emailEl) emailEl.textContent = email;
  if (avatarEl && email) {
    avatarEl.textContent = email[0].toUpperCase();
    avatarEl.title = email;
  }

  const params = new URLSearchParams(location.search);
  const calc = params.get('calc') === '1';

  if (!calc) {
    const cat = params.get('cat');

    // Machines has its own dedicated page
    if (cat === 'machines') {
      window.location.replace('/cloud/machines');
      return;
    }

    // Update topbar label if category selected
    const catLabels = {
      compute: 'Virtual Machines',
      gpu: 'GPU Computing',
      storage: 'Object Storage',
      comms: 'Phone Numbers',
      apps: 'Domains',
    };
    const topbarLabel = document.getElementById('topbar-cat-label');
    if (topbarLabel && cat && catLabels[cat]) {
      topbarLabel.textContent = catLabels[cat];
    }

    if (cat && CAT_MAP[cat]) {
      renderRight(CAT_MAP[cat]);
    } else {
      renderStoreLanding();
    }
  } else {
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
