/* First-run wizard — accent picker binding (loads after 00_core.js). */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  if (!App) { return; }
  var Identity = window.GBIdentity;
  var draft = App.draft;

  function persistDraft() { return App.persistDraft(); }

  function applyAccent(hex) {
    draft.appearance = draft.appearance || { accent: "" };
    draft.appearance.accent = hex || "";
    if (Identity) {
      if (hex) { Identity.apply(hex); }
      else { Identity.apply(Identity.presets[0].hex); }
    }
    persistDraft();
  }

  function bindAccentPicker() {
    if (!Identity) { return; }
    var swatches = document.querySelectorAll(".accent-swatch");
    var customInput = document.getElementById("accentCustom");

    Array.prototype.forEach.call(swatches, function (swatch) {
      swatch.addEventListener("click", function () {
        var hex = swatch.getAttribute("data-accent");
        applyAccent(hex);
        syncSwatches(hex);
      });
    });

    if (customInput) {
      customInput.addEventListener("input", function () {
        var hex = customInput.value;
        applyAccent(hex);
        syncSwatches(hex);
      });
    }
  }

  function syncSwatches(activeHex) {
    var swatches = document.querySelectorAll(".accent-swatch");
    var customInput = document.getElementById("accentCustom");
    var isPreset = false;
    Array.prototype.forEach.call(swatches, function (swatch) {
      var hex = swatch.getAttribute("data-accent");
      var match = hex.toLowerCase() === (activeHex || "").toLowerCase();
      swatch.setAttribute("aria-pressed", match ? "true" : "false");
      if (match) { isPreset = true; }
    });
    if (customInput && activeHex) {
      customInput.value = activeHex;
    }
  }

  function accentPickerHtml(draftObj, esc, t) {
    if (!Identity || !Identity.presets) { return ""; }
    var locale = (window.WizardI18n && window.WizardI18n.getLocale()) || "en";
    var usePt = locale.indexOf("pt") === 0;
    var current = (draftObj.appearance && draftObj.appearance.accent) || Identity.presets[0].hex;
    var swatches = Identity.presets.map(function (p) {
      var pressed = p.hex.toLowerCase() === current.toLowerCase();
      var label = usePt ? p.pt : p.en;
      return '<button type="button" class="accent-swatch" data-accent="' + esc(p.hex) + '"' +
        ' aria-pressed="' + (pressed ? "true" : "false") + '"' +
        ' aria-label="' + esc(label) + '"' +
        ' title="' + esc(label) + '"' +
        ' style="--swatch-color:' + esc(p.hex) + '"></button>';
    }).join("");
    return '' +
      '<div class="field" style="margin-top:16px">' +
        '<label class="field-label">' + esc(t("welcome.accent-label")) + "</label>" +
        '<span class="hint">' + esc(t("welcome.accent-hint")) + "</span>" +
        '<div class="accent-picker">' + swatches +
          '<div class="accent-custom-wrap">' +
            '<input type="color" id="accentCustom" class="accent-custom-input"' +
              ' value="' + esc(current) + '" aria-label="' + esc(t("welcome.accent-custom")) + '" />' +
            '<span class="accent-custom-label">' + esc(t("welcome.accent-custom")) + "</span>" +
          "</div>" +
        "</div>" +
      "</div>";
  }

  App.applyAccent = applyAccent;
  App.bindAccentPicker = bindAccentPicker;
  App.syncSwatches = syncSwatches;
  App.accentPickerHtml = accentPickerHtml;
})();
