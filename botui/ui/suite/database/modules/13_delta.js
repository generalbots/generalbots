"use strict";
/* #2175 — progressive row streaming for database app. */
(function () {
  function apply(payload) {
    var tbody = document.querySelector("#db-results tbody, .db-grid tbody, table.results tbody");
    (payload.ops || []).forEach(function (op) {
      if (op.op === "clear" && tbody) { tbody.innerHTML = ""; return; }
      if (op.op === "set_rows" && tbody && Array.isArray(op.rows)) {
        op.rows.forEach(function (row) {
          var tr = document.createElement("tr");
          (row || []).forEach(function (cell) { var td=document.createElement("td"); td.textContent = cell==null?"":String(cell); tr.appendChild(td); });
          tbody.appendChild(tr);
        });
      }
      if (op.op === "set_query_stats") {
        var el = document.querySelector("#db-stats, .db-stats");
        if (el) el.textContent = (op.rows||0) + " rows";
      }
    });
  }
  if (window.GBDocDelta) window.GBDocDelta.register("database", apply);
  else document.addEventListener("DOMContentLoaded", function(){ if(window.GBDocDelta) window.GBDocDelta.register("database", apply); });
})();
