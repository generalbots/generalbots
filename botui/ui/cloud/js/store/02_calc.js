let currentProfile = null;
let selectedLLMPrice = 0;

function selectProfile(profile) {
  currentProfile = profile;
  document.querySelectorAll('.calc-prof-btn').forEach(b => b.classList.toggle('active', b.dataset.profile === profile));
  calcUpdate();
}

function getRec(profile, field, value) {
  const p = PROFILE_RECS[profile];
  if (!p) return null;
  const f = p[field];
  if (typeof f === 'object' && f[value]) return f[value];
  if (typeof f === 'string') return f;
  return null;
}

function selectLLM(provider, tokens, price, event) {
  if (event) { event.stopPropagation(); }
  selectedLLMPrice = price || 0;
  document.querySelectorAll('.calc-llm-card').forEach(c => c.classList.remove('selected'));
  document.querySelectorAll('.calc-pkg-btn').forEach(b => b.classList.remove('active'));
  if (provider) {
    const card = document.querySelector(`.calc-llm-card[data-provider="${provider}"]`);
    if (card) card.classList.add('selected');
    if (event && event.target && event.target.closest('.calc-pkg-btn')) {
      event.target.closest('.calc-pkg-btn').classList.add('active');
    }
  } else {
    const noCard = document.querySelector('.calc-llm-card:not([data-provider])');
    if (noCard) noCard.classList.add('selected');
  }
  calcUpdate();
}

function calcUpdate() {
  const sel = name => {
    const el = document.querySelector(`input[name="${name}"]:checked`);
    const label = el ? el.closest('.calc-vps-btn, .calc-addon-btn, .calc-prof-btn') : null;
    return {
      val: el ? el.value : null,
      price: label ? parseFloat(label.dataset.price || 0) : 0,
      labelEl: label
    };
  };

  const vps     = sel('calc-vps');
  const gpu     = sel('calc-gpu');
  const storage = sel('calc-storage');
  const phone   = sel('calc-phone');
  const domain  = sel('calc-domain');

  const total = vps.price + gpu.price + storage.price + phone.price + domain.price + selectedLLMPrice;

  document.querySelectorAll('.calc-vps-btn, .calc-addon-btn').forEach(o => o.classList.remove('selected'));
  [vps, gpu, storage, phone, domain].forEach(s => { if (s.labelEl) s.labelEl.classList.add('selected'); });

  let freeMsgs = 0;
  if (phone.val === '1') freeMsgs = 1000;
  else if (phone.val === '3') freeMsgs = 3000;
  else if (phone.val === '10') freeMsgs = 10000;

  const bonus = document.getElementById('calc-whatsapp-bonus');
  if (bonus) {
    bonus.style.display = freeMsgs > 0 ? '' : 'none';
    const msgsEl = document.getElementById('calc-whatsapp-msgs');
    if (msgsEl) msgsEl.textContent = freeMsgs.toLocaleString();
  }

  const totalEl = document.getElementById('calc-total');
  if (totalEl) totalEl.textContent = total.toFixed(2);

  const saasEl = document.getElementById('calc-saas-cost');
  const gbEl = document.getElementById('calc-gb-cost');
  const saveEl = document.getElementById('calc-savings-amount');
  const pctEl = document.getElementById('calc-savings-pct');
  if (saasEl && gbEl && saveEl && pctEl) {
    const saasGuess = 300;
    const savings = saasGuess * 12 - total * 12;
    const pct = saasGuess > 0 ? Math.round((1 - total / saasGuess) * 100) : 0;
    saasEl.textContent = '$' + saasGuess + '/mo';
    gbEl.textContent = '$' + total.toFixed(2) + '/mo';
    saveEl.textContent = '$' + savings.toFixed(0);
    pctEl.textContent = pct + '% cheaper';
  }

  const vpsTip = document.getElementById('calc-vps-tip');
  const gpuTip = document.getElementById('calc-gpu-tip');
  const recPanel = document.getElementById('calc-recommend');

  if (currentProfile) {
    const vpsKey = vps.val ? vps.val.replace('vps-', '') : 'small';
    const vpsRec = getRec(currentProfile, 'vps', vpsKey);
    const gpuRec = getRec(currentProfile, 'gpu');
    if (vpsTip) {
      if (vpsRec) { vpsTip.style.display = 'block'; vpsTip.innerHTML = vpsRec; }
      else vpsTip.style.display = 'none';
    }
    if (gpuTip) {
      if (gpuRec && gpu.val !== 'none') { gpuTip.style.display = 'block'; gpuTip.textContent = gpuRec; }
      else if (gpu.val === 'none' && gpuRec) { gpuTip.style.display = 'block'; gpuTip.textContent = gpuRec; }
      else gpuTip.style.display = 'none';
    }
    if (recPanel) {
      const pn = PROFILE_RECS[currentProfile]?.label || currentProfile;
      const s = PROFILE_RECS[currentProfile]?.summary || '';
      recPanel.textContent = 'Profile: ' + pn + ' — ' + s;
    }
  } else {
    if (vpsTip) vpsTip.style.display = 'none';
    if (gpuTip) gpuTip.style.display = 'none';
    if (recPanel) recPanel.textContent = 'Select a profile for personalized recommendations';
  }
}

function submitCalculator() {
  const sel = name => {
    const el = document.querySelector(`input[name="${name}"]:checked`);
    const label = el ? el.closest('.calc-vps-btn, .calc-addon-btn') : null;
    return { val: el ? el.value : null, price: label ? parseFloat(label.dataset.price || 0) : 0 };
  };
  const vps     = sel('calc-vps');
  const gpu     = sel('calc-gpu');
  const storage = sel('calc-storage');
  const phone   = sel('calc-phone');
  const domain  = sel('calc-domain');
  const total   = vps.price + gpu.price + storage.price + phone.price + domain.price + selectedLLMPrice;

  if (!vps.val) {
    showToast('Please select a server size first.', 'error');
    return;
  }

  const params = new URLSearchParams({
    plan: 'private-cloud',
    vps: vps.val,
    storage: storage.val || '0',
    gpu: gpu.val || 'none',
    phone: phone.val || '0',
    domain: domain.val || '',
    total: total.toFixed(2),
  });

  showToast('Redirecting to checkout…', 'info');
  setTimeout(() => {
    window.location.href = '/cloud/checkout?' + params.toString();
  }, 800);
}
