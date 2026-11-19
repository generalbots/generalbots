"use strict";
/* Sheets structured data (#2436) — local sparklines: inline SVG in a cell. */

(function () {
  const S = window.SheetStructured;
  const t = S.t;
  const TYPES = ["line", "bar", "winloss"];
  const TYPE_KEYS = {
    line: "sheet-structured-spark-type-line",
    bar: "sheet-structured-spark-type-bar",
    winloss: "sheet-structured-spark-type-winloss",
  };
  const TYPE_FALLBACK = { line: "Line", bar: "Column", winloss: "Win/loss" };
  const GREEN = "#22c55e";
  const RED = "#ef4444";
  const BLUE = "#38bdf8";

  let observer = null;
  let observedNode = null;
  let scheduled = false;
  let activeDialog = null;

  function numericValues(binding) {
    const range = S.parseRange(binding.source);
    if (!range) return [];
    const out = [];
    for (let r = range.start.row; r <= range.end.row; r++) {
      for (let c = range.start.col; c <= range.end.col; c++) {
        const value = S.numeric(S.cellValue(r, c));
        if (value != null) out.push(value);
      }
    }
    return out;
  }

  function fmt(value) {
    const rounded = Math.round(value * 1000) / 1000;
    return String(rounded);
  }

  function sparklineSvg(values, type, alt) {
    const width = 100;
    const height = 24;
    const pad = 2;
    if (!values.length) return "";
    const low = values.length ? Math.min.apply(null, values) : 0;
    const high = values.length ? Math.max.apply(null, values) : 0;
    const span = high - low || 1;
    const scaleY = function (value) {
      return height - pad - ((value - low) / span) * (height - pad * 2);
    };
    const step = values.length > 1 ? (width - pad * 2) / (values.length - 1) : 0;
    let shapes = "";
    if (type === "line") {
      const points = values.map(function (value, index) {
        return fmt(pad + index * step) + "," + fmt(scaleY(value));
      }).join(" ");
      shapes = '<polyline fill="none" stroke="' + BLUE + '" stroke-width="1.6" points="' + points + '" />';
      shapes += '<circle cx="' + fmt(pad + (values.length - 1) * step) + '" cy="' + fmt(scaleY(values[values.length - 1])) + '" r="1.8" fill="' + BLUE + '" />';
    } else {
      const barWidth = Math.max(1, (width - pad * 2) / values.length - 1);
      const zero = low > 0 ? height - pad : (high < 0 ? pad : scaleY(0));
      shapes = '<line x1="' + pad + '" y1="' + fmt(zero) + '" x2="' + (width - pad) + '" y2="' + fmt(zero) + '" stroke="var(--sheet-border, #334155)" stroke-width="0.8" />';
      values.forEach(function (value, index) {
        const x = pad + index * ((width - pad * 2) / values.length);
        const y = scaleY(value);
        const positive = type === "bar" || value >= 0;
        const top = type === "winloss" ? Math.min(y, zero) : y;
        const h = type === "winloss" ? Math.max(1, Math.abs(y - zero)) : Math.max(1, height - pad - y);
        shapes += '<rect x="' + fmt(x) + '" y="' + fmt(top) + '" width="' + fmt(barWidth) + '" height="' + fmt(h) + '" fill="' + (positive ? GREEN : RED) + '" />';
      });
    }
    return '<svg class="ss-spark" viewBox="0 0 ' + width + " " + height + '" preserveAspectRatio="none" role="img" aria-label="' + S.escapeHtml(alt) + '"><title>' + S.escapeHtml(alt) + "</title>" + shapes + "</svg>";
  }

  function altFor(binding, values) {
    const low = values.length ? Math.min.apply(null, values) : 0;
    const high = values.length ? Math.max.apply(null, values) : 0;
    return t("sheet-structured-spark-alt", "Sparkline for {cell}, from {range}: minimum {min}, maximum {max}.", {
      cell: binding.target, range: binding.source, min: fmt(low), max: fmt(high),
    });
  }

  function eachCellNode(callback) {
    const grid = S.activeGrid();
    const root = grid && (grid.bodyInner || grid.body);
    if (!root || !root.querySelectorAll) return;
    Array.prototype.slice.call(root.querySelectorAll("[data-ref]")).forEach(callback);
  }

  function repaintCells() {
    scheduled = false;
    const bindings = S.state().sparklines;
    if (!bindings.length) return;
    eachCellNode(function (node) {
      const ref = node.getAttribute("data-ref");
      const binding = bindings.filter(function (item) { return item.target === ref; })[0];
      if (!binding) {
        if (node.dataset.ssSpark) {
          node.dataset.ssSpark = "";
          node.textContent = String(S.cellValue(parseInt(node.dataset.row, 10), parseInt(node.dataset.col, 10)));
          node.removeAttribute("title");
        }
        return;
      }
      if (node.dataset.ssSpark === binding.id && node.querySelector("svg.ss-spark")) return;
      const values = numericValues(binding);
      if (!values.length) return;
      const alt = altFor(binding, values);
      node.dataset.ssSpark = binding.id;
      node.innerHTML = sparklineSvg(values, binding.type, alt);
      node.setAttribute("title", alt);
    });
  }

  function scheduleRepaint() {
    if (scheduled) return;
    scheduled = true;
    if (window.requestAnimationFrame) window.requestAnimationFrame(repaintCells);
    else repaintCells();
  }

  function ensureObserver() {
    const grid = S.activeGrid();
    const root = grid && (grid.bodyInner || grid.body);
    if (!root || !root.querySelectorAll || root === observedNode) return;
    if (observer) observer.disconnect();
    observedNode = root;
    if (!window.MutationObserver) return;
    observer = new window.MutationObserver(scheduleRepaint);
    observer.observe(root, { childList: true, characterData: true, subtree: true });
  }

  function fieldRow(labelKey, labelFallback, control) {
    const row = document.createElement("label");
    row.className = "ss-structured-field";
    const caption = document.createElement("span");
    caption.textContent = t(labelKey, labelFallback);
    caption.setAttribute("data-i18n", labelKey);
    control.style.width = "100%";
    row.appendChild(caption);
    row.appendChild(control);
    return row;
  }

  function input(value) {
    const node = document.createElement("input");
    node.type = "text";
    node.className = "ss-structured-input";
    if (value) node.value = value;
    return node;
  }

  function typeSelect(selected) {
    const node = document.createElement("select");
    node.className = "ss-structured-input";
    TYPES.forEach(function (type) {
      const option = document.createElement("option");
      option.value = type;
      option.textContent = t(TYPE_KEYS[type], TYPE_FALLBACK[type]);
      option.setAttribute("data-i18n", TYPE_KEYS[type]);
      if (type === selected) option.selected = true;
      node.appendChild(option);
    });
    return node;
  }

  function button(labelKey, fallback, className) {
    const node = document.createElement("button");
    node.type = "button";
    node.className = className || "ss-structured-btn";
    node.textContent = t(labelKey, fallback);
    node.setAttribute("data-i18n", labelKey);
    return node;
  }

  function closeActive() {
    if (activeDialog) {
      activeDialog.close();
      activeDialog = null;
    }
  }

  function open(existing) {
    closeActive();
    ensureObserver();
    const source = existing || {};
    const dialog = S.createDialog("sheet-structured-sparklines", "Sparklines");
    activeDialog = dialog;
    dialog.body.appendChild(S.localNote());

    const form = document.createElement("div");
    form.className = "ss-structured-form";
    const targetInput = input(source.target || "");
    const rangeInput = input(source.source || "");
    const type = typeSelect(source.type || "line");
    form.appendChild(fieldRow("sheet-structured-spark-target", "Target cell", targetInput));
    form.appendChild(fieldRow("sheet-structured-spark-source", "Source range", rangeInput));
    form.appendChild(fieldRow("sheet-structured-spark-type", "Type", type));

    const preview = document.createElement("div");
    preview.className = "ss-structured-preview";
    preview.setAttribute("role", "status");
    preview.setAttribute("aria-live", "polite");
    form.appendChild(preview);
    dialog.body.appendChild(form);

    const actions = document.createElement("div");
    actions.className = "ss-structured-actions";
    const apply = button("sheet-structured-apply", "Apply", "ss-structured-btn ss-structured-primary");
    actions.appendChild(apply);
    if (source.id) {
      const remove = button("sheet-structured-spark-remove", "Remove", "ss-structured-btn ss-structured-danger");
      actions.appendChild(remove);
      remove.addEventListener("click", function () {
        S.mutate(function (state) {
          state.sparklines = state.sparklines.filter(function (item) { return item.id !== source.id; });
        });
        scheduleRepaint();
        dialog.setStatus(t("sheet-structured-spark-removed", "Sparkline removed from {cell}.", { cell: source.target }), false);
        closeActive();
      });
    }
    dialog.footer.appendChild(actions);

    function refresh() {
      const values = numericValues({ source: rangeInput.value });
      if (!values.length) {
        preview.textContent = t("sheet-structured-spark-empty", "No numeric data in the source range.");
        return;
      }
      const alt = altFor({ target: targetInput.value || "?", source: rangeInput.value }, values);
      preview.innerHTML = sparklineSvg(values, type.value, alt);
    }
    form.addEventListener("input", refresh);
    form.addEventListener("change", refresh);

    apply.addEventListener("click", function () {
      const target = targetInput.value.trim();
      const range = rangeInput.value.trim();
      if (!S.parseRef(target) || !S.parseRange(range)) {
        dialog.setStatus(t("sheet-structured-spark-invalid", "Enter a target cell and a valid source range."), true);
        targetInput.focus();
        return;
      }
      const binding = {
        id: source.id || ("spark-" + Date.now() + "-" + Math.floor(Math.random() * 1000)),
        target: target.toUpperCase(), source: range.toUpperCase(), type: type.value,
      };
      S.mutate(function (state) {
        state.sparklines = state.sparklines.filter(function (item) { return item.id !== binding.id; });
        state.sparklines.push(binding);
      });
      ensureObserver();
      scheduleRepaint();
      dialog.setStatus(t("sheet-structured-spark-saved", "Sparkline added to {cell}.", { cell: binding.target }), false);
      S.notify(t("sheet-structured-spark-saved", "Sparkline added to {cell}.", { cell: binding.target }), false);
      closeActive();
      openList();
    });

    refresh();
    targetInput.focus();
  }

  function openList() {
    closeActive();
    const bindings = S.state().sparklines;
    const dialog = S.createDialog("sheet-structured-sparklines", "Sparklines");
    activeDialog = dialog;
    dialog.body.appendChild(S.localNote());
    const list = document.createElement("div");
    list.className = "ss-structured-list";
    if (!bindings.length) {
      const empty = document.createElement("p");
      empty.className = "ss-structured-empty";
      empty.textContent = t("sheet-structured-spark-none", "No sparklines yet.");
      empty.setAttribute("data-i18n", "sheet-structured-spark-none");
      list.appendChild(empty);
    } else {
      bindings.forEach(function (binding) {
        const row = document.createElement("div");
        row.className = "ss-structured-list-row";
        const label = document.createElement("span");
        label.textContent = binding.target + " · " + t(TYPE_KEYS[binding.type] || TYPE_KEYS.line, TYPE_FALLBACK[binding.type] || TYPE_FALLBACK.line);
        row.appendChild(label);
        const edit = button("sheet-structured-spark-edit", "Edit sparkline", "ss-structured-btn");
        edit.addEventListener("click", function () { open(binding); });
        row.appendChild(edit);
        list.appendChild(row);
      });
    }
    dialog.body.appendChild(list);
    const add = button("sheet-structured-spark-add", "Add sparkline", "ss-structured-btn ss-structured-primary");
    add.addEventListener("click", function () { open(null); });
    dialog.footer.appendChild(add);
  }

  window.SheetStructuredSparklines = {
    open: openList,
    openEditor: open,
    svg: sparklineSvg,
    values: numericValues,
    repaint: function () { ensureObserver(); scheduleRepaint(); },
  };
})();
