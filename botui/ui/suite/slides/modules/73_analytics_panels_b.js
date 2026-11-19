"use strict";
/* Slides analytics (#2489) — layer 7b, polls, follow-along and export panels.
 * Poll "live" results and device re-sync are simulated in local state; the
 * panel never opens a socket or calls a server endpoint. */
(function (window) {
  var A = window.GBSlidesAnalytics = window.GBSlidesAnalytics || {};
  var panels = A.panels = A.panels || {};

  function statusLabel(status) {
    return A.tr("slides-analytics-polls-status-" + status, status);
  }

  panels.polls = function (api, state) {
    api.body.appendChild(A.localNote());
    var form = A.section("slides-analytics-polls-create", "Create a poll");
    var question = A.field("slides-analytics-polls-question", "Poll question", "",
      "slides-analytics-polls-question-placeholder", "Ask the audience…");
    var type = A.select("slides-analytics-polls-type", "Poll type", [
      ["choice", "slides-analytics-polls-type-choice", "Multiple choice"],
      ["rating", "slides-analytics-polls-type-rating", "Rating"]
    ], "choice");
    var options = A.field("slides-analytics-polls-options", "Options (one per line)", "",
      "slides-analytics-polls-options-placeholder", "Option A, Option B, Option C");
    form.appendChild(question);
    form.appendChild(type);
    form.appendChild(options);
    form.appendChild(A.button("slides-analytics-polls-create-action", "Create draft", function () {
      var created = A.polls.create(state, {
        question: question.control.value,
        type: type.control.value,
        options: options.control.value
      });
      if (!created) {
        api.setStatus("slides-analytics-polls-invalid", "Add at least two options.");
        return;
      }
      A.save(state);
      api.setStatus("slides-analytics-polls-created", "Poll draft created locally");
      A.redraw(api, panels.polls);
    }, "primary"));
    api.body.appendChild(form);

    var list = A.section("slides-analytics-section-polls", "Polls");
    var polls = Array.isArray(state.polls) ? state.polls : [];
    if (!polls.length) {
      list.appendChild(A.el("p", "sa-hint", "slides-analytics-polls-empty", "No polls yet."));
    }
    polls.forEach(function (poll) {
      list.appendChild(pollBlock(api, state, poll));
    });
    api.body.appendChild(list);
  };

  function pollBlock(api, state, poll) {
    var block = document.createElement("div");
    block.className = "sa-poll";
    block.appendChild(A.text("sa-poll-question", poll.question));
    block.appendChild(A.text("sa-list-cell", statusLabel(poll.status)));
    if (poll.status === "draft") {
      block.appendChild(A.button("slides-analytics-polls-launch", "Launch live", function () {
        if (A.polls.launch(state, poll.id, Date.now())) {
          A.save(state);
          api.setStatus("slides-analytics-polls-launched", "Poll is live (simulated)");
          A.redraw(api, panels.polls);
        }
      }, "primary"));
    }
    if (poll.status === "live") {
      block.appendChild(A.button("slides-analytics-polls-respond", "Simulate 5 responses", function () {
        for (var k = 0; k < 5; k += 1) {
          A.polls.respond(state, poll.id, {
            viewerId: "v-" + (k + 1),
            value: poll.options[k % poll.options.length],
            ts: A.num(poll.launchedAt, Date.now()) + 800 + k * 100
          });
        }
        A.save(state);
        api.setStatus("slides-analytics-polls-responded", "Responses simulated locally");
        A.redraw(api, panels.polls);
      }));
      block.appendChild(A.button("slides-analytics-polls-close", "Close poll", function () {
        if (A.polls.close(state, poll.id, Date.now())) {
          A.save(state);
          api.setStatus("slides-analytics-polls-closed", "Poll closed");
          A.redraw(api, panels.polls);
        }
      }));
    }
    var result = A.polls.results(state, poll.id);
    if (result) {
      block.appendChild(A.metric("slides-analytics-polls-total", "Responses", result.total));
      block.appendChild(A.metric("slides-analytics-polls-latency", "First response latency",
        result.latencyMs + " ms"));
      result.options.forEach(function (option) {
        block.appendChild(A.listRow([
          A.text("sa-list-cell", option.label),
          A.text("sa-list-cell", String(option.count)),
          A.text("sa-list-cell", option.percent + "%")
        ]));
      });
      var box = A.csvBox();
      block.appendChild(box);
      block.appendChild(A.button("slides-analytics-polls-export", "Export responses CSV", function () {
        box.value = A.polls.responsesCsv(state, poll.id, state.options.anonymise);
        api.setStatus("slides-analytics-status-saved", "Saved locally");
      }));
      block.appendChild(A.button("slides-analytics-polls-export-results", "Export results CSV", function () {
        box.value = A.polls.resultsCsv(state, poll.id);
        api.setStatus("slides-analytics-status-saved", "Saved locally");
      }));
    }
    return block;
  }

  panels.follow = function (api, state) {
    api.body.appendChild(A.localNote());
    var summary = A.follow.summary(state);
    var status = A.section("slides-analytics-follow-title", "Follow-along");
    status.appendChild(A.metric("slides-analytics-follow-presenter", "Presenter slide",
      summary.presenterSlide + 1));
    status.appendChild(A.metric("slides-analytics-follow-connected", "Connected", summary.connected));
    status.appendChild(A.metric("slides-analytics-follow-offline", "Offline", summary.offline));
    status.appendChild(A.metric("slides-analytics-follow-budget", "Re-sync budget",
      A.tr("slides-analytics-follow-budget", "Re-sync budget: { $n } ms, at most one slide behind",
        { n: A.RESYNC_BUDGET_MS })));
    api.body.appendChild(status);

    var devices = A.section("slides-analytics-follow-devices", "Audience devices");
    var rows = A.follow.devices(state);
    if (!rows.length) {
      devices.appendChild(A.el("p", "sa-hint", "slides-analytics-follow-empty",
        "No simulated devices yet."));
    }
    rows.forEach(function (device) {
      devices.appendChild(A.listRow([
        A.text("sa-list-cell", device.label),
        A.text("sa-list-cell", device.connected
          ? A.tr("slides-analytics-follow-connected", "Connected")
          : A.tr("slides-analytics-follow-offline", "Offline")),
        A.text("sa-list-cell", A.tr("slides-analytics-follow-slide", "Slide { $n }",
          { n: device.slide + 1 })),
        A.text("sa-list-cell", A.tr("slides-analytics-report-ms", "{ $n } ms",
          { n: device.resyncLatencyMs }))
      ]));
    });
    api.body.appendChild(devices);

    var controls = A.section("slides-analytics-tools", "Audience tools");
    controls.appendChild(A.button("slides-analytics-follow-add", "Add simulated device", function () {
      A.follow.addDevice(state, { id: A.uid("device"), ts: Date.now() });
      A.save(state);
      A.redraw(api, panels.follow);
    }));
    controls.appendChild(A.button("slides-analytics-follow-advance", "Advance presenter slide", function () {
      A.follow.advance(state, Date.now());
      A.save(state);
      A.redraw(api, panels.follow);
    }));
    controls.appendChild(A.button("slides-analytics-follow-disconnect", "Simulate 30 s disconnect", function () {
      var target = A.follow.devices(state)[0];
      if (target && A.follow.simulateThirtySecondDrop(state, target.id, Date.now())) {
        api.setStatus("slides-analytics-follow-re-synced", "Re-synced to slide { $n } in { $ms } ms",
          { n: A.num(state.follow.presenterSlide, 0) + 1, ms: target.resyncLatencyMs });
      }
      A.save(state);
      A.redraw(api, panels.follow);
    }));
    controls.appendChild(A.button("slides-analytics-follow-reconnect", "Reconnect device", function () {
      var target = A.follow.devices(state)[0];
      if (target && A.follow.reconnect(state, target.id, Date.now())) {
        api.setStatus("slides-analytics-follow-re-synced", "Re-synced to slide { $n } in { $ms } ms",
          { n: A.num(state.follow.presenterSlide, 0) + 1, ms: target.resyncLatencyMs });
      }
      A.save(state);
      A.redraw(api, panels.follow);
    }));
    api.body.appendChild(controls);
  };

  panels.export = function (api, state) {
    api.body.appendChild(A.localNote());
    var section = A.section("slides-analytics-export-title", "Export and retention");
    var toggle = A.el("label", "sa-check", "slides-analytics-export-anonymise",
      "Strip personal data (anonymise)");
    var checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.checked = !!(state.options && state.options.anonymise);
    checkbox.addEventListener("change", function () {
      state.options.anonymise = !!checkbox.checked;
      A.save(state);
      api.setStatus("slides-analytics-status-saved", "Saved locally");
    });
    toggle.appendChild(checkbox);
    toggle.control = checkbox;
    section.appendChild(toggle);
    var retention = A.field("slides-analytics-export-retention", "Retention (days)",
      A.num(state.options && state.options.retentionDays, 90));
    retention.control.type = "number";
    section.appendChild(retention);
    section.appendChild(A.button("slides-analytics-export-apply", "Apply retention", function () {
      state.options.retentionDays = A.num(retention.control.value, 90);
      var removed = A.prune(state, Date.now());
      A.save(state);
      api.setStatus("slides-analytics-export-applied", "Old events removed", { n: removed });
    }));
    var box = A.csvBox();
    section.appendChild(box);
    section.appendChild(A.button("slides-analytics-export-viewers", "Export viewers CSV", function () {
      box.value = A.csv.viewers(state, state.options.anonymise);
      api.setStatus("slides-analytics-status-saved", "Saved locally");
    }));
    section.appendChild(A.button("slides-analytics-report-export", "Export summary CSV", function () {
      box.value = A.csv.summary(A.compute(state));
      api.setStatus("slides-analytics-status-saved", "Saved locally");
    }));
    section.appendChild(A.el("p", "sa-hint", "slides-analytics-export-note",
      "Anonymised exports use pseudonyms and bucketed device/region values."));
    api.body.appendChild(section);
  };
})(window);
