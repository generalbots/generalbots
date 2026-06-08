"use strict";
/* BrazilEngine — Brazilian payroll, tax, banking primitives.
 *
 * Implements 2026 rules (approximations — always verify with current legislation):
 *   - INSS: progressive brackets (8%, 9%, 11%, 12%, 14%) up to ceiling
 *   - IRRF: progressive monthly brackets (exempt, 7.5%, 15%, 22.5%, 27.5%)
 *   - FGTS: 8% of gross salary (employer deposit)
 *   - 13th salary: INSS + IRRF separate
 *   - Férias: 1/3 constitucional
 *   - Vale-transporte: 6% discount optional
 *   - Pix: BR Code / EMV string generation (with CRC16-CCITT)
 *   - CPF/CNPJ: validate + format + generate (test data only)
 *
 * Public: window.BrazilEngine
 */
(function (window) {

  const INSS_BRACKETS_2024 = [
    { upTo: 1412.00, rate: 0.075 },
    { upTo: 2666.68, rate: 0.09 },
    { upTo: 4000.03, rate: 0.12 },
    { upTo: 7786.02, rate: 0.14 }
  ];
  const INSS_CEILING = 7786.02;
  const INSS_MAX_CONTRIBUTION = 908.85;

  const IRRF_BRACKETS_2024 = [
    { upTo: 2259.20, rate: 0, deduction: 0 },
    { upTo: 2826.65, rate: 0.075, deduction: 169.44 },
    { upTo: 3751.05, rate: 0.15, deduction: 381.44 },
    { upTo: 4664.68, rate: 0.225, deduction: 662.77 },
    { upTo: Infinity, rate: 0.275, deduction: 896.00 }
  ];
  const IRRF_EXEMPTION_2024 = 2259.20;
  const IRRF_DEPENDENT_DEDUCTION = 189.59;
  const IRRF_SIMPLIFIED_DISCOUNT = 564.80;

  function calcINSS(grossSalary) {
    let remaining = Math.min(grossSalary, INSS_CEILING);
    let total = 0;
    let prevCeiling = 0;
    for (const b of INSS_BRACKETS_2024) {
      if (remaining <= 0) break;
      const slab = Math.min(remaining, b.upTo - prevCeiling);
      total += slab * b.rate;
      remaining -= slab;
      prevCeiling = b.upTo;
    }
    return { inss: Math.min(total, INSS_MAX_CONTRIBUTION), capped: total > INSS_MAX_CONTRIBUTION };
  }

  function calcIRRF(grossSalary, inss, dependents, simplified) {
    const base = Math.max(0, grossSalary - inss - (dependents || 0) * IRRF_DEPENDENT_DEDUCTION);
    let taxable = base;
    if (simplified) taxable = base - IRRF_SIMPLIFIED_DISCOUNT;
    if (taxable <= IRRF_EXEMPTION_2024) return { irrf: 0, base: taxable, exempt: true };
    for (const b of IRRF_BRACKETS_2024) {
      if (taxable <= b.upTo) {
        return { irrf: Math.max(0, taxable * b.rate - b.deduction), base: taxable, exempt: false };
      }
    }
    return { irrf: 0, base: taxable, exempt: false };
  }

  function calcFGTS(grossSalary) {
    return grossSalary * 0.08;
  }

  function calc13th(grossSalary, monthsWorked) {
    const proportional = (grossSalary / 12) * (monthsWorked || 12);
    const inss13 = calcINSS(proportional).inss;
    const irrf13 = calcIRRF(proportional, inss13, 0, false).irrf;
    return { gross: proportional, inss: inss13, irrf: irrf13, net: proportional - inss13 - irrf13 };
  }

  function calcFerias(grossSalary, days) {
    const proportional = (grossSalary / 30) * (days || 30);
    const tercoConstitucional = proportional / 3;
    const inss = calcINSS(proportional + tercoConstitucional).inss;
    const irrf = calcIRRF(proportional + tercoConstitucional, inss, 0, false).irrf;
    return {
      proportional: proportional,
      tercoConstitucional: tercoConstitucional,
      gross: proportional + tercoConstitucional,
      inss: inss,
      irrf: irrf,
      net: proportional + tercoConstitucional - inss - irrf
    };
  }

  function calcValeTransporte(grossSalary, optIn) {
    if (!optIn) return 0;
    return Math.min(grossSalary * 0.06, grossSalary);
  }

  function calcPayroll(opts) {
    const gross = opts.grossSalary || 0;
    const dependents = opts.dependents || 0;
    const simplified = !!opts.simplifiedIR;
    const vtOptIn = !!opts.valeTransporte;
    const otherDiscounts = opts.otherDiscounts || 0;

    const inss = calcINSS(gross);
    const irrf = calcIRRF(gross, inss.inss, dependents, simplified);
    const vt = calcValeTransporte(gross, vtOptIn);
    const fgts = calcFGTS(gross);

    const totalDiscounts = inss.inss + irrf.irrf + vt + otherDiscounts;
    const net = gross - totalDiscounts;

    return {
      gross: gross,
      inss: inss.inss,
      irrf: irrf.irrf,
      valeTransporte: vt,
      fgts: fgts,
      otherDiscounts: otherDiscounts,
      totalDiscounts: totalDiscounts,
      net: net,
      employerCost: gross + fgts + (gross * 0.10) + (gross * 0.03) + (gross * 0.02)
    };
  }

  function validateCPF(cpf) {
    if (!cpf) return false;
    const d = String(cpf).replace(/\D/g, "");
    if (d.length !== 11) return false;
    if (/^(\d)\1{10}$/.test(d)) return false;
    let s = 0;
    for (let i = 0; i < 9; i++) s += parseInt(d.charAt(i), 10) * (10 - i);
    let r = s % 11;
    const dv1 = r < 2 ? 0 : 11 - r;
    if (dv1 !== parseInt(d.charAt(9), 10)) return false;
    s = 0;
    for (let i = 0; i < 10; i++) s += parseInt(d.charAt(i), 10) * (11 - i);
    r = s % 11;
    const dv2 = r < 2 ? 0 : 11 - r;
    return dv2 === parseInt(d.charAt(10), 10);
  }

  function formatCPF(cpf) {
    const d = String(cpf).replace(/\D/g, "").padStart(11, "0");
    return d.slice(0, 3) + "." + d.slice(3, 6) + "." + d.slice(6, 9) + "-" + d.slice(9);
  }

  function generateCPF() {
    const n = [];
    for (let i = 0; i < 9; i++) n.push(Math.floor(Math.random() * 10));
    let s = 0;
    for (let i = 0; i < 9; i++) s += n[i] * (10 - i);
    let r = s % 11;
    n.push(r < 2 ? 0 : 11 - r);
    s = 0;
    for (let i = 0; i < 10; i++) s += n[i] * (11 - i);
    r = s % 11;
    n.push(r < 2 ? 0 : 11 - r);
    return n.join("");
  }

  function validateCNPJ(cnpj) {
    if (!cnpj) return false;
    const d = String(cnpj).replace(/\D/g, "");
    if (d.length !== 14) return false;
    if (/^(\d)\1{13}$/.test(d)) return false;
    const w1 = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    const w2 = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let s = 0;
    for (let i = 0; i < 12; i++) s += parseInt(d.charAt(i), 10) * w1[i];
    let r = s % 11;
    const dv1 = r < 2 ? 0 : 11 - r;
    if (dv1 !== parseInt(d.charAt(12), 10)) return false;
    s = 0;
    for (let i = 0; i < 13; i++) s += parseInt(d.charAt(i), 10) * w2[i];
    r = s % 11;
    const dv2 = r < 2 ? 0 : 11 - r;
    return dv2 === parseInt(d.charAt(13), 10);
  }

  function formatCNPJ(cnpj) {
    const d = String(cnpj).replace(/\D/g, "").padStart(14, "0");
    return d.slice(0, 2) + "." + d.slice(2, 5) + "." + d.slice(5, 8) + "/" + d.slice(8, 12) + "-" + d.slice(12);
  }

  function generateCNPJ() {
    const n = [];
    for (let i = 0; i < 8; i++) n.push(Math.floor(Math.random() * 10));
    n.push(0); n.push(0); n.push(0); n.push(1);
    const w1 = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    const w2 = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let s = 0;
    for (let i = 0; i < 12; i++) s += n[i] * w1[i];
    let r = s % 11;
    n.push(r < 2 ? 0 : 11 - r);
    s = 0;
    for (let i = 0; i < 13; i++) s += n[i] * w2[i];
    r = s % 11;
    n.push(r < 2 ? 0 : 11 - r);
    return n.join("");
  }

  function crc16ccitt(payload) {
    let crc = 0xFFFF;
    for (let i = 0; i < payload.length; i++) {
      crc ^= payload.charCodeAt(i) << 8;
      for (let j = 0; j < 8; j++) {
        if (crc & 0x8000) crc = (crc << 1) ^ 0x1021;
        else crc = crc << 1;
      }
      crc &= 0xFFFF;
    }
    return crc.toString(16).toUpperCase().padStart(4, "0");
  }

  function tlv(id, value) {
    const v = String(value);
    return id + v.length.toString().padStart(2, "0") + v;
  }

  function generatePixCode(opts) {
    const key = (opts && opts.key) || "";
    const merchant = (opts && opts.merchant) || "RECEBEDOR";
    const city = (opts && opts.city) || "BRASILIA";
    const amount = (opts && opts.amount) ? parseFloat(opts.amount).toFixed(2) : "";
    const txid = (opts && opts.txid) || "***";
    const gui = "br.gov.bcb.pix";

    let payload = tlv("00", gui) + tlv("01", key);
    const merchantAcct = payload;
    if (merchant.length > 25) merchant.slice(0, 25);
    if (city.length > 15) city.slice(0, 15);

    let m = tlv("26", merchantAcct);
    m += tlv("52", "0000");
    m += tlv("53", "986");
    if (amount) m += tlv("54", amount);
    m += tlv("58", "BR");
    m += tlv("59", merchant);
    m += tlv("60", city);
    m += tlv("62", tlv("05", txid));

    m += "6304";
    const crc = crc16ccitt(m);
    return m + crc;
  }

  function generateBoletoLine(opts) {
    const bank = (opts && opts.bank) || "001";
    const amount = ((opts && opts.amount) || 0).toFixed(2).replace(".", "").padStart(10, "0");
    const agency = ((opts && opts.agency) || "0001").padStart(4, "0");
    const account = ((opts && opts.account) || "00000000").padStart(8, "0");
    const dv = Math.floor(Math.random() * 10);
    return bank + "9" + amount + agency + account + dv + "0";
  }

  function formatBRL(value) {
    return parseFloat(value).toLocaleString("pt-BR", { style: "currency", currency: "BRL" });
  }

  function parseBRL(value) {
    return parseFloat(String(value).replace(/[R$\s.]/g, "").replace(",", ".")) || 0;
  }

  function workingDaysInMonth(year, month) {
    let count = 0;
    const days = new Date(year, month + 1, 0).getDate();
    for (let d = 1; d <= days; d++) {
      const dow = new Date(year, month, d).getDay();
      if (dow !== 0 && dow !== 6) count++;
    }
    return count;
  }

  function calcRescission(opts) {
    const gross = opts.grossSalary || 0;
    const monthsWorked = opts.monthsWorked || 0;
    const daysWorked = opts.daysWorked || 0;
    const avisoPrevio = opts.avisoPrevio !== false;
    const fgtsBalance = opts.fgtsBalance || 0;
    const fgtsFine = opts.fgtsFine !== false;
    const tipo = opts.tipo || "sem-justa-causa";

    const saldoSalario = (gross / 30) * daysWorked;
    const feriasVencidas = (gross / 12) * Math.floor(monthsWorked / 12);
    const feriasProporcional = (gross / 12) * (monthsWorked % 12);
    const tercoFerias = (feriasVencidas + feriasProporcional) / 3;
    const decimoTerceiroProp = (gross / 12) * (monthsWorked % 12);
    let aviso = 0;
    if (avisoPrevio) aviso = gross * 0.3333;
    let multaFGTS = 0;
    if (fgtsFine && fgtsBalance > 0) {
      multaFGTS = tipo === "sem-justa-causa" ? fgtsBalance * 0.40 : 0;
    }
    const totalBruto = saldoSalario + feriasVencidas + feriasProporcional + tercoFerias + decimoTerceiroProp + aviso;
    const inss = calcINSS(totalBruto).inss;
    const irrf = calcIRRF(totalBruto, inss, 0, false).irrf;
    const totalLiquido = totalBruto - inss - irrf;

    return {
      saldoSalario: saldoSalario,
      feriasVencidas: feriasVencidas,
      feriasProporcional: feriasProporcional,
      tercoFerias: tercoFerias,
      decimoTerceiroProp: decimoTerceiroProp,
      avisoPrevio: aviso,
      multaFGTS: multaFGTS,
      totalBruto: totalBruto + multaFGTS,
      inss: inss,
      irrf: irrf,
      totalLiquido: totalLiquido,
      fgtsBalance: fgtsBalance
    };
  }

  function generateNFeAccessKey(opts) {
    const uf = (opts && opts.uf) || "35";
    const year = (opts && opts.year) || new Date().getFullYear().toString().slice(-2);
    const month = (opts && opts.month) || String(new Date().getMonth() + 1).padStart(2, "0");
    const cnpj = (opts && opts.cnpj) || generateCNPJ();
    const serie = (opts && opts.serie) || "1";
    const number = (opts && opts.number) || String(Math.floor(Math.random() * 1e9)).padStart(9, "0");
    const type = (opts && opts.type) || "1";
    const code = (opts && opts.code) || String(Math.floor(Math.random() * 1e8)).padStart(8, "0");
    const base = uf + year + month + cnpj + serie + number + type + code;
    return base + mod11(base);
  }

  function mod11(num) {
    let seq = "43298765432";
    let s = 0;
    for (let i = 0; i < num.length; i++) s += parseInt(num.charAt(i), 10) * parseInt(seq.charAt(i), 10);
    const r = s % 11;
    return r < 2 ? 0 : 11 - r;
  }

  window.BrazilEngine = {
    INSS_BRACKETS_2024: INSS_BRACKETS_2024,
    INSS_CEILING: INSS_CEILING,
    INSS_MAX_CONTRIBUTION: INSS_MAX_CONTRIBUTION,
    IRRF_BRACKETS_2024: IRRF_BRACKETS_2024,
    IRRF_EXEMPTION_2024: IRRF_EXEMPTION_2024,
    IRRF_DEPENDENT_DEDUCTION: IRRF_DEPENDENT_DEDUCTION,
    IRRF_SIMPLIFIED_DISCOUNT: IRRF_SIMPLIFIED_DISCOUNT,
    calcINSS: calcINSS,
    calcIRRF: calcIRRF,
    calcFGTS: calcFGTS,
    calc13th: calc13th,
    calcFerias: calcFerias,
    calcValeTransporte: calcValeTransporte,
    calcPayroll: calcPayroll,
    calcRescission: calcRescission,
    validateCPF: validateCPF,
    formatCPF: formatCPF,
    generateCPF: generateCPF,
    validateCNPJ: validateCNPJ,
    formatCNPJ: formatCNPJ,
    generateCNPJ: generateCNPJ,
    generatePixCode: generatePixCode,
    generateBoletoLine: generateBoletoLine,
    generateNFeAccessKey: generateNFeAccessKey,
    formatBRL: formatBRL,
    parseBRL: parseBRL,
    workingDaysInMonth: workingDaysInMonth,
    crc16ccitt: crc16ccitt,
    tlv: tlv,
    mod11: mod11
  };
})(window);
