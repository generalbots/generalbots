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
  const planName = payload.plan === 'private-cloud' ? 'Sovereign Private Cloud' : (payload.plan.charAt(0).toUpperCase() + payload.plan.slice(1) + ' Plan');
  const periodLabel = payload.period === 'yearly' ? '/yr' : '/mo';
  
  let html = `
    <div class="saas-order-row">
      <span><strong>${planName}</strong></span>
      <span><strong>$${(payload.total || 0).toFixed(2)}${periodLabel}</strong></span>
    </div>
  `;

  if (payload.vps) {
    const vpsName = payload.vps.replace('vps-', 'VM ').toUpperCase();
    html += `
      <div class="saas-order-row" style="font-size: 0.8rem; opacity: 0.8; padding-left: 10px;">
        <span>• Server Size: ${vpsName}</span>
        <span>Included</span>
      </div>
    `;
  }
  if (payload.storage && payload.storage > 0) {
    html += `
      <div class="saas-order-row" style="font-size: 0.8rem; opacity: 0.8; padding-left: 10px;">
        <span>• Storage: ${payload.storage}GB</span>
        <span>Included</span>
      </div>
    `;
  }
  if (payload.gpu && payload.gpu !== 'none') {
    const gpuName = payload.gpu.replace('gpu-', '').toUpperCase();
    html += `
      <div class="saas-order-row" style="font-size: 0.8rem; opacity: 0.8; padding-left: 10px;">
        <span>• Dedicated GPU: ${gpuName}</span>
        <span>Included</span>
      </div>
    `;
  }
  if (payload.phone && payload.phone !== '0') {
    html += `
      <div class="saas-order-row" style="font-size: 0.8rem; opacity: 0.8; padding-left: 10px;">
        <span>• Phone: ${payload.phone} Line(s)</span>
        <span>Included</span>
      </div>
    `;
  }
  if (payload.domain) {
    html += `
      <div class="saas-order-row" style="font-size: 0.8rem; opacity: 0.8; padding-left: 10px;">
        <span>• Custom Domain: ${payload.domain}</span>
        <span>Included</span>
      </div>
    `;
  }
  if (payload.ai && payload.ai.length > 0) {
    html += `
      <div class="saas-order-row" style="font-size: 0.8rem; opacity: 0.8; padding-left: 10px;">
        <span>• AI Models: ${payload.ai.join(', ')}</span>
        <span>Included</span>
      </div>
    `;
  }

  html += `
    <div class="saas-order-total" style="margin-top: 1rem; border-top: 1px dashed var(--border); padding-top: 0.75rem;">
      <span>Total</span>
      <span>$${(payload.total || 0).toFixed(2)}${periodLabel}</span>
    </div>
  `;
  container.innerHTML = html;
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

  const token = getToken();

  try {
    const res = await fetch(`${API_BASE}/checkout`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
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
