const CAT_MAP = { compute: 'vps-small', gpu: 'gpu-basic', storage: 'storage-50', comms: 'number-local', apps: 'domain-search' };

// ── Product catalogue — dynamically loaded from API ──
// All prices, specs, and descriptions come from /api/products/items

let CATALOGUE = {};     // Populated by loadCatalogProducts()
let TLD_PRICES = [];    // Populated from domain-type products
let PRODUCT_MAP = {};   // All products keyed by sku

async function loadCatalogProducts() {
  let items;
  try {
    const res = await fetch('/api/products/items?limit=100');
    if (!res.ok) throw new Error('HTTP ' + res.status);
    items = await res.json();
  } catch(e) {
    console.warn('Catalog API unavailable, using fallback:', e);
  }

  if (!items || !items.length) {
    items = getFallbackProducts();
  }

  // Build product map
  PRODUCT_MAP = {};
  for (const item of items) PRODUCT_MAP[item.sku] = item;

  // Build TLD_PRICES from domain products
  TLD_PRICES = items
    .filter(p => p.sku && p.sku.startsWith('domain-') && p.sku !== 'domain-search')
    .map(p => ({
      ext: '.' + (p.attributes?.tld || p.sku.replace('domain-', '')),
      price: '$' + parseFloat(p.price).toFixed(2),
      period: 'yr',
      sku: p.sku
    }));

  // Build CATALOGUE from infrastructure + communication products
  CATALOGUE = buildCatalogue(items);

  // Update store UI if already rendered
  syncStoreFromAPI();
  return items;
}

function getFallbackProducts() {
  return [
    { sku: 'vps-small', name: 'Small', price: 9.99, description: '4 vCPU · 8 GB RAM · 100 GB NVMe', attributes: { vcpu: 4, ram_gb: 8, storage_gb: 100, bandwidth_tb: 2 } },
    { sku: 'vps-medium', name: 'Medium', price: 19.99, description: '6 vCPU · 16 GB RAM · 200 GB NVMe', attributes: { vcpu: 6, ram_gb: 16, storage_gb: 200, bandwidth_tb: 4 } },
    { sku: 'vps-large', name: 'Large', price: 39.99, description: '8 vCPU · 32 GB RAM · 400 GB NVMe', attributes: { vcpu: 8, ram_gb: 32, storage_gb: 400, bandwidth_tb: 8 } },
    { sku: 'vps-xl', name: 'XL', price: 79.99, description: '16 vCPU · 64 GB RAM · 800 GB NVMe', attributes: { vcpu: 16, ram_gb: 64, storage_gb: 800, bandwidth_tb: 16 } },
    { sku: 'gpu-basic', name: 'GPU Basic', price: 39.99, description: 'NVIDIA RTX 3060 · 12 GB VRAM', attributes: { gpu_model: 'RTX 3060', vram_gb: 12, vcpu: 4, ram_gb: 16, storage_gb: 100 } },
    { sku: 'gpu-pro', name: 'GPU Pro', price: 99.99, description: 'NVIDIA RTX 4090 · 24 GB VRAM', attributes: { gpu_model: 'RTX 4090', vram_gb: 24, vcpu: 8, ram_gb: 32, storage_gb: 200 } },
    { sku: 'gpu-enterprise', name: 'GPU Enterprise', price: 299.99, description: 'NVIDIA A100 · 80 GB VRAM', attributes: { gpu_model: 'A100', vram_gb: 80, vcpu: 16, ram_gb: 64, storage_gb: 500 } },
    { sku: 'storage-50', name: 'Starter Storage', price: 9.99, description: '50 GB S3-compatible storage', attributes: { size_gb: 50 } },
    { sku: 'storage-250', name: 'Growth Storage', price: 29.99, description: '250 GB S3-compatible storage', attributes: { size_gb: 250 } },
    { sku: 'storage-1tb', name: 'Scale Storage', price: 99.99, description: '1 TB S3-compatible storage', attributes: { size_gb: 1000 } },
    { sku: 'storage-10tb', name: 'Power Storage', price: 299.99, description: '10 TB S3-compatible storage', attributes: { size_gb: 10000 } },
    { sku: 'number-local', name: 'Local Number', price: 5.99, description: 'Virtual number with SMS + Voice', attributes: { type: 'local' } },
    { sku: 'number-global', name: 'Global Number', price: 19.99, description: 'Multi-country virtual number', attributes: { type: 'global' } },
    { sku: 'number-business', name: 'Business Number', price: 49.99, description: 'Toll-free + local numbers bundle', attributes: { type: 'business' } },
    { sku: 'domain-com', name: '.com Domain', price: 21.99, description: '.com domain with free WHOIS privacy', attributes: { tld: 'com' } },
    { sku: 'domain-org', name: '.org Domain', price: 19.99, description: '.org domain with free WHOIS privacy', attributes: { tld: 'org' } },
    { sku: 'domain-net', name: '.net Domain', price: 21.99, description: '.net domain with free WHOIS privacy', attributes: { tld: 'net' } },
    { sku: 'domain-io', name: '.io Domain', price: 71.99, description: '.io domain with free WHOIS privacy', attributes: { tld: 'io' } },
    { sku: 'llm-tokens-1m', name: '1M LLM Tokens', price: 9.99, description: '1 million LLM tokens', attributes: {} },
    { sku: 'llm-tokens-10m', name: '10M LLM Tokens', price: 79.99, description: '10 million LLM tokens', attributes: {} },
  ];
}

const GB_ICON = '<svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="24" cy="24" r="16"/><circle cx="24" cy="24" r="6"/><line x1="24" y1="4" x2="24" y2="10"/><line x1="24" y1="38" x2="24" y2="44"/><line x1="4" y1="24" x2="10" y2="24"/><line x1="38" y1="24" x2="44" y2="24"/><line x1="10.5" y1="10.5" x2="14.5" y2="14.5"/><line x1="33.5" y1="33.5" x2="37.5" y2="37.5"/><line x1="37.5" y1="10.5" x2="33.5" y2="14.5"/><line x1="14.5" y1="33.5" x2="10.5" y2="37.5"/></svg>';

function buildCatalogue(items) {
  const C = {};

  // Helper: add product group
  function addGroup(skus, group) {
    const plans = skus.map(sku => items.find(p => p.sku === sku)).filter(Boolean);
    if (!plans.length) return;
    const first = plans[0];
    C[skus[0]] = {
      tag: group.tag, icon: group.icon,
      title: group.title, cat: group.cat,
      desc: first.description || group.desc,
      bullets: group.bullets || [],
      serviceType: group.serviceType,
      plans: plans.map(p => ({
        id: p.sku,
        tier: group.tierMap ? group.tierMap[p.sku] : p.sku,
        name: p.name,
        amount: parseFloat(p.price).toFixed(2),
        period: group.period || 'mo',
        currency: '$',
        specs: buildSpecs(p, group.specKeys),
        featured: p.sku === (group.featured || '')
      }))
    };
    // Aliases
    for (const sku of skus.slice(1)) C[sku] = { alias: skus[0] };
  }

  function buildSpecs(product, keys) {
    const a = product.attributes || {};
    const parts = [];
    if (keys) {
      for (const key of keys) {
        const val = a[key];
        if (val !== undefined && val !== null) {
          const label = (key === 'vcpu' ? 'vCPU' : key === 'ram_gb' ? 'GB RAM' : key === 'storage_gb' ? 'GB' : key === 'vram_gb' ? 'GB VRAM' : key === 'bandwidth_tb' ? 'TB bandwidth' : key === 'storage_type' ? '' : key === 'gpu_model' ? '' : key);
          if (key === 'storage_type') continue;
          if (key === 'gpu_model') { parts.push(val); continue; }
          parts.push(val + (label ? ' ' + label : ''));
        }
      }
    }
    return parts.length ? parts : [product.description || product.name];
  }

  addGroup(['vps-small','vps-medium','vps-large','vps-xl'], {
    tag: 'Virtual Machine', icon: GB_ICON,
    title: 'Virtual Machines', cat: 'compute',
    desc: 'Linux VMs with full root access, NVMe SSD, DDoS protection.',
    bullets: ['Full root SSH access','NVMe SSD storage','DDoS protection','Hourly snapshots','Upgradeable anytime'],
    serviceType: 'virtual-machine',
    period: 'mo',
    tierMap: { 'vps-small':'Small','vps-medium':'Medium','vps-large':'Large','vps-xl':'XL' },
    specKeys: ['vcpu','ram_gb','storage_gb','bandwidth_tb'],
    featured: 'vps-medium'
  });

  addGroup(['gpu-basic','gpu-pro','gpu-enterprise'], {
    tag: 'GPU Computing', icon: GB_ICON,
    title: 'GPU Computing', cat: 'gpu',
    desc: 'Dedicated NVIDIA GPUs for AI inference and LLM fine-tuning.',
    bullets: ['Dedicated NVIDIA GPU','CUDA 12 pre-installed','Persistent storage','Upgradeable anytime','Cancel independently'],
    serviceType: 'gpu',
    period: 'mo',
    tierMap: { 'gpu-basic':'Basic','gpu-pro':'Pro','gpu-enterprise':'Enterprise' },
    specKeys: ['gpu_model','vram_gb','vcpu','ram_gb','storage_gb'],
    featured: 'gpu-pro'
  });

  addGroup(['storage-50','storage-250','storage-1tb','storage-10tb'], {
    tag: 'Object Storage', icon: GB_ICON,
    title: 'Object Storage', cat: 'storage',
    desc: 'S3-compatible storage for bot data, backups and media.',
    bullets: ['S3-compatible API','Versioning & lifecycle rules','End-to-end encryption','Zero-egress option','Upgrade anytime'],
    serviceType: 'storage',
    period: 'mo',
    tierMap: { 'storage-50':'Starter','storage-250':'Growth','storage-1tb':'Scale','storage-10tb':'Power' },
    specKeys: ['size_gb'],
    featured: 'storage-250'
  });

  addGroup(['number-local','number-global','number-business'], {
    tag: 'Phone Numbers', icon: GB_ICON,
    title: 'Virtual Phone Numbers', cat: 'comms',
    desc: 'Virtual numbers for WhatsApp, SMS and voice.',
    bullets: ['150+ countries','WhatsApp Business ready','SMS + Voice','Instant activation','Cancel any number'],
    serviceType: 'number',
    period: 'mo',
    tierMap: { 'number-local':'Local','number-global':'Global','number-business':'Business' },
    specKeys: ['type'],
    featured: 'number-local'
  });

  // Domain search (not a plan group — uses search UI + TLD cards)
  C['domain-search'] = {
    type: 'domain-search',
    tag: 'Domains', icon: GB_ICON,
    title: 'Domain Names',
    cat: 'apps',
    desc: 'Register and manage domains with free WHOIS privacy and managed DNS.',
    bullets: ['Free WHOIS privacy','Managed DNS','Auto-renewal','Transfer assistance','150+ TLDs'],
  };

  return C;
}

// ── Sync hardcoded store UI with live API data ──
function syncStoreFromAPI() {
  if (!Object.keys(PRODUCT_MAP).length) return;

  // 1. Update calculator VPS buttons
  document.querySelectorAll('.calc-vps-btn').forEach(el => {
    const sku = el.dataset.value;
    const p = PRODUCT_MAP[sku];
    if (!p) return;
    const price = parseFloat(p.price).toFixed(2);
    el.dataset.price = price;
    const em = el.querySelector('em');
    if (em) em.innerHTML = '$' + price + '<small>/mo</small>';
    const spec = el.querySelector('.calc-vps-spec');
    if (spec) {
      const a = p.attributes || {};
      const parts = [];
      if (a.vcpu) parts.push(a.vcpu + ' vCPU');
      if (a.ram_gb) parts.push(a.ram_gb + ' GB RAM');
      if (a.storage_gb) parts.push(a.storage_gb + ' GB' + (a.storage_type === 'ssd' ? ' SSD' : ''));
      if (a.bandwidth_tb) parts.push(a.bandwidth_tb + ' TB bandwidth');
      if (parts.length) spec.textContent = parts.join(' \u00B7 ');
    }
  });

  // 2. Update calculator GPU buttons
  document.querySelectorAll('.calc-gpu-options .calc-addon-btn').forEach(el => {
    const sku = el.dataset.value || el.querySelector('input')?.value;
    if (!sku || sku === 'none') return;
    const p = PRODUCT_MAP[sku];
    if (!p) return;
    const price = parseFloat(p.price).toFixed(2);
    el.dataset.price = price;
    const small = el.querySelector('small');
    if (small) small.textContent = '$' + price;
  });

  // 3. Update calculator storage buttons
  document.querySelectorAll('.calc-storage-options .calc-addon-btn').forEach(el => {
    const input = el.querySelector('input');
    const val = input?.value;
    if (!val || val === '0') return;
    const sku = 'storage-' + val.replace('000', 'k').replace(/0+$/, '').replace('k', '000');
    const p = PRODUCT_MAP[sku];
    if (!p) return;
    const price = parseFloat(p.price).toFixed(2);
    el.dataset.price = price;
    const small = el.querySelector('small');
    if (small) small.textContent = '$' + price;
  });

  // 4. Update calculator phone buttons
  document.querySelectorAll('.calc-phone-options .calc-addon-btn').forEach(el => {
    const input = el.querySelector('input');
    const val = input?.value;
    if (!val || val === '0') return;
    const sku = val === '1' ? 'number-local' : val === '3' ? 'number-global' : val === '10' ? 'number-business' : null;
    if (!sku) return;
    const p = PRODUCT_MAP[sku];
    if (!p) return;
    const price = parseFloat(p.price).toFixed(2);
    el.dataset.price = price;
    const small = el.querySelector('small');
    if (small) small.textContent = '$' + price;
  });

  // 5. Update calculator domain buttons
  document.querySelectorAll('.calc-domain-options .calc-addon-btn').forEach(el => {
    const input = el.querySelector('input');
    const ext = input?.value;
    if (!ext) return;
    const tld = ext.replace('.', '');
    const p = PRODUCT_MAP['domain-' + tld];
    if (!p) return;
    const monthlyPrice = (parseFloat(p.price) / 12).toFixed(2);
    el.dataset.price = monthlyPrice;
    const small = el.querySelector('small');
    if (small) small.textContent = '$' + monthlyPrice;
  });

  // 6. Update LLM token prices (hardcoded but use product data)
  document.querySelectorAll('.calc-pkg-btn').forEach(btn => {
    const span = btn.querySelector('span');
    if (!span) return;
    const onclick = btn.getAttribute('onclick') || '';
    const match = onclick.match(/selectLLM\([^,]+,[^,]+,[^,]+/);
    // LLM prices are hardcoded in onclick - we keep them for now since the LLM products
    // use different pricing tiers than the simple token bundles
  });

  // 7. Update domain TLD cards in store landing
  document.querySelectorAll('.domain-tld-card').forEach(card => {
    const extEl = card.querySelector('.domain-tld-ext');
    const priceEl = card.querySelector('.domain-tld-price strong');
    if (!extEl || !priceEl) return;
    const ext = extEl.textContent.trim();
    const tld = ext.replace('.', '');
    const p = PRODUCT_MAP['domain-' + tld];
    if (!p) return;
    priceEl.textContent = '$' + parseFloat(p.price).toFixed(2);
    const extData = TLD_PRICES.find(t => t.ext === ext);
    if (extData) card.onclick = function() { orderDomain(ext); };
  });

  // 8. Update plan cards in store landing
  document.querySelectorAll('.plan-card').forEach(card => {
    const nameEl = card.querySelector('.plan-name');
    const amountEl = card.querySelector('.plan-amount');
    if (!nameEl || !amountEl) return;
    const name = nameEl.textContent.trim();
    const p = Object.values(PRODUCT_MAP).find(pp => pp.name === name);
    if (!p) return;
    amountEl.textContent = parseFloat(p.price).toFixed(2);
  });

  // 9. Recalc
  if (window.calcUpdate) calcUpdate();
}

// Keep loadCatalogProducts callable for backward compat
const catalogApiProducts = [];
function getCatalogProduct(sku) {
  return PRODUCT_MAP[sku] || null;
}

// ── Profile recommendations ──
const PROFILE_RECS = {
  whatsapp: {
    label: 'WhatsApp',
    vps: {
      small: 'Good fit for up to 5k contacts/day',
      medium: 'Recommended: handles 15k+ contacts/day',
      large: 'For high-volume campaigns (50k+)',
      xl: 'Enterprise scale (100k+ contacts)'
    },
    gpu: 'Not required for WhatsApp bots',
    summary: 'VPS + phone numbers for WhatsApp Business API'
  },
  llm: {
    label: 'AI / LLM',
    vps: {
      small: 'Suitable for API-based LLM calls',
      medium: 'Recommended for self-hosted small models',
      large: 'For fine-tuning or medium models (7B-13B)',
      xl: 'Enterprise inference (70B+ models)'
    },
    gpu: 'GPU accelerates local inference significantly',
    summary: 'VPS + GPU + LLM tokens for AI workloads'
  },
  webapp: {
    label: 'Web Apps',
    vps: {
      small: 'Great for static sites and small APIs',
      medium: 'Recommended for most web apps',
      large: 'High-traffic or database-heavy apps',
      xl: 'Multi-service or media-heavy applications'
    },
    gpu: 'Optional: video rendering or ML features',
    summary: 'VPS + storage for web applications'
  },
  enterprise: {
    label: 'Enterprise',
    vps: {
      small: 'Development and testing only',
      medium: 'Staging and QA environments',
      large: 'Recommended for production workloads',
      xl: 'Mission-critical with redundancy'
    },
    gpu: 'For enterprise AI pipelines',
    summary: 'Full stack: VPS + GPU + storage + phone'
  }
};

// ── Store landing page ──
const STORE_CATEGORIES = [
  { key: 'compute', icon: '\uD83D\uDDA5\uFE0F', name: 'Virtual Machines', desc: 'Linux VPS with NVMe SSD, hourly snapshots, DDoS protection. Starts at $7.99/mo.' },
  { key: 'gpu', icon: '\u26A1', name: 'GPU Computing', desc: 'Dedicated NVIDIA GPUs for AI inference, rendering, and HPC workloads.' },
  { key: 'storage', icon: '\uD83D\uDCBE', name: 'Object Storage', desc: 'S3-compatible storage with versioning, encryption, and zero-egress option.' },
  { key: 'comms', icon: '\uD83D\uDCDE', name: 'Phone Numbers', desc: 'Virtual numbers with SMS, voice, and WhatsApp Business API support.' },
  { key: 'apps', icon: '\uD83C\uDF10', name: 'Domains', desc: 'Register domains with free WHOIS privacy, managed DNS, and auto-renew.' }
];

function renderStoreLanding() {
  const right = document.getElementById('store-right');
  if (!right) return;
  right.innerHTML =
`<div class="mission-banner">
    <div class="mission-banner-inner">
      <div class="mission-badge">New: Sovereign Cloud</div>
      <div class="mission-title">Your data, your infrastructure, your rules.</div>
      <div class="mission-desc">
        General Bots Cloud lets you provision bare-metal servers, GPU nodes, object storage,
        phone numbers and domains — all in one place. No vendor lock-in, full root access,
        transparent pricing with no surprise bills.
      </div>
      <div class="mission-badges">
        <span class="mission-badge-sm">\uD83D\uDD12 End-to-end encryption</span>
        <span class="mission-badge-sm">\uD83C\uDF0D 150+ countries</span>
        <span class="mission-badge-sm">\u269B\uFE0F NVMe SSD storage</span>
        <span class="mission-badge-sm">\uD83D\uDEE1\uFE0F DDoS protection included</span>
      </div>
    </div>
  </div>
  <div class="store-landing">${STORE_CATEGORIES.map(c => {
    const p = PRODUCT_MAP[CAT_MAP[c.key]];
    const price = p ? '$' + parseFloat(p.price).toFixed(2) + '/mo' : '';
    return `<div class="store-landing-card" onclick="renderRight('${CAT_MAP[c.key] || c.key}')">
      <div class="store-landing-icon">${c.icon}</div>
      <div class="store-landing-name">${c.name}</div>
      <div class="store-landing-desc">${c.desc}</div>
      <div style="display:flex;align-items:center;justify-content:space-between;margin-top:auto">
        <span class="store-landing-action">Browse</span>
        ${price ? '<span style="font-size:.78rem;color:var(--accent2);font-weight:700">' + price + '</span>' : ''}
      </div>
    </div>`;
  }).join('')}</div>`;
}
