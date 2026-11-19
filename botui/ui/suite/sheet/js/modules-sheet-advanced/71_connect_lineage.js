"use strict";
/* Sheets data connectivity (#2492) - cell lineage and the no-credential
 * self-check. window.SheetConnectLineage. The lineage matrix is captured when
 * a local snapshot is refreshed; clicking a cell resolves step + source field.
 * The self-check scans only the local definition and a mock export. */

(function () {
  var C = window.SheetConnectCore;
  var F = window.SheetConnectFeed;
  var U = window.SheetConnectUi;

  function stepLabel(target, stepId) {
    if (!stepId || stepId === "source") {
      return C.t("sheet-connect-lineage-source-step", "Source field");
    }
    var step = (target.steps || []).filter(function (item) { return item.id === stepId; })[0];
    if (!step) return stepId;
    var types = window.SheetConnectQuery ? window.SheetConnectQuery.TYPES : [];
    var found = types.filter(function (item) { return item.value === step.type; })[0];
    return found ? C.t(found.key, found.fallback) : step.type;
  }

  function selectCell(row, col, ctx) {
    C.mutate(function (target) {
      var snapshot = target.snapshot;
      if (!snapshot || !snapshot.lineage || !snapshot.lineage[row]) return;
      var cell = snapshot.lineage[row][col] || {};
      target.selectedCell = {
        row: row, col: col, stepId: cell.stepId || "source",
        sourceField: cell.sourceField || "",
      };
    });
    ctx.rerender();
  }

  function renderPanel(container, target) {
    container.appendChild(U.title("sheet-connect-lineage-detail", "Selected cell"));
    var selected = target.selectedCell;
    if (!selected) {
      container.appendChild(U.note("sheet-connect-lineage-hint",
        "Select a cell in the snapshot to see its query step and source field."));
      return;
    }
    var panel = U.el("div", "ss-connect-lineage-panel");
    panel.appendChild(U.keyValue("sheet-connect-lineage-cell", "Cell",
      "R" + (selected.row + 1) + "C" + (selected.col + 1)));
    panel.appendChild(U.keyValue("sheet-connect-lineage-step", "Query step", stepLabel(target, selected.stepId)));
    panel.appendChild(U.keyValue("sheet-connect-lineage-source", "Source field", selected.sourceField || "-"));
    container.appendChild(panel);
  }

  function runCheck(ctx) {
    var result = F.noCredentialScan(ctx.state());
    if (result.pass) {
      ctx.setStatus(C.t("sheet-connect-check-pass",
        "PASS - no credential values in the local definition or mock export."));
    } else {
      ctx.setStatus(C.t("sheet-connect-check-fail", "FAIL - possible credential value found: { $where }",
        { where: result.findings[0] }), true);
    }
    return result;
  }

  function renderCheck(container, ctx) {
    container.appendChild(U.title("sheet-connect-check-credentials", "No credential in workbook - self-check"));
    container.appendChild(U.note("sheet-connect-check-note",
      "Checks the stored local state, definition and a mock export for secret values; only vault:// references are allowed."));
    var actions = U.el("div", "ss-connect-actions");
    actions.appendChild(U.button("sheet-connect-check-run", "Run self-check", function () { runCheck(ctx); }));
    container.appendChild(actions);
  }

  function render(container, ctx) {
    var target = ctx.state();
    container.appendChild(U.title("sheet-connect-lineage-title", "Cell lineage"));
    if (!target.snapshot) {
      container.appendChild(U.note("sheet-connect-no-snapshot", "No cached snapshot yet. Run a refresh first."));
    } else {
      container.appendChild(U.table(target.snapshot.columns, target.snapshot.rows.map(function (row) {
        return row.map(function (cell) { return String(cell); });
      }), function (row, col) { selectCell(row, col, ctx); }));
    }
    renderPanel(container, target);
    renderCheck(container, ctx);
    if (window.SheetConnectRefresh) window.SheetConnectRefresh.renderAudit(container, ctx, target);
  }

  window.SheetConnectLineage = { render: render, runCheck: runCheck, stepLabel: stepLabel };
})();
