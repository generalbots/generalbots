/**
 * UAT gap step-level suite (#2031).
 * Run with: node botui/ui/suite/tests/webtest_uat_gaps.js
 *
 * Each module carries a real [UAT:<ID>] label and drives shipped app code
 * (not "app mounts" assertions) through at least two concrete scenario steps.
 */
"use strict";

var harness = require("./webtest-apps/00_harness");

[
  "01_notes_reopen",
  "02_goals_30km",
  "03_goals_12books",
  "04_calendar_45min",
  "05_tasks_complete_delete",
  "06_db_tables_rows_sort",
  "07_db_query_console",
  "08_compliance_score_checks",
  "09_canvas_draw_select_fill",
  "10_canvas_undo_pdf",
  "11_canvas_wireframe_png",
  "12_canvas_flowchart_svg",
  "13_canvas_icon_svg",
  "14_db_sales_by_state",
  "15_pos_two_items_card",
  "16_canvas_fig_roundtrip",
  "17_canvas_share_collab",
  "18_canvas_org_chart_zoom",
  "19_canvas_screenshot_wireframe",
  "20_canvas_mindmap_rebalance",
].forEach(function (name) {
  require("./webtest-uat-gaps/" + name);
});

harness.finish();
