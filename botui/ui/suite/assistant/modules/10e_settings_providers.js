"use strict";
/**
 * 10e_settings_providers.js - Settings > Modelo > "Servidor ou chave própria".
 *
 * A searchable list of model providers (logo, name, what it offers, Conectar).
 * Choosing one fills the endpoint by itself; the person only picks a model
 * and pastes the key. "Servidor da sua organização" keeps the free URL field.
 *
 * `url` is what is saved (the server picks its client from it: Anthropic and
 * Gemini have their own clients, the rest speak the OpenAI shape). `test` is
 * the OpenAI-compatible address the connection test posts to, when different.
 */
(function () {
var A = window.GBAssistant;
function esc(s){return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");}
function S() { return A._modelShared; }

var P = [
  { id:"openai", name:"OpenAI", co:"OpenAI", pt:"Modelos GPT", en:"GPT models",
    url:"https://api.openai.com/v1/chat/completions", models:["gpt-5.6-sol","gpt-5-mini"],
    keys:"https://platform.openai.com/api-keys", list:"https://platform.openai.com/docs/models" },
  { id:"anthropic", name:"Anthropic", co:"Claude", pt:"Claude Opus, Sonnet e Haiku", en:"Claude Opus, Sonnet and Haiku",
    url:"https://api.anthropic.com",
    models:["claude-sonnet-5-5","claude-opus-5-5","claude-haiku-5-5"],
    keys:"https://console.anthropic.com/settings/keys", list:"https://docs.anthropic.com/en/docs/about-claude/models" },
  { id:"gemini", name:"Google Gemini", co:"Google", pt:"Gemini pelo Google AI Studio", en:"Gemini through Google AI Studio",
    url:"https://generativelanguage.googleapis.com", test:"https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
    models:["gemini-3.5-flash","gemini-2.5-pro"],
    keys:"https://aistudio.google.com/apikey", list:"https://ai.google.dev/gemini-api/docs/models" },
  { id:"xai", name:"xAI", co:"Grok", pt:"Modelos Grok", en:"Grok models",
    url:"https://api.x.ai/v1/chat/completions", models:["grok-4"],
    keys:"https://console.x.ai", list:"https://docs.x.ai/docs/models" },
  { id:"meta", name:"Meta Llama", co:"Meta", pt:"Llama pela API da Meta", en:"Llama through Meta’s API",
    url:"https://api.llama.com/compat/v1/chat/completions", models:["Llama-4-Maverick-17B-128E-Instruct-FP8"],
    keys:"https://llama.developer.meta.com", list:"https://llama.developer.meta.com/docs/models" },
  { id:"mistral", name:"Mistral AI", co:"Mistral", pt:"Mistral Large e Small (Europa)", en:"Mistral Large and Small (Europe)",
    url:"https://api.mistral.ai/v1/chat/completions", models:["mistral-large-latest","mistral-small-latest"],
    keys:"https://console.mistral.ai/api-keys", list:"https://docs.mistral.ai/getting-started/models/" },
  { id:"deepseek", name:"DeepSeek", co:"DeepSeek", pt:"DeepSeek Chat e Reasoner", en:"DeepSeek Chat and Reasoner",
    url:"https://api.deepseek.com/chat/completions", models:["deepseek-chat","deepseek-reasoner"],
    keys:"https://platform.deepseek.com/api_keys", list:"https://api-docs.deepseek.com/quick_start/pricing" },
  { id:"qwen", name:"Qwen", co:"Alibaba Cloud", pt:"Qwen pelo Model Studio", en:"Qwen through Model Studio",
    url:"https://dashscope-intl.aliyuncs.com/compatible-mode/v1/chat/completions", models:["qwen-plus","qwen-max"],
    keys:"https://modelstudio.console.alibabacloud.com/", list:"https://www.alibabacloud.com/help/en/model-studio/models" },
  { id:"kimi", name:"Kimi", co:"Moonshot AI", pt:"Modelos Kimi", en:"Kimi models",
    url:"https://api.moonshot.ai/v1/chat/completions", models:["kimi-latest"],
    keys:"https://platform.moonshot.ai/console/api-keys", list:"https://platform.moonshot.ai/docs" },
  { id:"zhipu", name:"GLM", co:"Zhipu (Z.ai)", pt:"Modelos GLM", en:"GLM models",
    url:"https://api.z.ai/api/paas/v4/chat/completions", models:["glm-5.2","glm-4.6"],
    keys:"https://z.ai/manage-apikey/apikey-list", list:"https://docs.z.ai" },
  { id:"hunyuan", name:"Hunyuan", co:"Tencent Cloud", pt:"Modelos Hunyuan", en:"Hunyuan models",
    url:"https://api.hunyuan.cloud.tencent.com/v1/chat/completions", models:["hunyuan-turbos-latest"],
    keys:"https://console.cloud.tencent.com/hunyuan/api-key", list:"https://cloud.tencent.com/document/product/1729" },
  { id:"minimax", name:"MiniMax", co:"MiniMax", pt:"Modelos MiniMax", en:"MiniMax models",
    url:"https://api.minimax.io/v1/chat/completions", models:["MiniMax-M2"],
    keys:"https://www.minimax.io/platform", list:"https://www.minimax.io/platform/document" },
  { id:"openrouter", name:"OpenRouter", co:"OpenRouter", pt:"Centenas de modelos com uma chave só", en:"Hundreds of models with one key",
    url:"https://openrouter.ai/api/v1/chat/completions", models:["openrouter/auto"],
    keys:"https://openrouter.ai/keys", list:"https://openrouter.ai/models" },
  { id:"groq", name:"Groq", co:"Groq", pt:"Modelos abertos com resposta muito rápida", en:"Open models with very fast answers",
    url:"https://api.groq.com/openai/v1/chat/completions", models:["llama-3.3-70b-versatile"],
    keys:"https://console.groq.com/keys", list:"https://console.groq.com/docs/models" }
];

var L = { "pt-BR": {
  intro:"Escolha o provedor. O endereço é preenchido sozinho; você só escolhe o modelo e cola a chave.",
  search:"Buscar provedor", connect:"Conectar", inUse:"Em uso", none:"Nenhum provedor com esse nome.",
  orgT:"Servidor da sua organização", orgD:"Qualquer servidor compatível com a API da OpenAI (vLLM, Ollama, LM Studio, Azure…)",
  add:"Adicionar", back:"Todos os provedores", address:"Endereço", auto:"automático",
  model:"Modelo", other:"Outro modelo…", otherPh:"Nome exato do modelo", models:"Ver modelos",
  key:"Chave de API", getKey:"Onde conseguir a chave", keyPh:"Cole a chave aqui",
  keyAgain:"Há uma chave guardada. Cole a chave de novo para salvar.", keyNeeded:"Cole a chave de API.",
  keyOpt:"Deixe em branco se o servidor não pede chave.", urlPh:"https://servidor.exemplo/v1/chat/completions",
  urlNeeded:"Informe o endereço do servidor.", modelNeeded:"Escolha o modelo.",
  test:"Testar conexão", use:"Usar este provedor", testing:"Testando…",
  ok:"Funcionou: o modelo respondeu em %ms ms.", fail:"Não funcionou (%e).",
  e401:"a chave foi recusada", e400:"o provedor recusou o pedido: confira a chave e o modelo",e404:"modelo ou endereço não encontrado", e429:"limite de uso atingido",
  eConn:"o servidor não respondeu", eTimeout:"o servidor demorou demais",
  e5xx:"o provedor está com problema agora; tente mais tarde", eUrl:"o endereço do servidor não é válido",
  eNoUrl:"falta o endereço do servidor", eClient:"o aplicativo não conseguiu abrir a conexão; reinicie o aplicativo",
  eProvider:"o provedor escolhido não é válido", eAdmin:"só um administrador pode mudar isso",
  eNoText:"o servidor respondeu, mas sem texto do modelo", eOther:"ocorreu um erro inesperado",
  eVaultWrite:"Não foi possível guardar a chave no cofre do computador. Tente novamente; se continuar, reinicie o aplicativo.",
  eVaultDown:"O cofre do computador não está disponível agora. Reinicie o aplicativo e tente novamente.",
  sent:"O que você escrever e anexar vai para %h, fora deste computador.", said:"O provedor disse:" },
en: {
  intro:"Pick the provider. The address fills itself in; you only choose the model and paste the key.",
  search:"Search providers", connect:"Connect", inUse:"In use", none:"No provider by that name.",
  orgT:"Your organization’s server", orgD:"Any server compatible with the OpenAI API (vLLM, Ollama, LM Studio, Azure…)",
  add:"Add", back:"All providers", address:"Address", auto:"automatic",
  model:"Model", other:"Other model…", otherPh:"Exact model name", models:"See models",
  key:"API key", getKey:"Where to get the key", keyPh:"Paste the key here",
  keyAgain:"A key is stored. Paste the key again to save.", keyNeeded:"Paste the API key.",
  keyOpt:"Leave empty if the server needs no key.", urlPh:"https://server.example/v1/chat/completions",
  urlNeeded:"Enter the server address.", modelNeeded:"Choose the model.",
  test:"Test connection", use:"Use this provider", testing:"Testing…",
  ok:"It worked: the model answered in %ms ms.", fail:"It did not work (%e).",
  e401:"the key was refused", e400:"the provider refused the request: check the key and the model",e404:"model or address not found", e429:"usage limit reached",
  eConn:"the server did not answer", eTimeout:"the server took too long",
  e5xx:"the provider is having trouble; try again later", eUrl:"the server address is not valid",
  eNoUrl:"the server address is missing", eClient:"the app could not open the connection; restart the app",
  eProvider:"the chosen provider is not valid", eAdmin:"only an administrator can change this",
  eNoText:"the server answered, but with no text from the model", eOther:"an unexpected error occurred",
  eVaultWrite:"Could not store the key in the computer’s vault. Try again; if it keeps failing, restart the app.",
  eVaultDown:"The computer’s vault is not available right now. Restart the app and try again.",
  sent:"What you write and attach goes to %h, outside this computer.", said:"The provider said:" } };
function pt() { return !!(A.locale && A.locale.startsWith("pt")); }
function g(k) { var l = pt() ? L["pt-BR"] : L.en; return l[k] || L.en[k] || k; }

/** An error code from the server in the person's words; never the raw code. */
var CODES = { connection_refused:"eConn", request_failed:"eConn", timeout:"eTimeout", invalid_endpoint_url:"eUrl",
  endpoint_required:"eNoUrl", http_client_failed:"eClient", invalid_provider:"eProvider", admin_required:"eAdmin",
  endpoint_responded_but_no_choices:"eNoText" };
function reason(code) {
  var e = String(code || "");
  if (/40[13]$/.test(e)) { return g("e401"); } if (/400$/.test(e)) { return g("e400"); } if (/404$/.test(e)) { return g("e404"); }
  if (/429$/.test(e)) { return g("e429"); } if (/_5\d\d$/.test(e)) { return g("e5xx"); }
  return g(CODES[e] || "eOther");
}
/** `tpl` with the reason in place of %e; a vault failure is a whole sentence of its own. */
var WHOLE = { vault_write_failed:"eVaultWrite", vault_unavailable:"eVaultDown" };
function failMsg(tpl, code) { return WHOLE[code] ? g(WHOLE[code]) : tpl.replace("%e", reason(code)); }
function host(u) { try { return new URL(u).hostname; } catch (e) { return ""; } }
function logo(id) {
  var svg = (A._providerLogos || {})[id];
  if (svg) { return '<span class="gb-prov-logo">' + svg + '</span>'; }
  return '<span class="gb-prov-logo gb-prov-logo--org"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">' +
    '<rect x="3" y="4" width="18" height="7" rx="1.5"/><rect x="3" y="13" width="18" height="7" rx="1.5"/><path d="M7 7.5h.01M7 16.5h.01"/></svg></span>';
}
/** The provider the saved configuration points at, if any. */
function current() {
  var c = S().cfg;
  if (!c || c.mode !== "own_key") { return null; }
  var h = c.endpoint_host || "";
  for (var i = 0; i < P.length; i++) { if (host(P[i].url) === h) { return P[i]; } }
  return { id:"org" };
}

function renderList(el, filter) {
  var cur = current();
  var q = String(filter || "").trim().toLowerCase();
  var rows = P.filter(function (p) {
    return !q || (p.name + " " + p.co + " " + p.pt + " " + p.en).toLowerCase().indexOf(q) >= 0;
  }).map(function (p) {
    var on = cur && cur.id === p.id;
    return '<div class="gb-prov-row' + (on ? ' is-on' : '') + '" data-prov="' + p.id + '">' + logo(p.id) +
      '<div class="gb-prov-text"><div class="gb-prov-name">' + esc(p.name) +
        (p.co !== p.name ? ' <span class="gb-prov-co">' + esc(p.co) + '</span>' : '') +
        (on ? ' <span class="gb-prov-badge">' + esc(g("inUse")) + '</span>' : '') + '</div>' +
      '<div class="gb-prov-desc">' + esc(pt() ? p.pt : p.en) + ' · ' + esc(host(p.url)) + '</div></div>' +
      '<button type="button" class="gb-prov-btn" data-prov="' + p.id + '">' + esc(g("connect")) + '</button></div>';
  }).join("");
  var list = el.querySelector(".gb-prov-list");
  list.innerHTML = rows || '<p class="gb-set-hint gb-prov-none">' + esc(g("none")) + '</p>';
  list.querySelectorAll(".gb-prov-row").forEach(function (r) {
    r.addEventListener("click", function () { renderSetup(el, r.getAttribute("data-prov")); });
  });
}

/** Entry point: replaces the old endpoint/model/key form. */
function renderOwnKey(el) {
  var cur = current();
  el.innerHTML = '<div class="gb-prov">' +
    '<p class="gb-set-desc">' + esc(g("intro")) + '</p>' +
    '<label class="gb-prov-search"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="11" cy="11" r="7"/><path d="M20 20l-3.5-3.5"/></svg>' +
      '<input type="search" id="gbProvSearch" placeholder="' + esc(g("search")) + '" aria-label="' + esc(g("search")) + '"/></label>' +
    '<div class="gb-prov-list" role="list"></div>' +
    '<div class="gb-prov-row gb-prov-org' + (cur && cur.id === "org" ? ' is-on' : '') + '" data-prov="org">' + logo("org") +
      '<div class="gb-prov-text"><div class="gb-prov-name">' + esc(g("orgT")) +
        (cur && cur.id === "org" ? ' <span class="gb-prov-badge">' + esc(g("inUse")) + '</span>' : '') + '</div>' +
      '<div class="gb-prov-desc">' + esc(g("orgD")) + '</div></div>' +
      '<button type="button" class="gb-prov-btn">' + esc(g("add")) + '</button></div></div>';
  renderList(el, "");
  var s = el.querySelector("#gbProvSearch");
  s.addEventListener("input", function () { renderList(el, s.value); });
  el.querySelector(".gb-prov-org").addEventListener("click", function () { renderSetup(el, "org"); });
}

function renderSetup(el, id) {
  var p = P.filter(function (x) { return x.id === id; })[0] || null;
  var c = S().cfg || {};
  var cur = current();
  var same = cur && cur.id === id;
  var savedModel = same && c.model ? c.model : "";
  var models = p ? p.models.slice() : [];
  if (savedModel && models.indexOf(savedModel) < 0) { models.unshift(savedModel); }
  var opts = models.map(function (m) {
    return '<option value="' + esc(m) + '"' + (m === (savedModel || models[0]) ? ' selected' : '') + '>' + esc(m) + '</option>';
  }).join("") + '<option value="__other">' + esc(g("other")) + '</option>';
  var link = function (href, text) {
    return '<a class="gb-prov-link" href="' + esc(href) + '" target="_blank" rel="noopener">' + esc(text) + ' ↗</a>';
  };
  el.innerHTML = '<div class="gb-prov gb-prov-setup">' +
    '<button type="button" class="gb-gar-back gb-prov-back">‹ ' + esc(g("back")) + '</button>' +
    '<div class="gb-prov-head">' + logo(id) + '<div><div class="gb-prov-title">' + esc(p ? p.name : g("orgT")) + '</div>' +
      '<div class="gb-prov-desc">' + esc(p ? (pt() ? p.pt : p.en) : g("orgD")) + '</div></div></div>' +
    '<div class="gb-set-form">' +
      '<label class="gb-set-label" for="gbProvUrl">' + esc(g("address")) + (p ? ' <small>(' + esc(g("auto")) + ')</small>' : '') + '</label>' +
      (p ? '<div class="gb-prov-url" id="gbProvUrl">' + esc(p.url) + '</div>'
         : '<input class="gb-set-input" id="gbProvUrl" placeholder="' + esc(g("urlPh")) + '"/>') +
      '<label class="gb-set-label" for="gbProvModel">' + esc(g("model")) + (p ? ' ' + link(p.list, g("models")) : '') + '</label>' +
      (p ? '<select class="gb-set-input" id="gbProvModel">' + opts + '</select>' : '') +
      '<input class="gb-set-input" id="gbProvModelOther" placeholder="' + esc(g("otherPh")) + '"' +
        (p ? ' hidden' : ' value="' + esc(savedModel) + '"') + '/>' +
      '<label class="gb-set-label" for="gbProvKey">' + esc(g("key")) + (p ? ' ' + link(p.keys, g("getKey")) : '') + '</label>' +
      '<input class="gb-set-input" id="gbProvKey" type="password" autocomplete="off" placeholder="' + esc(g("keyPh")) + '"/>' +
      '<p class="gb-set-hint">' + esc(!p ? g("keyOpt") : (same && c.has_key ? g("keyAgain") : g("sent").replace("%h", host(p.url)))) + '</p>' +
      '<div class="gb-set-form-actions">' +
        '<button type="button" class="gb-set-btn-sm" id="gbProvTest">' + esc(g("test")) + '</button>' +
        '<button type="button" class="gb-set-btn-sm gb-set-btn-accent" id="gbProvSave">' + esc(g("use")) + '</button>' +
      '</div><div id="gbProvMsg" role="status"></div></div>' +
    '<div id="gbProvKeyModels"></div></div>';

  el.querySelector(".gb-prov-back").addEventListener("click", function () { renderOwnKey(el); });
  var sel = el.querySelector("#gbProvModel"), other = el.querySelector("#gbProvModelOther");
  if (sel) { sel.addEventListener("change", function () { other.hidden = sel.value !== "__other"; if (!other.hidden) { other.focus(); } }); }

  function read() {
    var url = p ? p.url : el.querySelector("#gbProvUrl").value.trim();
    var model = sel && sel.value !== "__other" ? sel.value : other.value.trim();
    return { url: url, test: p && p.test ? p.test : url, model: model, key: el.querySelector("#gbProvKey").value.trim() };
  }
  function say(cls, text) { el.querySelector("#gbProvMsg").innerHTML = '<p class="' + cls + '">' + esc(text) + '</p>'; }
  function check(v) {
    if (!v.url) { say("gb-set-warn", g("urlNeeded")); return false; }
    if (!v.model) { say("gb-set-warn", g("modelNeeded")); return false; }
    if (p && !v.key) { say("gb-set-warn", g("keyNeeded")); return false; }
    return true;
  }
  el.querySelector("#gbProvTest").addEventListener("click", function () {
    var v = read(); if (!check(v)) { return; }
    say("gb-set-hint", g("testing"));
    var body = { endpoint: v.test, model: v.model }; if (v.key) { body.api_key = v.key; }
    A.apiFetch("/api/llm/config/test", { method: "POST", body: JSON.stringify(body) })
      .then(function (r) { return r.json(); })
      .then(function (d) {
        if (d.reachable && d.model_responded) { say("gb-set-hint gb-set-ok", g("ok").replace("%ms", String(d.latency_ms || "?"))); return; }
        say("gb-set-warn", failMsg(g("fail"), d.error) + (d.detail ? " " + g("said") + " “" + d.detail + "”" : ""));
      })
      .catch(function (e) { say("gb-set-warn", g("fail").replace("%e", String(e && e.message || e))); });
  });
  el.querySelector("#gbProvSave").addEventListener("click", function () {
    var v = read(); if (!check(v)) { return; }
    var btn = el.querySelector("#gbProvSave");
    S().inlineConfirm(S().t("confirmExt").replace("%h", host(v.url) || v.url), function () {
      var body = { mode: "own_key", endpoint: v.url, model: v.model }; if (v.key) { body.api_key = v.key; }
      S().doSwitchMode(body, el.querySelector("#gbProvMsg"), btn);
      el.querySelector("#gbProvKey").value = "";
    });
  });
  if (A._keyModels) {
    A._keyModels.mount(el.querySelector("#gbProvKeyModels"), {
      id: p ? p.id : "org", name: p ? p.name : g("orgT"), keyOptional: !p,
      url: function () { return read().url; }, key: function () { return read().key; }
    });
  }
}

A._modelProviders = { render: renderOwnKey, list: P, t: g, reason: reason, failMsg: failMsg, esc: esc };
})();
