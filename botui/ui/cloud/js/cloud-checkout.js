const API_BASE = '/api/cloud';

document.addEventListener('DOMContentLoaded', () => {
  const params = new URLSearchParams(window.location.search);
  const raw = params.get('payload');
  if (!raw) {
    document.querySelector('.saas-auth-card').innerHTML = '<h1>Invalid Checkout</h1><p>No plan data found. Please go back and choose a plan.</p><a href="https://pragmatismo.com.br/hosting" class="btn-primary">Back to Plans</a>';
    return;
  }
  try {
    const payload = JSON.parse(decodeURIComponent(raw));
    renderSummary(payload);
  } catch (e) {
    document.querySelector('.saas-auth-card').innerHTML = '<h1>Invalid Data</h1><p>Could not parse plan data.</p>';
  }
});

function renderSummary(payload) {
  const container = document.getElementById('order-summary');
  const planName = payload.plan.charAt(0).toUpperCase() + payload.plan.slice(1);
  const periodLabel = payload.period === 'yearly' ? '/yr' : '/mo';
  container.innerHTML = `
    <div class="saas-order-row">
      <span>${planName} Plan</span>
      <span>$${(payload.total || 0).toFixed(2)}${periodLabel}</span>
    </div>
    ${payload.storage > 0 ? `
    <div class="saas-order-row">
      <span>Storage: ${payload.storage}GB</span>
      <span>Included</span>
    </div>` : ''}
    ${payload.ai && payload.ai.length > 0 ? `
    <div class="saas-order-row">
      <span>AI: ${payload.ai.join(', ')}</span>
      <span>Included</span>
    </div>` : ''}
    <div class="saas-order-total">
      <span>Total</span>
      <span>$${(payload.total || 0).toFixed(2)}${periodLabel}</span>
    </div>
  `;
}

async function doCheckout(e) {
  e.preventDefault();
  const btn = e.target.querySelector('button[type="submit"]');
  btn.textContent = 'Processing...';
  btn.disabled = true;

  const email = document.getElementById('email').value;
  const orgName = document.getElementById('org-name').value;

  const params = new URLSearchParams(window.location.search);
  const raw = params.get('payload');
  const payload = JSON.parse(decodeURIComponent(raw));

  try {
    const res = await fetch(`${API_BASE}/checkout`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        payload: JSON.stringify(payload),
        email,
        organization_name: orgName || undefined,
        return_url: window.location.origin + '/cloud/dashboard',
      }),
    });

    if (!res.ok) {
      const err = await res.json();
      alert(err.detail || 'Checkout failed');
      btn.textContent = 'Proceed to Payment →';
      btn.disabled = false;
      return;
    }

    const data = await res.json();
    if (data.checkout_url) {
      window.location.href = data.checkout_url;
    } else {
      alert('Unexpected response');
      btn.textContent = 'Proceed to Payment →';
      btn.disabled = false;
    }
  } catch (err) {
    alert('Network error: ' + err.message);
    btn.textContent = 'Proceed to Payment →';
    btn.disabled = false;
  }
}
