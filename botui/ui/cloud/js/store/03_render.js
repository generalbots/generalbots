// ── Auth & render helpers ──
// ── Auth helpers ──
function requireAuth() {
  let t = localStorage.getItem('management_token');
  if (!t) {
    if (location.hostname === 'localhost' || location.hostname === '127.0.0.1') {
      localStorage.setItem('management_token', 'dev-token');
      localStorage.setItem('management_email', 'admin@generalbots.com');
      devAutoLogin();
      t = 'dev-token';
    } else {
      window.location.href = '/cloud/login';
    }
  }
  return t;
}

async function devAutoLogin() {
  if (location.hostname !== 'localhost' && location.hostname !== '127.0.0.1') return;
  try {
    const res = await fetch('/api/cloud/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email: 'admin@generalbots.com', password: 'dev' })
    });
    if (!res.ok) return;
    const data = await res.json();
    if (data.token) {
      localStorage.setItem('management_token', data.token);
      localStorage.setItem('management_email', data.email || 'admin@generalbots.com');
    }
  } catch (_) { /* silent */ }
}
function doLogout() {
  localStorage.removeItem('management_token');
  localStorage.removeItem('management_email');
  window.location.href = '/cloud';
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
  selectedPlan = plan;
  document.getElementById('modal-title').textContent = `Order: ${plan.name}`;
  document.getElementById('modal-body').innerHTML = `
    <div style="background:var(--bg3);border-radius:8px;padding:1rem;margin-bottom:1rem;border:1px solid var(--border)">
      <div style="font-size:1.3rem;font-weight:800;color:var(--text)">${plan.currency}${plan.amount}<small style="font-size:.75rem;font-weight:400;color:var(--muted)"> / ${plan.period}</small></div>
      <ul style="list-style:none;margin-top:.5rem">${plan.specs.map(s=>`<li style="font-size:.82rem;color:var(--muted);padding:.1rem 0">• ${esc(s)}</li>`).join('')}</ul>
    </div>
    <div class="form-group">
      <label class="form-label">Name this service (optional)</label>
      <input id="order-name" class="form-control" type="text" placeholder="e.g. WhatsApp Bot Server">
    </div>`;
  document.getElementById('purchase-modal').classList.add('open');
}

async function confirmOrder() {
  if (!selectedPlan) return;
  const btn = document.getElementById('modal-confirm-btn');
  btn.textContent = 'Processing…';
  btn.disabled = true;
  // Simulate provisioning
  await new Promise(r => setTimeout(r, 1200));
  btn.textContent = 'Confirm & Purchase';
  btn.disabled = false;
  hideModal();
  showToast(`✅ ${selectedPlan.name} provisioned! Check My Services.`);
}

function hideModal() { document.getElementById('purchase-modal').classList.remove('open'); }

// ── Domain helpers ──
let domainTimer;
function domainSearchDebounce() { clearTimeout(domainTimer); domainTimer = setTimeout(searchDomain, 600); }

function searchDomain() {
  const q = document.getElementById('domain-input')?.value?.trim();
  if (!q) return;
  const res = document.getElementById('domain-result');
  if (!res) return;
  // Simulate availability check
  const available = Math.random() > 0.3;
  const domain = q.includes('.') ? q : q + '.com';
  res.innerHTML = `
    <div style="background:var(--card);border:1px solid ${available?'rgba(0,212,170,.3)':'rgba(255,107,107,.3)'};border-radius:var(--radius);padding:1rem;margin-bottom:1rem;display:flex;align-items:center;gap:1rem">
      <span style="font-size:1.5rem">${available?'✅':'❌'}</span>
      <div style="flex:1">
        <div style="font-size:1rem;font-weight:700;color:var(--text)">${esc(domain)}</div>
        <div style="font-size:.82rem;color:var(--muted)">${available?'Available — register now!':'Not available. Try another extension.'}</div>
      </div>
      ${available?`<button class="btn btn-primary btn-sm" onclick="orderDomainName('${esc(domain)}')">Register $21.99/yr</button>`:''}
    </div>`;
}

function orderDomain(ext) {
  selectedPlan = { name: `Domain ${ext}`, amount: TLD_PRICES.find(t=>t.ext===ext)?.price?.replace('$','') || '21.99', currency:'$', period:'yr', specs:[`${ext} domain`,'Free WHOIS privacy','Managed DNS'] };
  document.getElementById('modal-title').textContent = `Register ${ext} Domain`;
  document.getElementById('modal-body').innerHTML = `
    <div class="form-group"><label class="form-label">Domain Name</label><input id="order-domain" class="form-control" type="text" placeholder="${ext.replace('.','')}.com"></div>
    <p style="font-size:.82rem;color:var(--muted)">Includes free WHOIS privacy, managed DNS and auto-renew.</p>`;
  document.getElementById('purchase-modal').classList.add('open');
}

function orderDomainName(domain) {
  selectedPlan = { name: domain, amount:'21.99', currency:'$', period:'yr', specs:['Free WHOIS privacy','Managed DNS'] };
  document.getElementById('modal-title').textContent = `Register ${domain}`;
  document.getElementById('modal-body').innerHTML = `<p style="color:var(--muted)">Register <strong style="color:var(--text)">${esc(domain)}</strong> for $21.99/yr including free WHOIS privacy.</p>`;
  document.getElementById('purchase-modal').classList.add('open');
}

// ── Number search ──
function searchNumbers() {
  const country = document.getElementById('ns-country').value;
  const tbody = document.getElementById('number-tbody');
  const mockNumbers = [
    `+1 415-555-0101`, `+1 415-555-0182`, `+1 415-555-0207`,
    `+1 650-555-0143`, `+1 650-555-0167`,
  ];
  tbody.innerHTML = mockNumbers.map(n => `
    <tr>
      <td style="font-family:monospace;font-size:.9rem">${n}</td>
      <td>United States</td>
      <td><span class="badge badge-active">SMS + Voice</span></td>
      <td>$5.99/mo</td>
      <td><button class="btn btn-secondary btn-sm" onclick="orderNumber('${n}')">Buy</button></td>
    </tr>`).join('');
}

function orderNumber(number) {
  selectedPlan = { name:'Virtual Number', amount:'5.99', currency:'$', period:'mo', specs:[number,'SMS + Voice','WhatsApp-ready'] };
  document.getElementById('modal-title').textContent = `Order Number ${number}`;
  document.getElementById('modal-body').innerHTML = `<p style="color:var(--muted)">Purchase <strong style="color:var(--text)">${esc(number)}</strong> for $5.99/mo with SMS + Voice capabilities.</p>`;
  document.getElementById('purchase-modal').classList.add('open');
}

// ── Toast ──
function showToast(msg) {
  const t = document.createElement('div');
  t.textContent = msg;
  Object.assign(t.style, { position:'fixed', bottom:'1.5rem', right:'1.5rem', background:'var(--card)', border:'1px solid var(--accent2)', borderRadius:'10px', padding:'.75rem 1.25rem', color:'var(--text)', fontSize:'.875rem', fontWeight:'600', zIndex:1000, boxShadow:'0 8px 24px rgba(0,0,0,.4)', animation:'modalIn .2s ease' });
  document.body.appendChild(t);
  setTimeout(() => t.remove(), 3500);
}

