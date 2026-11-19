// ── Auth: delegates to cloud.js -- anonymous browsing allowed ──
function doLogout() {
  localStorage.removeItem('management_token');
  localStorage.removeItem('management_email');
  window.location.href = '/';
}
function esc(s) {
  if (!s) return '';
  const d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

// ── State ──
let currentProduct = 'vps-small';
let selectedPlan = null;

function catalogPriceBadge(sku) {
  const p = getCatalogProduct(sku);
  if (!p) return '';
  const price = parseFloat(p.price).toFixed(2);
  return `<span style="font-size:.55rem;color:var(--accent2);margin-left:.3rem;opacity:.7" title="CRM verified">✓ ${p.currency || '$'}${price}</span>`;
}

// ── Product catalogue (all prices in USD, doubled) ──

function renderRight(productKey) {
  let data = CATALOGUE[productKey];
  if (!data) return;
  if (data.alias) data = CATALOGUE[data.alias];

  const right = document.getElementById('store-right');

  if (data.type === 'number-search') {
    right.innerHTML = `
      ${heroHtml(data)}
      <div class="store-content">
        <div class="number-search">
          <select id="ns-country">
            <option value="US">🇺🇸 United States (+1)</option>
            <option value="GB">🇬🇧 United Kingdom (+44)</option>
            <option value="BR">🇧🇷 Brazil (+55)</option>
            <option value="DE">🇩🇪 Germany (+49)</option>
            <option value="FR">🇫🇷 France (+33)</option>
            <option value="CA">🇨🇦 Canada (+1)</option>
            <option value="AU">🇦🇺 Australia (+61)</option>
            <option value="MX">🇲🇽 Mexico (+52)</option>
            <option value="ES">🇪🇸 Spain (+34)</option>
            <option value="IT">🇮🇹 Italy (+39)</option>
          </select>
          <select id="ns-cap">
            <option value="all">SMS + Voice</option>
            <option value="sms">SMS only</option>
            <option value="voice">Voice only</option>
          </select>
          <button class="btn btn-primary" onclick="searchNumbers()">Search</button>
        </div>
        <div style="background:var(--card);border:1px solid var(--border);border-radius:var(--radius);overflow:auto" id="number-results">
          <table class="numbers-table">
            <thead><tr><th>Number</th><th>Country</th><th>Capabilities</th><th>Monthly</th><th></th></tr></thead>
            <tbody id="number-tbody">
              <tr><td colspan="5" style="text-align:center;color:var(--muted);padding:2rem">Select a country and click Search</td></tr>
            </tbody>
          </table>
        </div>
      </div>`;
    return;
  }

  if (data.type === 'domain-search') {
    right.innerHTML = `
      ${heroHtml(data)}
      <div class="store-content">
        <div class="domain-search">
          <input type="text" id="domain-input" placeholder="yourbrand.com — type to search…" oninput="domainSearchDebounce()">
          <button class="btn btn-primary" onclick="searchDomain()">Check Availability</button>
        </div>
        <div id="domain-result"></div>
        <div class="mgmt-section-title" style="margin:1.25rem 0 .75rem">Popular extensions</div>
        <div class="domain-tld-grid">${TLD_PRICES.map(t=>`
          <div class="domain-tld-card" onclick="orderDomain('${t.ext}')">
            <div class="domain-tld-ext">${t.ext}</div>
            <div class="domain-tld-price"><strong>${t.price}</strong> / ${t.period}</div>
          </div>`).join('')}
        </div>
      </div>`;
    return;
  }

  if (data.type === 'domain-tlds') {
    right.innerHTML = `
      ${heroHtml(data)}
      <div class="store-content">
        <div class="domain-tld-grid">${TLD_PRICES.map(t=>`
          <div class="domain-tld-card" onclick="orderDomain('${t.ext}')">
            <div class="domain-tld-ext">${t.ext}</div>
            <div class="domain-tld-price"><strong>${t.price}</strong> / ${t.period}</div>
          </div>`).join('')}
        </div>
      </div>`;
    return;
  }

  // Default: plan cards
  right.innerHTML = `
    ${heroHtml(data)}
    <div class="store-content">
      <div class="store-plans-grid">
        ${(data.plans || []).map(p => `
          <div class="plan-card ${p.featured?'featured':''}" onclick="openOrder(${JSON.stringify(p)})">
            <div class="plan-tier">${esc(p.tier)}</div>
            <div class="plan-name">${esc(p.name)}</div>
            <div class="plan-price-row">
              <span class="plan-currency">${p.currency}</span>
              <span class="plan-amount">${p.amount}</span>
              <span class="plan-period">/ ${p.period}</span>
            </div>
            <ul class="plan-specs">${p.specs.map(s=>`<li>${esc(s)}</li>`).join('')}</ul>
            <button class="plan-cta">Setup</button>
          </div>`).join('')}
      </div>
    </div>`;
}

function heroHtml(data) {
  return `
    <div class="store-hero">
      <div class="store-hero-icon">${data.icon}</div>
      <div class="store-hero-body">
        <div class="store-hero-tag">${data.tag}</div>
        <div class="store-hero-title">${data.title}</div>
        <div class="store-hero-desc">${data.desc}</div>
        <div class="store-hero-bullets">${(data.bullets||[]).map(b=>`<span class="store-hero-bullet">${b}</span>`).join('')}</div>
      </div>
    </div>`;
}

// ── Order modal ──
function openOrder(plan) {
  selectedPlan = plan || null;
  const name = selectedPlan && selectedPlan.name ? String(selectedPlan.name) : gbT('cloud-store-order-service', 'Order Service');
  const detail = selectedPlan
    ? `${name}${selectedPlan.amount ? ` — ${selectedPlan.currency || '$'}${selectedPlan.amount}${selectedPlan.period ? `/${selectedPlan.period}` : ''}` : ''}`
    : name;
  openSalesContact(gbT('cloud-store-purchase-request', 'Purchase request'), detail);
}

async function confirmOrder() {
  // No automatic purchase/provisioning is available in this environment.
  openSalesContact(
    gbT('cloud-store-purchase-request', 'Purchase request'),
    selectedPlan && selectedPlan.name ? String(selectedPlan.name) : ''
  );
}

function hideModal() {
  document.getElementById('purchase-modal').classList.remove('open');
  resetConfirmButton();
}

// ── Domain helpers ──
let domainTimer;
function domainSearchDebounce() { clearTimeout(domainTimer); domainTimer = setTimeout(searchDomain, 600); }

function searchDomain() {
  const q = document.getElementById('domain-input')?.value?.trim();
  if (!q) return;
  const res = document.getElementById('domain-result');
  if (!res) return;
  const domain = q.includes('.') ? q : q + '.com';
  res.innerHTML = `
    <div style="background:var(--card);border:1px solid var(--border);border-radius:var(--radius);padding:1rem;margin-bottom:1rem;display:flex;align-items:center;gap:1rem">
      <span style="font-size:1.5rem" aria-hidden="true">ℹ️</span>
      <div style="flex:1">
        <div style="font-size:1rem;font-weight:700;color:var(--text)">${esc(domain)}</div>
        <div style="font-size:.82rem;color:var(--muted)">${gbT('cloud-store-domain-unsupported', 'Automated availability checks are not available in this environment. Contact sales to check availability and register this domain.')}</div>
      </div>
      <button class="btn btn-secondary btn-sm" onclick="openSalesContact(gbT('cloud-store-domain-sales-subject', 'Domain registration request'), '${esc(domain)}')">${gbT('cloud-store-contact-sales', 'Contact sales')}</button>
    </div>`;
  // #2539 — there is no automated availability service behind this page: the
  // old request went to `/api/store/domain/check`, which no route serves (the
  // API lives under `/api/cloud/…`), so it only produced a 404 in the console.
  // The panel above already tells the truth (contact sales) and opens the
  // request form with the domain pre-filled, which is the only path that can
  // actually register it.
}

function openSalesContact(subject, detail) {
  selectedPlan = null;
  const text = detail ? `${subject}: ${detail}` : subject;
  document.getElementById('modal-title').textContent = gbT('cloud-store-contact-sales', 'Contact sales');
  document.getElementById('modal-body').innerHTML = `
    <p style="color:var(--muted);margin-bottom:.75rem">${gbT('cloud-store-sales-unavailable', 'This item cannot be purchased automatically yet. Our team can confirm availability and fulfillment.')}</p>
    <div style="background:var(--bg3);border-radius:8px;padding:1rem;border:1px solid var(--border)">
      <div style="font-size:.82rem;color:var(--text);font-weight:600;margin-bottom:.25rem">${esc(subject)}</div>
      <div style="font-size:.82rem;color:var(--muted);word-break:break-word">${esc(detail || '')}</div>
    </div>
    <p style="font-size:.82rem;color:var(--muted);margin-top:.75rem">${gbT('cloud-store-sales-next', 'Use your sales/support channel to request this item. No purchase or provisioning was started.')}</p>`;
  document.getElementById('purchase-modal').classList.add('open');
  const btn = document.getElementById('modal-confirm-btn');
  if (btn) {
    btn.textContent = gbT('cloud-store-close', 'Close');
    btn.disabled = false;
    btn.onclick = function () { hideModal(); resetConfirmButton(); };
  }
}

function resetConfirmButton() {
  const btn = document.getElementById('modal-confirm-btn');
  if (!btn) return;
  btn.textContent = gbT('cloud-store-close', 'Close');
  btn.disabled = false;
  btn.onclick = function () { hideModal(); };
}

function orderDomain(ext) {
  openSalesContact(gbT('cloud-store-domain-sales-subject', 'Domain registration request'), `.${String(ext || '').replace(/^\./, '')}`);
}

function orderDomainName(domain) {
  openSalesContact(gbT('cloud-store-domain-sales-subject', 'Domain registration request'), String(domain || ''));
}

// ── Number search ──
function searchNumbers() {
  const tbody = document.getElementById('number-tbody');
  if (!tbody) return;
  tbody.innerHTML = `
    <tr>
      <td colspan="5" style="text-align:center;color:var(--muted);padding:2rem">
        ${gbT('cloud-store-numbers-unsupported', 'Live number inventory is not available in this environment. No numbers are shown or sold automatically.')}
        <div style="margin-top:.75rem"><button class="btn btn-secondary btn-sm" onclick="openSalesContact(gbT('cloud-store-number-sales-subject', 'Virtual number request'), document.getElementById('ns-country')?.value || '')">${gbT('cloud-store-contact-sales', 'Contact sales')}</button></div>
      </td>
    </tr>`;
}

function orderNumber(number) {
  openSalesContact(gbT('cloud-store-number-sales-subject', 'Virtual number request'), String(number || ''));
}

