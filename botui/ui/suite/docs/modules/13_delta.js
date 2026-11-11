"use strict";
/* #2172 — apply gb-doc-delta/v1 ops live while the VM generates. */
(function () {
  function ensureEditor() {
    return document.getElementById("doc-content") || document.querySelector(".docs-editor");
  }
  function esc(s) {
    return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }
  function renderRuns(runs) {
    if (!Array.isArray(runs)) return esc(runs || "");
    return runs.map(function (r) {
      var t = esc(typeof r === "string" ? r : (r.text || ""));
      if (r && r.bold) t = "<strong>" + t + "</strong>";
      if (r && r.italic) t = "<em>" + t + "</em>";
      if (r && r.underline) t = "<u>" + t + "</u>";
      return t;
    }).join("");
  }
  function applyDelta(payload) {
    var root = ensureEditor();
    if (!root) return;
    var ops = payload.ops || [];
    ops.forEach(function (op) {
      if (!op) return;
      if (op.op === "clear") { root.innerHTML = ""; return; }
      if (op.op === "set_title" && op.title) {
        var t = document.getElementById("docTitle");
        if (t) t.value = op.title;
        return;
      }
      if (op.op === "append_block" || op.op === "update_block") {
        var type = op.type || "paragraph";
        var id = op.id || ("gbd-" + Math.random().toString(36).slice(2));
        var existing = root.querySelector('[data-delta-id="' + id + '"]');
        var el = existing || document.createElement(type === "heading" ? ("h" + (op.level || 2)) : "p");
        el.setAttribute("data-delta-id", id);
        if (type === "table" && Array.isArray(op.rows)) {
          var html = "<table><tbody>";
          op.rows.forEach(function (row) { html += "<tr>" + (row || []).map(function (c) { return "<td>" + esc(c) + "</td>"; }).join("") + "</tr>"; });
          html += "</tbody></table>";
          el.innerHTML = html;
        } else {
          el.innerHTML = renderRuns(op.runs != null ? op.runs : op.text);
        }
        if (!existing) root.appendChild(el);
        return;
      }
      if (op.op === "delete_block" && op.id) {
        var del = root.querySelector('[data-delta-id="' + op.id + '"]');
        if (del) del.remove();
        return;
      }
      if (op.op === "finalize") {
        var s = document.getElementById("saveStatus");
        if (s) s.textContent = "Saved to Drive";
      }
    });
  }
  if (window.GBDocDelta) window.GBDocDelta.register("docs", applyDelta);
  else document.addEventListener("DOMContentLoaded", function () { if (window.GBDocDelta) window.GBDocDelta.register("docs", applyDelta); });
})();
