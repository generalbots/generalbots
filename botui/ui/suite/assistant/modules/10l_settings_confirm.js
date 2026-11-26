"use strict";
/**
 * 10l_settings_confirm.js - in-page confirmation banner/dialog and error-code
 * sentences. Split from 10_settings_model.js to stay under the 450-line limit.
 */
(function () {
var A = window.GBAssistant;
function esc(s){return String(s==null?"":s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;");}
function t(key){return A._modelShared && A._modelShared.t ? A._modelShared.t(key) : key;}

/* ---- In-page confirm: a banner at the top of the detail tray, or a dialog ---- */
function inlineConfirm(text, onYes) {
  var A2 = window.GBAssistant;
  var yesLabel = A2 && A2.t ? A2.t("assistant.confirm-continue", "Continue") : "OK";
  var noLabel = A2 && A2.t ? A2.t("assistant.cancel", "Cancel") : "Cancel";
  var slot = document.getElementById("gbOfConfirm");
  if (slot && slot.isConnected) {
    slot.innerHTML = '<div class="gb-of-confirm" role="alertdialog" aria-label="' + esc(text) + '">' + (A._modelGarage ? A._modelGarage.svg("M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0zM12 9v4M12 17h.01", 16, 1.8).replace('stroke="currentColor"', 'stroke="var(--id-warn)"') : '') +
      '<span>' + esc(text) + '</span><div><button type="button" class="gb-of-btn" data-act="no">' + esc(noLabel) + '</button>' +
      '<button type="button" class="gb-of-btn pri" data-act="yes">' + esc(yesLabel) + '</button></div></div>';
    var no = slot.querySelector('[data-act="no"]');
    no.focus({ preventScroll: true });
    if (slot.scrollIntoView) { slot.scrollIntoView({ block: "nearest" }); }
    no.addEventListener("click", function () { slot.innerHTML = ""; });
    slot.querySelector('[data-act="yes"]').addEventListener("click", function () { slot.innerHTML = ""; onYes(); });
    return;
  }
  var overlay = document.createElement("div");
  overlay.className = "gb-inline-dialog";
  overlay.innerHTML =
    '<div class="gb-inline-dialog-backdrop"></div>' +
    '<div class="gb-inline-dialog-box">' +
      '<div class="gb-inline-dialog-text">' + esc(text) + '</div>' +
      '<div class="gb-inline-dialog-actions">' +
        '<button class="gb-set-btn-sm" data-act="no">' + esc(noLabel) + '</button>' +
        '<button class="gb-set-btn-sm gb-set-btn-accent" data-act="yes">' + esc(yesLabel) + '</button>' +
      '</div></div>';
  document.body.appendChild(overlay);
  overlay.querySelector('[data-act="no"]').focus();
  function close() { overlay.remove(); document.removeEventListener("keydown", escH); }
  function escH(e) { if (e.key === "Escape") { e.preventDefault(); close(); } }
  document.addEventListener("keydown", escH);
  overlay.querySelector(".gb-inline-dialog-backdrop").addEventListener("click", close);
  overlay.querySelector('[data-act="no"]').addEventListener("click", close);
  overlay.querySelector('[data-act="yes"]').addEventListener("click", function () { close(); onYes(); });
}

/* Server error code -> sentence. Unknown codes keep the code in brackets. */
var ERR = { engine_too_old:"errEngineOld", not_enough_memory:"errMemory", runtime_start_failed:"errStart",
  vault_not_enabled:"errVault", vault_write_failed:"errVault", model_not_downloaded:"errNotDownloaded",
  admin_required:"errAdmin" };
function errText(code) {
  var c = String(code || "");
  return ERR[c] ? t(ERR[c]) : t("errGeneric") + (c ? " (" + c + ")" : "");
}

A._inlineConfirm = inlineConfirm;
A._errText = errText;
})();
