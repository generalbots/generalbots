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
  var OPEN_FENCE = /```(svg|html|htm|xml)\s*\n([\s\S]*)$/i;
  // Models often answer a drawing request with the bare element instead of a
  // fenced block. A message that is *only* an <svg> is treated the same way.
  var BARE_SVG = /^\s*(<svg[\s>][\s\S]*<\/svg>)\s*$/i;
  var BARE_OPEN_SVG = /^\s*(<svg[\s>][\s\S]*)$/i;

  /* ---------------------------------------------------------------- blocks */

  function lastBlock(content) {
    var text = String(content || "");
    var bare = BARE_SVG.exec(text);
    if (bare && bare[1].trim().length >= 12) {
      return { lang: "svg", body: bare[1].trim(), closed: true };
    }
    var match = OPEN_FENCE.exec(text);
    if (match) {
      var rest = match[2];
      var end = rest.indexOf("```");
      var closed = end >= 0;
      var body = (closed ? rest.slice(0, end) : rest).trim();
      if (body.length < 12) return null;
      return { lang: match[1].toLowerCase(), body: body, closed: closed };
    }
    // Still streaming a bare <svg> with no fence yet: preview it too, instead
    // of flashing the raw markup until </svg> lands.
    if (text.indexOf("```") < 0) {
      var open = BARE_OPEN_SVG.exec(text);
      if (open) {
        var partial = open[1].trim();
        if (partial.length >= 12) {
          return { lang: "svg", body: partial, closed: /<\/svg\s*>\s*$/i.test(partial) };
        }
      }
    }
    return null;
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
    function deliver() {
      try {
        document.dispatchEvent(new CustomEvent("gb:canvas-set-background", {
          detail: detail,
        }));
      } catch (e) { /* CustomEvent unavailable */ }
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
    var expand = document.createElement("button");
    expand.type = "button";
    expand.className = "gb-attachment-inline-btn";
    expand.textContent = "Expand";
    // Expanded previews widen to the conversation width and stay pinned to the
    // top of the scrolling transcript instead of scrolling away.
    expand.addEventListener("click", function () {
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
      root: null, frame: null, applied: {}, containers: {}, parts: 0,
    };
    container.appendChild(box);
    return box;
  }

  function render(container, content) {
    if (!container) return;
    var block = lastBlock(content);
    var box = container.querySelector(".gb-live-artifact");
    if (!block) {
      if (box) box.remove();
      return;
    }
    if (!box) box = build(container);
    var state = box.__gbLiveSvg;
    if (!state) return;
    var isSvg = block.lang === "svg" || /^\s*<svg[\s>]/i.test(block.body);
    if (state.frame) state.frame.hidden = isSvg;
    state.host.hidden = !isSvg;
    state.send.hidden = !isSvg;
    if (!isSvg) {
      renderHtml(state, block);
      return;
    }
    if (window.GBLiveSvg) window.GBLiveSvg.render(state, block);
  }

  return { render: render, lastBlock: lastBlock };
})();
