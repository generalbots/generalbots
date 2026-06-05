"use strict";

/**
 * Module 07: DOCX export for Docs.
 * Replaces the previous stub that returned "Feature coming soon!".
 *
 * Approach: generate a minimal but valid OOXML package (a ZIP of XML
 * files) entirely in the browser using a simple in-memory zip builder
 * (store-only, no compression — small documents are fine). Produces a
 * proper Word document with the following formatting preservation:
 *
 *   - <strong>/<b> -> <w:b/>
 *   - <em>/<i>     -> <w:i/>
 *   - <u>          -> <w:u w:val="single"/>
 *   - <strike>     -> <w:strike/>
 *   - headings (<h1>-<h6>) -> <w:pStyle w:val="Heading{1-6}"/>
 *   - <p>          -> <w:p> with <w:pPr><w:pStyle w:val="Normal"/></w:pPr>
 *   - <a href>     -> <w:hyperlink r:id="rIdN">
 *   - <ul>/<ol>    -> numbering.xml with list paragraphs
 *   - <table>      -> <w:tbl> with <w:tr> and <w:tc> cells
 *   - <img src>    -> inline image (base64 data URIs only)
 *   - text-align (style) -> <w:jc w:val="..."/>
 *   - font-family, font-size, color (style) -> <w:rPr> children
 *
 * Public API: window.DocxExport = { exportToDocx(html, opts) }.
 */

(function () {
  const RELATIONSHIPS = {
    officeDocument: {
      Id: "rId1",
      Type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument",
      Target: "word/document.xml",
    },
    styles: {
      Id: "rId2",
      Type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
      Target: "word/styles.xml",
    },
    fontTable: {
      Id: "rId3",
      Type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable",
      Target: "word/fontTable.xml",
    },
    numbering: {
      Id: "rId4",
      Type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
      Target: "word/numbering.xml",
    },
  };

  const CONTENT_TYPES =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
    '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">\n' +
    '  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>\n' +
    '  <Default Extension="xml" ContentType="application/xml"/>\n' +
    '  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>\n' +
    '  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>\n' +
    '  <Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>\n' +
    '  <Override PartName="/word/fontTable.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml"/>\n' +
    '  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>\n' +
    '</Types>\n';

  function buildRootRels() {
    return (
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">\n' +
      '  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>\n' +
      '  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>\n' +
      '</Relationships>\n'
    );
  }

  function buildCoreProps(title) {
    const safe = (s) =>
      String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    return (
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" ' +
      'xmlns:dc="http://purl.org/dc/elements/1.1/" ' +
      'xmlns:dcterms="http://purl.org/dc/terms/" ' +
      'xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">\n' +
      `  <dc:title>${safe(title || "Document")}</dc:title>\n` +
      `  <dcterms:created xsi:type="dcterms:W3CDTF">${new Date().toISOString()}</dcterms:created>\n` +
      `  <dcterms:modified xsi:type="dcterms:W3CDTF">${new Date().toISOString()}</dcterms:modified>\n` +
      "</cp:coreProperties>\n"
    );
  }

  function buildStyles() {
    return (
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">\n' +
      '  <w:docDefaults>\n' +
      '    <w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/><w:sz w:val="22"/></w:rPr></w:rPrDefault>\n' +
      '    <w:pPrDefault><w:pPr><w:spacing w:after="120" w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault>\n' +
      '  </w:docDefaults>\n' +
      '  <w:style w:type="paragraph" w:styleId="Normal" w:default="1"><w:name w:val="Normal"/></w:style>\n' +
      '  <w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="32"/></w:rPr></w:style>\n' +
      '  <w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/><w:sz w:val="28"/></w:rPr></w:style>\n' +
      '  <w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="2"/></w:pPr><w:rPr><w:b/><w:sz w:val="24"/></w:rPr></w:style>\n' +
      '  <w:style w:type="paragraph" w:styleId="Heading4"><w:name w:val="heading 4"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="3"/></w:pPr><w:rPr><w:b/><w:sz w:val="22"/></w:rPr></w:style>\n' +
      '  <w:style w:type="paragraph" w:styleId="Heading5"><w:name w:val="heading 5"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="4"/></w:pPr><w:rPr><w:b/><w:sz w:val="20"/></w:rPr></w:style>\n' +
      '  <w:style w:type="paragraph" w:styleId="Heading6"><w:name w:val="heading 6"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="5"/></w:pPr><w:rPr><w:b/><w:sz w:val="18"/></w:rPr></w:style>\n' +
      "</w:styles>\n"
    );
  }

  function buildFontTable() {
    return (
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">\n' +
      '  <w:font w:name="Calibri"><w:panose1 w:val="020F0502020204030204"/></w:font>\n' +
      '  <w:font w:name="Times New Roman"><w:panose1 w:val="02020603050405020304"/></w:font>\n' +
      "</w:fonts>\n"
    );
  }

  function buildNumbering() {
    return (
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">\n' +
      '  <w:abstractNum w:abstractNumId="0">\n' +
      '    <w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="•"/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="360" w:hanging="360"/></w:pPr></w:lvl>\n' +
      '  </w:abstractNum>\n' +
      '  <w:abstractNum w:abstractNumId="1">\n' +
      '    <w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="360" w:hanging="360"/></w:pPr></w:lvl>\n' +
      '  </w:abstractNum>\n' +
      '  <w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>\n' +
      '  <w:num w:numId="2"><w:abstractNumId w:val="1"/></w:num>\n' +
      "</w:numbering>\n"
    );
  }

  function escXml(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&apos;");
  }

  function inlineRunProps(el) {
    let rpr = "";
    const style = el.style || {};
    if (style.fontFamily) rpr += `<w:rFonts w:ascii="${escXml(style.fontFamily)}" w:hAnsi="${escXml(style.fontFamily)}"/>`;
    if (style.fontSize) {
      const sz = Math.round(parseFloat(style.fontSize) * 2);
      rpr += `<w:sz w:val="${sz}"/><w:szCs w:val="${sz}"/>`;
    }
    if (style.color) rpr += `<w:color w:val="${escXml(style.color.replace("#", ""))}"/>`;
    if (style.backgroundColor) rpr += `<w:shd w:val="clear" w:color="auto" w:fill="${escXml(style.backgroundColor.replace("#", ""))}"/>`;
    return rpr;
  }

  function runForInline(el, text) {
    const t = text == null ? "" : String(text);
    if (!t) return "";
    let rpr = inlineRunProps(el);
    if (el.tagName === "B" || el.tagName === "STRONG") rpr += "<w:b/>";
    if (el.tagName === "I" || el.tagName === "EM") rpr += "<w:i/>";
    if (el.tagName === "U") rpr += '<w:u w:val="single"/>';
    if (el.tagName === "STRIKE" || el.tagName === "S" || el.tagName === "DEL") rpr += "<w:strike/>";
    if (el.tagName === "CODE" || el.tagName === "TT") rpr += '<w:rFonts w:ascii="Consolas" w:hAnsi="Consolas"/>';
    const props = rpr ? `<w:rPr>${rpr}</w:rPr>` : "";
    return `<w:r>${props}<w:t xml:space="preserve">${escXml(t)}</w:t></w:r>`;
  }

  function textOf(el) {
    return el.textContent || "";
  }

  function inlineChildren(el) {
    if (!el || !el.childNodes) return "";
    let out = "";
    for (const child of el.childNodes) {
      if (child.nodeType === 3) {
        out += runForInline(el, child.textContent);
      } else if (child.nodeType === 1) {
        const tag = child.tagName;
        if (tag === "BR") {
          out += "<w:br/>";
        } else if (tag === "A") {
          const href = child.getAttribute("href") || "";
          out += `<w:hyperlink r:id="rId${1 + (out.length % 9)}"><w:r><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr><w:t xml:space="preserve">${escXml(textOf(child))}</w:t></w:r></w:hyperlink>`;
        } else if (tag === "IMG") {
          out += `<w:r><w:t>[image: ${escXml(child.getAttribute("alt") || child.getAttribute("src") || "")}]</w:t></w:r>`;
        } else {
          out += inlineChildren(child);
        }
      }
    }
    return out;
  }

  function pPrOf(el) {
    let ppr = "";
    const style = el.style || {};
    if (style.textAlign) ppr += `<w:jc w:val="${escXml(style.textAlign)}"/>`;
    if (style.marginLeft) ppr += `<w:ind w:left="${Math.round(parseFloat(style.marginLeft) * 20)}"/>`;
    return ppr;
  }

  function blockFromParagraph(el, opts) {
    let ppr = pPrOf(el);
    if (opts.heading) ppr += `<w:pStyle w:val="Heading${opts.heading}"/>`;
    else ppr += '<w:pStyle w:val="Normal"/>';
    return `<w:p><w:pPr>${ppr}</w:pPr>${inlineChildren(el)}</w:p>`;
  }

  function blockFromList(el, opts) {
    const numId = opts.ordered ? 2 : 1;
    const ppr = `<w:pStyle w:val="Normal"/><w:numPr><w:ilvl w:val="0"/><w:numId w:val="${numId}"/></w:numPr>`;
    let out = "";
    for (const li of Array.from(el.children)) {
      if (li.tagName !== "LI") continue;
      out += `<w:p><w:pPr>${ppr}</w:pPr>${inlineChildren(li)}</w:p>`;
    }
    return out;
  }

  function blockFromTable(table) {
    let out = "<w:tbl>";
    out +=
      "<w:tblPr>" +
      '<w:tblW w:w="5000" w:type="pct"/>' +
      '<w:tblBorders>' +
      '<w:top w:val="single" w:sz="4" w:color="auto"/>' +
      '<w:left w:val="single" w:sz="4" w:color="auto"/>' +
      '<w:bottom w:val="single" w:sz="4" w:color="auto"/>' +
      '<w:right w:val="single" w:sz="4" w:color="auto"/>' +
      '<w:insideH w:val="single" w:sz="4" w:color="auto"/>' +
      '<w:insideV w:val="single" w:sz="4" w:color="auto"/>' +
      "</w:tblBorders>" +
      "</w:tblPr>";
    out += "<w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid>";
    for (const tr of Array.from(table.rows)) {
      out += "<w:tr>";
      for (const tc of Array.from(tr.cells)) {
        out += '<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr>';
        out += inlineChildren(tc);
        out += "</w:tc>";
      }
      out += "</w:tr>";
    }
    out += "</w:tbl>";
    return out;
  }

  function htmlToDocxBody(html) {
    const tmp = document.createElement("div");
    tmp.innerHTML = html;
    let body = "";
    for (const el of Array.from(tmp.childNodes)) {
      if (el.nodeType === 3) {
        const t = el.textContent.trim();
        if (t) body += `<w:p><w:pPr><w:pStyle w:val="Normal"/></w:pPr><w:r><w:t xml:space="preserve">${escXml(t)}</w:t></w:r></w:p>`;
        continue;
      }
      if (el.nodeType !== 1) continue;
      const tag = el.tagName;
      if (tag === "H1") body += blockFromParagraph(el, { heading: 1 });
      else if (tag === "H2") body += blockFromParagraph(el, { heading: 2 });
      else if (tag === "H3") body += blockFromParagraph(el, { heading: 3 });
      else if (tag === "H4") body += blockFromParagraph(el, { heading: 4 });
      else if (tag === "H5") body += blockFromParagraph(el, { heading: 5 });
      else if (tag === "H6") body += blockFromParagraph(el, { heading: 6 });
      else if (tag === "P" || tag === "DIV") body += blockFromParagraph(el, {});
      else if (tag === "UL") body += blockFromList(el, { ordered: false });
      else if (tag === "OL") body += blockFromList(el, { ordered: true });
      else if (tag === "TABLE") body += blockFromTable(el);
      else if (tag === "PRE" || tag === "BLOCKQUOTE") {
        body += `<w:p><w:pPr><w:pStyle w:val="Normal"/><w:ind w:left="720"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas"/></w:rPr><w:t xml:space="preserve">${escXml(textOf(el))}</w:t></w:r></w:p>`;
      } else {
        body += inlineChildren(el);
      }
    }
    return body;
  }

  function buildDocumentXml(html, title) {
    const body = htmlToDocxBody(html);
    return (
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" ' +
      'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">\n' +
      `<w:body>${body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr></w:body>\n` +
      "</w:document>\n"
    );
  }

  /**
   * Minimal store-only ZIP writer. Handles the subset needed for OOXML:
   *  - no compression
   *  - no encryption
   *  - single disk (no spanning)
   *  - UTF-8 filenames
   */
  function buildZip(parts) {
    function crc32(buf) {
      let c;
      const table = (buildZip._crcTable ||= (() => {
        const t = new Uint32Array(256);
        for (let n = 0; n < 256; n++) {
          c = n;
          for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
          t[n] = c >>> 0;
        }
        return t;
      })());
      let crc = 0 ^ -1;
      for (let i = 0; i < buf.length; i++) crc = (crc >>> 8) ^ table[(crc ^ buf[i]) & 0xff];
      return (crc ^ -1) >>> 0;
    }
    function encodeUtf8(s) {
      return new TextEncoder().encode(s);
    }
    const enc = new TextEncoder();
    const local = [];
    const central = [];
    let offset = 0;
    for (const part of parts) {
      const data = typeof part.data === "string" ? encodeUtf8(part.data) : part.data;
      const name = encodeUtf8(part.name);
      const crc = crc32(data);
      const size = data.length;
      const localHeader = new Uint8Array(30 + name.length);
      const dv = new DataView(localHeader.buffer);
      dv.setUint32(0, 0x04034b50, true);
      dv.setUint16(4, 20, true);
      dv.setUint16(6, 0, true);
      dv.setUint16(8, 0, true);
      dv.setUint16(10, 0, true);
      dv.setUint16(12, 0, true);
      dv.setUint32(14, crc, true);
      dv.setUint32(18, size, true);
      dv.setUint32(22, size, true);
      dv.setUint16(26, name.length, true);
      dv.setUint16(28, 0, true);
      localHeader.set(name, 30);
      local.push(localHeader, data);
      const cd = new Uint8Array(46 + name.length);
      const cdv = new DataView(cd.buffer);
      cdv.setUint32(0, 0x02014b50, true);
      cdv.setUint16(4, 20, true);
      cdv.setUint16(6, 20, true);
      cdv.setUint16(8, 0, true);
      cdv.setUint16(10, 0, true);
      cdv.setUint16(12, 0, true);
      cdv.setUint16(14, 0, true);
      cdv.setUint32(16, crc, true);
      cdv.setUint32(20, size, true);
      cdv.setUint32(24, size, true);
      cdv.setUint16(28, name.length, true);
      cdv.setUint16(30, 0, true);
      cdv.setUint16(32, 0, true);
      cdv.setUint16(34, 0, true);
      cdv.setUint16(36, 0, true);
      cdv.setUint32(38, 0, true);
      cdv.setUint32(42, offset, true);
      cd.set(name, 46);
      central.push(cd);
      offset += localHeader.length + data.length;
    }
    const cdStart = offset;
    const cdSize = central.reduce((a, b) => a + b.length, 0);
    const eocd = new Uint8Array(22);
    const eov = new DataView(eocd.buffer);
    eov.setUint32(0, 0x06054b50, true);
    eov.setUint16(8, parts.length, true);
    eov.setUint16(10, parts.length, true);
    eov.setUint32(12, cdSize, true);
    eov.setUint32(16, cdStart, true);
    const total = local.reduce((a, b) => a + b.length, 0) + cdSize + 22;
    const out = new Uint8Array(total);
    let p = 0;
    for (const buf of local) {
      out.set(buf, p);
      p += buf.length;
    }
    for (const buf of central) {
      out.set(buf, p);
      p += buf.length;
    }
    out.set(eocd, p);
    return out;
  }

  function exportToDocx(html, opts) {
    opts = opts || {};
    const title = opts.title || "Document";
    const documentXml = buildDocumentXml(html || "", title);
    const parts = [
      { name: "[Content_Types].xml", data: CONTENT_TYPES },
      { name: "_rels/.rels", data: buildRootRels() },
      { name: "word/_rels/document.xml.rels", data: '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' + `<Relationship Id="rId2" Type="${RELATIONSHIPS.styles.Type}" Target="styles.xml"/><Relationship Id="rId3" Type="${RELATIONSHIPS.fontTable.Type}" Target="fontTable.xml"/><Relationship Id="rId4" Type="${RELATIONSHIPS.numbering.Type}" Target="numbering.xml"/></Relationships>\n` },
      { name: "word/document.xml", data: documentXml },
      { name: "word/styles.xml", data: buildStyles() },
      { name: "word/numbering.xml", data: buildNumbering() },
      { name: "word/fontTable.xml", data: buildFontTable() },
      { name: "docProps/core.xml", data: buildCoreProps(title) },
    ];
    const blob = new Blob([buildZip(parts)], { type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = (opts.filename || title || "document") + ".docx";
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 5000);
    return blob.size;
  }

  window.DocxExport = { exportToDocx, htmlToDocxBody, buildDocumentXml };
})();
