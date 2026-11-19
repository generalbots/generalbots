"use strict";
/* Slides analytics (#2489) — layer 7a, console, per-deck report and Q&A
 * panels. All labels come from catalogues; every action mutates only the
 * deck-scoped local state and reports through a polite status region. */
(function (window) {
  var A = window.GBSlidesAnalytics = window.GBSlidesAnalytics || {};
  var panels = A.panels = A.panels || {};

  A.consoleSnapshot = function (state) {
    var report = A.compute(state);
    var poll = A.polls.latest(state);
    var devices = A.follow.summary(state);
    return {
      elapsed: A.formatTime(Date.now() - A.num(state.console && state.console.startedAt, Date.now())),
      questions: A.qa.count(state),
      pending: A.qa.queue(state).length,
      pollStatus: poll ? poll.status : "none",
      pollId: poll ? poll.id : "",
      devices: devices,
      report: report
    };
  };

  A.csvBox = function () {
    var box = document.createElement("textarea");
    box.className = "sa-csv";
    box.readOnly = true;
    box.setAttribute("data-i18n-aria-label", "slides-analytics-export-title");
    box.setAttribute("aria-label", A.tr("slides-analytics-export-title", "Export and retention"));
    return box;
  };

  A.redraw = function (api, render) {
    api.body.textContent = "";
    render(api.body, A.state);
  };

  function menuButton(api, key, fallback, open) {
    return A.button(key, fallback, function () {
      api.close();
      open();
    }, "menu");
  }

  panels.console = function (api, state) {
    api.body.appendChild(A.localNote());
    var snapshot = A.consoleSnapshot(state);
    var section = A.section("slides-analytics-console-summary", "Console overview");
    section.appendChild(A.metric("slides-analytics-console-elapsed", "Elapsed", snapshot.elapsed));
    section.appendChild(A.metric("slides-analytics-console-questions", "Questions", snapshot.questions));
    section.appendChild(A.metric("slides-analytics-console-poll", "Poll status",
      A.tr("slides-analytics-polls-status-" + snapshot.pollStatus,
        snapshot.pollStatus === "none" ? "No active poll" : snapshot.pollStatus)));
    section.appendChild(A.metric("slides-analytics-report-unique", "Unique viewers",
      snapshot.report.uniqueViewers));
    section.appendChild(A.metric("slides-analytics-report-completion", "Completion rate",
      snapshot.report.completionRate + "%"));
    section.appendChild(A.metric("slides-analytics-follow-devices", "Audience devices",
      snapshot.devices.connected + "/" + snapshot.devices.total));
    api.body.appendChild(section);
    api.body.appendChild(A.el("p", "sa-hint", "slides-analytics-deferred",
      "Server telemetry, live transport and multi-device sync are not available in this local preview."));
    var tools = A.section("slides-analytics-tools", "Audience tools");
    [["slides-analytics-section-report", "Analytics report", "report"],
      ["slides-analytics-section-qa", "Q&A", "qa"],
      ["slides-analytics-section-polls", "Polls", "polls"],
      ["slides-analytics-section-follow", "Follow-along", "follow"],
      ["slides-analytics-section-export", "Export", "export"]].forEach(function (entry) {
      tools.appendChild(menuButton(api, entry[0], entry[1], function () { A.open[entry[2]](); }));
    });
    api.body.appendChild(tools);
  };

  panels.report = function (api, state) {
    api.body.appendChild(A.localNote());
    var report = A.compute(state);
    if (!report.views) {
      api.body.appendChild(A.el("p", "sa-hint", "slides-analytics-report-empty",
        "No events yet — simulate viewers to build a report."));
      api.body.appendChild(A.button("slides-analytics-simulate", "Simulate 20 viewers", function () {
        A.seed(state, 20);
        A.save(state);
        api.setStatus("slides-analytics-status-events", "Simulated events", { n: state.events.length });
        A.redraw(api, panels.report);
      }, "primary"));
      return;
    }
    var stats = A.section("slides-analytics-report-title", "Presentation analytics");
    stats.appendChild(A.metric("slides-analytics-report-views", "Views", report.views));
    stats.appendChild(A.metric("slides-analytics-report-unique", "Unique viewers", report.uniqueViewers));
    stats.appendChild(A.metric("slides-analytics-report-first", "First view", report.firstView));
    stats.appendChild(A.metric("slides-analytics-report-last", "Last view", report.lastView));
    stats.appendChild(A.metric("slides-analytics-report-completion", "Completion rate",
      report.completionRate + "%"));
    stats.appendChild(A.metric("slides-analytics-report-dropoff", "Drop-off slide",
      "#" + (report.dropOffSlide + 1)));
    api.body.appendChild(stats);

    var table = A.section("slides-analytics-section-report", "Analytics report");
    report.perSlide.forEach(function (row) {
      table.appendChild(A.listRow([
        A.text("sa-list-cell", A.tr("slides-analytics-follow-slide", "Slide { $n }", { n: row.slide + 1 })),
        A.text("sa-list-cell", String(row.entered)),
        A.text("sa-list-cell", A.tr("slides-analytics-report-ms", "{ $n } ms", { n: row.dwellMs })),
        A.text("sa-list-cell", A.tr("slides-analytics-report-ms", "{ $n } ms", { n: row.avgDwellMs })),
        A.text("sa-list-cell", String(row.exits))
      ]));
    });
    api.body.appendChild(table);

    var buckets = A.section("slides-analytics-report-devices", "Device and region buckets");
    Object.keys(report.deviceBuckets).forEach(function (name) {
      if (report.deviceBuckets[name]) {
        buckets.appendChild(A.metric(name, name, report.deviceBuckets[name]));
      }
    });
    Object.keys(report.geoBuckets).forEach(function (name) {
      if (report.geoBuckets[name]) {
        buckets.appendChild(A.metric(name, name, report.geoBuckets[name]));
      }
    });
    api.body.appendChild(buckets);

    var box = A.csvBox();
    api.body.appendChild(box);
    api.body.appendChild(A.button("slides-analytics-report-export", "Export summary CSV", function () {
      box.value = A.csv.summary(report);
      api.setStatus("slides-analytics-status-saved", "Saved locally");
    }));
    api.body.appendChild(A.button("slides-analytics-export-viewers", "Export viewers CSV", function () {
      box.value = A.csv.viewers(state, state.options.anonymise);
      api.setStatus("slides-analytics-export-note",
        "Anonymised exports use pseudonyms and bucketed device/region values.");
    }));
  };

  panels.qa = function (api, state) {
    api.body.appendChild(A.localNote());
    var form = A.section("slides-analytics-qa-submit-title", "Submit a question");
    var name = A.field("slides-analytics-qa-name", "Display name", "");
    var email = A.field("slides-analytics-qa-email", "Email", "");
    var viewer = A.field("slides-analytics-follow-device", "Device", "viewer-local");
    var policy = A.select("slides-analytics-qa-policy", "Attribution policy", [
      ["attributable", "slides-analytics-qa-policy-attributable", "Attributable"],
      ["anonymous", "slides-analytics-qa-policy-anonymous", "Anonymous"]
    ], "attributable");
    var question = A.el("label", "sa-field", "slides-analytics-qa-question", "Question");
    var input = document.createElement("textarea");
    A.labelled(input, "data-i18n-placeholder", "slides-analytics-qa-question-placeholder",
      "Type your question…");
    question.appendChild(input);
    question.control = input;
    form.appendChild(name);
    form.appendChild(email);
    form.appendChild(viewer);
    form.appendChild(policy);
    form.appendChild(question);
    form.appendChild(A.button("slides-analytics-qa-send", "Submit question", function () {
      var created = A.qa.submit(state, {
        text: input.value,
        viewerId: viewer.control.value,
        name: name.control.value,
        email: email.control.value,
        policy: policy.control.value
      });
      if (!created) {
        api.setStatus("slides-analytics-qa-invalid", "Enter a question first.");
        return;
      }
      A.save(state);
      api.setStatus("slides-analytics-qa-sent", "Question submitted locally");
      A.redraw(api, panels.qa);
    }, "primary"));
    api.body.appendChild(form);

    var queue = A.section("slides-analytics-qa-queue", "Moderation queue");
    A.qa.queue(state).forEach(function (item) {
      queue.appendChild(questionRow(api, state, item, true));
    });
    if (!A.qa.queue(state).length) {
      queue.appendChild(A.el("p", "sa-hint", "slides-analytics-qa-empty", "No questions here yet."));
    }
    api.body.appendChild(queue);

    var approved = A.section("slides-analytics-qa-approved", "Approved questions");
    A.qa.approved(state).forEach(function (item) {
      approved.appendChild(questionRow(api, state, item, false));
    });
    api.body.appendChild(approved);

    var archive = A.section("slides-analytics-qa-archive", "Answered archive");
    A.qa.archive(state).forEach(function (item) {
      var row = A.listRow([A.text("sa-list-cell", item.text),
        A.text("sa-list-cell", item.answer),
        A.text("sa-list-cell", A.qa.attribution(item))]);
      archive.appendChild(row);
    });
    api.body.appendChild(archive);
  };

  function questionRow(api, state, item, moderate) {
    var row = A.listRow([A.text("sa-question-text", item.text),
      A.text("sa-list-cell", A.qa.attribution(item)),
      A.text("sa-list-cell", A.tr("slides-analytics-qa-votes", "{ $n } votes", { n: item.votes }))]);
    if (moderate) {
      row.appendChild(A.button("slides-analytics-qa-approve", "Approve", function () {
        A.qa.moderate(state, item.id, "approve");
        A.save(state);
        A.redraw(api, panels.qa);
      }));
      row.appendChild(A.button("slides-analytics-qa-reject", "Reject", function () {
        A.qa.moderate(state, item.id, "reject");
        A.save(state);
        A.redraw(api, panels.qa);
      }));
      row.appendChild(A.button("slides-analytics-qa-answer", "Mark answered", function () {
        A.qa.answer(state, item.id, A.tr("slides-analytics-qa-answered-label", "Answered"));
        A.save(state);
        A.redraw(api, panels.qa);
      }));
    } else {
      row.appendChild(A.button("slides-analytics-qa-upvote", "Upvote", function () {
        if (A.qa.upvote(state, item.id, "viewer-local")) A.save(state);
        A.redraw(api, panels.qa);
      }));
      row.appendChild(A.button("slides-analytics-qa-answer", "Mark answered", function () {
        A.qa.answer(state, item.id, A.tr("slides-analytics-qa-answered-label", "Answered"));
        A.save(state);
        A.redraw(api, panels.qa);
      }));
    }
    return row;
  }
})(window);
