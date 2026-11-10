/* Plan app — module 4/4: public globals and boot (split from plan/plan.js for #1731) */

(function (app) {
  "use strict";
  const {
    Store,
    View,
    render,
    initViewTabs,
    initAuth,
    initCollab,
    ensureServerPlan,
    wireToolbar,
    bindPlanCommentsBadge,
  } = app;

  window.addEventListener("DOMContentLoaded", function () {
    Store.load();
    Store.subscribe(render);
    initViewTabs();
    initAuth();
    wireToolbar();
    render();
    window.PlanStore = Store;
    window.PlanView = View;
    // #2136: the collaboration socket only joins a plan that exists on the
    // server and belongs to the caller, so the plan is created first.
    ensureServerPlan().then(function (id) {
      bindPlanCommentsBadge();
      // Without a server plan the socket has nothing to join.
      if (id) initCollab();
    });
  });
})(window.__gbPlan);
