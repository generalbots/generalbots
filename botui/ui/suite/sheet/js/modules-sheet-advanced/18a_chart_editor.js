"use strict";
/* SHEET-01 step 4 — the chart editor panel.
 *
 * Double-clicking a chart opens this themed panel instead of a native prompt
 * (the suite does not use prompt/alert dialogs). The panel only collects the
 * edit; the caller owns the request and the redraw, so this file stays
 * independent of the grid internals.
 */
window.SheetChartEditor = (function () {
  function field(labelText, control) {
    const row = document.createElement("label");
    row.style.cssText = "display:block;margin-bottom:8px;";
    const span = document.createElement("span");
    span.textContent = labelText;
    span.style.cssText = "display:block;font-size:11px;color:var(--sheet-muted,var(--id-gray-400));margin-bottom:3px;";
    control.style.cssText =
      "width:100%;box-sizing:border-box;padding:5px 7px;border-radius:5px;" +
      "border:1px solid var(--sheet-border,var(--id-ink));background:var(--sheet-bg,var(--id-ink));" +
      "color:var(--sheet-fg,var(--id-gray-100));font:13px system-ui,sans-serif;";
    row.appendChild(span);
    row.appendChild(control);
    return row;
  }

  function open(chart, anchorEl, onApply) {
    const existing = document.getElementById("ss-chart-editor");
    if (existing) existing.remove();

    const box = document.createElement("div");
    box.id = "ss-chart-editor";
    box.setAttribute("role", "dialog");
    box.setAttribute("aria-label", "Edit chart");
    box.style.cssText =
      "position:fixed;z-index:var(--z-overlay);min-width:250px;padding:14px;border-radius:8px;" +
      "background:var(--sheet-bg-elevated,var(--id-ink));color:var(--sheet-fg,var(--id-gray-100));" +
      "border:1px solid var(--sheet-border,var(--id-ink));box-shadow:0 10px 30px color-mix(in srgb, var(--id-ink) 45%, transparent);" +
      "font:13px system-ui,sans-serif;";

    const titleInput = document.createElement("input");
    titleInput.type = "text";
    titleInput.value = chart.title || "";
    titleInput.id = "ssChartTitle";

    const typeSelect = document.createElement("select");
    typeSelect.id = "ssChartType";
    [["bar", "Bar"], ["line", "Line"], ["pie", "Pie"]].forEach(function (pair) {
      const option = document.createElement("option");
      option.value = pair[0];
      option.textContent = pair[1];
      if (chart.chart_type === pair[0]) option.selected = true;
      typeSelect.appendChild(option);
    });

    const rangeInput = document.createElement("input");
    rangeInput.type = "text";
    rangeInput.value = chart.data_range || "";
    rangeInput.id = "ssChartRange";
    rangeInput.placeholder = "A1:B11";

    const actions = document.createElement("div");
    actions.style.cssText = "display:flex;gap:8px;justify-content:flex-end;margin-top:10px;";
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.textContent = "Cancel";
    cancel.style.cssText =
      "padding:5px 12px;border-radius:5px;border:1px solid var(--sheet-border,var(--id-ink));" +
      "background:transparent;color:var(--sheet-fg,var(--id-gray-100));cursor:pointer;";
    const save = document.createElement("button");
    save.type = "button";
    save.textContent = "Apply";
    save.id = "ssChartApply";
    save.style.cssText =
      "padding:5px 12px;border-radius:5px;border:1px solid var(--id-success);background:var(--id-success);" +
      "color:var(--id-surface-solid);cursor:pointer;";

    actions.appendChild(cancel);
    actions.appendChild(save);
    box.appendChild(field("Title", titleInput));
    box.appendChild(field("Type", typeSelect));
    box.appendChild(field("Data range", rangeInput));
    box.appendChild(actions);
    document.body.appendChild(box);

    const rect = anchorEl && anchorEl.getBoundingClientRect
      ? anchorEl.getBoundingClientRect()
      : { left: 40, top: 40, bottom: 40 };
    const top = Math.min(window.innerHeight - box.offsetHeight - 10, rect.bottom + 6);
    const left = Math.min(window.innerWidth - box.offsetWidth - 10, Math.max(10, rect.left));
    box.style.top = Math.max(10, top) + "px";
    box.style.left = Math.max(10, left) + "px";

    const close = function () {
      box.remove();
      document.removeEventListener("pointerdown", dismiss);
      document.removeEventListener("keydown", onKey);
    };
    const dismiss = function (event) {
      if (!box.contains(event.target)) close();
    };
    const onKey = function (event) {
      if (event.key === "Escape") close();
    };
    cancel.addEventListener("click", close);
    save.addEventListener("click", function () {
      close();
      if (typeof onApply === "function") {
        onApply({
          chart_type: typeSelect.value,
          title: titleInput.value,
          data_range: rangeInput.value.trim(),
        });
      }
    });
    setTimeout(function () {
      document.addEventListener("pointerdown", dismiss);
      document.addEventListener("keydown", onKey);
    }, 0);
    titleInput.focus();
  }


  return { open: open };
})();
