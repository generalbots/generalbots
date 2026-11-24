/**
 * Dependency-free, store-only ZIP reader and OOXML text/count primitives.
 *
 * This is deliberately independent of the shipped makeZipStored() writer: the
 * corpus is packaged by the REAL helper, then read back here. A bug in either
 * side therefore shows up as a FAIL instead of being hidden by shared code.
 * All failures are short deterministic codes, so malformed input can be
 * fuzzed without exceptions escaping as process crashes.
 */
"use strict";

var LOCAL_SIG = 0x04034b50;
var CENTRAL_SIG = 0x02014b50;
var EOCD_SIG = 0x06054b50;

function u16(bytes, offset) {
  return bytes[offset] | (bytes[offset + 1] << 8);
}
function u32(bytes, offset) {
  return ((bytes[offset] | (bytes[offset + 1] << 8) |
    (bytes[offset + 2] << 16) | (bytes[offset + 3] << 24)) >>> 0);
}

function crc32(bytes) {
  var crc = 0xffffffff;
  for (var i = 0; i < bytes.length; i++) {
    crc ^= bytes[i];
    for (var bit = 0; bit < 8; bit++) {
      crc = (crc >>> 1) ^ (0xedb88320 & -(crc & 1));
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function decode(bytes) {
  return Buffer.from(bytes).toString("utf8").replace(/^\uFEFF/, "");
}

function decodeEntities(text) {
  return String(text)
    .replace(/&lt;/g, "<").replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"').replace(/&apos;/g, "'")
    .replace(/&#(\d+);/g, function (_all, digits) {
      return String.fromCharCode(Number(digits));
    })
    .replace(/&amp;/g, "&");
}

function normalizeText(text) {
  return decodeEntities(text).replace(/\s+/g, " ").trim();
}

/**
 * Parse a stored (uncompressed) ZIP into a name -> bytes map.
 * Hard caps make hostile input fail fast rather than exhaust memory.
 */
function readStoredZip(bytes, options) {
  var opts = options || {};
  var maxEntries = opts.maxEntries || 512;
  var maxTotal = opts.maxTotalBytes || 16 * 1024 * 1024;
  var maxName = opts.maxNameBytes || 4096;
  if (!bytes || bytes.length < 22) throw new Error("not-a-zip:too-short");
  if (u32(bytes, 0) !== LOCAL_SIG) throw new Error("not-a-zip:bad-local-signature");

  var searchStart = Math.max(0, bytes.length - 65557);
  var eocd = -1;
  for (var i = bytes.length - 22; i >= searchStart; i--) {
    if (u32(bytes, i) === EOCD_SIG) { eocd = i; break; }
  }
  if (eocd < 0) throw new Error("bad-zip:no-eocd");
  var total = u16(bytes, eocd + 8);
  if (total > maxEntries) throw new Error("limit:too-many-entries:" + total);
  var centralSize = u32(bytes, eocd + 12);
  var centralOffset = u32(bytes, eocd + 16);
  if (centralOffset + centralSize > bytes.length) throw new Error("bad-zip:central-out-of-range");

  var files = Object.create(null);
  var names = [];
  var totalBytes = 0;
  var cursor = centralOffset;
  for (var entry = 0; entry < total; entry++) {
    if (u32(bytes, cursor) !== CENTRAL_SIG) throw new Error("bad-zip:central-signature@" + entry);
    var method = u16(bytes, cursor + 10);
    if (method !== 0) throw new Error("bad-zip:unsupported-compression:" + method);
    var expectedCrc = u32(bytes, cursor + 16);
    var compressed = u32(bytes, cursor + 20);
    var uncompressed = u32(bytes, cursor + 24);
    var nameLength = u16(bytes, cursor + 28);
    var extraLength = u16(bytes, cursor + 30);
    var commentLength = u16(bytes, cursor + 32);
    var localOffset = u32(bytes, cursor + 42);
    if (nameLength > maxName) throw new Error("limit:name-too-long");
    if (compressed !== uncompressed) throw new Error("bad-zip:size-mismatch");
    totalBytes += uncompressed;
    if (totalBytes > maxTotal) throw new Error("limit:too-much-data");
    if (localOffset + 30 > bytes.length) throw new Error("bad-zip:local-out-of-range");

    var name = decode(bytes.subarray(cursor + 46, cursor + 46 + nameLength));
    if (name.charAt(0) === "/" || /\.\./.test(name)) throw new Error("bad-zip:unsafe-name:" + name);
    if (u32(bytes, localOffset) !== LOCAL_SIG) throw new Error("bad-zip:local-signature:" + name);
    var localNameLength = u16(bytes, localOffset + 26);
    var localExtraLength = u16(bytes, localOffset + 28);
    var dataStart = localOffset + 30 + localNameLength + localExtraLength;
    if (dataStart + compressed > bytes.length) throw new Error("bad-zip:data-out-of-range:" + name);
    var localName = decode(bytes.subarray(localOffset + 30, localOffset + 30 + localNameLength));
    if (localName !== name) throw new Error("bad-zip:name-mismatch:" + name);
    var data = bytes.subarray(dataStart, dataStart + compressed);
    if (crc32(data) !== expectedCrc) throw new Error("bad-zip:crc:" + name);
    files[name] = data;
    names.push(name);
    cursor += 46 + nameLength + extraLength + commentLength;
    if (cursor > bytes.length) throw new Error("bad-zip:central-overrun");
  }
  return { files: files, names: names, entries: total, centralOffset: centralOffset };
}

/**
 * Read entries by walking local headers from byte 0. This ignores the central
 * directory entirely, so a corpus file with a malformed central directory is
 * still scorable — the defect is reported separately instead of hiding the
 * structural content. Stored method and CRC are still verified per entry.
 */
function readStoredZipLocal(bytes, options) {
  var opts = options || {};
  var maxEntries = opts.maxEntries || 512;
  var maxTotal = opts.maxTotalBytes || 16 * 1024 * 1024;
  var maxName = opts.maxNameBytes || 4096;
  if (!bytes || bytes.length < 30) throw new Error("not-a-zip:too-short");
  var files = Object.create(null);
  var names = [];
  var totalBytes = 0;
  var cursor = 0;
  while (cursor + 30 <= bytes.length) {
    if (u32(bytes, cursor) !== LOCAL_SIG) break;
    var method = u16(bytes, cursor + 8);
    if (method !== 0) throw new Error("bad-zip:unsupported-compression:" + method);
    var expectedCrc = u32(bytes, cursor + 14);
    var size = u32(bytes, cursor + 18);
    var nameLength = u16(bytes, cursor + 26);
    var extraLength = u16(bytes, cursor + 28);
    if (nameLength > maxName) throw new Error("limit:name-too-long");
    if (names.length >= maxEntries) throw new Error("limit:too-many-entries");
    totalBytes += size;
    if (totalBytes > maxTotal) throw new Error("limit:too-much-data");
    var name = decode(bytes.subarray(cursor + 30, cursor + 30 + nameLength));
    if (!name) throw new Error("bad-zip:empty-name");
    if (name.charAt(0) === "/" || /\.\./.test(name)) throw new Error("bad-zip:unsafe-name:" + name);
    var dataStart = cursor + 30 + nameLength + extraLength;
    if (dataStart + size > bytes.length) throw new Error("bad-zip:data-out-of-range:" + name);
    var data = bytes.subarray(dataStart, dataStart + size);
    if (crc32(data) !== expectedCrc) throw new Error("bad-zip:crc:" + name);
    files[name] = data;
    names.push(name);
    cursor = dataStart + size;
  }
  if (!names.length) throw new Error("not-a-zip:no-local-entries");
  return { files: files, names: names, entries: names.length, mode: "local-scan" };
}

function readXml(files, name) {
  if (!Object.prototype.hasOwnProperty.call(files, name)) return null;
  return decode(files[name]);
}

/** Count opening tags by namespace-agnostic local name (prefix may vary). */
function countTag(xml, local) {
  if (!xml) return 0;
  var pattern = new RegExp("<(?:[A-Za-z_][\\w.-]*:)?" + local + "(?=[\\s/>])", "g");
  return (xml.match(pattern) || []).length;
}

/** Count elements carrying an attribute (local name) with an exact value. */
function countAttr(xml, attr, value) {
  if (!xml) return 0;
  var pattern = new RegExp("<[^>]*?\\b(?:[A-Za-z_][\\w.-]*:)?" +
    attr + "\\s*=\\s*[\"']" + value + "[\"'][^>]*>", "g");
  return (xml.match(pattern) || []).length;
}

/** Normalised signature of all <t> text nodes in a part set. */
function textOf(parts) {
  var pattern = /<(?:[A-Za-z_][\w.-]*:)?t(?:\s[^>]*)?>([\s\S]*?)<\/(?:[A-Za-z_][\w.-]*:)?t>/g;
  var chunks = [];
  parts.forEach(function (xml) {
    if (!xml) return;
    var match;
    pattern.lastIndex = 0;
    while ((match = pattern.exec(xml)) !== null) chunks.push(match[1]);
  });
  return normalizeText(chunks.join(" "));
}

module.exports = {
  LOCAL_SIG: LOCAL_SIG, CENTRAL_SIG: CENTRAL_SIG, EOCD_SIG: EOCD_SIG,
  u16: u16, u32: u32, crc32: crc32, decode: decode,
  decodeEntities: decodeEntities, normalizeText: normalizeText,
  readStoredZip: readStoredZip, readStoredZipLocal: readStoredZipLocal, readXml: readXml,
  countTag: countTag, countAttr: countAttr, textOf: textOf,
};
