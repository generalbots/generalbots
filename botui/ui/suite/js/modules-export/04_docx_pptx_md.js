"use strict";
/* ExportImport module 04: DOCX (HTML→.doc fallback), PPTX (minimal OOXML), Markdown
 *
 * DOCX: full Office Open XML is complex. We use a pragmatic two-mode approach:
 *   1. .doc (Word 97-2003) — saves as HTML with .doc extension. Word opens it.
 *   2. .docx (OOXML) — uses the same minimal stored-zip from XLSX module.
 *
 * PPTX: minimal OOXML zip. Tables and charts supplied as structured slide data
 * are written as native DrawingML (p:graphicFrame/a:tbl and real chart parts).
 * Flat text is never dressed up as a table/chart — that silent degradation
 * (#2457) is what the server exporter already stopped doing.
 */
(function (window) {
  const EI = window.ExportImportCSV;
  const EIX = window.ExportImportXLSX;
  const PPTX_MIME = "application/vnd.openxmlformats-officedocument.presentationml.presentation";
  const DOCX_MIME = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
  function download(filename, mime, data) { return EI.download(filename, mime, data); }
  function escapeXml(s) { return EIX.escapeXml(s); }

  // ---- shared primitives (local only, no CDN) -------------------------
  const EMU_PER_PX = 9525;
  const MAX_ROWS = 50, MAX_COLS = 20, MAX_CELL_CHARS = 1000;
  const MAX_SERIES = 8, MAX_POINTS = 100, MAX_TEXT = 200, MAX_VALUE = 1e12;
  const BORDERS = '<a:lnL w="6350"><a:solidFill><a:srgbClr val="D9D9D9"/></a:solidFill></a:lnL>' +
    '<a:lnR w="6350"><a:solidFill><a:srgbClr val="D9D9D9"/></a:solidFill></a:lnR>' +
    '<a:lnT w="6350"><a:solidFill><a:srgbClr val="D9D9D9"/></a:solidFill></a:lnT>' +
    '<a:lnB w="6350"><a:solidFill><a:srgbClr val="D9D9D9"/></a:solidFill></a:lnB>';

  function finite(value, fallback) {
    const n = Number(value);
    return Number.isFinite(n) ? n : fallback;
  }
  function clamp(n, lo, hi) { return Math.min(hi, Math.max(lo, n)); }
  function emu(px) { return Math.round(finite(px, 0) * EMU_PER_PX); }
  function trunc(value, limit) { return String(value == null ? "" : value).slice(0, limit); }
  function hex(value, fallback) {
    const s = String(value == null ? "" : value).replace(/^#/, "");
    return /^[0-9a-fA-F]{6}$/.test(s) ? s : fallback;
  }
  function dimension(value, fallback) {
    const n = finite(value, fallback);
    return emu(clamp(n > 0 ? n : fallback, 1, 20000));
  }
  function xfrm(element, cx, cy) {
    return '<a:xfrm><a:off x="' + emu(element.x) + '" y="' + emu(element.y) +
      '"/><a:ext cx="' + cx + '" cy="' + cy + '"/></a:xfrm>';
  }
  function dataOf(element, key) {
    return (element.content && element.content[key]) || element[key] || null;
  }

  // ---- native table writer --------------------------------------------
  function cellOf(row, col, data) {
    const raw = Array.isArray(data.cells) && data.cells[row] && data.cells[row][col];
    const cell = raw && typeof raw === "object" ? raw : { content: raw };
    return {
      text: cell.content == null ? "" : String(cell.content),
      colspan: clamp(Math.floor(finite(cell.colspan, 1)), 1, MAX_COLS),
      rowspan: clamp(Math.floor(finite(cell.rowspan, 1)), 1, MAX_ROWS),
      style: cell.style || {}
    };
  }

  function tableFrameXml(element, id) {
    const data = dataOf(element, "table_data");
    let rows = 0, widest = 0;
    if (Array.isArray(data.cells)) {
      rows = data.cells.length;
      data.cells.forEach(row => { if (Array.isArray(row)) widest = Math.max(widest, row.length); });
    }
    rows = clamp(Math.floor(finite(data.rows, rows)), 1, MAX_ROWS);
    const cols = clamp(Math.floor(finite(data.cols, widest)), 1, MAX_COLS);
    const colFallback = finite(element.width, cols * 100) / cols;
    const rowFallback = finite(element.height, rows * 40) / rows;
    const widths = [], heights = [];
    for (let c = 0; c < cols; c++) widths.push(dimension((data.col_widths || [])[c], colFallback));
    for (let r = 0; r < rows; r++) heights.push(dimension((data.row_heights || [])[r], rowFallback));

    const covered = [];
    for (let r = 0; r < rows; r++) covered.push(new Array(cols).fill(""));
    let body = "";
    for (let r = 0; r < rows; r++) {
      let cells = "";
      for (let c = 0; c < cols; c++) {
        if (covered[r][c]) {
          const tag = covered[r][c] === "h" ? ' hMerge="1"' : ' vMerge="1"';
          cells += '<a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p/></a:txBody><a:tcPr' + tag + '/></a:tc>';
          continue;
        }
        const cell = cellOf(r, c, data);
        const spanC = Math.min(cell.colspan, cols - c);
        const spanR = Math.min(cell.rowspan, rows - r);
        for (let cc = c + 1; cc < c + spanC; cc++) covered[r][cc] = "h";
        for (let rr = r + 1; rr < r + spanR; rr++) {
          for (let cc = c; cc < c + spanC; cc++) covered[rr][cc] = "v";
        }
        let attrs = "";
        if (spanC > 1) attrs += ' gridSpan="' + spanC + '"';
        if (spanR > 1) attrs += ' rowSpan="' + spanR + '"';
        const color = hex(cell.style.color, "000000");
        const bold = /^(bold|[6-9]00)$/i.test(String(cell.style.font_weight || "")) ? ' b="1"' : "";
        const sz = clamp(Math.round(finite(cell.style.font_size, 14) * 100), 600, 7200);
        const paragraphs = trunc(cell.text, MAX_CELL_CHARS).split("\n").map(line =>
          '<a:p><a:r><a:rPr lang="en-US" sz="' + sz + '"' + bold +
          '><a:solidFill><a:srgbClr val="' + color + '"/></a:solidFill></a:rPr><a:t>' +
          escapeXml(line) + '</a:t></a:r></a:p>').join("");
        cells += '<a:tc><a:txBody><a:bodyPr wrap="square"/><a:lstStyle/>' + paragraphs +
          '</a:txBody><a:tcPr' + attrs + '>' + BORDERS + '</a:tcPr></a:tc>';
      }
      body += '<a:tr h="' + heights[r] + '">' + cells + '</a:tr>';
    }
    const grid = widths.map(w => '<a:gridCol w="' + w + '"/>').join("");
    const cx = widths.reduce((a, b) => a + b, 0);
    const cy = heights.reduce((a, b) => a + b, 0);
    return '<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="' + id + '" name="Table ' + id + '"/>' +
      '<p:cNvGraphicFramePr><a:graphicFrameLocks noGrp="1"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr>' +
      xfrm(element, cx, cy) + '<a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table">' +
      '<a:tbl><a:tblPr firstRow="1" bandRow="1"/><a:tblGrid>' + grid + '</a:tblGrid>' + body +
      '</a:tbl></a:graphicData></a:graphic></p:graphicFrame>';
  }

  // ---- native chart writer --------------------------------------------
  function chartPartXml(data, title) {
    const labels = Array.isArray(data.labels) ? data.labels : [];
    const datasets = Array.isArray(data.datasets) ? data.datasets : [];
    const type = String(data.chart_type || "bar");
    let kind = "barChart", dir = "col";
    if (type === "line") kind = "lineChart";
    else if (type === "pie") kind = "pieChart";
    else if (type === "doughnut") kind = "doughnutChart";
    else if (type === "area") kind = "areaChart";
    else if (type === "horizontal") dir = "bar";

    let series = "";
    datasets.slice(0, MAX_SERIES).forEach((dataset, index) => {
      const values = Array.isArray(dataset.data) ? dataset.data : [];
      const count = Math.min(values.length, labels.length, MAX_POINTS);
      let cats = "", vals = "";
      for (let p = 0; p < count; p++) {
        cats += '<c:pt idx="' + p + '"><c:v>' + escapeXml(trunc(labels[p], MAX_TEXT)) + '</c:v></c:pt>';
        const value = clamp(finite(values[p], 0), -MAX_VALUE, MAX_VALUE);
        vals += '<c:pt idx="' + p + '"><c:v>' + value + '</c:v></c:pt>';
      }
      const letter = String.fromCharCode(65 + index);
      series += '<c:ser><c:idx val="' + index + '"/><c:order val="' + index + '"/>' +
        '<c:tx><c:v>' + escapeXml(trunc(dataset.label, MAX_TEXT)) + '</c:v></c:tx>' +
        '<c:spPr><a:solidFill><a:srgbClr val="' + hex(dataset.color, "4F81BD") + '"/></a:solidFill></c:spPr>' +
        '<c:cat><c:strRef><c:f>Sheet1!$A$2:$A$' + (count + 1) + '</c:f><c:strCache><c:ptCount val="' +
        count + '"/>' + cats + '</c:strCache></c:strRef></c:cat>' +
        '<c:val><c:numRef><c:f>Sheet1!$' + letter + '$2:$' + letter + '$' + (count + 1) +
        '</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="' + count + '"/>' +
        vals + '</c:numCache></c:numRef></c:val></c:ser>';
    });

    const axes = (kind === "pieChart" || kind === "doughnutChart") ? "" :
      '<c:catAx><c:axId val="111111111"/><c:scaling><c:orientation val="minMax"/></c:scaling>' +
      '<c:delete val="0"/><c:axPos val="b"/><c:crossAx val="222222222"/><c:crosses val="autoZero"/>' +
      '<c:auto val="1"/></c:catAx><c:valAx><c:axId val="222222222"/>' +
      '<c:scaling><c:orientation val="minMax"/></c:scaling><c:delete val="0"/><c:axPos val="l"/>' +
      '<c:majorGridlines/><c:crossAx val="111111111"/><c:crosses val="autoZero"/>' +
      '<c:crossBetween val="between"/></c:valAx>';
    let group;
    if (kind === "pieChart") group = '<c:pieChart><c:varyColors val="1"/>' + series + '</c:pieChart>';
    else if (kind === "doughnutChart") group = '<c:doughnutChart><c:varyColors val="1"/><c:holeSize val="50"/>' + series + '</c:doughnutChart>';
    else if (kind === "lineChart") group = '<c:lineChart><c:grouping val="standard"/>' + series + '<c:axId val="111111111"/><c:axId val="222222222"/></c:lineChart>';
    else if (kind === "areaChart") group = '<c:areaChart><c:grouping val="standard"/>' + series + '<c:axId val="111111111"/><c:axId val="222222222"/></c:areaChart>';
    else group = '<c:barChart><c:barDir val="' + dir + '"/><c:grouping val="clustered"/>' + series + '<c:axId val="111111111"/><c:axId val="222222222"/></c:barChart>';
    const titleXml = title && String(title).trim()
      ? '<c:title><c:tx><c:rich><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US" sz="1400"/><a:t>' +
        escapeXml(trunc(title, MAX_TEXT)) + '</a:t></a:r></a:p></c:rich></c:tx><c:overlay val="0"/></c:title>'
      : "";
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" ' +
      'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" ' +
      'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">' +
      '<c:chart>' + titleXml + '<c:autoTitleDeleted val="0"/><c:plotArea><c:layout/>' + group + axes +
      '</c:plotArea><c:legend><c:legendPos val="b"/><c:overlay val="0"/></c:legend>' +
      '<c:plotVisOnly val="1"/><c:dispBlanksAs val="gap"/></c:chart></c:chartSpace>';
  }

  function chartFrameXml(element, id, charts, slideIndex) {
    const data = dataOf(element, "chart_data");
    const relId = "rIdC" + (charts.length + 1);
    const fileName = "chart" + (charts.length + 1) + ".xml";
    charts.push({
      relId: relId, fileName: fileName, slideIndex: slideIndex,
      xml: chartPartXml(data, element.title || (element.content && element.content.text))
    });
    return '<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="' + id + '" name="Chart ' + id + '"/>' +
      '<p:cNvGraphicFramePr><a:graphicFrameLocks noGrp="1"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr>' +
      xfrm(element, emu(finite(element.width, 640)), emu(finite(element.height, 480))) +
      '<a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart">' +
      '<c:chart r:id="' + relId + '"/></a:graphicData></a:graphic></p:graphicFrame>';
  }

  // ---- slide assembly -------------------------------------------------
  function textShapeXml(text, id, sz, bold, x, y, cx, cy) {
    const paragraphs = String(text == null ? "" : text).split("\n").map(line =>
      '<a:p><a:r><a:rPr lang="en-US" sz="' + sz + '"' + (bold ? ' b="1"' : "") +
      '><a:solidFill><a:srgbClr val="000000"/></a:solidFill></a:rPr><a:t>' + escapeXml(line) +
      '</a:t></a:r></a:p>').join("");
    return '<p:sp><p:nvSpPr><p:cNvPr id="' + id + '" name="Text ' + id + '"/><p:cNvSpPr txBox="1"/>' +
      '<p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="' + x + '" y="' + y + '"/><a:ext cx="' + cx +
      '" cy="' + cy + '"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:noFill/></p:spPr>' +
      '<p:txBody><a:bodyPr wrap="square"><a:spAutoFit/></a:bodyPr><a:lstStyle/>' + paragraphs + '</p:txBody></p:sp>';
  }

  function structuredElements(slide) {
    const out = [];
    if (Array.isArray(slide.elements)) slide.elements.forEach(e => { if (e) out.push(e); });
    if (Array.isArray(slide.tables)) slide.tables.forEach(t => out.push({ element_type: "table", table_data: t }));
    if (Array.isArray(slide.charts)) slide.charts.forEach(c => out.push({ element_type: "chart", chart_data: c }));
    return out;
  }

  function buildSlideXml(slide, index, charts) {
    const title = slide.title || "Slide " + (index + 1);
    const body = slide.body || slide.textContent || "";
    let shapes = textShapeXml(title, 1, 44, true, 0, 0, 9144000, 1143000) +
      textShapeXml(body, 2, 28, false, 0, 1500000, 9144000, 6000000);
    let id = 3;
    structuredElements(slide).forEach(element => {
      const type = element.element_type;
      if (type === "table") {
        if (dataOf(element, "table_data")) shapes += tableFrameXml(element, id++);
        else console.error("exportPPTX: table element without structured data was skipped");
      } else if (type === "chart") {
        if (dataOf(element, "chart_data")) shapes += chartFrameXml(element, id++, charts, index);
        else console.error("exportPPTX: chart element without structured data was skipped");
      }
      // No native writer for other element types: their text is not invented
      // as a fake table/chart (#2457).
    });
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" ' +
      'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" ' +
      'xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" ' +
      'xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">' +
      '<p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>' +
      '<p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/>' +
      '<a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>' + shapes + '</p:spTree></p:cSld></p:sld>';
  }

  // ---- DOCX / DOC ------------------------------------------------------
  function exportDOC(elem, opts) {
    if (opts && opts.format === "docx") return exportDOCX(elem, opts);
    const html = "<!DOCTYPE html><html><head><meta charset='utf-8'></head><body>" + elem.outerHTML + "</body></html>";
    download((opts && opts.filename) || "export.doc", "application/msword", html);
    return html;
  }

  function exportDOCX(elem, opts) {
    const text = elem.textContent || "";
    const paragraphs = Array.from(elem.querySelectorAll("p, h1, h2, h3, h4, h5, h6, li, blockquote")).map(p => {
      const tag = p.tagName.toLowerCase();
      const style = tag.startsWith("h") ? "Heading" + tag.charAt(1) : tag === "blockquote" ? "Quote" : "Normal";
      return '<w:p><w:pPr><w:pStyle w:val="' + style + '"/></w:pPr><w:r><w:t xml:space="preserve">' + escapeXml(p.textContent) + '</w:t></w:r></w:p>';
    });
    if (paragraphs.length === 0) paragraphs.push('<w:p><w:r><w:t xml:space="preserve">' + escapeXml(text) + '</w:t></w:r></w:p>');

    const docXml = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">' +
        '<w:body>' + paragraphs.join("") + '</w:body>' +
      '</w:document>';

    const files = {
      "[Content_Types].xml":
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">' +
          '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>' +
          '<Default Extension="xml" ContentType="application/xml"/>' +
          '<Override PartName="/word/document.xml" ContentType="' + DOCX_MIME + '"/>' +
        '</Types>',
      "_rels/.rels":
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
        '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
          '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>' +
        '</Relationships>',
      "word/document.xml": docXml
    };
    const zip = EIX.makeZipStored(files);
    const blob = new Blob([zip], { type: DOCX_MIME });
    download((opts && opts.filename) || "export.docx", DOCX_MIME, blob);
    return blob;
  }

  // ---- PPTX ------------------------------------------------------------
  function exportPPTX(slides, opts) {
    const arr = Array.isArray(slides) ? slides : Array.from(slides || []);
    const charts = [];
    const files = {};

    arr.forEach((slide, i) => {
      files["ppt/slides/slide" + (i + 1) + ".xml"] = buildSlideXml(slide || {}, i, charts);
    });
    arr.forEach((_, i) => {
      const slideCharts = charts.filter(chart => chart.slideIndex === i);
      if (!slideCharts.length) return;
      files["ppt/slides/_rels/slide" + (i + 1) + ".xml.rels"] =
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
        '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
        slideCharts.map(chart => '<Relationship Id="' + chart.relId +
          '" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart" Target="../charts/' +
          chart.fileName + '"/>').join("") + '</Relationships>';
    });
    charts.forEach(chart => { files["ppt/charts/" + chart.fileName] = chart.xml; });

    const presXml = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" ' +
      'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">' +
      '<p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>' +
      '<p:sldIdLst>' + arr.map((_, i) => '<p:sldId id="' + (256 + i) + '" r:id="rId' + (2 + i) + '"/>').join("") +
      '</p:sldIdLst></p:presentation>';
    files["ppt/presentation.xml"] = presXml;
    files["ppt/_rels/presentation.xml.rels"] =
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
      '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>' +
      arr.map((_, i) => '<Relationship Id="rId' + (2 + i) +
        '" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide' +
        (i + 1) + '.xml"/>').join("") + '</Relationships>';
    files["ppt/slideMasters/slideMaster1.xml"] =
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" ' +
      'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:cSld><p:spTree>' +
      '<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>' +
      '<p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/>' +
      '<a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr></p:spTree></p:cSld>' +
      '<p:clrMap accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" ' +
      'accent6="accent6" bg1="lt1" bg2="lt2" folHlink="folHlink" hlink="hlink" tx1="dk1" tx2="dk2"/>' +
      '</p:sldMaster>';

    files["[Content_Types].xml"] =
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">' +
      '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>' +
      '<Default Extension="xml" ContentType="application/xml"/>' +
      '<Override PartName="/ppt/presentation.xml" ContentType="' + PPTX_MIME + '.main+xml"/>' +
      '<Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>' +
      arr.map((_, i) => '<Override PartName="/ppt/slides/slide' + (i + 1) +
        '.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>').join("") +
      charts.map(chart => '<Override PartName="/ppt/charts/' + chart.fileName +
        '" ContentType="application/vnd.openxmlformats-officedocument.drawingml.chart+xml"/>').join("") +
      '</Types>';
    files["_rels/.rels"] =
      '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
      '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>' +
      '</Relationships>';

    const zip = EIX.makeZipStored(files);
    const blob = new Blob([zip], { type: PPTX_MIME });
    download((opts && opts.filename) || "export.pptx", PPTX_MIME, blob);
    return blob;
  }

  // ---- Markdown --------------------------------------------------------
  function exportMarkdown(elem, opts) {
    function walk(node, depth) {
      if (node.nodeType === 3) return node.textContent;
      if (node.nodeType !== 1) return "";
      const tag = node.tagName.toLowerCase();
      if (tag === "h1") return "# " + node.textContent + "\n\n";
      if (tag === "h2") return "## " + node.textContent + "\n\n";
      if (tag === "h3") return "### " + node.textContent + "\n\n";
      if (tag === "h4") return "#### " + node.textContent + "\n\n";
      if (tag === "p") return Array.from(node.childNodes).map(c => walk(c, depth)).join("") + "\n\n";
      if (tag === "strong" || tag === "b") return "**" + node.textContent + "**";
      if (tag === "em" || tag === "i") return "*" + node.textContent + "*";
      if (tag === "code") return "`" + node.textContent + "`";
      if (tag === "pre") return "```\n" + node.textContent + "\n```\n\n";
      if (tag === "a") return "[" + node.textContent + "](" + (node.getAttribute("href") || "") + ")";
      if (tag === "ul") return Array.from(node.children).map(c => "- " + c.textContent + "\n").join("") + "\n";
      if (tag === "ol") return Array.from(node.children).map((c, i) => (i + 1) + ". " + c.textContent + "\n").join("") + "\n";
      if (tag === "li") return "";
      if (tag === "blockquote") return "> " + node.textContent + "\n\n";
      if (tag === "br") return "\n";
      return Array.from(node.childNodes).map(c => walk(c, depth)).join("");
    }
    const md = walk(elem, 0);
    download((opts && opts.filename) || "export.md", "text/markdown;charset=utf-8", md);
    return md;
  }

  window.ExportImportDocs = {
    exportDOC: exportDOC,
    exportDOCX: exportDOCX,
    exportPPTX: exportPPTX,
    exportMarkdown: exportMarkdown
  };
})(window);
