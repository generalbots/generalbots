const CAT_MAP = { compute: 'vps-small', gpu: 'gpu-basic', storage: 'storage-50', comms: 'number-local', apps: 'domain-search' };

// ── Product catalogue — dynamically loaded from API ──
// All prices, specs, and descriptions come from /api/products/items

let CATALOGUE = {};     // Populated by loadCatalogProducts()
let TLD_PRICES = [];    // Populated from domain-type products
let PRODUCT_MAP = {};   // All products keyed by sku

async function loadCatalogProducts() {
  try {
    const res = await fetch('/api/products/items?limit=100');
    if (!res.ok) throw new Error('HTTP ' + res.status);
    const items = await res.json();

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
  } catch(e) {
    console.warn('Catalog API unavailable, using fallback:', e);
  }
}

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
    tag: 'Virtual Machine', icon: '\uD83D\uDDA5\uFE0F',
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
    tag: 'GPU Computing', icon: '\u26A1',
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
    tag: 'Object Storage', icon: '\uD83D\uDCBE',
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
    tag: 'Phone Numbers', icon: '\uD83D\uDCDE',
    title: 'Virtual Phone Numbers', cat: 'comms',
    desc: 'Virtual numbers for WhatsApp, SMS and voice.',
    bullets: ['150+ countries','WhatsApp Business ready','SMS + Voice','Instant activation','Cancel any number'],
    serviceType: 'number',
    period: 'mo',
    tierMap: { 'number-local':'Local','number-global':'Global','number-business':'Business' },
    specKeys: ['type'],
    featured: 'number-local'
  });

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
