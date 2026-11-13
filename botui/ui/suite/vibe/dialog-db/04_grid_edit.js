/* Vibe Schema dialog — split from vibe/vibe-dialog-db.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__db;
    var D = window.VibeDialogs;
    function renderTableList() {
        var list = document.getElementById("vibeDbTableList");
        if (!list) return;
        if (!M.state.tables.length) {
            list.innerHTML = '<div class="vibe-empty">No tables found.</div>';
            var grid = document.getElementById("vibeDbGrid");
            if (grid) grid.innerHTML = '<div class="vibe-empty">The schema is empty.</div>';
            return;
        }
        list.innerHTML = "";
        M.state.tables.forEach(function (t) {
            var row = D.el("div", "vibe-list-item");
            if (M.state.table && M.state.table.name === t.name) row.classList.add("active");
            row.innerHTML = "<span>🗄️ " + D.esc(t.name) + "</span>" +
                '<span class="meta">' + t.row_count + " rows</span>";
            row.addEventListener("click", function () {
                M.state.table = t;
                M.state.page = 1;
                M.renderTableList();
                M.loadTableData();
            });
            list.appendChild(row);
        });
    }

    /* ------------------------------------------------- cell editing */

    function pkValue(row) {
        // The backend remaps rows to real column names (#906), so the PK
        // value lives under `row[pk_column]`; fall back to the first column.
        var pk = M.state.pk && M.state.cols.indexOf(M.state.pk) >= 0 ? M.state.pk : (M.state.cols[0] || null);
        if (!pk) return "";
        var v = row[pk];
        if (v == null && row["c1"] != null) v = row["c1"];
        return v != null ? String(v) : "";
    }

    function cellInput(row, col, current) {
        var input = document.createElement("input");
        input.type = "text";
        input.value = current == null ? "" : String(current);
        input.className = "vibe-input";
        input.style.width = "100%";
        input.style.boxSizing = "border-box";
        input.dataset.col = col;
        var pk = M.pkValue(row);
        input.dataset.pk = pk;
        input.dataset.orig = current == null ? "" : String(current);
        return input;
    }

    function saveCell(input) {
        var tableName = M.state.table ? M.state.table.name : null;
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
        M.apiEnv(url, { method: "PUT", body: req }).then(function (data) {
            input.disabled = false;
            if (data && data.success) {
                input.dataset.orig = value;
                var status = document.getElementById("vibeDbTableLabel");
                if (status) {
                    status.textContent = "saved " + col + " = " + value.substring(0, 40);
                    status.className = "vibe-status ok";
                }
                setTimeout(function () {
                    if (status && M.state.table) {
                        status.textContent = M.state.table.name + " — " + M.state.cols.length + " cols";
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

    function dbConfirm(message, title) {
        if (window.GBModal && typeof window.GBModal.confirm === "function") {
            return new Promise(function (resolve) {
                window.GBModal.confirm(String(message), title || "Confirm", function () { resolve(true); }, function () { resolve(false); });
            });
        }
        return Promise.resolve(false);
    }

    function dbPrompt(message, defaultValue, title) {
        if (window.GBModal && typeof window.GBModal.prompt === "function") {
            return new Promise(function (resolve) {
                window.GBModal.prompt(String(message), defaultValue == null ? "" : String(defaultValue), title || "Input", function (v) {
                    resolve(v == null ? null : String(v));
                });
            });
        }
        return Promise.resolve(null);
    }

    async function deleteRow(row) {
        var tableName = M.state.table ? M.state.table.name : null;
        if (!tableName) return;
        var pk = M.pkValue(row);
        var ok = await dbConfirm("Delete row with " + (M.state.pk || "id") + " = " + pk + "?", "Delete row");
        if (!ok) return;
        var url = "/api/database/table/" + encodeURIComponent(tableName) + "/row/" + encodeURIComponent(pk);
        M.apiEnv(url, { method: "DELETE" }).then(function (data) {
            var status = document.getElementById("vibeDbTableLabel");
            if (data && data.success) {
                if (status) { status.textContent = "row deleted"; status.className = "vibe-status ok"; }
                M.loadTableData();
            } else if (status) {
                status.textContent = "delete failed: " + ((data && data.error) || "error");
                status.className = "vibe-status err";
            }
        }).catch(function (err) {
            var status = document.getElementById("vibeDbTableLabel");
            if (status) { status.textContent = "delete error: " + err; status.className = "vibe-status err"; }
        });
    }

    async function insertRow() {
        var tableName = M.state.table ? M.state.table.name : null;
        if (!tableName) return;
        var data = {};
        for (var i = 0; i < M.state.cols.length; i++) {
            var c = M.state.cols[i];
            var v = await dbPrompt("Value for " + c + " (blank = NULL):", "", "Insert row");
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
        }
        var url = "/api/database/table/" + encodeURIComponent(tableName) + "/row";
        M.apiEnv(url, { method: "POST", body: { data: data } }).then(function (res) {
            var status = document.getElementById("vibeDbTableLabel");
            if (res && res.success) {
                if (status) { status.textContent = "row inserted"; status.className = "vibe-status ok"; }
                M.loadTableData();
            } else if (status) {
                status.textContent = "insert failed: " + ((res && res.error) || "error");
                status.className = "vibe-status err";
            }
        }).catch(function (err) {
            var status = document.getElementById("vibeDbTableLabel");
            if (status) { status.textContent = "insert error: " + err; status.className = "vibe-status err"; }
        });
    }

    M.renderTableList = renderTableList;
    M.pkValue = pkValue;
    M.cellInput = cellInput;
    M.saveCell = saveCell;
    M.deleteRow = deleteRow;
    M.insertRow = insertRow;
})(window.VibeDialogs);
