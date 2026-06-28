// ── Product catalogue — General Bots Cloud Store ──
// Service consistency rules:
//   RECURRING  = billed monthly, tied to workspace (VPS, GPU, Storage, Numbers, LLM)
//   ONE-TIME   = purchase once, no subscription (Domains, Call top-ups, App Publishing)
//   MANAGED    = tied services that cannot be removed independently (bundled w/ offers)
//   UPGRADEABLE = can scale up/down anytime (VPS, Storage)
//   REMOVABLE   = can cancel anytime (Numbers, standalone GPU)

const SERVICE_META = {
  'virtual-machine': { recurring: true,  upgradeable: true,  removable: true,  managed: false },
  'gpu':             { recurring: true,  upgradeable: true,  removable: true,  managed: false },
  'storage':         { recurring: true,  upgradeable: true,  removable: true,  managed: false },
  'number':          { recurring: true,  upgradeable: false, removable: true,  managed: false },
  'llm-tokens':      { recurring: true,  upgradeable: true,  removable: true,  managed: false },
  'domain':          { recurring: false, upgradeable: false, removable: false, managed: false },
  'calls':           { recurring: false, upgradeable: false, removable: false, managed: false },
  'appstore':        { recurring: false, upgradeable: false, removable: false, managed: false },
};

const PROFILE_RECS = {
  whatsapp: {
    label: 'WhatsApp Bots',
    vps: { small: 'VM Small handles up to 5 bots comfortably. Scale to Medium for 20+.', medium: 'VM Medium recommended for multiple high-volume bots.', large: 'VM Large for dozens of simultaneous bots.' },
    gpu: 'GPU not needed for WhatsApp bots. VM handles the processing efficiently.',
    phone: 'Phone numbers are essential — each includes 1,000 free WhatsApp messages/month.',
    storage: '50 GB included is enough to start. Add more as you grow.',
    domain: 'A custom domain adds credibility to your bots.',
    summary: 'For WhatsApp Bots: VM Small + 1 phone number + .com domain. GPU not needed.'
  },
  llm: {
    label: 'AI / LLM',
    vps: { small: 'VM Small runs small models (1.5B). Medium handles 7B–8B.', medium: 'VM Medium ideal for LLaMA 3.1 8B and DeepSeek R1.', large: 'VM Large + GPU Pro for 20B+ models in production.' },
    gpu: 'GPU highly recommended for LLMs. GPU Basic for testing, Pro for production.',
    phone: 'Phone optional for LLMs. Prioritize VM and GPU.',
    storage: 'GGUF models range 1–40 GB. Invest in extra storage.',
    domain: 'Custom domain for your LLM API endpoint.',
    summary: 'For AI/LLM: VM Medium + GPU Pro is the ideal production starting point.'
  },
  webapp: {
    label: 'Web Apps',
    vps: { small: 'VM Small for simple low-traffic apps.', medium: 'VM Medium for apps with database + API.', large: 'VM Large for high-traffic with multiple services.' },
    gpu: 'GPU generally not needed for web apps.',
    phone: 'Phone numbers for 2FA and SMS notifications.',
    storage: 'Storage for uploads, assets and backups.',
    domain: 'Domain is essential for any web application.',
    summary: 'For Web Apps: VM Medium + .com domain + 50 GB storage is the go-to combo.'
  },
  enterprise: {
    label: 'Enterprise',
    vps: { small: 'Minimum VM Medium for enterprise.', medium: 'VM Medium for staging/QA environments.', large: 'VM Large+ for enterprise production with HA.' },
    gpu: 'A100 GPU for enterprise ML workloads.',
    phone: 'Business Pack (10 numbers) for corporate communications.',
    storage: 'Minimum 1 TB for logs, backups and customer data.',
    domain: 'Multiple domains recommended.',
    summary: 'Enterprise: VM Large + A100 GPU + Business Pack + 1 TB storage.'
  }
};

const CATALOGUE = {
  // ── Virtual Machines (renamed from VPS, upgradeable + removable) ──
  'vps-small': {
    tag: 'Virtual Machine', icon: '🖥️',
    title: 'Virtual Machines', cat: 'compute',
    desc: 'Linux VMs with full root access, NVMe SSD, DDoS protection. Scale up or down anytime. Cancel whenever you need.',
    bullets: ['Full root SSH access', 'NVMe SSD storage', 'DDoS protection', 'Hourly snapshots', 'Upgradeable anytime'],
    serviceType: 'virtual-machine',
    plans: [
      { id:'vps-small',  tier:'Small',  name:'VM Small',  amount:'9.99',  period:'mo', currency:'$', specs:['4 vCPU','8 GB RAM','100 GB NVMe','2 TB bandwidth'],  featured:false },
      { id:'vps-medium', tier:'Medium', name:'VM Medium', amount:'19.99', period:'mo', currency:'$', specs:['6 vCPU','16 GB RAM','200 GB NVMe','4 TB bandwidth'], featured:true  },
      { id:'vps-large',  tier:'Large',  name:'VM Large',  amount:'39.99', period:'mo', currency:'$', specs:['8 vCPU','32 GB RAM','400 GB NVMe','8 TB bandwidth'], featured:false },
      { id:'vps-xl',     tier:'XL',     name:'VM XL',     amount:'79.99', period:'mo', currency:'$', specs:['16 vCPU','64 GB RAM','800 GB NVMe','16 TB bandwidth'],featured:false },
    ]
  },
  'vps-medium': { alias: 'vps-small' },
  'vps-large':  { alias: 'vps-small' },
  'vps-xl':     { alias: 'vps-small' },

  // ── GPU (recurring, upgradeable, removable) ──
  'gpu-basic': {
    tag: 'GPU Computing', icon: '⚡',
    title: 'GPU Computing', cat: 'gpu',
    desc: 'Dedicated NVIDIA GPUs for AI inference, LLM fine-tuning and HPC. Add to any VM or use standalone. Upgrade between tiers at any time.',
    bullets: ['Dedicated NVIDIA GPU', 'CUDA 12 pre-installed', 'Persistent storage', 'Upgradeable anytime', 'Cancel independently'],
    serviceType: 'gpu',
    plans: [
      { id:'gpu-basic',      tier:'Basic',      name:'RTX 3060',  amount:'39.99',  period:'mo', currency:'$', specs:['RTX 3060 12GB','4 vCPU','8 GB RAM','100 GB NVMe'],   featured:false },
      { id:'gpu-pro',        tier:'Pro',        name:'RTX 4090',  amount:'99.99',  period:'mo', currency:'$', specs:['RTX 4090','8 vCPU','32 GB RAM','200 GB NVMe'],   featured:true  },
      { id:'gpu-enterprise', tier:'Enterprise', name:'A100 80GB', amount:'299.99', period:'mo', currency:'$', specs:['A100 80GB','16 vCPU','64 GB RAM','400 GB NVMe'], featured:false },
    ]
  },
  'gpu-pro':        { alias: 'gpu-basic' },
  'gpu-enterprise': { alias: 'gpu-basic' },

  // ── Storage (recurring, upgradeable, removable) ──
  'storage-50': {
    tag: 'Object Storage', icon: '💾',
    title: 'Object Storage', cat: 'storage',
    desc: 'S3-compatible storage for bot documents, backups and datasets. Upgrade capacity at any time without downtime.',
    bullets: ['S3-compatible API', 'Versioning & lifecycle rules', 'End-to-end encryption', 'Zero-egress option', 'Upgrade anytime'],
    serviceType: 'storage',
    plans: [
      { id:'storage-50',   tier:'Starter', name:'50 GB',   amount:'9.99',   period:'mo', currency:'$', specs:['50 GB storage','100 GB egress','S3-compatible','99.9% uptime'],    featured:false },
      { id:'storage-250',  tier:'Growth',  name:'250 GB',  amount:'29.99',  period:'mo', currency:'$', specs:['250 GB storage','500 GB egress','S3-compatible','Versioning'],      featured:true  },
      { id:'storage-1tb',  tier:'Scale',   name:'1 TB',    amount:'59.99',  period:'mo', currency:'$', specs:['1 TB storage','2 TB egress','Zero-egress opt.','Lifecycle mgmt'], featured:false },
      { id:'storage-10tb', tier:'Power',   name:'10 TB',   amount:'199.99', period:'mo', currency:'$', specs:['10 TB storage','20 TB egress','Zero-egress opt.','Priority'],      featured:false },
    ]
  },
  'storage-250': { alias:'storage-50' },
  'storage-1tb':  { alias:'storage-50' },
  'storage-10tb': { alias:'storage-50' },

  // ── Phone Numbers (recurring, removable, NOT upgradeable) ──
  'number-local': {
    tag: 'Phone Numbers', icon: '📞',
    title: 'Virtual Phone Numbers', cat: 'comms',
    desc: 'Local and international phone numbers for WhatsApp, SMS and voice in 150+ countries. Each number is an independent subscription.',
    bullets: ['150+ countries', 'WhatsApp Business ready', 'SMS + Voice', 'Instant activation', 'Cancel any number independently'],
    serviceType: 'number',
    plans: [
      { id:'number-local',    tier:'Local',    name:'Local Number',  amount:'5.99',  period:'mo', currency:'$', specs:['1 number','1 country','SMS + Voice','WhatsApp-ready'],           featured:false },
      { id:'number-global',   tier:'Global',   name:'Global Bundle', amount:'19.99', period:'mo', currency:'$', specs:['3 numbers','3+ countries','SMS + Voice','Priority routing'],     featured:true  },
      { id:'number-business', tier:'Business', name:'Business Pack', amount:'49.99', period:'mo', currency:'$', specs:['10 numbers','Any countries','SMS + Voice + WA','Dedicated'],    featured:false },
    ]
  },
  'number-global':   { alias:'number-local' },
  'number-business': { alias:'number-local' },

  'number-search': {
    tag: 'Search Numbers', icon: '🔍',
    title: 'Find Available Numbers', cat: 'comms',
    desc: 'Search for available numbers by country and capabilities.',
    bullets: ['Search by country','Filter by SMS, Voice, WhatsApp','Instant purchase','Port your existing number'],
    type: 'number-search'
  },

  // ── Call top-ups (ONE-TIME, not recurring) ──
  'calls-100': {
    tag: 'Call Minutes — Top-Up', icon: '📱',
    title: 'Call Minute Bundles', cat: 'comms',
    desc: 'Pre-paid outbound call minutes. One-time purchase — top up whenever you need. No subscription, no commitment.',
    bullets: ['One-time purchase','Global outbound calls','HD voice quality','CDR analytics'],
    serviceType: 'calls',
    plans: [
      { id:'calls-100',  tier:'Starter', name:'100 Minutes',  amount:'9.99',  period:'once', currency:'$', specs:['100 min outbound','Global coverage','HD voice','CDR report'],       featured:false },
      { id:'calls-500',  tier:'Growth',  name:'500 Minutes',  amount:'39.99', period:'once', currency:'$', specs:['500 min outbound','Global coverage','HD voice','Priority routing'], featured:true  },
      { id:'calls-1000', tier:'Power',   name:'1,000+ Minutes',amount:'69.99',period:'once', currency:'$', specs:['1000 min outbound','Global coverage','HD voice','Dedicated'],       featured:false },
    ]
  },
  'calls-500':  { alias:'calls-100' },
  'calls-1000': { alias:'calls-100' },
  'sim': { alias:'calls-100' },

  // ── Domains (ONE-TIME/annual, NOT removable once registered) ──
  'domain-search': {
    tag: 'Domains — Annual', icon: '🌐',
    title: 'Domain Registration', cat: 'apps',
    desc: 'Register the perfect domain. Annual billing — renew yearly. Includes free WHOIS privacy and managed DNS. Domains cannot be removed after purchase.',
    bullets: ['Annual billing','Free WHOIS privacy','Managed DNS','SSL ready','Auto-renew available'],
    serviceType: 'domain',
    type: 'domain-search'
  },
  'domain-tlds': {
    tag: 'Domains', icon: '🌐',
    title: 'Domain Extensions & Pricing', cat: 'apps',
    desc: 'Popular TLDs with free WHOIS protection, DNS management and auto-renew.',
    bullets: ['Free WHOIS privacy','Managed DNS','SSL ready','Transfer-in supported'],
    serviceType: 'domain',
    type: 'domain-tlds'
  },
  'domains': { alias:'domain-tlds' },
  'numbers': { alias:'number-local' },
};

const TLD_PRICES = [
  { ext:'.com', price:'$21.99', period:'yr' },
  { ext:'.net', price:'$25.99', period:'yr' },
  { ext:'.org', price:'$23.99', period:'yr' },
  { ext:'.io',  price:'$71.99', period:'yr' },
  { ext:'.ai',  price:'$159.99',period:'yr' },
  { ext:'.app', price:'$29.99', period:'yr' },
  { ext:'.dev', price:'$27.99', period:'yr' },
  { ext:'.co',  price:'$35.99', period:'yr' },
  { ext:'.me',  price:'$19.99', period:'yr' },
  { ext:'.info',price:'$17.99', period:'yr' },
  { ext:'.biz', price:'$21.99', period:'yr' },
  { ext:'.us',  price:'$15.99', period:'yr' },
];

// ── Store landing — three paths to purchase ──
function renderStoreLanding() {
  const right = document.getElementById('store-right');
  right.innerHTML = `
    <!-- Mission strip -->
    <div class="store-hero" style="background:linear-gradient(135deg,#060d0a 0%,#0a1a10 60%,#060e08 100%);padding:2rem 2rem 1.5rem">
      <div class="store-hero-body">
        <div class="store-hero-tag">General Bots Cloud Store</div>
        <div class="store-hero-title" style="font-size:1.5rem;margin:.5rem 0 .4rem">Your sovereign AI infrastructure</div>
        <div class="store-hero-desc">Buy individual services, compose a bundle with Offers, or use the Calculator to price your full Private Cloud — your way.</div>
      </div>
    </div>

    <!-- Three purchase paths -->
    <div style="padding:1.5rem 2rem 1rem">
      <div style="font-size:.68rem;font-weight:700;letter-spacing:.1em;text-transform:uppercase;color:var(--muted);margin-bottom:.75rem">How to buy</div>
      <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:.75rem;margin-bottom:2rem">
        <a href="/cloud/offers" style="text-decoration:none;display:flex;flex-direction:column;gap:.55rem;background:var(--card);border:1px solid rgba(255,0,128,.2);border-radius:var(--radius);padding:1.15rem;transition:all .18s" onmouseover="this.style.borderColor='var(--accent)'" onmouseout="this.style.borderColor='rgba(255,0,128,.2)'">
          <div style="font-size:1.4rem">⭐</div>
          <div style="font-size:.88rem;font-weight:700;color:var(--text)">Offers &amp; Bundles</div>
          <div style="font-size:.75rem;color:var(--muted);line-height:1.5">Pre-composed packages at the best price. Multiple services at once, single invoice.</div>
          <div style="font-size:.78rem;font-weight:600;color:var(--accent);margin-top:auto">Browse Offers →</div>
        </a>
        <div style="display:flex;flex-direction:column;gap:.55rem;background:var(--card);border:1px solid var(--border);border-radius:var(--radius);padding:1.15rem;cursor:pointer;transition:all .18s" onclick="document.getElementById('calc-panel').scrollIntoView({behavior:'smooth'})" onmouseover="this.style.borderColor='rgba(255,0,128,.2)'" onmouseout="this.style.borderColor='var(--border)'">
          <div style="font-size:1.4rem">🧮</div>
          <div style="font-size:.88rem;font-weight:700;color:var(--text)">Calculator</div>
          <div style="font-size:.75rem;color:var(--muted);line-height:1.5">Configure VPS, GPU, storage, phone and LLM tokens. See your total before committing.</div>
          <div style="font-size:.78rem;font-weight:600;color:var(--accent);margin-top:auto">Use Calculator →</div>
        </div>
        <div style="display:flex;flex-direction:column;gap:.55rem;background:var(--card);border:1px solid var(--border);border-radius:var(--radius);padding:1.15rem;cursor:pointer;transition:all .18s" onclick="document.getElementById('store-categories').scrollIntoView({behavior:'smooth'})" onmouseover="this.style.borderColor='rgba(255,0,128,.2)'" onmouseout="this.style.borderColor='var(--border)'">
          <div style="font-size:1.4rem">🛒</div>
          <div style="font-size:.88rem;font-weight:700;color:var(--text)">Individual Services</div>
          <div style="font-size:.75rem;color:var(--muted);line-height:1.5">Pick exactly what you need. Each service managed independently — upgrade or cancel anytime.</div>
          <div style="font-size:.78rem;font-weight:600;color:var(--accent);margin-top:auto">Browse Store →</div>
        </div>
      </div>

      <!-- Categories -->
      <div id="store-categories">
        <div style="font-size:.68rem;font-weight:700;letter-spacing:.1em;text-transform:uppercase;color:var(--muted);margin-bottom:.75rem">Services catalog</div>
        <div class="store-landing">` +
    [
      { cat:'compute',  icon:'🖥️', name:'Virtual Machines',    badge:'Upgradeable', desc:'Linux VMs with full root access, NVMe SSD, DDoS protection. Scale up or down anytime.' },
      { cat:'gpu',      icon:'⚡',  name:'GPU Computing',       badge:'Upgradeable', desc:'Dedicated NVIDIA GPUs for AI inference and LLM fine-tuning. Add to any VM.' },
      { cat:'storage',  icon:'💾',  name:'Object Storage',      badge:'Upgradeable', desc:'S3-compatible storage for bot data, backups and media. Grow capacity on demand.' },
      { cat:'comms',    icon:'📞',  name:'Phone Numbers',       badge:'Recurring',   desc:'Virtual numbers for WhatsApp, SMS and voice in 150+ countries.' },
      { cat:'apps',     icon:'🌐',  name:'Domains',             badge:'Annual',      desc:'Domain registration with free WHOIS privacy, DNS and auto-renew.' },
      { cat:'machines', icon:'🖥',  name:'Physical Machines',   badge:'Partner',     desc:'Certified on-premises AI hardware. RTX builds to GPU clusters via global distributors.' },
    ].map(c => `
          <div class="store-landing-card" onclick="location.href='/cloud/store?cat=${c.cat}'">
            <div style="display:flex;align-items:center;justify-content:space-between">
              <div class="store-landing-icon">${c.icon}</div>
              <span style="font-size:.58rem;font-weight:700;letter-spacing:.07em;text-transform:uppercase;padding:.15rem .5rem;border-radius:99px;background:rgba(255,0,128,.1);color:var(--accent);border:1px solid rgba(255,0,128,.2)">${c.badge}</span>
            </div>
            <div class="store-landing-name">${c.name}</div>
            <div class="store-landing-desc">${c.desc}</div>
            <div class="store-landing-action">Browse ${c.name}</div>
          </div>`).join('') + `
        </div>
      </div>
    </div>`;
}
