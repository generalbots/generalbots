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

function t(key, fallback) {
  return window.gbT ? window.gbT(key, fallback) : fallback;
}

function showToast(message, type = 'success') {
  const toast = document.getElementById('toast');
  toast.textContent = message;
  toast.className = `toast ${type} show`;
  setTimeout(() => toast.classList.remove('show'), 3000);
}

function formatCurrency(value) {
  return gbFormatCurrency(value, 'BRL', 'pt-BR');
}

function formatDateTime(dateStr) {
  if (!dateStr) return '-';
  return gbFormatDateTime(new Date(dateStr), { locale: 'pt-BR' });
}

function getRiskClass(score) {
  if (score >= 70) return 'high';
  if (score >= 40) return 'medium';
  return 'low';
}

function getStatusBadge(status) {
  const map = { approved: 'approved', aprovado: 'approved', review: 'review', revisao: 'review', blocked: 'blocked', bloqueado: 'blocked' };
  const cls = map[status] || 'review';
  const label = { approved: t('fraud-status-approved', 'Approved'), review: t('fraud-status-review', 'Review'), blocked: t('fraud-status-blocked', 'Blocked') };
  return `<span class="status-badge ${cls}">${label[status] || status}</span>`;
}

function renderTransactions(data) {
  const container = document.getElementById('transactionsTableContainer');
  if (!data.length) {
    container.innerHTML = '<div class="empty-state"><span>' + t('fraud-empty-transactions', 'No transactions found') + '</span></div>';
    return;
  }
  container.innerHTML = `
    <table>
      <thead>
        <tr>
          <th>${t('fraud-col-id','ID')}</th>
          <th>${t('fraud-col-value','Value')}</th>
          <th>${t('fraud-col-risk','Risk')}</th>
          <th>${t('fraud-col-status','Status')}</th>
          <th>${t('fraud-col-date','Date')}</th>
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
    container.innerHTML = '<div class="empty-state"><span>' + t('fraud-empty-rules', 'No rules configured') + '</span></div>';
    return;
  }
  container.innerHTML = `
    <table>
      <thead>
        <tr>
          <th>${t('fraud-col-name','Name')}</th>
          <th>${t('fraud-col-condition','Condition')}</th>
          <th>${t('fraud-col-action','Action')}</th>
          <th>${t('fraud-col-priority','Priority')}</th>
          <th>${t('fraud-col-active','Active')}</th>
          <th>${t('fraud-col-actions','Actions')}</th>
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
              <button class="btn btn-danger btn-sm" data-delete-rule="${r.id}" data-i18n="action-delete">${t('action-delete', 'Delete')}</button>
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
        showToast(t('fraud-error-update-rule', 'Error updating rule'), 'error');
      }
    });
  });

  container.querySelectorAll('[data-delete-rule]').forEach(btn => {
    btn.addEventListener('click', async () => {
      const ruleId = btn.dataset.deleteRule;
      try {
        await fetch(`${API_BASE}/rules/${ruleId}`, { method: 'DELETE' });
        showToast(t('fraud-rule-deleted', 'Rule deleted'));
        loadRules();
      } catch (err) {
        showToast(t('fraud-error-delete-rule', 'Error deleting rule'), 'error');
      }
    });
  });
}

function renderBlocklist(data) {
  const container = document.getElementById('blocklistTableContainer');
  if (!data.length) {
    container.innerHTML = '<div class="empty-state"><span>' + t('fraud-empty-blocklist', 'Blocklist is empty') + '</span></div>';
    return;
  }
  container.innerHTML = `
    <table>
      <thead>
        <tr>
          <th>${t('fraud-col-type','Type')}</th>
          <th>${t('fraud-col-value','Value')}</th>
          <th>${t('fraud-col-reason','Reason')}</th>
          <th>${t('fraud-col-date','Date')}</th>
          <th>${t('fraud-col-actions','Actions')}</th>
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
              <button class="btn btn-danger btn-sm" data-delete-block="${b.id}">${t('fraud-remove','Remove')}</button>
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
        showToast(t('fraud-item-removed', 'Item removed from list'));
        loadBlocklist();
      } catch (err) {
        showToast(t('fraud-error-remove-item', 'Error removing item'), 'error');
      }
    });
  });
}

function renderReports(data) {
  const container = document.getElementById('reportsContainer');
  if (!data.length) {
    container.innerHTML = '<div class="empty-state"><span>' + t('fraud-empty-reports', 'No reports generated') + '</span></div>';
    return;
  }
  container.innerHTML = `
    <table>
      <thead>
        <tr>
          <th>${t('fraud-col-period','Period')}</th>
          <th>${t('fraud-col-transactions','Transactions')}</th>
          <th>${t('fraud-col-blocked','Blocked')}</th>
          <th>${t('fraud-col-review','Review')}</th>
          <th>${t('fraud-col-false-positive','False Positive')}</th>
          <th>${t('fraud-col-actions','Actions')}</th>
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
              <button class="btn btn-secondary btn-sm" data-download-report="${r.id}">${t('action-download','Download')}</button>
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
        showToast(t('fraud-error-download-report', 'Error downloading report'), 'error');
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
    showToast(t('fraud-name-condition-required', 'Name and condition are required'), 'error');
    return;
  }

  try {
    const resp = await fetch(`${API_BASE}/rules`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    showToast(t('fraud-rule-saved', 'Rule saved successfully'));
    document.getElementById('ruleModal').classList.remove('active');
    document.getElementById('ruleName').value = '';
    document.getElementById('ruleCondition').value = '';
    loadRules();
  } catch (err) {
    showToast(t('fraud-error-save-rule', 'Error saving rule') + ': ' + err.message, 'error');
  }
}

async function saveBlocklist() {
  const payload = {
    type: document.getElementById('blocklistType').value,
    value: document.getElementById('blocklistValue').value,
    reason: document.getElementById('blocklistReason').value
  };

  if (!payload.value) {
    showToast(t('fraud-value-required', 'Value is required'), 'error');
    return;
  }

  try {
    const resp = await fetch(`${API_BASE}/blocklist`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    showToast(t('fraud-item-added-blocklist', 'Item added to blocklist'));
    document.getElementById('blocklistModal').classList.remove('active');
    document.getElementById('blocklistValue').value = '';
    document.getElementById('blocklistReason').value = '';
    loadBlocklist();
  } catch (err) {
    showToast(t('fraud-error-add-item', 'Error adding item') + ': ' + err.message, 'error');
  }
}

async function gerarRelatorio() {
  try {
    const resp = await fetch(`${API_BASE}/reports`, { method: 'POST' });
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    showToast(t('fraud-report-generated', 'Report generated successfully'));
    loadAll();
  } catch (err) {
    showToast(t('fraud-error-generate-report', 'Error generating report') + ': ' + err.message, 'error');
  }
}

document.getElementById('filterStatus').addEventListener('change', (e) => {
  const filter = e.target.value;
  const filtered = filter ? state.transactions.filter(t => t.status === filter) : state.transactions;
  renderTransactions(filtered);
});

document.getElementById('btnAddRule').addEventListener('click', () => {
  document.getElementById('ruleModalTitle').textContent = t('fraud-new-rule', 'New Rule');
  document.getElementById('ruleModal').classList.add('active');
});

document.getElementById('btnAddRule2').addEventListener('click', () => {
  document.getElementById('ruleModalTitle').textContent = t('fraud-new-rule', 'New Rule');
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
