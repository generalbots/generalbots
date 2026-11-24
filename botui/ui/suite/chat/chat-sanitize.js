/* #2354 / #2743 — the single, versioned sanitizer for model-authored HTML.
 *
 * Bot output is untrusted: a KB document, a web-search result, an inbound
 * email, a tool OUTPUT or a prompt injection can make the model emit markup.
 * It used to be handed to `innerHTML` (and `<script>` re-executed with
 * `new Function`) in the suite origin, where a persisted live bearer token
 * could be exfiltrated — a stored-XSS to account-takeover chain.
 *
 * Every inline render path runs the parsed markdown through
 * `GBSanitize.html()` first. Interactive code belongs in the artifact iframe
 * (`messages/40_artifact.js`, sandbox without allow-same-origin), never in the
 * chat document.
 *
 * #2743 — the policy is now an ALLOWLIST with an explicit version (mirrored in
 * chat/sanitize-manifest.json). Parse -> clean -> serialize -> re-parse is the
 * mutation-XSS exposure class, so html() re-sanitizes until the serialized
 * string is a fixed point of the parser; a payload that never converges fails
 * closed to escaped text. `GBSanitize.selfTest()` runs the mXSS corpus in a
 * real DOM; the Node corpus test checks the same policy decisions offline.
 */
var GBSanitize = window.GBSanitize || (function () {
  "use strict";

  var VERSION = "2.0.0";
  var MAX_PASSES = 3;
  var HTML_NS = "http://www.w3.org/1999/xhtml";
  var MATH_NS = "http://www.w3.org/1998/Math/MathML";

  function setOf(list) {
    var out = Object.create(null);
    list.forEach(function (value) { out[value] = true; });
    return out;
  }

  // Tags kept in the HTML namespace. Anything not listed here is either
  // unwrapped (unknown inline chrome) or removed with its subtree (DROP_TREE).
  var ALLOWED_TAGS = setOf([
    "a", "abbr", "b", "blockquote", "br", "caption", "cite", "code", "col",
    "colgroup", "dd", "del", "details", "div", "dl", "dt", "em", "figcaption",
    "figure", "h1", "h2", "h3", "h4", "h5", "h6", "hr", "i", "img", "ins",
    "kbd", "li", "mark", "ol", "p", "pre", "q", "s", "samp", "section",
    "small", "span", "strong", "sub", "summary", "sup", "table", "tbody",
    "td", "tfoot", "th", "thead", "time", "tr", "u", "ul", "var", "wbr",
  ]);

  // MathML is a separate namespace, not HTML: checking namespaceURI as well as
  // localName is what closes namespace-confusion payloads (<svg><p>, mglyph).
  var ALLOWED_MATH_TAGS = setOf([
    "math", "semantics", "annotation", "mrow", "mi", "mn", "mo", "mtext",
    "mspace", "mfrac", "msqrt", "msup", "msub", "msubsup", "munder", "mover",
    "munderover", "mpadded", "mstyle", "menclose", "mtable", "mtr", "mtd",
  ]);

  // Removed WITH their content: raw-text parsers and foreign-content gadgets
  // that mutate once the serialized string is re-parsed into the live bubble.
  var DROP_TREE = setOf([
    "script", "style", "iframe", "frame", "frameset", "object", "embed",
    "applet", "link", "meta", "base", "form", "input", "button", "select",
    "option", "textarea", "label", "fieldset", "noscript", "noembed",
    "noframes", "template", "title", "xmp", "plaintext", "listing", "marquee",
    "portal", "slot", "audio", "video", "source", "track", "canvas",
  ]);

  var URI_ATTRS = setOf([
    "href", "src", "cite", "action", "formaction", "poster", "background",
    "data", "srcset", "srcdoc", "xlink:href",
  ]);

  // Attributes kept on allowed elements. `id`/`name` are deliberately absent:
  // they are DOM-clobbering gadgets against globals such as window.GBLiveArtifact.
  var ALLOWED_ATTRS = setOf([
    "class", "title", "role", "dir", "lang", "alt", "width", "height",
    "loading", "colspan", "rowspan", "align", "start", "reversed", "href",
    "src", "cite", "style", "target", "rel", "encoding", "mathvariant",
    "display",
  ]);

  function attrAllowed(name) {
    var attr = String(name == null ? "" : name).toLowerCase();
    if (!attr || attr.indexOf("on") === 0) return false;
    if (attr === "id" || attr === "name" || attr.indexOf("data-") === 0) return false;
    if (attr.indexOf("aria-") === 0) return true;
    return !!ALLOWED_ATTRS[attr];
  }

  function tagAllowed(localName, namespaceURI) {
    var local = String(localName == null ? "" : localName).toLowerCase();
    if (namespaceURI === MATH_NS) return !!ALLOWED_MATH_TAGS[local];
    if (namespaceURI && namespaceURI !== HTML_NS) return false;
    return !!ALLOWED_TAGS[local] && !DROP_TREE[local];
  }

  function stripControls(value) {
    return String(value == null ? "" : value).replace(/[\u0000-\u0020\u00A0]/g, "").toLowerCase();
  }

  function uriIsAllowed(value) {
    var val = stripControls(value);
    if (!val) return true;
    if (/^(javascript|vbscript|livescript|data):/.test(val)) {
      // Inert raster images only: data:text/html and data:image/svg+xml can
      // carry script when opened as a document.
      return /^data:image\/(png|jpe?g|gif|webp|bmp)[;,]/.test(val);
    }
    if (/^(blob|filesystem|about|chrome|resource|view-source):/.test(val)) return false;
    return true;
  }

  function styleIsAllowed(value) {
    var val = stripControls(value);
    if (/expression\(|behavior:|-moz-binding:|@import/.test(val)) return false;
    var urls = String(value == null ? "" : value).match(/url\(([^)]*)\)/gi) || [];
    for (var i = 0; i < urls.length; i++) {
      var inner = urls[i].replace(/^url\(/i, "").replace(/\)$/, "").replace(/['"]/g, "");
      if (!uriIsAllowed(inner)) return false;
    }
    return true;
  }

  function escapeText(text) {
    return String(text == null ? "" : text)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;")
      .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  function cleanAttrs(el, localName) {
    var attrs = Array.prototype.slice.call(el.attributes || []);
    for (var i = 0; i < attrs.length; i++) {
      var attr = attrs[i];
      var name = String(attr.name || "").toLowerCase();
      var value = attr.value;
      var remove = !attrAllowed(name);
      if (!remove && URI_ATTRS[name] && !uriIsAllowed(value)) remove = true;
      if (!remove && name === "style" && !styleIsAllowed(value)) remove = true;
      if (remove) el.removeAttribute(name);
    }
    if (localName === "a" && el.getAttribute("href")) {
      el.setAttribute("rel", "noopener noreferrer");
    }
  }

  function unwrap(el) {
    var parent = el.parentNode;
    if (!parent) return;
    while (el.firstChild) parent.insertBefore(el.firstChild, el);
    parent.removeChild(el);
  }

  function cleanFragment(root) {
    var nodes = root.querySelectorAll ? Array.prototype.slice.call(root.querySelectorAll("*")) : [];
    for (var i = nodes.length - 1; i >= 0; i--) {
      var el = nodes[i];
      if (!el.parentNode) continue;
      var local = String(el.localName || el.nodeName || "").toLowerCase();
      if (!tagAllowed(local, el.namespaceURI)) {
        if (el.namespaceURI && el.namespaceURI !== HTML_NS) el.parentNode.removeChild(el);
        else if (DROP_TREE[local]) el.parentNode.removeChild(el);
        else unwrap(el);
        continue;
      }
      cleanAttrs(el, local);
    }
  }

  function cleanOnce(input) {
    var tpl = document.createElement("template");
    tpl.innerHTML = input;
    cleanFragment(tpl.content);
    return tpl.innerHTML;
  }

  // A fixed point means the string the caller assigns to innerHTML re-parses
  // to exactly the DOM we sanitized. Non-convergent input fails closed.
  function html(input) {
    var raw = String(input == null ? "" : input);
    var current = raw;
    var output = raw;
    for (var pass = 0; pass < MAX_PASSES; pass++) {
      output = cleanOnce(current);
      if (output === current) return output;
      current = output;
    }
    return escapeText(raw);
  }

  // An inert DOM has no element or attribute that the policy would reject.
  function inertIssues(output) {
    var issues = [];
    var tpl = document.createElement("template");
    tpl.innerHTML = output;
    var nodes = tpl.content.querySelectorAll("*");
    for (var i = 0; i < nodes.length; i++) {
      var el = nodes[i];
      var local = String(el.localName || el.nodeName || "").toLowerCase();
      if (!tagAllowed(local, el.namespaceURI)) issues.push("tag:" + local);
      var attrs = Array.prototype.slice.call(el.attributes || []);
      for (var j = 0; j < attrs.length; j++) {
        var attr = attrs[j];
        var name = String(attr.name || "").toLowerCase();
        if (!attrAllowed(name)) issues.push("attr:" + name);
        if (URI_ATTRS[name] && !uriIsAllowed(attr.value)) issues.push("url:" + name);
        if (name === "style" && !styleIsAllowed(attr.value)) issues.push("style");
      }
    }
    return issues;
  }

  // XSS / mXSS corpus. `nodes`, `attrs` and `urls` are the offline policy
  // expectations; `html` is the browser payload exercised by selfTest().
  var XSS_CORPUS = [
    { id: "script", category: "element", html: "<script>alert(1)</script>", nodes: ["script"] },
    { id: "img-handler", category: "event-handler", html: '<img src=x onerror="alert(1)">', attrs: ["onerror"] },
    { id: "div-handler", category: "event-handler", html: '<div onmouseover="alert(1)">x</div>', attrs: ["onmouseover"] },
    { id: "js-href", category: "uri", html: '<a href="javascript:alert(1)">x</a>', urls: ["javascript:alert(1)"] },
    { id: "tab-js-href", category: "uri", html: '<a href="java&#9;script:alert(1)">x</a>', urls: ["javascript:alert(1)"] },
    { id: "data-html", category: "uri", html: '<a href="data:text/html,<script>alert(1)</script>">x</a>', urls: ["data:text/html,<script>alert(1)</script>"] },
    { id: "svg-script", category: "namespace", html: '<svg><script>alert(1)</script></svg>', nodes: ["svg"] },
    { id: "svg-foreignobject", category: "namespace", html: '<svg><foreignObject><img src=x onerror=alert(1)></foreignObject></svg>', nodes: ["svg"] },
    { id: "svg-mutation", category: "attribute-mutation", html: '<svg><p><style><a id="</style><img src=x onerror=alert(1)>">', nodes: ["svg"] },
    { id: "math-mglyph", category: "nested-foreign", html: '<math><mtext><table><mglyph><style><!--</style><img src=x onerror=alert(1)>--></mglyph></table></mtext></math>', nodes: ["mglyph"] },
    { id: "p-mutation", category: "attribute-mutation", html: '<p title="</p><img src=x onerror=alert(1)">">x</p>', attrs: ["onerror"] },
    { id: "noscript-mutation", category: "attribute-mutation", html: '<noscript><p title="</noscript><img src=x onerror=alert(1)>">', nodes: ["noscript"] },
    { id: "iframe-srcdoc", category: "element", html: '<iframe srcdoc="<script>alert(1)</script>"></iframe>', nodes: ["iframe"], attrs: ["srcdoc"] },
    { id: "form-action", category: "uri", html: '<form action="javascript:alert(1)"><button>x</button></form>', nodes: ["form"], urls: ["javascript:alert(1)"] },
    { id: "style-expression", category: "style", html: '<div style="width:expression(alert(1))">x</div>', styles: ["width:expression(alert(1))"] },
    { id: "dom-clobber", category: "dom-clobbering", html: '<div id="GBLiveArtifact">x</div>', attrs: ["id"] },
  ];

  function selfTest() {
    var result = { version: VERSION, total: XSS_CORPUS.length, failures: [], pass: false };
    if (typeof document === "undefined" || !document.createElement) {
      result.failures.push("DOM unavailable");
      return result;
    }
    XSS_CORPUS.forEach(function (payload) {
      var output = html(payload.html);
      inertIssues(output).forEach(function (issue) {
        result.failures.push(payload.id + ": " + issue + " -> " + output);
      });
      if (html(output) !== output) result.failures.push(payload.id + ": not idempotent");
    });
    result.pass = result.failures.length === 0;
    return result;
  }

  return {
    VERSION: VERSION,
    HTML_NS: HTML_NS,
    MATH_NS: MATH_NS,
    POLICY: {
      version: VERSION,
      allowedTags: Object.keys(ALLOWED_TAGS),
      allowedMathTags: Object.keys(ALLOWED_MATH_TAGS),
      droppedTags: Object.keys(DROP_TREE),
      uriAttrs: Object.keys(URI_ATTRS),
      allowedAttrs: Object.keys(ALLOWED_ATTRS),
    },
    XSS_CORPUS: XSS_CORPUS,
    html: html,
    uriIsAllowed: uriIsAllowed,
    styleIsAllowed: styleIsAllowed,
    attrAllowed: attrAllowed,
    tagAllowed: tagAllowed,
    selfTest: selfTest,
  };
})();
