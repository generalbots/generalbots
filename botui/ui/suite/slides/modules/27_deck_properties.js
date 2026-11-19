"use strict";
/* Slides deck properties (#2433): accessible local-only dialog for title,
 * author, subject, keywords, language and tags. PPTX core-property round-trip
 * requires a server compile gate and is deliberately not implemented here. */
(function () {
  var O = window.GBSlidesOutline;
  var STORE = "props";
  var FIELDS = [
    { name: "title", key: "slides-outline-props-title", fallback: "Title" },
    { name: "author", key: "slides-outline-props-author", fallback: "Author" },
    { name: "subject", key: "slides-outline-props-subject", fallback: "Subject" },
    { name: "keywords", key: "slides-outline-props-keywords", fallback: "Keywords" },
    { name: "language", key: "slides-outline-props-language", fallback: "Language" },
    { name: "tags", key: "slides-outline-props-tags", fallback: "Tags" }
  ];
  function current() {
    var stored = O.load(STORE, {});
    var titleField = document.getElementById("presTitle");
    if (!stored.title && titleField && titleField.value) stored.title = titleField.value;
    return stored;
  }
  function fieldRow(labelKey, labelFallback, input) {
    var wrapper = O.el("div", "slides-props-row");
    wrapper.appendChild(O.el("label", "slides-tools-field-label",
      O.t(labelKey, labelFallback), labelKey));
    wrapper.appendChild(input);
    return wrapper;
  }

  function open() {
    var before = current();
    var dialog = O.dialog({
      titleKey: "slides-outline-props-title-dialog", title: "Deck properties"
    });
    var inputs = {};
    FIELDS.forEach(function (field) {
      var input = document.createElement(field.name === "subject" || field.name === "keywords" ? "textarea" : "input");
      if (input.tagName.toLowerCase() === "input") input.type = "text";
      if (input.tagName.toLowerCase() === "textarea") input.rows = 2;
      input.value = before[field.name] || "";
      input.setAttribute("data-i18n-aria-label", field.key);
      input.setAttribute("aria-label", O.t(field.key, field.fallback));
      if (field.name === "keywords" || field.name === "tags") {
        input.setAttribute("data-i18n-placeholder", field.key);
        input.placeholder = O.t(field.key, field.fallback);
      }
      inputs[field.name] = input;
      dialog.body.appendChild(fieldRow(field.key, field.fallback, input));
    });
    dialog.body.appendChild(O.localBadge());
    var cancel = O.el("button", "slides-tools-btn", O.t("slides-outline-cancel", "Cancel"), "slides-outline-cancel");
    var save = O.el("button", "slides-tools-btn primary", O.t("slides-outline-save", "Save"), "slides-outline-save");
    cancel.type = "button"; save.type = "button";
    cancel.addEventListener("click", function () { dialog.dismiss(false); });
    save.addEventListener("click", function () {
      var after = {};
      FIELDS.forEach(function (field) { after[field.name] = (inputs[field.name].value || "").trim(); });
      O.save(STORE, after);
      dialog.dismiss(true);
      O.pushUndo({
        label: O.t("slides-outline-action-props", "edit deck properties"),
        undo: function () { O.save(STORE, before); },
        redo: function () { O.save(STORE, after); }
      });
      O.status(O.t("slides-outline-props-saved", "Deck properties saved locally"));
    });
    dialog.foot.appendChild(cancel);
    dialog.foot.appendChild(save);
    O.updateUndoUI();
    return dialog;
  }

  window.GBSlidesProperties = { open: open };
})();
