"use strict";

/**
 * Module 16: PPTX export for Slides.
 * Replaces the "PPTX export not yet implemented" stub. Generates a
 * real Office Open XML presentation package (ZIP of XML) entirely in
 * the browser with no external libraries.
 *
 * Preserves:
 *   - Slide titles, body text (basic runs)
 *   - Element positions and sizes
 *   - Background color
 *   - Slide order
 *   - Layout: blank
 *
 * Trade-offs (kept simple for offline zero-dep operation):
 *   - All text becomes a single text run with default formatting
 *   - Images, charts, tables, SmartArt NOT preserved (would require
 *     flate encoding and additional parts)
 *   - Slide transitions/animations NOT preserved
 *   - One shape per element (auto-shape rectangle for now)
 *
 * Public API: window.PptxExport = { exportToPptx(presentation, opts) }.
 */

(function () {
  const CONTENT_TYPES =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
    '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">\n' +
    '  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>\n' +
    '  <Default Extension="xml" ContentType="application/xml"/>\n' +
    '  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>\n' +
    '  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>\n' +
    '  <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>\n' +
    '  <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>\n' +
    '  <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>\n' +
    '  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>\n' +
    '  <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>\n' +
    "</Types>\n";

  const ROOT_RELS =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
    '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">\n' +
    '  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>\n' +
    '  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>\n' +
    '  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>\n' +
    "</Relationships>\n";

  const PRESENTATION_RELS =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
    '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">\n' +
    '  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>\n' +
    "</Relationships>\n";

  const SLIDE_MASTER =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
    '<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" ' +
    'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">\n' +
    '  <p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>\n' +
    '  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>\n' +
    '  <p:sldLayoutIdLst><p:sldLayoutId id="1" r:id="rId1"/></p:sldLayoutIdLst>\n' +
    "</p:sldMaster>\n";

  const SLIDE_MASTER_RELS =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
    '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">\n' +
    '  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>\n' +
    "</Relationships>\n";

  const SLIDE_LAYOUT =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
    '<p:sldLayout xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" ' +
    'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" type="blank" preserve="1">\n' +
    '  <p:cSld name="Blank Slide"><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>\n' +
    '  <p:hf sldNum="0" hdr="0" ftr="0"/>\n' +
    "</p:sldLayout>\n";

  const THEME =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
    '<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office">\n' +
    '  <a:themeElements>\n' +
    '    <a:clrScheme name="Office"><a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1><a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="1F497D"/></a:dk2><a:lt2><a:srgbClr val="EEECE1"/></a:lt2><a:accent1><a:srgbClr val="4F81BD"/></a:accent1><a:accent2><a:srgbClr val="C0504D"/></a:accent2><a:accent3><a:srgbClr val="9BBB59"/></a:accent3><a:accent4><a:srgbClr val="8064A2"/></a:accent4><a:accent5><a:srgbClr val="4BACC6"/></a:accent5><a:accent6><a:srgbClr val="F79646"/></a:accent6><a:hlink><a:srgbClr val="0000FF"/></a:hlink><a:folHlink><a:srgbClr val="800080"/></a:folHlink></a:clrScheme>\n' +
    '    <a:fontScheme name="Office"><a:majorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme>\n' +
    '    <a:fmtScheme name="Office"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:fillStyleLst><a:lnStyleLst><a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="12700"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="19050"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:bgFillStyleLst></a:fmtScheme>\n' +
    '  </a:themeElements>\n' +
    "</a:theme>\n";

  function escXml(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&apos;");
  }

  function emuFromPx(px) {
    return Math.round((px || 0) * 9525);
  }

  function extractText(el) {
    if (typeof el === "string") return el;
    if (el.text != null) return String(el.text);
    if (el.content != null) return String(el.content);
    if (el.value != null) return String(el.value);
    return "";
  }

  function shapeFromElement(el, shapeId) {
    const x = emuFromPx(el.x || 0);
    const y = emuFromPx(el.y || 0);
    const w = emuFromPx(el.width || el.w || 100);
    const h = emuFromPx(el.height || el.h || 50);
    const text = escXml(extractText(el));
    const fillColor = el.backgroundColor || el.fill || "FFFFFF";
    const fontSize = el.fontSize || 1800;
    return (
      `<p:sp><p:nvSpPr><p:cNvPr id="${shapeId}" name="Shape ${shapeId}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>` +
      `<p:spPr><a:xfrm xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:off x="${x}" y="${y}"/><a:ext cx="${w}" cy="${h}"/></a:xfrm><a:prstGeom xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" prst="rect"><a:avLst/></a:prstGeom><a:solidFill xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:srgbClr val="${escXml(fillColor.replace("#", ""))}"/></a:solidFill></p:spPr>` +
      `<p:txBody><a:bodyPr xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" wrap="square" rtlCol="0" anchor="t"/><a:lstStyle xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"/><a:p xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:r><a:rPr lang="en-US" sz="${fontSize}"/><a:t>${text}</a:t></a:r></a:p></p:txBody></p:sp>`
    );
  }

  function buildSlideXml(slide) {
    const shapes = [];
    let id = 2;
    if (slide.elements) {
      for (const el of slide.elements) {
        shapes.push(shapeFromElement(el, id));
        id++;
      }
    }
    return (
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" ' +
      'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" ' +
      'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">\n' +
      '<p:cSld><p:spTree>' +
      '<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>' +
      '<p:grpSpPr/>' +
      shapes.join("") +
      '</p:spTree></p:cSld>' +
      '<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>' +
      '</p:sld>\n'
    );
  }

  function buildPresentationXml(presentation, slideRelIds) {
    const slideRefs = slideRelIds.map((id, i) => `<p:sldId id="${i + 256}" r:id="${id}"/>`).join("");
    return (
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" ' +
      'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">\n' +
      `<p:sldMasterIdLst><p:sldMasterId id="1" r:id="rId1"/></p:sldMasterIdLst>` +
      `<p:sldIdLst>${slideRefs}</p:sldIdLst>` +
      '<p:sldSz cx="9144000" cy="6858000" type="screen4x3"/>' +
      '<p:notesSz cx="6858000" cy="9144000"/>' +
      '</p:presentation>\n'
    );
  }

  function buildCoreProps(title) {
    return (
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      '<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" ' +
      'xmlns:dc="http://purl.org/dc/elements/1.1/" ' +
      'xmlns:dcterms="http://purl.org/dc/terms/" ' +
      'xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">\n' +
      `<dc:title>${escXml(title || "Presentation")}</dc:title>\n` +
      `<dcterms:created xsi:type="dcterms:W3CDTF">${new Date().toISOString()}</dcterms:created>\n` +
      `<dcterms:modified xsi:type="dcterms:W3CDTF">${new Date().toISOString()}</dcterms:modified>\n` +
      "</cp:coreProperties>\n"
    );
  }

  function buildAppProps() {
    return (
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n' +
      "<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\" xmlns:vt=\"http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes\">\n" +
      "<Application>General Bots</Application>\n" +
      "</Properties>\n"
    );
  }

  function buildZip(parts) {
    function crc32(buf) {
      const table = (buildZip._crcTable ||= (() => {
        const t = new Uint32Array(256);
        for (let n = 0; n < 256; n++) {
          let c = n;
          for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
          t[n] = c >>> 0;
        }
        return t;
      })());
      let crc = 0 ^ -1;
      for (let i = 0; i < buf.length; i++) crc = (crc >>> 8) ^ table[(crc ^ buf[i]) & 0xff];
      return (crc ^ -1) >>> 0;
    }
    const enc = new TextEncoder();
    const local = [];
    const central = [];
    let offset = 0;
    for (const part of parts) {
      const data = typeof part.data === "string" ? enc.encode(part.data) : part.data;
      const name = enc.encode(part.name);
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

  function exportToPptx(presentation, opts) {
    opts = opts || {};
    const title = opts.title || "Presentation";
    const slides = (presentation && presentation.slides) || [];
    const slideRelIds = slides.map((_, i) => "rId" + (i + 2));
    const parts = [
      { name: "[Content_Types].xml", data: CONTENT_TYPES },
      { name: "_rels/.rels", data: ROOT_RELS },
      { name: "ppt/_rels/presentation.xml.rels", data: PRESENTATION_RELS + slideRelIds.map((id, i) => `<Relationship Id="${id}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide${i + 1}.xml"/>`).join("") },
      { name: "ppt/presentation.xml", data: buildPresentationXml(presentation || {}, slideRelIds) },
      { name: "ppt/slideMasters/_rels/slideMaster1.xml.rels", data: SLIDE_MASTER_RELS },
      { name: "ppt/slideMasters/slideMaster1.xml", data: SLIDE_MASTER },
      { name: "ppt/slideLayouts/slideLayout1.xml", data: SLIDE_LAYOUT },
      { name: "ppt/theme/theme1.xml", data: THEME },
      { name: "docProps/core.xml", data: buildCoreProps(title) },
      { name: "docProps/app.xml", data: buildAppProps() },
    ];
    slides.forEach((slide, i) => {
      parts.push({ name: "ppt/slides/_rels/slide" + (i + 1) + ".xml.rels", data: '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/></Relationships>\n' });
      parts.push({ name: "ppt/slides/slide" + (i + 1) + ".xml", data: buildSlideXml(slide) });
    });
    const blob = new Blob([buildZip(parts)], { type: "application/vnd.openxmlformats-officedocument.presentationml.presentation" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = (opts.filename || title || "presentation") + ".pptx";
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 5000);
    return blob.size;
  }

  window.PptxExport = { exportToPptx, buildSlideXml, buildPresentationXml };
})();
