"use strict";
/* AIFeatures — smart compose, autoformat, suggestions (all client-side, no LLM).
 *
 * Smart Compose: predicts next words based on trigram model of common Portuguese
 * and English patterns. Lightweight, no network.
 *
 * AutoFormat: detects cell content type and applies formatting:
 *   - "R$ 1.234,56" → currency
 *   - "12/05/2024" → date
 *   - "1.234,56" → number
 *   - "25%" → percentage
 *   - "(11) 91234-5678" → phone
 *   - "123.456.789-00" → CPF
 *
 * Suggestions: contextual hints based on:
 *   - column header (R$ → currency, "Total" → sum, "Data" → date)
 *   - previous rows
 *   - formula patterns
 */
(function (window) {

  const TRIGRAMS_PT = {
    "": ["O", "A", "E", "Os", "As", "É", "Para", "Com", "No", "Na", "Em", "Um", "Uma", "Que", "De", "Do", "Da", "Dos", "Das", "Por", "Se", "Não", "Sim", "Já", "Aqui", "Ali", "Quando", "Como", "Porque", "Então", "Mas"],
    "O ": ["que", "fato", "qual", "mesmo", "momento", "tempo", "dia", "ano", "mês", "processo", "resultado", "valor", "cliente", "servidor", "usuário", "sistema", "projeto", "trabalho"],
    "A ": ["que", "fato", "qual", "mesma", "empresa", "pessoa", "data", "hora", "maneira", "forma", "parte", "primeira", "segunda", "última", "nova", "mesma", "grande", "pequena"],
    "E ": ["que", "a", "o", "é", "foi", "será", "tem", "ter", "estar", "estava", "fazer", "fez", "vai", "pode", "deve", "precisa", "quer", "sabe", "conhece"],
    "Para ": ["que", "o", "a", "fazer", "isso", "isto", "aquilo", "onde", "quando", "quem", "qual", "realizar", "obter", "conseguir", "atingir"],
    "Com ": ["o", "a", "isso", "isto", "base", "certeza", "calma", "rapidez", "eficiência", "qualidade", "segurança"],
    "No ": ["entanto", "momento", "caso", "final", "início", "meio", "começo", "ano", "mês", "dia", "horário"],
    "Na ": ["verdade", "realidade", "prática", "teoria", "opinião", "opção", "alternativa", "hipótese", "tentativa", "sequência"],
    "Em ": ["que", "o", "a", "um", "uma", "relação", "geral", "particular", "suma", "média", "total", "análise", "estudo", "pesquisa"],
    "Que ": ["o", "a", "é", "foi", "será", "tem", "está", "faz", "fazia", "irá", "iria", "deve", "pode", "quer", "precisa", "sabe", "conhece"],
    "De ": ["que", "o", "a", "um", "uma", "fato", "modo", "forma", "maneira", "acordo", "novo", "acordo", "acordo", "igual", "novamente"],
    "Não ": ["é", "foi", "tem", "está", "faz", "irá", "pode", "deve", "quer", "sabe", "conhece", "tem", "há", "existe", "possui"]
  };

  const TRIGRAMS_EN = {
    "": ["The", "A", "An", "This", "That", "It", "There", "Here", "When", "Where", "How", "Why", "What", "Who", "Which", "If", "For", "To", "In", "On", "At", "By", "With", "From", "Of", "As", "I", "You", "We", "They", "He", "She"],
    "The ": ["fact", "result", "system", "process", "user", "client", "data", "value", "time", "year", "day", "month", "week", "moment", "reason", "way", "case"],
    "A ": ["new", "single", "specific", "particular", "general", "complete", "full", "partial", "total", "average", "small", "large", "good", "bad", "important"],
    "In ": ["the", "a", "an", "order", "case", "event", "addition", "summary", "general", "particular", "terms", "practice", "theory", "fact", "reality"],
    "To ": ["the", "a", "an", "be", "do", "make", "have", "get", "see", "know", "use", "find", "give", "tell", "work", "call", "try", "ask", "feel", "become", "leave", "put"],
    "It ": ["is", "was", "will", "has", "does", "can", "should", "would", "could", "might", "must", "may", "shall", "needs", "requires"],
    "I ": ["am", "was", "will", "have", "do", "can", "should", "would", "could", "might", "must", "know", "think", "want", "need", "see", "feel", "believe"]
  };

  function predictNext(prefix, lang) {
    if (!prefix) return [];
    const dict = lang === "en" ? TRIGRAMS_EN : TRIGRAMS_PT;
    const last3 = prefix.length >= 3 ? prefix.slice(-3) : (prefix.length >= 2 ? " " + prefix.slice(-2) : prefix);
    let candidates = dict[last3] || dict[prefix.slice(-2) + " "] || dict[prefix.slice(-1) + " "] || [];
    if (last3.length === 3 && dict[last3[1] + last3[2] + " "]) {
      candidates = candidates.concat(dict[last3[1] + last3[2] + " "]);
    }
    return candidates.slice(0, 5);
  }

  function detectFormat(value) {
    if (value == null) return null;
    const v = String(value).trim();
    if (!v) return null;
    if (/^R\$\s?\d/.test(v)) return "currency";
    if (/^\(\d{2}\)\s?\d{4,5}-?\d{4}$/.test(v)) return "phone";
    if (/^\d{3}\.\d{3}\.\d{3}-\d{2}$/.test(v)) return "cpf";
    if (/^\d{2}\.\d{3}\.\d{3}\/\d{4}-\d{2}$/.test(v)) return "cnpj";
    if (/^\d{1,2}\/\d{1,2}\/\d{2,4}$/.test(v)) return "date";
    if (/^\d{1,2}:\d{2}(:\d{2})?$/.test(v)) return "time";
    if (/^\d+%$/.test(v)) return "percentage";
    if (/^-?\d{1,3}(\.\d{3})*(,\d+)?$/.test(v) || /^-?\d+(\.\d+)?$/.test(v)) return "number";
    if (/^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/.test(v)) return "email";
    if (/^https?:\/\//.test(v)) return "url";
    return "text";
  }

  function applyFormat(value, format, locale) {
    const v = String(value).trim();
    const loc = locale || "pt-BR";
    if (format === "currency") {
      const n = parseFloat(v.replace(/[R$\s.]/g, "").replace(",", "."));
      if (isNaN(n)) return v;
      return n.toLocaleString(loc, { style: "currency", currency: "BRL" });
    }
    if (format === "date") {
      const m = v.match(/^(\d{1,2})\/(\d{1,2})\/(\d{2,4})$/);
      if (m) {
        const yr = m[3].length === 2 ? 2000 + parseInt(m[3], 10) : parseInt(m[3], 10);
        return new Date(yr, parseInt(m[2], 10) - 1, parseInt(m[1], 10)).toLocaleDateString(loc, { day: "2-digit", month: "2-digit", year: "numeric" });
      }
    }
    if (format === "percentage") {
      const n = parseFloat(v.replace("%", ""));
      if (isNaN(n)) return v;
      return n.toLocaleString(loc, { style: "percent", minimumFractionDigits: 2 });
    }
    if (format === "number") {
      const n = parseFloat(v.replace(/\./g, "").replace(",", "."));
      if (isNaN(n)) return v;
      return n.toLocaleString(loc, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
    }
    if (format === "phone") {
      const d = v.replace(/\D/g, "");
      if (d.length === 11) return "(" + d.slice(0, 2) + ") " + d.slice(2, 7) + "-" + d.slice(7);
      if (d.length === 10) return "(" + d.slice(0, 2) + ") " + d.slice(2, 6) + "-" + d.slice(6);
    }
    if (format === "cpf") {
      const d = v.replace(/\D/g, "").padStart(11, "0");
      return d.slice(0, 3) + "." + d.slice(3, 6) + "." + d.slice(6, 9) + "-" + d.slice(9);
    }
    if (format === "cnpj") {
      const d = v.replace(/\D/g, "").padStart(14, "0");
      return d.slice(0, 2) + "." + d.slice(2, 5) + "." + d.slice(5, 8) + "/" + d.slice(8, 12) + "-" + d.slice(12);
    }
    return v;
  }

  function autoFormatCell(value, locale) {
    const fmt = detectFormat(value);
    if (!fmt) return value;
    return applyFormat(value, fmt, locale);
  }

  function suggestFormula(header, prevValues) {
    const h = String(header || "").toLowerCase();
    if (h.includes("total") || h.includes("soma")) return "=SUM(" + (prevValues && prevValues.length ? prevValues.join(",") : "A1:A10") + ")";
    if (h.includes("média") || h.includes("media") || h.includes("avg") || h.includes("average")) return "=AVERAGE(A1:A10)";
    if (h.includes("max") || h.includes("máximo") || h.includes("maior")) return "=MAX(A1:A10)";
    if (h.includes("min") || h.includes("mínimo") || h.includes("menor")) return "=MIN(A1:A10)";
    if (h.includes("cont") || h.includes("count") || h.includes("quant")) return "=COUNT(A1:A10)";
    if (h.includes("%") || h.includes("percent") || h.includes("taxa")) return "=A1/B1*100";
    if (h.includes("preço") || h.includes("valor") || h.includes("price")) return "=A1*1.0";
    if (h.includes("data") || h.includes("date")) return "=TODAY()";
    if (h.includes("id") || h.includes("código")) return "=ROW()";
    return null;
  }

  function suggestColumnType(values) {
    if (!values || !values.length) return "text";
    let counts = { number: 0, date: 0, currency: 0, email: 0, phone: 0, cpf: 0, cnpj: 0, percentage: 0, text: 0 };
    values.forEach(v => {
      const f = detectFormat(v);
      if (f && counts[f] !== undefined) counts[f]++;
      else counts.text++;
    });
    let best = "text", max = 0;
    Object.keys(counts).forEach(k => { if (counts[k] > max) { max = counts[k]; best = k; } });
    return best;
  }

  function summarizeText(text, maxSentences) {
    const max = maxSentences || 3;
    if (!text) return "";
    const sentences = text.split(/[.!?]+/).map(s => s.trim()).filter(s => s.length > 10);
    if (sentences.length <= max) return sentences.join(". ") + ".";
    const words = text.toLowerCase().split(/\W+/).filter(w => w.length > 3);
    const freq = {};
    words.forEach(w => { freq[w] = (freq[w] || 0) + 1; });
    const scored = sentences.map(s => {
      const ws = s.toLowerCase().split(/\W+/);
      return { s: s, score: ws.reduce((a, w) => a + (freq[w] || 0), 0) };
    });
    scored.sort((a, b) => b.score - a.score);
    return scored.slice(0, max).map(x => x.s).join(". ") + ".";
  }

  function smartCompose(prefix, lang) {
    const candidates = predictNext(prefix, lang);
    if (!candidates.length) return "";
    return candidates[0];
  }

  window.AIFeatures = {
    predictNext: predictNext,
    smartCompose: smartCompose,
    detectFormat: detectFormat,
    applyFormat: applyFormat,
    autoFormatCell: autoFormatCell,
    suggestFormula: suggestFormula,
    suggestColumnType: suggestColumnType,
    summarizeText: summarizeText,
    TRIGRAMS_PT: TRIGRAMS_PT,
    TRIGRAMS_EN: TRIGRAMS_EN
  };
})(window);
