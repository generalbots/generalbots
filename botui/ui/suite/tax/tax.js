(function() {
'use strict';
const API_BASE = '/api/tax';
const state = {
  nfe: [],
  nfse: [],
  cte: [],
  sped: { contrib: null, fiscal: null }
};

document.getElementById('currentDate').textContent = new Date().toLocaleDateString('pt-BR', {
  weekday: 'long', year: 'numeric', month: 'long', day: 'numeric'
});

document.querySelectorAll('.tab').forEach(tab => {
  tab.addEventListener('click', () => {
    document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
    tab.classList.add('active');
    document.getElementById(`panel-${tab.dataset.tab}`).classList.add('active');
  });
});

document.querySelectorAll('[data-close]').forEach(btn => {
  btn.addEventListener('click', () => {
    const modalId = btn.dataset.close;
    document.getElementById(modalId).classList.remove('active');
  });
});

document.querySelectorAll('.modal-overlay').forEach(overlay => {
  overlay.addEventListener('click', (e) => {
    if (e.target === overlay) overlay.classList.remove('active');
  });
});

function showToast(message, type = 'success') {
  const toast = document.getElementById('toast');
  toast.textContent = message;
  toast.className = `toast ${type} show`;
  setTimeout(() => toast.classList.remove('show'), 3000);
}

function formatCurrency(value) {
  return Number(value).toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' });
}

function formatDateTime(dateStr) {
  if (!dateStr) return '-';
  return new Date(dateStr).toLocaleString('pt-BR');
}

function getStatusBadge(status) {
  const map = {
    autorizada: 'autorizada', autorizado: 'autorizada',
    pendente: 'pendente', aguardando: 'pendente',
    rejeitada: 'rejeitada', erro: 'rejeitada',
    enviada: 'enviada', processando: 'enviada'
  };
  const cls = map[status] || 'pendente';
  return `<span class="badge ${cls}">${status}</span>`;
}

function renderNfeTable(data) {
  const container = document.getElementById('nfeTableContainer');
  if (!data.length) {
    container.innerHTML = '<div class="empty-state"><span class="icon">📄</span><span>Nenhuma NFe encontrada</span></div>';
    return;
  }
  container.innerHTML = `
    <table>
      <thead>
        <tr>
          <th>Número</th>
          <th>Destinatário</th>
          <th>Valor</th>
          <th>Data</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        ${data.map(n => `
          <tr>
            <td>${n.numero || '-'}</td>
            <td>${n.destinatario || '-'}</td>
            <td>${formatCurrency(n.valor)}</td>
            <td>${formatDateTime(n.data_emissao)}</td>
            <td>${getStatusBadge(n.status)}</td>
          </tr>
        `).join('')}
      </tbody>
    </table>
  `;
}

function renderNfseTable(data) {
  const container = document.getElementById('nfseTableContainer');
  if (!data.length) {
    container.innerHTML = '<div class="empty-state"><span class="icon">📋</span><span>Nenhuma NFSe encontrada</span></div>';
    return;
  }
  container.innerHTML = `
    <table>
      <thead>
        <tr>
          <th>Número</th>
          <th>Tomador</th>
          <th>Descrição</th>
          <th>Valor</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        ${data.map(n => `
          <tr>
            <td>${n.numero || '-'}</td>
            <td>${n.tomador || '-'}</td>
            <td>${n.descricao || '-'}</td>
            <td>${formatCurrency(n.valor)}</td>
            <td>${getStatusBadge(n.status)}</td>
          </tr>
        `).join('')}
      </tbody>
    </table>
  `;
}

function renderCteTable(data) {
  const container = document.getElementById('cteTableContainer');
  if (!data.length) {
    container.innerHTML = '<div class="empty-state"><span class="icon">🚛</span><span>Nenhum CT-e encontrado</span></div>';
    return;
  }
  container.innerHTML = `
    <table>
      <thead>
        <tr>
          <th>Número</th>
          <th>Remetente</th>
          <th>Destinatário</th>
          <th>Rota</th>
          <th>Valor Frete</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        ${data.map(c => `
          <tr>
            <td>${c.numero || '-'}</td>
            <td>${c.remetente || '-'}</td>
            <td>${c.destinatario || '-'}</td>
            <td>${c.cidade_origem || ''} → ${c.cidade_destino || ''}</td>
            <td>${formatCurrency(c.valor_frete)}</td>
            <td>${getStatusBadge(c.status)}</td>
          </tr>
        `).join('')}
      </tbody>
    </table>
  `;
}

function updateStats() {
  const nfeHoje = state.nfe.filter(n => {
    if (!n.data_emissao) return false;
    const d = new Date(n.data_emissao);
    const today = new Date();
    return d.toDateString() === today.toDateString();
  }).length;

  const valorTotal = state.nfe.reduce((sum, n) => sum + (Number(n.valor) || 0), 0);
  const nfsePendentes = state.nfse.filter(n => n.status === 'pendente' || n.status === 'aguardando').length;
  const erros = [...state.nfe, ...state.nfse, ...state.cte].filter(n => n.status === 'rejeitada' || n.status === 'erro').length;

  document.getElementById('statNfeHoje').textContent = nfeHoje;
  document.getElementById('statValorTotal').textContent = formatCurrency(valorTotal);
  document.getElementById('statNfsePendentes').textContent = nfsePendentes;
  document.getElementById('statErros').textContent = erros;
}

async function fetchData(endpoint, stateKey) {
  try {
    const resp = await fetch(`${API_BASE}/${endpoint}`);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = await resp.json();
    state[stateKey] = Array.isArray(data) ? data : (data.items || data.data || []);
    return state[stateKey];
  } catch (err) {
    console.error(`Erro ao carregar ${endpoint}:`, err);
    state[stateKey] = [];
    return [];
  }
}

async function loadAll() {
  const [nfe, nfse, cte] = await Promise.all([
    fetchData('nfe', 'nfe'),
    fetchData('nfse', 'nfse'),
    fetchData('cte', 'cte')
  ]);
  renderNfeTable(nfe);
  renderNfseTable(nfse);
  renderCteTable(cte);
  updateStats();
}

async function loadSpedStatus() {
  try {
    const resp = await fetch(`${API_BASE}/sped`);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = await resp.json();
    if (data.contribuicoes) {
      state.sped.contrib = data.contribuicoes;
      document.getElementById('spedContribStatus').textContent = data.contribuicoes.status || 'Gerado';
      document.getElementById('btnDownloadSpedContrib').disabled = false;
    }
    if (data.fiscal) {
      state.sped.fiscal = data.fiscal;
      document.getElementById('spedFiscalStatus').textContent = data.fiscal.status || 'Gerado';
      document.getElementById('btnDownloadSpedFiscal').disabled = false;
    }
  } catch (err) {
    console.error('Erro ao carregar SPED:', err);
  }
}

async function postNfe() {
  const produtos = [];
  document.querySelectorAll('#nfeProdutos .form-row').forEach(row => {
    const desc = row.querySelector('.produto-desc').value;
    const valor = row.querySelector('.produto-valor').value;
    if (desc && valor) produtos.push({ descricao: desc, valor: parseFloat(valor) });
  });

  const payload = {
    destinatario: document.getElementById('nfeDestinatario').value,
    cfop: document.getElementById('nfeCfop').value,
    produtos,
    impostos: {
      icms: parseFloat(document.getElementById('nfeIcms').value) || 0,
      ipi: parseFloat(document.getElementById('nfeIpi').value) || 0,
      pis: parseFloat(document.getElementById('nfePis').value) || 0,
      cofins: parseFloat(document.getElementById('nfeCofins').value) || 0
    }
  };

  try {
    const resp = await fetch(`${API_BASE}/nfe`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    showToast('NFe emitida com sucesso');
    document.getElementById('nfeModal').classList.remove('active');
    loadAll();
  } catch (err) {
    showToast(`Erro ao emitir NFe: ${err.message}`, 'error');
  }
}

async function postNfse() {
  const payload = {
    tomador: document.getElementById('nfseTomador').value,
    descricao: document.getElementById('nfseDescricao').value,
    valor: parseFloat(document.getElementById('nfseValor').value) || 0,
    codigo_servico: document.getElementById('nfseCodigoServico').value,
    iss: parseFloat(document.getElementById('nfseIss').value) || 0
  };

  try {
    const resp = await fetch(`${API_BASE}/nfse`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    showToast('NFSe emitida com sucesso');
    document.getElementById('nfseModal').classList.remove('active');
    loadAll();
  } catch (err) {
    showToast(`Erro ao emitir NFSe: ${err.message}`, 'error');
  }
}

async function postCte() {
  const payload = {
    remetente: document.getElementById('cteRemetente').value,
    destinatario: document.getElementById('cteDestinatario').value,
    cidade_origem: document.getElementById('cteOrigem').value,
    cidade_destino: document.getElementById('cteDestino').value,
    peso: parseFloat(document.getElementById('ctePeso').value) || 0,
    valor_frete: parseFloat(document.getElementById('cteValorFrete').value) || 0,
    modalidade: parseInt(document.getElementById('cteModalidade').value)
  };

  try {
    const resp = await fetch(`${API_BASE}/cte`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    showToast('CT-e emitido com sucesso');
    document.getElementById('cteModal').classList.remove('active');
    loadAll();
  } catch (err) {
    showToast(`Erro ao emitir CT-e: ${err.message}`, 'error');
  }
}

async function gerarSped(tipo) {
  try {
    const resp = await fetch(`${API_BASE}/sped/${tipo}`, { method: 'POST' });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    showToast(`SPED ${tipo === 'contribuicoes' ? 'Contribuições' : 'Fiscal'} gerado com sucesso`);
    loadSpedStatus();
  } catch (err) {
    showToast(`Erro ao gerar SPED: ${err.message}`, 'error');
  }
}

async function downloadSped(tipo) {
  try {
    const resp = await fetch(`${API_BASE}/sped/${tipo}/download`);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const blob = await resp.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `sped_${tipo}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  } catch (err) {
    showToast(`Erro ao baixar SPED: ${err.message}`, 'error');
  }
}

document.getElementById('btnEmitirNfe').addEventListener('click', () => {
  document.getElementById('nfeModal').classList.add('active');
});

document.getElementById('btnEmitirNfse').addEventListener('click', () => {
  document.getElementById('nfseModal').classList.add('active');
});

document.getElementById('btnEmitirCte').addEventListener('click', () => {
  document.getElementById('cteModal').classList.add('active');
});

document.getElementById('btnSalvarNfe').addEventListener('click', postNfe);
document.getElementById('btnSalvarNfse').addEventListener('click', postNfse);
document.getElementById('btnSalvarCte').addEventListener('click', postCte);

document.getElementById('btnAddProduto').addEventListener('click', () => {
  const container = document.getElementById('nfeProdutos');
  const row = document.createElement('div');
  row.className = 'form-row';
  row.style.marginBottom = '8px';
  row.innerHTML = `
    <input type="text" placeholder="Descrição" class="produto-desc">
    <input type="number" placeholder="Valor (R$)" class="produto-valor">
  `;
  container.appendChild(row);
});

document.getElementById('btnGerarSpedContrib').addEventListener('click', () => gerarSped('contribuicoes'));
document.getElementById('btnGerarSpedFiscal').addEventListener('click', () => gerarSped('fiscal'));
document.getElementById('btnDownloadSpedContrib').addEventListener('click', () => downloadSped('contribuicoes'));
document.getElementById('btnDownloadSpedFiscal').addEventListener('click', () => downloadSped('fiscal'));

loadAll();
loadSpedStatus();
})();
