/* #2354/#2743 — the single sanitizer for model-authored HTML.
 *
 * Bot output is untrusted: a KB document, a web-search result, an inbound
 * email, a tool OUTPUT or a prompt injection can make the model emit markup.
 * It used to be handed to `innerHTML` (and `<script>` re-executed with
 * `new Function`) in the suite origin, where a persisted live bearer token
 * could be exfiltrated — a stored-XSS to account-takeover chain.
 *
 * Every inline render path runs the parsed markdown through `GBSanitize.html()`
 * first. Interactive code belongs in the artifact iframe
 * (`messages/40_artifact.js`, sandbox without allow-same-origin), never in the
 * chat document. #2707 added a fail-closed `textContent` path in multichat for
 * the case where this script has not loaded yet — do not remove it.
 *
 * #2743: parse -> scrub -> serialize -> parse -> scrub is deliberate. A
 * hand-rolled `<template>` filter that serializes once is vulnerable to
 * mutation XSS: the serialized string is re-parsed by `innerHTML` in the
 * caller with different namespace rules (svg/math/foreignObject). Scrubbing
 * the final serialized string once more closes that gap. This module is
 * versioned and its SHA-256 is recorded in chat-sanitize.manifest.json.
 */
var GB_SANITIZE_VERSION = "2.0.0";
var GBSanitize = window.GBSanitize || (function () {
  "use strict";

  // Elements that can execute, navigate, submit, restyle or change the
  // parsing namespace of the suite document. svg/math are removed wholesale:
  // markdown output has no legitimate use for them and they are the main
  // namespace-confusion/mXSS vector. KaTeX is applied by chat-math.js, not by
  // trusting model-authored MathML.
  var DROP = [
    "script", "style", "iframe", "frame", "frameset", "object", "embed",
    "applet", "link", "meta", "base", "form", "input", "textarea", "select",
    "option", "button", "foreignObject", "use", "set", "svg", "math",
    "annotation-xml", "mglyph", "malignmark", "template", "noscript",
    "isindex", "keygen", "portal",
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

  function scrub(fragment) {
    // Remove entire subtrees first: an <svg><foreignObject> pair can otherwise
    // leave executable markup behind when only element names are filtered.
    Array.prototype.forEach.call(fragment.querySelectorAll(DROP), function (node) {
      if (node.parentNode) node.parentNode.removeChild(node);
    });

    Array.prototype.forEach.call(fragment.querySelectorAll("*"), function (el) {
      Array.prototype.slice.call(el.attributes).forEach(function (attr) {
        var name = attr.name.toLowerCase();
        // Event handlers, namespace declarations and xml:* metadata are never
        // needed in a chat bubble and are the usual mutation-XSS carriers.
        if (name.indexOf("on") === 0 || name.indexOf("xmlns") === 0 ||
            name.indexOf("xml:") === 0) {
          el.removeAttribute(attr.name);
          return;
        }
        if (URI_ATTRS.indexOf(name) !== -1 && !uriIsAllowed(attr.value)) {
          el.removeAttribute(attr.name);
          return;
        }
        if (name === "style" && /url\s*\(|expression\s*\(|@import/i.test(attr.value)) {
          el.removeAttribute(attr.name);
          return;
        }
        // Strip control characters from every remaining attribute value so a
        // mutation pass cannot turn an inert value into a scheme.
        if (/[\u0000-\u001f]/.test(attr.value)) {
          el.setAttribute(attr.name, attr.value.replace(/[\u0000-\u001f]/g, ""));
        }
      });
      if (el.tagName && el.tagName.toLowerCase() === "a" && el.getAttribute("href")) {
        el.setAttribute("rel", "noopener noreferrer");
      }
    });
  }

  function parseAndScrub(serialized) {
    var tpl = document.createElement("template");
    tpl.innerHTML = serialized;
    scrub(tpl.content);
    return tpl.innerHTML;
  }

  function html(input) {
    var once = parseAndScrub(String(input == null ? "" : input));
    // The caller re-parses this string with innerHTML; run the same scrub on
    // the exact final serialization to defeat mutation XSS (#2743).
    return parseAndScrub(once);
  }

  return { html: html, uriIsAllowed: uriIsAllowed, version: GB_SANITIZE_VERSION };
})();
