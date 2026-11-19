"use strict";
/* Docs long-document structure (#2426), multilevel heading numbering.
 *
 * Numbering is bound to the paragraph style (h1..h6). A level can be decimal
 * (1, 1.1, 1.1.1) or legal (1., 1.1., 1.1.1.). Deeper levels restart under a
 * new higher-level heading unless that level is set to "continue previous";
 * an explicit restart marker on a heading restarts numbering at that heading.
 * The computed label is stored in data-gb-num and rendered in a non-editable
 * .gb-num-label span, so it survives re-serialisation and export.
 */
(function (window) {
  var NS = window.DocsStructure;
  if (!NS) return;
  var LEVELS = NS.LEVELS;

  function levelOf(node) {
    var tag = String(node && node.tagName || "").toLowerCase();
    return /^h[1-6]$/.test(tag) ? parseInt(tag.charAt(1), 10) : 0;
  }
  NS.headingLevel = levelOf;

  /* Pure formatter, also used by the offline harness. */
  NS.formatLevel = function (counters, level, scheme) {
    var parts = [];
    for (var i = 1; i <= level; i++) parts.push(String(counters[i] || 0));
    return scheme === "legal" ? parts.join(".") + "." : parts.join(".");
  };

  /* records: [{level, restart}] in document order → labels in the same order. */
  NS.computeNumbering = function (records, config) {
    var levels = (config && config.levels) || NS.defaultConfig().levels;
    var restartPerSection = !config || config.restartPerSection !== false;
    var counters = {};
    /* A "continue previous" level resumes the earlier list verbatim: both its
     * sequence counter and the parent prefix it started under. */
    var resumePrefix = {};
    for (var c = 1; c <= LEVELS; c++) counters[c] = 0;
    return (records || []).map(function (record) {
      var level = Math.max(1, Math.min(LEVELS, record.level || 1));
      var setting = levels[level - 1] || {};
      if (record.restart) {
        for (var r = level; r <= LEVELS; r++) counters[r] = 0;
        delete resumePrefix[level];
      }
      counters[level] += 1;
      for (var deeper = level + 1; deeper <= LEVELS; deeper++) {
        var deeperSetting = levels[deeper - 1] || {};
        if (restartPerSection && !deeperSetting.continuePrevious) {
          counters[deeper] = 0;
          delete resumePrefix[deeper];
        }
      }
      var continued = restartPerSection && setting.continuePrevious === true;
      if (continued && !resumePrefix[level]) {
        var prefix = {};
        for (var p = 1; p < level; p++) prefix[p] = counters[p] || 0;
        resumePrefix[level] = prefix;
      }
      var effective = {};
      for (var i = 1; i <= level; i++) {
        effective[i] = (continued && i < level && resumePrefix[level])
          ? (resumePrefix[level][i] || 0)
          : (counters[i] || 0);
      }
      effective[level] = counters[level];
      return NS.formatLevel(effective, level, setting.scheme);
    });
  };

  function walk(root) {
    if (!root || !root.querySelectorAll) return [];
    return Array.prototype.slice.call(root.querySelectorAll("h1,h2,h3,h4,h5,h6"));
  }
  NS.headings = walk;

  function recordsOf(headings) {
    return headings.map(function (node) {
      return {
        level: levelOf(node) || 1,
        restart: node.getAttribute && node.getAttribute("data-gb-num-restart") === "1",
      };
    });
  }

  function clearLabels(root) {
    var old = root.querySelectorAll ? root.querySelectorAll(".gb-num-label") : [];
    Array.prototype.slice.call(old).forEach(function (node) {
      if (node.parentNode) node.parentNode.removeChild(node);
    });
  }

  /* The heading title without the generated number, kept as an attribute so
   * cross-reference fields can show "text only" without parsing rendered HTML. */
  function cleanTitle(heading) {
    var clone = heading.cloneNode(true);
    clearLabels(clone);
    return (clone.textContent || "").trim();
  }

  /* Render the labels on a clone; called both by applyNumbering() and by the
   * update-all pass so the numbering is always fresh before fields resolve. */
  NS.renderNumbering = function (root) {
    var headings = walk(root);
    var labels = NS.computeNumbering(recordsOf(headings), NS.config());
    var doc = root.ownerDocument || document;
    clearLabels(root);
    headings.forEach(function (heading, index) {
      var label = labels[index] || "";
      heading.setAttribute("data-gb-num", label);
      heading.setAttribute("data-gb-num-level", String(levelOf(heading) || 1));
      heading.setAttribute("data-gb-title", cleanTitle(heading));
      var span = doc.createElement("span");
      span.className = "gb-num-label";
      span.setAttribute("contenteditable", "false");
      span.textContent = label + "\u00a0";
      heading.insertBefore(span, heading.firstChild);
    });
    return labels;
  };

  NS.applyNumbering = function () {
    if (!NS.getArticle()) {
      NS.statusKey("docs-structure-status-no-article", "Open a document first");
      return false;
    }
    NS.commitArticle(NS.renderNumbering);
    NS.statusKey("docs-structure-status-numbered", "Heading numbers updated");
    return true;
  };

  function isRestartMarked(heading) {
    return !!heading && !!heading.getAttribute && heading.getAttribute("data-gb-num-restart") === "1";
  }

  /* Mark the heading containing the caret as a restart point, then renumber. */
  NS.restartHere = function () {
    var a = NS.getArticle();
    var sel = NS.selection();
    if (!a || !sel || !sel.rangeCount) {
      NS.statusKey("docs-structure-status-no-selection", "Place the cursor in a heading first");
      return false;
    }
    var node = sel.getRangeAt(0).startContainer;
    while (node && node !== a && !(node.tagName && /^H[1-6]$/.test(node.tagName))) {
      node = node.parentNode;
    }
    if (!node || node === a || !node.tagName) {
      NS.statusKey("docs-structure-status-no-selection", "Place the cursor in a heading first");
      return false;
    }
    var clone = node.cloneNode(true);
    clone.setAttribute("data-gb-num-restart", isRestartMarked(node) ? "0" : "1");
    var html = clone.outerHTML || "";
    if (!html) return false;
    NS.commitNode(html, node);
    NS.applyNumbering();
    NS.statusKey(isRestartMarked(node)
      ? "docs-structure-status-restart-off"
      : "docs-structure-status-restart-on", "Restart marker updated");
    return true;
  };
})(window);
