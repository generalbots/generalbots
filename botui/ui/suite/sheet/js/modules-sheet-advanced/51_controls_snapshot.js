"use strict";
/* Sheets enterprise controls (#2437) — signed-snapshot indicator preview.
 * The badge shows a non-cryptographic local hash only: no real signing,
 * certificate handling, Vault credential use or server storage is claimed. */

(function () {
  var C = window.SheetControls;

  function payload() {
    var state = C.state();
    return JSON.stringify({
      protection: state.protection,
      inputs: state.inputs,
      acl: state.acl,
      classification: state.classification,
      links: state.links,
      auditEntries: state.audit.entries,
    });
  }

  function has() {
    var snapshot = C.state().snapshot;
    return Boolean(snapshot && snapshot.hash);
  }

  function create() {
    var hash = C.hash(payload());
    C.mutate(function (state) {
      state.snapshot = { hash: hash, when: new Date().toISOString(), localOnly: true };
    });
    return hash;
  }

  function verify() {
    var snapshot = C.state().snapshot;
    if (!snapshot || !snapshot.hash) return false;
    return C.hash(payload()) === snapshot.hash;
  }

  function statusKey() {
    if (!has()) return "sheet-ctrl-snap-missing";
    return verify() ? "sheet-ctrl-snap-match" : "sheet-ctrl-snap-tampered";
  }

  function statusText(key) {
    var fallbacks = {
      "sheet-ctrl-snap-missing": "No local snapshot preview yet.",
      "sheet-ctrl-snap-match": "Hash matches the local snapshot.",
      "sheet-ctrl-snap-tampered": "Hash mismatch - a change was detected.",
    };
    return C.t(key, fallbacks[key]);
  }

  function render(container, ctx) {
    var doc = container.ownerDocument || document;
    function make(tag, className, text) {
      var node = doc.createElement(tag);
      if (className) node.className = className;
      if (text != null) node.textContent = text;
      return node;
    }
    var grid = make("div", "ss-ctrl-form");
    var note = make("p", "ss-ctrl-scope-note", C.t("sheet-ctrl-snap-note",
      "Preview only: no real signature, Vault credential use, or server storage."));
    note.setAttribute("data-i18n", "sheet-ctrl-snap-note");
    grid.appendChild(note);

    var badge = make("div", "ss-ctrl-snapshot-badge");
    badge.id = "ss-ctrl-snapshot-badge";
    badge.setAttribute("role", "status");
    badge.setAttribute("aria-live", "polite");
    grid.appendChild(badge);

    var actions = make("div", "ss-ctrl-actions");
    var createButton = make("button", "ss-ctrl-button", C.t("sheet-ctrl-snap-create", "Create snapshot preview"));
    createButton.type = "button";
    createButton.id = "ss-ctrl-snap-create";
    createButton.setAttribute("data-i18n", "sheet-ctrl-snap-create");
    var verifyButton = make("button", "ss-ctrl-button", C.t("sheet-ctrl-snap-verify", "Verify snapshot"));
    verifyButton.type = "button";
    verifyButton.id = "ss-ctrl-snap-verify";
    verifyButton.setAttribute("data-i18n", "sheet-ctrl-snap-verify");
    actions.appendChild(createButton);
    actions.appendChild(verifyButton);
    grid.appendChild(actions);
    container.appendChild(grid);

    function refresh() {
      var snapshot = C.state().snapshot;
      var key = statusKey();
      badge.textContent = snapshot && snapshot.hash
        ? C.t("sheet-ctrl-snap-hash", "Local hash preview") + ": " + snapshot.hash
        : C.t("sheet-ctrl-snap-missing", "No local snapshot preview yet.");
      badge.setAttribute("data-status", key);
      ctx.setStatus(statusText(key), key === "sheet-ctrl-snap-tampered");
    }
    refresh();

    createButton.addEventListener("click", function () {
      create();
      refresh();
    });
    verifyButton.addEventListener("click", function () {
      refresh();
    });
  }

  window.SheetControlsSnapshot = {
    payload: payload,
    has: has,
    create: create,
    verify: verify,
    statusKey: statusKey,
    render: render,
  };
})();
