"use strict";
/* Shared HTML sanitizer for Docs document bodies (#2840).
 *
 * Document content reaches the editor from three untrusted surfaces:
 *   - GET  /api/docs/open-from-drive   (modules/99_init.js)
 *   - POST /api/docs/history           (modules/11_history_server.js)
 *   - collab "edit" WebSocket message (modules/05_collab.js)
 * The read-only share viewer sanitizes the same class of content. This is that
 * vetted sanitizer, extracted so every sink uses ONE implementation. Authored
 * formatting (p/b/i/table/span/...) survives; active content does not.
 *
 * Loaded by docs.html (editor) and shared.html (viewer) before their scripts.
 * Exposes window.DocsSanitize and the short global docsSanitizeHtml().
 */
(function (global) {
  "use strict";

  var BLOCKED_TAGS = ["script", "iframe", "object", "embed", "form", "meta", "link", "base"];
  var URL_ATTRS = ["href", "src", "xlink:href", "action", "formaction"];

  function escapeText(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  // DOMParser has already decoded HTML entities in the attribute value; strip
  // whitespace/control characters so "java\nscript:" and "java&#10;script:"
  // cannot slip past the scheme check.
  function isScriptUrl(value) {
    var v = String(value == null ? "" : value)
      .replace(/[\u0000-\u0020\u007f]/g, "")
      .toLowerCase();
    return v.indexOf("javascript:") === 0 ||
      v.indexOf("vbscript:") === 0 ||
      v.indexOf("data:text/html") === 0;
  }

  function sanitize(html) {
    var input = String(html == null ? "" : html);
    if (typeof DOMParser === "undefined") {
      // Non-DOM runtime: fail closed, never hand raw markup back to a sink.
      return escapeText(input);
    }
    var doc;
    try {
      doc = new DOMParser().parseFromString(input, "text/html");
    } catch (e) {
      return escapeText(input);
    }
    BLOCKED_TAGS.forEach(function (tag) {
      var found = doc.querySelectorAll(tag);
      for (var i = found.length - 1; i >= 0; i--) {
        var node = found[i];
        if (node.parentNode) node.parentNode.removeChild(node);
      }
    });
    var all = doc.querySelectorAll("*");
    for (var j = 0; j < all.length; j++) {
      var el = all[j];
      var attrs = Array.prototype.slice.call(el.attributes || []);
      for (var k = 0; k < attrs.length; k++) {
        var attr = attrs[k];
        var name = String(attr.name || "").toLowerCase();
        var value = String(attr.value == null ? "" : attr.value);
        if (name.indexOf("on") === 0 || name === "srcdoc") {
          el.removeAttribute(attr.name);
          continue;
        }
        if (URL_ATTRS.indexOf(name) !== -1 && isScriptUrl(value)) {
          el.removeAttribute(attr.name);
        }
      }
      if (el.tagName === "A") {
        el.setAttribute("rel", "noopener noreferrer");
        el.setAttribute("target", "_blank");
      }
    }
    return doc.body.innerHTML;
  }

  // Small, side-effect-free self-test. Open the editor with
  // "#docs-sanitize-selftest" to log it, or call DocsSanitize.selfTest().
  function selfTest() {
    var cases = [
      { name: "formatting preserved", input: "<p>Hello <b>world</b></p>", keep: "<b>world</b>" },
      { name: "script removed", input: "<p>a</p><script>alert(1)</script><p>b</p>", drop: "alert(1)" },
      { name: "event handler removed", input: '<img src="x" onerror="alert(1)">', drop: "onerror" },
      { name: "javascript url removed", input: '<a href="javascript:alert(1)">x</a>', drop: "javascript:" },
      { name: "iframe removed", input: '<iframe src="x"></iframe>ok', drop: "<iframe" },
      { name: "obfuscated url removed", input: '<a href="java\nscript:alert(1)">x</a>', drop: "script:alert" }
    ];
    var results = cases.map(function (c) {
      var out = sanitize(c.input).toLowerCase();
      var ok = c.keep
        ? out.indexOf(c.keep.toLowerCase()) !== -1
        : out.indexOf(String(c.drop).toLowerCase()) === -1;
      return { name: c.name, pass: !!ok };
    });
    return {
      pass: results.filter(function (r) { return r.pass; }).length,
      total: results.length,
      results: results
    };
  }

  global.DocsSanitize = { sanitize: sanitize, html: sanitize, escapeText: escapeText, selfTest: selfTest };
  global.docsSanitizeHtml = sanitize;

  if (global.location && /docs-sanitize-selftest/.test(String(global.location.hash))) {
    try { console.log("[DocsSanitize] selfTest", selfTest()); } catch (e) { /* no console */ }
  }
})(typeof window !== "undefined" ? window : this);
