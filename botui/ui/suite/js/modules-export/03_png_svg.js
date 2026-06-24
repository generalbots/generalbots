"use strict";
/* ExportImport module 03: PNG (canvas), SVG (raw)
 *
 * For grid: rasterize via foreignObject → SVG → image → canvas → PNG.
 * For SVG: serialize live SVG elements to file.
 */
(function (window) {
  const EI = window.ExportImportCSV;
  function download(filename, mime, data) { return EI.download(filename, mime, data); }

  async function exportPNG(elem, opts) {
    const w = (opts && opts.width) || elem.offsetWidth || 800;
    const h = (opts && opts.height) || elem.offsetHeight || 600;
    const scale = (opts && opts.scale) || 2;
    const bg = (opts && opts.background) || "#0f172a";

    const styles = collectStyles(elem);
    const svgStr =
      '<svg xmlns="http://www.w3.org/2000/svg" width="' + w + '" height="' + h + '">' +
        '<foreignObject width="100%" height="100%">' +
          '<div xmlns="http://www.w3.org/1999/xhtml" style="background:' + bg + ';width:' + w + 'px;height:' + h + 'px;font-family:sans-serif;">' +
            '<style>' + styles + '</style>' +
            elem.outerHTML +
          '</div>' +
        '</foreignObject>' +
      '</svg>';

    const blob = new Blob([svgStr], { type: "image/svg+xml;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    try {
      const img = await loadImage(url);
      const canvas = document.createElement("canvas");
      canvas.width = w * scale;
      canvas.height = h * scale;
      const ctx = canvas.getContext("2d");
      ctx.fillStyle = bg;
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
      return new Promise((resolve) => {
        canvas.toBlob((pngBlob) => {
          download((opts && opts.filename) || "export.png", "image/png", pngBlob);
          URL.revokeObjectURL(url);
          resolve(pngBlob);
        }, "image/png");
      });
    } catch (e) {
      URL.revokeObjectURL(url);
      throw e;
    }
  }

  function loadImage(url) {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => resolve(img);
      img.onerror = (e) => reject(e);
      img.src = url;
    });
  }

  function collectStyles(root) {
    let css = "";
    for (const sheet of document.styleSheets) {
      try {
        for (const rule of sheet.cssRules) css += rule.cssText + "\n";
      } catch (_) {}
    }
    return css;
  }

  function exportSVG(elem, opts) {
    let svg = elem.outerHTML;
    if (opts && opts.standalone !== false) {
      const w = (opts && opts.width) || elem.getAttribute("width") || 800;
      const h = (opts && opts.height) || elem.getAttribute("height") || 600;
      svg = '<?xml version="1.0" encoding="UTF-8"?>\n' +
        '<svg xmlns="http://www.w3.org/2000/svg" width="' + w + '" height="' + h + '">\n' +
        elem.innerHTML + '\n</svg>';
    }
    download((opts && opts.filename) || "export.svg", "image/svg+xml;charset=utf-8", svg);
    return svg;
  }

  window.ExportImportImage = {
    exportPNG: exportPNG,
    exportSVG: exportSVG,
    loadImage: loadImage,
    collectStyles: collectStyles
  };
})(window);
