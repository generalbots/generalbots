"use strict";
/* ExportImport module 02: XLSX (minimal, SpreadsheetML 2003 / Office Open XML stub)
 *
 * Produces an .xlsx file (OOXML zip with workbook.xml, sheet1.xml, [Content_Types].xml, _rels).
 * Uses a tiny store-only ZIP writer (no compression, "stored" method) — every byte of the
 * resulting file is uncompressed. This is ~5x larger than deflate but works in all browsers
 * without external libraries.
 *
 * Structure:
 *   [Content_Types].xml
 *   _rels/.rels
 *   xl/_rels/workbook.xml.rels
 *   xl/workbook.xml
 *   xl/worksheets/sheet1.xml
 *   xl/sharedStrings.xml
 *   xl/styles.xml
 */
(function (window) {
  const EI = window.ExportImportCSV;
  function download(filename, mime, data) { return EI.download(filename, mime, data); }

  function colLetter(idx) {
    let s = "";
    let n = idx;
    while (n >= 0) { s = String.fromCharCode(65 + (n % 26)) + s; n = Math.floor(n / 26) - 1; }
    return s;
  }

  function colToIdx(letters) { return EI.colToIdx(letters); }

  function buildXLSX(grid, opts) {
    const data = EI.readGridAsArray(grid, (opts && opts.range) || null);
    const shared = [];
    const sm = {};
    function si(s) {
      if (sm[s] !== undefined) return sm[s];
      sm[s] = shared.length;
      shared.push(s);
      return sm[s];
    }
    let rows = "";
    data.forEach((row, ri) => {
      let cells = "";
      row.forEach((val, ci) => {
        if (val == null || val === "") return;
        const ref = colLetter(ci) + (ri + 1);
        const n = parseFloat(val);
        if (!isNaN(n) && isFinite(n) && String(n) === String(val).trim()) {
          cells += '<c r="' + ref + '"><v>' + n + '</v></c>';
        } else {
          const ix = si(String(val));
          cells += '<c r="' + ref + '" t="s"><v>' + ix + '</v></c>';
        }
      });
      rows += '<row r="' + (ri + 1) + '">' + cells + '</row>';
    });

    const sheetXml = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">' +
        '<sheetData>' + rows + '</sheetData>' +
      '</worksheet>';

    const ssXml = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="' + shared.length + '" uniqueCount="' + shared.length + '">' +
        shared.map(s => '<si><t xml:space="preserve">' + escapeXml(s) + '</t></si>').join("") +
      '</sst>';

    const wbXml = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">' +
        '<sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets>' +
      '</workbook>';

    const wbRels = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
        '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>' +
      '</Relationships>';

    const rootRels = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
        '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>' +
      '</Relationships>';

    const ctXml = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
      '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">' +
        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>' +
        '<Default Extension="xml" ContentType="application/xml"/>' +
        '<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>' +
        '<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>' +
        '<Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>' +
      '</Types>';

    const files = {
      "[Content_Types].xml": ctXml,
      "_rels/.rels": rootRels,
      "xl/workbook.xml": wbXml,
      "xl/_rels/workbook.xml.rels": wbRels,
      "xl/worksheets/sheet1.xml": sheetXml,
      "xl/sharedStrings.xml": ssXml
    };

    const zip = makeZipStored(files);
    const blob = new Blob([zip], { type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" });
    download((opts && opts.filename) || "export.xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", blob);
    return blob;
  }

  function escapeXml(s) {
    return String(s).replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&apos;" })[c]);
  }

  function makeZipStored(files) {
    function crc32(buf) {
      const T = (typeof window !== "undefined" && window.crcTable) || (function () {
        const t = new Uint32Array(256);
        for (let n = 0; n < 256; n++) {
          let c = n;
          for (let k = 0; k < 8; k++) c = (c & 1) ? (0xEDB88320 ^ (c >>> 1)) : (c >>> 1);
          t[n] = c >>> 0;
        }
        window.crcTable = t;
        return t;
      })();
      let c = 0xFFFFFFFF;
      for (let i = 0; i < buf.length; i++) c = T[(c ^ buf[i]) & 0xFF] ^ (c >>> 8);
      return (c ^ 0xFFFFFFFF) >>> 0;
    }
    function strToBytes(s) {
      const out = new Uint8Array(s.length);
      for (let i = 0; i < s.length; i++) out[i] = s.charCodeAt(i) & 0xFF;
      return out;
    }
    function dosTime(d) {
      return ((d.getHours() & 0x1F) << 11) | ((d.getMinutes() & 0x3F) << 5) | (Math.floor(d.getSeconds() / 2) & 0x1F);
    }
    function dosDate(d) {
      return (((d.getFullYear() - 1980) & 0x7F) << 9) | (((d.getMonth() + 1) & 0x0F) << 5) | (d.getDate() & 0x1F);
    }

    const now = new Date();
    const time = dosTime(now);
    const date = dosDate(now);
    const encoder = new TextEncoder();
    const names = Object.keys(files);
    const localParts = [];
    const centralParts = [];
    let offset = 0;

    names.forEach(name => {
      const data = encoder.encode(files[name]);
      const crc = crc32(data);
      const nameBytes = strToBytes(name);

      const local = new Uint8Array(30 + nameBytes.length);
      const dv = new DataView(local.buffer);
      dv.setUint32(0, 0x04034b50, true);
      dv.setUint16(4, 20, true);
      dv.setUint16(6, 0, true);
      dv.setUint16(8, 0, true);
      dv.setUint16(10, time, true);
      dv.setUint16(12, date, true);
      dv.setUint32(14, crc, true);
      dv.setUint32(18, data.length, true);
      dv.setUint32(22, data.length, true);
      dv.setUint16(26, nameBytes.length, true);
      dv.setUint16(28, 0, true);
      local.set(nameBytes, 30);
      localParts.push(local, data);

      const central = new Uint8Array(46 + nameBytes.length);
      const cdv = new DataView(central.buffer);
      cdv.setUint32(0, 0x02014b50, true);
      cdv.setUint16(4, 20, true);
      cdv.setUint16(6, 20, true);
      cdv.setUint16(8, 0, true);
      cdv.setUint16(10, 0, true);
      cdv.setUint16(12, time, true);
      cdv.setUint16(14, date, true);
      cdv.setUint32(16, crc, true);
      cdv.setUint32(20, data.length, true);
      cdv.setUint32(24, data.length, true);
      cdv.setUint16(28, nameBytes.length, true);
      cdv.setUint16(30, 0, true);
      cdv.setUint16(32, 0, true);
      cdv.setUint16(34, 0, true);
      cdv.setUint16(36, 0, true);
      cdv.setUint32(38, 0, true);
      cdv.setUint32(42, offset, true);
      central.set(nameBytes, 46);
      centralParts.push(central);
      offset += local.length;
    });

    const centralStart = offset;
    const centralSize = centralParts.reduce((acc, p) => acc + p.length, 0);
    const end = new Uint8Array(22);
    const edv = new DataView(end.buffer);
    edv.setUint32(0, 0x06054b50, true);
    edv.setUint16(4, 0, true);
    edv.setUint16(6, 0, true);
    edv.setUint16(8, names.length, true);
    edv.setUint16(10, names.length, true);
    edv.setUint32(12, centralSize, true);
    edv.setUint32(16, centralStart, true);
    edv.setUint16(20, 0, true);

    let total = 0;
    localParts.forEach(p => total += p.length);
    centralParts.forEach(p => total += p.length);
    total += end.length;
    const out = new Uint8Array(total);
    let p = 0;
    localParts.forEach(part => { out.set(part, p); p += part.length; });
    centralParts.forEach(part => { out.set(part, p); p += part.length; });
    out.set(end, p);
    return out;
  }

  window.ExportImportXLSX = {
    buildXLSX: buildXLSX,
    colLetter: colLetter,
    colToIdx: colToIdx,
    makeZipStored: makeZipStored,
    escapeXml: escapeXml
  };
})(window);
