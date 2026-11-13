if (window.GBAppLifecycle) GBAppLifecycle.begin("adminpanel");
/* =============================================================================
   ADMIN MODULE - Missing Function Handlers
   These functions are called by onclick handlers in admin HTML files
   ============================================================================= */

(function () {
  "use strict";

  function notify(title, message) {
    try {
      if (window.GBModal && typeof window.GBModal.alert === "function") {
        window.GBModal.alert(message, title);
        return;
      }
      if (window.GBModal && typeof window.GBModal.open === "function") {
        window.GBModal.open({ title: title, content: message, buttons: [{ label: "OK", primary: true }] });
        return;
      }
    } catch (e) {
      console.debug("[admin-handlers] modal notify failed", e);
    }
    console.info("[admin-handlers]", title, message);
  }

  window.showTscDetails = window.showTscDetails || function (category) {
    var name = String(category || "category");
    notify("TSC Details", "Details for " + name + " controls are not available in this view yet.");
  };

  window.showControlRemediation = window.showControlRemediation || function (controlId) {
    var id = String(controlId || "");
    notify("Remediation", id ? ("Remediation workflow for control " + id + " is not configured yet.") : "No control selected.");
  };

  window.filterControls = window.filterControls || function (severity) {
    var value = String(severity || "all").toLowerCase();
    var items = document.querySelectorAll(".control-item, .controls-list .control-item, [data-severity]");
    items.forEach(function (item) {
      var sev = (item.getAttribute("data-severity") || "").toLowerCase();
      if (!sev) {
        if (item.classList.contains("critical")) sev = "critical";
        else if (item.classList.contains("high")) sev = "high";
        else if (item.classList.contains("medium")) sev = "medium";
        else if (item.classList.contains("low")) sev = "low";
      }
      var show = value === "all" || !sev || sev === value;
      item.style.display = show ? "" : "none";
    });
  };

  window.sortEndpoints = window.sortEndpoints || function (sortBy) {
    var by = String(sortBy || "requests");
    var list = document.querySelector(".endpoint-list");
    if (!list) return;
    var items = Array.prototype.slice.call(list.querySelectorAll(".endpoint-item"));
    function metricValue(item, kind) {
      var metrics = item.querySelectorAll(".endpoint-metric .metric-value");
      if (kind === "latency") {
        var t = metrics[1] ? metrics[1].textContent : "0";
        return parseFloat(t.replace(/[^0-9.]/g, "")) || 0;
      }
      if (kind === "errors") {
        var e = metrics[2] ? metrics[2].textContent : "0";
        return parseFloat(e.replace(/[^0-9.]/g, "")) || 0;
      }
      var r = metrics[0] ? metrics[0].textContent : "0";
      return parseFloat(r.replace(/[^0-9.]/g, "")) || 0;
    }
    items.sort(function (a, b) {
      var va = metricValue(a, by), vb = metricValue(b, by);
      return vb - va;
    });
    items.forEach(function (item) { list.appendChild(item); });
  };

  window.toggleMetric = window.toggleMetric || function (metric) {
    document.querySelectorAll('[data-metric="' + String(metric || "") + '"]').forEach(function (el) {
      var hidden = el.getAttribute("data-hidden") === "true";
      el.setAttribute("data-hidden", hidden ? "false" : "true");
      el.style.display = hidden ? "" : "none";
    });
    console.debug("[admin-handlers] toggleMetric", metric);
  };

  window.updateBotMetric = window.updateBotMetric || function (value) {
    var metric = String(value || "api_calls");
    document.querySelectorAll("[data-bot-metric-label]").forEach(function (el) {
      el.textContent = metric.replace(/_/g, " ");
    });
    console.debug("[admin-handlers] updateBotMetric", metric);
  };

  window.toggleBilling = window.toggleBilling || function () {
    var annual = document.getElementById("annualToggle");
    var isAnnual = annual ? !!annual.checked : false;
    document.querySelectorAll(".plan-card .price-amount").forEach(function (el) {
      if (!el.dataset.monthlyPrice) {
        el.dataset.monthlyPrice = el.textContent.trim();
      }
      var monthly = parseFloat(String(el.dataset.monthlyPrice).replace(/[^0-9.]/g, ""));
      if (!isFinite(monthly)) return;
      if (isAnnual) {
        var annualPrice = Math.round(monthly * 12 * 0.8);
        el.textContent = monthly === 0 ? "$0" : "$" + annualPrice;
      } else {
        el.textContent = el.dataset.monthlyPrice;
      }
    });
    document.querySelectorAll(".plan-card .price-period").forEach(function (el) {
      el.textContent = isAnnual ? "/year" : "/month";
    });
  };

  window.refreshHealth = window.refreshHealth || function () {
    try {
      if (window.htmx) {
        var grid = document.querySelector(".service-grid[hx-get]");
        if (grid) { htmx.trigger(grid, "load"); return; }
      }
    } catch (e) {
      console.debug("[admin-handlers] refreshHealth failed", e);
    }
  };
})();
