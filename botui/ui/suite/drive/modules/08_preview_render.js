/* Drive preview — renderers for CSV, JSON, code, logs and Markdown (#2105).
 *
 * Every renderer appends into the body element of the shared preview shell
 * (08_preview.js) and never builds the shell itself. HTML is produced from
 * escaped text only: the tokenizers in 08_preview_parse.js escape every
 * token, and Markdown still goes through sanitizeHtml().
 */
"use strict";

var PREVIEW_ROW_LIMIT = 500;
var PREVIEW_COL_LIMIT = 100;
var PREVIEW_CHAR_LIMIT = 2000000;
var PREVIEW_TEXT_LIMIT = 400000;
var PREVIEW_BYTE_LIMIT = 4194304;

function drivePreviewSourceNotes(context) {
  if (!context || !context.byteLimited) return [];
  return ["Showing the first " + (PREVIEW_BYTE_LIMIT / 1048576).toFixed(0) + " MB of this file."];
}

var PREVIEW_LANG_BY_EXT = {
  js: "js", mjs: "js", cjs: "js", jsx: "js", ts: "ts", tsx: "ts",
  rs: "rs", py: "py", go: "go", java: "java", c: "c", h: "c",
  cpp: "cpp", cc: "cpp", hpp: "cpp", sql: "sql", html: "html",
  htm: "html", xml: "xml", svg: "xml", css: "css", scss: "css",
  sh: "sh", bash: "sh", bas: "bas", yml: "yaml", yaml: "yaml",
  toml: "toml", ini: "ini", cfg: "ini", conf: "ini"
};

function drivePreviewIsLogExt(ext) {
  return ext === "log" || ext === "out" || ext === "err";
}

function drivePreviewFormatCount(value) {
  var api = window.DrivePreviewParse;
  return api ? api.formatCount(value) : String(value);
}

function drivePreviewElement(tag, className, text) {
  var element = document.createElement(tag);
  if (className) element.className = className;
  if (text !== undefined && text !== null) element.textContent = String(text);
  return element;
}

function drivePreviewButton(label, onClick, extraClass) {
  var button = drivePreviewElement("button", "gb-preview-btn" + (extraClass ? " " + extraClass : ""), label);
  button.type = "button";
  button.addEventListener("click", onClick);
  return button;
}

function drivePreviewNoticeBar(notes, buttons) {
  var bar = drivePreviewElement("div", "gb-preview-notice");
  var text = drivePreviewElement("span", "gb-preview-notice-text", (notes || []).join(" "));
  bar.appendChild(text);
  (buttons || []).forEach(function (button) { bar.appendChild(button); });
  return bar;
}

/* Saves an already-fetched blob, honouring the desktop Save dialog when the
 * shell provides it (same behaviour as downloadFile in 02_api.js). */
function drivePreviewSaveBlob(blob, fileName) {
  if (!blob) {
    if (typeof showNotification === "function") showNotification("Nothing to download", "error");
    return;
  }
  try {
    if (window.GBShared && typeof window.GBShared.saveBlob === "function") {
      window.GBShared.saveBlob(blob, fileName).catch(function () {
        if (typeof showNotification === "function") showNotification("Download failed", "error");
      });
      return;
    }
    var url = URL.createObjectURL(blob);
    var link = document.createElement("a");
    link.href = url;
    link.download = fileName;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  } catch (err) {
    if (typeof showNotification === "function") showNotification("Download failed: " + err.message, "error");
  }
}

function drivePreviewDownloadButton(context, label) {
  return drivePreviewButton(label || "Download", function () {
    drivePreviewSaveBlob(context.blob, context.fileName);
  }, "is-primary");
}

function renderDrivePreviewError(body, title, detail) {
  var box = drivePreviewElement("div", "gb-preview-error");
  box.setAttribute("role", "alert");
  box.appendChild(drivePreviewElement("p", "gb-preview-error-title", title));
  if (detail) box.appendChild(drivePreviewElement("p", "gb-preview-error-detail", detail));
  body.appendChild(box);
}

function renderDrivePreviewMessage(body, message) {
  var box = drivePreviewElement("div", "gb-preview-loading");
  box.appendChild(drivePreviewElement("span", "gb-preview-spinner"));
  box.appendChild(drivePreviewElement("span", null, message));
  body.appendChild(box);
  return box;
}

/* Rendered view + lazily created raw view, driven by one toggle button. */
function drivePreviewAttachRawToggle(body, rawText, renderedEl, labels) {
  var texts = labels || {};
  var rawEl = drivePreviewElement("pre", "gb-preview-raw", rawText);
  rawEl.hidden = true;
  body.appendChild(rawEl);
  var button = drivePreviewButton(texts.show || "View raw", function () {
    var showingRaw = rawEl.hidden;
    rawEl.hidden = !showingRaw;
    renderedEl.hidden = showingRaw;
    button.textContent = showingRaw ? (texts.back || "View table") : (texts.show || "View raw");
    button.setAttribute("aria-pressed", showingRaw ? "true" : "false");
  });
  button.setAttribute("aria-pressed", "false");
  return button;
}

function renderDriveCsvPreview(body, text, context) {
  var api = window.DrivePreviewParse;
  if (!api) {
    renderDrivePreviewError(body, "CSV parser unavailable", "The Drive preview modules did not load. Refresh the page.");
    return;
  }
  var result = api.parseCsv(text, {
    maxRows: PREVIEW_ROW_LIMIT,
    maxCols: PREVIEW_COL_LIMIT,
    maxChars: PREVIEW_CHAR_LIMIT
  });
  if (!result.rows.length) {
    renderDrivePreviewError(body, "Empty table", "This file has no rows to preview. Use Download to inspect the raw file.");
    return;
  }

  var types = api.columnTypes(result.body);
  var wrap = drivePreviewElement("div", "gb-preview-table-wrap");
  var table = drivePreviewElement("table", "gb-preview-table");
  table.appendChild(drivePreviewElement("caption", "gb-preview-caption",
    "Preview of " + context.fileName + ": " + api.formatCount(result.shownRows) + " rows, " +
    api.formatCount(result.columns) + " columns (" + result.delimiterLabel + "-separated)"));

  var thead = document.createElement("thead");
  var headRow = document.createElement("tr");
  result.header.forEach(function (cell, column) {
    var th = drivePreviewElement("th", "gb-preview-th gb-col-" + (types[column] || "text"), cell);
    th.scope = "col";
    headRow.appendChild(th);
  });
  thead.appendChild(headRow);
  table.appendChild(thead);

  var tbody = document.createElement("tbody");
  result.body.forEach(function (cells, rowIndex) {
    var tr = drivePreviewElement("tr", rowIndex % 2 === 1 ? "gb-preview-row is-odd" : "gb-preview-row");
    cells.forEach(function (cell, column) {
      var kind = api.classifyCell(cell);
      tr.appendChild(drivePreviewElement("td",
        "gb-preview-td gb-cell-" + kind + " gb-col-" + (types[column] || "text"), cell));
    });
    tbody.appendChild(tr);
  });
  table.appendChild(tbody);
  wrap.appendChild(table);
  body.appendChild(wrap);

  var rawButton = drivePreviewAttachRawToggle(body, text, wrap, { show: "View raw", back: "View table" });
  var notes = drivePreviewSourceNotes(context).concat(api.truncationNotes(result));
  body.insertBefore(drivePreviewNoticeBar(notes, [drivePreviewDownloadButton(context, "Download full file"), rawButton]), wrap);
}

function drivePreviewCodeMarkup(code, options) {
  var api = window.DrivePreviewParse;
  var opts = options || {};
  if (!api) {
    return String(code == null ? "" : code)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }
  if (opts.mode === "log") return api.highlightLog(code);
  if (opts.mode === "json") return api.highlightJson(code);
  if (opts.mode === "text") return api.escapeHtml(code);
  return api.highlightCode(code, opts.lang);
}

function renderDriveCodeBlock(body, code, options) {
  var opts = options || {};
  var shell = drivePreviewElement("div", "gb-preview-code" + (opts.wrap === false ? "" : " is-wrap"));
  var gutter = drivePreviewElement("div", "gb-preview-gutter");
  gutter.setAttribute("aria-hidden", "true");
  var lines = code.split("\n");
  var numbers = [];
  for (var index = 1; index <= lines.length; index++) numbers.push(String(index));
  gutter.textContent = numbers.join("\n");

  var pre = drivePreviewElement("pre", "gb-preview-pre");
  pre.innerHTML = drivePreviewCodeMarkup(code, opts);
  shell.appendChild(gutter);
  shell.appendChild(pre);
  body.appendChild(shell);

  var wrapButton = drivePreviewButton("Wrap", function () {
    var wrapped = shell.classList.toggle("is-wrap");
    wrapButton.setAttribute("aria-pressed", wrapped ? "true" : "false");
  });
  wrapButton.setAttribute("aria-pressed", shell.classList.contains("is-wrap") ? "true" : "false");

  var copyButton = drivePreviewButton("Copy", function () {
    drivePreviewCopy(code, copyButton);
  });

  var buttons = [copyButton, wrapButton];
  if (opts.context) buttons.push(drivePreviewDownloadButton(opts.context));
  if (opts.rawText !== undefined) {
    buttons.push(drivePreviewAttachRawToggle(body, opts.rawText, shell, { show: "View raw", back: "View formatted" }));
  }
  body.insertBefore(drivePreviewNoticeBar(opts.notes, buttons), shell);
}

function drivePreviewCopy(text, button) {
  function done() {
    var previous = button.textContent;
    button.textContent = "Copied";
    setTimeout(function () { button.textContent = previous; }, 1500);
  }
  try {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(done, function () {
        if (typeof showNotification === "function") showNotification("Copy failed", "error");
      });
      return;
    }
  } catch (err) {
    // fall through to the textarea fallback
  }
  try {
    var area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "readonly");
    area.className = "gb-preview-copy-source";
    document.body.appendChild(area);
    area.select();
    document.execCommand("copy");
    document.body.removeChild(area);
    done();
  } catch (err) {
    if (typeof showNotification === "function") showNotification("Copy failed", "error");
  }
}

function renderDriveJsonPreview(body, text, context) {
  var notes = drivePreviewSourceNotes(context);
  var formatted = text;
  try {
    formatted = JSON.stringify(JSON.parse(text), null, 2);
  } catch (err) {
    notes.push("Invalid JSON: " + err.message + ". Showing the raw text.");
    formatted = text;
  }
  if (formatted.length > PREVIEW_TEXT_LIMIT) {
    formatted = formatted.slice(0, PREVIEW_TEXT_LIMIT);
    notes.push("Only the first " + drivePreviewFormatCount(PREVIEW_TEXT_LIMIT) + " characters are shown.");
  }
  renderDriveCodeBlock(body, formatted, {
    mode: "json",
    notes: notes,
    context: context,
    rawText: text,
    wrap: false
  });
}

function renderDriveTextPreview(body, text, context) {
  var notes = drivePreviewSourceNotes(context);
  var limited = text;
  if (text.length > PREVIEW_TEXT_LIMIT) {
    limited = text.slice(0, PREVIEW_TEXT_LIMIT);
    notes.push("Only the first " + drivePreviewFormatCount(PREVIEW_TEXT_LIMIT) + " characters are shown.");
  }
  if (drivePreviewIsLogExt(context.ext)) {
    renderDriveCodeBlock(body, limited, { mode: "log", notes: notes, context: context, wrap: false });
    return;
  }
  renderDriveCodeBlock(body, limited, {
    mode: "code",
    lang: PREVIEW_LANG_BY_EXT[context.ext] || context.ext,
    notes: notes,
    context: context,
    wrap: false
  });
}

function renderDriveMarkdownPreview(body, text, context) {
  var limited = text.length > PREVIEW_TEXT_LIMIT ? text.slice(0, PREVIEW_TEXT_LIMIT) : text;
  var sourceNotes = drivePreviewSourceNotes(context);
  if (typeof marked === "undefined") {
    renderDriveCodeBlock(body, limited, {
      mode: "text",
      notes: sourceNotes.concat(["Markdown rendering is unavailable; showing plain text."]),
      context: context
    });
    return;
  }
  var rendered = drivePreviewElement("div", "gb-preview-markdown");
  rendered.innerHTML = sanitizeHtml(marked.parse(limited));
  body.appendChild(rendered);
  var rawButton = drivePreviewAttachRawToggle(body, text, rendered, { show: "View raw", back: "View rendered" });
  body.insertBefore(drivePreviewNoticeBar(sourceNotes, [drivePreviewDownloadButton(context), rawButton]), rendered);
}
