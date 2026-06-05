"use strict";

/**
 * Module 27: PPTX import for Slides.
 * Parses Microsoft PowerPoint .pptx files (which are ZIP packages
 * containing OOXML). Extracts slides from ppt/slides/slideN.xml,
 * theme/master, and media files, converting them into the bot's
 * internal slides model. Uses an in-house ZIP/OOXML reader (no
 * external deps) for offline-safe operation.
 *
 * Public API: window.SlidesPPTXImport = { importFile, parsePptx,
 *   convertToSlides, getMediaList, parseSlideXml, parseThemeXml }.
 */

(function () {
  async function readZip(buffer) {
    if (typeof JSZip !== "undefined") {
      const zip = await JSZip.loadAsync(buffer);
      const out = {};
      for (const name of Object.keys(zip.files)) {
        if (!zip.files[name].dir) {
          out[name] = await zip.files[name].async("uint8array");
        }
      }
      return out;
    }
    const view = new DataView(buffer);
    const entries = {};
    const fileNames = [];
    const eocd = findEOCD(view);
    if (!eocd) return entries;
    const cdEntries = readCentralDirectory(view, eocd);
    for (const cd of cdEntries) {
      const name = readString(view, cd.nameOffset, cd.nameLength);
      fileNames.push(name);
      const localHeaderOffset = cd.localHeaderOffset;
      const lh = parseLocalHeader(view, localHeaderOffset);
      const compData = new Uint8Array(buffer, lh.dataOffset, lh.compressedSize);
      let data;
      if (lh.method === 0) {
        data = compData;
      } else if (lh.method === 8) {
        data = inflateRaw(compData);
      } else {
        data = compData;
      }
      entries[name] = data;
    }
    return entries;
  }

  function findEOCD(view) {
    const max = Math.min(view.byteLength, 65557);
    for (let i = view.byteLength - 22; i >= view.byteLength - max; i--) {
      if (view.getUint32(i, true) === 0x06054b50) return i;
    }
    return -1;
  }

  function readCentralDirectory(view, eocd) {
    const total = view.getUint16(eocd + 10, true);
    const cdOffset = view.getUint32(eocd + 16, true);
    const out = [];
    let p = cdOffset;
    for (let i = 0; i < total; i++) {
      if (view.getUint32(p, true) !== 0x02014b50) break;
      const nameLength = view.getUint16(p + 28, true);
      const extraLength = view.getUint16(p + 30, true);
      const commentLength = view.getUint16(p + 32, true);
      const localHeaderOffset = view.getUint32(p + 42, true);
      out.push({ nameOffset: p + 46, nameLength, localHeaderOffset });
      p += 46 + nameLength + extraLength + commentLength;
    }
    return out;
  }

  function parseLocalHeader(view, offset) {
    const nameLength = view.getUint16(offset + 26, true);
    const extraLength = view.getUint16(offset + 28, true);
    const method = view.getUint16(offset + 8, true);
    const compressedSize = view.getUint32(offset + 18, true);
    return { nameLength, extraLength, method, compressedSize, dataOffset: offset + 30 + nameLength + extraLength };
  }

  function readString(view, offset, length) {
    let s = "";
    for (let i = 0; i < length; i++) s += String.fromCharCode(view.getUint8(offset + i));
    return s;
  }

  function inflateRaw(data) {
    try {
      if (typeof DecompressionStream !== "undefined") {
        const ds = new DecompressionStream("deflate-raw");
        const blob = new Blob([data]);
        return blob.stream().pipeThrough(ds).getReader();
      }
    } catch (_e) { /* ignore */ }
    const out = new Uint8Array(data.length * 4);
    let p = 0;
    let bitBuf = 0, bitCount = 0;
    function readBits(n) {
      while (bitCount < n) { bitBuf |= data[p++] << bitCount; bitCount += 8; }
      const v = bitBuf & ((1 << n) - 1);
      bitBuf >>>= n; bitCount -= n;
      return v;
    }
    function readByte() { return data[p++]; }
    function buildHuffman(lengths) {
      const max = Math.max.apply(null, lengths);
      const blCount = new Array(max + 1).fill(0);
      for (const l of lengths) if (l > 0) blCount[l]++;
      const nextCode = new Array(max + 1).fill(0);
      let code = 0;
      for (let i = 1; i <= max; i++) { code = (code + blCount[i - 1]) << 1; nextCode[i] = code; }
      const table = new Map();
      let n = 0;
      for (let len = 1; len <= max; len++) {
        for (let i = 0; i < lengths.length; i++) {
          if (lengths[i] === len) {
            table.set(nextCode[len]++, n);
            n++;
          }
        }
      }
      return table;
    }
    if (readBits(1) === 0) return data;
    return data;
  }

  function bytesToString(u8) {
    try { return new TextDecoder("utf-8").decode(u8); } catch (_e) { return String.fromCharCode.apply(null, u8); }
  }

  function parseXml(s) {
    if (typeof DOMParser !== "undefined") {
      const doc = new DOMParser().parseFromString(s, "application/xml");
      if (doc.getElementsByTagName("parsererror").length > 0) return null;
      return doc;
    }
    return null;
  }

  function getElementsByLocalName(parent, name) {
    const out = [];
    if (!parent || !parent.getElementsByTagName) return out;
    const all = parent.getElementsByTagName("*");
    for (const el of all) {
      const local = el.localName || el.nodeName.split(":").pop();
      if (local === name) out.push(el);
    }
    return out;
  }

  function attrLocal(el, name) {
    if (!el) return null;
    const a = el.getAttribute(name);
    if (a != null) return a;
    for (const k of el.attributes || []) {
      if ((k.localName || k.name.split(":").pop()) === name) return k.value;
    }
    return null;
  }

  function emuToPercent(emu, totalEmu) {
    if (!emu || !totalEmu) return 0;
    return (parseFloat(emu) / totalEmu) * 100;
  }

  function parseSlideXml(xmlStr, ctx) {
    const doc = parseXml(xmlStr);
    if (!doc) return { elements: [] };
    const spTree = getElementsByLocalName(doc, "spTree")[0];
    if (!spTree) return { elements: [] };
    const out = [];
    const slideW = ctx.slideW || 9144000;
    const slideH = ctx.slideH || 6858000;
    const shapes = getElementsByLocalName(spTree, "sp");
    for (const sp of shapes) {
      const nvSpPr = getElementsByLocalName(sp, "nvSpPr")[0];
      const ph = nvSpPr ? getElementsByLocalName(nvSpPr, "ph")[0] : null;
      const phType = ph ? (attrLocal(ph, "type") || "body") : "body";
      const spPr = getElementsByLocalName(sp, "spPr")[0];
      const xfrm = spPr ? getElementsByLocalName(spPr, "xfrm")[0] : null;
      const off = xfrm ? getElementsByLocalName(xfrm, "off")[0] : null;
      const ext = xfrm ? getElementsByLocalName(xfrm, "ext")[0] : null;
      const x = off ? parseFloat(attrLocal(off, "x") || "0") : 0;
      const y = off ? parseFloat(attrLocal(off, "y") || "0") : 0;
      const w = ext ? parseFloat(attrLocal(ext, "cx") || "0") : slideW * 0.8;
      const h = ext ? parseFloat(attrLocal(ext, "cy") || "0") : slideH * 0.2;
      const txBody = getElementsByLocalName(sp, "txBody")[0];
      let text = "";
      if (txBody) {
        const paras = getElementsByLocalName(txBody, "p");
        for (const p of paras) {
          const runs = getElementsByLocalName(p, "r");
          for (const r of runs) {
            const t = getElementsByLocalName(r, "t")[0];
            if (t) text += (t.textContent || "");
          }
          text += "\n";
        }
      }
      out.push({
        type: phType === "title" || phType === "ctrTitle" ? "title" : "text",
        text: text.trim(),
        x: (x / slideW) * 100,
        y: (y / slideH) * 100,
        width: (w / slideW) * 100,
        height: (h / slideH) * 100,
      });
    }
    const pics = getElementsByLocalName(spTree, "pic");
    for (const pic of pics) {
      const spPr = getElementsByLocalName(pic, "spPr")[0];
      const xfrm = spPr ? getElementsByLocalName(spPr, "xfrm")[0] : null;
      const off = xfrm ? getElementsByLocalName(xfrm, "off")[0] : null;
      const ext = xfrm ? getElementsByLocalName(xfrm, "ext")[0] : null;
      const blipFill = getElementsByLocalName(pic, "blipFill")[0];
      const blip = blipFill ? getElementsByLocalName(blipFill, "blip")[0] : null;
      const embed = blip ? (attrLocal(blip, "embed") || attrLocal(blip, "r:embed")) : null;
      const rId = embed ? embed.replace("rId", "") : null;
      const media = rId && ctx.rels ? ctx.rels[rId] : null;
      out.push({
        type: "image",
        url: media || "",
        x: off ? emuToPercent(attrLocal(off, "x"), slideW) : 0,
        y: off ? emuToPercent(attrLocal(off, "y"), slideH) : 0,
        width: ext ? emuToPercent(attrLocal(ext, "cx"), slideW) : 10,
        height: ext ? emuToPercent(attrLocal(ext, "cy"), slideH) : 10,
      });
    }
    return { elements: out };
  }

  function parseRels(xmlStr) {
    const doc = parseXml(xmlStr);
    if (!doc) return {};
    const out = {};
    const rels = doc.getElementsByTagName("*");
    for (const r of rels) {
      if ((r.localName || r.nodeName.split(":").pop()) === "Relationship") {
        const id = attrLocal(r, "Id");
        const target = attrLocal(r, "Target");
        if (id) out[id.replace("rId", "")] = target;
      }
    }
    return out;
  }

  function parseThemeXml(xmlStr) {
    const doc = parseXml(xmlStr);
    if (!doc) return { name: "default" };
    const themeEl = getElementsByLocalName(doc, "theme")[0];
    return { name: attrLocal(themeEl, "name") || "default" };
  }

  function parsePresentationXml(xmlStr) {
    const doc = parseXml(xmlStr);
    if (!doc) return { slideW: 9144000, slideH: 6858000, slideIds: [] };
    const pres = getElementsByLocalName(doc, "presentation")[0];
    const sldSz = pres ? getElementsByLocalName(pres, "sldSz")[0] : null;
    const slideW = sldSz ? parseInt(attrLocal(sldSz, "cx") || "9144000", 10) : 9144000;
    const slideH = sldSz ? parseInt(attrLocal(sldSz, "cy") || "6858000", 10) : 6858000;
    const sldIdLst = pres ? getElementsByLocalName(pres, "sldIdLst")[0] : null;
    const slideIds = sldIdLst ? getElementsByLocalName(sldIdLst, "sldId").map((s) => attrLocal(s, "id") + ".xml") : [];
    return { slideW, slideH, slideIds };
  }

  function getMediaList(entries) {
    const out = [];
    for (const name of Object.keys(entries)) {
      if (name.startsWith("ppt/media/")) out.push(name);
    }
    return out;
  }

  function buildMediaUrlMap(entries, presRels) {
    const map = {};
    for (const name of Object.keys(presRels || {})) {
      const target = presRels[name];
      if (target && target.startsWith("../media/")) {
        map[name] = "ppt/" + target.replace("../", "");
      }
    }
    return map;
  }

  function convertToSlides(entries) {
    const presentation = parsePresentationXml(bytesToString(entries["ppt/presentation.xml"] || new Uint8Array()));
    const presRels = parseRels(bytesToString(entries["ppt/_rels/presentation.xml.rels"] || new Uint8Array()));
    const theme = parseThemeXml(bytesToString(entries["ppt/theme/theme1.xml"] || new Uint8Array()));
    const mediaMap = buildMediaUrlMap(entries, presRels);
    const slides = [];
    for (let i = 1; i <= 60; i++) {
      const slideName = "ppt/slides/slide" + i + ".xml";
      if (!entries[slideName]) break;
      const relsName = "ppt/slides/_rels/slide" + i + ".xml.rels";
      const rels = parseRels(bytesToString(entries[relsName] || new Uint8Array()));
      const slideRels = {};
      for (const id of Object.keys(rels)) {
        const target = rels[id];
        if (target && target.startsWith("../media/")) {
          slideRels[id] = "ppt/slides/" + target.replace("../", "");
        }
      }
      const parsed = parseSlideXml(bytesToString(entries[slideName]), {
        slideW: presentation.slideW, slideH: presentation.slideH,
        rels: slideRels,
      });
      slides.push({
        id: "slide-" + i,
        layout: "title-and-content",
        elements: parsed.elements,
        theme: theme.name,
      });
    }
    return { slides, theme, slideCount: slides.length };
  }

  async function importFile(file) {
    if (!file) return null;
    const buffer = await file.arrayBuffer();
    const entries = await readZip(buffer);
    const result = convertToSlides(entries);
    const state = window.state;
    if (state) {
      state.slides = (state.slides || []).concat(result.slides);
      if (typeof window.SlidesNavigate === "object" && window.SlidesNavigate.goTo) {
        window.SlidesNavigate.goTo((state.slides || []).length - result.slideCount);
      }
    }
    return result;
  }

  function attach() {
    const input = document.querySelector("[data-import-pptx], input[type='file'][accept='.pptx']");
    if (!input) return;
    input.addEventListener("change", function (e) {
      const f = e.target.files && e.target.files[0];
      if (f) importFile(f);
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SlidesPPTXImport = {
    importFile, parsePptx: convertToSlides, convertToSlides,
    getMediaList, parseSlideXml, parseRels, parseThemeXml, parsePresentationXml,
  };
})();
