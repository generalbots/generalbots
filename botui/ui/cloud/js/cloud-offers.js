const API_BASE = '/api/cloud';

document.addEventListener('DOMContentLoaded', async () => {
  const token = getToken();
  const email = localStorage.getItem('management_email') || '';

  if (token) {
    await loadOrgs(token);
    await loadInvoicesAndBalance(token, email);
  }
});

let organizations = [];
let currentBalance = 0.0;

async function loadOrgs(token) {
  try {
    const res = await fetch(`${API_BASE}/organizations`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) return;
    const data = await res.json();
    organizations = data.organizations || [];

    const select = document.getElementById('topup-org');
    if (select) {
      select.innerHTML = organizations.map(o => 
        `<option value="${o.id}">${escapeHtml(o.name)}</option>`
      ).join('');
    }
  } catch (err) {
    console.error('Failed to load organizations:', err);
  }
}

async function loadInvoicesAndBalance(token, email) {
  try {
    const res = await fetch(`${API_BASE}/invoices`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });
    if (!res.ok) return;
    const data = await res.json();
    const invoices = data.invoices || [];

    // Calcular o saldo somando faturas do tipo INV-TOPUP que foram pagas
    let balance = 0.0;
    const topupInvoices = invoices.filter(inv => inv.number.startsWith('INV-TOPUP') && inv.status === 'paid');
    
    topupInvoices.forEach(inv => {
      balance += parseFloat(inv.total) || 0.0;
    });

    currentBalance = balance;
    document.getElementById('current-balance').textContent = `$${currentBalance.toFixed(2)}`;

    // Render top-up history
    const tbody = document.getElementById('topup-history-tbody');
    if (tbody) {
      if (topupInvoices.length === 0) {
        tbody.innerHTML = `<tr><td colspan="4" style="text-align:center;color:var(--muted);padding:1.5rem">No top-ups found yet.</td></tr>`;
      } else {
        tbody.innerHTML = topupInvoices.map(inv => `
          <tr>
            <td style="font-family:monospace;font-size:.85rem">${inv.number}</td>
            <td>${inv.issue_date}</td>
            <td style="color:var(--accent2);font-weight:700">+$${parseFloat(inv.total).toFixed(2)}</td>
            <td><span class="badge badge-active" style="background:rgba(0,212,170,.15);color:var(--accent2)">Paid</span></td>
          </tr>
        `).join('');
      }
    }
  } catch (err) {
    console.error('Failed to load invoices / balance:', err);
  }
}

// Set top-up amount when clicking quick buttons
function selectQuickAmount(amount) {
  const input = document.getElementById('topup-custom-amount');
  if (input) {
    input.value = amount.toFixed(2);
  }
  
  // Update button visual style
  document.querySelectorAll('.amount-btn').forEach(btn => {
    const btnAmt = parseFloat(btn.dataset.amount);
    if (btnAmt === amount) {
      btn.classList.add('active');
    } else {
      btn.classList.remove('active');
    }
  });
}

// Submeter recarga de saldo
async function executeTopup() {
  const token = requireAuth();
  const orgId = document.getElementById('topup-org').value;
  const amountInput = document.getElementById('topup-custom-amount');
  const amount = parseFloat(amountInput ? amountInput.value : 0);
  const email = localStorage.getItem('management_email') || '';

  if (!orgId) {
    showToast('❌ Please select or create an organization first.', 'error');
    return;
  }

  if (isNaN(amount) || amount <= 0) {
    showToast('❌ Please enter a valid amount greater than $0.', 'error');
    return;
  }

  const btn = document.getElementById('topup-btn');
  if (btn) {
    btn.disabled = true;
    btn.textContent = 'Processing Top-up...';
  }

  try {
    const res = await fetch(`${API_BASE}/topup`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`
      },
      body: JSON.stringify({
        org_id: orgId,
        amount: amount,
        email: email
      })
    });

    if (!res.ok) {
      const err = await res.json();
      showToast(`❌ Top-up failed: ${err.detail || 'Server error'}`, 'error');
      if (btn) {
        btn.disabled = false;
        btn.textContent = 'Top-up Now';
      }
      return;
    }

    const data = await res.json();
    showToast(`✅ Top-up of $${amount.toFixed(2)} completed successfully!`);
    
    // Atualizar UI
    if (amountInput) amountInput.value = '';
    document.querySelectorAll('.amount-btn').forEach(b => b.classList.remove('active'));
    
    await loadInvoicesAndBalance(token, email);

  } catch (err) {
    showToast(`❌ Connection error: ${err.message}`, 'error');
  } finally {
    if (btn) {
      btn.disabled = false;
      btn.textContent = 'Top-up Now';
    }
  }
}

// Save promo code to clipboard / localStorage for user convenience
function copyPromoCode(code, elementId) {
  navigator.clipboard.writeText(code).then(() => {
    showToast(`📋 Code "${code}" copied to clipboard!`);
    localStorage.setItem('active_promo_code', code);
    
    const el = document.getElementById(elementId);
    if (el) {
      const originalText = el.textContent;
      el.textContent = 'Copied!';
      el.style.background = 'var(--accent2)';
      el.style.borderColor = 'var(--accent2)';
      el.style.color = '#000';
      
      setTimeout(() => {
        el.textContent = originalText;
        el.style.background = '';
        el.style.borderColor = '';
        el.style.color = '';
      }, 2000);
    }
  }).catch(err => {
    localStorage.setItem('active_promo_code', code);
    showToast(`✅ Code "${code}" saved for checkout!`);
  });
}

function escapeHtml(str) {
  if (!str) return '';
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function showToast(msg, type = 'success') {
  const t = document.createElement('div');
  t.textContent = msg;
  const borderCol = type === 'success' ? 'var(--accent2)' : 'var(--red)';
  Object.assign(t.style, {
    position: 'fixed',
    bottom: '1.5rem',
    right: '1.5rem',
    background: 'var(--card)',
    border: `1px solid ${borderCol}`,
    borderRadius: '10px',
    padding: '.75rem 1.25rem',
    color: 'var(--text)',
    fontSize: '.875rem',
    fontWeight: '600',
    zIndex: 1000,
    boxShadow: '0 8px 24px rgba(0,0,0,.4)',
    animation: 'modalIn .2s ease'
  });
  document.body.appendChild(t);
  setTimeout(() => t.remove(), 3500);
}

function doLogout() {
  localStorage.removeItem('management_token');
  localStorage.removeItem('management_email');
  window.location.href = '/';
}
