"use strict";
/* Signature / redaction / protection dialogs (#2427). Everything rendered
 * here is explicitly a preview: no PAdES signing, no byte removal and no
 * server-side permission enforcement. */
(function (window) {
  var NS = window.DocsForms;
  if (!NS) return;
  var D = NS.dialog;
  if (!D) return;
  var UI = NS.dialogs;
  var S = (NS.security = NS.security || {});

  S.signaturePreview = function (name) {
    var label = NS.tr("docs-forms-sign-name", "Signer name");
    return "<div class=\"gb-forms-signature gb-forms-preview-note\">" +
      "<span class=\"gb-forms-signature-label\">" + NS.esc(label) + ": " + NS.esc(name) + "</span>" +
      "<span class=\"gb-forms-preview-note\">" + NS.esc(NS.previewNote()) + "</span></div>";
  };

  S.redactionPreview = function (text) {
    var original = String(text == null ? "" : text);
    var bars = new Array(Math.min(Math.max(original.length, 1), 16) + 1).join("\u2588");
    return "<span class=\"gb-forms-redaction gb-forms-preview-note\" data-form-original=\"" +
      NS.esc(original) + "\">" + bars + "</span>" +
      "<span class=\"gb-forms-preview-note\">" + NS.esc(NS.previewNote()) + "</span>";
  };

  function insertHtml(html) {
    var host = NS.getArticle();
    if (!host || !host.insertAdjacentHTML) return false;
    host.insertAdjacentHTML("beforeend", "<p>" + html + "</p>");
    return true;
  }

  UI.openSign = function () {
    var shell = D.shell({
      titleKey: "docs-forms-sign-title", titleFallback: "Signature and redaction preview", preview: true
    });
    var name = D.input("input", "docs-forms-sign-name-placeholder", "Type the signer name");
    shell.body.appendChild(D.field("docs-forms-sign-name", "Signer name", name));
    var insertSign = D.button("docs-forms-sign-insert", "Insert signature line");
    insertSign.addEventListener("click", function () {
      if (insertHtml(S.signaturePreview(name.value))) {
        shell.status.textContent = NS.tr("docs-forms-sign-applied", "Signature preview applied — not cryptographically signed");
      }
    });
    var applySign = D.button("docs-forms-sign-apply", "Apply signature (preview)", true);
    applySign.addEventListener("click", function () {
      NS.state.signature = { name: name.value, at: Date.now() };
      NS.persist();
      if (insertHtml(S.signaturePreview(name.value))) {
        shell.status.textContent = NS.tr("docs-forms-sign-applied", "Signature preview applied — not cryptographically signed");
      }
    });
    shell.body.appendChild(insertSign);
    shell.body.appendChild(applySign);

    var text = D.input("input", "docs-forms-redact-placeholder-hint", "Type the text to cover in the preview");
    shell.body.appendChild(D.field("docs-forms-redact-placeholder", "Text to redact", text));
    var insertRedact = D.button("docs-forms-redact-insert", "Insert redaction marker");
    insertRedact.addEventListener("click", function () {
      if (insertHtml(S.redactionPreview(text.value))) {
        shell.status.textContent = NS.tr("docs-forms-redact-applied", "Redaction preview applied — stored text is unchanged");
      }
    });
    var applyRedact = D.button("docs-forms-redact-apply", "Mark text as redacted (preview)", true);
    applyRedact.addEventListener("click", function () {
      NS.state.redactions.push({ id: NS.uid("red"), text: text.value });
      NS.persist();
      if (insertHtml(S.redactionPreview(text.value))) {
        shell.status.textContent = NS.tr("docs-forms-redact-applied", "Redaction preview applied — stored text is unchanged");
      }
    });
    shell.body.appendChild(insertRedact);
    shell.body.appendChild(applyRedact);
    return shell.close;
  };

  function passwordField(key, fallback) {
    var input = D.el("input", "gb-forms-input");
    input.setAttribute("type", "password");
    input.setAttribute("data-i18n-placeholder", key);
    input.setAttribute("placeholder", NS.tr(key, fallback));
    return input;
  }

  UI.openProtect = function () {
    var shell = D.shell({ titleKey: "docs-forms-protect-title", titleFallback: "Protect document" });
    var password = passwordField("docs-forms-protect-password", "Password (preview)");
    var confirm = passwordField("docs-forms-protect-confirm", "Confirm password");
    shell.body.appendChild(D.field("docs-forms-protect-password", "Password (preview)", password));
    shell.body.appendChild(D.field("docs-forms-protect-confirm", "Confirm password", confirm));
    var apply = D.button("docs-forms-protect-apply", "Apply protection", true);
    apply.addEventListener("click", function () {
      if (!password.value) {
        shell.status.textContent = NS.tr("docs-forms-protect-empty", "Enter a password (preview)");
        return;
      }
      if (password.value !== confirm.value) {
        shell.status.textContent = NS.tr("docs-forms-protect-mismatch", "Passwords do not match (preview)");
        return;
      }
      NS.protection.enable(password.value);
      shell.status.textContent = NS.tr("docs-forms-protect-on", "Protection on — editing limited to forms and comments");
      shell.close();
    });
    var remove = D.button("docs-forms-protect-remove", "Remove protection");
    remove.addEventListener("click", function () {
      NS.protection.disable();
      shell.status.textContent = NS.tr("docs-forms-protect-off", "Protection off");
      shell.close();
    });
    shell.body.appendChild(apply);
    shell.body.appendChild(remove);
    return shell.close;
  };
})(window);
