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

  function lastBlock(content) {
    var text = String(content || "");
    var match = OPEN_FENCE.exec(text);
    if (!match) return null;
    var body = match[2].replace(/```[\s\S]*$/, "").trim();
    if (body.length < 12) return null;
    return { lang: match[1].toLowerCase(), body: body };
  }

  function documentFor(block) {
    var isSvg = block.lang === "svg" || /^\s*<svg[\s>]/i.test(block.body);
    if (!isSvg) return block.body;
    return (
      "<!doctype html><html><head><meta charset=\"utf-8\">" +
      "<style>html,body{margin:0;padding:8px;background:#fff}" +
      "svg{max-width:100%;height:auto;display:block}</style></head>" +
      "<body>" + block.body + "</body></html>"
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
      expand.textContent = box.classList.contains("expanded") ? "Collapse" : "Expand";
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
