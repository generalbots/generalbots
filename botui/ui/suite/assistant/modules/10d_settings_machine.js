"use strict";
/**
 * 10d_settings_machine.js - What this computer can run, for Settings > Modelo.
 * Reads the hardware probe the server sends (graphics card, its memory, the
 * free part of it, system memory, CUDA) and turns it into:
 *   - card(hw, models, engineRunning): the "Seu computador" scan, with a verdict
 *     and the rule behind it;
 *   - affinity(m, hw): how well one model fits this machine, 0-100 %, green,
 *     orange or red, with the reason in words;
 *   - verdict(hw, models): the same verdict in one short line for the table.
 * The rule is the server's (catalog::fit_verdict): a model needs its minimum
 * video memory; with 25 % more it runs comfortably.
 */
(function () {
var A = window.GBAssistant;
function esc(s){return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");}
function pt() { return !!(A.locale && A.locale.startsWith("pt")); }
function tr(p, e) { return pt() ? p : e; }
function gb(n) { var v = Math.round((Number(n) || 0) * 10) / 10; return (pt() ? String(v).replace(".", ",") : String(v)) + " GB"; }

/* Smallest video memory any local model needs (the 7B model). */
var MIN_LOCAL = 6;

function localSpecs(models) {
  var out = [];
  ((models && models.tiers) || []).forEach(function (t) {
    (t.models || []).forEach(function (m) { if (!m.server_only) out.push(m); });
  });
  return out;
}
/* Memory the engine can use: the card when there is one, otherwise system RAM. */
function budget(hw) { return hw.cuda_usable && hw.vram_gb > 0 ? hw.vram_gb : (hw.ram_gb || 0); }

/* ---- One model against this machine ---- */
function affinity(m, hw) {
  hw = hw || {};
  var need = Number(m.min_vram_gb) || 0;
  if (m.server_only || !need) {
    return { pct: 0, tone: "none", label: tr("apenas servidor", "server only"),
      why: tr("Este modelo é grande demais para um computador de mesa: roda só em servidor.",
        "This model is too large for a desktop computer: it runs on servers only.") };
  }
  var have = budget(hw);
  var comfy = need * 1.25;
  var pct = Math.max(0, Math.min(100, Math.round(have / comfy * 100)));
  if (!hw.cuda_usable) {
    return { pct: Math.min(pct, 35), tone: "bad", label: tr("devagar, sem placa de vídeo", "slow, no graphics card"),
      why: tr("Sem placa NVIDIA com CUDA, rodaria no processador usando a memória RAM (%r). Funciona, mas cada resposta pode levar minutos.",
        "Without an NVIDIA card with CUDA it would run on the processor using system memory (%r). It works, but each answer can take minutes.")
        .replace("%r", gb(hw.ram_gb)) };
  }
  var v = m.fit_verdict || (have >= comfy ? "comfortable" : have >= need ? "tight" : "wont_run");
  if (v === "comfortable") {
    return { pct: 100, tone: "good", label: tr("roda com folga", "runs comfortably"),
      why: tr("Precisa de %n de memória de vídeo; esta placa tem %h. Sobra espaço para conversas longas e documentos.",
        "Needs %n of video memory; this card has %h. There is room left for long conversations and documents.")
        .replace("%n", gb(need)).replace("%h", gb(have)) };
  }
  if (v === "tight") {
    return { pct: Math.max(pct, 60), tone: "mid", label: tr("roda, no limite", "runs, at the limit"),
      why: tr("Precisa de %n de memória de vídeo; esta placa tem %h. Roda, mas sem folga: feche jogos e editores de vídeo antes de usar. Com folga seriam %c.",
        "Needs %n of video memory; this card has %h. It runs, without room to spare: close games and video editors first. Comfortable would be %c.")
        .replace("%n", gb(need)).replace("%h", gb(have)).replace("%c", gb(comfy)) };
  }
  return { pct: Math.min(pct, 59), tone: "bad", label: tr("não roda aqui", "won’t run here"),
    why: tr("Precisa de %n de memória de vídeo; esta placa tem %h. Faltam %f.",
      "Needs %n of video memory; this card has %h. It is %f short.")
      .replace("%n", gb(need)).replace("%h", gb(have)).replace("%f", gb(need - have)) };
}

/* ---- The whole machine ---- */
function verdict(hw, models) {
  hw = hw || {};
  var specs = localSpecs(models);
  var have = budget(hw);
  var runs = specs.filter(function (m) { return hw.cuda_usable && m.min_vram_gb <= have; });
  var comfy = runs.filter(function (m) { return have >= m.min_vram_gb * 1.25; });
  if (!hw.device && !hw.cuda_usable && !hw.ram_gb) {
    return { tone: "none", short: tr("Não foi possível examinar este computador", "This computer could not be examined"), long: "" };
  }
  if (!hw.cuda_usable) {
    return { tone: "bad", short: tr("Sem placa de vídeo adequada", "No suitable graphics card"),
      long: tr("Não encontramos uma placa de vídeo NVIDIA com CUDA. Os modelos locais rodariam no processador, muito mais devagar: uma resposta pode levar minutos. Para manter os textos na sua rede, prefira o servidor da sua organização.",
        "No NVIDIA graphics card with CUDA was found. Local models would run on the processor, much slower: one answer can take minutes. To keep your text inside your network, prefer your organization’s server.") };
  }
  if (have < MIN_LOCAL) {
    return { tone: "bad", short: tr("Placa com pouca memória (%v)", "Card with too little memory (%v)").replace("%v", gb(have)),
      long: tr("A placa tem %v de memória de vídeo; o menor modelo local precisa de %m.", "The card has %v of video memory; the smallest local model needs %m.")
        .replace("%v", gb(have)).replace("%m", gb(MIN_LOCAL)) };
  }
  if (!specs.length) {
    return { tone: have >= 10 ? "good" : "mid", short: have >= 10 ? tr("Serve bem para modelos locais", "Good for local models") : tr("Serve, no limite", "Suitable, at the limit"),
      long: tr("Esta placa roda os modelos que pedem até %v de memória de vídeo.", "This card runs the models that need up to %v of video memory.").replace("%v", gb(have)) };
  }
  var all = ((models && models.tiers) || []).reduce(function (n, t) { return n + (t.models || []).length; }, 0) || specs.length;
  var biggest = runs.reduce(function (b, m) { return !b || m.min_vram_gb > b.min_vram_gb ? m : b; }, null);
  var total = all;
  var text = tr("Roda %n de %t modelos locais%c. O maior que cabe: %b.", "Runs %n of %t local models%c. The largest that fits: %b.")
    .replace("%n", String(runs.length)).replace("%t", String(total))
    .replace("%c", comfy.length ? tr(" (%k com folga)", " (%k comfortably)").replace("%k", String(comfy.length)) : "")
    .replace("%b", biggest ? biggest.name : "—");
  var good = comfy.length && comfy.length === runs.length;
  return { tone: good ? "good" : "mid", short: good ? tr("Serve bem para modelos locais", "Good for local models")
      : tr("Serve, no limite", "Suitable, at the limit"), long: text,
    count: tr("Roda %n de %t modelos", "Runs %n of %t models").replace("%n", String(runs.length)).replace("%t", String(total)) };
}

/* Short card name for the panel head: "NVIDIA GeForce RTX 4070 Laptop GPU" -> "RTX 4070". */
function shortCard(d) { return String(d || "").replace(/^NVIDIA\s+(GeForce\s+)?/i, "").replace(/\s+(Laptop\s+)?GPU$/i, "").replace(/\s+Laptop$/i, ""); }

/* Section A of the local detail: the scan of this computer, with the verdict. */
function card(hw, models) {
  hw = hw || {};
  var v = verdict(hw, models), G = A._modelGarage;
  var head = [shortCard(hw.device), hw.vram_gb ? gb(hw.vram_gb) : "", tr("EXAMINADO AGORA", "CHECKED JUST NOW")].filter(Boolean).join(" · ").toUpperCase();
  var html = '<div class="gb-of-sec"><div class="gb-of-sec-h"><span class="gb-of-letter">A</span><span class="gb-of-sec-t">' + esc(tr("Seu computador", "Your computer")) + '</span>' +
    (G ? G.help(tr("Cada modelo precisa de um mínimo de memória de vídeo (o arquivo do modelo mais o espaço da conversa), de 6 GB no menor a 32 GB no maior. Com 25% a mais que o mínimo, roda com folga.",
      "Each model needs a minimum of video memory (the model file plus room for the conversation), from 6 GB for the smallest to 32 GB for the largest. With 25% more than the minimum it runs comfortably."),
      tr("Como medimos", "How we measure")) : '') +
    '<span class="gb-of-sp"></span><span class="gb-of-meta">' + esc(head) + '</span></div><div class="gb-of-hw">';
  html += '<div class="wide"><span class="gb-of-lab">' + tr("PLACA DE VÍDEO", "GRAPHICS CARD") + '</span><span class="v">' + esc(hw.device || tr("nenhuma encontrada", "none found")) + '</span>' +
    (hw.cuda_usable ? '<span class="s ok">✓ ' + esc(tr("CUDA disponível: o modelo usa a placa", "CUDA available: the model uses the card")) + '</span>'
      : '<span class="s">– ' + esc(tr("CUDA indisponível: o modelo usaria o processador", "CUDA unavailable: the model would use the processor")) + '</span>') + '</div>';
  if (hw.vram_gb) {
    var total = Number(hw.vram_gb) || 0, free = Math.min(total, Number(hw.vram_free_gb) || 0);
    html += '<div class="mem"><span class="gb-of-lab">' + tr("MEMÓRIA DE VÍDEO", "VIDEO MEMORY") + '</span><span class="v">' + esc(gb(total)) + '</span>' +
      '<div class="gb-of-bar" role="meter" aria-valuemin="0" aria-valuemax="100" aria-valuenow="' + Math.round((total - free) / total * 100) + '" aria-label="' + esc(tr("Memória de vídeo em uso", "Video memory in use")) + '">' +
      '<i style="width:' + Math.round((total - free) / total * 100) + '%"></i></div><span class="s">' +
      esc((free >= 2 ? tr("%f livres agora", "%f free now") : tr("%f livre agora", "%f free now")).replace("%f", gb(free))) + '</span></div>';
  }
  html += '<div><span class="gb-of-lab">' + (hw.driver ? tr("RAM · DRIVER", "RAM · DRIVER") : "RAM") + '</span><span class="v">' + esc(gb(hw.ram_gb)) + '</span>' +
    (hw.driver ? '<span class="s mono">' + esc(hw.driver) + '</span>' : '') + '</div>';
  html += '<div class="verdict" data-tone="' + v.tone + '"' + (v.long ? ' tabindex="0" data-tip="' + esc(v.long) + '"' : '') + '><span class="gb-of-lab">' + tr("VEREDITO", "VERDICT") +
    '</span><span class="v">' + esc(v.short) + '</span>' + (v.count ? '<span class="s">' + esc(v.count) + '</span>' : '') + '</div>';
  return html + '</div></div>';
}

A._modelMachine = { affinity: affinity, verdict: verdict, card: card, gb: gb };
})();
