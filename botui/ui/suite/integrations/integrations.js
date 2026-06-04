const API_BASE = '/api/integrations';
let currentTab = 'available';
let connectorsData = [];
let connectedData = [];
let etlData = [];

document.addEventListener('DOMContentLoaded', () => {
initTabs();
initEvents();
loadTab(currentTab);
});

function initTabs() {
document.querySelectorAll('.tab').forEach(tab => {
tab.addEventListener('click', () => {
document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
tab.classList.add('active');
const tabName = tab.dataset.tab;
document.getElementById('panel-' + tabName).classList.add('active');
currentTab = tabName;
loadTab(tabName);
});
});
}

function initEvents() {
document.getElementById('refreshBtn').addEventListener('click', () => loadTab(currentTab));
document.getElementById('createJobBtn').addEventListener('click', () => openModal('etlModal'));
document.getElementById('closeEtlModal').addEventListener('click', () => closeModal('etlModal'));
document.getElementById('cancelEtlBtn').addEventListener('click', () => closeModal('etlModal'));
document.getElementById('etlForm').addEventListener('submit', handleEtlSubmit);
document.getElementById('connectorSearch').addEventListener('input', filterConnectors);
}

async function loadTab(tab) {
try {
switch (tab) {
case 'available': await loadConnectors(); break;
case 'connected': await loadConnected(); break;
case 'etl': await loadEtl(); break;
}
} catch (e) {
console.error('Failed to load ' + tab, e);
}
}

async function fetchApi(url, options) {
const res = await fetch(API_BASE + url, options);
if (!res.ok) throw new Error('API error: ' + res.status);
return res.json();
}

async function loadConnectors() {
const data = await fetchApi('/connectors');
connectorsData = data.connectors || [];
renderConnectorGrid(connectorsData);
}

function renderConnectorGrid(connectors) {
const grid = document.getElementById('connectorGrid');
grid.innerHTML = connectors.map(c => `
<div class="connector-card" data-id="${esc(c.id)}">
<div class="connector-icon ${esc(c.type)}">${getConnectorIcon(c.type)}</div>
<div class="connector-name">${esc(c.name)}</div>
<div class="connector-desc">${esc(c.description)}</div>
<div class="connector-status">
<span class="status-dot ${c.connected ? 'connected' : 'available'}"></span>
${c.connected ? 'Connected' : 'Available'}
</div>
<button class="btn-connect" ${c.connected ? 'disabled' : ''} onclick="connectConnector('${esc(c.id)}')">
${c.connected ? 'Connected' : 'Connect'}
</button>
</div>
`).join('');
}

function getConnectorIcon(type) {
const icons = {
salesforce: '\u2601',
sap: '\u2699',
totvs: '\u2630',
bling: '\u26a1',
rest: '\u2191',
database: '\u2637'
};
return icons[type] || '\u25cf';
}

function filterConnectors() {
const q = document.getElementById('connectorSearch').value.toLowerCase();
const filtered = connectorsData.filter(c =>
c.name.toLowerCase().includes(q) || c.type.toLowerCase().includes(q)
);
renderConnectorGrid(filtered);
}

async function connectConnector(id) {
try {
await fetchApi('/connectors/' + id + '/connect', { method: 'POST' });
loadConnectors();
loadConnected();
} catch (e) {
console.error('Failed to connect', e);
}
}

async function loadConnected() {
const data = await fetchApi('/connectors?connected=true');
connectedData = data.connectors || [];
renderConnectedTable(connectedData);
}

function renderConnectedTable(connectors) {
const tbody = document.querySelector('#connectedTable tbody');
tbody.innerHTML = connectors.map(c => `
<tr>
<td>${esc(c.name)}</td>
<td><span class="badge badge-info">${esc(c.type)}</span></td>
<td><span class="badge ${c.active ? 'badge-success' : 'badge-warning'}">${c.active ? 'Active' : 'Paused'}</span></td>
<td>${esc(c.last_sync || 'Never')}</td>
<td>${c.records_synced || 0}</td>
<td>
<button class="btn-sm btn-primary" onclick="syncConnector('${esc(c.id)}')">Sync Now</button>
<button class="btn-danger-sm" onclick="disconnectConnector('${esc(c.id)}')">Disconnect</button>
</td>
</tr>
`).join('');
}

async function syncConnector(id) {
try {
await fetchApi('/connectors/' + id + '/sync', { method: 'POST' });
loadConnected();
} catch (e) {
console.error('Sync failed', e);
}
}

async function disconnectConnector(id) {
try {
await fetchApi('/connectors/' + id + '/disconnect', { method: 'DELETE' });
loadConnected();
loadConnectors();
} catch (e) {
console.error('Disconnect failed', e);
}
}

async function loadEtl() {
const data = await fetchApi('/etl');
etlData = data.jobs || [];
renderEtlTable(etlData);
await loadEtlSources();
}

function renderEtlTable(jobs) {
const tbody = document.querySelector('#etlTable tbody');
tbody.innerHTML = jobs.map(j => `
<tr>
<td>${esc(j.name)}</td>
<td>${esc(j.source)}</td>
<td>${esc(j.destination)}</td>
<td><span class="badge badge-neutral">${esc(j.schedule)}</span></td>
<td>${esc(j.last_run || 'Never')}</td>
<td><span class="badge ${jobStatusBadge(j.status)}">${esc(j.status)}</span></td>
<td>
<button class="btn-sm btn-primary" onclick="runJob('${esc(j.id)}')">Run</button>
<button class="btn-danger-sm" onclick="deleteJob('${esc(j.id)}')">Delete</button>
</td>
</tr>
`).join('');
}

function jobStatusBadge(status) {
const map = {
success: 'badge-success',
running: 'badge-info',
failed: 'badge-danger',
pending: 'badge-warning'
};
return map[(status || '').toLowerCase()] || 'badge-neutral';
}

async function loadEtlSources() {
try {
const data = await fetchApi('/connectors?connected=true');
const sources = data.connectors || [];
const srcSelect = document.getElementById('jobSource');
const destSelect = document.getElementById('jobDest');
if (srcSelect.options.length <= 1) {
sources.forEach(s => {
const opt = document.createElement('option');
opt.value = s.id;
opt.textContent = s.name;
srcSelect.appendChild(opt);
const opt2 = opt.cloneNode(true);
destSelect.appendChild(opt2);
});
}
} catch (e) {
console.error('Failed to load sources', e);
}
}

async function handleEtlSubmit(e) {
e.preventDefault();
try {
await fetchApi('/etl', {
method: 'POST',
headers: { 'Content-Type': 'application/json' },
body: JSON.stringify({
name: document.getElementById('jobName').value,
source: document.getElementById('jobSource').value,
destination: document.getElementById('jobDest').value,
schedule: document.getElementById('jobSchedule').value,
transform: document.getElementById('jobTransform').value || null
})
});
closeModal('etlModal');
document.getElementById('etlForm').reset();
loadEtl();
} catch (e) {
console.error('Failed to create job', e);
}
}

async function runJob(id) {
try {
await fetchApi('/etl/' + id + '/run', { method: 'POST' });
loadEtl();
} catch (e) {
console.error('Run failed', e);
}
}

async function deleteJob(id) {
try {
await fetchApi('/etl/' + id, { method: 'DELETE' });
loadEtl();
} catch (e) {
console.error('Delete failed', e);
}
}

function openModal(id) {
document.getElementById(id).classList.add('open');
}

function closeModal(id) {
document.getElementById(id).classList.remove('open');
}

function esc(str) {
if (str === null || str === undefined) return '';
const div = document.createElement('div');
div.textContent = String(str);
return div.innerHTML;
}