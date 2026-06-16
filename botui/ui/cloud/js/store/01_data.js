// ── Product catalogue & calculator profiles ──

const PROFILE_RECS = {
  whatsapp: {
    label: 'WhatsApp Bots',
    vps: { small: 'VPS Small is ideal for up to 5 bots. Medium for up to 20.', medium: 'VPS Medium recommended for multiple high-volume bots.', large: 'VPS Large for dozens of simultaneous bots.' },
    gpu: 'GPU is not needed for WhatsApp bots. VPS handles the processing.',
    phone: 'Phone numbers are essential for WhatsApp. Each number comes with a thousand free messages!',
    storage: '50 GB is enough to start. Expand as needed.',
    domain: 'A custom domain gives your bots more credibility.',
    summary: 'For WhatsApp Bots, we recommend VPS Small/Medium + 1 phone number + .com domain. GPU is not needed.'
  },
  llm: {
    label: 'AI / LLM',
    vps: { small: 'VPS Small can run small models (1.5B). Medium for 7B-8B.', medium: 'VPS Medium ideal for LLaMA 3.1 8B and DeepSeek R1.', large: 'VPS Large + GPU Pro for 20B+ models with good performance.' },
    gpu: 'GPU is highly recommended for LLMs! GPU Basic for testing, GPU Pro for production.',
    phone: 'Phone is optional for LLMs. Focus on VPS and GPU first.',
    storage: 'GGUF models range from 1GB to 40GB. Invest in extra storage.',
    domain: 'Custom domain for your LLM API.',
    summary: 'For AI/LLM, we recommend VPS Medium+GPU Pro as the ideal starting point. VPS Large+GPU Enterprise for heavy production.'
  },
  webapp: {
    label: 'Web Apps',
    vps: { small: 'VPS Small for simple low-traffic apps.', medium: 'VPS Medium for apps with database + API.', large: 'VPS Large for high-traffic apps with multiple services.' },
    gpu: 'GPU is generally not needed for web apps.',
    phone: 'Phone numbers for 2FA authentication and SMS notifications.',
    storage: 'Storage for assets, uploads and backups.',
    domain: 'Domain essential for any web application.',
    summary: 'For Web Apps, VPS Medium + .com domain + 50GB storage is the ideal combo.'
  },
  enterprise: {
    label: 'Enterprise',
    vps: { small: 'Minimum VPS Medium for enterprise environments.', medium: 'VPS Medium for staging/QA environments.', large: 'VPS Large+ for enterprise production with high availability.' },
    gpu: 'Enterprise GPU (A100) for enterprise ML workloads.',
    phone: 'Business Pack (10 numbers) for corporate communications.',
    storage: 'Minimum 1 TB for logs, backups and customer data.',
    domain: 'Multiple domains recommended.',
    summary: 'For Enterprise, we recommend VPS Large + Enterprise GPU + Business Pack + 1 TB storage.'
  }
};


const CATALOGUE = {
  'vps-small': {
    tag: '🖥️ Virtual Server', icon: '🖥️',
    title: 'VPS Small', cat: 'Compute',
    desc: 'Perfect for WhatsApp bots, lightweight APIs and dev environments. Full root access, NVMe SSD, instant provisioning.',
    bullets: ['Full root SSH access', 'NVMe SSD storage', 'DDoS protection included', 'Hourly snapshots'],
    plans: [
      { id:'vps-small', tier:'Small', name:'VPS Small', amount:'9.99', period:'mo', currency:'$', specs:['4 vCPU','8 GB RAM','100 GB NVMe','2 TB bandwidth'], featured:false },
      { id:'vps-medium', tier:'Medium', name:'VPS Medium', amount:'19.99', period:'mo', currency:'$', specs:['6 vCPU','16 GB RAM','200 GB NVMe','4 TB bandwidth'], featured:true },
      { id:'vps-large', tier:'Large', name:'VPS Large', amount:'39.99', period:'mo', currency:'$', specs:['8 vCPU','32 GB RAM','400 GB NVMe','8 TB bandwidth'], featured:false },
      { id:'vps-xl', tier:'XL', name:'VPS XL', amount:'79.99', period:'mo', currency:'$', specs:['16 vCPU','64 GB RAM','800 GB NVMe','16 TB bandwidth'], featured:false },
    ]
  },
  'vps-medium': { alias: 'vps-small' },
  'vps-large':  { alias: 'vps-small' },
  'vps-xl':     { alias: 'vps-small' },

  'gpu-basic': {
    tag: '⚡ GPU Computing', icon: '⚡',
    title: 'GPU Computing', cat: 'GPU',
    desc: 'Dedicated NVIDIA GPUs for AI inference, LLM fine-tuning, video rendering, and high-performance computing workloads.',
    bullets: ['Dedicated NVIDIA GPU', 'NVLink-ready instances', 'Pre-installed CUDA 12', 'Persistent storage included'],
    plans: [
      { id:'gpu-basic',      tier:'Basic',      name:'GPU Basic',      amount:'29.99',  period:'mo', currency:'$', specs:['GT 730 GPU','4 vCPU','8 GB RAM','100 GB NVMe'],  featured:false },
      { id:'gpu-pro',        tier:'Pro',        name:'GPU Pro',        amount:'99.99',  period:'mo', currency:'$', specs:['RTX 4090','8 vCPU','32 GB RAM','200 GB NVMe'],  featured:true  },
      { id:'gpu-enterprise', tier:'Enterprise', name:'GPU Enterprise', amount:'299.99', period:'mo', currency:'$', specs:['A100 80GB','16 vCPU','64 GB RAM','400 GB NVMe'], featured:false },
    ]
  },
  'gpu-pro':        { alias: 'gpu-basic' },
  'gpu-enterprise': { alias: 'gpu-basic' },

  'storage-50': {
    tag: '💾 Object Storage', icon: '💾',
    title: 'Object Storage', cat: 'Storage',
    desc: 'S3-compatible object storage. Store bot documents, backups, media files and datasets at a fraction of cloud prices.',
    bullets: ['S3-compatible API (AWS SDK)','Versioning & lifecycle rules','End-to-end encryption at rest','Zero-egress option on large plans'],
    plans: [
      { id:'storage-50',   tier:'Starter', name:'50 GB',   amount:'9.99',   period:'mo', currency:'$', specs:['50 GB storage','100 GB egress','S3-compatible','99.9% uptime'], featured:false },
      { id:'storage-250',  tier:'Growth',  name:'250 GB',  amount:'29.99',  period:'mo', currency:'$', specs:['250 GB storage','500 GB egress','S3-compatible','Versioning'], featured:true  },
      { id:'storage-1tb',  tier:'Scale',   name:'1 TB',    amount:'59.99',  period:'mo', currency:'$', specs:['1 TB storage','2 TB egress','Zero-egress opt.','Lifecycle mgmt'], featured:false },
      { id:'storage-10tb', tier:'Power',   name:'10 TB',   amount:'199.99', period:'mo', currency:'$', specs:['10 TB storage','20 TB egress','Zero-egress opt.','Priority support'], featured:false },
    ]
  },
  'storage-250': { alias:'storage-50' },
  'storage-1tb':  { alias:'storage-50' },
  'storage-10tb': { alias:'storage-50' },

  'number-local': {
    tag: '📞 Virtual Numbers', icon: '📞',
    title: 'Virtual Phone Numbers', cat: 'Numbers',
    desc: 'Local and international phone numbers for WhatsApp, SMS and voice. Connect bots to real numbers in 150+ countries.',
    bullets: ['150+ countries covered','WhatsApp Business API ready','SMS + Voice capabilities','Instant activation'],
    plans: [
      { id:'number-local',    tier:'Local',    name:'Local Number',   amount:'5.99',  period:'mo', currency:'$', specs:['1 number','1 country','SMS + Voice','WhatsApp-ready'], featured:false },
      { id:'number-global',   tier:'Global',   name:'Global Bundle',  amount:'19.99', period:'mo', currency:'$', specs:['3 numbers','Different countries','SMS + Voice','Priority routing'], featured:true  },
      { id:'number-business', tier:'Business', name:'Business Pack',  amount:'49.99', period:'mo', currency:'$', specs:['10 numbers','Any countries','SMS + Voice + WA','Dedicated routing'], featured:false },
    ]
  },
  'number-global':   { alias:'number-local' },
  'number-business': { alias:'number-local' },

  'number-search': {
    tag: '🔍 Search Numbers', icon: '🔍',
    title: 'Find Available Numbers', cat: 'Numbers',
    desc: 'Search for available numbers by country and capabilities. Pick your exact number instantly.',
    bullets: ['Search by country','Filter by SMS, Voice, WhatsApp','Instant purchase','Port your existing number'],
    type: 'number-search'
  },

  'calls-100': {
    tag: '📱 Call Bundles', icon: '📱',
    title: 'Call Minute Bundles', cat: 'Calls',
    desc: 'Pre-paid outbound call minutes for your bots. Works globally with competitive per-minute rates.',
    bullets: ['Global outbound calls','No contracts, buy more anytime','HD voice quality','CDRs & analytics included'],
    plans: [
      { id:'calls-100',  tier:'Starter', name:'100 Minutes',  amount:'9.99',  period:'once', currency:'$', specs:['100 min outbound','Global coverage','HD voice','CDR report'], featured:false },
      { id:'calls-500',  tier:'Growth',  name:'500 Minutes',  amount:'39.99', period:'once', currency:'$', specs:['500 min outbound','Global coverage','HD voice','Priority routing'], featured:true  },
      { id:'calls-1000', tier:'Power',   name:'1000+ Minutes',amount:'69.99', period:'once', currency:'$', specs:['1000 min outbound','Global coverage','HD voice','Dedicated routes'], featured:false },
    ]
  },
  'calls-500':  { alias:'calls-100' },
  'calls-1000': { alias:'calls-100' },

  'sim': { alias:'calls-100' },

  'domain-search': {
    tag: '🌐 Domains', icon: '🌐',
    title: 'Domain Name Search', cat: 'Domains',
    desc: 'Register the perfect domain for your bot or product. One-click DNS setup, auto-renew, free WHOIS privacy.',
    bullets: ['Free WHOIS privacy','Auto-renew available','One-click DNS setup','Transfer in supported'],
    type: 'domain-search'
  },

  'domain-tlds': {
    tag: '🌐 Domains', icon: '🌐',
    title: 'Domain Extensions & Pricing', cat: 'Domains',
    desc: 'Choose from the most popular TLDs. All include free WHOIS protection, DNS management and auto-renew.',
    bullets: ['Free WHOIS privacy','Managed DNS','SSL ready','Transfer-in supported'],
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

// ── Store landing (all categories) ──
function renderStoreLanding() {
  const right = document.getElementById('store-right');
  const cats = [
    { cat:'compute', icon:'🖥️', name:'Virtual Servers', desc:'VPS instances for WhatsApp bots, APIs, dev environments. Full root access, NVMe SSD, instant provisioning.' },
    { cat:'gpu',     icon:'⚡',  name:'GPU Computing',  desc:'Dedicated NVIDIA GPUs for AI inference, LLM fine-tuning, video rendering and HPC workloads.' },
    { cat:'storage', icon:'💾',  name:'Object Storage',  desc:'S3-compatible storage for bot documents, backups, media files. Versioning, encryption, zero egress.' },
    { cat:'comms',   icon:'📞',  name:'Phone Numbers',   desc:'Local and international numbers for WhatsApp, SMS and voice. 150+ countries covered.' },
    { cat:'apps',    icon:'🌐',  name:'Domains',         desc:'Domain registration with free WHOIS privacy, DNS management, auto-renew and SSL support.' },
  ];
  right.innerHTML = '<div class="store-landing">' +
    cats.map(c => '<div class="store-landing-card" onclick="location.href=\'/cloud/store?cat=' + c.cat + '\'">' +
      '<div class="store-landing-icon">' + c.icon + '</div>' +
      '<div class="store-landing-name">' + c.name + '</div>' +
      '<div class="store-landing-desc">' + c.desc + '</div>' +
      '<div class="store-landing-action">Browse ' + c.name + '</div>' +
      '</div>').join('') +
    '</div>';
}

// ── Render right panel ──
