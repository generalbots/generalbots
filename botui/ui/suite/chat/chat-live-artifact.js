/* #1693 — live preview for SVG/HTML that the model draws while streaming.
 *
 * A fenced ```svg / ```html block is rendered into a sandboxed iframe that is
 * refreshed as each chunk arrives, so a chart or drawing appears progressively
 * instead of only after the turn completes. The frame is opaque (no
 * allow-same-origin): the generated document cannot read the suite DOM,
 * cookies or storage. Nothing is fetched while no block is present.
 */
"use strict";

window.GBLiveArtifact = window.GBLiveArtifact || (function () {
  var OPEN_FENCE = /```(svg|html|htm|xml)\s*\n([\s\S]*)$/i;
  // Models often answer a drawing request with the bare element instead of a
  // fenced block. A message that is *only* an <svg> is treated the same way;
  // an svg embedded in prose stays inline.
  var BARE_SVG = /^\s*(<svg[\s>][\s\S]*<\/svg>)\s*$/i;

  function lastBlock(content) {
    var text = String(content || "");
    var bare = BARE_SVG.exec(text);
    if (bare && bare[1].trim().length >= 12) {
      return { lang: "svg", body: bare[1].trim() };
    }
    var match = OPEN_FENCE.exec(text);
    if (!match) return null;
    var body = match[2].replace(/```[\s\S]*$/, "").trim();
    if (body.length < 12) return null;
    return { lang: match[1].toLowerCase(), body: body };
  }

  // An <svg> without a viewBox cannot be scaled: shrinking the element clips
  // the drawing instead of resizing it (a 512px circle previewed as its top
  // left quarter). Numeric width/height give the same box, so derive the
  // missing viewBox from them.
  function withViewBox(svg) {
    var open = /^<svg\b[^>]*>/i.exec(svg);
    if (!open) return svg;
    var tag = open[0];
    if (/\bviewbox\s*=/i.test(tag)) return svg;
    var w = /\bwidth\s*=\s*["']?([\d.]+)(?:px)?["']?/i.exec(tag);
    var h = /\bheight\s*=\s*["']?([\d.]+)(?:px)?["']?/i.exec(tag);
    if (!w || !h) return svg;
    var patched = tag.replace(
      /^<svg\b/i,
      '<svg viewBox="0 0 ' + w[1] + " " + h[1] + '"'
    );
    return patched + svg.slice(tag.length);
  }

  function documentFor(block) {
    var isSvg = block.lang === "svg" || /^\s*<svg[\s>]/i.test(block.body);
    if (!isSvg) return block.body;
    // The frame is a fixed 280px tall window; the drawing is scaled to fit it
    // instead of overflowing into a scrollbar (a 512px circle used to show
    // only its top edge). "Expand" gives the full-height view.
    return (
      "<!doctype html><html><head><meta charset=\"utf-8\">" +
      "<style>html,body{margin:0;height:100%;background:#fff}" +
      "body{display:flex;align-items:center;justify-content:center;" +
      "padding:8px;box-sizing:border-box}" +
      "svg{max-width:100%;max-height:100%;width:auto;height:auto;" +
      "display:block}</style></head>" +
      "<body>" + withViewBox(block.body) + "</body></html>"
    );
  }

  function build(container) {
    var box = document.createElement("div");
    box.className = "gb-live-artifact";
    var head = document.createElement("div");
    head.className = "gb-live-artifact-head";
    var label = document.createElement("span");
    label.className = "gb-live-artifact-title";
    label.textContent = "Live preview";
    head.appendChild(label);
    var expand = document.createElement("button");
    expand.type = "button";
    expand.className = "gb-attachment-inline-btn";
    expand.textContent = "Expand";
    expand.addEventListener("click", function () {
      box.classList.toggle("expanded");
      var expanded = box.classList.contains("expanded");
      expand.textContent = expanded ? "Collapse" : "Expand";
      // Expanded previews widen to the conversation width and stay pinned to
      // the top of the scrolling transcript instead of scrolling away.
      if (expanded) {
        box.style.position = "sticky";
        box.style.top = "0";
        box.style.zIndex = "5";
        box.style.width = "100%";
        box.style.maxWidth = "100%";
      } else {
        box.style.position = "";
        box.style.top = "";
        box.style.zIndex = "";
        box.style.width = "";
        box.style.maxWidth = "";
      }
    });
    head.appendChild(expand);
    var frame = document.createElement("iframe");
    frame.className = "gb-live-artifact-frame";
    frame.setAttribute("referrerpolicy", "no-referrer");
    frame.setAttribute("loading", "lazy");
    frame.setAttribute("title", "Live generated preview");
    var body = document.createElement("div");
    body.className = "gb-live-artifact-body";
    body.appendChild(frame);
    box.appendChild(head);
    box.appendChild(body);
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
    var frame = box.querySelector(".gb-live-artifact-frame");
    if (!frame) return;
    // HTML can carry charts that need scripts; the sandbox keeps the frame
    // in an opaque origin. Static SVG needs no script at all.
    var needsScripts = block.lang !== "svg" && !/^\s*<svg[\s>]/i.test(block.body);
    frame.setAttribute("sandbox", needsScripts ? "allow-scripts allow-forms" : "");
    var next = documentFor(block);
    if (frame.dataset.doc === next) return;
    frame.dataset.doc = next;
    frame.srcdoc = next;
  }

  return { render: render, lastBlock: lastBlock };
})();
