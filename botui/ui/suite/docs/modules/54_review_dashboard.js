"use strict";
/* Docs enterprise review workflow (#2425), review dashboard.
 *
 * Counts come straight from the pure model (computeDashboard) so the numbers
 * shown here are the same ones the offline harness verifies. Time in review
 * is elapsed wall-clock since the local review session started. */
(function (window) {
  var NS = window.DocsReview;
  if (!NS) return;

  function el(tag, cls) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    return node;
  }

  function i18n(node, key, fallback) {
    node.setAttribute("data-i18n", key);
    node.textContent = NS.tr(key, fallback);
    return node;
  }

  function metric(host, key, fallback, value) {
    var row = el("div", "gb-review-metric");
    var label = el("span", "gb-review-metric-label");
    i18n(label, key, fallback);
    var number = el("strong", "gb-review-metric-value");
    number.textContent = value;
    row.appendChild(label);
    row.appendChild(number);
    host.appendChild(row);
  }

  function authorRow(author, counts) {
    var row = el("tr", "gb-review-author-row");
    var name = el("td");
    name.textContent = author;
    row.appendChild(name);
    ["open", "accepted", "rejected", "total"].forEach(function (field) {
      var cell = el("td");
      cell.textContent = counts[field];
      row.appendChild(cell);
    });
    return row;
  }

  function renderDashboard(host) {
    var panel = host || document.getElementById("docs-review-dashboard");
    if (!panel) return;
    panel.textContent = "";
    panel.className = "gb-review-dashboard";
    var data = NS.computeDashboard();

    var summary = el("div", "gb-review-metrics");
    metric(summary, "docs-review-dash-total-open", "Total open changes", data.totalOpen);
    metric(summary, "docs-review-dash-unresolved", "Unresolved threads", data.unresolvedThreads);
    var timeRow = el("div", "gb-review-metric");
    var timeLabel = el("span", "gb-review-metric-label");
    i18n(timeLabel, "docs-review-dash-time", "Time in review");
    var timeValue = el("strong", "gb-review-metric-value");
    timeValue.setAttribute("data-i18n", "docs-review-time-minutes");
    timeValue.setAttribute("data-i18n-params",
      JSON.stringify({ count: data.timeInReviewMinutes }));
    timeValue.textContent = NS.tr("docs-review-time-minutes", "{ $count } min",
      { count: data.timeInReviewMinutes });
    timeRow.appendChild(timeLabel);
    timeRow.appendChild(timeValue);
    summary.appendChild(timeRow);
    var signKey = "docs-review-signoff-" + data.signOff;
    var signFallback = { pending: "Pending", approved: "Approved", changes: "Changes requested" }[data.signOff];
    metric(summary, "docs-review-dash-signoff", "Sign-off status", NS.tr(signKey, signFallback));
    panel.appendChild(summary);

    var table = el("table", "gb-review-author-table");
    var thead = el("thead");
    var headRow = el("tr");
    ["docs-review-change-author", "docs-review-open-changes", "docs-review-accepted",
      "docs-review-rejected", "docs-review-dash-by-author"].forEach(function (key, index) {
      var th = el("th");
      var fallback = ["Author", "Open", "Accepted", "Rejected", "Total"][index];
      i18n(th, key, fallback);
      headRow.appendChild(th);
    });
    thead.appendChild(headRow);
    table.appendChild(thead);
    var tbody = el("tbody");
    Object.keys(data.byAuthor).sort().forEach(function (author) {
      tbody.appendChild(authorRow(author, data.byAuthor[author]));
    });
    table.appendChild(tbody);
    panel.appendChild(table);

    var signActions = el("div", "gb-review-signoff-actions");
    [["approved", "docs-review-signoff-approved", "Approved"],
     ["changes", "docs-review-signoff-changes", "Changes requested"],
     ["pending", "docs-review-signoff-pending", "Pending"]].forEach(function (spec) {
      var button = el("button", "gb-review-btn");
      button.setAttribute("type", "button");
      button.setAttribute("data-i18n", spec[1]);
      button.textContent = NS.tr(spec[1], spec[2]);
      button.addEventListener("click", function () {
        NS.setSignOff(spec[0]);
        NS.statusKey("docs-review-status-signoff", "Sign-off status updated");
        renderDashboard(panel);
      });
      signActions.appendChild(button);
    });
    panel.appendChild(signActions);
  }
  NS.renderDashboard = renderDashboard;

  function openDashboard() {
    if (!NS.dialog || !NS.dialog.shell) return;
    var shell = NS.dialog.shell({
      titleKey: "docs-review-dash-title",
      titleFallback: "Review dashboard"
    });
    renderDashboard(shell.body);
    return shell.close;
  }
  NS.openDashboard = openDashboard;
})(window);
