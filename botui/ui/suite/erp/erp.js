(function() {
'use strict';
const API_BASE = '/api/erp';
let currentTab = 'financial';
let financialData = null;
let inventoryData = null;
let procurementData = null;
let branchesData = null;

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
document.getElementById('createPoBtn').addEventListener('click', () => openModal('poModal'));
document.getElementById('closePoModal').addEventListener('click', () => closeModal('poModal'));
document.getElementById('cancelPoBtn').addEventListener('click', () => closeModal('poModal'));
document.getElementById('poForm').addEventListener('submit', handlePoSubmit);
document.getElementById('addItemBtn').addEventListener('click', addPoItem);
document.getElementById('invSearch').addEventListener('input', filterInventory);
document.getElementById('invStatusFilter').addEventListener('change', filterInventory);
}

async function loadTab(tab) {
try {
switch (tab) {
case 'financial': await loadFinancial(); break;
case 'inventory': await loadInventory(); break;
case 'procurement': await loadProcurement(); break;
case 'branches': await loadBranches(); break;
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

async function loadFinancial() {
const data = await fetchApi('/financial');
financialData = data;
document.getElementById('revenue').textContent = formatCurrency(data.revenue);
document.getElementById('expenses').textContent = formatCurrency(data.expenses);
document.getElementById('netProfit').textContent = formatCurrency(data.net_profit);
document.getElementById('margin').textContent = data.margin.toFixed(1) + '%';
renderArTable(data.accounts_receivable || []);
renderApTable(data.accounts_payable || []);
}

function renderArTable(items) {
const tbody = document.querySelector('#arTable tbody');
tbody.innerHTML = items.map(item => `
<tr>
<td>${esc(item.invoice)}</td>
<td>${esc(item.customer)}</td>
<td>${formatCurrency(item.amount)}</td>
<td>${esc(item.due_date)}</td>
<td><span class="badge ${statusBadgeClass(item.status)}">${esc(item.status)}</span></td>
</tr>
`).join('');
}

function renderApTable(items) {
const tbody = document.querySelector('#apTable tbody');
tbody.innerHTML = items.map(item => `
<tr>
<td>${esc(item.bill)}</td>
<td>${esc(item.vendor)}</td>
<td>${formatCurrency(item.amount)}</td>
<td>${esc(item.due_date)}</td>
<td><span class="badge ${statusBadgeClass(item.status)}">${esc(item.status)}</span></td>
</tr>
`).join('');
}

async function loadInventory() {
const data = await fetchApi('/inventory');
inventoryData = data.items || [];
renderInventoryTable(inventoryData);
}

function renderInventoryTable(items) {
const tbody = document.querySelector('#invTable tbody');
tbody.innerHTML = items.map(item => {
const qty = item.quantity;
const reorder = item.reorder_point;
let status, badge;
if (qty === 0) { status = 'Out of Stock'; badge = 'badge-danger'; }
else if (qty <= reorder) { status = 'Low Stock'; badge = 'badge-warning'; }
else { status = 'In Stock'; badge = 'badge-success'; }
return `
<tr>
<td>${esc(item.product)}</td>
<td>${esc(item.sku)}</td>
<td>${qty}</td>
<td>${reorder}</td>
<td><span class="badge ${badge}">${status}</span></td>
</tr>
`;
}).join('');
}

function filterInventory() {
if (!inventoryData) return;
const q = document.getElementById('invSearch').value.toLowerCase();
const s = document.getElementById('invStatusFilter').value;
let filtered = inventoryData;
if (q) filtered = filtered.filter(i => i.product.toLowerCase().includes(q) || i.sku.toLowerCase().includes(q));
if (s) {
filtered = filtered.filter(i => {
if (s === 'out') return i.quantity === 0;
if (s === 'low') return i.quantity > 0 && i.quantity <= i.reorder_point;
return i.quantity > i.reorder_point;
});
}
renderInventoryTable(filtered);
}

async function loadProcurement() {
const data = await fetchApi('/procurement');
procurementData = data;
renderPoTable(data.purchase_orders || []);
await loadVendors();
await loadProducts();
}

function renderPoTable(items) {
const tbody = document.querySelector('#poTable tbody');
tbody.innerHTML = items.map(item => `
<tr>
<td>${esc(item.po_number)}</td>
<td>${esc(item.vendor)}</td>
<td>${esc(item.date)}</td>
<td>${formatCurrency(item.total)}</td>
<td><span class="badge ${statusBadgeClass(item.status)}">${esc(item.status)}</span></td>
<td>
<button class="btn-primary btn-sm" onclick="viewPo('${esc(item.po_number)}')">View</button>
</td>
</tr>
`).join('');
}

async function loadVendors() {
const select = document.getElementById('poVendor');
if (select.options.length > 1) return;
try {
const data = await fetchApi('/procurement/vendors');
(data.vendors || []).forEach(v => {
const opt = document.createElement('option');
opt.value = v.id;
opt.textContent = v.name;
select.appendChild(opt);
});
} catch (e) { console.error('Failed to load vendors', e); }
}

async function loadProducts() {
const selects = document.querySelectorAll('.po-item-select');
selects.forEach(select => {
if (select.options.length > 1) return;
fetchApi('/procurement/products').then(data => {
(data.products || []).forEach(p => {
const opt = document.createElement('option');
opt.value = p.id;
opt.textContent = p.name + ' (' + formatCurrency(p.price) + ')';
select.appendChild(opt);
});
}).catch(e => console.error('Failed to load products', e));
});
}

function addPoItem() {
const container = document.getElementById('poItems');
const row = document.createElement('div');
row.className = 'po-item-row';
const prodSelect = document.createElement('select');
prodSelect.className = 'po-item-select';
prodSelect.required = true;
const qtyInput = document.createElement('input');
qtyInput.type = 'number';
qtyInput.className = 'po-item-qty';
qtyInput.value = '1';
qtyInput.min = '1';
qtyInput.required = true;
const removeBtn = document.createElement('button');
removeBtn.type = 'button';
removeBtn.className = 'btn-remove-item';
removeBtn.textContent = '\u00d7';
removeBtn.addEventListener('click', () => row.remove());
row.appendChild(prodSelect);
row.appendChild(qtyInput);
row.appendChild(removeBtn);
container.appendChild(row);
loadProducts();
}

async function handlePoSubmit(e) {
e.preventDefault();
const vendor = document.getElementById('poVendor').value;
const notes = document.getElementById('poNotes').value;
const itemRows = document.querySelectorAll('.po-item-row');
const items = [];
itemRows.forEach(row => {
const productId = row.querySelector('.po-item-select').value;
const qty = row.querySelector('.po-item-qty').value;
if (productId) items.push({ product_id: productId, quantity: parseInt(qty) });
});
try {
await fetchApi('/procurement/orders', {
method: 'POST',
headers: { 'Content-Type': 'application/json' },
body: JSON.stringify({ vendor_id: vendor, items, notes })
});
closeModal('poModal');
document.getElementById('poForm').reset();
const container = document.getElementById('poItems');
container.innerHTML = '';
addPoItem();
loadProcurement();
} catch (e) {
console.error('Failed to create PO', e);
}
}

function openModal(id) {
document.getElementById(id).classList.add('open');
}

function closeModal(id) {
document.getElementById(id).classList.remove('open');
}

async function loadBranches() {
const data = await fetchApi('/branches');
branchesData = data;
document.getElementById('currentBranch').textContent = data.selected_branch || 'All Branches';
renderBranchTable(data.branches || []);
}

function renderBranchTable(branches) {
const tbody = document.querySelector('#branchTable tbody');
tbody.innerHTML = branches.map(b => `
<tr>
<td>${esc(b.name)}</td>
<td>${formatCurrency(b.revenue)}</td>
<td>${formatCurrency(b.expenses)}</td>
<td>${formatCurrency(b.profit)}</td>
<td>${b.headcount}</td>
<td>${b.tickets}</td>
</tr>
`).join('');
}

function viewPo(poNumber) {
console.log('View PO:', poNumber);
}

function formatCurrency(val) {
if (val === null || val === undefined) return '$0.00';
return '$' + Number(val).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function statusBadgeClass(status) {
const map = {
paid: 'badge-success', completed: 'badge-success', active: 'badge-success', in_stock: 'badge-success', approved: 'badge-success',
pending: 'badge-warning', processing: 'badge-warning', low: 'badge-warning', review: 'badge-warning',
overdue: 'badge-danger', cancelled: 'badge-danger', out_of_stock: 'badge-danger', out: 'badge-danger', rejected: 'badge-danger',
draft: 'badge-neutral', open: 'badge-info', shipped: 'badge-info'
};
return map[(status || '').toLowerCase()] || 'badge-neutral';
}

function esc(str) {
if (str === null || str === undefined) return '';
const div = document.createElement('div');
div.textContent = String(str);
return div.innerHTML;
}
})();
