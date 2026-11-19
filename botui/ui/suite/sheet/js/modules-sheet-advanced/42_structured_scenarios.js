"use strict";
/* Sheets structured data (#2436) — local scenario manager with comparison. */

(function () {
  const S = window.SheetStructured;
  const t = S.t;
  let activeDialog = null;

  function closeActive() {
    if (activeDialog) {
      activeDialog.close();
      activeDialog = null;
    }
  }

  function fieldRow(labelKey, fallback, control) {
    const row = document.createElement("label");
    row.className = "ss-structured-field";
    const caption = document.createElement("span");
    caption.textContent = t(labelKey, fallback);
    caption.setAttribute("data-i18n", labelKey);
    control.style.width = "100%";
    row.appendChild(caption);
    row.appendChild(control);
    return row;
  }

  function button(labelKey, fallback, className) {
    const node = document.createElement("button");
    node.type = "button";
    node.className = className || "ss-structured-btn";
    node.textContent = t(labelKey, fallback);
    node.setAttribute("data-i18n", labelKey);
    return node;
  }

  function splitRefs(text) {
    return String(text || "").split(",").map(function (part) { return part.trim().toUpperCase(); })
      .filter(function (ref) { return Boolean(S.parseRef(ref)); });
  }

  function capture(refs) {
    return refs.map(function (ref) {
      const pos = S.parseRef(ref);
      return { ref: ref, value: String(S.cellValue(pos.row, pos.col) || "") };
    });
  }

  function valueOf(scenario, ref) {
    const found = scenario.values.filter(function (item) { return item.ref === ref; })[0];
    return found ? found.value : "";
  }

  function applyScenario(entry, dialog) {
    const entries = entry.values.map(function (item) {
      return { ref: item.ref, value: item.value };
    });
    S.writeValues(entries, "sheet-structured-scenarios", "Scenarios");
    const message = t("sheet-structured-scenario-applied", "Scenario {name} applied.", { name: entry.name });
    if (dialog) dialog.setStatus(message, false);
    S.notify(message, false);
    render();
  }

  function renderComparison(host, scenarios) {
    host.innerHTML = "";
    if (!scenarios.length) {
      const empty = document.createElement("p");
      empty.className = "ss-structured-empty";
      empty.textContent = t("sheet-structured-scenario-empty", "No scenarios saved yet.");
      empty.setAttribute("data-i18n", "sheet-structured-scenario-empty");
      host.appendChild(empty);
      return;
    }
    const refs = [];
    scenarios.forEach(function (scenario) {
      scenario.values.forEach(function (item) {
        if (refs.indexOf(item.ref) < 0) refs.push(item.ref);
      });
    });
    const caption = document.createElement("h3");
    caption.className = "ss-structured-subtitle";
    caption.textContent = t("sheet-structured-scenario-compare", "Comparison");
    caption.setAttribute("data-i18n", "sheet-structured-scenario-compare");
    host.appendChild(caption);
    const html = ["<table class=\"ss-structured-grid\"><thead><tr>"];
    html.push("<th scope=\"col\">" + S.escapeHtml(t("sheet-structured-scenario-cell", "Cell")) + "</th>");
    scenarios.forEach(function (scenario) {
      html.push("<th scope=\"col\">" + S.escapeHtml(scenario.name) + "</th>");
    });
    html.push("<th scope=\"col\">" + S.escapeHtml(t("sheet-structured-scenario-current", "Current")) + "</th>");
    html.push("</tr></thead><tbody>");
    refs.forEach(function (ref) {
      const pos = S.parseRef(ref);
      html.push("<tr><th scope=\"row\">" + S.escapeHtml(ref) + "</th>");
      scenarios.forEach(function (scenario) {
        html.push("<td>" + S.escapeHtml(valueOf(scenario, ref)) + "</td>");
      });
      html.push("<td>" + S.escapeHtml(pos ? String(S.cellValue(pos.row, pos.col) || "") : "") + "</td>");
      html.push("</tr>");
    });
    html.push("</tbody></table>");
    host.innerHTML = html.join("");
  }

  function render() {
    if (!activeDialog || !activeDialog.dialog.isConnected) return;
    const dialog = activeDialog;
    const list = dialog.body.querySelector(".ss-structured-scenario-list");
    const comparison = dialog.body.querySelector(".ss-structured-scenario-compare-host");
    const scenarios = S.state().scenarios;
    if (list) {
      list.innerHTML = "";
      if (!scenarios.length) {
        const empty = document.createElement("p");
        empty.className = "ss-structured-empty";
        empty.textContent = t("sheet-structured-scenario-empty", "No scenarios saved yet.");
        empty.setAttribute("data-i18n", "sheet-structured-scenario-empty");
        list.appendChild(empty);
      } else {
        scenarios.forEach(function (entry) {
          const row = document.createElement("div");
          row.className = "ss-structured-list-row";
          const label = document.createElement("span");
          label.textContent = entry.name + " (" + entry.values.length + ")";
          row.appendChild(label);
          const recall = button("sheet-structured-scenario-recall", "Apply", "ss-structured-btn");
          recall.addEventListener("click", function () { applyScenario(entry, dialog); });
          row.appendChild(recall);
          const remove = button("sheet-structured-scenario-delete", "Delete", "ss-structured-btn ss-structured-danger");
          remove.addEventListener("click", function () {
            S.mutate(function (state) {
              state.scenarios = state.scenarios.filter(function (item) { return item.id !== entry.id; });
            });
            render();
          });
          row.appendChild(remove);
          list.appendChild(row);
        });
      }
    }
    if (comparison) renderComparison(comparison, scenarios);
  }

  function open() {
    closeActive();
    const dialog = S.createDialog("sheet-structured-scenarios", "Scenarios");
    activeDialog = dialog;
    dialog.body.appendChild(S.localNote());

    const form = document.createElement("div");
    form.className = "ss-structured-form";
    const nameInput = document.createElement("input");
    nameInput.type = "text";
    nameInput.className = "ss-structured-input";
    const cellsInput = document.createElement("input");
    cellsInput.type = "text";
    cellsInput.className = "ss-structured-input";
    form.appendChild(fieldRow("sheet-structured-scenario-name", "Scenario name", nameInput));
    form.appendChild(fieldRow("sheet-structured-scenario-cells", "Changing cells (comma-separated)", cellsInput));
    dialog.body.appendChild(form);

    const list = document.createElement("div");
    list.className = "ss-structured-list ss-structured-scenario-list";
    dialog.body.appendChild(list);
    const comparison = document.createElement("div");
    comparison.className = "ss-structured-scenario-compare-host";
    dialog.body.appendChild(comparison);

    const save = button("sheet-structured-scenario-save", "Save scenario", "ss-structured-btn ss-structured-primary");
    save.addEventListener("click", function () {
      const refs = splitRefs(cellsInput.value);
      const name = nameInput.value.trim();
      if (!name || !refs.length) {
        dialog.setStatus(t("sheet-structured-goalseek-invalid", "Enter numeric values for the target and changing cells."), true);
        nameInput.focus();
        return;
      }
      const entry = {
        id: "scenario-" + Date.now() + "-" + Math.floor(Math.random() * 1000),
        name: name,
        values: capture(refs),
      };
      S.mutate(function (state) { state.scenarios.push(entry); });
      nameInput.value = "";
      dialog.setStatus(t("sheet-structured-scenario-saved", "Scenario {name} saved.", { name: name }), false);
      render();
    });
    dialog.footer.appendChild(save);
    render();
    nameInput.focus();
  }

  window.SheetStructuredScenarios = { open: open, apply: applyScenario, capture: capture };
})();
