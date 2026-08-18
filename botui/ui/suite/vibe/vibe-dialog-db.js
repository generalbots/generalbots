/**
 * Vibe Schema dialog — real database browser.
 * Sidebar lists tables from /api/database/schema; main pane shows
 * columns, data grid and a SQL runner (/api/database/query).
 * Rows are editable inline (PUT row/:id per cell), deletable, and new rows
 * can be inserted (POST row) — so the DB grid is not read-only.
 */
(function () {
    "use strict";

    var D = window.VibeDialogs;
    var state = { tables: [], table: null, page: 1, pageSize: 100, cols: [], rows: [], pk: null };

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
        var insert = D.el("button", "vibe-btn", "➕ Insert Row");
        insert.addEventListener("click", insertRow);
        var spacer = D.el("span");
        spacer.style.flex = "1";
        var exportBtn = D.el("button", "vibe-btn", "Export CSV");
        exportBtn.addEventListener("click", exportCsv);
        toolbar.appendChild(label);
        toolbar.appendChild(spacer);
        toolbar.appendChild(insert);
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
            var grid = document.getElementById("vibeDbGrid");
            if (grid) grid.innerHTML = '<div class="vibe-empty">The schema is empty.</div>';
            return;
        }
        list.innerHTML = "";
        state.tables.forEach(function (t) {
            var row = D.el("div", "vibe-list-item");
            if (state.table && state.table.name === t.name) row.classList.add("active");
            row.innerHTML = "<span>🗄️ " + D.esc(t.name) + "</span>" +
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

    /* ------------------------------------------------- cell editing */

    function pkValue(row) {
        // The data grid returns rows keyed by aliased columns c1..cN in order
        // of `cols`; the backend also reports `pk_column`. Find the pk value
        // from the position of the pk column, or fall back to the first col.
        var idx = state.pk ? state.cols.indexOf(state.pk) : -1;
        if (idx < 0) idx = 0;
        var key = "c" + (idx + 1);
        return row[key] != null ? String(row[key]) : "";
    }

    function cellInput(row, col, current) {
        var input = document.createElement("input");
        input.type = "text";
        input.value = current == null ? "" : String(current);
        input.className = "vibe-input";
        input.style.width = "100%";
        input.style.boxSizing = "border-box";
        input.dataset.col = col;
        var pk = pkValue(row);
        input.dataset.pk = pk;
        input.dataset.orig = current == null ? "" : String(current);
        return input;
    }

    function saveCell(input) {
        var tableName = state.table ? state.table.name : null;
        if (!tableName) return;
        var col = input.dataset.col;
        var pk = input.dataset.pk;
        var value = input.value;
        input.disabled = true;
        var req = {
            column: col,
            value: value === "" ? null : value,
        };
        // The row/:id PUT expects a plain column/value body.
        var url = "/api/database/table/" + encodeURIComponent(tableName) + "/row/" + encodeURIComponent(pk);
        D.api(url, { method: "PUT", body: req }).then(function (data) {
            input.disabled = false;
            if (data && data.success) {
                input.dataset.orig = value;
                var status = document.getElementById("vibeDbTableLabel");
                if (status) {
                    status.textContent = "saved " + col + " = " + value.substring(0, 40);
                    status.className = "vibe-status ok";
                }
                setTimeout(function () {
                    if (status && state.table) {
                        status.textContent = state.table.name + " — " + state.cols.length + " cols";
                        status.className = "vibe-status ok";
                    }
                }, 1800);
            } else {
                input.disabled = false;
                input.value = input.dataset.orig;
                var status = document.getElementById("vibeDbTableLabel");
                if (status) {
                    status.textContent = "save failed: " + ((data && data.error) || "error");
                    status.className = "vibe-status err";
                }
            }
        }).catch(function (err) {
            input.disabled = false;
            input.value = input.dataset.orig;
            var status = document.getElementById("vibeDbTableLabel");
            if (status) {
                status.textContent = "save error: " + err;
                status.className = "vibe-status err";
            }
        });
    }

    function deleteRow(row) {
        var tableName = state.table ? state.table.name : null;
        if (!tableName) return;
        var pk = pkValue(row);
        if (!confirm("Delete row with " + (state.pk || "id") + " = " + pk + "?")) return;
        var url = "/api/database/table/" + encodeURIComponent(tableName) + "/row/" + encodeURIComponent(pk);
        D.api(url, { method: "DELETE" }).then(function (data) {
            var status = document.getElementById("vibeDbTableLabel");
            if (data && data.success) {
                if (status) { status.textContent = "row deleted"; status.className = "vibe-status ok"; }
                loadTableData();
            } else if (status) {
                status.textContent = "delete failed: " + ((data && data.error) || "error");
                status.className = "vibe-status err";
            }
        }).catch(function (err) {
            var status = document.getElementById("vibeDbTableLabel");
            if (status) { status.textContent = "delete error: " + err; status.className = "vibe-status err"; }
        });
    }

    function insertRow() {
        var tableName = state.table ? state.table.name : null;
        if (!tableName) return;
        var data = {};
        state.cols.forEach(function (c) {
            var v = prompt("Value for " + c + " (blank = NULL):", "");
            if (v === null) return; // cancelled
            if (v === "") {
                data[c] = null;
            } else {
                // Best-effort type inference so numbers/bools stay typed.
                if (/^-?\d+(\.\d+)?$/.test(v) && String(parseFloat(v)) === v) data[c] = parseFloat(v);
                else if (v === "true") data[c] = true;
                else if (v === "false") data[c] = false;
                else data[c] = v;
            }
        });
        var url = "/api/database/table/" + encodeURIComponent(tableName) + "/row";
        D.api(url, { method: "POST", body: { data: data } }).then(function (res) {
            var status = document.getElementById("vibeDbTableLabel");
            if (res && res.success) {
                if (status) { status.textContent = "row inserted"; status.className = "vibe-status ok"; }
                loadTableData();
            } else if (status) {
                status.textContent = "insert failed: " + ((res && res.error) || "error");
                status.className = "vibe-status err";
            }
        }).catch(function (err) {
            var status = document.getElementById("vibeDbTableLabel");
            if (status) { status.textContent = "insert error: " + err; status.className = "vibe-status err"; }
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
        state.cols.forEach(function (c) {
            html += "<th>" + D.esc(c) + (state.pk === c ? " 🔑" : "") + "</th>";
        });
        html += "<th></th></tr></thead><tbody>";
        state.rows.forEach(function (row, i) {
            html += "<tr><td>" + ((state.page - 1) * state.pageSize + i + 1) + "</td>";
            state.cols.forEach(function (c) {
                var key = "c" + (state.cols.indexOf(c) + 1);
                var v = row[key];
                var display = v == null ? "NULL" : String(v);
                html += "<td><input class='vibe-input vibe-cell' data-col='" + D.esc(c) +
                    "' data-pk='" + D.esc(pkValue(row)) +
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
                if (e.key === "Enter") { e.preventDefault(); saveCell(input); }
                if (e.key === "Escape") { input.value = input.dataset.orig; input.blur(); }
            });
            input.addEventListener("blur", function () {
                if (input.value !== input.dataset.orig) saveCell(input);
            });
        });
        grid.querySelectorAll(".vibe-del-btn").forEach(function (btn, i) {
            btn.addEventListener("click", function () { deleteRow(state.rows[i]); });
        });
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
            state.pk = data.pk_column || null;
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
                var key = "c" + (state.cols.indexOf(c) + 1);
                var v = String(row[key] == null ? "" : row[key]);
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
            state = { tables: [], table: null, page: 1, pageSize: 100, cols: [], rows: [], pk: null };
        },
    });
})();
