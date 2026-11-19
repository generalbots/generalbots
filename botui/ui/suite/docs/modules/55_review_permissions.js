"use strict";
/* Docs enterprise review workflow (#2425), permission preview.
 *
 * A role/capability matrix so a reviewer can see what each role WOULD be able
 * to do. This is a preview only: server-side enforcement on every review
 * endpoint is explicitly deferred, and the dialog says so in translated text.
 * `can()` is a pure function shared with the matrix and the offline harness. */
(function (window) {
  var NS = window.DocsReview;
  if (!NS) return;

  var CAPABILITIES = [
    ["edit", "docs-review-perm-edit", "Edit content"],
    ["suggest", "docs-review-perm-suggest", "Suggest changes"],
    ["comment", "docs-review-perm-comment", "Comment"],
    ["review", "docs-review-perm-review", "Accept / reject changes"],
    ["signoff", "docs-review-perm-signoff", "Sign off"],
    ["manage", "docs-review-perm-manage", "Manage permissions"]
  ];
  NS.PERMISSION_CAPABILITIES = CAPABILITIES;

  var MATRIX = {
    owner: { edit: true, suggest: true, comment: true, review: true, signoff: true, manage: true },
    editor: { edit: true, suggest: true, comment: true, review: true, signoff: false, manage: false },
    commenter: { edit: false, suggest: true, comment: true, review: false, signoff: false, manage: false },
    viewer: { edit: false, suggest: false, comment: true, review: false, signoff: false, manage: false }
  };
  NS.PERMISSION_MATRIX = MATRIX;

  function can(role, capability) {
    var row = MATRIX[role];
    return !!(row && row[capability]);
  }
  NS.can = can;

  function el(tag, cls) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    return node;
  }

  function i18n(node, key, fallback) {
    node.setAttribute("data-i18n", key);
    node.textContent = NS.tr(key, fallback);
    return node;
  }

  function renderPermissions(host) {
    var panel = host || document.getElementById("docs-review-permissions");
    if (!panel) return;
    panel.textContent = "";
    panel.className = "gb-review-permissions";
    var note = el("p", "gb-review-preview-note");
    i18n(note, "docs-review-perm-preview-note",
      "Preview only — server-side enforcement is not implemented yet.");
    panel.appendChild(note);

    var bar = el("div", "gb-review-roles-bar");
    var label = i18n(el("label"), "docs-review-role-label", "Role");
    var select = el("select", "gb-review-role-select");
    NS.ROLES.forEach(function (role) {
      var option = el("option");
      option.value = role;
      var key = "docs-review-role-" + role;
      var fallback = { owner: "Owner", editor: "Editor", commenter: "Commenter", viewer: "Viewer" }[role];
      i18n(option, key, fallback);
      select.appendChild(option);
    });
    label.appendChild(select);
    bar.appendChild(label);
    panel.appendChild(bar);

    var table = el("table", "gb-review-perm-table");
    var thead = el("thead");
    var headRow = el("tr");
    var capHead = el("th");
    i18n(capHead, "docs-review-perm-capability", "Capability");
    headRow.appendChild(capHead);
    var roleHead = el("th");
    i18n(roleHead, "docs-review-role-label", "Role");
    headRow.appendChild(roleHead);
    thead.appendChild(headRow);
    table.appendChild(thead);
    var tbody = el("tbody");
    panel.appendChild(table);

    function refresh() {
      tbody.textContent = "";
      var role = select.value || "editor";
      CAPABILITIES.forEach(function (spec) {
        var row = el("tr");
        var name = el("td");
        i18n(name, spec[1], spec[2]);
        row.appendChild(name);
        var allowed = can(role, spec[0]);
        var value = el("td");
        var chip = el("span", allowed ? "gb-review-allow" : "gb-review-deny");
        i18n(chip, allowed ? "docs-review-yes" : "docs-review-no", allowed ? "Yes" : "No");
        value.appendChild(chip);
        row.appendChild(value);
        tbody.appendChild(row);
      });
    }
    table.appendChild(tbody);
    select.addEventListener("change", refresh);
    refresh();
  }
  NS.renderPermissions = renderPermissions;

  function openPermissions() {
    if (!NS.dialog || !NS.dialog.shell) return;
    var shell = NS.dialog.shell({
      titleKey: "docs-review-permissions",
      titleFallback: "Permissions preview"
    });
    renderPermissions(shell.body);
    return shell.close;
  }
  NS.openPermissions = openPermissions;
})(window);
