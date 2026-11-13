/* #2354 — the single sanitizer for model-authored HTML.
 *
 * Bot output is untrusted: a KB document, a web-search result, an inbound
 * email, a tool OUTPUT or a prompt injection can make the model emit markup.
 * It used to be handed to `innerHTML` (and `<script>` re-executed with
 * `new Function`) in the suite origin, where a persisted live bearer token
 * could be exfiltrated — a stored-XSS to account-takeover chain.
 *
 * Every inline render path now runs the parsed markdown through
 * `GBSanitize.html()` first. Interactive code belongs in the artifact iframe
 * (`messages/40_artifact.js`, sandbox without allow-same-origin), never in the
 * chat document.
 */
var GBSanitize = window.GBSanitize || (function () {
  "use strict";

  // Elements that can execute, navigate, submit or restyle the suite document.
  var DROP = [
    "script", "style", "iframe", "frame", "frameset", "object", "embed",
    "applet", "link", "meta", "base", "form", "input", "textarea", "select",
    "option", "button", "foreignObject", "use", "set",
  ].join(",");

  // Attributes whose value is fetched or navigated to.
  var URI_ATTRS = ["href", "src", "xlink:href", "action", "formaction", "poster", "background", "data", "srcset", "srcdoc"];

  function uriIsAllowed(value) {
    // Strip control characters and whitespace first: `java\tscript:` and
    // `java\nscript:` both execute in a browser.
    var val = String(value == null ? "" : value).replace(/[\u0000-\u0020\u00A0]/g, "").toLowerCase();
    if (!val) return true;
    if (/^(javascript|vbscript|livescript):/.test(val)) return false;
    if (val.indexOf("data:") === 0) {
      // Inert image payloads only; data:text/html and data:image/svg+xml can
      // carry script when opened as a document.
      return /^data:image\/(png|jpe?g|gif|webp|bmp)[;,]/.test(val);
    }
    return true;
  }

  function html(input) {
    var tpl = document.createElement("template");
    tpl.innerHTML = String(input == null ? "" : input);

    Array.prototype.forEach.call(tpl.content.querySelectorAll(DROP), function (node) {
      if (node.parentNode) node.parentNode.removeChild(node);
    });

    Array.prototype.forEach.call(tpl.content.querySelectorAll("*"), function (el) {
      Array.prototype.slice.call(el.attributes).forEach(function (attr) {
        var name = attr.name.toLowerCase();
        if (name.indexOf("on") === 0) {
          el.removeAttribute(attr.name);
          return;
        }
        if (URI_ATTRS.indexOf(name) !== -1 && !uriIsAllowed(attr.value)) {
          el.removeAttribute(attr.name);
        }
      });
      if (el.tagName && el.tagName.toLowerCase() === "a" && el.getAttribute("href")) {
        el.setAttribute("rel", "noopener noreferrer");
      }
    });

    return tpl.innerHTML;
  }

  return { html: html, uriIsAllowed: uriIsAllowed };
})();
