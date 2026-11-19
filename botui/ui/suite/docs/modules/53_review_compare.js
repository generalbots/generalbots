"use strict";
/* Docs enterprise review workflow (#2425), compare-two-documents UI.
 *
 * Produces a local redline (per-hunk accept/reject) and a merge-result preview.
 * Export of the redline to DOCX/PDF and colour-coded author summaries are
 * deferred — the preview here is plain text with no server round-trip. */
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

  function hunkRow(hunk, refresh) {
    var row = el("div", "gb-review-hunk gb-review-hunk-" + hunk.type +
      (hunk.status === "open" ? "" : " gb-review-hunk-" + hunk.status));
    row.setAttribute("data-review-hunk", hunk.id);
    var head = el("div", "gb-review-hunk-head");
    var author = el("strong");
    author.textContent = hunk.author;
    head.appendChild(author);
    var kind = el("span", "gb-review-kind");
    kind.textContent = NS.typeLabel(hunk.type);
    head.appendChild(kind);
    var status = el("span", "gb-review-hunk-status");
    status.setAttribute("data-i18n", hunk.status === "open"
      ? "docs-review-open-changes" : ("docs-review-" + hunk.status));
    status.textContent = NS.tr(hunk.status === "open"
      ? "docs-review-open-changes" : ("docs-review-" + hunk.status),
      hunk.status === "open" ? "Open" : (hunk.status === "accepted" ? "Accepted" : "Rejected"));
    head.appendChild(status);
    row.appendChild(head);
    var text = el("pre", "gb-review-hunk-text");
    text.textContent = hunk.text;
    row.appendChild(text);
    if (hunk.status === "open") {
      var actions = el("div", "gb-review-hunk-actions");
      var accept = el("button", "gb-review-btn");
      i18n(accept, "docs-review-accept", "Accept");
      accept.setAttribute("type", "button");
      accept.addEventListener("click", function () {
        NS.acceptHunk(hunk.id);
        NS.statusKey("docs-review-status-hunk-accepted", "Hunk accepted");
        refresh();
      });
      actions.appendChild(accept);
      var reject = el("button", "gb-review-btn");
      i18n(reject, "docs-review-reject", "Reject");
      reject.setAttribute("type", "button");
      reject.addEventListener("click", function () {
        NS.rejectHunk(hunk.id);
        NS.statusKey("docs-review-status-hunk-rejected", "Hunk rejected");
        refresh();
      });
      actions.appendChild(reject);
      row.appendChild(actions);
    }
    return row;
  }

  function renderCompare(host) {
    var panel = host || document.getElementById("docs-review-compare");
    if (!panel) return;
    panel.textContent = "";
    panel.className = "gb-review-compare";
    var fields = el("div", "gb-review-compare-inputs");
    var left = document.createElement("textarea");
    left.className = "gb-review-textarea";
    left.id = "docs-review-left";
    left.setAttribute("data-i18n-placeholder", "docs-review-compare-left-placeholder");
    left.placeholder = NS.tr("docs-review-compare-left-placeholder", "Paste the original text");
    var right = document.createElement("textarea");
    right.className = "gb-review-textarea";
    right.id = "docs-review-right";
    right.setAttribute("data-i18n-placeholder", "docs-review-compare-right-placeholder");
    right.placeholder = NS.tr("docs-review-compare-right-placeholder", "Paste the edited text");
    var leftLabel = i18n(el("label"), "docs-review-compare-left", "Original text");
    leftLabel.appendChild(left);
    var rightLabel = i18n(el("label"), "docs-review-compare-right", "Edited text");
    rightLabel.appendChild(right);
    fields.appendChild(leftLabel);
    fields.appendChild(rightLabel);
    panel.appendChild(fields);
    var run = el("button", "gb-review-btn gb-review-btn-primary");
    run.setAttribute("type", "button");
    i18n(run, "docs-review-compare-run", "Show redline");
    panel.appendChild(run);
    var results = el("div", "gb-review-compare-results");
    panel.appendChild(results);

    function refresh() {
      results.textContent = "";
      var list = el("div", "gb-review-hunk-list");
      var hunks = NS.hunks();
      if (!hunks.length) {
        var none = el("p", "gb-review-empty");
        i18n(none, "docs-review-compare-no-diff", "No differences found");
        results.appendChild(none);
      } else {
        var heading = el("h4");
        i18n(heading, "docs-review-compare-hunks", "Redline hunks");
        results.appendChild(heading);
        hunks.forEach(function (hunk) { list.appendChild(hunkRow(hunk, refresh)); });
        results.appendChild(list);
      }
      var merge = el("div", "gb-review-merge");
      var mergeTitle = el("h4");
      i18n(mergeTitle, "docs-review-merge-preview", "Merge result preview");
      merge.appendChild(mergeTitle);
      var pre = el("pre", "gb-review-merge-text");
      pre.textContent = NS.mergeResult();
      merge.appendChild(pre);
      results.appendChild(merge);
    }

    run.addEventListener("click", function () {
      NS.computeRedline(left.value, right.value);
      refresh();
    });
    refresh();
  }
  NS.renderCompare = renderCompare;

  function openCompare() {
    if (!NS.dialog || !NS.dialog.shell) return;
    var shell = NS.dialog.shell({
      titleKey: "docs-review-compare-title",
      titleFallback: "Compare two documents"
    });
    renderCompare(shell.body);
    return shell.close;
  }
  NS.openCompare = openCompare;
})(window);
