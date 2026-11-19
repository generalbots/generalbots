"use strict";
/* Sheets ingest (#2494) — local privacy/retention policy editor. This is a
 * preview of a policy: it never uploads, processes in a tenant or deletes a
 * real original, and "Remove OCR text" only clears local preview text. */

(function () {
  var C = window.SheetIngestCore;

  function t(key, fallback, params) { return C.t(key, fallback, params); }

  function savePolicy(days, stored, ctx) {
    var parsed = Number(days);
    if (!Number.isFinite(parsed) || parsed < 1 || parsed > 3650) {
      if (ctx) ctx.setStatus(t("sheet-ingest-policy-invalid",
        "Enter a retention period from 1 to 3650 days."), true);
      return false;
    }
    C.mutate(function (target) {
      target.policy.retainDays = Math.floor(parsed);
      target.policy.ocrStored = Boolean(stored);
    });
    if (ctx) ctx.setStatus(t("sheet-ingest-policy-saved",
      "Local retention policy saved."), false);
    return true;
  }

  function removeOcrText(ctx) {
    C.mutate(function (target) {
      target.policy.ocrText = "";
      target.policy.ocrStored = false;
      target.policy.lastRemoved = new Date().toISOString();
    });
    if (ctx) ctx.setStatus(t("sheet-ingest-policy-removed",
      "OCR text removed from the local preview."), false);
  }

  function fieldLabel(host, text, htmlFor) {
    var label = C.element("label", "ss-ingest-field-label", text);
    label.htmlFor = htmlFor;
    host.appendChild(label);
  }

  function render(body, ctx) {
    var policy = ctx.state().policy;
    var section = C.element("section", "ss-ingest-section");
    section.appendChild(C.element("h3", "ss-ingest-section-title",
      t("sheet-ingest-policy-title", "Privacy and retention (local preview)")));
    section.appendChild(C.element("p", "ss-ingest-note",
      t("sheet-ingest-policy-note",
        "No originals are uploaded, processed in a tenant or deleted by this preview.")));

    fieldLabel(section, t("sheet-ingest-policy-retain", "Retain originals (days)"),
      "ss-ingest-retain-days");
    var days = C.element("input", "ss-ingest-input");
    days.type = "number";
    days.min = "1";
    days.max = "3650";
    days.id = "ss-ingest-retain-days";
    days.value = String(policy.retainDays);
    section.appendChild(days);

    var stored = C.element("input", "ss-ingest-checkbox");
    stored.type = "checkbox";
    stored.id = "ss-ingest-ocr-stored";
    stored.checked = Boolean(policy.ocrStored);
    var storedLabel = C.element("label", "ss-ingest-inline-label",
      t("sheet-ingest-policy-ocr", "Store OCR text with sheet"));
    storedLabel.htmlFor = stored.id;
    var storedRow = C.element("div", "ss-ingest-inline-row");
    storedRow.appendChild(stored);
    storedRow.appendChild(storedLabel);
    section.appendChild(storedRow);

    var save = C.element("button", "ss-ingest-btn", t("sheet-ingest-policy-save", "Save policy"));
    save.type = "button";
    save.addEventListener("click", function () {
      if (savePolicy(days.value, stored.checked, ctx)) ctx.rerender();
    });
    section.appendChild(save);

    var remove = C.element("button", "ss-ingest-btn ss-ingest-btn-danger",
      t("sheet-ingest-policy-remove", "Remove OCR text"));
    remove.type = "button";
    remove.addEventListener("click", function () {
      removeOcrText(ctx);
      ctx.rerender();
    });
    section.appendChild(remove);

    if (policy.ocrText) {
      section.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-policy-ocr-preview", "Stored OCR text (local preview): { $text }",
          { text: policy.ocrText.slice(0, 200) })));
    } else {
      section.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-policy-ocr-empty", "No OCR text stored locally.")));
    }
    if (policy.lastRemoved) {
      section.appendChild(C.element("p", "ss-ingest-detail",
        t("sheet-ingest-policy-last-removed", "Removed locally at { $time }.",
          { time: policy.lastRemoved })));
    }
    body.appendChild(section);
  }

  window.SheetIngestPolicy = {
    render: render,
    savePolicy: savePolicy,
    removeOcrText: removeOcrText,
  };
})();
