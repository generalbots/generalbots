"use strict";
/* Docs large .docx load path (#2533), deterministic fixture.
 *
 * The "5/10/20 MB" sizes are SIMULATED by block count (nominal 1 KiB/block):
 * the manifest is a tiny descriptor and block text is generated lazily, so no
 * huge string is ever allocated. This is a local estimate input to the window,
 * save and baseline views — not a real DOCX import. */
(function (window) {
  var NS = window.DocsLarge;
  if (!NS) return;
  var F = (NS.Fixture = NS.Fixture || {});

  F.NOMINAL_BLOCK_BYTES = 1024;
  F.WORDS = [
    "report", "quarter", "figure", "appendix", "section", "review", "table",
    "summary", "baseline", "measure", "render", "window", "block", "draft"
  ];

  function bytesForMb(sizeMb) { return Math.round(Number(sizeMb) * 1024 * 1024); }
  F.bytesForMb = bytesForMb;

  function blockCount(sizeMb) {
    return Math.ceil(bytesForMb(sizeMb) / F.NOMINAL_BLOCK_BYTES);
  }
  F.blockCount = blockCount;

  function blockId(index) {
    var n = Math.max(0, Math.floor(Number(index) || 0));
    var tail = String(n);
    while (tail.length < 6) tail = "0" + tail;
    return "blk-" + tail;
  }
  F.blockId = blockId;

  /* Short deterministic text. The nominal size is accounted for by block
   * count, not by materialising 1 KiB per block. */
  function blockText(index) {
    var n = Math.max(0, Math.floor(Number(index) || 0));
    return "Paragraph " + (n + 1) + " — " + F.WORDS[n % F.WORDS.length];
  }
  F.blockText = blockText;

  function manifest(sizeMb) {
    var mb = Number(sizeMb) || 0;
    return {
      sizeMb: mb,
      blockCount: blockCount(mb),
      nominalBytes: bytesForMb(mb),
      nominalBlockBytes: F.NOMINAL_BLOCK_BYTES
    };
  }
  F.manifest = manifest;

  /* Blocks are produced only for an explicit index range (the visible
   * window). This is what keeps memory bounded as the document grows. */
  function blocksInRange(sizeMb, start, end) {
    var total = blockCount(sizeMb);
    var from = Math.max(0, Math.floor(start) || 0);
    var to = Math.min(total, Math.floor(end) || 0);
    var list = [];
    for (var index = from; index < to; index += 1) {
      list.push({ id: blockId(index), index: index, text: blockText(index) });
    }
    return list;
  }
  F.blocksInRange = blocksInRange;
})(typeof window !== "undefined" ? window : globalThis);
