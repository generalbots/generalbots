/* Vibe Schema dialog — split from vibe/vibe-dialog-db.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__db;
    var D = window.VibeDialogs;
    function renderGrid() {
        var grid = document.getElementById("vibeDbGrid");
        if (!grid) return;
        var table = document.getElementById("vibeDbTableLabel");
        if (table && M.state.table) {
            table.textContent = M.state.table.name + " — " + M.state.cols.length + " cols";
            table.className = "vibe-status ok";
        }
        if (!M.state.cols.length) {
            grid.innerHTML = '<div class="vibe-empty">No data.</div>';
            return;
        }
        var html = '<table class="vibe-table"><thead><tr><th>#</th>';
        M.state.cols.forEach(function (c) {
            html += "<th>" + D.esc(c) + (M.state.pk === c ? " 🔑" : "") + "</th>";
        });
        html += "<th></th></tr></thead><tbody>";
        M.state.rows.forEach(function (row, i) {
            html += "<tr><td>" + ((M.state.page - 1) * M.state.pageSize + i + 1) + "</td>";
            M.state.cols.forEach(function (c) {
                // Rows arrive keyed by real column names (#906) — reading the
                // old aliased `c1..cN` keys rendered every cell empty.
                var v = row[c];
                if (v === undefined && row["c" + (M.state.cols.indexOf(c) + 1)] !== undefined) {
                    v = row["c" + (M.state.cols.indexOf(c) + 1)];
                }
                var display = v == null ? "NULL" : String(v);
                html += "<td><input class='vibe-input vibe-cell' data-col='" + D.esc(c) +
                    "' data-pk='" + D.esc(M.pkValue(row)) +
                    "' data-orig='" + D.esc(v == null ? "" : String(v)).replace(/'/g, "&#39;") +
                    "' value='" + D.esc(display).replace(/'/g, "&#39;") + "' style='width:100%;box-sizing:border-box;padding:3px 6px;font-size:11px;background:var(--bg,#0e0e1a);color:" +
                    (v == null ? "#666" : "var(--text,#ddd)") + ";border:1px solid transparent;border-radius:4px;' " +
                    "title='" + D.esc(display).replace(/'/g, "&#39;") + "'></td>";
            });
            html += "<td><button class='vibe-btn vibe-del-btn' title='Delete row' style='padding:2px 7px;'>🗑</button></td></tr>";
        });
        html += "</tbody></table>";
        grid.innerHTML = html;

        // Wire cell save (Enter / blur) and row delete.
        grid.querySelectorAll(".vibe-cell").forEach(function (input) {
            input.addEventListener("keydown", function (e) {
                if (e.key === "Enter") { e.preventDefault(); M.saveCell(input); }
                if (e.key === "Escape") { input.value = input.dataset.orig; input.blur(); }
            });
            input.addEventListener("blur", function () {
                if (input.value !== input.dataset.orig) M.saveCell(input);
            });
        });
        grid.querySelectorAll(".vibe-del-btn").forEach(function (btn, i) {
            btn.addEventListener("click", function () { M.deleteRow(M.state.rows[i]); });
        });
    }

    function loadSchema(quiet) {
        // #1386b — no selection, no misleading data: without a project the
        // pane refuses to guess a database (it used to fall back to the
        // global default bot's, which is exactly the confusion this fixes).
        if (!M.projectId()) {
            M.state.tables = [];
            M.state.table = null;
            var list0 = document.getElementById("vibeDbTableList");
            if (list0) list0.innerHTML = '<div class="vibe-empty" role="status" aria-live="polite">Select a Vibe project to inspect its database.</div>';
            var grid0 = document.getElementById("vibeDbGrid");
            if (grid0) grid0.innerHTML = '<div class="vibe-empty" role="status" aria-live="polite">No project selected — open a project in Vibe first.</div>';
            return;
        }
        if (!quiet) {
            var list = document.getElementById("vibeDbTableList");
            if (list) list.innerHTML = '<div class="vibe-empty" role="status" aria-live="polite">Loading schema...</div>';
        }
        M.apiEnv("/api/database/schema").then(function (data) {
            M.state.tables = (data && data.tables) || [];
            M.renderTableList();
            if (M.state.table) M.loadTableData();
        }).catch(function (err) {
            var list = document.getElementById("vibeDbTableList");
            if (list) list.innerHTML = '<div class="vibe-empty" role="alert" aria-live="assertive">Schema error: ' + D.esc(err) + "</div>";
        });
    }

    function loadTableData() {
        if (!M.state.table) return;
        var grid = document.getElementById("vibeDbGrid");
        if (grid) grid.innerHTML = '<div class="vibe-empty" role="status" aria-live="polite">Loading ' + D.esc(M.state.table.name) + "...</div>";
        M.apiEnv("/api/database/table/" + encodeURIComponent(M.state.table.name) + "/data?page=" +
            M.state.page + "&page_size=" + M.state.pageSize).then(function (data) {
            if (!data || !data.columns) {
                if (grid) grid.innerHTML = '<div class="vibe-empty" role="status" aria-live="polite">Table has no data.</div>';
                M.state.cols = []; M.state.rows = [];
                return;
            }
            M.state.cols = data.columns || [];
            M.state.rows = data.rows || [];
            M.state.total = data.total_rows || 0;
            M.state.pk = data.pk_column || null;
            M.renderGrid();
            M.renderPager();
        }).catch(function (err) {
            if (grid) grid.innerHTML = '<div class="vibe-empty" role="alert" aria-live="assertive">Load error: ' + D.esc(err) + "</div>";
        });
    }

    function renderPager() {
        var pager = document.getElementById("vibeDbPager");
        if (!pager) return;
        var pages = Math.max(1, Math.ceil(M.state.total / M.state.pageSize));
        var info = document.getElementById("vibeDbPageInfo");
        var count = document.getElementById("vibeDbRowCount");
        if (info) info.textContent = "Page " + M.state.page + " of " + pages;
        if (count) count.textContent = M.state.total + " rows";
        var prev = document.getElementById("vibeDbPrev");
        var next = document.getElementById("vibeDbNext");
        if (prev) prev.disabled = M.state.page <= 1;
        if (next) next.disabled = M.state.page >= pages;
    }

    function runQuery() {
        var sql = document.getElementById("vibeDbSql");
        if (!sql || !sql.value.trim()) return;
        var grid = document.getElementById("vibeDbGrid");
        if (grid) grid.innerHTML = '<div class="vibe-empty" role="status" aria-live="polite">Running query...</div>';
        // Backend QueryRequest field is `query` (not `sql`) — sending the
        // wrong key returned 422 and the SQL runner never worked.
        M.apiEnv("/api/database/query", {
            method: "POST",
            body: { query: sql.value.trim() },
        }).then(function (data) {
            if (!data || data.error || !Array.isArray(data.columns)) {
                if (grid) grid.innerHTML = '<div class="vibe-empty" role="alert" aria-live="assertive">Query error: ' + D.esc((data && data.error) || (data && data.message) || "invalid response") + "</div>";
                return;
            }
            M.state.cols = (data.columns || []).map(function (c) { return String(c); });
            M.state.rows = data.rows || [];
            M.state.total = M.state.rows.length;
            M.state.page = 1;
            M.renderGrid();
            M.renderPager();
        }).catch(function (err) {
            if (grid) grid.innerHTML = '<div class="vibe-empty" role="alert" aria-live="assertive">Query error: ' + D.esc(err) + "</div>";
        });
    }

    function exportCsv() {
        if (!M.state.cols.length) return;
        var lines = [M.state.cols.join(",")];
        M.state.rows.forEach(function (row) {
            lines.push(M.state.cols.map(function (c) {
                var key = "c" + (M.state.cols.indexOf(c) + 1);
                var v = String(row[key] == null ? "" : row[key]);
                return '"' + v.replace(/"/g, '""') + '"';
            }).join(","));
        });
        var blob = new Blob([lines.join("\n")], { type: "text/csv" });
        var a = document.createElement("a");
        a.href = URL.createObjectURL(blob);
        a.download = (M.state.table ? M.state.table.name : "query") + ".csv";
        a.click();
        URL.revokeObjectURL(a.href);
    }

    M.renderGrid = renderGrid;
    M.loadSchema = loadSchema;
    M.loadTableData = loadTableData;
    M.renderPager = renderPager;
    M.runQuery = runQuery;
    M.exportCsv = exportCsv;
})(window.VibeDialogs);
