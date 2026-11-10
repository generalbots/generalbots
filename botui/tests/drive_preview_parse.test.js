"use strict";
/* #2105 — Drive preview: CSV parser and text tokenizers.
 *
 * Zero-dependency unit tests (run: node tests/drive_preview_parse.test.js).
 * The module under test is the same file the browser loads, evaluated in a
 * vm sandbox with no DOM, which is why the parser must stay pure.
 */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const MODULE = path.join(
  __dirname,
  "..",
  "ui",
  "suite",
  "drive",
  "modules",
  "08_preview_parse.js",
);

let passed = 0;
let failed = 0;

function check(condition, label, extra) {
  if (condition) {
    passed++;
  } else {
    failed++;
    console.error("FAIL " + label + (extra === undefined ? "" : "\n  " + extra));
  }
}

function assertEqual(actual, expected, label) {
  const a = JSON.stringify(actual);
  const e = JSON.stringify(expected);
  check(a === e, label, "expected: " + e + "\n  actual:   " + a);
}

function loadParser() {
  const sandbox = { console: console };
  sandbox.window = sandbox;
  vm.runInNewContext(fs.readFileSync(MODULE, "utf8"), sandbox, {
    filename: MODULE,
  });
  return sandbox.DrivePreviewParse;
}

const P = loadParser();
check(!!P, "module exports DrivePreviewParse");

// 1. Simple comma table.
let result = P.parseCsv("name,age\nAna,31\nBob,42");
assertEqual(result.delimiter, ",", "comma is detected");
assertEqual(result.rows, [["name", "age"], ["Ana", "31"], ["Bob", "42"]], "simple rows");
assertEqual(result.totalRows, 3, "total rows");
assertEqual(result.columns, 2, "columns");

// 2. Quoted comma stays inside one field.
result = P.parseCsv('name,note\n"Doe, John",hi');
assertEqual(result.body[0][0], "Doe, John", "quoted comma");
assertEqual(result.columns, 2, "quoted comma keeps two columns");

// 3. Quoted newline and escaped quotes.
result = P.parseCsv('a,b\n"line1\nline2",x\n"say ""hi""",y');
assertEqual(result.rows[1][0], "line1\nline2", "quoted newline");
assertEqual(result.rows[2][0], 'say "hi"', "escaped double quotes");
assertEqual(result.totalRows, 3, "quoted newlines do not add rows");

// 4. CRLF, lone CR and trailing terminator.
result = P.parseCsv("a,b\r\n1,2\r\n");
assertEqual(result.rows, [["a", "b"], ["1", "2"]], "CRLF without phantom row");
result = P.parseCsv("a,b\r1,2");
assertEqual(result.rows, [["a", "b"], ["1", "2"]], "lone CR terminates a record");

// 5. BOM and empty fields.
result = P.parseCsv("\uFEFFid,v\n1,");
assertEqual(result.header, ["id", "v"], "BOM stripped from the header");
assertEqual(result.body[0], ["1", ""], "trailing empty field kept");
assertEqual(P.classifyCell(""), "empty", "empty cell type");

// 6. Ragged rows are padded and counted.
result = P.parseCsv("a,b,c\n1,2\n3,4,5,6");
assertEqual(result.rows[1], ["1", "2", "", ""], "short row padded");
assertEqual(result.columns, 4, "extra column widens the table");
assertEqual(result.raggedRows, 2, "ragged rows counted");

// 7. Semicolon, tab and pipe delimiters.
assertEqual(P.parseCsv("a;b;c\n1;2;3").delimiter, ";", "semicolon detected");
assertEqual(P.parseCsv("a\tb\n1\t2").delimiter, "\t", "tab detected");
assertEqual(P.parseCsv("a|b\n1|2").delimiter, "|", "pipe detected");
assertEqual(P.parseCsv("a;b\n1;2").rows[1], ["1", "2"], "semicolon splits fields");

// 8. Row limit keeps counts honest.
let big = ["h1,h2"];
for (let i = 0; i < 1000; i++) big.push(i + ",x" + i);
result = P.parseCsv(big.join("\n"), { maxRows: 10 });
assertEqual(result.shownRows, 10, "row limit caps rendered rows");
assertEqual(result.totalRows, 1001, "row limit still counts every row");
check(result.truncatedRows === true, "truncatedRows flag");
assertEqual(P.truncationNotes(result)[0], "Showing first 9 of 1,000 rows.", "row note");

// 9. Column limit.
result = P.parseCsv("a,b,c,d,e\n1,2,3,4,5", { maxCols: 2 });
assertEqual(result.columns, 2, "column limit caps rendered columns");
assertEqual(result.totalCols, 5, "column limit still counts every column");
check(result.truncatedCols === true, "truncatedCols flag");
assertEqual(P.truncationNotes(result)[0], "Showing first 2 of 5 columns.", "column note");

// 10. Character cap drops the record it cut in half.
const longCsv = ["a,b", "1,2", "3,4", "5,6"].join("\n");
result = P.parseCsv(longCsv, { maxChars: 10 });
check(result.sourceTruncated === true, "character cap flag");
assertEqual(result.totalRows, 2, "partial trailing record dropped");
assertEqual(result.rows[1], ["1", "2"], "complete rows survive the cap");
check(/Only the first/.test(P.truncationNotes(result).join(" ")), "size note");

// 11. Cell typing and per-column alignment.
assertEqual(P.classifyCell("42"), "number", "integer");
assertEqual(P.classifyCell("-1.5e3"), "number", "exponent");
assertEqual(P.classifyCell("12%"), "number", "percentage");
assertEqual(P.classifyCell("2026-11-10"), "date", "ISO date");
assertEqual(P.classifyCell("10/11/2026"), "date", "day/month/year date");
assertEqual(P.classifyCell("true"), "boolean", "boolean");
assertEqual(P.classifyCell("  "), "empty", "blank");
assertEqual(P.classifyCell("abc"), "text", "text");
assertEqual(P.columnTypes([["1", "x"], ["2", "y"]]), ["number", "text"], "column types");

// 12. JSON highlighting escapes and colours.
let html = P.highlightJson('{"a": 1, "b": [true, null], "s": "x"}');
check(html.indexOf('class="gb-tok-key"') !== -1, "json keys coloured");
check(html.indexOf('class="gb-tok-num"') !== -1, "json numbers coloured");
check(html.indexOf('class="gb-tok-bool"') !== -1, "json booleans coloured");
html = P.highlightJson('{"a": "<script>alert(1)</script>"}');
check(html.indexOf("<script") === -1, "json values escaped");

// 13. Code highlighting for JS, Rust and SQL.
html = P.highlightCode('// note\nconst x = "a<b";', "js");
check(html.indexOf('class="gb-tok-com"') !== -1, "js comment coloured");
check(html.indexOf('class="gb-tok-kw"') !== -1, "js keyword coloured");
check(html.indexOf('class="gb-tok-str"') !== -1, "js string coloured");
check(html.indexOf("<b>") === -1, "code text escaped");
check(P.highlightCode("fn main() {}", "rs").indexOf('class="gb-tok-kw"') !== -1, "rust keyword");
check(P.highlightCode("SELECT * FROM t -- x", "sql").indexOf('class="gb-tok-com"') !== -1, "sql comment");
check(P.highlightCode("# note\nx = 1", "py").indexOf('class="gb-tok-com"') !== -1, "python comment");

// 14. Log highlighting.
html = P.highlightLog("2026-11-10T10:00:00Z ERROR boom\n2026-11-10 10:00:01 WARN slow");
check(html.indexOf('class="gb-tok-err"') !== -1, "error level coloured");
check(html.indexOf('class="gb-tok-warn"') !== -1, "warn level coloured");
check(html.indexOf('class="gb-tok-ts"') !== -1, "timestamps coloured");

// 15. Escaping and number formatting helpers.
assertEqual(P.escapeHtml('<img src=x onerror="a">'), "&lt;img src=x onerror=&quot;a&quot;&gt;", "escapeHtml");
assertEqual(P.formatCount(12345), "12,345", "thousands separator");
assertEqual(P.formatCount(7), "7", "small count");

console.log(`drive_preview_parse: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
