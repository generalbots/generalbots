"use strict";
/* Free-model disclosure — shared parity module for Chat, Drive and Settings.
 *
 * The Assistant shell ships this as assistant/modules/10a_settings_model_disclosure.js
 * (text) plus composer/confirm rendering in 04b_status.js and 05d_engine_guard.js.
 * Classic surfaces get the same notice from this single module so the wording
 * and links cannot drift. All user-facing text comes from the i18n catalog.
 */
(function () {
  if (window.GBFreeDisclosure) return;

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function esc(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;")
      .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  var TERMS_URL = "https://kilo.ai/terms";
  var PRIVACY_URL = "https://kilo.ai/privacy";

  function notice() {
    return t("assistant-free-disclosure-notice",
      "The free model runs outside this computer: your message, the conversation and the text of attached files go over the internet to the Kilo service, in the United States, and to the company that runs the model (today NVIDIA), which may log the content and use it to improve their own models and services. The service is for trial use only: do not send personal, confidential or government data; for those, use the local model.");
  }
  function short() {
    return t("assistant-free-disclosure-short",
      "Free model: do not send personal, confidential or government data.");
  }
  function linksHtml() {
    return '<a href="' + esc(TERMS_URL) + '" target="_blank" rel="noopener noreferrer">' +
      esc(t("assistant-free-disclosure-terms", "Terms of service")) + '</a> · ' +
      '<a href="' + esc(PRIVACY_URL) + '" target="_blank" rel="noopener noreferrer">' +
      esc(t("assistant-free-disclosure-privacy", "Privacy")) + '</a>';
  }
  function noticeHtml() {
    return '<p class="gb-free-disclosure">' + esc(notice()) + '</p>' +
      '<p class="gb-free-disclosure-links">' + linksHtml() + '</p>';
  }
  function shortHtml() {
    return '<span class="gb-free-reminder">' + esc(short()) +
      ' <button type="button" class="gb-free-reminder-link" data-open-model-settings>' +
      esc(t("assistant-free-disclosure-learn", "Learn more")) + '</button></span>';
  }

  function isFreeActive() {
    var A = window.GBAssistant;
    if (A && typeof A.engineMode === "string") return A.engineMode === "free";
    var select = document.getElementById("modelPicker");
    if (select && select.value) return /(^|\/)free/i.test(select.value);
    try { return localStorage.getItem("gb-engine-mode") === "free"; } catch (e) { return false; }
  }

  function renderComposerBar() {
    var existing = document.querySelectorAll(".gb-free-bar");
    if (!isFreeActive()) {
      Array.prototype.forEach.call(existing, function (el) { el.remove(); });
      return;
    }
    if (existing.length) return;
    var anchor = document.getElementById("chatForm") || document.querySelector(".input-container") ||
      document.getElementById("drive-toolbar") || document.querySelector(".drive-toolbar");
    if (!anchor || !anchor.parentNode) return;
    var bar = document.createElement("div");
    bar.className = "gb-free-bar";
    bar.setAttribute("role", "status");
    bar.setAttribute("aria-live", "polite");
    bar.innerHTML = shortHtml();
    anchor.parentNode.insertBefore(bar, anchor);
  }

  function renderSettings() {
    var host = document.getElementById("mdl-modes");
    if (!host || document.querySelector(".gb-free-disclosure-settings")) return;
    var block = document.createElement("div");
    block.className = "gb-free-disclosure-settings";
    block.innerHTML = noticeHtml();
    host.parentNode.insertBefore(block, host.nextSibling);
  }

  window.GBFreeDisclosure = {
    notice: notice, short: short, noticeHtml: noticeHtml, shortHtml: shortHtml,
    linksHtml: linksHtml, isFreeActive: isFreeActive,
    renderComposerBar: renderComposerBar, renderSettings: renderSettings,
    TERMS_URL: TERMS_URL, PRIVACY_URL: PRIVACY_URL
  };
})();
