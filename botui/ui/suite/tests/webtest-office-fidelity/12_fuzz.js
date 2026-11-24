/**
 * Offline importer/container fuzzing: malformed ZIPs, hostile names, CRC
 * corruption and resource caps must fail deterministically, never crash.
 * A real 24h fuzz run against the Rust importers remains a CI/hardware gate;
 * this is the browser-side, deterministic subset of that work.
 */
"use strict";

var h = require("./00_harness");
var host = require("./01_host");
var zip = require("./02_zip");

var ctx = require("./10_roundtrip").ctx;
var valid = host.blobBytes(ctx.XLSX.buildXLSX(host.makeGrid([["A"], ["1"]])));

function copy(bytes) { return new Uint8Array(bytes); }
function setU16(bytes, offset, value) {
  bytes[offset] = value & 0xff;
  bytes[offset + 1] = (value >>> 8) & 0xff;
}

function findEocd(bytes) {
  for (var i = bytes.length - 22; i >= Math.max(0, bytes.length - 65557); i--) {
    if (zip.u32(bytes, i) === zip.EOCD_SIG) return i;
  }
  return -1;
}

h.test("fuzz empty and short inputs fail deterministically", function () {
  [null, new Uint8Array(0), new Uint8Array(4), new Uint8Array(21)].forEach(function (input) {
    h.assertThrows(function () { zip.readStoredZip(input); }, /not-a-zip/);
    h.assertThrows(function () { zip.readStoredZipLocal(input); }, /not-a-zip:too-short/);
  });
});

h.test("fuzz junk with a local signature never yields a part", function () {
  var junk = new Uint8Array(64);
  junk[0] = 0x50; junk[1] = 0x4b; junk[2] = 0x03; junk[3] = 0x04;
  h.assertThrows(function () { zip.readStoredZip(junk); }, /no-eocd/);
  h.assertThrows(function () { zip.readStoredZipLocal(junk); }, /empty-name/);
});

h.test("fuzz corrupted central directory is rejected", function () {
  var bytes = copy(valid);
  var eocd = findEocd(bytes);
  h.assert(eocd >= 0, "valid fixture has EOCD");
  var central = zip.u32(bytes, eocd + 16);
  bytes[central] = 0x00;
  h.assertThrows(function () { zip.readStoredZip(bytes); }, /central-signature/);
});

h.test("fuzz flipped data byte is caught by CRC", function () {
  var bytes = copy(valid);
  var nameLength = zip.u16(bytes, 26);
  var dataStart = 30 + nameLength + zip.u16(bytes, 28);
  bytes[dataStart] = bytes[dataStart] ^ 0xff;
  h.assertThrows(function () { zip.readStoredZipLocal(bytes); }, /bad-zip:crc/);
});

h.test("fuzz huge declared entry count is capped", function () {
  var bytes = copy(valid);
  var eocd = findEocd(bytes);
  setU16(bytes, eocd + 8, 60000);
  h.assertThrows(function () { zip.readStoredZip(bytes, { maxEntries: 512 }); }, /too-many-entries/);
});

h.test("fuzz deflate method is rejected (stored only)", function () {
  var bytes = copy(valid);
  setU16(bytes, 8, 8);
  h.assertThrows(function () { zip.readStoredZipLocal(bytes); }, /unsupported-compression/);
});

h.test("fuzz path traversal in entry name is rejected", function () {
  var bytes = ctx.XLSX.makeZipStored({ "../evil.xml": "<x/>" });
  h.assertThrows(function () { zip.readStoredZipLocal(bytes); }, /unsafe-name/);
});

h.test("fuzz resource caps bound work before allocation", function () {
  h.assertThrows(function () { zip.readStoredZipLocal(valid, { maxEntries: 1 }); }, /too-many-entries/);
  h.assertThrows(function () { zip.readStoredZipLocal(valid, { maxTotalBytes: 1 }); }, /too-much-data/);
  h.assertThrows(function () { zip.readStoredZipLocal(valid, { maxNameBytes: 2 }); }, /name-too-long/);
});

h.test("fuzz XML primitives tolerate hostile strings [docx:text]", function () {
  var hostile = "\u0000<w:p>&amp;<w:p \u0000bad='1'>";
  h.assertEqual(zip.countTag(hostile, "p"), 2, "counts local names without crashing");
  h.assertEqual(zip.countTag("\u0000\u0001", "p"), 0, "binary junk has no tags");
  h.assertEqual(zip.textOf(["<w:t>a</w:t>", "<a:t>b</a:t>"]), "a b", "text signature");
});

h.note("Out of scope offline: 24h Rust importer fuzzing with OOM/timeout caps " +
  "(criterion 5), rasterised visual diff, CI dashboard, four-browser smoke matrix.");
