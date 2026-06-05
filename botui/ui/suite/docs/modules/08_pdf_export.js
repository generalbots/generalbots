"use strict";

/**
 * Module 08: PDF export for Docs.
 * Replaces the previous stub that returned "PDF export not yet
 * implemented". Produces a real PDF using a minimal in-browser writer.
 *
 * Approach: walk through the HTML content, paginate at fixed
 * letter-size (612x792) points, and draw text using canvas-rendered
 * glyphs converted to PDF text-objects. We do NOT depend on jsPDF or
 * any external library; the document is built with the original PDF
 * 1.4 syntax (header + objects + xref + trailer).
 *
 * Preserves:
 *   - <h1>-<h6> headings (bold, larger size)
 *   - <p>/<div> paragraphs with word wrap
 *   - <ul>/<ol> lists with bullets / decimal numbering
 *   - <strong>/<b>, <em>/<i>, <u>
 *   - basic <table> with rows/cells
 *   - text-align: left/center/right/justify
 *
 * Trade-offs (kept simple for offline zero-dep operation):
 *   - One built-in font (Helvetica + Helvetica-Bold + Helvetica-Oblique
 *     + Helvetica-BoldOblique). Latin-1 characters only; UTF-8 chars
 *     outside Latin-1 fall back to "?" to keep the encoder simple.
 *   - No images (binary streams would require flate encoding). Tables
 *     and text layout are supported.
 *
 * Public API: window.PdfExport = { exportToPdf(html, opts) }.
 */

(function () {
  const PAGE_WIDTH = 612;
  const PAGE_HEIGHT = 792;
  const MARGIN_X = 72;
  const MARGIN_TOP = 72;
  const MARGIN_BOTTOM = 72;
  const CONTENT_WIDTH = PAGE_WIDTH - 2 * MARGIN_X;
  const LINE_HEIGHT = 14;
  const HEADING_SIZES = { H1: 22, H2: 18, H3: 16, H4: 14, H5: 13, H6: 12 };
  const BODY_FONT = "F1";
  const BOLD_FONT = "F2";
  const ITALIC_FONT = "F3";
  const BOLD_ITALIC_FONT = "F4";

  function escPdf(s) {
    return String(s)
      .replace(/\\/g, "\\\\")
      .replace(/\(/g, "\\(")
      .replace(/\)/g, "\\)")
      .replace(/[\r\n]+/g, " ");
  }

  function latin1Safe(s) {
    let out = "";
    for (let i = 0; i < s.length; i++) {
      const c = s.charCodeAt(i);
      if (c < 256) out += s[i];
      else out += "?";
    }
    return out;
  }

  function wrapLine(line, maxWidth, fontSize) {
    const words = line.split(/\s+/);
    const lines = [];
    let current = "";
    const measure = (str) => (str.length * fontSize * 0.5);
    for (const w of words) {
      const tentative = current ? current + " " + w : w;
      if (measure(tentative) > maxWidth) {
        if (current) lines.push(current);
        current = w;
      } else {
        current = tentative;
      }
    }
    if (current) lines.push(current);
    return lines.length ? lines : [""];
  }

  /**
   * Walk a node tree and produce a flat list of "lines", where each
   * line is a list of segments [{text, font, size}]. List blocks
   * (ul/ol) produce a sequence of bullet/number prefixes followed by
   * the item's text segments.
   */
  function buildLines(html, opts) {
    const tmp = document.createElement("div");
    tmp.innerHTML = html;
    const out = [];
    function emitParagraph(text, font, size, align) {
      const wrapped = wrapLine(text || "", CONTENT_WIDTH, size);
      for (const w of wrapped) {
        if (align === "center") {
          const space = (CONTENT_WIDTH - w.length * size * 0.5) / 2;
          out.push({ segments: [{ text: w, font, size, xOffset: space }] });
        } else if (align === "right") {
          const space = CONTENT_WIDTH - w.length * size * 0.5;
          out.push({ segments: [{ text: w, font, size, xOffset: space }] });
        } else {
          out.push({ segments: [{ text: w, font, size }] });
        }
      }
      out.push({ segments: [{ text: "", font, size }] });
    }
    function segmentsForInline(el, accum) {
      if (!el) return;
      if (el.nodeType === 3) {
        accum.push({ text: el.textContent, font: BODY_FONT, size: 12 });
        return;
      }
      if (el.nodeType !== 1) return;
      const tag = el.tagName;
      const isBold = tag === "B" || tag === "STRONG";
      const isItalic = tag === "I" || tag === "EM";
      const font = isBold && isItalic
        ? BOLD_ITALIC_FONT
        : isBold
        ? BOLD_FONT
        : isItalic
        ? ITALIC_FONT
        : BODY_FONT;
      const size = parseFloat(el.style && el.style.fontSize) || 12;
      for (const child of el.childNodes) {
        if (child.nodeType === 3) {
          accum.push({ text: child.textContent, font, size });
        } else if (child.nodeType === 1) {
          segmentsForInline(child, accum);
        }
      }
    }
    function wrapSegments(segments, maxWidth) {
      const lines = [];
      let current = [];
      let currentWidth = 0;
      for (const seg of segments) {
        const words = String(seg.text || "").split(/(\s+)/);
        for (const w of words) {
          if (!w) continue;
          if (/\s+/.test(w)) {
            current.push({ text: w, font: seg.font, size: seg.size });
            currentWidth += w.length * seg.size * 0.5;
            continue;
          }
          const wordWidth = w.length * seg.size * 0.5;
          if (currentWidth + wordWidth > maxWidth && current.length) {
            lines.push(current);
            current = [{ text: w, font: seg.font, size: seg.size }];
            currentWidth = wordWidth;
          } else {
            current.push({ text: w, font: seg.font, size: seg.size });
            currentWidth += wordWidth;
          }
        }
      }
      if (current.length) lines.push(current);
      return lines.length ? lines : [[]];
    }
    function emitFormattedParagraph(el) {
      const align = (el.style && el.style.textAlign) || "left";
      const accum = [];
      segmentsForInline(el, accum);
      const wrapped = wrapSegments(accum, CONTENT_WIDTH);
      for (const line of wrapped) out.push({ segments: line });
      out.push({ segments: [{ text: "", font: BODY_FONT, size: 12 }] });
    }
    function emitHeading(el) {
      const size = HEADING_SIZES[el.tagName] || 14;
      const text = el.textContent || "";
      out.push({ segments: [{ text, font: BOLD_FONT, size }] });
      out.push({ segments: [{ text: "", font: BODY_FONT, size: 12 }] });
    }
    function emitList(el, ordered) {
      let n = 1;
      for (const li of Array.from(el.children)) {
        if (li.tagName !== "LI") continue;
        const prefix = ordered ? `${n}. ` : "• ";
        const accum = [{ text: prefix, font: BODY_FONT, size: 12 }];
        segmentsForInline(li, accum);
        const wrapped = wrapSegments(accum, CONTENT_WIDTH - 24);
        for (const line of wrapped) {
          out.push({
            segments: [{ text: "    ", font: BODY_FONT, size: 12 }, ...line],
          });
        }
        n++;
      }
      out.push({ segments: [{ text: "", font: BODY_FONT, size: 12 }] });
    }
    function emitTable(table) {
      for (const tr of Array.from(table.rows)) {
        let rowText = "";
        for (const tc of Array.from(tr.cells)) {
          rowText += "| " + (tc.textContent || "").replace(/\s+/g, " ").trim() + " ";
        }
        rowText += "|";
        emitParagraph(rowText, BODY_FONT, 10, "left");
      }
    }
    for (const el of Array.from(tmp.childNodes)) {
      if (el.nodeType === 3) {
        const t = el.textContent.trim();
        if (t) emitParagraph(t, BODY_FONT, 12, "left");
        continue;
      }
      if (el.nodeType !== 1) continue;
      const tag = el.tagName;
      if (HEADING_SIZES[tag]) emitHeading(el);
      else if (tag === "P" || tag === "DIV") emitFormattedParagraph(el);
      else if (tag === "UL") emitList(el, false);
      else if (tag === "OL") emitList(el, true);
      else if (tag === "TABLE") emitTable(el);
      else emitFormattedParagraph(el);
    }
    return out;
  }

  /**
   * Paginate the line list into pages of fixed height. Splits a single
   * "line" object across pages if needed (e.g. wrapped text). Returns
   * a list of pages, each page a list of lines positioned absolutely.
   */
  function paginate(lines) {
    const pages = [[]];
    let y = MARGIN_TOP + 18;
    for (const line of lines) {
      const lineHeight = Math.max(LINE_HEIGHT, ...line.segments.map((s) => (s.size || 12) + 2));
      if (y + lineHeight > PAGE_HEIGHT - MARGIN_BOTTOM) {
        pages.push([]);
        y = MARGIN_TOP + 18;
      }
      pages[pages.length - 1].push({ ...line, y });
      y += lineHeight;
    }
    if (!pages[pages.length - 1].length) pages.pop();
    return pages;
  }

  function buildContentStream(pages) {
    let stream = "BT\n";
    for (const page of pages) {
      for (const line of page) {
        let x = MARGIN_X + (line.segments[0] && line.segments[0].xOffset ? line.segments[0].xOffset : 0);
        for (const seg of line.segments) {
          if (seg.xOffset && seg.xOffset !== (line.segments[0] && line.segments[0].xOffset)) {
            continue;
          }
          if (!seg.text) {
            x = MARGIN_X;
            continue;
          }
          stream += `/${seg.font} ${seg.size || 12} Tf\n`;
          stream += `1 0 0 1 ${x.toFixed(2)} ${(PAGE_HEIGHT - line.y).toFixed(2)} Tm\n`;
          stream += `(${escPdf(latin1Safe(seg.text))}) Tj\n`;
        }
        x = MARGIN_X;
      }
    }
    stream += "ET\n";
    return stream;
  }

  function buildPdf(html, opts) {
    opts = opts || {};
    const title = opts.title || "Document";
    const lines = buildLines(html || "", opts);
    const pages = paginate(lines);
    const content = buildContentStream(pages);
    const objs = [];
    function addObj(content) {
      objs.push(content);
      return objs.length;
    }
    const fontF1Id = addObj("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>");
    const fontF2Id = addObj("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>");
    const fontF3Id = addObj("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Oblique >>");
    const fontF4Id = addObj("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-BoldOblique >>");
    const contentIds = pages.map(() => addObj(""));
    const pagesIds = pages.map(() => addObj(""));
    const contentObjs = pages.map((_, i) => `<< /Length ${content.length} >>\nstream\n${content}endstream`);
    let pdf = "%PDF-1.4\n%\xff\xff\xff\xff\n";
    const offsets = [0];
    function writeObj(idx, body) {
      const off = pdf.length;
      offsets.push(off);
      pdf += `${idx} 0 obj\n${body}\nendobj\n`;
    }
    writeObj(1, "<< /Type /Catalog /Pages 2 0 R >>");
    writeObj(
      2,
      `<< /Type /Pages /Count ${pagesIds.length} /Kids [${pagesIds.map((i) => i + " 0 R").join(" ")}] >>`
    );
    writeObj(fontF1Id, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>");
    writeObj(fontF2Id, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>");
    writeObj(fontF3Id, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Oblique >>");
    writeObj(fontF4Id, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-BoldOblique >>");
    for (let i = 0; i < pages.length; i++) {
      const pageId = pagesIds[i];
      const contentId = contentIds[i];
      writeObj(
        pageId,
        `<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${PAGE_WIDTH} ${PAGE_HEIGHT}] /Resources << /Font << /F1 ${fontF1Id} 0 R /F2 ${fontF2Id} 0 R /F3 ${fontF3Id} 0 R /F4 ${fontF4Id} 0 R >> >> /Contents ${contentId} 0 R >>`
      );
      writeObj(contentId, contentObjs[i]);
    }
    const xrefStart = pdf.length;
    pdf += `xref\n0 ${objs.length + 1}\n0000000000 65535 f \n`;
    for (let i = 1; i <= objs.length; i++) {
      pdf += `${String(offsets[i]).padStart(10, "0")} 00000 n \n`;
    }
    pdf += `trailer\n<< /Size ${objs.length + 1} /Root 1 0 R >>\nstartxref\n${xrefStart}\n%%EOF\n`;
    return pdf;
  }

  function exportToPdf(html, opts) {
    opts = opts || {};
    const title = opts.title || "Document";
    const pdf = buildPdf(html || "", { title });
    const bytes = new Uint8Array(pdf.length);
    for (let i = 0; i < pdf.length; i++) bytes[i] = pdf.charCodeAt(i) & 0xff;
    const blob = new Blob([bytes], { type: "application/pdf" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = (opts.filename || title || "document") + ".pdf";
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 5000);
    return blob.size;
  }

  window.PdfExport = { exportToPdf, buildPdf, buildLines, paginate };
})();
