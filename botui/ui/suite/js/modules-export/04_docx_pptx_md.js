"use strict";
/* ExportImport module 04: DOCX (HTML→.doc fallback), PPTX (minimal OOXML), Markdown
 *
 * DOCX: full Office Open XML is complex. We use a pragmatic two-mode approach:
 *   1. .doc (Word 97-2003) — saves as HTML with .doc extension. Word opens it.
 *   2. .docx (OOXML) — uses the same minimal stored-zip from XLSX module.
 *
 * PPTX: minimal OOXML zip with a single slide containing a text frame per slide.
 */
(function (window) {
  const EI = window.ExportImportCSV;
  const EIX = window.ExportImportXLSX;
  function download(filename, mime, data) { return EI.download(filename, mime, data); }
  function escapeXml(s) { return EIX.escapeXml(s); }

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
          '<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>' +
        '</Types>',
      "_rels/.rels":
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
        '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
          '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>' +
        '</Relationships>',
      "word/document.xml": docXml
    };
    const zip = EIX.makeZipStored(files);
    const blob = new Blob([zip], { type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document" });
    download((opts && opts.filename) || "export.docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document", blob);
    return blob;
  }

  function exportPPTX(slides, opts) {
    const arr = Array.isArray(slides) ? slides : Array.from(slides);
    let slideXmls = "";
    arr.forEach((slide, i) => {
      const title = (slide.title || "Slide " + (i + 1));
      const body = (slide.body || slide.textContent || "");
      slideXmls += '<w:sld><w:cSld><w:spTree>' +
        '<w:sp><w:spPr><w:xfrm><w:off x="0" y="0"/><w:ext cx="9144000" cy="1143000"/></w:xfrm></w:spPr><w:txbxContent>' +
          '<w:p><w:r><w:rPr><w:b/><w:sz w:val="44"/></w:rPr><w:t>' + escapeXml(title) + '</w:t></w:r></w:p>' +
        '</w:txbxContent></w:sp>' +
        '<w:sp><w:spPr><w:xfrm><w:off x="0" y="1500000"/><w:ext cx="9144000" cy="6000000"/></w:xfrm></w:spPr><w:txbxContent>' +
          '<w:p><w:r><w:rPr><w:sz w:val="28"/></w:rPr><w:t>' + escapeXml(body) + '</w:t></w:r></w:p>' +
        '</w:txbxContent></w:sp>' +
      '</w:spTree></w:cSld></w:sld>';
    });
    const presXml = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">' +
        '<p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>' +
        '<p:sldIdLst>' + arr.map((_, i) => '<p:sldId id="' + (256 + i) + '" r:id="rId' + (2 + i) + '"/>').join("") + '</p:sldIdLst>' +
        slideXmls +
      '</p:presentation>';
    const files = {
      "[Content_Types].xml":
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">' +
          '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>' +
          '<Default Extension="xml" ContentType="application/xml"/>' +
          '<Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>' +
          arr.map((_, i) => '<Override PartName="/ppt/slides/slide' + (i + 1) + '.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>').join("") +
        '</Types>',
      "_rels/.rels":
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
        '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
          '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>' +
        '</Relationships>',
      "ppt/presentation.xml": presXml
    };
    arr.forEach((slide, i) => {
      const title = (slide.title || "Slide " + (i + 1));
      const body = (slide.body || slide.textContent || "");
      files["ppt/slides/slide" + (i + 1) + ".xml"] =
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
        '<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">' +
          '<p:cSld><p:spTree>' +
            '<p:sp><p:spPr><a:xfrm xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:off x="0" y="0"/><a:ext cx="9144000" cy="1143000"/></a:xfrm></p:spPr><p:txBody>' +
              '<a:p xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:r><a:rPr b="1" sz="44"/><a:t>' + escapeXml(title) + '</a:t></a:r></a:p>' +
            '</p:txBody></p:sp>' +
            '<p:sp><p:spPr><a:xfrm xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:off x="0" y="1500000"/><a:ext cx="9144000" cy="6000000"/></a:xfrm></p:spPr><p:txBody>' +
              '<a:p xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:r><a:rPr sz="28"/><a:t>' + escapeXml(body) + '</a:t></a:r></a:p>' +
            '</p:txBody></p:sp>' +
          '</p:spTree></p:cSld>' +
        '</p:sld>';
    });
    const zip = EIX.makeZipStored(files);
    const blob = new Blob([zip], { type: "application/vnd.openxmlformats-officedocument.presentationml.presentation" });
    download((opts && opts.filename) || "export.pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation", blob);
    return blob;
  }

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
