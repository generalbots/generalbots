if (window.GBAppLifecycle) GBAppLifecycle.begin("video");
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
  if (!toast) return;
  toast.textContent = message;
  toast.className = `toast ${type} show`;
  setTimeout(() => toast.classList.remove('show'), 3000);
}

function t(key, fallback) {
  return window.gbT ? window.gbT(key, fallback) : fallback;
}

function escapeHtml(str) {
  if (str == null) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function formatTime(dateStr) {
  if (!dateStr) return '-';
  const d = new Date(dateStr);
  const now = new Date();
  const diffMs = now - d;
  const diffMin = Math.floor(diffMs / 60000);
  if (diffMin < 1) return t('video-now', 'now');
  if (diffMin < 60) return `${diffMin} ${t('video-min-ago', 'min ago')}`;
  const diffH = Math.floor(diffMin / 60);
  if (diffH < 24) return `${diffH} ${t('video-hour-ago', 'h ago')}`;
  return gbFormatDate(d);
}

function renderCameras(cameras) {
  const grid = document.getElementById('camerasGrid');
  // The desktop window manager removes the app DOM on close while the
  // polling interval keeps firing; no-op instead of throwing (#908).
  if (!grid) return;
  if (!cameras.length) {
    grid.innerHTML = '<div class="loading">' + escapeHtml(t('video-no-cameras', 'No cameras configured')) + '</div>';
    return;
  }
  grid.innerHTML = cameras.map(cam => {
    const status = (cam.status === 'online') ? 'online' : 'offline';
    const name = escapeHtml(cam.name || cam.nome || t('video-no-name', 'No name'));
    const loc = escapeHtml(cam.location || cam.localizacao || '');
    const lastAlert = escapeHtml(cam.last_alert || cam.ultimo_alerta || '');
    return `
    <div class="camera-card" data-id="${escapeHtml(cam.id != null ? String(cam.id) : '')}">
      <div class="camera-preview">
        ${status === 'online' ? '<span class="live-badge">' + escapeHtml(t('video-live', 'Live feed')) + '</span>' : ''}
        <span>📷 ${status === 'online' ? escapeHtml(t('video-live', 'Live feed')) : escapeHtml(t('video-offline', 'Offline'))}</span>
      </div>
      <div class="camera-info">
        <span class="camera-name">${name}</span>
        <span class="camera-status">
          <span class="status-dot ${status}"></span>
          ${escapeHtml(status === 'online' ? t('status-online', 'Online') : t('video-offline', 'Offline'))}
        </span>
      </div>
      ${loc ? `<div class="camera-location">📍 ${loc}</div>` : ''}
      ${lastAlert ? `<div class="camera-last-alert">⚠️ ${lastAlert}</div>` : ''}
    </div>
  `;
  }).join('');
}

function renderAlerts(alerts) {
  const list = document.getElementById('alertsList');
  if (!list) return;
  if (!alerts.length) {
    list.innerHTML = '<div class="loading">' + escapeHtml(t('video-no-alerts', 'No alerts recorded')) + '</div>';
    return;
  }
  list.innerHTML = alerts.map(a => {
    const sev = String(a.severity || a.severidade || 'medium').toLowerCase();
    const sevSafe = ['low', 'medium', 'high', 'critical'].includes(sev) ? sev : 'medium';
    return `
    <div class="alert-item">
      <div class="alert-time">${escapeHtml(formatTime(a.timestamp))}</div>
      <div class="alert-content">
        <div class="alert-camera">${escapeHtml(a.camera || a.camera_name || t('video-camera', 'Camera'))}</div>
        <div class="alert-type">${escapeHtml(a.type || a.tipo || t('video-motion-detected', 'Motion detected'))}</div>
        <span class="severity-badge ${sevSafe}">${escapeHtml(sevSafe.toUpperCase())}</span>
      </div>
    </div>
  `;
  }).join('');
}

function renderAnalytics(analytics) {
  const list = document.getElementById('analyticsList');
  if (!list) return;
  const num = (value) => {
    const n = Number(value);
    return Number.isFinite(n) ? n : 0;
  };
  const items = [
    { label: t('video-detections-today', 'Detections today'), value: num(analytics.detections_today || analytics.detecoes_hoje) },
    { label: t('video-uptime', 'Uptime'), value: `${num(analytics.uptime)}%` },
    { label: t('video-storage', 'Storage'), value: `${num(analytics.storage_used || analytics.armazenamento)} GB` },
    { label: t('video-cameras-online', 'Cameras online'), value: `${num(analytics.cameras_online)} / ${num(analytics.cameras_total)}` }
  ];
  list.innerHTML = items.map(i => `
    <div class="analytics-item">
      <span class="analytics-label">${escapeHtml(i.label)}</span>
      <span class="analytics-value">${escapeHtml(i.value)}</span>
    </div>
  `).join('');

  const statDetections = document.getElementById('statDetections');
  const statUptime = document.getElementById('statUptime');
  const statStorage = document.getElementById('statStorage');
  const statCamerasActive = document.getElementById('statCamerasActive');
  if (statDetections) statDetections.textContent = num(analytics.detections_today || analytics.detecoes_hoje);
  if (statUptime) statUptime.textContent = `${num(analytics.uptime)}%`;
  if (statStorage) statStorage.textContent = `${num(analytics.storage_used || analytics.armazenamento)} GB`;
  if (statCamerasActive) statCamerasActive.textContent = num(analytics.cameras_online);
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
    name: (document.getElementById('cameraName').value || '').trim(),
    rtsp_url: (document.getElementById('cameraRtsp').value || '').trim(),
    location: (document.getElementById('cameraLocation').value || '').trim()
  };

  if (!payload.name) {
    showToast(t('video-name-required', 'Name is required'), 'error');
    return;
  }
  if (!payload.rtsp_url) {
    showToast(t('video-rtsp-required', 'RTSP URL is required'), 'error');
    return;
  }
  // #2403 — validate the stream URL instead of accepting any string: only
  // rtsp/rtsps are valid, and a malformed value is rejected before it reaches
  // the server (where it would be stored and later rendered back).
  let parsedUrl = null;
  try {
    parsedUrl = new URL(payload.rtsp_url);
  } catch (err) {
    showToast(t('video-rtsp-invalid', 'RTSP URL must look like rtsp://host/stream'), 'error');
    return;
  }
  if (!['rtsp:', 'rtsps:'].includes(parsedUrl.protocol)) {
    showToast(t('video-rtsp-scheme', 'RTSP URL must use rtsp:// or rtsps://'), 'error');
    return;
  }
  if (!parsedUrl.hostname) {
    showToast(t('video-rtsp-host', 'RTSP URL must include a host'), 'error');
    return;
  }

  try {
    const resp = await fetch(`${API_BASE}/cameras`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    showToast(t('video-camera-added', 'Camera added'));
    document.getElementById('addCameraModal').classList.remove('active');
    document.getElementById('cameraName').value = '';
    document.getElementById('cameraRtsp').value = '';
    document.getElementById('cameraLocation').value = '';
    loadCameras();
  } catch (err) {
    showToast(`${t('video-add-failed', 'Failed to add camera')}: ${err.message}`, 'error');
  }
}

const btnAddCamera = document.getElementById('btnAddCamera');
const btnSaveCamera = document.getElementById('btnSaveCamera');
const btnRefreshCameras = document.getElementById('btnRefreshCameras');
const btnRefreshAlerts = document.getElementById('btnRefreshAlerts');
if (btnAddCamera) btnAddCamera.addEventListener('click', () => {
  const modal = document.getElementById('addCameraModal');
  modal.classList.add('active');
  // #2403 — dialog keyboard contract: focus the first field and close on Esc.
  const firstField = document.getElementById('cameraName');
  if (firstField) firstField.focus();
});
// #2403 — Esc closes the Add Camera dialog like every other suite modal.
document.addEventListener('keydown', (e) => {
  if (e.key !== 'Escape') return;
  const modal = document.getElementById('addCameraModal');
  if (modal && modal.classList.contains('active')) {
    modal.classList.remove('active');
    if (btnAddCamera) btnAddCamera.focus();
  }
});
if (btnSaveCamera) btnSaveCamera.addEventListener('click', addCamera);
if (btnRefreshCameras) btnRefreshCameras.addEventListener('click', loadCameras);
if (btnRefreshAlerts) btnRefreshAlerts.addEventListener('click', loadAlerts);

loadAll();
// Re-opening the app re-injects this script; drop any previous poll so
// closed windows do not keep piling up intervals against a removed DOM.
if (window.__videoPollTimer) clearInterval(window.__videoPollTimer);
window.__videoPollTimer = window.GBAppLifecycle
  ? GBAppLifecycle.interval("video", loadAlerts, 30000)
  : setInterval(loadAlerts, 30000);
})();
