/* #2953 — Mail must never load an external asset.
 *
 * The Suite CSP is `style-src 'self'; font-src 'self'; img-src 'self'`, but an
 * inbound message, a saved signature or an API-produced suggestion can still
 * carry `<link>`, `@font-face`, remote `url()` or a tracking pixel. Browsers
 * try to fetch those before CSP reports the violation, so the markup has to be
 * cleaned *before* it reaches innerHTML.
 *
 * `stripRemote()` is a DOM-free allowlist of load contexts: it removes the
 * elements that can request an asset and every attribute/CSS declaration that
 * can name an external URL. `html()` then runs the shared, versioned
 * GBSanitize policy when it is loaded (chat/chat-sanitize.js), and fails closed
 * to escaped text when it is not. A final sweep guarantees that a URL kept by
 * the general sanitizer (it permits https for links) cannot become a load in
 * the Mail chrome. `selfTest()` exercises the hostile corpus offline.
 */
(function (root) {
  "use strict";

  var VERSION = "1.0.0";

  // Tags that can initiate a request or host executable/style content. Message
  // bodies are read-only; none of these belong in the suite document.
  var DROP_TAGS = {
    script: true, style: true, link: true, meta: true, base: true,
    iframe: true, frame: true, frameset: true, object: true, embed: true,
    applet: true, video: true, audio: true, source: true, track: true,
    form: true, input: true, button: true, select: true, textarea: true,
  };

  // Every attribute that can name a resource. `href` on an anchor is included:
  // a click-through URL in a message is a tracking token, not a local asset.
  var URI_ATTRS = {
    href: true, src: true, srcset: true, cite: true, action: true,
    formaction: true, poster: true, background: true, data: true,
    srcdoc: true, ping: true, "xlink:href": true, "xmlns:xlink": true,
  };

  var HOSTILE_CORPUS = [
    {
      id: "tracking-pixel",
      html: '<p>Hello</p><img src="https://tracking.inflection.io/open?m=1" width="1" height="1">',
      forbidden: ["tracking.inflection.io"],
    },
    {
      id: "remote-stylesheet",
      html: '<link rel="stylesheet" href="https://client-data.knak.io/assets/fonts.css">',
      forbidden: ["knak.io", "<link"],
    },
    {
      id: "google-fonts",
      html: '<LINK HREF="https://fonts.googleapis.com/css?family=Roboto" REL="stylesheet">',
      forbidden: ["fonts.googleapis.com", "<link"],
    },
    {
      id: "font-face",
      html: '<style>@font-face{font-family:x;src:url("https://cdn.openai.com/common/fonts/openai-sans/x.woff2")}</style><p>x</p>',
      forbidden: ["cdn.openai.com", "@font-face"],
    },
    {
      id: "inline-css-url",
      html: '<div style="background-image:url(https://r.openai.com/pixel.gif);color:#fff">x</div>',
      forbidden: ["r.openai.com", "url("],
    },
    {
      id: "srcset-and-protocol-relative",
      html: '<img srcset="https://email.investmentnetwork.cn/a.png 1x, //cdn.openai.com/b.png 2x" alt="x">',
      forbidden: ["investmentnetwork.cn", "cdn.openai.com"],
    },
    {
      id: "protocol-relative-image",
      html: '<img src="//tracking.inflection.io/px.gif" alt="x">',
      forbidden: ["//tracking.inflection.io"],
    },
    {
      id: "entity-obfuscated",
      html: '<img src="https&#58;//tracking.inflection.io/px.gif" alt="x">',
      forbidden: ["tracking.inflection.io"],
    },
    {
      id: "frame-and-object",
      html: '<iframe src="https://cdn.openai.com/x.html"></iframe><object data="https://knak.io/x"></object>',
      forbidden: ["iframe", "object", "cdn.openai.com", "knak.io"],
    },
    {
      id: "meta-refresh",
      html: '<meta http-equiv="refresh" content="0;url=https://tracking.inflection.io/redir">',
      forbidden: ["meta", "tracking.inflection.io"],
    },
    {
      id: "anchor-href",
      html: '<a href="https://r.openai.com/click">Read</a>',
      forbidden: ["r.openai.com"],
    },
  ];

  function decodeEntities(value) {
    return String(value == null ? "" : value)
      .replace(/&#x([0-9a-f]+);/gi, function (_all, hex) {
        return String.fromCharCode(parseInt(hex, 16));
      })
      .replace(/&#([0-9]+);/g, function (_all, dec) {
        return String.fromCharCode(parseInt(dec, 10));
      })
      .replace(/&colon;/gi, ":")
      .replace(/&sol;/gi, "/");
  }

  function isExternalUrl(value) {
    var val = decodeEntities(value).replace(/[\u0000-\u0020\u00A0]/g, "").toLowerCase();
    if (!val) return false;
    if (/^(https?:)?\/\//.test(val) || /^[\\/]{2}/.test(val)) return true;
    if (/^(blob|filesystem|about|chrome|resource|view-source|javascript|vbscript):/.test(val)) return true;
    if (/^data:/.test(val)) {
      return !/^data:image\/(png|jpe?g|gif|webp|bmp)[;,]/.test(val);
    }
    return false;
  }

  function stripCss(value) {
    var css = String(value == null ? "" : value);
    css = css.replace(/@import\b[^;]*;?/gi, "");
    css = css.replace(/@font-face\b[^}]*\}/gi, "");
    css = css.replace(/url\(\s*(['"]?)([^'")]*)\1\s*\)/gi, function (all, _quote, url) {
      return isExternalUrl(url) ? "" : all;
    });
    var urls = css.match(/(?:https?:)?\/\/[^\s'")]+/gi) || [];
    for (var i = 0; i < urls.length; i++) {
      if (isExternalUrl(urls[i])) return "";
    }
    return css.trim();
  }

  function stripTag(tagText) {
    var nameMatch = /^<\s*\/?\s*([a-z][\w:-]*)/i.exec(tagText);
    if (!nameMatch) return "";
    var tag = nameMatch[1].toLowerCase();
    if (DROP_TAGS[tag]) return "";

    var out = tagText.replace(
      /([:\w-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/g,
      function (all, attr, doubleQuoted, singleQuoted, bare) {
        var name = attr.toLowerCase();
        var value = doubleQuoted !== undefined ? doubleQuoted
          : singleQuoted !== undefined ? singleQuoted : bare;
        if (name === "style") {
          var css = stripCss(value);
          return css ? attr + '="' + css.replace(/"/g, "&quot;") + '"' : "";
        }
        if (name.indexOf("on") === 0) return "";
        if (URI_ATTRS[name] && isExternalUrl(value)) return "";
        return all;
      },
    );

    // An image whose src was remote is a tracker, not a broken local image.
    if (tag === "img" && !/\bsrc\s*=/i.test(out)) return "";
    return out;
  }

  function stripRemote(input) {
    var out = String(input == null ? "" : input);
    out = out.replace(/<style\b[^>]*>[\s\S]*?<\/style\s*>/gi, "");
    out = out.replace(/<script\b[^>]*>[\s\S]*?<\/script\s*>/gi, "");
    out = out.replace(/<!--[\s\S]*?-->/g, "");
    out = out.replace(/<\?([\s\S]*?)\?>/g, "");
    return out.replace(/<(?:[^>"']|"[^"]*"|'[^']*')*>/g, stripTag);
  }

  function escapeText(input) {
    return String(input == null ? "" : input)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function html(input) {
    var scrubbed = stripRemote(input);
    var shared = root.GBSanitize;
    if (shared && typeof shared.html === "function") {
      return stripRemote(shared.html(scrubbed));
    }
    // No shared sanitizer: render nothing interactive rather than risk a load.
    return escapeText(scrubbed);
  }

  // For values placed inside an inline `onclick="fn('…')"` handler: HTML-first
  // escaping prevents markup injection, then JS string escaping keeps an
  // apostrophe in the value from terminating the handler early.
  function escapeJsAttr(input) {
    var value = escapeText(input).replace(/&#39;/g, "'");
    return value.replace(/\\/g, "\\\\").replace(/'/g, "\\'");
  }

  function cleanElement(element) {
    if (!element || typeof element.innerHTML !== "string") return false;
    var clean = html(element.innerHTML);
    if (clean === element.innerHTML) return false;
    element.innerHTML = clean;
    return true;
  }

  // htmx exposes the not-yet-inserted markup as `serverResponse` on
  // `htmx:beforeSwap`. Cleaning there stops the browser from ever requesting a
  // remote asset; the afterSwap pass below is a second line of defence.
  function cleanFragment(fragment) {
    var out = String(fragment == null ? "" : fragment);
    var openRe = /<div\b(?=[^>]*\bclass\s*=\s*"[^"]*\bmail-body\b[^"]*")[^>]*>/gi;
    var pieces = [];
    var cursor = 0;
    var open;
    while ((open = openRe.exec(out)) !== null) {
      var start = open.index + open[0].length;
      var depth = 1;
      var tagRe = /<\/?div\b[^>]*>/gi;
      tagRe.lastIndex = start;
      var tag;
      var end = -1;
      while (depth > 0 && (tag = tagRe.exec(out)) !== null) {
        if (/^<\/div/i.test(tag[0])) depth--;
        else if (!/\/\s*>$/.test(tag[0])) depth++;
        if (depth === 0) end = tag.index;
      }
      if (end < 0) continue;
      pieces.push(out.slice(cursor, start));
      pieces.push(stripRemote(out.slice(start, end)));
      cursor = end;
      openRe.lastIndex = end;
    }
    if (!pieces.length) return out;
    pieces.push(out.slice(cursor));
    return pieces.join("");
  }

  function hasToken(output, token) {
    var escaped = String(token).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    return new RegExp(escaped, "i").test(output);
  }

  function selfTest() {
    var result = {
      version: VERSION,
      total: HOSTILE_CORPUS.length,
      failures: [],
      pass: false,
    };
    HOSTILE_CORPUS.forEach(function (item) {
      var scrubbed = stripRemote(item.html);
      var cleaned = html(item.html);
      item.forbidden.forEach(function (token) {
        if (hasToken(scrubbed, token)) {
          result.failures.push(item.id + ": " + token + " survived stripRemote -> " + scrubbed);
        }
        if (hasToken(cleaned, token)) {
          result.failures.push(item.id + ": " + token + " survived html() -> " + cleaned);
        }
      });
    });
    result.pass = result.failures.length === 0;
    return result;
  }

  var API = {
    VERSION: VERSION,
    HOSTILE_CORPUS: HOSTILE_CORPUS,
    DROP_TAGS: DROP_TAGS,
    URI_ATTRS: URI_ATTRS,
    decodeEntities: decodeEntities,
    isExternalUrl: isExternalUrl,
    stripCss: stripCss,
    stripRemote: stripRemote,
    escapeText: escapeText,
    escapeJsAttr: escapeJsAttr,
    html: html,
    cleanElement: cleanElement,
    cleanFragment: cleanFragment,
    selfTest: selfTest,
    hookInstalled: false,
  };
  root.GBMailSanitize = API;

  // Every message body arrives through an htmx swap into #mail-content. Clean
  // before the browser paints it; the listeners are installed once per page.
  if (!API.hookInstalled && typeof document !== "undefined"
      && document.addEventListener && document.querySelectorAll) {
    API.hookInstalled = true;
    document.addEventListener("htmx:beforeSwap", function (evt) {
      if (!evt || !evt.detail || typeof evt.detail.serverResponse !== "string") return;
      evt.detail.serverResponse = cleanFragment(evt.detail.serverResponse);
    });
    document.addEventListener("htmx:afterSwap", function () {
      var bodies = document.querySelectorAll(".mail-body");
      for (var i = 0; i < bodies.length; i++) cleanElement(bodies[i]);
    });
  }
})(typeof window !== "undefined" ? window : this);
