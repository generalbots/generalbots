"use strict";
/* #2105 — Drive preview renderers (CSV table, JSON, code, log, Markdown).
 *
 * Zero-dependency tests (run: node tests/drive_preview_render.test.js). A tiny
 * DOM stub is enough because the renderers only use createElement,
 * appendChild/insertBefore, classList, attributes and textContent.
 */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const DRIVE = path.join(__dirname, "..", "ui", "suite", "drive", "modules");
const PARSE = path.join(DRIVE, "08_preview_parse.js");
const RENDER = path.join(DRIVE, "08_preview_render.js");

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

function makeElement(tag) {
  const element = {
    tagName: String(tag).toUpperCase(),
    children: [],
    className: "",
    textContent: "",
    innerHTML: "",
    hidden: false,
    attributes: {},
    classList: {
      add: function (name) {
        if (element.className.split(" ").indexOf(name) === -1) {
          element.className = (element.className + " " + name).trim();
        }
      },
      contains: function (name) {
        return element.className.split(" ").indexOf(name) !== -1;
      },
      toggle: function (name) {
        const has = element.classList.contains(name);
        if (has) {
          element.className = element.className.split(" ").filter((c) => c && c !== name).join(" ");
        } else {
          element.classList.add(name);
        }
        return !has;
      },
    },
    appendChild: function (child) {
      element.children.push(child);
      return child;
    },
    insertBefore: function (child, reference) {
      const index = element.children.indexOf(reference);
      if (index === -1) element.children.push(child);
      else element.children.splice(index, 0, child);
      return child;
    },
    remove: function () {
      element.children.length = 0;
    },
    setAttribute: function (name, value) {
      element.attributes[name] = String(value);
    },
    getAttribute: function (name) {
      return Object.prototype.hasOwnProperty.call(element.attributes, name)
        ? element.attributes[name]
        : null;
    },
    addEventListener: function () {},
    focus: function () {},
  };
  return element;
}

function loadModules() {
  const sandbox = {
    console: console,
    setTimeout: function (fn) { return fn; },
    document: {
      createElement: makeElement,
      body: makeElement("body"),
      getElementById: function () { return null; },
    },
    navigator: {},
    URL: { createObjectURL: function () { return "blob:test"; }, revokeObjectURL: function () {} },
  };
  sandbox.window = sandbox;
  vm.runInNewContext(fs.readFileSync(PARSE, "utf8"), sandbox, { filename: PARSE });
  vm.runInNewContext(fs.readFileSync(RENDER, "utf8"), sandbox, { filename: RENDER });
  return sandbox;
}

const sandbox = loadModules();
check(!!sandbox.DrivePreviewParse, "parser available to the renderer");

function findByClass(root, className) {
  if ((root.className || "").split(" ").indexOf(className) !== -1) return root;
  for (const child of root.children) {
    const hit = findByClass(child, className);
    if (hit) return hit;
  }
  return null;
}

// 1. CSV renders a real table with caption, header and typed cells.
let body = makeElement("div");
sandbox.renderDriveCsvPreview(body, 'name,age\n"Doe, John",31\nBob,42', {
  fileName: "people.csv",
  ext: "csv",
  blob: { size: 42 },
});

const table = findByClass(body, "gb-preview-table");
check(!!table, "table rendered");
const caption = table.children[0];
check(caption.tagName === "CAPTION", "caption is first");
check(/people\.csv/.test(caption.textContent), "caption names the file", caption.textContent);
check(/comma-separated/.test(caption.textContent), "caption names the delimiter", caption.textContent);
check(/3 rows/.test(caption.textContent), "caption counts rows", caption.textContent);

const headerCells = table.children[1].children[0].children;
check(headerCells.length === 2, "two header cells");
check(headerCells[0].tagName === "TH" && headerCells[0].scope === "col", "scoped headers");
check(headerCells[1].textContent === "age", "header text", headerCells[1].textContent);

const rows = table.children[2].children;
check(rows.length === 2, "two body rows", String(rows.length));
check(rows[0].children[0].textContent === "Doe, John", "quoted comma kept in one cell");
check(/gb-cell-number/.test(rows[0].children[1].className), "numeric cell marked");
check(rows[0].children[0].innerHTML === "", "cells use textContent, never innerHTML");

// 2. Truncation notice + download + raw toggle for a big table.
body = makeElement("div");
const bigRows = ["h1,h2"];
for (let i = 0; i < 1000; i++) bigRows.push(i + ",x" + i);
sandbox.renderDriveCsvPreview(body, bigRows.join("\n"), {
  fileName: "big.csv",
  ext: "csv",
  blob: { size: 9000 },
});
const notice = findByClass(body, "gb-preview-notice");
check(!!notice, "notice bar rendered");
check(/Showing first 499 of 1,000 rows\./.test(notice.children[0].textContent), "row count notice", notice.children[0].textContent);
check(notice.children.some((c) => c.textContent === "Download full file"), "download action offered");
check(notice.children.some((c) => c.textContent === "View raw"), "raw toggle offered");
const raw = findByClass(body, "gb-preview-raw");
check(raw && raw.hidden === true, "raw view starts hidden");
check(raw && raw.textContent.indexOf("x999") !== -1, "raw view keeps the full text");

// 3. Markup in a cell stays inert.
body = makeElement("div");
sandbox.renderDriveCsvPreview(body, 'a,b\n"<img src=x onerror=alert(1)>",2', {
  fileName: "evil.csv",
  ext: "csv",
  blob: { size: 10 },
});
const evilTable = findByClass(body, "gb-preview-table");
const evilCell = evilTable.children[2].children[0].children[0];
check(evilCell.textContent.indexOf("<img") === 0, "markup kept as text");
check(evilCell.innerHTML === "", "markup never reaches innerHTML");

// 4. Empty CSV degrades to an explicit error, not a blank pane.
body = makeElement("div");
sandbox.renderDriveCsvPreview(body, "", { fileName: "empty.csv", ext: "csv", blob: { size: 0 } });
const emptyError = findByClass(body, "gb-preview-error");
check(!!emptyError, "empty CSV shows an error");
check(emptyError.getAttribute("role") === "alert", "error is announced");

// 5. Invalid JSON explains itself and keeps the text.
body = makeElement("div");
sandbox.renderDriveJsonPreview(body, '{"a": }', { fileName: "broken.json", ext: "json", blob: { size: 8 } });
const jsonNotice = findByClass(body, "gb-preview-notice");
check(/Invalid JSON/.test(jsonNotice.children[0].textContent), "invalid JSON explained", jsonNotice.children[0].textContent);
check(findByClass(body, "gb-tok-key") === null, "no token spans for malformed input");
const jsonPre = findByClass(body, "gb-preview-pre");
check(jsonPre && jsonPre.textContent === "", "pre is filled via innerHTML tokens");

// 6. Code preview: line numbers, colours and wrap state.
body = makeElement("div");
sandbox.renderDriveTextPreview(body, 'const a = "x";\n// done', {
  fileName: "a.js",
  ext: "js",
  blob: { size: 20 },
});
const gutter = findByClass(body, "gb-preview-gutter");
check(gutter && gutter.textContent === "1\n2", "line numbers", gutter && gutter.textContent);
const codeShell = findByClass(body, "gb-preview-code");
check(codeShell && !codeShell.classList.contains("is-wrap"), "long lines start unwrapped");
const codePre = findByClass(body, "gb-preview-pre");
check(/gb-tok-kw/.test(codePre.innerHTML), "keywords coloured");
check(/gb-tok-com/.test(codePre.innerHTML), "comments coloured");

// 7. Log preview colours levels.
body = makeElement("div");
sandbox.renderDriveTextPreview(body, "2026-11-10T10:00:00Z ERROR boom", {
  fileName: "app.log",
  ext: "log",
  blob: { size: 30 },
});
const logPre = findByClass(body, "gb-preview-pre");
check(/gb-tok-err/.test(logPre.innerHTML), "log level coloured");
check(/gb-tok-ts/.test(logPre.innerHTML), "log timestamp coloured");

// 8. Markdown without the vendored library falls back to plain text.
body = makeElement("div");
sandbox.renderDriveMarkdownPreview(body, "# Title", { fileName: "r.md", ext: "md", blob: { size: 7 } });
const mdNotice = findByClass(body, "gb-preview-notice");
check(/Markdown rendering is unavailable/.test(mdNotice.children[0].textContent), "markdown fallback explained");

// 9. A byte-capped read says so.
body = makeElement("div");
sandbox.renderDriveTextPreview(body, "line", { fileName: "huge.log", ext: "log", blob: { size: 99999999 }, byteLimited: true });
const cappedNotice = findByClass(body, "gb-preview-notice");
check(/Showing the first 4 MB of this file\./.test(cappedNotice.children[0].textContent), "byte cap note", cappedNotice.children[0].textContent);

console.log(`drive_preview_render: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
