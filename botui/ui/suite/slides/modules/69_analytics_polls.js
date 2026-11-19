"use strict";
/* Slides polls (#2489) — layer 4, DOM-free local poll model. Draft/live/closed
 * transitions, simulated responses, aggregate results and CSV export. The
 * "live" transport is local state only, never a real network channel. */
(function (window) {
  var A = window.GBSlidesAnalytics = window.GBSlidesAnalytics || {};
  var P = A.polls = A.polls || {};

  P.TYPES = ["choice", "rating"];
  P.STATUSES = ["draft", "live", "closed"];

  function optionsOf(input) {
    if (Array.isArray(input.options)) {
      return input.options.map(function (value) { return String(value).trim(); })
        .filter(function (value) { return !!value; });
    }
    if (input.type === "rating") return ["1", "2", "3", "4", "5"];
    return String(input.options || "").split(",").map(function (value) {
      return value.trim();
    }).filter(function (value) { return !!value; });
  }

  P.create = function (state, input) {
    var values = input || {};
    var question = String(values.question || "").trim();
    var type = values.type === "rating" ? "rating" : "choice";
    var options = optionsOf({ options: values.options, type: type });
    if (!question || options.length < 2) return null;
    var poll = {
      id: A.uid("poll"),
      question: question,
      type: type,
      options: options,
      status: "draft",
      createdAt: A.num(values.createdAt, Date.now()),
      launchedAt: 0,
      closedAt: 0,
      responses: []
    };
    state.polls = Array.isArray(state.polls) ? state.polls : [];
    state.polls.push(poll);
    return poll;
  };

  P.get = function (state, id) {
    var rows = Array.isArray(state.polls) ? state.polls : [];
    for (var i = 0; i < rows.length; i += 1) {
      if (rows[i].id === id) return rows[i];
    }
    return null;
  };

  P.launch = function (state, id, ts) {
    var poll = P.get(state, id);
    if (!poll || poll.status !== "draft") return false;
    poll.status = "live";
    poll.launchedAt = A.num(ts, Date.now());
    return true;
  };

  P.close = function (state, id, ts) {
    var poll = P.get(state, id);
    if (!poll || poll.status !== "live") return false;
    poll.status = "closed";
    poll.closedAt = A.num(ts, Date.now());
    return true;
  };

  P.respond = function (state, id, input) {
    var poll = P.get(state, id);
    var values = input || {};
    var value = String(values.value || "");
    if (!poll || poll.status !== "live" || poll.options.indexOf(value) < 0) return false;
    poll.responses = Array.isArray(poll.responses) ? poll.responses : [];
    poll.responses.push({
      id: A.uid("resp"),
      viewerId: String(values.viewerId || ""),
      value: value,
      ts: A.num(values.ts, Date.now())
    });
    return true;
  };

  P.results = function (state, id) {
    var poll = P.get(state, id);
    if (!poll) return null;
    var total = Array.isArray(poll.responses) ? poll.responses.length : 0;
    var rows = poll.options.map(function (option) {
      var count = 0;
      poll.responses.forEach(function (response) {
        if (response.value === option) count += 1;
      });
      return {
        label: option,
        count: count,
        percent: total ? Math.round((count / total) * 10000) / 100 : 0
      };
    });
    var firstTs = total ? Math.min.apply(null, poll.responses.map(function (response) {
      return A.num(response.ts, 0);
    })) : 0;
    var latencyMs = total && poll.launchedAt ? Math.max(0, firstTs - poll.launchedAt) : 0;
    return {
      id: poll.id,
      question: poll.question,
      type: poll.type,
      status: poll.status,
      options: rows,
      total: total,
      latencyMs: latencyMs,
      withinBudget: total > 0 && latencyMs <= A.POLL_RESPONSE_BUDGET_MS,
      renderMs: 0,
      renderWithinBudget: true
    };
  };

  P.byStatus = function (state, status) {
    var rows = Array.isArray(state.polls) ? state.polls : [];
    return rows.filter(function (poll) { return poll.status === status; });
  };

  P.latest = function (state) {
    var rows = Array.isArray(state.polls) ? state.polls : [];
    return rows.length ? rows[rows.length - 1] : null;
  };

  function cell(value) {
    var text = value === null || value === undefined ? "" : String(value);
    return "\"" + text.split("\"").join("\"\"") + "\"";
  }

  function line(values) {
    return values.map(cell).join(",") + "\n";
  }

  P.resultsCsv = function (state, id) {
    var result = P.results(state, id);
    if (!result) return "";
    var out = line(["poll_id", "question", "option", "count", "percent"]);
    result.options.forEach(function (option) {
      out += line([result.id, result.question, option.label, option.count, option.percent]);
    });
    return out;
  };

  P.responsesCsv = function (state, id, anonymise) {
    var poll = P.get(state, id);
    if (!poll) return "";
    var strip = anonymise === undefined ? !!(state.options && state.options.anonymise) : !!anonymise;
    var out = strip
      ? line(["poll_id", "pseudonym", "value", "ts"])
      : line(["poll_id", "viewer_id", "value", "ts"]);
    (poll.responses || []).forEach(function (response) {
      out += line([poll.id, strip ? A.pseudonym(response.viewerId) : response.viewerId,
        response.value, response.ts]);
    });
    return out;
  };
})(typeof window !== "undefined" ? window : globalThis);
