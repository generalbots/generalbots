const API_BASE = '/api/management';

document.addEventListener('DOMContentLoaded', async () => {
  const token = requireAuth();
  document.getElementById('user-email').textContent = localStorage.getItem('management_email') || '';
  await loadStore(token);
});

let storeItems = [];
let selectedItem = null;

async function loadStore(token) {
  try {
    const res = await fetch(`${API_BASE}/store`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) { showError('Failed to load store'); return; }
    const data = await res.json();
    storeItems = data.items || [];
    renderStore('all');
  } catch (err) {
    showError('Error: ' + err.message);
  }
}

function renderStore(category) {
  const grid = document.getElementById('store-grid');
  const filtered = category === 'all' ? storeItems : storeItems.filter(i => i.category === category);

  if (filtered.length === 0) {
    grid.innerHTML = '<div class="saas-loading">No items in this category.</div>';
    return;
  }

  grid.innerHTML = filtered.map(item => `
    <div class="store-card">
      <div class="store-icon">${item.icon || '📦'}</div>
      <h3>${escapeHtml(item.name)}</h3>
      <p class="store-desc">${escapeHtml(item.description || '')}</p>
      <div class="store-price">${formatStorePrice(item)}</div>
      <button class="btn-primary btn-block" onclick="openPurchase('${item.id}')">Buy Now</button>
    </div>
  `).join('');
}

function filterStore(cat) {
  document.querySelectorAll('.category-btn').forEach(b => b.classList.toggle('active', b.dataset.cat === cat));
  renderStore(cat);
}

function formatStorePrice(item) {
  if (item.price_type === 'free') return 'Free';
  if (item.price_type === 'custom') return 'Contact Us';
  const amt = (item.amount || 0) / 100;
  return '$' + amt.toFixed(2) + '/' + (item.period || 'mo');
}

function openPurchase(id) {
  selectedItem = storeItems.find(i => i.id === id);
  if (!selectedItem) return;
  document.getElementById('modal-title').textContent = 'Purchase: ' + selectedItem.name;
  document.getElementById('modal-body').innerHTML = `
    <p>${escapeHtml(selectedItem.description || '')}</p>
    <p><strong>Price:</strong> ${formatStorePrice(selectedItem)}</p>
    <div class="form-group">
      <label for="purchase-org">Organization</label>
      <select id="purchase-org"></select>
    </div>
  `;
  loadOrgsForPurchase();
  document.getElementById('purchase-modal').style.display = 'flex';
}

async function loadOrgsForPurchase() {
  const token = requireAuth();
  try {
    const res = await fetch(`${API_BASE}/organizations`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) return;
    const orgs = await res.json();
    const sel = document.getElementById('purchase-org');
    sel.innerHTML = orgs.map(o => `<option value="${o.id}">${escapeHtml(o.name)}</option>`).join('');
  } catch (_) {}
}

async function confirmPurchase() {
  const token = requireAuth();
  const orgId = document.getElementById('purchase-org').value;
  if (!selectedItem || !orgId) return;
  try {
    const res = await fetch(`${API_BASE}/store/purchase`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
      body: JSON.stringify({ item_id: selectedItem.id, org_id: orgId }),
    });
    if (!res.ok) { const err = await res.json(); alert(err.detail || 'Purchase failed'); return; }
    alert('Purchase initiated! Check My Services for status.');
    hideModal();
  } catch (err) {
    alert('Error: ' + err.message);
  }
}

function hideModal() { document.getElementById('purchase-modal').style.display = 'none'; }

function requireAuth() {
  const token = localStorage.getItem('management_token');
  if (!token) window.location.href = '/management/login';
  return token;
}

function doLogout() {
  localStorage.removeItem('management_token');
  localStorage.removeItem('management_email');
  window.location.href = '/management';
}

function escapeHtml(str) {
  if (!str) return '';
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function showError(msg) {
  document.getElementById('store-grid').innerHTML = '<div class="saas-loading" style="color:red">' + escapeHtml(msg) + '</div>';
}
