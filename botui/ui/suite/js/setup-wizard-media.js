/* setup-wizard-media.js — optional media capability block for the setup wizard.
 *
 * The desktop shell exposes the authoritative probe through
 * `wizard_media_capabilities`; in a plain browser the platform-level
 * `media_capabilities` from /api/product is used instead. A missing command,
 * service or payload hides the block: setup never blocks on this optional
 * information (#1794).
 */
"use strict";

(function () {
  function tauriMediaCapabilities() {
    var tauri = window.__TAURI__;
    if (tauri && tauri.core && typeof tauri.core.invoke === "function") {
      return tauri.core.invoke("wizard_media_capabilities");
    }
    if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === "function") {
      return window.__TAURI_INTERNALS__.invoke("wizard_media_capabilities");
    }
    return Promise.reject(new Error("Tauri runtime is not available"));
  }

  function normalizeMediaCapability(value) {
    if (!value || typeof value !== "object" || typeof value.available !== "boolean") {
      return null;
    }
    return {
      available: value.available,
      detail: value.detail || value.reason || "",
      remediation: value.remediation || ""
    };
  }

  function normalizeProductMediaCapabilities(capabilities) {
    if (!capabilities || typeof capabilities !== "object") return null;
    var barcode = normalizeMediaCapability(capabilities.barcode);
    var ocr = normalizeMediaCapability(capabilities.ocr);
    var botmodels = capabilities.botmodels || {};
    var platformAvailable = typeof botmodels.available === "boolean"
      ? botmodels.available
      : (typeof capabilities.vision === "boolean" ? capabilities.vision : null);
    if (!barcode && !ocr && platformAvailable === null) return null;
    var reason = botmodels.reason || "The AI media service is unavailable on this platform.";
    var platformNote = "Platform-level capability from /api/product.";
    if (!barcode) {
      barcode = platformAvailable === false
        ? { available: false, detail: reason, remediation: "" }
        : { available: true, detail: platformNote, remediation: "" };
    }
    if (!ocr) {
      ocr = platformAvailable === false
        ? { available: false, detail: reason, remediation: "" }
        : { available: true, detail: platformNote, remediation: "" };
    }
    return { barcode: barcode, ocr: ocr, source: "product" };
  }

  function fetchProductMediaCapabilities() {
    return fetch("/api/product", { cache: "no-store" })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.json();
      })
      .then(function (product) {
        return normalizeProductMediaCapabilities(product && product.media_capabilities);
      });
  }

  function ensureMediaFeatures() {
    var group = document.getElementById("setupMediaFeatures");
    if (group) return group;
    // The desktop shell has its own inline copy of the wizard markup. Create
    // the block there too instead of editing that file from this module.
    var step = document.getElementById("setupStep4");
    var host = step ? step.querySelector(".setup-form") : null;
    if (!host) return null;
    group = document.createElement("div");
    group.className = "setup-form-group";
    group.id = "setupMediaFeatures";
    group.style.display = "none";
    group.innerHTML = '<label>Media features</label>' +
      '<div class="setup-file-list" id="setupMediaFeatureRows"></div>' +
      '<p class="setup-drop-hint" id="setupMediaFeatureNote" style="margin-top: 8px;"></p>';
    host.appendChild(group);
    return group;
  }

  function renderMediaFeatures(capabilities) {
    var group = ensureMediaFeatures();
    var rows = document.getElementById("setupMediaFeatureRows");
    var note = document.getElementById("setupMediaFeatureNote");
    if (!group || !rows) return;
    var entries = [
      ["Barcode / QR", capabilities.barcode],
      ["Text recognition (OCR)", capabilities.ocr]
    ];
    rows.innerHTML = entries.map(function (entry) {
      var label = entry[0];
      var capability = entry[1] || { available: null, detail: "Not reported.", remediation: "" };
      var state = capability.available === true ? "Available"
        : (capability.available === false ? "Unavailable" : "Unknown");
      var kind = capability.available === true ? "uploaded"
        : (capability.available === false ? "error" : "uploading");
      var detail = "";
      if (capability.available !== true) {
        var parts = [capability.detail, capability.remediation].filter(function (part) { return !!part; });
        if (parts.length) {
          detail = '<span class="setup-drop-hint" style="display:block;margin:2px 0 0">' +
            escapeHtml(parts.join(" — ")) + "</span>";
        }
      }
      return '<div class="setup-file-item"><span class="file-name">' + escapeHtml(label) + detail +
        '</span><span class="file-status ' + kind + '">' + state + "</span></div>";
    }).join("");
    if (note) {
      note.textContent = capabilities.source === "botmodels"
        ? "Reported by the AI media service on this machine."
        : "Platform-level report from /api/product; per-feature status is checked by the AI media service.";
    }
    group.style.display = "block";
  }

  function loadMediaFeatures() {
    var group = ensureMediaFeatures();
    if (!group) return;
    var task;
    if (window.__TAURI__ || window.__TAURI_INTERNALS__) {
      task = tauriMediaCapabilities()
        .then(function (payload) {
          if (!payload) return null;
          var barcode = normalizeMediaCapability(payload.barcode);
          var ocr = normalizeMediaCapability(payload.ocr);
          if (!barcode && !ocr) return null;
          return {
            barcode: barcode || { available: false, detail: "Not reported by the AI media service.", remediation: "" },
            ocr: ocr || { available: false, detail: "Not reported by the AI media service.", remediation: "" },
            source: payload.source || "botmodels"
          };
        })
        .catch(function () { return fetchProductMediaCapabilities(); });
    } else {
      task = fetchProductMediaCapabilities();
    }
    task.then(function (capabilities) {
      if (!capabilities) {
        group.style.display = "none";
        return;
      }
      renderMediaFeatures(capabilities);
    }).catch(function (error) {
      console.warn("[SetupWizard] Media capabilities unavailable:", error);
      group.style.display = "none";
    });
  }

  window.SetupWizardMedia = { load: loadMediaFeatures, render: renderMediaFeatures };
})();
