"use strict";
/* Docs table accessibility check (#2482). Reports missing header scope,
 * inconsistent column counts and unlabelled tables, then jumps to the
 * offending table. The results panel reuses the focus-trapped modal. */
(function (window) {
  var NS = (window.DocsTables = window.DocsTables || {});
  var UI = (window.DocsTablesUI = window.DocsTablesUI || {});
  var A11Y = (window.DocsTablesA11y = window.DocsTablesA11y || {});

  function allTables() {
    var article = NS.getArticle();
    if (!article || !article.querySelectorAll) return [];
    return Array.prototype.slice.call(article.querySelectorAll("table"));
  }

  function labelOf(table, index) {
    return NS.tr("docs-tables-a11y-table-n", "Table {$n}", { n: index + 1 });
  }

  function checkTable(table) {
    var issues = [];
    var hasCaption = false;
    var kids = table.childNodes || [];
    for (var i = 0; i < kids.length; i++) {
      if (kids[i].nodeType === 1 && String(kids[i].tagName).toLowerCase() === "caption") hasCaption = true;
    }
    var named = !!table.getAttribute("aria-label") || !!table.getAttribute("aria-labelledby");
    if (!hasCaption && !named) {
      issues.push({ key: "docs-tables-a11y-unlabelled", fallback: "Table has no caption or accessible name" });
    }
    var headers = table.querySelectorAll ? table.querySelectorAll("th,td[scope]") : [];
    Array.prototype.forEach.call(headers, function (cell) {
      if (NS.closestEl(cell, "table") !== table) return;
      var isHeader = String(cell.tagName).toLowerCase() === "th";
      if (isHeader && !cell.getAttribute("scope")) {
        issues.push({ key: "docs-tables-a11y-no-header-scope", fallback: "Header cell without a scope attribute" });
      }
    });
    var model = NS.buildModel(table);
    if (model && model.rows) {
      var occ = NS.layout(model);
      var broken = false;
      for (var r = 0; r < model.rows && !broken; r++) {
        for (var c = 0; c < model.cols; c++) {
          if (!occ[r][c]) { broken = true; break; }
        }
      }
      if (broken) {
        issues.push({ key: "docs-tables-a11y-inconsistent-cols", fallback: "Rows have inconsistent column counts" });
      }
    }
    return issues;
  }

  function dedupe(issues) {
    var seen = {};
    return issues.filter(function (issue) {
      if (seen[issue.key]) return false;
      seen[issue.key] = true;
      return true;
    });
  }

  A11Y.check = function () {
    var tables = allTables();
    var results = [];
    tables.forEach(function (table, index) {
      var issues = dedupe(checkTable(table));
      if (issues.length) results.push({ table: table, index: index, issues: issues });
    });
    return { tables: tables.length, results: results };
  };

  function jumpTo(table) {
    if (table.scrollIntoView) {
      try { table.scrollIntoView({ block: "center", behavior: "smooth" }); } catch (error) { /* old engine */ }
    }
    NS.focusCell(table, 0, 0);
    if (UI.refresh) UI.refresh(table);
  }

  A11Y.open = function () {
    var report = A11Y.check();
    var closeDialog = UI.openDialog({
      titleKey: "docs-tables-a11y-title", titleFallback: "Table accessibility check",
      applyKey: null,
      render: function (body) {
        var status = UI.el("div", "gb-dtbl-status-line");
        status.setAttribute("role", "status");
        status.setAttribute("aria-live", "polite");
        body.appendChild(status);
        var list = UI.el("div", "gb-dtbl-results");
        body.appendChild(list);
        if (!report.tables) {
          status.textContent = NS.tr("docs-tables-a11y-none", "This document has no tables");
          return;
        }
        if (!report.results.length) {
          status.textContent = NS.tr("docs-tables-a11y-clean", "No issues found ({$n} tables checked)", { n: report.tables });
          return;
        }
        var count = report.results.reduce(function (sum, item) { return sum + item.issues.length; }, 0);
        status.textContent = NS.tr("docs-tables-a11y-summary", "{$n} issue(s) found in {$m} tables",
          { n: count, m: report.results.length });
        report.results.forEach(function (item) {
          var card = UI.el("div", "gb-dtbl-result");
          var text = UI.el("div", "gb-dtbl-issue");
          text.textContent = labelOf(item.table, item.index) + " — " + item.issues.map(function (issue) {
            return NS.tr(issue.key, issue.fallback);
          }).join("; ");
          card.appendChild(text);
          var go = UI.el("button");
          go.setAttribute("type", "button");
          go.setAttribute("data-i18n", "docs-tables-a11y-jump");
          go.textContent = NS.tr("docs-tables-a11y-jump", "Go to table");
          go.addEventListener("click", function () { closeDialog(); jumpTo(item.table); });
          card.appendChild(go);
          list.appendChild(card);
        });
      },
    });
    return closeDialog;
  };
})(window);
