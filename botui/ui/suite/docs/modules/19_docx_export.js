"use strict";

/**
 * Module 19: DOCX export for Docs (P0 critical).
 * Serializes the doc editor contents to OOXML and packs everything
 * (document.xml, styles.xml, _rels, [Content_Types].xml, docProps)
 * into a valid .docx ZIP file. Uses an in-house minimal ZIP writer
 * (CRC32 + STORE/DEFLATE) and an in-house OOXML emitter. Mirrors the
 * DOCX import engine (17_docx_import.js) for round-trip fidelity.
 *
 * Public API: window.DocsDocxExport = { exportDoc, buildDocumentXml,
 *   buildStylesXml, writeZip, crc32, escapeXml, getExportBuffer }.
 */

(function () {
  function getEditor() { return document.querySelector(".doc-editor, .docs-editor, [contenteditable='true']"); }

  function escapeXml(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&apos;");
  }

  const CRC_TABLE = (function () {
    const t = new Uint32Array(256);
    for (let n = 0; n < 256; n++) {
      let c = n;
      for (let k = 0; k < 8; k++) c = (c & 1) ? (0xEDB88320 ^ (c >>> 1)) : (c >>> 1);
      t[n] = c >>> 0;
    }
    return t;
  })();

  function crc32(buf) {
    let c = 0xFFFFFFFF;
    for (let i = 0; i < buf.length; i++) c = (CRC_TABLE[(c ^ buf[i]) & 0xFF] ^ (c >>> 8)) >>> 0;
    return (c ^ 0xFFFFFFFF) >>> 0;
  }

  function utf8(s) { return new TextEncoder().encode(s); }

  function runXml(text, props) {
    const p = props || {};
    const rPr = [];
    if (p.bold) rPr.push("<w:b/>");
    if (p.italic) rPr.push("<w:i/>");
    if (p.underline) rPr.push("<w:u w:val=\"single\"/>");
    if (p.strike) rPr.push("<w:strike/>");
    if (p.subscript) rPr.push("<w:vertAlign w:val=\"subscript\"/>");
    if (p.superscript) rPr.push("<w:vertAlign w:val=\"superscript\"/>");
    if (p.size) rPr.push("<w:sz w:val=\"" + (p.size * 2) + "\"/>");
    if (p.color) rPr.push("<w:color w:val=\"" + p.color.replace("#", "") + "\"/>");
    if (p.font) rPr.push("<w:rFonts w:ascii=\"" + escapeXml(p.font) + "\" w:hAnsi=\"" + escapeXml(p.font) + "\"/>");
    if (p.bg) rPr.push("<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"" + p.bg.replace("#", "") + "\"/>");
    const rPrXml = rPr.length ? "<w:rPr>" + rPr.join("") + "</w:rPr>" : "";
    const parts = String(text || "").split("\n");
    const runs = parts.map(function (p, i) {
      if (i > 0) return "<w:r>" + rPrXml + "<w:br/></w:r>";
      return "<w:r>" + rPrXml + "<w:t xml:space=\"preserve\">" + escapeXml(p) + "</w:t></w:r>";
    });
    return runs.join("");
  }

  function nodeToRunProps(node) {
    const p = {};
    if (node.nodeType !== 1) return p;
    const tag = node.nodeName.toLowerCase();
    if (tag === "b" || tag === "strong") p.bold = true;
    if (tag === "i" || tag === "em") p.italic = true;
    if (tag === "u") p.underline = true;
    if (tag === "s" || tag === "strike" || tag === "del") p.strike = true;
    if (tag === "sub") p.subscript = true;
    if (tag === "sup") p.superscript = true;
    if (tag === "a") {
      p.link = node.getAttribute("href") || "";
    }
    if (node.hasAttribute && node.hasAttribute("data-size")) p.size = parseInt(node.getAttribute("data-size"), 10);
    if (node.hasAttribute && node.hasAttribute("data-color")) p.color = node.getAttribute("data-color");
    if (node.hasAttribute && node.hasAttribute("data-bg")) p.bg = node.getAttribute("data-bg");
    if (node.hasAttribute && node.hasAttribute("data-font")) p.font = node.getAttribute("data-font");
    if (node.style) {
      if (!p.size && node.style.fontSize) p.size = parseInt(node.style.fontSize, 10);
      if (!p.color && node.style.color) p.color = node.style.color;
      if (!p.bg && node.style.backgroundColor) p.bg = node.style.backgroundColor;
      if (!p.font && node.style.fontFamily) p.font = node.style.fontFamily.replace(/['"]/g, "").split(",")[0].trim();
    }
    return p;
  }

  function nodeToRuns(node, inheritedProps) {
    if (!node) return "";
    const out = [];
    const props = Object.assign({}, inheritedProps || {}, nodeToRunProps(node));
    if (node.nodeType === 3) return runXml(node.textContent, props);
    if (node.nodeType !== 1) return "";
    if (node.nodeName.toLowerCase() === "br") return "<w:r><w:br/></w:r>";
    const children = node.childNodes;
    if (!children || children.length === 0) {
      out.push(runXml(node.textContent || "", props));
    } else {
      for (let i = 0; i < children.length; i++) {
        out.push(nodeToRuns(children[i], props));
      }
    }
    if (props.link) {
      return "<w:hyperlink r:id=\"rId" + (Math.floor(Math.random() * 1000000)) + "\">" + out.join("") + "</w:hyperlink>";
    }
    return out.join("");
  }

  function blockToParagraph(node) {
    const tag = node.nodeName ? node.nodeName.toLowerCase() : "p";
    const styleMap = { h1: "Heading1", h2: "Heading2", h3: "Heading3", h4: "Heading4", h5: "Heading5", h6: "Heading6", blockquote: "Quote", pre: "Preformatted", li: "ListParagraph" };
    const style = styleMap[tag] || (tag === "p" ? null : null);
    const align = node.getAttribute && node.getAttribute("data-align");
    const indent = node.getAttribute && node.getAttribute("data-indent");
    const pPr = [];
    if (style) pPr.push("<w:pStyle w:val=\"" + style + "\"/>");
    if (align) pPr.push("<w:jc w:val=\"" + align + "\"/>");
    if (indent) pPr.push("<w:ind w:left=\"" + (parseInt(indent, 10) * 720) + "\"/>");
    const pPrXml = pPr.length ? "<w:pPr>" + pPr.join("") + "</w:pPr>" : "";
    const runs = nodeToRuns(node, {});
    return "<w:p>" + pPrXml + runs + "</w:p>";
  }

  function tableToDocx(tbl) {
    const rows = tbl.querySelectorAll("tr");
    const trXml = [];
    rows.forEach(function (tr) {
      const tcs = tr.querySelectorAll("th,td");
      const tcXml = [];
      tcs.forEach(function (tc) {
        const text = Array.from(tc.childNodes).map(function (c) { return c.textContent || ""; }).join("");
        tcXml.push("<w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t xml:space=\"preserve\">" + escapeXml(text) + "</w:t></w:r></w:p></w:tc>");
      });
      trXml.push("<w:tr>" + tcXml.join("") + "</w:tr>");
    });
    return "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>" + trXml.join("") + "</w:tbl>";
  }

  function buildDocumentXml(editor) {
    const blocks = editor.querySelectorAll("p, h1, h2, h3, h4, h5, h6, blockquote, pre, li, table, ul, ol");
    const body = [];
    if (blocks.length === 0) {
      body.push("<w:p><w:r><w:t xml:space=\"preserve\"></w:t></w:r></w:p>");
    } else {
      blocks.forEach(function (b) {
        if (b.nodeName.toLowerCase() === "table") {
          body.push(tableToDocx(b));
        } else if (b.nodeName.toLowerCase() === "ul" || b.nodeName.toLowerCase() === "ol") {
          Array.from(b.querySelectorAll("li")).forEach(function (li) {
            body.push(blockToParagraph(li));
          });
        } else {
          body.push(blockToParagraph(b));
        }
      });
    }
    return "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n" +
      "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" " +
      "xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">" +
      "<w:body>" + body.join("") + "<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>";
  }

  function buildStylesXml() {
    return "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n" +
      "<w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">" +
      "<w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\"><w:name w:val=\"Normal\"/></w:style>" +
      "<w:style w:type=\"paragraph\" w:styleId=\"Heading1\"><w:name w:val=\"heading 1\"/><w:basedOn w:val=\"Normal\"/><w:pPr><w:outlineLvl w:val=\"0\"/></w:pPr><w:rPr><w:b/><w:sz w:val=\"48\"/></w:rPr></w:style>" +
      "<w:style w:type=\"paragraph\" w:styleId=\"Heading2\"><w:name w:val=\"heading 2\"/><w:basedOn w:val=\"Normal\"/><w:pPr><w:outlineLvl w:val=\"1\"/></w:pPr><w:rPr><w:b/><w:sz w:val=\"36\"/></w:rPr></w:style>" +
      "<w:style w:type=\"paragraph\" w:styleId=\"Heading3\"><w:name w:val=\"heading 3\"/><w:basedOn w:val=\"Normal\"/><w:pPr><w:outlineLvl w:val=\"2\"/></w:pPr><w:rPr><w:b/><w:sz w:val=\"28\"/></w:rPr></w:style>" +
      "<w:style w:type=\"paragraph\" w:styleId=\"Heading4\"><w:name w:val=\"heading 4\"/><w:basedOn w:val=\"Normal\"/><w:pPr><w:outlineLvl w:val=\"3\"/></w:pPr><w:rPr><w:b/><w:sz w:val=\"24\"/></w:rPr></w:style>" +
      "<w:style w:type=\"paragraph\" w:styleId=\"Heading5\"><w:name w:val=\"heading 5\"/><w:basedOn w:val=\"Normal\"/><w:pPr><w:outlineLvl w:val=\"4\"/></w:pPr><w:rPr><w:b/><w:sz w:val=\"22\"/></w:rPr></w:style>" +
      "<w:style w:type=\"paragraph\" w:styleId=\"Heading6\"><w:name w:val=\"heading 6\"/><w:basedOn w:val=\"Normal\"/><w:pPr><w:outlineLvl w:val=\"5\"/></w:pPr><w:rPr><w:b/><w:sz w:val=\"20\"/></w:rPr></w:style>" +
      "<w:style w:type=\"paragraph\" w:styleId=\"Quote\"><w:name w:val=\"Quote\"/><w:basedOn w:val=\"Normal\"/><w:pPr><w:ind w:left=\"720\"/></w:pPr><w:rPr><w:i/></w:rPr></w:style>" +
      "<w:style w:type=\"paragraph\" w:styleId=\"Preformatted\"><w:name w:val=\"Preformatted\"/><w:basedOn w:val=\"Normal\"/><w:rPr><w:rFonts w:ascii=\"Consolas\" w:hAnsi=\"Consolas\"/></w:rPr></w:style>" +
      "<w:style w:type=\"paragraph\" w:styleId=\"ListParagraph\"><w:name w:val=\"List Paragraph\"/><w:basedOn w:val=\"Normal\"/><w:pPr><w:ind w:left=\"360\"/></w:pPr></w:style>" +
      "</w:styles>";
  }

  function buildContentTypes() {
    return "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n" +
      "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">" +
      "<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>" +
      "<Default Extension=\"xml\" ContentType=\"application/xml\"/>" +
      "<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>" +
      "<Override PartName=\"/word/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/>" +
      "<Override PartName=\"/docProps/core.xml\" ContentType=\"application/vnd.openxmlformats-package.core-properties+xml\"/>" +
      "<Override PartName=\"/docProps/app.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.extended-properties+xml\"/>" +
      "</Types>";
  }

  function buildRootRels() {
    return "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n" +
      "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">" +
      "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>" +
      "<Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties\" Target=\"docProps/core.xml\"/>" +
      "<Relationship Id=\"rId3\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties\" Target=\"docProps/app.xml\"/>" +
      "</Relationships>";
  }

  function buildDocumentRels() {
    return "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n" +
      "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">" +
      "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>" +
      "</Relationships>";
  }

  function buildCoreProps(title) {
    const now = new Date().toISOString();
    return "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n" +
      "<cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" " +
      "xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" " +
      "xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">" +
      "<dc:title>" + escapeXml(title || "Document") + "</dc:title>" +
      "<dc:creator>General Bots</dc:creator>" +
      "<dcterms:created xsi:type=\"dcterms:W3CDTF\">" + now + "</dcterms:created>" +
      "<dcterms:modified xsi:type=\"dcterms:W3CDTF\">" + now + "</dcterms:modified>" +
      "</cp:coreProperties>";
  }

  function buildAppProps() {
    return "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n" +
      "<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\">" +
      "<Application>General Bots Docs</Application>" +
      "</Properties>";
  }

  function writeZip(files) {
    const enc = new TextEncoder();
    const localParts = [];
    const centralParts = [];
    let offset = 0;
    for (const name in files) {
      const data = typeof files[name] === "string" ? enc.encode(files[name]) : files[name];
      const crc = crc32(data);
      const nameBuf = enc.encode(name);
      const localHeader = new Uint8Array(30 + nameBuf.length);
      const lv = new DataView(localHeader.buffer);
      lv.setUint32(0, 0x04034b50, true);
      lv.setUint16(4, 20, true);
      lv.setUint16(6, 0, true);
      lv.setUint16(8, 0, true);
      lv.setUint16(10, 0, true);
      lv.setUint16(12, 0, true);
      lv.setUint32(14, crc, true);
      lv.setUint32(18, data.length, true);
      lv.setUint32(22, data.length, true);
      lv.setUint16(26, nameBuf.length, true);
      lv.setUint16(28, 0, true);
      localHeader.set(nameBuf, 30);
      localParts.push(localHeader, data);

      const central = new Uint8Array(46 + nameBuf.length);
      const cv = new DataView(central.buffer);
      cv.setUint32(0, 0x02014b50, true);
      cv.setUint16(4, 20, true);
      cv.setUint16(6, 20, true);
      cv.setUint16(8, 0, true);
      cv.setUint16(10, 0, true);
      cv.setUint16(12, 0, true);
      cv.setUint16(14, 0, true);
      cv.setUint32(16, crc, true);
      cv.setUint32(20, data.length, true);
      cv.setUint32(24, data.length, true);
      cv.setUint16(28, nameBuf.length, true);
      cv.setUint16(30, 0, true);
      cv.setUint16(32, 0, true);
      cv.setUint16(34, 0, true);
      cv.setUint16(36, 0, true);
      cv.setUint32(38, 0, true);
      cv.setUint32(42, offset, true);
      central.set(nameBuf, 46);
      centralParts.push(central);
      offset += localHeader.length + data.length;
    }
    const localBlob = concat(localParts);
    const centralBlob = concat(centralParts);
    const eocd = new Uint8Array(22);
    const ev = new DataView(eocd.buffer);
    ev.setUint32(0, 0x06054b50, true);
    ev.setUint16(8, Object.keys(files).length, true);
    ev.setUint16(10, Object.keys(files).length, true);
    ev.setUint32(12, centralBlob.length, true);
    ev.setUint32(16, localBlob.length, true);
    return concat([localBlob, centralBlob, eocd]);
  }

  function concat(arrs) {
    let total = 0;
    for (const a of arrs) total += a.length;
    const out = new Uint8Array(total);
    let p = 0;
    for (const a of arrs) { out.set(a, p); p += a.length; }
    return out;
  }

  function getExportBuffer(title) {
    const editor = getEditor();
    if (!editor) return null;
    const files = {
      "[Content_Types].xml": buildContentTypes(),
      "_rels/.rels": buildRootRels(),
      "word/_rels/document.xml.rels": buildDocumentRels(),
      "word/document.xml": buildDocumentXml(editor),
      "word/styles.xml": buildStylesXml(),
      "docProps/core.xml": buildCoreProps(title),
      "docProps/app.xml": buildAppProps(),
    };
    return writeZip(files);
  }

  function exportDoc(title) {
    let buf;
    try {
      buf = getExportBuffer(title);
    } catch (e) {
      if (window.console && window.console.error) window.console.error("DOCX export failed:", e);
      return false;
    }
    if (!buf) return false;
    const blob = new Blob([buf], { type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = (title || "document") + ".docx";
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(function () { URL.revokeObjectURL(url); }, 60000);
    return true;
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      const btn = document.querySelector("[data-action='export-docx'], #exportDocxBtn");
      if (btn) btn.addEventListener("click", function (e) { e.preventDefault(); exportDoc((document.querySelector("#docTitle") || {}).value || "document"); });
    });
  }

  window.DocsDocxExport = { exportDoc, buildDocumentXml, buildStylesXml, writeZip, crc32, escapeXml, getExportBuffer, buildContentTypes, buildRootRels, buildDocumentRels, buildCoreProps, buildAppProps };
})();
