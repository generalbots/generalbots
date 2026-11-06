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
    wireToolbar,
    bindPlanCommentsBadge,
  } = app;

  window.addEventListener("DOMContentLoaded", function () {
    Store.load();
    Store.subscribe(render);
    initViewTabs();
    initAuth();
    initCollab();
    wireToolbar();
    render();
    window.PlanStore = Store;
    window.PlanView = View;
    bindPlanCommentsBadge();
  });
})(window.__gbPlan);
