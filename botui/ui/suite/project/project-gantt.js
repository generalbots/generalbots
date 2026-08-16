"use strict";

/* =============================================================================
   PROJECT APP — Gantt renderer (zoom-aware, date-positioned).
   Loaded before project.js; project.js calls window.renderGantt(tasks, zoom).
   All functions are global (classic scripts, not ES modules) so the injected
   HTML's inline onclick handlers can reach them.
   ============================================================================= */

// Pixels per day at each zoom level.
var GANTT_PX_PER_DAY = { day: 40, week: 18, month: 6, quarter: 2 };
// Fixed row height shared by the table and the chart so bars line up.
var GANTT_ROW_HEIGHT = 36;

function ganttDayNum(dateStr) {
  if (!dateStr) return NaN;
  return Math.round(Date.parse(dateStr) / 86400000);
}

function ganttDateFromDay(dayNum) {
  return new Date(dayNum * 86400000).toISOString().slice(0, 10);
}

function ganttTaskStart(task) {
  return task.start_date || task.startDate;
}
function ganttTaskEnd(task) {
  return task.end_date || task.endDate;
}

/* Compute the inclusive [start, end] date range over all tasks, padded by one
   day so the first/last bar never touches the edge. */
function ganttDateRange(tasks) {
  var min = Infinity;
  var max = -Infinity;
  tasks.forEach(function (t) {
    var s = ganttDayNum(ganttTaskStart(t));
    var e = ganttDayNum(ganttTaskEnd(t));
    if (isNaN(s) || isNaN(e)) return;
    if (s < min) min = s;
    if (e > max) max = e;
  });
  if (min === Infinity || max === -Infinity) {
    min = ganttDayNum(new Date().toISOString().slice(0, 10));
    max = min + 30;
  }
  return { start: min - 1, end: max + 1, days: (max + 1) - (min - 1) + 1 };
}

/* Timeline header: day/week/month/quarter columns anchored to the range start. */
function ganttRenderHeader(range, pxPerDay, zoom) {
  var html = "";
  var cursor = range.start;
  var end = range.end;

  while (cursor <= end) {
    var d = new Date(cursor * 86400000);
    var label, widthDays;

    if (zoom === "quarter") {
      var qMonth = Math.floor(d.getUTCMonth() / 3) * 3;
      var qEnd = Date.UTC(d.getUTCFullYear(), qMonth + 3, 1) / 86400000;
      widthDays = qEnd - cursor;
      label = "Q" + (Math.floor(d.getUTCMonth() / 3) + 1) + " " + d.getUTCFullYear();
    } else if (zoom === "month") {
      var mEnd = Date.UTC(d.getUTCFullYear(), d.getUTCMonth() + 1, 1) / 86400000;
      widthDays = mEnd - cursor;
      label = d.toLocaleDateString("en-US", { month: "short", year: "numeric", timeZone: "UTC" });
    } else if (zoom === "week") {
      widthDays = 7;
      label = "Wk " + ganttDateFromDay(cursor);
    } else {
      widthDays = 1;
      label = d.toLocaleDateString("en-US", { weekday: "short", timeZone: "UTC" }) + " " + d.getUTCDate();
    }

    var remaining = end - cursor + 1;
    if (widthDays > remaining) widthDays = remaining;

    html +=
      '<div class="timeline-day" style="width:' + (widthDays * pxPerDay) + 'px">' +
      '<div class="tl-label">' + label + "</div></div>";

    cursor += widthDays;
  }
  return html;
}

/* One task row: an absolutely-positioned bar (+ progress fill) or a milestone
   diamond. */
function ganttRenderRow(task, range, pxPerDay, totalWidth) {
  var s = ganttDayNum(ganttTaskStart(task));
  var e = ganttDayNum(ganttTaskEnd(task));
  if (isNaN(s)) s = range.start;
  if (isNaN(e)) e = s;
  var left = (s - range.start) * pxPerDay;
  var durDays = Math.max(1, e - s + 1);
  var width = durDays * pxPerDay;
  var pct = task.percent_complete || task.percentComplete || 0;
  var isMilestone = task.is_milestone || (task.task_type && String(task.task_type).toLowerCase() === "milestone");
  var isCritical = task.is_critical;
  var isSummary = task.is_summary;

  if (isMilestone) {
    return (
      '<div class="gantt-row" style="height:' + GANTT_ROW_HEIGHT + 'px;position:relative;width:' + totalWidth + 'px">' +
      '<div class="gantt-milestone ' + (isCritical ? "critical" : "") + '" style="left:' + left + 'px" title="' +
      (task.name || "").replace(/"/g, "&quot;") + '"></div></div>'
    );
  }

  var cls = "gantt-bar";
  if (isCritical) cls += " critical";
  if (isSummary) cls += " summary";

  return (
    '<div class="gantt-row" style="height:' + GANTT_ROW_HEIGHT + 'px;position:relative;width:' + totalWidth + 'px">' +
    '<div class="' + cls + '" style="left:' + left + 'px;width:' + width + 'px" title="' +
    (task.name || "").replace(/"/g, "&quot;") + ' (' + pct + '%)">' +
    '<span class="gantt-bar-label">' + (task.name || "Task") + "</span>" +
    '<div class="gantt-bar-fill" style="width:' + pct + '%"></div>' +
    "</div></div>"
  );
}

/* SVG arrows from each dependency's predecessor end to the successor start. */
function ganttRenderArrows(tasks, range, pxPerDay, rowIndexById) {
  var links = [];
  tasks.forEach(function (succ) {
    var deps = succ.dependencies || [];
    deps.forEach(function (dep) {
      var pred = tasks.find(function (t) { return t.id === dep.predecessor_id; });
      if (!pred) return;
      links.push({ pred: pred, succ: succ, type: dep.dependency_type || "finish_to_start", lag: dep.lag_days || 0 });
    });
  });

  if (!links.length) return "";

  var totalWidth = range.days * pxPerDay;
  var totalHeight = tasks.length * GANTT_ROW_HEIGHT;

  var paths = "";
  links.forEach(function (link) {
    var predIdx = rowIndexById[link.pred.id];
    var succIdx = rowIndexById[link.succ.id];
    if (predIdx === undefined || succIdx === undefined) return;

    var predEnd = ganttDayNum(ganttTaskEnd(link.pred));
    var succStart = ganttDayNum(ganttTaskStart(link.succ));
    if (isNaN(predEnd)) predEnd = range.start;
    if (isNaN(succStart)) succStart = range.start;

    var x1 = (predEnd - range.start + 1) * pxPerDay;
    var y1 = predIdx * GANTT_ROW_HEIGHT + GANTT_ROW_HEIGHT / 2;
    var x2 = (succStart - range.start) * pxPerDay;
    var y2 = succIdx * GANTT_ROW_HEIGHT + GANTT_ROW_HEIGHT / 2;

    var dx = x2 - x1;
    var bend = dx >= 0 ? Math.max(24, dx / 2) : -Math.max(24, -dx / 2);
    paths +=
      '<path d="M ' + x1 + " " + y1 + " C " + (x1 + bend) + " " + y1 + ", " +
      (x2 - bend) + " " + y2 + ", " + x2 + " " + y2 + '" marker-end="url(#ganttArrow)"/>';
  });

  return (
    '<svg class="gantt-arrows" width="' + totalWidth + '" height="' + totalHeight + '" ' +
    'viewBox="0 0 ' + totalWidth + " " + totalHeight + '">' +
    '<defs><marker id="ganttArrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">' +
    '<path d="M 0 0 L 10 5 L 0 10 z" fill="currentColor"/></marker></defs>' + paths + "</svg>"
  );
}

/* Master entry point: render header + bars + arrows for the current zoom. */
function renderGantt(tasks, zoom) {
  var headerEl = document.getElementById("gantt-timeline-header");
  var bodyEl = document.getElementById("gantt-chart-body");
  if (!headerEl || !bodyEl) return;

  zoom = zoom || "week";
  tasks = tasks || [];
  var pxPerDay = GANTT_PX_PER_DAY[zoom] || GANTT_PX_PER_DAY.week;
  var range = ganttDateRange(tasks);
  var totalWidth = range.days * pxPerDay;

  headerEl.innerHTML = ganttRenderHeader(range, pxPerDay, zoom);
  headerEl.style.width = totalWidth + "px";

  var rowIndexById = {};
  tasks.forEach(function (t, i) { rowIndexById[t.id] = i; });

  var rowsHtml = "";
  tasks.forEach(function (t) {
    rowsHtml += ganttRenderRow(t, range, pxPerDay, totalWidth);
  });

  if (!tasks.length) {
    bodyEl.innerHTML = '<div class="empty-state-inline"><p>No tasks to display</p></div>';
    bodyEl.style.width = "";
    return;
  }

  bodyEl.innerHTML = rowsHtml + ganttRenderArrows(tasks, range, pxPerDay, rowIndexById);
  bodyEl.style.width = totalWidth + "px";

  syncGanttScroll();
}

/* Keep the left table and right chart vertically aligned. */
function syncGanttScroll() {
  var table = document.querySelector(".gantt-table");
  var chart = document.querySelector(".gantt-chart");
  if (!table || !chart) return;

  // Guard against feedback loops by flagging the active scroller.
  var syncing = false;
  table.addEventListener("scroll", function () {
    if (syncing) return;
    syncing = true;
    chart.scrollTop = table.scrollTop;
    window.setTimeout(function () { syncing = false; }, 0);
  });
  chart.addEventListener("scroll", function () {
    if (syncing) return;
    syncing = true;
    table.scrollTop = chart.scrollTop;
    window.setTimeout(function () { syncing = false; }, 0);
  });
}

window.renderGantt = renderGantt;
window.GANTT_ROW_HEIGHT = GANTT_ROW_HEIGHT;
