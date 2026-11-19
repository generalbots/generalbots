"use strict";
/* Docs large .docx load path (#2533), viewport windowing.
 *
 * Pure range math + a mount plan. Only the visible block range is mounted and
 * only those blocks are contenteditable; the container never is. The mounted
 * count is bounded by the viewport (plus overscan), independent of document
 * size. Block ids are stable because they are derived from the block index. */
(function (window) {
  var NS = window.DocsLarge;
  if (!NS) return;
  var W = (NS.Window = NS.Window || {});
  W.ROW_HEIGHT = 28;
  W.OVERSCAN = 4;

  function visibleRange(scrollTop, viewportHeight, total, rowHeight, overscan) {
    var row = Math.max(1, Number(rowHeight) || W.ROW_HEIGHT);
    var over = Math.max(0, Number(overscan) || 0);
    var top = Math.max(0, Number(scrollTop) || 0);
    var height = Math.max(row, Number(viewportHeight) || row);
    var count = Math.max(0, Math.floor(Number(total) || 0));
    if (!count) return { start: 0, end: 0, count: 0 };
    var start = Math.max(0, Math.floor(top / row) - over);
    var span = Math.ceil(height / row) + over * 2;
    var end = Math.min(count, start + span);
    start = Math.max(0, Math.min(start, end));
    return { start: start, end: end, count: end - start };
  }
  W.visibleRange = visibleRange;

  function mountPlan(sizeMb, range) {
    var blocks = NS.Fixture.blocksInRange(sizeMb, range.start, range.end);
    return {
      containerEditable: false,
      start: range.start,
      end: range.end,
      total: NS.Fixture.blockCount(sizeMb),
      blocks: blocks.map(function (block) {
        return { id: block.id, index: block.index, text: block.text, editable: true };
      })
    };
  }
  W.mountPlan = mountPlan;

  /* Render into a host with createElement + textContent only. */
  function mount(host, plan, editable) {
    if (!host) return plan;
    host.textContent = "";
    var enabled = editable !== false;
    plan.blocks.forEach(function (block) {
      var node = NS.el("div", "gb-large-block");
      node.setAttribute("data-block-id", block.id);
      node.setAttribute("contenteditable", enabled && block.editable ? "true" : "false");
      node.textContent = block.text;
      host.appendChild(node);
    });
    return plan;
  }
  W.mount = mount;

  W.label = function (plan, tr) {
    return tr("docs-large-window", "Mounted range: { $s }–{ $e } of { $t }",
      { s: plan.start, e: plan.end, t: plan.total });
  };
})(typeof window !== "undefined" ? window : globalThis);
