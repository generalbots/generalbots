/* #1693/#2005 — live preview of what the model draws while it streams.
 *
 * #1693 previewed a fenced drawing by re-assigning iframe.srcdoc on every
 * chunk. That re-parsed the whole document from scratch on each frame, so the
 * preview flickered and only made sense once </svg> had arrived: a static
 * picture at the end, never a drawing being drawn.
 *
 * SVG now grows on a persistent surface owned by `chat-live-artifact-svg.js`
 * (tokenizer + incremental append + per-element animation). This module keeps
 * the block detection, the card chrome and the "Open in Canvas" handoff.
 *
 * HTML/JS artifacts keep the opaque sandboxed iframe from #1693 — they need
 * scripting and must never share the suite origin.
 */
"use strict";

window.GBLiveArtifact = window.GBLiveArtifact || (function () {
  // #2019 — models fence drawings as ```svg, ~~~svg, ```svg+xml or even a
  // language-less fence, and sometimes precede the element with an XML prolog
  // or a doctype. All of those must reach the live preview.
  var FENCE = /(```|~~~)\s*([a-z0-9+._-]*)\s*\r?\n([\s\S]*)$/i;
  var SVG_LANGS = ["svg", "svg+xml", "image/svg+xml"];
  var HTML_LANGS = ["html", "htm", "xml", "html5", "xhtml"];
  // Models often answer a drawing request with the bare element instead of a
  // fenced block. A message that is *only* an SVG is treated the same way.
  var BARE_SVG = /^\s*((?:<\?xml[\s\S]*?\?>\s*)?(?:<!doctype[^>]*>\s*)?<svg[\s>][\s\S]*<\/svg\s*>)\s*$/i;
  var BARE_OPEN_SVG = /^\s*((?:<\?xml[\s\S]*?\?>\s*)?(?:<!doctype[^>]*>\s*)?<svg[\s>][\s\S]*)$/i;

  function hasSvgRoot(body) {
    return /^\s*(?:<\?xml[\s\S]*?\?>\s*)?(?:<!doctype[^>]*>\s*)?<svg[\s>]/i.test(body);
  }

  function fenceLang(lang, body) {
    var normalized = (lang || "").toLowerCase();
    if (SVG_LANGS.indexOf(normalized) >= 0) return "svg";
    if (HTML_LANGS.indexOf(normalized) >= 0) return hasSvgRoot(body) ? "svg" : normalized;
    if (!normalized && hasSvgRoot(body)) return "svg";
    return normalized;
  }

  /* ---------------------------------------------------------------- blocks */

  // #2019 — models often answer a drawing request with the markup
  // HTML-escaped (`&lt;svg …&gt;…&lt;/svg&gt;`), because they "write code".
  // Detection has to see real tags or the reply stays plain text in the bubble.
  // Only unescape when the text really carries escaped markup, so ordinary
  // prose containing a literal `&lt;` is left alone.
  function unescapeMarkup(text) {
    return text
      .replace(/&lt;/gi, "<")
      .replace(/&gt;/gi, ">")
      .replace(/&quot;/gi, '"')
      .replace(/&#0*39;/g, "'")
      .replace(/&amp;/gi, "&");
  }

  function escapeMarkup(text) {
    return text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function detectionText(text) {
    return /&lt;\s*\/?\s*(svg|html|htm|xml|!doctype)/i.test(text)
      ? unescapeMarkup(text)
      : text;
  }

  // #2019 — a tool-call envelope may carry the drawing as a JSON string
  // (`{"svg": "<svg …>"}`). Unwrap the last SVG-bearing string before
  // detection so the payload does not print as JSON in the bubble.
  function findSvgString(value, depth) {
    if (value === null || value === undefined || depth > 4) return null;
    if (typeof value === "string") {
      return /<svg[\s>]/i.test(value) ? value : null;
    }
    if (Array.isArray(value)) {
      for (var i = value.length - 1; i >= 0; i -= 1) {
        var fromArray = findSvgString(value[i], depth + 1);
        if (fromArray) return fromArray;
      }
      return null;
    }
    if (typeof value === "object") {
      var preferred = ["svg", "content", "html", "markup", "artifact", "result", "text", "output"];
      for (var k = 0; k < preferred.length; k += 1) {
        var hit = findSvgString(value[preferred[k]], depth + 1);
        if (hit) return hit;
      }
      for (var key in value) {
        if (preferred.indexOf(key) >= 0) continue;
        var nested = findSvgString(value[key], depth + 1);
        if (nested) return nested;
      }
    }
    return null;
  }

  function unwrapJsonEnvelope(text) {
    var trimmed = text.trim();
    if (!/^[[{]/.test(trimmed)) return null;
    try {
      return findSvgString(JSON.parse(trimmed), 0);
    } catch (e) {
      return null;
    }
  }

  // #2019 — "Here is your robot: <svg>…</svg> — enjoy!" must still preview.
  // The drawing wins when it outweighs the prose around it.
  function dominantSvg(text) {
    var start = text.search(/<svg[\s>]/i);
    if (start < 0) return null;
    var lower = text.toLowerCase();
    var close = lower.lastIndexOf("</svg");
    if (close < start) return null;
    var end = text.indexOf(">", close);
    if (end < 0) return null;
    var svg = text.slice(start, end + 1).trim();
    if (svg.length < 12) return null;
    var outside = (text.slice(0, start) + text.slice(end + 1)).replace(/\s+/g, "");
    if (outside.length > svg.length) return null;
    return { lang: "svg", body: svg, closed: true };
  }

  function lastBlock(content) {
    var text = detectionText(String(content || ""));
    var unwrapped = unwrapJsonEnvelope(text);
    if (unwrapped) text = detectionText(unwrapped);

    var match = FENCE.exec(text);
    if (match) {
      var fence = match[1];
      var rest = match[3];
      var end = rest.indexOf(fence);
      var closed = end >= 0;
      var body = (closed ? rest.slice(0, end) : rest).trim();
      var lang = fenceLang(match[2], body);
      // A language-less fence is only a drawing when its body is an SVG; any
      // other bare fence stays ordinary markdown.
      if (body.length >= 12 && (match[2] !== "" || lang === "svg")) {
        return { lang: lang || "html", body: body, closed: closed };
      }
    }

    var bare = BARE_SVG.exec(text);
    if (bare && bare[1].trim().length >= 12) {
      return { lang: "svg", body: bare[1].trim(), closed: true };
    }
    // Still streaming a bare <svg> with no fence yet: preview it too, instead
    // of flashing the raw markup until </svg> lands.
    if (text.indexOf("```") < 0 && text.indexOf("~~~") < 0) {
      var open = BARE_OPEN_SVG.exec(text);
      if (open) {
        var partial = open[1].trim();
        if (partial.length >= 12) {
          return { lang: "svg", body: partial, closed: /<\/svg\s*>\s*$/i.test(partial) };
        }
      }
    }

    var dominant = dominantSvg(text);
    if (dominant) return dominant;
    return null;
  }

  // #2019 — remove whichever variant was detected (raw, escaped, fenced,
  // prolog-prefixed or prose-wrapped) from the bubble text.
  function strip(content) {
    var text = String(content || "");
    var block = lastBlock(text);
    if (!block) return text;

    // A JSON tool envelope carries the drawing as data, not as prose: drop the
    // whole envelope instead of leaving `{"svg":""}` behind.
    if (unwrapJsonEnvelope(text)) return "";

    var out = text;
    // The detected body may include the XML prolog/doctype, so remove it
    // verbatim first (raw or HTML-escaped form).
    var bodyVariants = [block.body, escapeMarkup(block.body)];
    for (var i = 0; i < bodyVariants.length; i += 1) {
      var at = out.indexOf(bodyVariants[i]);
      if (at >= 0) {
        out = out.slice(0, at) + out.slice(at + bodyVariants[i].length);
        break;
      }
    }

    var fenceRe = /(```|~~~)[^\n]*\n([\s\S]*?)\1/g;
    out = out.replace(fenceRe, function (all, marker, body) {
      var decoded = unescapeMarkup(body).trim();
      return hasSvgRoot(decoded) || fenceLang("", decoded) === "svg" ? "" : all;
    });
    // Still-streaming fence with no closing marker yet.
    out = out.replace(/(```|~~~)[^\n]*\n([\s\S]*)$/, function (all, marker, body) {
      var decoded = unescapeMarkup(body).trim();
      return hasSvgRoot(decoded) ? "" : all;
    });

    if (block.lang === "svg") {
      out = out.replace(/&lt;\s*svg[\s>][\s\S]*?&lt;\s*\/\s*svg\s*&gt;/gi, "");
      out = out.replace(/<svg[\s>][\s\S]*?<\/svg\s*>/gi, "");
      out = out.replace(/&lt;\s*svg[\s>][\s\S]*$/i, "");
      out = out.replace(/<svg[\s>][\s\S]*$/i, "");
      // Empty fences and a dangling opening marker left by the removal.
      out = out.replace(/(```|~~~)[^\n]*\n\s*\1/g, "");
      out = out.replace(/(```|~~~)\s*[a-z0-9+._-]*\s*$/i, "");
      // Orphaned prolog/doctype left by the removed drawing.
      out = out.replace(/<\?xml[\s\S]*?\?>/gi, "");
      out = out.replace(/<!doctype[^>]*>/gi, "");
    }
    return out.trim();
  }


  /* ---------------------------------------------------------------- surfaces */

  // #1693 — HTML/JS artifacts run in an opaque, sandboxed frame. They keep the
  // srcdoc reload: a generated app is only meaningful once its document is
  // complete, and it must never share the suite origin.
  function renderHtml(state, block) {
    if (!state.frame) {
      var frame = document.createElement("iframe");
      frame.className = "gb-live-artifact-frame";
      frame.setAttribute("referrerpolicy", "no-referrer");
      frame.setAttribute("loading", "lazy");
      frame.setAttribute("title", "Live generated preview");
      frame.setAttribute("sandbox", "allow-scripts allow-forms");
      state.body.appendChild(frame);
      state.frame = frame;
    }
    state.frame.hidden = false;
    state.host.hidden = true;
    state.badge.textContent = block.closed ? "ready" : "writing…";
    if (state.frame.dataset.doc === block.body) return;
    state.frame.dataset.doc = block.body;
    state.frame.srcdoc = block.body;
  }

  // #2006 — hand the drawing to the Canvas app. When the app is not mounted,
  // open it first and deliver the SVG once its module registers.
  function canvasReady() {
    return !!(window.GBLiveCanvas &&
      typeof window.GBLiveCanvas.isMounted === "function" &&
      window.GBLiveCanvas.isMounted());
  }

  function sendToCanvas(svg) {
    var detail = { svg: svg, title: "Chat drawing" };
    // The canvas module binds its listener when `08_live_canvas.js` runs, which
    // is *after* the window markup (and `#canvas-bg-svg`) is injected, so
    // `isMounted()` can be true with nothing listening yet. Delivering once
    // silently dropped the drawing. Deliver, then re-deliver until the
    // background layer actually holds shapes (#2144).
    function deliver() {
      var tries = 0;
      var timer = null;
      var stop = function () {
        if (timer) { clearInterval(timer); timer = null; }
      };
      var dispatch = function () {
        try {
          document.dispatchEvent(new CustomEvent("gb:canvas-set-background", {
            detail: detail,
          }));
        } catch (e) { /* CustomEvent unavailable */ }
      };
      var landed = function () {
        var layer = document.getElementById("canvas-bg-svg");
        return !!(layer && layer.childElementCount > 0);
      };
      dispatch();
      if (landed()) return;
      timer = setInterval(function () {
        tries += 1;
        dispatch();
        if (landed() || tries >= 12) stop();
      }, 250);
    }
    if (canvasReady()) { deliver(); return; }
    if (window.wm && typeof window.wm.openDeepLink === "function") {
      try {
        window.wm.openDeepLink("canvas");
      } catch (e) { deliver(); return; }
      var tries = 0;
      var timer = setInterval(function () {
        tries += 1;
        if (canvasReady()) {
          clearInterval(timer);
          deliver();
        } else if (tries > 40) {
          clearInterval(timer);
        }
      }, 250);
      return;
    }
    deliver();
  }

  /* -------------------------------------------------------------- downloads */

  // #2144 — the drawing has to leave chat as a file, not only through
  // Save-to-Drive. `js/artifact-download.js` serialises the live root (a DOM
  // closes whatever the model has not finished, so the *current* state of a
  // streaming drawing is always a valid document), paints the preview
  // background and rasterises PNG at 2x without any external library.
  function currentDrawing(box) {
    var state = box.__gbLiveSvg;
    var root = state && state.root;
    return root ? root.outerHTML || "" : "";
  }

  function downloadButton(box, format, label, title) {
    if (!window.GBArtifactExport) return document.createElement("span");
    return window.GBArtifactExport.drawingButton(
      format,
      function () { return currentDrawing(box); },
      {
        label: label,
        title: title,
        className: "gb-attachment-inline-btn gb-live-artifact-download",
      }
    );
  }

  /* ------------------------------------------------------------------ chrome */

  function build(container) {
    var box = document.createElement("div");
    box.className = "gb-live-artifact";
    var head = document.createElement("div");
    head.className = "gb-live-artifact-head";
    var label = document.createElement("span");
    label.className = "gb-live-artifact-title";
    label.textContent = "Live preview";
    head.appendChild(label);
    var badge = document.createElement("span");
    badge.className = "gb-live-artifact-badge";
    head.appendChild(badge);
    var send = document.createElement("button");
    send.type = "button";
    send.className = "gb-attachment-inline-btn gb-live-artifact-send";
    send.textContent = "Open in Canvas";
    send.addEventListener("click", function () {
      var state = box.__gbLiveSvg;
      var svg = state && state.root ? state.root.outerHTML : "";
      if (svg) sendToCanvas(svg);
    });
    head.appendChild(send);
    // #2144 — Download sits next to "Open in Canvas" so the drawing can leave
    // chat as a file even before the model finishes streaming.
    var downloadSvg = downloadButton(
      box, "svg", "SVG", "Download the drawing as SVG"
    );
    head.appendChild(downloadSvg);
    var downloadPng = downloadButton(
      box, "png", "PNG", "Download the drawing as PNG (2x)"
    );
    head.appendChild(downloadPng);
    var expand = document.createElement("button");
    expand.type = "button";
    expand.className = "gb-attachment-inline-btn";
    expand.textContent = "Expand";
    expand.setAttribute("aria-expanded", "false");
    // #2099 — expanding opens the preview in a modal layer instead of only
    // growing it inside the 80% bubble. The box itself is portaled, so the
    // streaming renderer keeps writing into the same node.
    expand.addEventListener("click", function () {
      if (window.GBArtifactOverlay) {
        window.GBArtifactOverlay.toggle(box, expand, { label: "Live preview" });
        return;
      }
      box.classList.toggle("expanded");
      expand.textContent = box.classList.contains("expanded")
        ? "Collapse" : "Expand";
    });
    head.appendChild(expand);
    var body = document.createElement("div");
    body.className = "gb-live-artifact-body";
    var host = document.createElement("div");
    host.className = "gb-live-artifact-svg";
    body.appendChild(host);
    box.appendChild(head);
    box.appendChild(body);
    box.__gbLiveSvg = {
      head: head, badge: badge, body: body, host: host, send: send,
      downloadSvg: downloadSvg, downloadPng: downloadPng,
      root: null, frame: null, applied: {}, containers: {}, parts: 0,
    };
    container.appendChild(box);
    // #2099 — the box can be portaled into the expand overlay, leaving the
    // message; keep a handle so the streaming render reuses it instead of
    // building a second preview inside the bubble.
    container.__gbLiveBox = box;
    return box;
  }

  function render(container, content) {
    if (!container) return;
    var block = lastBlock(content);
    var box = container.querySelector(".gb-live-artifact");
    if (!box && container.__gbLiveBox && container.__gbLiveBox.isConnected) {
      box = container.__gbLiveBox;
    }
    if (!block) {
      if (box) {
        if (window.GBArtifactOverlay && window.GBArtifactOverlay.isOpen(box)) {
          window.GBArtifactOverlay.close(box);
        }
        box.remove();
        if (container.__gbLiveBox === box) container.__gbLiveBox = null;
      }
      return;
    }
    if (!box) box = build(container);
    var state = box.__gbLiveSvg;
    if (!state) return;
    var isSvg = block.lang === "svg" || hasSvgRoot(block.body);
    if (state.frame) state.frame.hidden = isSvg;
    state.host.hidden = !isSvg;
    state.send.hidden = !isSvg;
    // A generated HTML app is not a drawing: there is no SVG to export, and
    // the opaque iframe cannot be serialised from the suite document.
    state.downloadSvg.hidden = !isSvg;
    state.downloadPng.hidden = !isSvg;
    if (!isSvg) {
      renderHtml(state, block);
      return;
    }
    if (window.GBLiveSvg) window.GBLiveSvg.render(state, block);
  }

  return { render: render, lastBlock: lastBlock, strip: strip };
})();
