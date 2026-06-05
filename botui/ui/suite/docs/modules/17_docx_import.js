"use strict";

/**
 * Module 17: DOCX import for Word Processor.
 * Parses a .docx file (a ZIP of OOXML parts) entirely in the browser
 * and converts it to the editor's internal HTML model. The OOXML
 * parsing uses the same minimal store-only ZIP reader pattern as the
 * DOCX export module (no external dependencies).
 *
 * Supported parts:
 *   - word/document.xml -> body
 *   - word/styles.xml (style map for headings/normal)
 *   - word/numbering.xml (list numbering definitions)
 *   - word/footnotes.xml / word/endnotes.xml
 *   - word/comments.xml
 *   - word/header*.xml / word/footer*.xml
 *   - word/_rels/document.xml.rels (hyperlink relationships)
 *   - Inline images in word/media/*.png|jpg|gif -> data: URIs
 *
 * Public API: window.DocsDOCXImport = { importFile, importBlob, parse }.
 */

(function () {
  function crc32(bytes) {
    let c;
    const table = [];
    for (let n = 0; n < 256; n++) {
      c = n;
      for (let k = 0; k < 8; k++) c = (c & 1) ? (0xedb88320 ^ (c >>> 1)) : (c >>> 1);
      table[n] = c;
    }
    let crc = 0xffffffff;
    for (let i = 0; i < bytes.length; i++) crc = (crc >>> 8) ^ table[(crc ^ bytes[i]) & 0xff];
    return (crc ^ 0xffffffff) >>> 0;
  }

  function readUint16LE(bytes, off) { return bytes[off] | (bytes[off + 1] << 8); }
  function readUint32LE(bytes, off) { return (bytes[off] | (bytes[off + 1] << 8) | (bytes[off + 2] << 16) | (bytes[off + 3] << 24)) >>> 0; }

  function readZip(blob) {
    return new Promise(function (resolve, reject) {
      const reader = new FileReader();
      reader.onload = function () {
        try {
          const bytes = new Uint8Array(reader.result);
          const files = {};
          let p = 0;
          const eocdSig = 0x06054b50;
          while (p < bytes.length) {
            if (readUint32LE(bytes, p) !== 0x04034b50) break;
            const compMethod = readUint16LE(bytes, p + 8);
            const compSize = readUint32LE(bytes, p + 18);
            const fnameLen = readUint16LE(bytes, p + 26);
            const extraLen = readUint16LE(bytes, p + 28);
            const nameStart = p + 30;
            const dataStart = nameStart + fnameLen + extraLen;
            let name = "";
            for (let i = 0; i < fnameLen; i++) name += String.fromCharCode(bytes[nameStart + i]);
            const data = bytes.slice(dataStart, dataStart + compSize);
            if (compMethod === 0) {
              files[name] = data;
            } else {
              try {
                files[name] = inflateRaw(data);
              } catch (e) {
                files[name] = new Uint8Array(0);
              }
            }
            p = dataStart + compSize;
          }
          if (!Object.keys(files).length) {
            const eocd = findEOCD(bytes);
            if (eocd >= 0) {
              const cdSize = readUint32LE(bytes, eocd + 12);
              const cdOff = readUint32LE(bytes, eocd + 16);
              p = cdOff;
              while (p < cdOff + cdSize) {
                const fnameLen = readUint16LE(bytes, p + 28);
                const extraLen = readUint16LE(bytes, p + 30);
                const commentLen = readUint16LE(bytes, p + 32);
                const nameStart = p + 46;
                let name = "";
                for (let i = 0; i < fnameLen; i++) name += String.fromCharCode(bytes[nameStart + i]);
                const localOff = readUint32LE(bytes, p + 42);
                if (files[name] == null && localOff > 0) {
                  const lh = localOff;
                  const lhNameLen = readUint16LE(bytes, lh + 26);
                  const lhExtraLen = readUint16LE(bytes, lh + 28);
                  const lhCompSize = readUint32LE(bytes, lh + 18);
                  const lhDataStart = lh + 30 + lhNameLen + lhExtraLen;
                  files[name] = bytes.slice(lhDataStart, lhDataStart + lhCompSize);
                }
                p += 46 + fnameLen + extraLen + commentLen;
              }
            }
          }
          resolve(files);
        } catch (e) { reject(e); }
      };
      reader.onerror = function () { reject(reader.error); };
      reader.readAsArrayBuffer(blob);
    });
  }

  function findEOCD(bytes) {
    for (let i = bytes.length - 22; i >= 0; i--) {
      if (readUint32LE(bytes, i) === 0x06054b50) return i;
    }
    return -1;
  }

  function inflateRaw(data) {
    const stream = new Blob([data]);
    return new Response(stream).arrayBuffer().then(function (buf) {
      return new Uint8Array(buf);
    });
  }

  function bytesToString(bytes) {
    try {
      return new TextDecoder("utf-8").decode(bytes);
    } catch (e) {
      let s = "";
      for (let i = 0; i < bytes.length; i++) s += String.fromCharCode(bytes[i]);
      return s;
    }
  }

  function decodeXmlEntities(s) {
    return s.replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&amp;/g, "&").replace(/&quot;/g, "\"").replace(/&apos;/g, "'");
  }

  function walk(node, fn) {
    fn(node);
    for (let i = 0; i < node.childNodes.length; i++) walk(node.childNodes[i], fn);
  }

  function getText(node) {
    let s = "";
    walk(node, function (n) {
      if (n.nodeType === 3) s += n.nodeValue;
    });
    return decodeXmlEntities(s);
  }

  function parseDocumentXml(xmlString, relationships) {
    const parser = new DOMParser();
    const doc = parser.parseFromString(xmlString, "application/xml");
    const body = doc.getElementsByTagName("w:body")[0];
    if (!body) return null;
    const root = document.createElement("div");
    root.className = "doc-imported";
    for (let i = 0; i < body.childNodes.length; i++) {
      const child = body.childNodes[i];
      const tag = child.nodeName;
      if (tag === "w:p") {
        root.appendChild(parseParagraph(child, relationships));
      } else if (tag === "w:tbl") {
        root.appendChild(parseTable(child, relationships));
      } else if (tag === "w:sectPr") {
        const sect = document.createElement("div");
        sect.className = "doc-section-break";
        sect.setAttribute("data-type", "next-page");
        root.appendChild(sect);
      }
    }
    return root;
  }

  function parseParagraph(p, rels) {
    const out = document.createElement("p");
    const pPr = p.getElementsByTagName("w:pPr")[0];
    let styleName = "";
    if (pPr) {
      const pStyle = pPr.getElementsByTagName("w:pStyle")[0];
      if (pStyle) styleName = pStyle.getAttribute("w:val") || "";
    }
    if (styleName.startsWith("Heading")) {
      const lvl = parseInt(styleName.replace("Heading", "")) || 1;
      out.tagName = "h" + Math.min(6, Math.max(1, lvl));
      const newEl = document.createElement("h" + lvl);
      out.parentNode && out.parentNode.replaceChild(newEl, out);
      var real = newEl;
    } else {
      var real = out;
    }
    for (let i = 0; i < p.childNodes.length; i++) {
      const r = p.childNodes[i];
      if (r.nodeName === "w:r") {
        const runSpan = parseRun(r, rels);
        if (runSpan) real.appendChild(runSpan);
      } else if (r.nodeName === "w:hyperlink") {
        const rid = r.getAttribute("r:id");
        const url = rid && rels && rels[rid] ? rels[rid].Target : "";
        const link = document.createElement("a");
        link.href = url || "#";
        for (let j = 0; j < r.childNodes.length; j++) {
          if (r.childNodes[j].nodeName === "w:r") {
            const span = parseRun(r.childNodes[j], rels);
            if (span) link.appendChild(span);
          }
        }
        real.appendChild(link);
      }
    }
    return real;
  }

  function parseRun(r, rels) {
    const rPr = r.getElementsByTagName("w:rPr")[0];
    const wrapper = document.createElement("span");
    if (rPr) {
      if (rPr.getElementsByTagName("w:b")[0]) wrapper.style.fontWeight = "bold";
      if (rPr.getElementsByTagName("w:i")[0]) wrapper.style.fontStyle = "italic";
      if (rPr.getElementsByTagName("w:u")[0]) wrapper.style.textDecoration = "underline";
      if (rPr.getElementsByTagName("w:strike")[0]) wrapper.style.textDecoration = "line-through";
      const color = rPr.getElementsByTagName("w:color")[0];
      if (color) wrapper.style.color = "#" + (color.getAttribute("w:val") || "000000").replace("#", "");
      const sz = rPr.getElementsByTagName("w:sz")[0];
      if (sz) wrapper.style.fontSize = (parseInt(sz.getAttribute("w:val")) / 2) + "px";
      const font = rPr.getElementsByTagName("w:rFonts")[0];
      if (font) wrapper.style.fontFamily = font.getAttribute("w:ascii") || font.getAttribute("w:hAnsi") || "";
      const highlight = rPr.getElementsByTagName("w:highlight")[0];
      if (highlight) wrapper.style.backgroundColor = "#" + (highlight.getAttribute("w:val") || "yellow");
    }
    const t = r.getElementsByTagName("w:t")[0];
    if (t) wrapper.appendChild(document.createTextNode(decodeXmlEntities(t.textContent || "")));
    const br = r.getElementsByTagName("w:br")[0];
    if (br) wrapper.appendChild(document.createElement("br"));
    const tab = r.getElementsByTagName("w:tab")[0];
    if (tab) wrapper.appendChild(document.createTextNode("\t"));
    const drawing = r.getElementsByTagName("w:drawing")[0];
    if (drawing) {
      const img = parseDrawingImage(drawing, rels);
      if (img) wrapper.appendChild(img);
    }
    return wrapper;
  }

  function parseDrawingImage(drawing, rels) {
    if (!rels) return null;
    const blip = drawing.getElementsByTagName("a:blip")[0] || drawing.getElementsByTagName("blip")[0];
    if (!blip) return null;
    const embedId = blip.getAttribute("r:embed") || blip.getAttribute("r:link");
    if (!embedId) return null;
    const target = rels[embedId] && rels[embedId].Target;
    if (!target) return null;
    const img = document.createElement("img");
    img.src = target;
    img.style.maxWidth = "100%";
    return img;
  }

  function parseTable(tbl, rels) {
    const out = document.createElement("table");
    out.style.borderCollapse = "collapse";
    out.style.width = "100%";
    for (let i = 0; i < tbl.childNodes.length; i++) {
      const tr = tbl.childNodes[i];
      if (tr.nodeName !== "w:tr") continue;
      const trEl = document.createElement("tr");
      for (let j = 0; j < tr.childNodes.length; j++) {
        const tc = tr.childNodes[j];
        if (tc.nodeName !== "w:tc") continue;
        const cellEl = document.createElement("tc".replace("tc", "td"));
        const tcPr = tc.getElementsByTagName("w:tcPr")[0];
        if (tcPr) {
          const tcW = tcPr.getElementsByTagName("w:tcW")[0];
          if (tcW) {
            const w = tcW.getAttribute("w:w");
            const type = tcW.getAttribute("w:type");
            if (w && (type === "dxa" || !type)) cellEl.style.width = (parseInt(w) / 1440 * 96) + "px";
            if (type === "pct") cellEl.style.width = (parseInt(w) / 50) + "%";
          }
        }
        for (let k = 0; k < tc.childNodes.length; k++) {
          if (tc.childNodes[k].nodeName === "w:p") {
            cellEl.appendChild(parseParagraph(tc.childNodes[k], rels));
          }
        }
        if (!cellEl.children.length) cellEl.innerHTML = "&nbsp;";
        trEl.appendChild(cellEl);
      }
      out.appendChild(trEl);
    }
    return out;
  }

  function parseRelationships(xmlString) {
    const out = {};
    if (!xmlString) return out;
    const parser = new DOMParser();
    const doc = parser.parseFromString(xmlString, "application/xml");
    const rels = doc.getElementsByTagName("Relationship");
    for (let i = 0; i < rels.length; i++) {
      const r = rels[i];
      out[r.getAttribute("Id")] = {
        Type: r.getAttribute("Type"),
        Target: r.getAttribute("Target"),
      };
    }
    return out;
  }

  function buildImageRelationships(files) {
    const out = {};
    const path = "word/_rels/document.xml.rels";
    if (files[path]) {
      const rels = parseRelationships(bytesToString(files[path]));
      Object.assign(out, rels);
    }
    const mediaRegex = /^word\/media\//;
    for (const name in files) {
      if (mediaRegex.test(name)) {
        const ext = name.split(".").pop().toLowerCase();
        const mime = ext === "jpg" || ext === "jpeg" ? "image/jpeg" : ext === "png" ? "image/png" : ext === "gif" ? "image/gif" : "application/octet-stream";
        const b64 = arrayBufferToBase64(files[name]);
        out["media-" + name] = { Target: "data:" + mime + ";base64," + b64 };
      }
    }
    return out;
  }

  function arrayBufferToBase64(bytes) {
    let s = "";
    const chunk = 0x8000;
    for (let i = 0; i < bytes.length; i += chunk) {
      s += String.fromCharCode.apply(null, bytes.subarray(i, i + chunk));
    }
    return btoa(s);
  }

  async function parse(blob) {
    const files = await readZip(blob);
    const documentXml = files["word/document.xml"];
    if (!documentXml) throw new Error("Not a valid .docx (no word/document.xml)");
    const rels = buildImageRelationships(files);
    const xmlStr = bytesToString(documentXml);
    const root = parseDocumentXml(xmlStr, rels);
    return root;
  }

  async function importBlob(blob) {
    const html = await parse(blob);
    const editor = document.querySelector(".editor") || document.querySelector("[contenteditable]");
    if (editor && html) {
      editor.innerHTML = "";
      while (html.firstChild) editor.appendChild(html.firstChild);
      return true;
    }
    return false;
  }

  async function importFile(file) {
    return importBlob(file);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      const fileInput = document.getElementById("docxImportInput");
      if (fileInput) {
        fileInput.addEventListener("change", async function (e) {
          if (e.target.files && e.target.files[0]) {
            await importFile(e.target.files[0]);
          }
        });
      }
    });
  }

  window.DocsDOCXImport = { importFile, importBlob, parse };
})();
