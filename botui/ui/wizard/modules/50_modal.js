/* First-run wizard — reset confirmation modal. */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  var el = App.el, t = App.t;

  function openModal(options) {
    el.modalTitle.textContent = options.title;
    el.modalText.textContent = options.text;
    el.modalField.hidden = !options.fieldLabel;
    el.modalFieldLabel.textContent = options.fieldLabel || "";
    el.modalInput.value = "";
    el.modalConfirm.textContent = options.confirmLabel || t("wizard.confirm");
    el.modal.hidden = false;
    App.modalAction = options.onConfirm;
    if (options.fieldLabel) { el.modalInput.focus(); } else { el.modalConfirm.focus(); }
  }

  function closeModal() {
    el.modal.hidden = true;
    App.modalAction = null;
    el.heading.focus();
  }

  App.openModal = openModal;
  App.closeModal = closeModal;
})();
