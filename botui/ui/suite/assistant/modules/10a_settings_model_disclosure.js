"use strict";
/**
 * 10a_settings_model_disclosure.js - Free-model disclosure text.
 *
 * The notice that tells people exactly where their data goes when the free
 * hosted model is active, and the short reminder shown near the composer.
 * Extracted so that 10_settings_model.js stays under 450 lines.
 */
(function () {
var A = window.GBAssistant;
function esc(s) { return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;"); }
function pt() { return !!(A.locale && A.locale.startsWith("pt")); }
function tr(p, e) { return pt() ? p : e; }

var TERMS_URL = "https://kilo.ai/terms";
var PRIVACY_URL = "https://kilo.ai/privacy";

var NOTICE = {
  "pt-BR": "O modelo gratuito roda fora deste computador: sua mensagem, a conversa e o texto dos arquivos anexados vão pela internet para o serviço Kilo, nos Estados Unidos, e para a empresa que opera o modelo (hoje a NVIDIA), que podem registrar o conteúdo e usá-lo para melhorar os próprios modelos e serviços. O serviço é só para teste: não envie dados pessoais, sigilosos ou de governo; para esses casos, use o modelo local.",
  en: "The free model runs outside this computer: your message, the conversation and the text of attached files go over the internet to the Kilo service, in the United States, and to the company that runs the model (today NVIDIA), which may log the content and use it to improve their own models and services. The service is for trial use only: do not send personal, confidential or government data; for those, use the local model."
};

var SHORT = {
  "pt-BR": "Modelo gratuito: não envie dados pessoais, sigilosos ou de governo.",
  en: "Free model: do not send personal, confidential or government data."
};

var LEARN_MORE = { "pt-BR": "Saiba mais", en: "Learn more" };
var TERMS_LABEL = { "pt-BR": "Termos do serviço", en: "Terms of service" };
var PRIVACY_LABEL = { "pt-BR": "Privacidade", en: "Privacy" };

function notice() { return pt() ? NOTICE["pt-BR"] : NOTICE.en; }
function short() { return pt() ? SHORT["pt-BR"] : SHORT.en; }

function linksHtml() {
  return '<a href="' + esc(TERMS_URL) + '" target="_blank" rel="noopener noreferrer">' +
    esc(tr(TERMS_LABEL["pt-BR"], TERMS_LABEL.en)) + '</a> · ' +
    '<a href="' + esc(PRIVACY_URL) + '" target="_blank" rel="noopener noreferrer">' +
    esc(tr(PRIVACY_LABEL["pt-BR"], PRIVACY_LABEL.en)) + '</a>';
}

/** Full disclosure block (HTML): the notice paragraph plus the two links. */
function noticeHtml() {
  return '<p class="gb-free-disclosure">' + esc(notice()) + '</p>' +
    '<p class="gb-free-disclosure-links">' + linksHtml() + '</p>';
}

/** One-line reminder (HTML) with a "Learn more" link to Settings > Modelo. */
function shortHtml() {
  return '<span class="gb-free-reminder">' + esc(short()) +
    ' <a href="#" class="gb-free-reminder-link" data-open-model-settings>' +
    esc(tr(LEARN_MORE["pt-BR"], LEARN_MORE.en)) + '</a></span>';
}

/** Is the free hosted model currently the active engine? */
function isFreeActive() {
  if (A.engineMode === "free") { return true; }
  var dest = A.destination || {};
  if (dest.scope === "external" && !A.selectedModel) { return A.engineMode === "free"; }
  return false;
}

A._freeDisclosure = {
  notice: notice,
  short: short,
  noticeHtml: noticeHtml,
  shortHtml: shortHtml,
  linksHtml: linksHtml,
  isFreeActive: isFreeActive,
  TERMS_URL: TERMS_URL,
  PRIVACY_URL: PRIVACY_URL
};
})();
