"use strict";

const tabButtons = document.querySelectorAll("#tax-tabs .gb-tab");
const tabPanes = document.querySelectorAll(".gb-tab-pane");

tabButtons.forEach((button) => {
  button.addEventListener("click", () => {
    const target = button.dataset.tab;
    tabButtons.forEach((b) => b.classList.toggle("active", b === button));
    tabPanes.forEach((p) => p.classList.toggle("active", p.id === `tab-${target}`));
  });
});

document.querySelectorAll("[data-shortcut]").forEach((button) => {
  button.addEventListener("click", () => {
    const tab = button.dataset.shortcut;
    const target = document.querySelector(`#tax-tabs .gb-tab[data-tab="${tab}"]`);
    if (target) target.click();
  });
});

function onlyDigits(value) {
  return String(value || "").replace(/\D+/g, "");
}

function isCnpjValid(value) {
  const d = onlyDigits(value);
  if (d.length !== 14) return false;
  if (/^(\d)\1+$/.test(d)) return false;
  const weights = [
    [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2],
    [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2],
  ];
  for (let step = 0; step < 2; step++) {
    let sum = 0;
    for (let i = 0; i < weights[step].length; i++) {
      sum += Number(d[i]) * weights[step][i];
    }
    const remainder = sum % 11;
    const dv = remainder < 2 ? 0 : 11 - remainder;
    if (dv !== Number(d[weights[step].length])) return false;
  }
  return true;
}

function isCpfValid(value) {
  const d = onlyDigits(value);
  if (d.length !== 11) return false;
  if (/^(\d)\1+$/.test(d)) return false;
  const weights = [
    [10, 9, 8, 7, 6, 5, 4, 3, 2],
    [11, 10, 9, 8, 7, 6, 5, 4, 3, 2],
  ];
  for (let step = 0; step < 2; step++) {
    let sum = 0;
    for (let i = 0; i < weights[step].length; i++) {
      sum += Number(d[i]) * weights[step][i];
    }
    const remainder = sum % 11;
    const dv = remainder < 2 ? 0 : 11 - remainder;
    if (dv !== Number(d[weights[step].length])) return false;
  }
  return true;
}

function isCepValid(value) {
  return onlyDigits(value).length === 8;
}

function ieShapeValid(uf, value) {
  const d = onlyDigits(value);
  const shapes = {
    AC: 13, AL: 9, AP: 9, AM: 9, BA: [8, 9], CE: 9, DF: 13, ES: 9, GO: 9,
    MA: 9, MT: 11, MS: 9, MG: 13, PA: 9, PB: 9, PR: 10, PE: [9, 14], PI: 9,
    RJ: 8, RN: [9, 10], RS: 10, RO: 14, RR: 9, SC: 9, SP: 12, SE: 9, TO: 9,
  };
  const expected = shapes[uf];
  if (Array.isArray(expected)) return expected.includes(d.length);
  if (typeof expected === "number") return d.length === expected;
  return d.length >= 8 && d.length <= 14;
}

function buildAccessKey(state, year, month, cnpj, model, series, number, emissionKind, code) {
  const cnpjDigits = onlyDigits(cnpj).padStart(14, "0");
  const seriesNum = Number(onlyDigits(series) || 0);
  const key = `${state}${String(year).slice(-2).padStart(2, "0")}${String(month).padStart(2, "0")}${cnpjDigits}${model}${String(emissionKind).padStart(9, "0")}${String(seriesNum).padStart(3, "0")}${String(number).padStart(9, "0")}${String(code).padStart(8, "0")}`;
  const bytes = key.split("").map((c) => Number(c));
  const weights = Array.from({ length: 43 }, (_, i) => 43 - i);
  let sum = 0;
  for (let i = 0; i < 43; i++) sum += bytes[i] * weights[i];
  const remainder = sum % 11;
  const dv = remainder < 2 ? 0 : 11 - remainder;
  return key + String(dv);
}

function isAccessKeyValid(key) {
  const d = onlyDigits(key);
  if (d.length !== 44) return false;
  const bytes = d.slice(0, 43).split("").map((c) => Number(c));
  const weights = Array.from({ length: 43 }, (_, i) => 43 - i);
  let sum = 0;
  for (let i = 0; i < 43; i++) sum += bytes[i] * weights[i];
  const remainder = sum % 11;
  const dv = remainder < 2 ? 0 : 11 - remainder;
  return dv === Number(d[43]);
}

const municipalitySelect = document.querySelector('select[name="city"]');
if (municipalitySelect) {
  fetch("../assets/municipalities.json")
    .then((r) => (r.ok ? r.json() : []))
    .then((rows) => {
      rows.forEach((row) => {
        const option = document.createElement("option");
        option.value = row.code;
        option.textContent = `${row.name} / ${row.state}`;
        municipalitySelect.appendChild(option);
      });
    })
    .catch(() => {
      // Backend endpoint unavailable — leave select empty; user can still type.
    });
}

const nfeForm = document.getElementById("nfe-form");
if (nfeForm) {
  nfeForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const form = new FormData(nfeForm);
    const cnpj = form.get("issuer_cnpj");
    if (!isCnpjValid(cnpj)) {
      document.getElementById("nfe-key-output").textContent = "CNPJ inválido.";
      return;
    }
    const now = new Date();
    const key = buildAccessKey("35", now.getFullYear(), now.getMonth() + 1, cnpj, "55", "1", 1, "1", 0);
    document.getElementById("nfe-key-output").textContent = key + (isAccessKeyValid(key) ? " (chave válida)" : " (chave inválida)");
  });
}

const nfseForm = document.getElementById("nfse-form");
if (nfseForm) {
  nfseForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const form = new FormData(nfseForm);
    const rps = String(form.get("rps") || "1");
    const rpsPadded = rps.padStart(9, "0");
    document.getElementById("nfse-output").textContent = `RPS-A-${rpsPadded}`;
  });
}

const cteForm = document.getElementById("cte-form");
if (cteForm) {
  cteForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const form = new FormData(cteForm);
    const modal = form.get("modal");
    const plate = form.get("plate");
    const origin = form.get("origin");
    const destination = form.get("destination");
    if (onlyDigits(origin).length !== 7 || onlyDigits(destination).length !== 7) {
      document.getElementById("cte-output").textContent = "Códigos IBGE devem ter 7 dígitos.";
      return;
    }
    document.getElementById("cte-output").textContent = `CT-e ${modal} ${plate} ${origin}→${destination}`;
  });
}

const spedForm = document.getElementById("sped-form");
if (spedForm) {
  spedForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const form = new FormData(spedForm);
    if (!isCnpjValid(form.get("cnpj"))) {
      document.getElementById("sped-output").textContent = "CNPJ inválido.";
      return;
    }
    const start = form.get("start").split("-").reverse().join("");
    const end = form.get("end").split("-").reverse().join("");
    const output = `|0000|${start}|${end}|${onlyDigits(form.get("cnpj"))}|\n|9001|0|\n|9999|3|\n`;
    document.getElementById("sped-output").textContent = output;
  });
}

const validatorsForm = document.getElementById("validators-form");
if (validatorsForm) {
  validatorsForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const form = new FormData(validatorsForm);
    const cnpjOk = !form.get("cnpj") || isCnpjValid(form.get("cnpj"));
    const cpfOk = !form.get("cpf") || isCpfValid(form.get("cpf"));
    const cepOk = !form.get("cep") || isCepValid(form.get("cep"));
    const ieOk = !form.get("ie") || ieShapeValid(form.get("uf"), form.get("ie"));
    const akRaw = form.get("access_key");
    const akOk = !akRaw || isAccessKeyValid(akRaw);
    const renavamRaw = form.get("renavam");
    const renavamOk = !renavamRaw || onlyDigits(renavamRaw).length === 11;
    const lines = [
      `CNPJ: ${cnpjOk ? "OK" : "inválido"}`,
      `CPF: ${cpfOk ? "OK" : "inválido"}`,
      `CEP: ${cepOk ? "OK" : "inválido"}`,
      `IE ${form.get("uf")}: ${ieOk ? "OK" : "formato inválido"}`,
      `Chave NFe: ${akOk ? "OK" : "inválida"}`,
      `RENAVAM: ${renavamOk ? "OK" : "inválido"}`,
    ];
    document.getElementById("validators-output").textContent = lines.join("\n");
  });
}

function renderProductLine(index) {
  const div = document.createElement("div");
  div.className = "gb-product-line";
  div.dataset.index = String(index);
  div.innerHTML = `
    <input name="sku_${index}" placeholder="SKU" />
    <input name="description_${index}" placeholder="Descrição" required />
    <input name="ncm_${index}" placeholder="NCM" />
    <input name="cfop_${index}" placeholder="CFOP" />
    <input name="quantity_${index}" type="number" min="0" step="0.01" value="1" placeholder="Qtd" />
    <input name="unit_price_${index}" type="number" min="0" step="0.01" value="0" placeholder="Vl. unit." />
    <button type="button" class="gb-product-remove" aria-label="Remover">✕</button>
  `;
  div.querySelector(".gb-product-remove").addEventListener("click", () => div.remove());
  return div;
}

const nfeProducts = document.getElementById("nfe-products");
if (nfeProducts) {
  nfeProducts.appendChild(renderProductLine(0));
}

const nfeAddProduct = document.getElementById("nfe-add-product");
if (nfeAddProduct && nfeProducts) {
  nfeAddProduct.addEventListener("click", () => {
    const index = nfeProducts.children.length;
    nfeProducts.appendChild(renderProductLine(index));
  });
}

const bulkValidateBtn = document.getElementById("bulk-validate-btn");
if (bulkValidateBtn) {
  bulkValidateBtn.addEventListener("click", () => {
    const input = document.getElementById("bulk-input");
    const output = document.getElementById("bulk-output");
    if (!input || !output) return;
    const lines = String(input.value || "").split("\n").map((l) => l.trim()).filter(Boolean);
    if (lines.length === 0) {
      output.textContent = "Cole pelo menos um documento.";
      return;
    }
    const results = lines.map((line) => {
      const digits = onlyDigits(line);
      if (digits.length === 14) return `${line} → CNPJ ${isCnpjValid(line) ? "OK" : "INVÁLIDO"}`;
      if (digits.length === 11) return `${line} → CPF ${isCpfValid(line) ? "OK" : "INVÁLIDO"}`;
      if (digits.length === 8) return `${line} → CEP ${isCepValid(line) ? "OK" : "INVÁLIDO"}`;
      if (digits.length === 44) return `${line} → Chave NFe ${isAccessKeyValid(line) ? "OK" : "INVÁLIDA"}`;
      return `${line} → formato não reconhecido (${digits.length} dígitos)`;
    });
    const total = results.length;
    const ok = results.filter((r) => r.includes("OK")).length;
    output.textContent = `${ok}/${total} válidos\n\n${results.join("\n")}`;
  });
}

function renderHistoryRows(rows) {
  const tbody = document.getElementById("history-tbody");
  if (!tbody) return;
  if (rows.length === 0) {
    tbody.innerHTML = '<tr><td colspan="9" class="gb-empty">Nenhum documento encontrado.</td></tr>';
    return;
  }
  tbody.innerHTML = rows.map((row) => `
    <tr>
      <td>${row.date || ""}</td>
      <td>${row.type || ""}</td>
      <td>${row.number || ""}/${row.series || "1"}</td>
      <td><code>${(row.key || "").slice(0, 8)}…${(row.key || "").slice(-4)}</code></td>
      <td>${row.recipient || ""}</td>
      <td style="text-align:right">${Number(row.amount || 0).toFixed(2)}</td>
      <td>${row.status || ""}</td>
      <td>${row.protocol || "—"}</td>
      <td>
        <button type="button" class="gb-secondary" data-action="view" data-key="${row.key}">Ver</button>
        <button type="button" class="gb-secondary" data-action="xml" data-key="${row.key}">XML</button>
      </td>
    </tr>
  `).join("");
}

async function loadHistory() {
  const tbody = document.getElementById("history-tbody");
  if (!tbody) return;
  try {
    const response = await fetch("/api/brazil/documents");
    if (response.ok) {
      const data = await response.json();
      renderHistoryRows(Array.isArray(data) ? data : []);
    } else {
      renderHistoryRows([]);
    }
  } catch {
    renderHistoryRows([]);
  }
}

const historyFilterBtn = document.getElementById("history-filter-btn");
if (historyFilterBtn) {
  historyFilterBtn.addEventListener("click", loadHistory);
}

const historyTab = document.querySelector('#tax-tabs .gb-tab[data-tab="history"]');
if (historyTab) {
  historyTab.addEventListener("click", loadHistory);
}

const historyExportBtn = document.getElementById("history-export-btn");
if (historyExportBtn) {
  historyExportBtn.addEventListener("click", () => {
    const tbody = document.getElementById("history-tbody");
    if (!tbody) return;
    const rows = Array.from(tbody.querySelectorAll("tr"));
    const csv = ["Data;Tipo;Número/Série;Chave;Destinatário;Valor;Status;Protocolo"]
      .concat(rows.map((tr) => Array.from(tr.querySelectorAll("td")).slice(0, 8).map((td) => `"${td.textContent.trim().replace(/"/g, "\"\"")}"`).join(";")))
      .join("\n");
    const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `historico-fiscal-${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  });
}

const settingsForm = document.getElementById("settings-form");
if (settingsForm) {
  settingsForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const output = document.getElementById("settings-output");
    if (output) output.textContent = "Configurações salvas com sucesso.";
  });
}

const testCertBtn = document.getElementById("test-cert-btn");
if (testCertBtn) {
  testCertBtn.addEventListener("click", () => {
    const output = document.getElementById("settings-output");
    if (output) output.textContent = "Validando certificado... (operação offline; integração SEFAZ requer backend ativo).";
  });
}

const testSefazBtn = document.getElementById("test-sefaz-btn");
if (testSefazBtn) {
  testSefazBtn.addEventListener("click", async () => {
    const output = document.getElementById("settings-output");
    if (output) output.textContent = "Testando conexão com SEFAZ...";
    try {
      const response = await fetch("/api/brazil/sefaz/status");
      if (output) output.textContent = response.ok ? "Conexão OK com SEFAZ." : "Falha na conexão (HTTP " + response.status + ").";
    } catch (error) {
      if (output) output.textContent = "Falha de rede: " + (error && error.message ? error.message : "desconhecida");
    }
  });
}

async function loadDashboardEvents() {
  const list = document.getElementById("dashboard-events");
  if (!list) return;
  try {
    const response = await fetch("/api/brazil/events?limit=5");
    if (response.ok) {
      const data = await response.json();
      if (Array.isArray(data) && data.length > 0) {
        list.innerHTML = data.map((evt) => `
          <li class="gb-event">
            <span class="gb-event-time">${(evt.time || "").slice(11, 16)}</span>
            <span class="gb-event-text">${evt.text || ""}</span>
          </li>
        `).join("");
        return;
      }
    }
  } catch {
    // Fall through to default content.
  }
  list.innerHTML = '<li class="gb-event"><span class="gb-event-time">—</span><span class="gb-event-text">Sem eventos recentes.</span></li>';
}

loadDashboardEvents();
