"use strict";
/* Slides chart editor (#2486) — layer 6b: Sheet-link panel, accessibility
 * panel, live SVG preview, optional data table and the local round-trip check. */
(function (window) {
  var C = (window.GBSlidesCharts = window.GBSlidesCharts || {});
  var E = C.editor;
  var U = C.ui;
  var S = C.style;
  var L = C.link;

  function linkBadge(working) {
    var status = L.status(working);
    var badge = U.badge(status.stale ? "slides-charts-stale" : "slides-charts-fresh",
      status.stale ? "STALE" : "Up to date", status.stale ? "danger" : "ok");
    var wrap = document.createElement("div");
    wrap.className = "gbch-link-status";
    wrap.appendChild(badge);
    var stamp = status.lastRefreshed ?
      C.tr("slides-charts-last-refresh", "Last refreshed { $ts }", {
        ts: new Date(status.lastRefreshed).toISOString().replace("T", " ").slice(0, 19) + " UTC"
      }) : C.tr("slides-charts-never-refreshed", "Never refreshed");
    wrap.appendChild(U.text("gbch-link-stamp", stamp));
    return wrap;
  }

  function linkPanel(working, redraw, api) {
    var section = U.section("slides-charts-link", "Sheet data link");
    var range = U.field("slides-charts-range", "Sheet range", working.link.range,
      "slides-charts-range-placeholder", "Sheet1!A1:C12");
    section.appendChild(range);
    var open = U.checkbox("slides-charts-refresh-open", "Refresh on open", working.link.refreshOnOpen);
    open.control.addEventListener("change", function () { working.link.refreshOnOpen = open.control.checked; });
    section.appendChild(open);
    section.appendChild(linkBadge(working));
    var actions = document.createElement("div");
    actions.className = "gbch-grid-actions";
    actions.appendChild(U.button("slides-charts-link-apply", "Set link", function () {
      var value = range.control.value.trim();
      if (!L.validRange(value)) {
        api.setStatus("slides-charts-link-invalid", "Use a range like Sheet1!A1:C12.");
        return;
      }
      working.link = { range: value, refreshOnOpen: open.control.checked, lastRefreshed: 0, stale: false };
      api.setStatus("slides-charts-link-set", "Link set to { $range }", { range: value });
      redraw(true);
    }));
    actions.appendChild(U.button("slides-charts-source-changed", "Simulate source change", function () {
      if (!working.link.range) { api.setStatus("slides-charts-link-invalid", "Use a range like Sheet1!A1:C12."); return; }
      working.link.stale = true;
      redraw(true);
    }));
    actions.appendChild(U.button("slides-charts-refresh", "Refresh now", function () {
      if (!working.link.range) { api.setStatus("slides-charts-link-invalid", "Use a range like Sheet1!A1:C12."); return; }
      working.link.lastRefreshed = Date.now();
      working.link.stale = false;
      api.setStatus("slides-charts-status-refreshed", "Chart refreshed locally");
      redraw(true);
    }));
    actions.appendChild(U.button("slides-charts-link-clear", "Clear link", function () {
      working.link = { range: "", refreshOnOpen: false, lastRefreshed: 0, stale: false };
      redraw(true);
    }, "danger"));
    section.appendChild(actions);
    return section;
  }

  function a11yPanel(working, redraw, api) {
    var section = U.section("slides-charts-a11y", "Accessibility");
    var alt = U.field("slides-charts-alt-text", "Alt text", working.a11y.altText,
      "slides-charts-alt-placeholder", "Describe the chart for screen readers");
    alt.control.addEventListener("input", function () { working.a11y.altText = alt.control.value; redraw(false); });
    section.appendChild(alt);
    [["slides-charts-data-table", "Show data table", "showDataTable"],
      ["slides-charts-color-blind", "Colour-blind-safe patterns", "colorBlindSafe"],
      ["slides-charts-contrast-check", "Check series contrast", "contrastCheck"]].forEach(function (entry) {
      var box = U.checkbox(entry[0], entry[1], working.a11y[entry[2]]);
      box.control.addEventListener("change", function () {
        working.a11y[entry[2]] = box.control.checked; redraw(true);
      });
      section.appendChild(box);
    });
    var warnings = S.contrastWarnings(working, E.theme());
    if (warnings.length) {
      warnings.forEach(function (warning) {
        section.appendChild(U.el("p", "gbch-contrast-warning", "slides-charts-contrast-warn",
          "Low contrast on series { $n } ({ $ratio }:1)"));
        section.lastChild.textContent = C.tr("slides-charts-contrast-warn",
          "Low contrast on series { $n } ({ $ratio }:1)",
          { n: warning.seriesIndex + 1, ratio: warning.ratio.toFixed(2) });
      });
    } else if (working.a11y.contrastCheck) {
      section.appendChild(U.el("p", "gbch-contrast-ok", "slides-charts-contrast-ok",
        "All series pass the contrast check."));
    }
    var check = U.button("slides-charts-selftest", "Run round-trip self-check", function () {
      var result = C.selfCheck();
      api.setStatus(result.pass ? "slides-charts-selftest-pass" : "slides-charts-selftest-fail",
        result.pass ? "Round-trip self-check passed: { $n } checks" : "Round-trip self-check failed: { $detail }",
        { n: result.checks.length, detail: result.failed.join(", ") });
    });
    section.appendChild(check);
    return section;
  }

  function previewPanel(working, holder) {
    var section = U.section("slides-charts-preview", "Live preview");
    var frame = document.createElement("div");
    frame.className = "gbch-preview";
    var svg = C.materialize(C.build(working, { theme: E.theme(), width: 440, height: 260 }));
    if (svg) frame.appendChild(svg);
    section.appendChild(frame);
    if (working.a11y.showDataTable) {
      var table = C.materialize(C.dataTable(working), null, true);
      if (table) section.appendChild(table);
    }
    return section;
  }

  E.extraPanels = function (api, working, existing, holder) {
    api.body.appendChild(linkPanel(working, holder.redraw, api));
    api.body.appendChild(a11yPanel(working, holder.redraw, api));
    api.body.appendChild(previewPanel(working, holder));
  };
})(typeof window !== "undefined" ? window : globalThis);
