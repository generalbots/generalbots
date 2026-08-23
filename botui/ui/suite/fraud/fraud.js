if (window.GBAppLifecycle) GBAppLifecycle.begin("fraud");
(function() {
'use strict';
const API_BASE = '/api/fraud';
const state = { transactions: [], rules: [], blocklist: [], reports: [] };

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
    document.getElementById(btn.dataset.close).classList.remove('active');
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

function getRiskClass(score) {
  if (score >= 70) return 'high';
  if (score >= 40) return 'medium';
  return 'low';
}

function getStatusBadge(status) {
  const map = { approved: 'approved', aprovado: 'approved', review: 'review', revisao: 'review', blocked: 'blocked', bloqueado: 'blocked' };
  const cls = map[status] || 'review';
  const label = { approved: 'Aprovado', review: 'Revisão', blocked: 'Bloqueado' };
  return `<span class="status-badge ${cls}">${label[status] || status}</span>`;
}

function renderTransactions(data) {
  const container = document.getElementById('transactionsTableContainer');
  if (!data.length) {
    container.innerHTML = '<div class="empty-state"><span>Nenhuma transação encontrada</span></div>';
    return;
  }
  container.innerHTML = `
    <table>
      <thead>
        <tr>
          <th>ID</th>
          <th>Valor</th>
          <th>Risco</th>
          <th>Status</th>
          <th>Data</th>
        </tr>
      </thead>
      <tbody>
        ${data.map(t => `
          <tr>
            <td>${t.id || '-'}</td>
            <td>${formatCurrency(t.amount || t.valor)}</td>
            <td>
              <div class="risk-bar">
                <div class="risk-fill ${getRiskClass(t.risk_score || t.score || 0)}" style="width:${t.risk_score || t.score || 0}%"></div>
              </div>
              <span class="risk-text">${t.risk_score || t.score || 0}</span>
            </td>
            <td>${getStatusBadge(t.status)}</td>
            <td>${formatDateTime(t.timestamp || t.created_at)}</td>
          </tr>
        `).join('')}
      </tbody>
    </table>
  `;
}

function renderRules(data) {
  const container = document.getElementById('rulesTableContainer');
  if (!data.length) {
    container.innerHTML = '<div class="empty-state"><span>Nenhuma regra configurada</span></div>';
    return;
  }
  container.innerHTML = `
    <table>
      <thead>
        <tr>
          <th>Nome</th>
          <th>Condição</th>
          <th>Ação</th>
          <th>Prioridade</th>
          <th>Ativa</th>
          <th>Ações</th>
        </tr>
      </thead>
      <tbody>
        ${data.map(r => `
          <tr>
            <td>${r.name || r.nome || '-'}</td>
            <td><code style="font-size:12px;background:var(--bg-primary);padding:2px 6px;border-radius:4px;">${r.condition || r.condicao || '-'}</code></td>
            <td>${r.action || r.acao || '-'}</td>
            <td>${r.priority || r.prioridade || '-'}</td>
            <td>
              <label class="toggle-switch">
                <input type="checkbox" ${r.enabled !== false ? 'checked' : ''} data-rule-id="${r.id}" class="rule-toggle">
                <span class="toggle-slider"></span>
              </label>
            </td>
            <td>
              <button class="btn btn-danger btn-sm" data-delete-rule="${r.id}">Excluir</button>
            </td>
          </tr>
        `).join('')}
      </tbody>
    </table>
  `;

  container.querySelectorAll('.rule-toggle').forEach(toggle => {
    toggle.addEventListener('change', async (e) => {
      const ruleId = e.target.dataset.ruleId;
      try {
        await fetch(`${API_BASE}/rules/${ruleId}`, {
          method: 'PATCH',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ enabled: e.target.checked })
        });
      } catch (err) {
        showToast('Erro ao atualizar regra', 'error');
      }
    });
  });

  container.querySelectorAll('[data-delete-rule]').forEach(btn => {
    btn.addEventListener('click', async () => {
      const ruleId = btn.dataset.deleteRule;
      try {
        await fetch(`${API_BASE}/rules/${ruleId}`, { method: 'DELETE' });
        showToast('Regra excluída');
        loadRules();
      } catch (err) {
        showToast('Erro ao excluir regra', 'error');
      }
    });
  });
}

function renderBlocklist(data) {
  const container = document.getElementById('blocklistTableContainer');
  if (!data.length) {
    container.innerHTML = '<div class="empty-state"><span>Lista de bloqueio vazia</span></div>';
    return;
  }
  container.innerHTML = `
    <table>
      <thead>
        <tr>
          <th>Tipo</th>
          <th>Valor</th>
          <th>Motivo</th>
          <th>Data</th>
          <th>Ações</th>
        </tr>
      </thead>
      <tbody>
        ${data.map(b => `
          <tr>
            <td>${b.type || b.tipo || '-'}</td>
            <td>${b.value || b.valor || '-'}</td>
            <td>${b.reason || b.motivo || '-'}</td>
            <td>${formatDateTime(b.created_at || b.timestamp)}</td>
            <td>
              <button class="btn btn-danger btn-sm" data-delete-block="${b.id}">Remover</button>
            </td>
          </tr>
        `).join('')}
      </tbody>
    </table>
  `;

  container.querySelectorAll('[data-delete-block]').forEach(btn => {
    btn.addEventListener('click', async () => {
      const blockId = btn.dataset.deleteBlock;
      try {
        await fetch(`${API_BASE}/blocklist/${blockId}`, { method: 'DELETE' });
        showToast('Item removido da lista');
        loadBlocklist();
      } catch (err) {
        showToast('Erro ao remover item', 'error');
      }
    });
  });
}

function renderReports(data) {
  const container = document.getElementById('reportsContainer');
  if (!data.length) {
    container.innerHTML = '<div class="empty-state"><span>Nenhum relatório gerado</span></div>';
    return;
  }
  container.innerHTML = `
    <table>
      <thead>
        <tr>
          <th>Período</th>
          <th>Transações</th>
          <th>Bloqueadas</th>
          <th>Revisão</th>
          <th>Falso Positivo</th>
          <th>Ações</th>
        </tr>
      </thead>
      <tbody>
        ${data.map(r => `
          <tr>
            <td>${r.period || r.periodo || '-'}</td>
            <td>${r.total_transactions || r.total || 0}</td>
            <td>${r.blocked || r.bloqueadas || 0}</td>
            <td>${r.review || r.revisao || 0}</td>
            <td>${r.false_positive_rate || r.falso_positivo || '0%'}</td>
            <td>
              <button class="btn btn-secondary btn-sm" data-download-report="${r.id}">Baixar</button>
            </td>
          </tr>
        `).join('')}
      </tbody>
    </table>
  `;

  container.querySelectorAll('[data-download-report]').forEach(btn => {
    btn.addEventListener('click', async () => {
      const reportId = btn.dataset.downloadReport;
      try {
        const resp = await fetch(`${API_BASE}/reports/${reportId}/download`);
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
        const blob = await resp.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `relatorio_fraude_${reportId}.csv`;
        a.click();
        URL.revokeObjectURL(url);
      } catch (err) {
        showToast('Erro ao baixar relatório', 'error');
      }
    });
  });
}

function updateStats() {
  const blocked = state.transactions.filter(t => t.status === 'blocked' || t.status === 'bloqueado').length;
  const review = state.transactions.filter(t => t.status === 'review' || t.status === 'revisao').length;
  const total = state.transactions.length;
  const falsePositives = state.transactions.filter(t => t.false_positive || t.falso_positivo).length;
  const fpRate = total > 0 ? ((falsePositives / total) * 100).toFixed(1) : '0';

  document.getElementById('statBlocked').textContent = blocked;
  document.getElementById('statReview').textContent = review;
  document.getElementById('statFalsePositive').textContent = `${fpRate}%`;
  document.getElementById('statAnalyzed').textContent = total;
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
  const [transactions, rules, blocklist, reports] = await Promise.all([
    fetchData('transactions', 'transactions'),
    fetchData('rules', 'rules'),
    fetchData('blocklist', 'blocklist'),
    fetchData('reports', 'reports')
  ]);
  renderTransactions(transactions);
  renderRules(rules);
  renderBlocklist(blocklist);
  renderReports(reports);
  updateStats();
}

async function loadRules() {
  await fetchData('rules', 'rules');
  renderRules(state.rules);
}

async function loadBlocklist() {
  await fetchData('blocklist', 'blocklist');
  renderBlocklist(state.blocklist);
}

async function saveRule() {
  const payload = {
    name: document.getElementById('ruleName').value,
    condition: document.getElementById('ruleCondition').value,
    action: document.getElementById('ruleAction').value,
    priority: document.getElementById('rulePriority').value,
    enabled: true
  };

  if (!payload.name || !payload.condition) {
    showToast('Nome e condição são obrigatórios', 'error');
    return;
  }

  try {
    const resp = await fetch(`${API_BASE}/rules`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    showToast('Regra salva com sucesso');
    document.getElementById('ruleModal').classList.remove('active');
    document.getElementById('ruleName').value = '';
    document.getElementById('ruleCondition').value = '';
    loadRules();
  } catch (err) {
    showToast(`Erro ao salvar regra: ${err.message}`, 'error');
  }
}

async function saveBlocklist() {
  const payload = {
    type: document.getElementById('blocklistType').value,
    value: document.getElementById('blocklistValue').value,
    reason: document.getElementById('blocklistReason').value
  };

  if (!payload.value) {
    showToast('Valor é obrigatório', 'error');
    return;
  }

  try {
    const resp = await fetch(`${API_BASE}/blocklist`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    showToast('Item adicionado à lista de bloqueio');
    document.getElementById('blocklistModal').classList.remove('active');
    document.getElementById('blocklistValue').value = '';
    document.getElementById('blocklistReason').value = '';
    loadBlocklist();
  } catch (err) {
    showToast(`Erro ao adicionar: ${err.message}`, 'error');
  }
}

async function gerarRelatorio() {
  try {
    const resp = await fetch(`${API_BASE}/reports`, { method: 'POST' });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    showToast('Relatório gerado com sucesso');
    loadAll();
  } catch (err) {
    showToast(`Erro ao gerar relatório: ${err.message}`, 'error');
  }
}

document.getElementById('filterStatus').addEventListener('change', (e) => {
  const filter = e.target.value;
  const filtered = filter ? state.transactions.filter(t => t.status === filter) : state.transactions;
  renderTransactions(filtered);
});

document.getElementById('btnAddRule').addEventListener('click', () => {
  document.getElementById('ruleModalTitle').textContent = 'Nova Regra';
  document.getElementById('ruleModal').classList.add('active');
});

document.getElementById('btnAddRule2').addEventListener('click', () => {
  document.getElementById('ruleModalTitle').textContent = 'Nova Regra';
  document.getElementById('ruleModal').classList.add('active');
});

document.getElementById('btnAddBlocklist').addEventListener('click', () => {
  document.getElementById('blocklistModal').classList.add('active');
});

document.getElementById('btnSaveRule').addEventListener('click', saveRule);
document.getElementById('btnSaveBlocklist').addEventListener('click', saveBlocklist);
document.getElementById('btnGerarRelatorio').addEventListener('click', gerarRelatorio);

loadAll();
})();
