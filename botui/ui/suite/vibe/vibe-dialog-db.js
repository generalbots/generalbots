/**
 * Vibe Schema dialog — real database browser.
 * Sidebar lists tables from /api/database/schema; main pane shows
 * columns, data grid and a SQL runner (/api/database/query).
 */
(function () {
    "use strict";

    var D = window.VibeDialogs;
    var state = { tables: [], table: null, page: 1, pageSize: 100, cols: [], rows: [] };

    function sidebar() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-sidebar";

        var list = D.el("div", "vibe-list");
        list.id = "vibeDbTableList";
        list.innerHTML = '<div class="vibe-empty">Loading schema...</div>';

        var foot = D.el("div", "vibe-sql-bar");
        var sql = D.el("textarea", "vibe-textarea");
        sql.id = "vibeDbSql";
        sql.placeholder = "SELECT * FROM table ...";
        sql.rows = 3;
        var run = D.el("button", "vibe-btn primary", "Run");
        run.addEventListener("click", runQuery);
        foot.appendChild(sql);
        foot.appendChild(run);

        box.appendChild(list);
        box.appendChild(foot);
        return box;
    }

    function main() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-main";

        var toolbar = D.el("div", "vibe-dialog-toolbar");
        var label = D.el("span", "vibe-status info", "no table selected");
        label.id = "vibeDbTableLabel";
        var refresh = D.el("button", "vibe-btn", "↻ Refresh");
        refresh.addEventListener("click", function () { loadSchema(true); });
        var spacer = D.el("span");
        spacer.style.flex = "1";
        var exportBtn = D.el("button", "vibe-btn", "Export CSV");
        exportBtn.addEventListener("click", exportCsv);
        toolbar.appendChild(label);
        toolbar.appendChild(spacer);
        toolbar.appendChild(refresh);
        toolbar.appendChild(exportBtn);

        var gridWrap = D.el("div", "vibe-grid");
        gridWrap.id = "vibeDbGrid";
        gridWrap.innerHTML = '<div class="vibe-empty">Select a table from the schema list.</div>';

        var pager = D.el("div", "vibe-browser-status");
        pager.id = "vibeDbPager";
        pager.innerHTML = '<span id="vibeDbRowCount">—</span><span style="flex:1"></span>' +
            '<button class="vibe-btn" id="vibeDbPrev">◀ Prev</button>' +
            '<span id="vibeDbPageInfo">Page 1</span>' +
            '<button class="vibe-btn" id="vibeDbNext">Next ▶</button>';

        box.appendChild(toolbar);
        box.appendChild(gridWrap);
        box.appendChild(pager);
        return box;
    }

    function renderTableList() {
        var list = document.getElementById("vibeDbTableList");
        if (!list) return;
        if (!state.tables.length) {
            list.innerHTML = '<div class="vibe-empty">No tables found.</div>';
            document.getElementById("vibeDbGrid").innerHTML =
                '<div class="vibe-empty">The schema is empty.</div>';
            return;
        }
        list.innerHTML = "";
        state.tables.forEach(function (t) {
            var row = D.el("div", "vibe-list-item");
            if (state.table && state.table.name === t.name) row.classList.add("active");
            row.innerHTML = "<span>" + D.esc(t.name) + "</span>" +
                '<span class="meta">' + t.row_count + " rows</span>";
            row.addEventListener("click", function () {
                state.table = t;
                state.page = 1;
                renderTableList();
                loadTableData();
            });
            list.appendChild(row);
        });
    }

    function renderGrid() {
        var grid = document.getElementById("vibeDbGrid");
        if (!grid) return;
        var table = document.getElementById("vibeDbTableLabel");
        if (table && state.table) {
            table.textContent = state.table.name + " — " + state.cols.length + " cols";
            table.className = "vibe-status ok";
        }
        if (!state.cols.length) {
            grid.innerHTML = '<div class="vibe-empty">No data.</div>';
            return;
        }
        var html = '<table class="vibe-table"><thead><tr><th>#</th>';
        state.cols.forEach(function (c) { html += "<th>" + D.esc(c) + "</th>"; });
        html += "</tr></thead><tbody>";
        state.rows.forEach(function (row, i) {
            html += "<tr><td>" + ((state.page - 1) * state.pageSize + i + 1) + "</td>";
            state.cols.forEach(function (c) {
                var v = row[c];
                html += "<td title='" + D.esc(String(v == null ? "" : v)).replace(/'/g, "&#39;") + "'>" +
                    D.esc(String(v == null ? "NULL" : v).substring(0, 120)) + "</td>";
            });
            html += "</tr>";
        });
        html += "</tbody></table>";
        grid.innerHTML = html;
    }

    function loadSchema(quiet) {
        if (!quiet) {
            var list = document.getElementById("vibeDbTableList");
            if (list) list.innerHTML = '<div class="vibe-empty">Loading schema...</div>';
        }
        D.api("/api/database/schema").then(function (data) {
            state.tables = (data && data.tables) || [];
            renderTableList();
            if (state.table) loadTableData();
        }).catch(function (err) {
            var list = document.getElementById("vibeDbTableList");
            if (list) list.innerHTML = '<div class="vibe-empty">Schema error: ' + D.esc(err) + "</div>";
        });
    }

    function loadTableData() {
        if (!state.table) return;
        var grid = document.getElementById("vibeDbGrid");
        if (grid) grid.innerHTML = '<div class="vibe-empty">Loading ' + D.esc(state.table.name) + "...</div>";
        D.api("/api/database/table/" + encodeURIComponent(state.table.name) + "/data?page=" +
            state.page + "&page_size=" + state.pageSize).then(function (data) {
            if (!data || !data.columns) {
                if (grid) grid.innerHTML = '<div class="vibe-empty">Table has no data.</div>';
                state.cols = []; state.rows = [];
                return;
            }
            state.cols = data.columns || [];
            state.rows = data.rows || [];
            state.total = data.total_rows || 0;
            renderGrid();
            renderPager();
        }).catch(function (err) {
            if (grid) grid.innerHTML = '<div class="vibe-empty">Load error: ' + D.esc(err) + "</div>";
        });
    }

    function renderPager() {
        var pager = document.getElementById("vibeDbPager");
        if (!pager) return;
        var pages = Math.max(1, Math.ceil(state.total / state.pageSize));
        var info = document.getElementById("vibeDbPageInfo");
        var count = document.getElementById("vibeDbRowCount");
        if (info) info.textContent = "Page " + state.page + " of " + pages;
        if (count) count.textContent = state.total + " rows";
        var prev = document.getElementById("vibeDbPrev");
        var next = document.getElementById("vibeDbNext");
        if (prev) prev.disabled = state.page <= 1;
        if (next) next.disabled = state.page >= pages;
    }

    function runQuery() {
        var sql = document.getElementById("vibeDbSql");
        if (!sql || !sql.value.trim()) return;
        var grid = document.getElementById("vibeDbGrid");
        if (grid) grid.innerHTML = '<div class="vibe-empty">Running query...</div>';
        // Backend QueryRequest field is `query` (not `sql`) — sending the
        // wrong key returned 422 and the SQL runner never worked.
        D.api("/api/database/query", {
            method: "POST",
            body: { query: sql.value.trim() },
        }).then(function (data) {
            // Backend returns {columns, rows, ...} — it has no `success`
            // field; treat a missing columns array as failure.
            if (!data || data.error || !Array.isArray(data.columns)) {
                if (grid) grid.innerHTML = '<div class="vibe-empty">Query error: ' + D.esc((data && data.error) || (data && data.message) || "invalid response") + "</div>";
                return;
            }
            state.cols = (data.columns || []).map(function (c) { return String(c); });
            state.rows = data.rows || [];
            state.total = state.rows.length;
            state.page = 1;
            renderGrid();
            renderPager();
        }).catch(function (err) {
            if (grid) grid.innerHTML = '<div class="vibe-empty">Query error: ' + D.esc(err) + "</div>";
        });
    }

    function exportCsv() {
        if (!state.cols.length) return;
        var lines = [state.cols.join(",")];
        state.rows.forEach(function (row) {
            lines.push(state.cols.map(function (c) {
                var v = String(row[c] == null ? "" : row[c]);
                return '"' + v.replace(/"/g, '""') + '"';
            }).join(","));
        });
        var blob = new Blob([lines.join("\n")], { type: "text/csv" });
        var a = document.createElement("a");
        a.href = URL.createObjectURL(blob);
        a.download = (state.table ? state.table.name : "query") + ".csv";
        a.click();
        URL.revokeObjectURL(a.href);
    }

    D.register("db", {
        build: function (body) {
            body.appendChild(sidebar());
            body.appendChild(main());
            loadSchema(false);
            var prev = document.getElementById("vibeDbPrev");
            var next = document.getElementById("vibeDbNext");
            if (prev) prev.addEventListener("click", function () {
                if (state.page > 1) { state.page--; loadTableData(); }
            });
            if (next) next.addEventListener("click", function () {
                state.page++; loadTableData();
            });
        },
        teardown: function () {
            state = { tables: [], table: null, page: 1, pageSize: 100, cols: [], rows: [] };
        },
    });
})();