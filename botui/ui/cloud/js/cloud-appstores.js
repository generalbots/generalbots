"use strict";
const API_BASE = '/api/cloud';

function requireAuth() {
  const token = localStorage.getItem('management_token');
  if (!token) {
    window.location.href = (window.GB_LOGIN_URL || '/login');
  }
  return token;
}

function doLogout() {
  localStorage.removeItem('management_token');
  localStorage.removeItem('management_email');
  localStorage.removeItem('management_name');
  window.location.href = '/';
}

function escapeHtml(str) {
  if (!str) return '';
  const d = document.createElement('div');
  d.textContent = str;
  return d.innerHTML;
}

function showToast(msg) {
  const existing = document.querySelector('.mgmt-toast');
  if (existing) existing.remove();
  const t = document.createElement('div');
  t.className = 'mgmt-toast';
  t.textContent = msg;
  Object.assign(t.style, {
    position: 'fixed', bottom: '1.5rem', right: '1.5rem',
    background: 'var(--accent2)', color: '#000', padding: '0.75rem 1.25rem',
    borderRadius: '8px', fontWeight: '600', zIndex: '9999',
    boxShadow: '0 4px 16px rgba(0,0,0,0.3)', fontSize: '0.9rem',
  });
  document.body.appendChild(t);
  setTimeout(() => t.remove(), 4000);
}

document.addEventListener('DOMContentLoaded', () => {
  const token = requireAuth();
  const email = localStorage.getItem('management_email') || '…';
  const name = localStorage.getItem('management_name') || email.split('@')[0] || 'User';
  document.getElementById('sidebar-email').textContent = email;
  document.getElementById('sidebar-avatar').textContent = name.charAt(0).toUpperCase();
});

async function purchaseAppStore(store) {
  const token = requireAuth();
  const email = localStorage.getItem('management_email');
  if (!email) { showToast('Please login first'); return; }

  const prices = { google: 125, apple: 199, bundle: 280 };
  const labels = {
    google: 'Google Play Publishing Consultancy',
    apple: 'Apple App Store Publishing Consultancy',
    bundle: 'Both Stores Publishing Bundle',
  };
  const amount = prices[store];
  const label = labels[store] || 'App Store Publishing';

  try {
    const res = await fetch(`${API_BASE}/appstore/purchase`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
      body: JSON.stringify({ store, amount, email, description: label }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Request failed' }));
      showToast('Error: ' + (err.detail || err.message || 'Unknown'));
      return;
    }
    const data = await res.json();
    if (data.checkout_url) {
      window.location.href = data.checkout_url;
    } else {
      showToast('Order placed! Invoice: ' + data.invoice_number);
      setTimeout(() => { window.location.href = '/invoices'; }, 1500);
    }
  } catch (err) {
    showToast('Network error: ' + err.message);
  }
}
