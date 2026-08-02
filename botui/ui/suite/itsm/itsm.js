(function() {
'use strict';
const API_BASE = '/api/itsm';
let currentTab = 'incidents';
let incidentsData = [];
let catalogData = [];
let cmdbData = [];
let kbData = [];

(function(){ var __cb = () => {
initTabs();
initEvents();
loadTab(currentTab);
}; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();

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
document.getElementById('priorityFilter').addEventListener('change', filterIncidents);
document.getElementById('statusFilter').addEventListener('change', filterIncidents);
document.getElementById('closeRequestModal').addEventListener('click', () => closeModal('requestModal'));
document.getElementById('cancelRequestBtn').addEventListener('click', () => closeModal('requestModal'));
document.getElementById('requestForm').addEventListener('submit', handleRequestSubmit);
document.getElementById('kbSearch').addEventListener('input', filterKb);
document.getElementById('kbCategoryFilter').addEventListener('change', filterKb);
document.getElementById('addCiBtn').addEventListener('click', addCi);
}

async function loadTab(tab) {
try {
switch (tab) {
case 'incidents': await loadIncidents(); break;
case 'requests': await loadCatalog(); break;
case 'cmdb': await loadCmdb(); break;
case 'kb': await loadKb(); break;
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

async function loadIncidents() {
const data = await fetchApi('/incidents');
incidentsData = data.incidents || [];
renderIncidents(incidentsData);
}

function renderIncidents(incidents) {
const tbody = document.querySelector('#incidentTable tbody');
tbody.innerHTML = incidents.map(i => {
const sla = getSlaInfo(i.sla_deadline);
return `
<tr>
<td><span class="badge badge-neutral">${esc(i.id)}</span></td>
<td>${esc(i.title)}</td>
<td><span class="badge ${priorityBadge(i.priority)}">${esc(i.priority)}</span></td>
<td><span class="badge ${statusBadge(i.status)}">${esc(i.status)}</span></td>
<td>${esc(i.assignee || 'Unassigned')}</td>
<td><span class="sla-countdown ${sla.class}">${sla.text}</span></td>
<td>
<button class="btn-sm btn-primary" onclick="updateIncident('${esc(i.id)}')">Update</button>
</td>
</tr>
`;
}).join('');
}

function getSlaInfo(deadline) {
if (!deadline) return { text: 'N/A', class: 'sla-ok' };
const now = new Date();
const dl = new Date(deadline);
const diff = dl - now;
const hours = Math.floor(diff / 3600000);
const mins = Math.floor((diff % 3600000) / 60000);
if (diff < 0) return { text: 'Breached', class: 'sla-breach' };
if (hours < 4) return { text: hours + 'h ' + mins + 'm', class: 'sla-warning' };
return { text: hours + 'h ' + mins + 'm', class: 'sla-ok' };
}

function priorityBadge(p) {
const map = { critical: 'badge-critical', high: 'badge-danger', medium: 'badge-warning', low: 'badge-info' };
return map[(p || '').toLowerCase()] || 'badge-neutral';
}

function statusBadge(s) {
const map = { open: 'badge-info', in_progress: 'badge-warning', resolved: 'badge-success', closed: 'badge-neutral' };
return map[(s || '').toLowerCase()] || 'badge-neutral';
}

function filterIncidents() {
const p = document.getElementById('priorityFilter').value;
const s = document.getElementById('statusFilter').value;
let filtered = incidentsData;
if (p) filtered = filtered.filter(i => i.priority === p);
if (s) filtered = filtered.filter(i => i.status === s);
renderIncidents(filtered);
}

async function updateIncident(id) {
console.log('Update incident:', id);
}

async function loadCatalog() {
const data = await fetchApi('/requests/catalog');
catalogData = data.catalog || [];
renderCatalog(catalogData);
}

function renderCatalog(items) {
const grid = document.getElementById('catalogGrid');
grid.innerHTML = items.map(i => `
<div class="catalog-card">
<div class="catalog-icon">${getRequestIcon(i.category)}</div>
<div class="catalog-title">${esc(i.name)}</div>
<div class="catalog-desc">${esc(i.description)}</div>
<button class="btn-outline" onclick="openRequestForm('${esc(i.id)}', '${esc(i.name)}')">Request</button>
</div>
`).join('');
}

function getRequestIcon(cat) {
const icons = {
hardware: '\ud83d\udd27',
software: '\ud83d\udcbb',
access: '\ud83d\udd10',
network: '\ud83d\udd0c',
email: '\u2709',
other: '\u2753'
};
return icons[(cat || '').toLowerCase()] || '\u2753';
}

function openRequestForm(id, name) {
document.getElementById('requestTypeId').value = id;
document.getElementById('requestTypeLabel').textContent = 'Requesting: ' + name;
openModal('requestModal');
}

async function handleRequestSubmit(e) {
e.preventDefault();
try {
await fetchApi('/requests', {
method: 'POST',
headers: { 'Content-Type': 'application/json' },
body: JSON.stringify({
type_id: document.getElementById('requestTypeId').value,
description: document.getElementById('requestDesc').value,
priority: document.getElementById('requestPriority').value
})
});
closeModal('requestModal');
document.getElementById('requestForm').reset();
} catch (e) {
console.error('Submit failed', e);
}
}

async function loadCmdb() {
const data = await fetchApi('/cmdb');
cmdbData = data.items || [];
renderCmdbTree(cmdbData);
}

function renderCmdbTree(items) {
const tree = document.getElementById('cmdbTree');
tree.innerHTML = items.map(i => `
<div class="tree-node" style="padding-left: ${20 + (i.level || 0) * 24}px">
<div class="tree-info">
<span class="tree-icon">${getCiIcon(i.type)}</span>
<span class="tree-name">${esc(i.name)}</span>
</div>
<div class="tree-meta">
<span class="badge badge-neutral">${esc(i.type)}</span>
<span class="badge ${ciStatusBadge(i.status)}">${esc(i.status)}</span>
<span>${esc(i.owner || '')}</span>
</div>
</div>
`).join('');
}

function getCiIcon(type) {
const icons = { server: '\ud83d\udda5', workstation: '\ud83d\udcbb', router: '\ud83d\udd0c', switch: '\ud83d\udd18', printer: '\ud83d\udda8', other: '\u2699' };
return icons[(type || '').toLowerCase()] || '\u2699';
}

function ciStatusBadge(s) {
const map = { active: 'badge-success', inactive: 'badge-neutral', maintenance: 'badge-warning', decommissioned: 'badge-danger' };
return map[(s || '').toLowerCase()] || 'badge-neutral';
}

function addCi() {
console.log('Add CI');
}

async function loadKb() {
const data = await fetchApi('/kb');
kbData = data.articles || [];
renderKbArticles(kbData);
}

function renderKbArticles(articles) {
const list = document.getElementById('articleList');
list.innerHTML = articles.map(a => `
<div class="article-card" onclick="viewArticle('${esc(a.id)}')">
<div class="article-header">
<span class="article-title">${esc(a.title)}</span>
<span class="article-category">${esc(a.category)}</span>
</div>
<div class="article-excerpt">${esc(a.excerpt)}</div>
<div class="article-meta">
<span>${a.views || 0} views</span>
<span>Updated: ${esc(a.updated_at)}</span>
<span>By ${esc(a.author)}</span>
</div>
</div>
`).join('');
}

function filterKb() {
const q = document.getElementById('kbSearch').value.toLowerCase();
const cat = document.getElementById('kbCategoryFilter').value;
let filtered = kbData;
if (q) filtered = filtered.filter(a => a.title.toLowerCase().includes(q) || a.excerpt.toLowerCase().includes(q));
if (cat) filtered = filtered.filter(a => a.category === cat);
renderKbArticles(filtered);
}

function viewArticle(id) {
console.log('View article:', id);
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
})();
