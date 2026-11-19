"use strict";
/* Sheets data connectivity (#2492) - scheduled refresh preview, cached
 * snapshots, offline/stale marker and refresh audit. window.SheetConnectRefresh.
 * No cron is registered and no notification is delivered: both are simulated
 * first-class state in the local workbook preview. */

(function () {
  var C = window.SheetConnectCore;
  var F = window.SheetConnectFeed;
  var U = window.SheetConnectUi;
  var t = C.t;

  var FREQUENCIES = [
    { value: "daily", key: "sheet-connect-freq-daily", fallback: "Daily" },
    { value: "weekly", key: "sheet-connect-freq-weekly", fallback: "Weekly" },
    { value: "monthly", key: "sheet-connect-freq-monthly", fallback: "Monthly" },
  ];

  function shortTime(iso) {
    return iso ? String(iso).slice(0, 16).replace("T", " ") : "";
  }

  function statusText(status) {
    if (status === "success") return t("sheet-connect-status-success", "Success");
    if (status === "failed") return t("sheet-connect-status-failed", "Failed");
    return t("sheet-connect-status-never", "Never");
  }

  function renderStatus(container, target) {
    var schedule = target.schedule;
    var status = U.el("div", "ss-connect-refresh-status");
    status.appendChild(U.keyValue("sheet-connect-last-refresh", "Last refresh",
      shortTime(schedule.lastRefreshAt) || t("sheet-connect-status-never", "Never")));
    var next = schedule.lastRefreshAt && F.nextRefresh(schedule.lastRefreshAt, schedule);
    status.appendChild(U.keyValue("sheet-connect-next-refresh", "Next refresh",
      next ? shortTime(next) + " " + schedule.timezone : t("sheet-connect-next-pending", "After the first refresh")));
    status.appendChild(U.keyValue("sheet-connect-refresh-state", "Refresh status", statusText(schedule.lastStatus)));
    container.appendChild(status);

    if (schedule.lastStatus === "failed") {
      var notice = U.el("div", "ss-connect-failure");
      notice.setAttribute("role", "status");
      notice.setAttribute("aria-live", "polite");
      notice.appendChild(U.el("strong", "", t("sheet-connect-notification-title", "Failure notification (simulated)")));
      notice.appendChild(U.el("span", "", t("sheet-connect-notification", "Refresh failed: { $error }",
        { error: schedule.lastError || t("sheet-connect-refresh-sim-error", "Simulated feed error (local).") })));
      container.appendChild(notice);
    }
  }

  function renderScheduleForm(container, ctx, target) {
    container.appendChild(U.title("sheet-connect-refresh-title", "Scheduled refresh"));
    var form = U.el("div", "ss-connect-form");
    var frequency = U.select("sheet-connect-field-frequency", "Frequency", FREQUENCIES, target.schedule.frequency);
    frequency.input.addEventListener("change", function () {
      C.mutate(function (state) { state.schedule.frequency = frequency.input.value; });
      ctx.setStatus(t("sheet-connect-schedule-saved", "Schedule saved locally (preview only)."));
      ctx.rerender();
    });
    form.appendChild(frequency);
    var hour = U.field("sheet-connect-field-hour", "Hour (0-23)", "number", target.schedule.hour);
    hour.input.addEventListener("change", function () {
      var value = Math.min(23, Math.max(0, Math.floor(Number(hour.input.value) || 0)));
      C.mutate(function (state) { state.schedule.hour = value; });
      ctx.setStatus(t("sheet-connect-schedule-saved", "Schedule saved locally (preview only)."));
    });
    form.appendChild(hour);
    var timezone = U.field("sheet-connect-field-timezone", "Timezone", "text", target.schedule.timezone);
    timezone.input.addEventListener("change", function () {
      C.mutate(function (state) { state.schedule.timezone = timezone.input.value.slice(0, 60); });
      ctx.setStatus(t("sheet-connect-schedule-saved", "Schedule saved locally (preview only)."));
    });
    form.appendChild(timezone);
    form.appendChild(U.checkbox("sheet-connect-field-notify", "Notify on failure",
      target.schedule.notifyOnFailure, function () {
        C.mutate(function (state) { state.schedule.notifyOnFailure = !state.schedule.notifyOnFailure; });
      }));
    form.appendChild(U.checkbox("sheet-connect-field-simulate-failure", "Simulate failure",
      target.simulateFailure, function () {
        C.mutate(function (state) { state.simulateFailure = !state.simulateFailure; });
        ctx.setStatus(t("sheet-connect-sim-toggle", "Failure simulation updated."));
      }));
    container.appendChild(form);
  }

  function renderSnapshot(container, ctx, target) {
    container.appendChild(U.title("sheet-connect-snapshot-title", "Cached snapshot"));
    var offline = U.checkbox("sheet-connect-offline", "Simulate offline", target.offline, function () {
      C.mutate(function (state) { state.offline = !state.offline; });
      ctx.setStatus(t("sheet-connect-offline-note",
        "Offline mode shows the last cached snapshot marked STALE."));
      ctx.rerender();
    });
    container.appendChild(offline);
    if (!target.snapshot) {
      container.appendChild(U.note("sheet-connect-no-snapshot", "No cached snapshot yet. Run a refresh first."));
      return;
    }
    var head = U.el("div", "ss-connect-snapshot-head");
    head.appendChild(U.el("span", "", shortTime(target.snapshot.createdAt)));
    if (target.offline) {
      var stale = U.el("span", "ss-connect-badge ss-connect-stale", t("sheet-connect-stale", "STALE"));
      stale.setAttribute("data-i18n", "sheet-connect-stale");
      head.appendChild(stale);
    }
    head.appendChild(U.el("span", "ss-connect-connector-sub",
      t("sheet-connect-snapshot-rows", "{ $rows } rows", { rows: target.snapshot.rowCount })));
    container.appendChild(head);
    container.appendChild(U.table(target.snapshot.columns, target.snapshot.rows.map(function (row) {
      return row.map(function (cell) { return String(cell); });
    })));
  }

  function renderAudit(container, ctx, target) {
    container.appendChild(U.title("sheet-connect-audit", "Refresh / audit log"));
    if (!target.audit.length) {
      container.appendChild(U.note("sheet-connect-audit-empty", "No refresh entries yet."));
      return;
    }
    var rows = target.audit.slice().reverse().map(function (entry) {
      return [shortTime(entry.time), statusText(entry.status), String(entry.rows), entry.error || "-"];
    });
    container.appendChild(U.table([
      t("sheet-connect-audit-time", "Time"),
      t("sheet-connect-audit-status", "Status"),
      t("sheet-connect-audit-rows", "Rows"),
      t("sheet-connect-audit-error", "Error"),
    ], rows));
  }

  function refresh(ctx) {
    var result = null;
    C.mutate(function (target) { result = F.refreshNow(target); });
    if (!result.ok) {
      var message = result.error
        || t(result.errorKey || "sheet-connect-refresh-sim-error", result.errorFallback || "Simulated feed error (local).");
      ctx.setStatus(t("sheet-connect-refresh-failed", "Refresh failed: { $error }", { error: message }), true);
      return result;
    }
    ctx.setStatus(t("sheet-connect-refresh-ok", "Snapshot updated: { $rows } rows (simulated).", { rows: result.rows }));
    ctx.rerender();
    return result;
  }

  function render(container, ctx) {
    var target = ctx.state();
    renderScheduleForm(container, ctx, target);
    var actions = U.el("div", "ss-connect-actions");
    actions.appendChild(U.button("sheet-connect-refresh-now", "Refresh now", function () { refresh(ctx); }));
    container.appendChild(actions);
    renderStatus(container, target);
    renderSnapshot(container, ctx, target);
    renderAudit(container, ctx, target);
  }

  window.SheetConnectRefresh = {
    render: render, refresh: refresh, renderAudit: renderAudit,
    statusText: statusText, shortTime: shortTime, FREQUENCIES: FREQUENCIES,
  };
})();
