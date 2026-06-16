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
  const cat = params.get('cat');

  // Update topbar label if category selected
  const catLabels = {
    compute: 'VPS / Compute',
    gpu: 'GPU Inference',
    storage: 'Object Storage',
    comms: 'Phone Numbers',
    apps: 'App Store Publishing',
    domains: 'Domains'
  };
  const topbarLabel = document.getElementById('topbar-cat-label');
  if (topbarLabel && cat && catLabels[cat]) {
    topbarLabel.textContent = catLabels[cat];
  }

  // Render catalog left panel
  if (cat && CAT_MAP[cat]) {
    renderRight(CAT_MAP[cat]);
  } else {
    renderStoreLanding();
  }

  // Calculator always visible in right panel — just initialize
  calcUpdate();
});
