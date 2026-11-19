"use strict";
/* Sheets ingest (#2494) — batch file list with independent per-file statuses.
 * One failed file never aborts the others. All transitions are simulated. */

(function () {
  var C = window.SheetIngestCore;
  var STATUSES = ["queued", "parsing", "review", "ready", "failed"];

  function t(key, fallback, params) { return C.t(key, fallback, params); }

  function statusLabel(status) {
    var map = {
      queued: ["sheet-ingest-status-queued", "Queued"],
      parsing: ["sheet-ingest-status-parsing", "Parsing"],
      review: ["sheet-ingest-status-review", "Review"],
      ready: ["sheet-ingest-status-ready", "Ready"],
      failed: ["sheet-ingest-status-failed", "Failed"],
    };
    var entry = map[status] || map.queued;
    return t(entry[0], entry[1]);
  }

  function summary() {
    var current = C.state();
    var out = { queued: 0, parsing: 0, review: 0, ready: 0, failed: 0, total: current.files.length };
    current.files.forEach(function (file) {
      if (out[file.status] != null) out[file.status] += 1;
    });
    return out;
  }

  function startBatch(ctx) {
    C.mutate(function (target) {
      target.files.forEach(function (file) {
        if (file.status !== "queued") return;
        if (/corrupt|bad/i.test(file.name)) {
          file.status = "failed";
          file.error = t("sheet-ingest-batch-error", "{ $name }: parse failed (simulated)",
            { name: file.name });
        } else {
          file.status = "parsing";
        }
      });
    });
    if (ctx) ctx.setStatus(t("sheet-ingest-batch-started",
      "Batch started; each file advances on its own."), false);
  }

  function completeBatch(ctx) {
    C.mutate(function (target) {
      target.files.forEach(function (file) {
        if (file.status !== "parsing") return;
        file.status = "review";
        window.SheetIngestSim.ensureParse(target, file.id);
      });
      var first = target.files.filter(function (file) { return file.status === "review"; })[0];
      if (first) target.activeFileId = first.id;
    });
    if (ctx) ctx.setStatus(t("sheet-ingest-batch-completed",
      "Parse finished for all readable files."), false);
  }

  function markReady(fileId, ctx) {
    C.mutate(function (target) {
      target.files.forEach(function (file) {
        if (file.id === fileId && file.status === "review") file.status = "ready";
      });
    });
    if (ctx) ctx.setStatus(t("sheet-ingest-batch-readied",
      "File marked ready in the local preview."), false);
  }

  function activate(fileId, ctx) {
    var file = C.state().files.filter(function (item) { return item.id === fileId; })[0];
    if (!file || file.status === "failed") return;
    C.mutate(function (target) {
      target.activeFileId = fileId;
      window.SheetIngestSim.ensureParse(target, fileId);
    });
    if (ctx) {
      ctx.clearSelection();
      ctx.setStatus(t("sheet-ingest-batch-active", "Active file switched to { $name }.",
        { name: file.name }), false);
      ctx.rerender();
    }
  }

  function render(body, ctx) {
    var current = ctx.state();
    var section = C.element("section", "ss-ingest-section");
    section.appendChild(C.element("h3", "ss-ingest-section-title",
      t("sheet-ingest-batch-title", "Batch files")));
    section.appendChild(C.element("p", "ss-ingest-note",
      t("sheet-ingest-batch-note", "Per-file status; a failed file never aborts the batch.")));

    var controls = C.element("div", "ss-ingest-controls");
    var start = C.element("button", "ss-ingest-btn", t("sheet-ingest-batch-start", "Start batch"));
    start.type = "button";
    start.addEventListener("click", function () { startBatch(ctx); ctx.rerender(); });
    var finish = C.element("button", "ss-ingest-btn",
      t("sheet-ingest-batch-complete", "Finish parse"));
    finish.type = "button";
    finish.addEventListener("click", function () { completeBatch(ctx); ctx.rerender(); });
    controls.appendChild(start);
    controls.appendChild(finish);
    section.appendChild(controls);

    if (!current.files.length) {
      section.appendChild(C.element("p", "ss-ingest-muted",
        t("sheet-ingest-batch-empty", "No files in the batch.")));
    }
    current.files.forEach(function (file) {
      var row = C.element("div", "ss-ingest-batch-row");
      row.dataset.fileId = file.id;
      row.dataset.status = file.status;
      row.appendChild(C.element("span", "ss-ingest-file-name", file.name));
      var badge = C.element("span", "ss-ingest-status-badge ss-ingest-status-" + file.status,
        statusLabel(file.status));
      row.appendChild(badge);
      if (file.error) row.appendChild(C.element("span", "ss-ingest-error", file.error));
      if (file.status === "review" || file.status === "ready") {
        var open = C.element("button", "ss-ingest-btn ss-ingest-btn-small",
          t("sheet-ingest-batch-activate", "Open in review"));
        open.type = "button";
        open.addEventListener("click", function () { activate(file.id, ctx); });
        row.appendChild(open);
      }
      if (file.status === "review") {
        var ready = C.element("button", "ss-ingest-btn ss-ingest-btn-small",
          t("sheet-ingest-batch-ready", "Mark ready"));
        ready.type = "button";
        ready.addEventListener("click", function () { markReady(file.id, ctx); ctx.rerender(); });
        row.appendChild(ready);
      }
      section.appendChild(row);
    });

    var counts = summary();
    section.appendChild(C.element("p", "ss-ingest-summary",
      t("sheet-ingest-batch-summary",
        "Queued { $queued }, parsing { $parsing }, review { $review }, ready { $ready }, failed { $failed }.",
        counts)));
    body.appendChild(section);
  }

  window.SheetIngestBatch = {
    STATUSES: STATUSES,
    summary: summary,
    startBatch: startBatch,
    completeBatch: completeBatch,
    markReady: markReady,
    activate: activate,
    statusLabel: statusLabel,
    render: render,
  };
})();
