/* Drive preview — dependency-free parsers and text tokenizers (#2105).
 *
 * Loaded as a classic script by drive-module-loader.js and unit-tested from
 * node (botui/tests/drive_preview_parse.test.js) through vm.runInNewContext,
 * so everything exported here is pure: no DOM, no network, no timers.
 */
(function (root) {
  "use strict";

  var DELIMITER_LABELS = { ",": "comma", ";": "semicolon", "\t": "tab", "|": "pipe" };
  var DELIMITER_CANDIDATES = [",", ";", "\t", "|"];
  var NUMBER_RE = /^[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?%?$/;
  var DATE_RE = /^\d{4}-\d{2}-\d{2}(?:[T ]\d{2}:\d{2}(?::\d{2})?(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?)?$/;
  var DATE_DMY_RE = /^\d{1,2}[/.]\d{1,2}[/.]\d{2,4}$/;
  var BOOL_RE = /^(?:true|false|yes|no)$/i;

  function stripBom(value) {
    var text = typeof value === "string" ? value : String(value == null ? "" : value);
    return text.charCodeAt(0) === 0xfeff ? text.slice(1) : text;
  }

  function escapeHtml(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function formatCount(value) {
    var number = Number(value);
    if (!isFinite(number)) return "0";
    return String(Math.trunc(number)).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  }

  function delimiterLabel(delimiter) {
    return DELIMITER_LABELS[delimiter] || "delimiter";
  }

  /* RFC 4180 state machine. Keeps at most `maxRows` records and `maxCols`
   * fields per record in memory, but still counts every record so the UI can
   * say "showing first 500 of 12,345". */
  function parseRecords(text, delimiter, maxRows, maxCols) {
    var rows = [];
    var row = [];
    var field = "";
    var fieldsInRow = 0;
    var totalRows = 0;
    var totalCols = 0;
    var endedAtRecordBoundary = true;
    var inQuotes = false;
    var index = 0;
    var length = text.length;

    function pushField() {
      fieldsInRow += 1;
      if (row.length < maxCols) row.push(field);
      field = "";
    }

    function pushRow() {
      totalRows += 1;
      if (fieldsInRow > totalCols) totalCols = fieldsInRow;
      if (rows.length < maxRows) rows.push(row);
      row = [];
      fieldsInRow = 0;
    }

    while (index < length) {
      var ch = text.charAt(index);
      if (inQuotes) {
        if (ch === '"') {
          if (text.charAt(index + 1) === '"') {
            field += '"';
            index += 2;
            continue;
          }
          inQuotes = false;
          index += 1;
          endedAtRecordBoundary = false;
          continue;
        }
        field += ch;
        index += 1;
        continue;
      }
      if (ch === '"' && field.length === 0) {
        inQuotes = true;
        index += 1;
        endedAtRecordBoundary = false;
        continue;
      }
      if (ch === delimiter) {
        pushField();
        index += 1;
        endedAtRecordBoundary = false;
        continue;
      }
      if (ch === "\r") {
        pushField();
        pushRow();
        index += text.charAt(index + 1) === "\n" ? 2 : 1;
        endedAtRecordBoundary = true;
        continue;
      }
      if (ch === "\n") {
        pushField();
        pushRow();
        index += 1;
        endedAtRecordBoundary = true;
        continue;
      }
      field += ch;
      index += 1;
      endedAtRecordBoundary = false;
    }

    if (field.length > 0 || fieldsInRow > 0) {
      pushField();
      pushRow();
      endedAtRecordBoundary = false;
    }

    return {
      rows: rows,
      totalRows: totalRows,
      totalCols: totalCols,
      endedAtRecordBoundary: endedAtRecordBoundary
    };
  }

  /* Picks the delimiter that yields the most consistent field count. Ties
   * keep the earlier candidate, so a comma wins over a stray semicolon. */
  function detectDelimiter(text) {
    var sample = stripBom(text).slice(0, 8192);
    var best = { delimiter: ",", score: -1 };
    DELIMITER_CANDIDATES.forEach(function (candidate) {
      var probe = parseRecords(sample, candidate, 20, 200);
      var tally = {};
      var frequency = 0;
      var mode = 0;
      probe.rows.forEach(function (record) {
        if (record.length < 2) return;
        tally[record.length] = (tally[record.length] || 0) + 1;
      });
      Object.keys(tally).forEach(function (width) {
        if (tally[width] > frequency) {
          frequency = tally[width];
          mode = Number(width);
        }
      });
      var score = frequency * (mode - 1);
      if (score > best.score) best = { delimiter: candidate, score: score };
    });
    return best.delimiter;
  }

  function parseCsv(text, options) {
    var opts = options || {};
    var maxRows = opts.maxRows > 0 ? opts.maxRows : 500;
    var maxCols = opts.maxCols > 0 ? opts.maxCols : 100;
    var maxChars = opts.maxChars > 0 ? opts.maxChars : 2000000;
    var source = stripBom(text);
    var sourceTruncated = source.length > maxChars;
    if (sourceTruncated) source = source.slice(0, maxChars);

    var delimiter = opts.delimiter || detectDelimiter(source);
    var parsed = parseRecords(source, delimiter, maxRows, maxCols);
    var rows = parsed.rows;
    var totalRows = parsed.totalRows;
    // The character cap can cut a record in half; drop that partial record so
    // the "of N rows" count never promises a row the preview cannot show.
    if (sourceTruncated && !parsed.endedAtRecordBoundary && totalRows > 0) {
      totalRows -= 1;
      if (rows.length > totalRows) rows.pop();
    }

    var headerWidth = rows.length ? rows[0].length : 0;
    var columns = Math.min(Math.max(headerWidth, parsed.totalCols), maxCols);
    var raggedRows = 0;
    var normalized = [];
    for (var r = 0; r < rows.length; r++) {
      var cells = rows[r];
      if (r > 0 && cells.length !== headerWidth) raggedRows += 1;
      var padded = cells.slice(0, columns);
      while (padded.length < columns) padded.push("");
      normalized.push(padded);
    }

    return {
      delimiter: delimiter,
      delimiterLabel: delimiterLabel(delimiter),
      rows: normalized,
      header: normalized.length ? normalized[0] : [],
      body: normalized.slice(1),
      columns: columns,
      totalCols: parsed.totalCols,
      totalRows: totalRows,
      shownRows: normalized.length,
      truncatedRows: totalRows > normalized.length,
      truncatedCols: parsed.totalCols > columns,
      raggedRows: raggedRows,
      sourceTruncated: sourceTruncated,
      maxRows: maxRows,
      maxCols: maxCols,
      maxChars: maxChars
    };
  }

  /* Human-readable truncation notes for the CSV toolbar. */
  function truncationNotes(result) {
    var notes = [];
    if (!result) return notes;
    var shownData = Math.max(result.shownRows - 1, 0);
    var totalData = Math.max(result.totalRows - 1, 0);
    if (result.truncatedRows) {
      notes.push("Showing first " + formatCount(shownData) + " of " + formatCount(totalData) + " rows.");
    }
    if (result.truncatedCols) {
      notes.push("Showing first " + formatCount(result.columns) + " of " +
        formatCount(result.totalCols) + " columns.");
    }
    if (result.raggedRows > 0) {
      notes.push(formatCount(result.raggedRows) + " row(s) had a different field count and were padded.");
    }
    if (result.sourceTruncated) {
      notes.push("Only the first " + formatBytesLabel(result.maxChars) + " were parsed.");
    }
    return notes;
  }

  function formatBytesLabel(chars) {
    if (chars >= 1048576) return (chars / 1048576).toFixed(1) + " MB";
    if (chars >= 1024) return Math.round(chars / 1024) + " KB";
    return formatCount(chars) + " characters";
  }

  function classifyCell(value) {
    var text = String(value == null ? "" : value).trim();
    if (!text) return "empty";
    if (BOOL_RE.test(text)) return "boolean";
    if (NUMBER_RE.test(text)) return "number";
    if (DATE_RE.test(text) || DATE_DMY_RE.test(text)) return "date";
    return "text";
  }

  /* Per-column type from the first data rows, so a whole column aligns the
   * same way instead of cell-by-cell guessing. */
  function columnTypes(rows, sampleLimit) {
    var limit = sampleLimit > 0 ? sampleLimit : 50;
    var width = 0;
    rows.forEach(function (row) {
      if (row.length > width) width = row.length;
    });
    var types = [];
    for (var column = 0; column < width; column++) {
      var seen = null;
      var settled = null;
      var sampled = 0;
      for (var r = 0; r < rows.length && sampled < limit; r++) {
        var kind = classifyCell(rows[r][column]);
        if (kind === "empty") continue;
        sampled += 1;
        if (seen === null) seen = kind;
        else if (seen !== kind) {
          settled = "text";
          break;
        }
      }
      types.push(settled || seen || "text");
    }
    return types;
  }

  var JSON_TOKEN_RE = /("(?:\\.|[^"\\])*")(\s*):|("(?:\\.|[^"\\])*")|\b(true|false|null)\b|(-?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?)/g;

  function highlightJson(text) {
    var source = String(text == null ? "" : text);
    var out = "";
    var last = 0;
    var match;
    JSON_TOKEN_RE.lastIndex = 0;
    while ((match = JSON_TOKEN_RE.exec(source)) !== null) {
      out += escapeHtml(source.slice(last, match.index));
      if (match[1] !== undefined) {
        out += '<span class="gb-tok-key">' + escapeHtml(match[1]) + "</span>" + escapeHtml(match[2]) + ":";
      } else if (match[3] !== undefined) {
        out += '<span class="gb-tok-str">' + escapeHtml(match[3]) + "</span>";
      } else if (match[4] !== undefined) {
        out += '<span class="gb-tok-bool">' + escapeHtml(match[4]) + "</span>";
      } else {
        out += '<span class="gb-tok-num">' + escapeHtml(match[5]) + "</span>";
      }
      last = JSON_TOKEN_RE.lastIndex;
    }
    out += escapeHtml(source.slice(last));
    return out;
  }

  var HASH_COMMENT_LANGS = ["py", "python", "sh", "bash", "zsh", "yml", "yaml", "toml", "ini", "cfg", "conf", "rb", "pl", "r", "bas"];
  var DASH_COMMENT_LANGS = ["sql", "lua", "hs"];
  var KEYWORDS = "\\b(?:if|else|elif|then|end|for|while|do|return|function|func|fn|def|const|let|var|" +
    "class|struct|enum|impl|trait|import|from|export|use|mod|pub|mut|match|case|switch|break|continue|" +
    "async|await|try|catch|except|finally|throw|raise|new|this|self|super|static|void|int|float|double|" +
    "char|bool|string|str|True|False|None|null|nil|true|false|SELECT|FROM|WHERE|INSERT|INTO|VALUES|" +
    "UPDATE|DELETE|JOIN|LEFT|RIGHT|INNER|OUTER|CREATE|ALTER|DROP|TABLE|INDEX|GROUP|ORDER|BY|HAVING|" +
    "LIMIT|OFFSET|AND|OR|NOT|NULL|PRIMARY|KEY|FOREIGN|REFERENCES)\\b";

  function codePattern(lang) {
    var commentParts = ["\\/\\/[^\\n]*", "\\/\\*[\\s\\S]*?\\*\\/"];
    if (HASH_COMMENT_LANGS.indexOf(lang) !== -1) commentParts.push("#[^\\n]*");
    if (DASH_COMMENT_LANGS.indexOf(lang) !== -1) commentParts.push("--[^\\n]*");
    if (lang === "html" || lang === "xml" || lang === "svg") commentParts.push("<!--[\\s\\S]*?-->");
    var strings = "\"(?:\\\\.|[^\"\\\\])*\"|'(?:\\\\.|[^'\\\\])*'|`(?:\\\\.|[^`\\\\])*`";
    return new RegExp("(" + commentParts.join("|") + ")|(" + strings + ")|(\\b\\d+(?:\\.\\d+)?\\b)|(" + KEYWORDS + ")", "g");
  }

  /* Small, dependency-free tokenizer for code/log previews. Returns escaped
   * HTML with gb-tok-* spans; everything outside a token is escaped as-is. */
  function highlightCode(text, lang) {
    var source = String(text == null ? "" : text);
    var pattern = codePattern(String(lang || "").toLowerCase());
    var out = "";
    var last = 0;
    var match;
    while ((match = pattern.exec(source)) !== null) {
      if (match[0].length === 0) {
        pattern.lastIndex += 1;
        continue;
      }
      out += escapeHtml(source.slice(last, match.index));
      var tokenClass = match[1] !== undefined ? "gb-tok-com"
        : match[2] !== undefined ? "gb-tok-str"
        : match[3] !== undefined ? "gb-tok-num"
        : "gb-tok-kw";
      out += '<span class="' + tokenClass + '">' + escapeHtml(match[0]) + "</span>";
      last = pattern.lastIndex;
    }
    out += escapeHtml(source.slice(last));
    return out;
  }

  function highlightLog(text) {
    return String(text == null ? "" : text)
      .split("\n")
      .map(function (line) {
        var out = escapeHtml(line);
        out = out.replace(/\b(ERROR|FATAL|CRITICAL|PANIC)\b/g, '<span class="gb-tok-err">$1</span>');
        out = out.replace(/\b(WARN|WARNING)\b/g, '<span class="gb-tok-warn">$1</span>');
        out = out.replace(/\b(INFO|NOTICE)\b/g, '<span class="gb-tok-info">$1</span>');
        out = out.replace(/\b(DEBUG|TRACE)\b/g, '<span class="gb-tok-debug">$1</span>');
        out = out.replace(/\b\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:[.,]\d+)?(?:Z|[+-]\d{2}:?\d{2})?\b/g,
          '<span class="gb-tok-ts">$&</span>');
        return out;
      })
      .join("\n");
  }

  root.DrivePreviewParse = {
    stripBom: stripBom,
    escapeHtml: escapeHtml,
    formatCount: formatCount,
    detectDelimiter: detectDelimiter,
    delimiterLabel: delimiterLabel,
    parseCsv: parseCsv,
    truncationNotes: truncationNotes,
    classifyCell: classifyCell,
    columnTypes: columnTypes,
    highlightJson: highlightJson,
    highlightCode: highlightCode,
    highlightLog: highlightLog
  };
})(typeof window !== "undefined" ? window : globalThis);
