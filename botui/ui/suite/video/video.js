(function() {
'use strict';
const API_BASE = '/api/video';
const state = { cameras: [], alerts: [], analytics: {} };

document.querySelectorAll('[data-close]').forEach(btn => {
  btn.addEventListener('click', () => {
    document.getElementById(btn.dataset.close).classList.remove('active');
  });
});

document.querySelectorAll('.modal-overlay').forEach(overlay => {
  overlay.addEventListener('click', (e) => {
    if (e.target === overlay) overlay.classList.remove('active');
  });
});

function showToast(message, type = 'success') {
  const toast = document.getElementById('toast');
  toast.textContent = message;
  toast.className = `toast ${type} show`;
  setTimeout(() => toast.classList.remove('show'), 3000);
}

function formatTime(dateStr) {
  if (!dateStr) return '-';
  const d = new Date(dateStr);
  const now = new Date();
  const diffMs = now - d;
  const diffMin = Math.floor(diffMs / 60000);
  if (diffMin < 1) return 'agora';
  if (diffMin < 60) return `${diffMin}min atrás`;
  const diffH = Math.floor(diffMin / 60);
  if (diffH < 24) return `${diffH}h atrás`;
  return d.toLocaleDateString('pt-BR');
}

function renderCameras(cameras) {
  const grid = document.getElementById('camerasGrid');
  if (!cameras.length) {
    grid.innerHTML = '<div class="loading">Nenhuma câmera cadastrada</div>';
    return;
  }
  grid.innerHTML = cameras.map(cam => `
    <div class="camera-card" data-id="${cam.id}">
      <div class="camera-preview">
        ${cam.status === 'online' ? '<span class="live-badge">LIVE</span>' : ''}
        <span>📷 ${cam.status === 'online' ? 'Feed ao vivo' : 'Offline'}</span>
      </div>
      <div class="camera-info">
        <span class="camera-name">${cam.name || cam.nome || 'Sem nome'}</span>
        <span class="camera-status">
          <span class="status-dot ${cam.status}"></span>
          ${cam.status === 'online' ? 'Online' : 'Offline'}
        </span>
      </div>
      ${cam.location || cam.localizacao ? `<div class="camera-location">📍 ${cam.location || cam.localizacao}</div>` : ''}
      ${cam.last_alert || cam.ultimo_alerta ? `<div class="camera-last-alert">⚠️ ${cam.last_alert || cam.ultimo_alerta}</div>` : ''}
    </div>
  `).join('');
}

function renderAlerts(alerts) {
  const list = document.getElementById('alertsList');
  if (!alerts.length) {
    list.innerHTML = '<div class="loading">Nenhum alerta registrado</div>';
    return;
  }
  list.innerHTML = alerts.map(a => `
    <div class="alert-item">
      <div class="alert-time">${formatTime(a.timestamp)}</div>
      <div class="alert-content">
        <div class="alert-camera">${a.camera || a.camera_name || 'Câmera'}</div>
        <div class="alert-type">${a.type || a.tipo || 'Movimento detectado'}</div>
        <span class="severity-badge ${a.severity || a.severidade || 'medium'}">${(a.severity || a.severidade || 'medium').toUpperCase()}</span>
      </div>
    </div>
  `).join('');
}

function renderAnalytics(analytics) {
  const list = document.getElementById('analyticsList');
  const items = [
    { label: 'Detecções Hoje', value: analytics.detections_today || analytics.detecoes_hoje || 0 },
    { label: 'Uptime', value: `${analytics.uptime || 0}%` },
    { label: 'Armazenamento', value: `${analytics.storage_used || analytics.armazenamento || 0} GB` },
    { label: 'Câmeras Online', value: `${analytics.cameras_online || 0} / ${analytics.cameras_total || 0}` }
  ];
  list.innerHTML = items.map(i => `
    <div class="analytics-item">
      <span class="analytics-label">${i.label}</span>
      <span class="analytics-value">${i.value}</span>
    </div>
  `).join('');

  document.getElementById('statDetections').textContent = analytics.detections_today || analytics.detecoes_hoje || 0;
  document.getElementById('statUptime').textContent = `${analytics.uptime || 0}%`;
  document.getElementById('statStorage').textContent = `${analytics.storage_used || analytics.armazenamento || 0} GB`;
  document.getElementById('statCamerasActive').textContent = analytics.cameras_online || 0;
}

async function loadCameras() {
  try {
    const resp = await fetch(`${API_BASE}/cameras`);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = await resp.json();
    state.cameras = Array.isArray(data) ? data : (data.items || data.data || []);
    renderCameras(state.cameras);
  } catch (err) {
    console.error('Erro ao carregar câmeras:', err);
    renderCameras([]);
  }
}

async function loadAlerts() {
  try {
    const resp = await fetch(`${API_BASE}/alerts`);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = await resp.json();
    state.alerts = Array.isArray(data) ? data : (data.items || data.data || []);
    renderAlerts(state.alerts);
  } catch (err) {
    console.error('Erro ao carregar alertas:', err);
    renderAlerts([]);
  }
}

async function loadAnalytics() {
  try {
    const resp = await fetch(`${API_BASE}/analytics`);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = await resp.json();
    state.analytics = data;
    renderAnalytics(data);
  } catch (err) {
    console.error('Erro ao carregar analytics:', err);
    renderAnalytics({});
  }
}

async function loadAll() {
  await Promise.all([loadCameras(), loadAlerts(), loadAnalytics()]);
}

async function addCamera() {
  const payload = {
    name: document.getElementById('cameraName').value,
    rtsp_url: document.getElementById('cameraRtsp').value,
    location: document.getElementById('cameraLocation').value
  };

  if (!payload.name) {
    showToast('Nome é obrigatório', 'error');
    return;
  }

  try {
    const resp = await fetch(`${API_BASE}/cameras`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    showToast('Câmera adicionada com sucesso');
    document.getElementById('addCameraModal').classList.remove('active');
    document.getElementById('cameraName').value = '';
    document.getElementById('cameraRtsp').value = '';
    document.getElementById('cameraLocation').value = '';
    loadCameras();
  } catch (err) {
    showToast(`Erro ao adicionar câmera: ${err.message}`, 'error');
  }
}

document.getElementById('btnAddCamera').addEventListener('click', () => {
  document.getElementById('addCameraModal').classList.add('active');
});

document.getElementById('btnSaveCamera').addEventListener('click', addCamera);
document.getElementById('btnRefreshCameras').addEventListener('click', loadCameras);
document.getElementById('btnRefreshAlerts').addEventListener('click', loadAlerts);

loadAll();
setInterval(loadAlerts, 30000);
})();
