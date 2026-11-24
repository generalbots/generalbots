/**
 * [UAT:TASK-01] "Create a task list for launching the site: content, design, QA, deploy"
 * Steps exercised: (2) checking a task posts the completion and refreshes the list;
 * (3) deleting a task asks for confirmation before issuing a DELETE.
 * Step 1 (four tasks created from chat) is LLM-driven and is not covered here.
 * Exercises the real top-level completeTask/deleteTask functions in
 * tasks/tasks-scoped-decisions.js.
 */
"use strict";

var env = require("./00_env");
var h = env.h;

function makeTasks() {
  var calls = [];
  var prompts = [];
  var toasts = [];
  var triggers = [];
  var doc = h.makeDocument({ readyState: "complete" });

  function fetchStub(url, opts) {
    calls.push({ url: url, opts: opts || {} });
    return Promise.resolve({
      ok: true,
      json: function () { return Promise.resolve({}); },
    });
  }

  var state = { confirmAnswer: false };
  var sandbox = env.baseSandbox(doc, env.makeStore(), {
    fetch: fetchStub,
    TasksState: { selectedTaskId: null, tasks: [] },
    showToast: function (message, type) { toasts.push({ message: message, type: type }); },
    closeTaskEditor: function () {},
    deselectTask: function () {},
    loadTaskDetails: function () {},
    htmx: { trigger: function (el, name) { triggers.push(name); } },
    gbConfirm: function (message, title) {
      prompts.push({ message: message, title: title });
      return Promise.resolve(state.confirmAnswer);
    },
  });
  env.loadAll(sandbox, ["tasks/tasks-scoped-decisions.js"]);
  return {
    sandbox: sandbox,
    calls: calls,
    prompts: prompts,
    toasts: toasts,
    triggers: triggers,
    state: state,
  };
}

function flush() {
  return new Promise(function (resolve) { setTimeout(resolve, 0); });
}

h.test("[UAT:TASK-01] complete a task and require confirmation before delete", function () {
  var ctx = makeTasks();
  h.assert(typeof ctx.sandbox.completeTask === "function", "completeTask not loaded");
  h.assert(typeof ctx.sandbox.deleteTask === "function", "deleteTask not loaded");

  // Step 2 — checking a task marks it done through the real API call.
  ctx.sandbox.completeTask("task-1");
  return flush().then(function () {
    var completion = ctx.calls.filter(function (c) {
      return c.opts.method === "POST" && /\/tasks\/task-1\/complete$/.test(c.url);
    });
    h.assert(completion.length === 1, "completion POST missing: " + JSON.stringify(ctx.calls));
    h.assert(
      ctx.toasts.some(function (t) { return t.message === "Task completed"; }),
      "no completion feedback shown",
    );
    h.assert(
      ctx.triggers.indexOf("taskCreated") !== -1,
      "task list was not refreshed after completion",
    );

    // Step 3 — declining the confirmation must not delete anything.
    ctx.state.confirmAnswer = false;
    return ctx.sandbox.deleteTask("task-2").then(function () {
      h.assert(
        ctx.prompts.some(function (p) { return /Delete this task/.test(p.message); }),
        "delete did not ask for confirmation",
      );
      h.assert(
        !ctx.calls.some(function (c) { return c.opts.method === "DELETE"; }),
        "task deleted without confirmation",
      );

      // Confirming then issues exactly one DELETE for that task.
      ctx.state.confirmAnswer = true;
      return ctx.sandbox.deleteTask("task-2").then(function () {
        var deletions = ctx.calls.filter(function (c) {
          return c.opts.method === "DELETE" && /\/tasks\/task-2$/.test(c.url);
        });
        h.assert(deletions.length === 1, "confirmed delete did not run: " + JSON.stringify(ctx.calls));
      });
    });
  });
});
