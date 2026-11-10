document.addEventListener('DOMContentLoaded', () => {
  const params = new URLSearchParams(window.location.search);
  const raw = params.get('payload');
  if (!raw) {
    document.querySelector('.saas-auth-card').innerHTML = '<h1>' + gbT('cloud-checkout-invalid-title', 'Invalid Checkout') + '</h1><p>' + gbT('cloud-checkout-invalid-desc', 'No plan data found. Please go back and choose a plan.') + '</p><a href="/plans" class="btn-primary">' + gbT('cloud-checkout-back', 'Back to Plans') + '</a>';
    return;
  }
  try {
    const payload = JSON.parse(decodeURIComponent(raw));
    renderSummary(payload);
  } catch (e) {
    document.querySelector('.saas-auth-card').innerHTML = '<h1>' + gbT('cloud-checkout-bad-data-title', 'Invalid Data') + '</h1><p>' + gbT('cloud-checkout-bad-data-desc', 'Could not parse plan data.') + '</p>';
  }
});

function renderSummary(payload) {
  const container = document.getElementById('order-summary');
  const planName = payload.plan === 'private-cloud'
    ? gbT('cloud-checkout-plan-private', 'Sovereign Private Cloud')
    : gbT('cloud-checkout-plan-' + payload.plan, payload.plan.charAt(0).toUpperCase() + payload.plan.slice(1) + ' Plan');
  const periodLabel = payload.period === 'yearly' ? gbT('cloud-plans-period-year', '/yr') : gbT('cloud-plans-period-month', '/mo');
  
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
        <span>• ${gbT('cloud-checkout-server-size', 'Server Size:')} ${vpsName}</span>
        <span>${gbT('cloud-checkout-included', 'Included')}</span>
      </div>
    `;
  }
  if (payload.storage && payload.storage > 0) {
    html += `
      <div class="saas-order-row" style="font-size: 0.8rem; opacity: 0.8; padding-left: 10px;">
        <span>• ${gbT('cloud-checkout-storage', 'Storage:')} ${payload.storage}GB</span>
        <span>${gbT('cloud-checkout-included', 'Included')}</span>
      </div>
    `;
  }
  if (payload.gpu && payload.gpu !== 'none') {
    const gpuName = payload.gpu.replace('gpu-', '').toUpperCase();
    html += `
      <div class="saas-order-row" style="font-size: 0.8rem; opacity: 0.8; padding-left: 10px;">
        <span>• ${gbT('cloud-checkout-gpu', 'Dedicated GPU:')} ${gpuName}</span>
        <span>${gbT('cloud-checkout-included', 'Included')}</span>
      </div>
    `;
  }
  if (payload.phone && payload.phone !== '0') {
    html += `
      <div class="saas-order-row" style="font-size: 0.8rem; opacity: 0.8; padding-left: 10px;">
        <span>• ${gbT('cloud-checkout-phone', 'Phone:')} ${gbT('cloud-checkout-phone-lines', '{count} Line(s)', { count: payload.phone })}</span>
        <span>${gbT('cloud-checkout-included', 'Included')}</span>
      </div>
    `;
  }
  if (payload.domain) {
    html += `
      <div class="saas-order-row" style="font-size: 0.8rem; opacity: 0.8; padding-left: 10px;">
        <span>• ${gbT('cloud-checkout-domain', 'Custom Domain:')} ${payload.domain}</span>
        <span>${gbT('cloud-checkout-included', 'Included')}</span>
      </div>
    `;
  }
  if (payload.ai && payload.ai.length > 0) {
    html += `
      <div class="saas-order-row" style="font-size: 0.8rem; opacity: 0.8; padding-left: 10px;">
        <span>• ${gbT('cloud-checkout-ai', 'AI Models:')} ${payload.ai.join(', ')}</span>
        <span>${gbT('cloud-checkout-included', 'Included')}</span>
      </div>
    `;
  }

  html += `
    <div class="saas-order-total" style="margin-top: 1rem; border-top: 1px dashed var(--border); padding-top: 0.75rem;">
      <span>${gbT('cloud-checkout-total', 'Total')}</span>
      <span>$${(payload.total || 0).toFixed(2)}${periodLabel}</span>
    </div>
  `;
  container.innerHTML = html;
}

async function doCheckout(e) {
  e.preventDefault();
  const btn = e.target.querySelector('button[type="submit"]');
  btn.textContent = gbT('cloud-checkout-processing', 'Processing...');
  btn.disabled = true;

  const params = new URLSearchParams(window.location.search);
  const raw = params.get('payload');
  const payload = JSON.parse(decodeURIComponent(raw));

  const token = getToken();
  var email = '';
  var orgName = '';
  if (token) {
    try {
      var parts = token.split('.');
      if (parts.length === 3) {
        var b64 = parts[1].replace(/-/g, '+').replace(/_/g, '/');
        var claims = JSON.parse(atob(b64));
        email = claims.email || claims.sub || '';
        orgName = claims.org_name || claims.org || '';
      }
    } catch (_) {}
  }

  try {
    const res = await fetch(`${API_BASE}/checkout`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' + token },
      body: JSON.stringify({
        payload: JSON.stringify(payload),
        email,
        organization_name: orgName || undefined,
        return_url: window.location.origin + '/dashboard',
      }),
    });

    if (!res.ok) {
      const err = await res.json();
      alert(err.detail || gbT('cloud-checkout-failed', 'Checkout failed'));
      btn.textContent = gbT('cloud-checkout-proceed', 'Proceed to Payment →');
      btn.disabled = false;
      return;
    }

    const data = await res.json();
    if (data.checkout_url) {
      window.location.href = data.checkout_url;
    } else {
      alert(gbT('cloud-checkout-unexpected', 'Unexpected response'));
      btn.textContent = gbT('cloud-checkout-proceed', 'Proceed to Payment →');
      btn.disabled = false;
    }
  } catch (err) {
    alert(gbT('cloud-settings-network-error', 'Network error:') + ' ' + err.message);
    btn.textContent = gbT('cloud-checkout-proceed', 'Proceed to Payment →');
    btn.disabled = false;
  }
}
