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

/* The bar shown in a model row, like the compatibility bar of a model hub. */
function affinityBar(m, hw) {
  var a = affinity(m, hw);
  if (a.tone === "none") {
    return '<div class="gb-mac-fit"><span class="gb-mac-fit-label">' + esc(tr("Afinidade com este computador", "Fit with this computer")) +
      '</span><span class="gb-mac-fit-word">' + esc(a.label) + '</span></div>';
  }
  return '<div class="gb-mac-fit gb-mac-' + a.tone + '">' +
    '<div class="gb-mac-fit-top"><span class="gb-mac-fit-label">' + esc(tr("Afinidade com este computador", "Fit with this computer")) + '</span>' +
    '<span class="gb-mac-fit-pct">' + a.pct + '%</span><span class="gb-mac-fit-word">' + esc(a.label) + '</span></div>' +
    '<div class="gb-mac-bar" role="meter" aria-valuemin="0" aria-valuemax="100" aria-valuenow="' + a.pct + '" aria-label="' +
    esc(tr("Afinidade", "Fit")) + '"><i style="width:' + a.pct + '%"></i></div>' +
    '<p class="gb-mac-fit-why">' + esc(a.why) + '</p></div>';
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
  var biggest = runs.reduce(function (b, m) { return !b || m.min_vram_gb > b.min_vram_gb ? m : b; }, null);
  var total = specs.length;
  var text = tr("Roda %n de %t modelos locais%c. O maior que cabe: %b.", "Runs %n of %t local models%c. The largest that fits: %b.")
    .replace("%n", String(runs.length)).replace("%t", String(total))
    .replace("%c", comfy.length ? tr(" (%k com folga)", " (%k comfortably)").replace("%k", String(comfy.length)) : "")
    .replace("%b", biggest ? biggest.name : "—");
  var good = comfy.length && comfy.length === runs.length;
  return { tone: good ? "good" : "mid", short: good ? tr("Serve bem para modelos locais", "Good for local models")
      : tr("Serve, no limite", "Suitable, at the limit"), long: text };
}

function vramBar(hw, engineRunning) {
  var total = Number(hw.vram_gb) || 0, free = Math.min(total, Number(hw.vram_free_gb) || 0);
  if (!total) return "";
  var used = Math.round((total - free) / total * 100);
  var note = engineRunning
    ? tr("%f livres agora; o modelo em uso ocupa parte da placa.", "%f free now; the model in use takes part of the card.")
    : (free < MIN_LOCAL ? tr("%f livres agora: outros programas estão usando a placa. Feche-os antes de usar um modelo local.",
        "%f free now: other programs are using the card. Close them before using a local model.")
      : tr("%f livres agora.", "%f free now."));
  return '<div class="gb-mac-vram"><div class="gb-mac-bar gb-mac-bar-use" aria-hidden="true"><i style="width:' + used + '%"></i></div>' +
    '<span class="gb-mac-sub">' + esc(note.replace("%f", gb(free))) + '</span></div>';
}

function row(label, value, extra) {
  return '<div class="gb-mac-row"><span class="gb-mac-key">' + esc(label) + '</span><div class="gb-mac-val">' + value + (extra || "") + '</div></div>';
}

function card(hw, models, engineRunning) {
  hw = hw || {};
  var v = verdict(hw, models);
  var yes = '<span class="gb-gar-mark yes" aria-hidden="true"></span>', no = '<span class="gb-gar-mark no" aria-hidden="true"></span>';
  var html = '<div class="gb-mac-card"><div class="gb-mac-head"><span class="gb-gar-kicker">' + esc(tr("Seu computador", "Your computer")) + '</span>' +
    '<span class="gb-mac-scan">' + esc(tr("examinado agora", "checked just now")) + '</span></div>';
  html += row(tr("Placa de vídeo", "Graphics card"), '<strong>' + esc(hw.device || tr("nenhuma encontrada", "none found")) + '</strong>');
  html += row("CUDA", '<span class="gb-mac-inline">' + (hw.cuda_usable ? yes + esc(tr("disponível: o modelo usa a placa", "available: the model uses the card"))
    : no + esc(tr("indisponível: o modelo usaria o processador", "unavailable: the model would use the processor"))) + '</span>');
  if (hw.vram_gb) { html += row(tr("Memória de vídeo", "Video memory"), '<strong>' + esc(gb(hw.vram_gb)) + '</strong>', vramBar(hw, engineRunning)); }
  html += row(tr("Memória RAM", "System memory"), esc(gb(hw.ram_gb)));
  if (hw.driver) { html += row(tr("Driver", "Driver"), esc(hw.driver)); }
  html += '<div class="gb-mac-verdict gb-mac-' + v.tone + '"><strong>' + esc(v.short) + '</strong>' + (v.long ? '<span>' + esc(v.long) + '</span>' : '') + '</div>';
  html += '<p class="gb-mac-rule">' + esc(tr(
    "Como medimos: cada modelo precisa de um mínimo de memória de vídeo (o arquivo do modelo mais o espaço da conversa), de 6 GB no menor a 32 GB no maior. Com 25% a mais que o mínimo, roda com folga.",
    "How we measure: each model needs a minimum of video memory (the model file plus room for the conversation), from 6 GB for the smallest to 32 GB for the largest. With 25% more than the minimum it runs comfortably.")) + '</p></div>';
  return html;
}

A._modelMachine = { affinity: affinity, affinityBar: affinityBar, verdict: verdict, card: card };
})();
