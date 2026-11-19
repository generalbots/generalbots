"use strict";
/* Sheets print, reporting and scheduled delivery (#2438) — report policy
 * editor. Policies and next-run calculations are local previews only: this
 * module never registers a cron job, exports a file, sends a message or
 * writes an audit entry. */

(function () {
  var C = window.SheetReportingCore;
  var t = C.t;
  var FORMATS = ["PDF", "XLSX", "CSV"];
  var FREQUENCIES = ["daily", "weekly", "monthly"];
  var FREQ_KEYS = {
    daily: "sheet-rpt-freq-daily",
    weekly: "sheet-rpt-freq-weekly",
    monthly: "sheet-rpt-freq-monthly",
  };
  var DAYS = [1, 2, 3, 4, 5, 6, 0];
  var DAY_KEYS = {
    1: "sheet-rpt-day-mon", 2: "sheet-rpt-day-tue", 3: "sheet-rpt-day-wed",
    4: "sheet-rpt-day-thu", 5: "sheet-rpt-day-fri", 6: "sheet-rpt-day-sat", 0: "sheet-rpt-day-sun",
  };
  var DAY_FALLBACKS = {
    1: "Monday", 2: "Tuesday", 3: "Wednesday", 4: "Thursday",
    5: "Friday", 6: "Saturday", 0: "Sunday",
  };

  function makeId(prefix) {
    return prefix + "-" + Date.now().toString(36) + "-" + Math.floor(Math.random() * 1e5).toString(36);
  }

  function recipients(value) {
    return String(value == null ? "" : value).split(",").map(function (item) { return item.trim(); })
      .filter(function (item) { return item.length > 0; });
  }

  function validatePolicy(policy) {
    var errors = [];
    var p = policy || {};
    if (!String(p.name || "").trim()) {
      errors.push({ key: "sheet-rpt-policy-name-missing", fallback: "Enter a report name." });
    }
    if (!C.parseRange(String(p.sourceRange || "").trim())) {
      errors.push({ key: "sheet-rpt-invalid-range", fallback: "Enter a valid range such as A1:H60." });
    }
    if (FORMATS.indexOf(p.format) < 0) {
      errors.push({ key: "sheet-rpt-policy-format-invalid", fallback: "Choose PDF, XLSX or CSV." });
    }
    if (FREQUENCIES.indexOf(p.frequency) < 0) {
      errors.push({ key: "sheet-rpt-policy-frequency-invalid", fallback: "Choose daily, weekly or monthly." });
    }
    var list = recipients(p.recipientList);
    if (!list.length || list.some(function (mail) { return !/^[^\s@,]+@[^\s@,]+\.[^\s@,]+$/.test(mail); })) {
      errors.push({ key: "sheet-rpt-policy-recipients-invalid", fallback: "Enter at least one valid email address." });
    }
    return { ok: errors.length === 0, errors: errors };
  }

  function nextRun(policy, from) {
    var p = policy || {};
    var now = from instanceof Date ? from : new Date(from);
    if (Number.isNaN(now.getTime()) || FREQUENCIES.indexOf(p.frequency) < 0) return null;
    var date = new Date(now.getTime());
    date.setHours(7, 0, 0, 0);
    if (p.frequency === "daily") {
      if (date.getTime() <= now.getTime()) date.setDate(date.getDate() + 1);
      return date;
    }
    if (p.frequency === "weekly") {
      var target = Number(p.dayOfWeek);
      if (!Number.isInteger(target) || target < 0 || target > 6) target = 1;
      var delta = (target - date.getDay() + 7) % 7;
      if (delta === 0 && date.getTime() <= now.getTime()) delta = 7;
      date.setDate(date.getDate() + delta);
      return date;
    }
    var dayOfMonth = Number(p.dayOfMonth);
    if (!Number.isInteger(dayOfMonth) || dayOfMonth < 1 || dayOfMonth > 28) dayOfMonth = 1;
    date.setDate(dayOfMonth);
    if (date.getTime() <= now.getTime()) {
      date.setMonth(date.getMonth() + 1);
      date.setDate(dayOfMonth);
    }
    return date;
  }

  function addPolicy(fields) {
    var valid = validatePolicy(fields);
    if (!valid.ok) return valid;
    var policy = {
      id: makeId("policy"),
      name: String(fields.name).trim().slice(0, 80),
      sourceRange: String(fields.sourceRange).trim().toUpperCase(),
      format: fields.format,
      frequency: fields.frequency,
      dayOfWeek: Number(fields.dayOfWeek),
      dayOfMonth: Number(fields.dayOfMonth),
      recipientList: recipients(fields.recipientList).join(", "),
      enabled: Boolean(fields.enabled),
    };
    C.mutate(function (state) { state.schedules.push(policy); });
    return { ok: true, policy: policy, key: "sheet-rpt-sched-added", fallback: "Policy saved to the local preview." };
  }

  function removePolicy(id) {
    C.mutate(function (state) {
      state.schedules = state.schedules.filter(function (item) { return item.id !== id; });
    });
    return true;
  }

  function option(value, labelKey, labelFallback) {
    var item = C.element("option", "", t(labelKey, labelFallback));
    item.value = value;
    item.setAttribute("data-i18n", labelKey);
    return item;
  }

  function render(container, ctx) {
    container.textContent = "";
    var note = C.element("p", "ss-rpt-sched-note", t("sheet-rpt-sched-note",
      "Local policy preview only - no real scheduling, PDF export, chat/email delivery or audit entry is created."));
    note.setAttribute("data-i18n", "sheet-rpt-sched-note");
    container.appendChild(note);
    var form = C.element("div", "ss-rpt-form");
    var inputs = {};
    function text(key, fallback, type) {
      var input = C.element("input", "ss-rpt-input");
      input.type = type || "text";
      input.placeholder = t(key, fallback);
      input.setAttribute("data-i18n-placeholder", key);
      return input;
    }
    inputs.name = text("sheet-rpt-sched-name-ph", "Report name");
    inputs.sourceRange = text("sheet-rpt-sched-range-ph", "A1:H60");
    inputs.recipientList = text("sheet-rpt-sched-recipients-ph", "team@example.com, manager@example.com");
    inputs.format = C.element("select", "ss-rpt-input");
    FORMATS.forEach(function (value) {
      inputs.format.appendChild(option(value, "sheet-rpt-format-" + value.toLowerCase(), value));
    });
    inputs.frequency = C.element("select", "ss-rpt-input");
    FREQUENCIES.forEach(function (value) {
      inputs.frequency.appendChild(option(value, "sheet-rpt-freq-" + value, value));
    });
    inputs.dayOfWeek = C.element("select", "ss-rpt-input");
    DAYS.forEach(function (value) {
      inputs.dayOfWeek.appendChild(option(String(value), DAY_KEYS[value], DAY_FALLBACKS[value]));
    });
    inputs.dayOfWeek.value = "1";
    inputs.dayOfMonth = text("sheet-rpt-sched-dom-ph", "1", "number");
    inputs.dayOfMonth.value = "1";
    inputs.enabled = C.element("input", "ss-rpt-checkbox");
    inputs.enabled.type = "checkbox";
    inputs.enabled.checked = true;
    function row(key, fallback, control) {
      var label = C.element("label", "ss-rpt-field");
      label.appendChild(C.element("span", "ss-rpt-label", t(key, fallback)));
      label.appendChild(control);
      form.appendChild(label);
    }
    row("sheet-rpt-sched-name", "Report name", inputs.name);
    row("sheet-rpt-sched-range", "Source range", inputs.sourceRange);
    row("sheet-rpt-sched-format", "Format", inputs.format);
    row("sheet-rpt-sched-frequency", "Frequency", inputs.frequency);
    row("sheet-rpt-sched-day", "Weekday", inputs.dayOfWeek);
    row("sheet-rpt-sched-dom", "Day of month", inputs.dayOfMonth);
    row("sheet-rpt-sched-recipients", "Recipients", inputs.recipientList);
    row("sheet-rpt-sched-enabled", "Enabled in local policy", inputs.enabled);
    var save = C.element("button", "ss-rpt-button", t("sheet-rpt-sched-add", "Save policy"));
    save.type = "button";
    save.setAttribute("data-i18n", "sheet-rpt-sched-add");
    save.addEventListener("click", function () {
      var result = addPolicy({
        name: inputs.name.value,
        sourceRange: inputs.sourceRange.value,
        format: inputs.format.value,
        frequency: inputs.frequency.value,
        dayOfWeek: inputs.dayOfWeek.value,
        dayOfMonth: inputs.dayOfMonth.value,
        recipientList: inputs.recipientList.value,
        enabled: inputs.enabled.checked,
      });
      ctx.setStatus(t(result.key, result.fallback), !result.ok);
      if (result.ok) ctx.rerender();
    });
    form.appendChild(save);
    container.appendChild(form);

    var preview = C.element("p", "ss-rpt-status", "");
    preview.setAttribute("role", "status");
    preview.setAttribute("aria-live", "polite");
    function updatePreview() {
      var when = nextRun({
        frequency: inputs.frequency.value,
        dayOfWeek: inputs.dayOfWeek.value,
        dayOfMonth: inputs.dayOfMonth.value,
      }, new Date());
      preview.textContent = when
        ? t("sheet-rpt-sched-next", "Next run (local preview): {when}", { when: when.toISOString().slice(0, 16).replace("T", " ") })
        : "";
    }
    [inputs.frequency, inputs.dayOfWeek, inputs.dayOfMonth].forEach(function (control) {
      control.addEventListener("change", updatePreview);
    });
    updatePreview();
    container.appendChild(preview);

    var list = C.element("ul", "ss-rpt-list");
    var policies = C.state().schedules;
    if (!policies.length) {
      list.appendChild(C.element("li", "ss-rpt-empty", t("sheet-rpt-sched-empty", "No scheduled report policies yet.")));
    }
    policies.forEach(function (policy) {
      var item = C.element("li", "ss-rpt-list-item");
      var stateKey = policy.enabled ? "sheet-rpt-state-enabled" : "sheet-rpt-state-disabled";
      item.appendChild(C.element("span", "ss-rpt-item-label", t("sheet-rpt-sched-list-item",
        "{name} - {frequency} - {format} - {state}", {
          name: policy.name,
          frequency: t(FREQ_KEYS[policy.frequency] || policy.frequency, policy.frequency),
          format: policy.format,
          state: t(stateKey, policy.enabled ? "Enabled" : "Disabled"),
        })));
      var remove = C.element("button", "ss-rpt-button-small", t("sheet-rpt-sched-remove", "Remove policy"));
      remove.type = "button";
      remove.setAttribute("data-i18n", "sheet-rpt-sched-remove");
      remove.addEventListener("click", function () {
        removePolicy(policy.id);
        ctx.setStatus(t("sheet-rpt-sched-removed", "Policy removed from the local preview."), false);
        ctx.rerender();
      });
      item.appendChild(remove);
      list.appendChild(item);
    });
    container.appendChild(list);
  }

  window.SheetReportingSchedule = {
    render: render,
    FORMATS: FORMATS,
    FREQUENCIES: FREQUENCIES,
    validatePolicy: validatePolicy,
    nextRun: nextRun,
    addPolicy: addPolicy,
    removePolicy: removePolicy,
  };
})();
