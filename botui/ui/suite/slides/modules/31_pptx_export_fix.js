"use strict";

/**
 * Module 31: PPTX export fix for Slides (P0 critical).
 * Replaces the "PPTX export not yet implemented" alert with a real
 * .pptx file using an in-house OOXML emitter + ZIP writer. Mirrors
 * 27_pptx_import.js (parser) for round-trip fidelity. Each slide
 * becomes a slideN.xml with a spTree containing text boxes, shapes,
 * images, tables, charts, and grouped elements.
 *
 * Public API: window.SlidesPptxExportFix = {
 *   exportPptx, buildPptx, writeZip, buildSlideXml, buildPresentationXml
 * }.
 */

(function () {
  function getState() { return window.state || null; }

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

  function escapeXml(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&apos;");
  }

  function emu(v) { return Math.round((v || 0) * 914400); }

  function buildElementXml(el, idx) {
    const x = emu(el.x || 0);
    const y = emu(el.y || 0);
    const w = emu(el.width || 20);
    const h = emu(el.height || 10);
    const id = (idx + 1) * 1000;
    let inner = "";
    const type = el.type || "text";
    if (type === "title" || type === "text" || type === "body") {
      const text = el.text || (el.runs || []).map(function (r) { return r.text || ""; }).join("");
      inner = "<p:nvSpPr><p:nvPr><p:ph type=\"" + (type === "title" ? "title" : "body") + "\"/></p:nvPr><p:cNvPr id=\"" + id + "\" name=\"Text " + id + "\"/><p:cNvSpPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x=\"" + x + "\" y=\"" + y + "\"/><a:ext cx=\"" + w + "\" cy=\"" + h + "\"/></a:xfrm><a:prstGeom prst=\"rect\"/></p:spPr><p:txBody><a:bodyPr wrap=\"square\"/><a:lstStyle/><a:p><a:r><a:rPr/><a:t>" + escapeXml(text) + "</a:t></a:r></a:p></p:txBody>";
    } else if (type === "shape") {
      const shape = el.shape || "rect";
      inner = "<p:nvSpPr><p:nvPr/><p:cNvPr id=\"" + id + "\" name=\"Shape " + id + "\"/><p:cNvSpPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x=\"" + x + "\" y=\"" + y + "\"/><a:ext cx=\"" + w + "\" cy=\"" + h + "\"/></a:xfrm><a:prstGeom prst=\"" + shape + "\"/><a:solidFill><a:srgbClr val=\"" + (el.fill || "1A73E8").replace("#", "") + "\"/></a:solidFill></p:spPr><p:txBody><a:bodyPr/><a:lstStyle/><a:p/></p:txBody>";
    } else if (type === "image") {
      const rId = "rIdImg" + idx;
      inner = "<p:nvPicPr><p:cNvPr id=\"" + id + "\" name=\"Image " + id + "\"/><p:cNvPicPr/></p:nvPicPr><p:blipFill><a:blip r:embed=\"" + rId + "\"/></p:blipFill><p:spPr><a:xfrm><a:off x=\"" + x + "\" y=\"" + y + "\"/><a:ext cx=\"" + w + "\" cy=\"" + h + "\"/></a:xfrm><a:prstGeom prst=\"rect\"/></p:spPr>";
      inner = "<p:nvSpPr>" + inner.split("p:nvSpPr")[1];
      inner = "<p:pic" + inner + "</p:pic>";
    } else if (type === "group") {
      const children = (el.children || []).map(function (c, i) { return buildElementXml(c, idx * 100 + i); }).join("");
      inner = "<p:nvSpPr><p:nvPr/><p:cNvPr id=\"" + id + "\" name=\"Group " + id + "\"/><p:cNvGrpSpPr/></p:nvSpPr><p:grpSpPr><a:xfrm><a:off x=\"" + x + "\" y=\"" + y + "\"/><a:ext cx=\"" + w + "\" cy=\"" + h + "\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"" + w + "\" cy=\"" + h + "\"/></a:xfrm></p:grpSpPr>" + children;
      inner = "<p:grpSp" + inner + "</p:grpSp>";
      return inner;
    } else if (type === "chart") {
      const rId = "rIdChart" + idx;
      inner = "<p:nvSpPr><p:nvPr/><p:cNvPr id=\"" + id + "\" name=\"Chart " + id + "\"/><p:cNvSpPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x=\"" + x + "\" y=\"" + y + "\"/><a:ext cx=\"" + w + "\" cy=\"" + h + "\"/></a:xfrm><a:prstGeom prst=\"rect\"/></p:spPr><p:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/chart\"><c:chart xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" r:id=\"" + rId + "\"/></a:graphicData></a:graphic>";
    } else if (type === "table") {
      const rows = el.rows || 3;
      const cols = el.cols || 3;
      const cells = [];
      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
          const colW = w / cols;
          const rowH = h / rows;
          const cx = x + c * colW;
          const cy = y + r * rowH;
          cells.push("<a:tc><a:tcPr><a:lnL w=\"6350\"/><a:lnR w=\"6350\"/><a:lnT w=\"6350\"/><a:lnB w=\"6350\"/></a:tcPr><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang=\"en-US\"/></a:p></a:txBody></a:tc>");
        }
      }
      let rowsXml = "";
      for (let r = 0; r < rows; r++) {
        rowsXml += "<a:tr h=\"" + (h / rows) + "\">" + cells.slice(r * cols, r * cols + cols).join("") + "</a:tr>";
      }
      inner = "<p:nvSpPr><p:nvPr/><p:cNvPr id=\"" + id + "\" name=\"Table " + id + "\"/><p:cNvSpPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></p:spPr><p:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/table\"><a:tbl><a:tblPr/><a:tblGrid>" + Array.from({ length: cols }, function () { return "<a:gridCol w=\"" + (w / cols) + "\"/>"; }).join("") + "</a:tblGrid>" + rowsXml + "</a:tbl></a:graphicData></a:graphic></p:spPr>";
    } else {
      inner = "<p:nvSpPr><p:nvPr/><p:cNvPr id=\"" + id + "\" name=\"Element " + id + "\"/><p:cNvSpPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x=\"" + x + "\" y=\"" + y + "\"/><a:ext cx=\"" + w + "\" cy=\"" + h + "\"/></a:xfrm><a:prstGeom prst=\"rect\"/></p:spPr><p:txBody><a:bodyPr/><a:lstStyle/><a:p/></p:txBody>";
    }
    if (!inner.startsWith("<p:") || inner.startsWith("<p:spc")) {
      if (type === "image") return inner;
      inner = "<p:sp>" + inner + "</p:sp>";
    }
    return inner;
  }

  function buildSlideXml(slide, idx) {
    const slideId = idx + 1;
    const elements = (slide.elements || []).map(function (el, i) { return buildElementXml(el, i); }).join("\n");
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" ' +
      'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" ' +
      'xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">' +
      '<p:cSld><p:spTree>' +
      '<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>' +
      '<p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>' +
      elements +
      '</p:spTree></p:cSld>' +
      '<p:transition' + (slide.transition && slide.transition !== "none" ? ' spd="med" p:="' + escapeXml(slide.transition) + '"/>' : '/>') +
      '</p:sld>';
  }

  function buildPresentationXml(slides) {
    const slideIds = slides.map(function (_, i) {
      return '<p:sldId id="' + (256 + i) + '" r:id="rIdSl' + (i + 1) + '"/>';
    }).join("");
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" ' +
      'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" ' +
      'xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">' +
      '<p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rIdMaster"/></p:sldMasterIdLst>' +
      '<p:sldIdLst>' + slideIds + '</p:sldIdLst>' +
      '<p:sldSz cx="9144000" cy="6858000" type="screen4x3"/>' +
      '<p:notesSz cx="6858000" cy="9144000"/>' +
      '</p:presentation>';
  }

  function buildSlideRels(slide, idx) {
    const rels = ['<Relationship Id="rIdSl" Target="../slideLayouts/slideLayout1.xml"/>'];
    for (let i = 0; i < (slide.elements || []).length; i++) {
      const el = slide.elements[i];
      if (el.type === "image" && el.url) {
        rels.push('<Relationship Id="rIdImg' + i + '" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image' + (idx + 1) + '_' + i + '.png"/>');
      } else if (el.type === "chart") {
        rels.push('<Relationship Id="rIdChart' + i + '" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart" Target="../charts/chart' + (idx + 1) + '_' + i + '.xml"/>');
      }
    }
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' + rels.join("") + '</Relationships>';
  }

  function buildSlideLayoutXml() {
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<p:sldLayout xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" type="blank" preserve="1"><p:cSld name="Blank"><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld></p:sldLayout>';
  }

  function buildSlideMasterXml() {
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld></p:sldMaster>';
  }

  function buildThemeXml() {
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office"><a:themeElements><a:clrScheme name="Office"><a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1><a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="44546A"/></a:dk2><a:lt2><a:srgbClr val="E7E6E6"/></a:lt2><a:accent1><a:srgbClr val="5B9BD5"/></a:accent1><a:accent2><a:srgbClr val="ED7D31"/></a:accent2><a:accent3><a:srgbClr val="A5A5A5"/></a:accent3><a:accent4><a:srgbClr val="FFC000"/></a:accent4><a:accent5><a:srgbClr val="4472C4"/></a:accent5><a:accent6><a:srgbClr val="70AD47"/></a:accent6><a:hlink><a:srgbClr val="0563C1"/></a:hlink><a:folHlink><a:srgbClr val="954F72"/></a:folHlink></a:clrScheme><a:fontScheme name="Office"><a:majorFont><a:latin typeface="Calibri Light"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme><a:fmtScheme name="Office"/></a:themeElements></a:theme>';
  }

  function buildContentTypes(slides) {
    const overrides = ['<Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>',
      '<Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>',
      '<Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>',
      '<Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>',
      '<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>',
      '<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>'];
    for (let i = 0; i < slides.length; i++) {
      overrides.push('<Override PartName="/ppt/slides/slide' + (i + 1) + '.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>');
    }
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">' +
      '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>' +
      '<Default Extension="xml" ContentType="application/xml"/>' +
      '<Default Extension="png" ContentType="image/png"/>' +
      '<Default Extension="jpeg" ContentType="image/jpeg"/>' +
      overrides.join("") + '</Types>';
  }

  function buildRootRels(slides) {
    const rels = [];
    for (let i = 0; i < slides.length; i++) {
      rels.push('<Relationship Id="rIdSl' + (i + 1) + '" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide' + (i + 1) + '.xml"/>');
    }
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
      rels.join("") +
      '<Relationship Id="rIdMaster" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>' +
      '<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>' +
      '<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>' +
      '</Relationships>';
  }

  function buildPresRels() {
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
      '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>' +
      '<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>' +
      '</Relationships>';
  }

  function buildCoreProps(title) {
    const now = new Date().toISOString();
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" ' +
      'xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" ' +
      'xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">' +
      '<dc:title>' + escapeXml(title || "Presentation") + '</dc:title>' +
      '<dc:creator>General Bots</dc:creator>' +
      '<dcterms:created xsi:type="dcterms:W3CDTF">' + now + '</dcterms:created>' +
      '<dcterms:modified xsi:type="dcterms:W3CDTF">' + now + '</dcterms:modified>' +
      '</cp:coreProperties>';
  }

  function buildAppProps() {
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">' +
      '<Application>General Bots Slides</Application>' +
      '<Slides>' + (getState() && (getState().slides || []).length || 0) + '</Slides>' +
      '</Properties>';
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

  function buildPptx(title) {
    const s = getState();
    if (!s) return null;
    const slides = s.slides || [];
    const files = {
      "[Content_Types].xml": buildContentTypes(slides),
      "_rels/.rels": buildRootRels(slides),
      "ppt/_rels/presentation.xml.rels": buildPresRels(),
      "ppt/presentation.xml": buildPresentationXml(slides),
      "ppt/slideMasters/slideMaster1.xml": buildSlideMasterXml(),
      "ppt/slideLayouts/slideLayout1.xml": buildSlideLayoutXml(),
      "ppt/theme/theme1.xml": buildThemeXml(),
      "docProps/core.xml": buildCoreProps(title),
      "docProps/app.xml": buildAppProps(),
    };
    for (let i = 0; i < slides.length; i++) {
      files["ppt/slides/slide" + (i + 1) + ".xml"] = buildSlideXml(slides[i], i);
      files["ppt/slides/_rels/slide" + (i + 1) + ".xml.rels"] = buildSlideRels(slides[i], i);
    }
    return writeZip(files);
  }

  function exportPptx(title) {
    const buf = buildPptx(title);
    if (!buf) return false;
    const blob = new Blob([buf], { type: "application/vnd.openxmlformats-officedocument.presentationml.presentation" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = (title || "presentation") + ".pptx";
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(function () { URL.revokeObjectURL(url); }, 60000);
    return true;
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      const btn = document.querySelector("[data-action='export-pptx'], #exportPptxBtn");
      if (btn) btn.addEventListener("click", function (e) { e.preventDefault(); exportPptx((document.querySelector("#presentationName") || {}).value || "presentation"); });
    });
  }

  window.SlidesPptxExportFix = { exportPptx, buildPptx, writeZip, buildSlideXml, buildPresentationXml };
})();
