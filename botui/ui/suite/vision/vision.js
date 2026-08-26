if (window.GBAppLifecycle) GBAppLifecycle.begin("vision");
(function() {
'use strict';
const API_BASE = '/api/vision';
let currentTab = 'ocr';
let selectedFile = null;

document.querySelectorAll('.tab').forEach(tab => {
  tab.addEventListener('click', () => {
    document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
    tab.classList.add('active');
    currentTab = tab.dataset.tab;
  });
});

function showToast(message, type = 'success') {
  const toast = document.getElementById('toast');
  toast.textContent = message;
  toast.className = `toast ${type} show`;
  setTimeout(() => toast.classList.remove('show'), 3000);
}

function getConfidenceClass(conf) {
  if (conf >= 80) return 'high';
  if (conf >= 50) return 'medium';
  return 'low';
}

function getTabLabel(tab) {
  const labels = { ocr: 'OCR', objects: 'Detecção de Objetos', damage: 'Análise de Danos', plates: 'Reconhecimento de Placas' };
  return labels[tab] || tab;
}

function getTabIcon(tab) {
  const icons = { ocr: '📝', objects: '🔍', damage: '🔧', plates: '🚗' };
  return icons[tab] || '📄';
}

const uploadArea = document.getElementById('uploadArea');
const fileInput = document.getElementById('fileInput');

uploadArea.addEventListener('click', () => fileInput.click());

uploadArea.addEventListener('dragover', (e) => {
  e.preventDefault();
  uploadArea.classList.add('dragover');
});

uploadArea.addEventListener('dragleave', () => {
  uploadArea.classList.remove('dragover');
});

uploadArea.addEventListener('drop', (e) => {
  e.preventDefault();
  uploadArea.classList.remove('dragover');
  const files = e.dataTransfer.files;
  if (files.length) handleFile(files[0]);
});

fileInput.addEventListener('change', (e) => {
  if (e.target.files.length) handleFile(e.target.files[0]);
});

function handleFile(file) {
  if (!file.type.startsWith('image/')) {
    showToast('Tipo de arquivo inválido', 'error');
    return;
  }
  if (file.size > 10 * 1024 * 1024) {
    showToast('Arquivo muito grande (máx. 10MB)', 'error');
    return;
  }
  selectedFile = file;
  analyzeImage(file);
}

async function analyzeImage(file) {
  const progressBar = document.getElementById('progressBar');
  const resultsArea = document.getElementById('resultsArea');
  const resultsTitle = document.getElementById('resultsTitle');
  const resultsBody = document.getElementById('resultsBody');

  progressBar.classList.add('active');
  resultsArea.style.display = 'grid';
  resultsTitle.textContent = `${getTabLabel(currentTab)} - Resultados`;
  resultsBody.innerHTML = '<div class="loading">Analisando imagem...</div>';

  let imageUrl;
  try {
    imageUrl = await new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result || ''));
      reader.onerror = () => reject(new Error('Não foi possível ler a imagem'));
      reader.readAsDataURL(file);
    });
  } catch (err) {
    resultsBody.innerHTML = `<div class="empty-state">Erro na análise: ${err.message}</div>`;
    showToast(`Erro na análise: ${err.message}`, 'error');
    progressBar.classList.remove('active');
    return;
  }

  try {
    const resp = await fetch(`${API_BASE}/analyze`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        image_url: imageUrl,
        kind: currentTab,
        parameters: { analysis_type: currentTab, filename: file.name }
      })
    });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = await resp.json();
    renderResults(data);
    loadHistory();
  } catch (err) {
    resultsBody.innerHTML = `<div class="empty-state">Erro na análise: ${err.message}</div>`;
    showToast(`Erro na análise: ${err.message}`, 'error');
  } finally {
    progressBar.classList.remove('active');
  }
}

function renderResults(data) {
  const resultsBody = document.getElementById('resultsBody');
  const items = data.results || data.items || data.detections ||
    (Array.isArray(data.labels) ? data.labels.map(label => ({
      label,
      confidence: Number(data.confidence || 0) * 100
    })) : []);

  if (!items.length) {
    resultsBody.innerHTML = '<div class="empty-state">Nenhum resultado encontrado</div>';
    return;
  }

  resultsBody.innerHTML = items.map(item => {
    let confidence = Number(item.confidence || item.confianca || 0);
    if (confidence > 0 && confidence <= 1) confidence *= 100;
    const confClass = getConfidenceClass(confidence);
    const label = item.label || item.text || item.nome || item.type || item.tipo || 'Item';
    const details = item.details || item.detalhes || item.bounding_box || '';

    return `
      <div class="result-item">
        <div class="result-label">
          <span>${label}</span>
          <span class="result-confidence">${confidence.toFixed(1)}%</span>
        </div>
        <div class="confidence-bar">
          <div class="confidence-fill ${confClass}" style="width:${confidence}%"></div>
        </div>
        ${details ? `<div class="result-details">${typeof details === 'string' ? details : JSON.stringify(details)}</div>` : ''}
      </div>
    `;
  }).join('');
}

async function loadHistory() {
  const historyList = document.getElementById('historyList');
  try {
    const resp = await fetch(`${API_BASE}/history`);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = await resp.json();
    const items = Array.isArray(data) ? data : (data.items || data.data || []);
    renderHistory(items);
  } catch (err) {
    historyList.innerHTML = '<div class="empty-state">Erro ao carregar histórico</div>';
  }
}

function renderHistory(items) {
  const historyList = document.getElementById('historyList');
  if (!items.length) {
    historyList.innerHTML = '<div class="empty-state">Nenhum item no histórico</div>';
    return;
  }
  historyList.innerHTML = items.map(item => {
    const type = item.analysis_type || item.tipo || currentTab;
    const time = item.created_at || item.timestamp || '';
    const count = item.results_count || item.total || 0;
    const timeStr = time ? new Date(time).toLocaleString('pt-BR') : '-';

    return `
      <div class="history-item">
        <div class="history-thumb">${getTabIcon(type)}</div>
        <div class="history-info">
          <div class="history-type">${getTabLabel(type)}</div>
          <div class="history-time">${timeStr}</div>
          <div class="history-count">${count} resultado(s)</div>
        </div>
      </div>
    `;
  }).join('');
}

document.getElementById('btnClearResults').addEventListener('click', () => {
  document.getElementById('resultsBody').innerHTML = '<div class="empty-state">Envie uma imagem para analisar</div>';
  document.getElementById('resultsArea').style.display = 'none';
  selectedFile = null;
  fileInput.value = '';
});

loadHistory();
})();
