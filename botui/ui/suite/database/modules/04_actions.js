(function () {
    'use strict';

    var DB = window.DB;

    var DBApp = window.DBApp = {};

    DBApp = Object.assign(DBApp, {
        refreshSchema: DB.loadSchema,

        filterTables: function (q) {
            var filtered = DB.tables.filter(function (t) {
                return t.name.toLowerCase().includes(q.toLowerCase());
            });
            DB.renderTableList(filtered);
        },

        selectTable: function (name) {
            DB.currentPage = 0;
            DB.sortColumn = null;
            DB.loadTableData(name);
            document.querySelectorAll('.db-table-item').forEach(function (el) {
                el.classList.toggle('active', el.getAttribute('data-table') === name);
            });
        },

        switchView: function (view) {
            DB.currentView = view;
            DB.selected.clear();
            document.querySelectorAll('.db-view-tab').forEach(function (el) {
                el.classList.toggle('active', el.getAttribute('data-view') === view);
            });
            if (view === 'fields') {
                DB.renderFieldsView();
            } else {
                DB.renderGrid();
            }
        },

        prevPage: function () { if (DB.currentPage > 0) { DB.currentPage--; DB.loadTableData(DB.currentTable); } },
        nextPage: function () { DB.currentPage++; DB.loadTableData(DB.currentTable); },
        setPageSize: function (s) { DB.pageSize = parseInt(s, 10) || 50; DB.currentPage = 0; if (DB.currentTable) DB.loadTableData(DB.currentTable); },

        sortBy: function (col) {
            if (!DB.currentTable) return;
            if (DB.sortColumn === col) {
                DB.sortOrder = (DB.sortOrder === 'asc') ? 'desc' : 'asc';
            } else {
                DB.sortColumn = col;
                DB.sortOrder = 'asc';
            }
            DB.currentPage = 0;
            DB.loadTableData(DB.currentTable);
        },

        applyFilter: function (input) {
            if (!DB.currentTable) return;
            DB.filterAndReload(input.getAttribute('data-col'), input.value.trim());
        },

        toggleSelectAll: function (checkbox) {
            DB.selected.clear();
            if (checkbox.checked) {
                for (var i = 0; i < DB.rows.length; i++) DB.selected.add(i);
            }
            var cbs = document.querySelectorAll('.db-td-check input[type="checkbox"]');
            cbs.forEach(function (cb) { cb.checked = checkbox.checked; });
            document.querySelectorAll('.db-tr').forEach(function (tr) {
                tr.classList.toggle('db-selected', checkbox.checked);
            });
            DB.updateSelectionBar();
        },

        toggleRowSelect: function (row, checked) {
            if (checked) DB.selected.add(row);
            else DB.selected.delete(row);
            var tr = document.querySelector('tr[data-row="' + row + '"]');
            if (tr) tr.classList.toggle('db-selected', checked);
            DB.updateSelectionBar();
        },

        clearSelection: function () {
            DB.selected.clear();
            var selAll = document.getElementById('db-select-all');
            if (selAll) selAll.checked = false;
            document.querySelectorAll('.db-td-check input[type="checkbox"]').forEach(function (cb) { cb.checked = false; });
            document.querySelectorAll('.db-tr').forEach(function (tr) { tr.classList.remove('db-selected'); });
            DB.updateSelectionBar();
        },

        batchDeleteSelected: function () {
            if (!DB.selected.size) return;
            if (!confirm('Delete ' + DB.selected.size + ' selected row(s)?')) return;
            DB.deleteRows(Array.from(DB.selected));
        },

        editCell: function (col, row) {
            DB.activeCell = { row: row, colIdx: 0, col: col };
            DB.beginCellEdit(row, col);
        },

        editRow: DB.beginCellEdit,

        openRowEditor: function (row) {
            var modal = document.getElementById('db-cell-edit-modal');
            var body = document.getElementById('db-row-edit-body');
            if (!modal || !body) { DB.beginCellEdit(row, DB.columns[0].name); return; }
            var rowData = DB.rows[row];
            var html = DB.columns.map(function (col) {
                var val = rowData ? rowData[col.name] : null;
                var isBool = DB.typeOf(col.name).indexOf('bool') !== -1;
                var disabled = DB.isPK(col.name) ? ' disabled' : '';
                var input;
                if (isBool) {
                    input = '<input type="checkbox" class="db-input db-bool-input"' + (val ? ' checked' : '') + disabled + '>';
                } else if (DB.isNumericType(col.name)) {
                    input = '<input type="number" class="db-input" value="' + DB.sanitize(val == null ? '' : val) + '"' + disabled + '>';
                } else {
                    input = '<input type="text" class="db-input" value="' + DB.sanitize(val == null ? '' : String(val)) + '"' + disabled + '>';
                }
                return '<div class="db-form-group"><label>' + DB.sanitize(col.name)
                    + (DB.isPK(col.name) ? ' <span class="db-pk-badge">PK</span>' : '')
                    + '</label>' + input + '</div>';
            }).join('');
            body.innerHTML = html;
            modal.style.display = 'flex';
            modal.setAttribute('data-row', row);
        },

        hideCellEdit: function () {
            document.getElementById('db-cell-edit-modal').style.display = 'none';
        },

        saveRowEdit: async function () {
            var modal = document.getElementById('db-cell-edit-modal');
            var row = parseInt(modal.getAttribute('data-row'), 10);
            if (isNaN(row) || !DB.rows[row]) return;
            var groups = modal.querySelectorAll('#db-row-edit-body .db-form-group');
            for (var i = 0; i < groups.length; i++) {
                var col = DB.columns[i];
                if (DB.isPK(col.name)) continue;
                var input = groups[i].querySelector('input');
                var value;
                if (input.type === 'checkbox') {
                    value = input.checked;
                } else if (DB.isNumericType(col.name)) {
                    var raw = input.value.trim();
                    if (raw === '') value = DB.isNullable(col.name) ? null : 0;
                    else {
                        var n = Number(raw);
                        value = isNaN(n) ? raw : n;
                    }
                } else {
                    value = input.value;
                }
                await DB.saveCellValue(col.name, row, value);
            }
            modal.style.display = 'none';
            DB.loadTableData(DB.currentTable);
        },

        deleteRow: function (row) {
            if (!confirm('Delete this row?')) return;
            DB.deleteRows([row]);
        },

        addNewRow: function () {
            if (!DB.currentTable) return;
            var body = document.getElementById('db-row-edit-body');
            var html = DB.columns.map(function (col) {
                var isBool = DB.typeOf(col.name).indexOf('bool') !== -1;
                if (DB.isPK(col.name) || col.default_value) {
                    return '<div class="db-form-group"><label>' + DB.sanitize(col.name)
                        + (DB.isPK(col.name) ? ' <span class="db-pk-badge">PK</span>' : ' <span class="db-default-hint">auto</span>')
                        + '</label><input type="text" class="db-input" disabled value="(auto)"></div>';
                }
                var input = isBool
                    ? '<input type="checkbox" class="db-input db-bool-input">'
                    : '<input type="' + (DB.isNumericType(col.name) ? 'number' : 'text') + '" class="db-input" placeholder="' + DB.sanitize(col.name) + '">';
                return '<div class="db-form-group"><label>' + DB.sanitize(col.name) + '</label>' + input + '</div>';
            }).join('');
            body.innerHTML = html;
            var modal = document.getElementById('db-cell-edit-modal');
            modal.setAttribute('data-new', '1');
            modal.removeAttribute('data-row');
            modal.style.display = 'flex';
        },

        saveNewRow: async function () {
            var modal = document.getElementById('db-cell-edit-modal');
            var body = modal.querySelector('#db-row-edit-body');
            var obj = {};
            var groups = body.querySelectorAll('.db-form-group');
            for (var i = 0; i < groups.length; i++) {
                var col = DB.columns[i];
                if (DB.isPK(col.name) || col.default_value) continue;
                var input = groups[i].querySelector('input');
                var key = col.name;
                if (input.type === 'checkbox') obj[key] = input.checked;
                else if (DB.isNumericType(col.name)) {
                    var raw = input.value.trim();
                    obj[key] = raw === '' ? null : (isNaN(Number(raw)) ? raw : Number(raw));
                } else obj[key] = input.value;
            }
            if (!Object.keys(obj).length) { alert('No editable fields'); return; }
            try {
                await DB.fetchJson(DB.API + '/table/' + encodeURIComponent(DB.currentTable) + '/row', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(obj)
                });
                modal.style.display = 'none';
                DB.loadTableData(DB.currentTable);
            } catch (e) {
                alert('Error creating row: ' + e.message);
            }
        },

        showQueryBuilder: function () {
            document.getElementById('db-sql-console').style.display = 'flex';
            document.getElementById('db-sql-input').focus();
        },

        hideQueryBuilder: function () {
            document.getElementById('db-sql-console').style.display = 'none';
        },

        executeQuery: async function () {
            var sql = document.getElementById('db-sql-input').value.trim();
            if (!sql) return;
            var result = document.getElementById('db-sql-result');
            result.innerHTML = '<div class="db-loading">Executing...</div>';
            try {
                var data = await DB.fetchJson(DB.API + '/query', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ query: sql })
                });
                if (data.columns && data.rows) {
                    var html = '<table class="db-grid"><thead><tr>';
                    data.columns.forEach(function (c) { html += '<th>' + DB.sanitize(c) + '</th>'; });
                    html += '</tr></thead><tbody>';
                    data.rows.forEach(function (row) {
                        html += '<tr>';
                        data.columns.forEach(function (c) {
                            var v = row[c];
                            html += '<td>' + (v === null ? '<span class="db-null">NULL</span>' : DB.sanitize(String(v))) + '</td>';
                        });
                        html += '</tr>';
                    });
                    html += '</tbody></table>';
                    result.innerHTML = html;
                } else if (data.error) {
                    result.innerHTML = '<div class="db-error">' + DB.sanitize(data.error) + '</div>';
                } else {
                    result.innerHTML = '<div class="db-success">Query executed successfully</div>';
                }
            } catch (e) {
                result.innerHTML = '<div class="db-error">' + DB.sanitize(e.message) + '</div>';
            }
        },

        explainQuery: function () {
            var sql = document.getElementById('db-sql-input').value.trim();
            if (!sql) return;
            document.getElementById('db-sql-input').value = 'EXPLAIN ANALYZE ' + sql;
            DBApp.executeQuery();
        },

        exportTableCSV: function () {
            if (!DB.currentTable) return;
            window.open(DB.API + '/table/' + encodeURIComponent(DB.currentTable) + '/export?format=csv');
        },

        importCSV: function () {
            document.getElementById('db-import-modal').style.display = 'flex';
        },

        hideImportModal: function () {
            document.getElementById('db-import-modal').style.display = 'none';
        },

        processImport: async function () {
            var file = document.getElementById('db-import-file').files[0];
            if (!file || !DB.currentTable) return;
            var reader = new FileReader();
            reader.onload = async function (e) {
                var lines = e.target.result.split('\n').filter(function (l) { return l.trim(); });
                if (lines.length < 2) return;
                var headers = lines[0].split(',').map(function (h) { return h.replace(/"/g, '').trim(); });
                for (var i = 1; i < lines.length; i++) {
                    var cols = lines[i].split(',').map(function (c) { return c.replace(/"/g, '').trim(); });
                    var row = {};
                    headers.forEach(function (h, idx) { row[h] = cols[idx]; });
                    await DB.fetchJson(DB.API + '/table/' + encodeURIComponent(DB.currentTable) + '/row', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(row)
                    });
                }
                DBApp.hideImportModal();
                DB.loadTableData(DB.currentTable);
            };
            reader.readAsText(file);
        },

        showNewTableModal: function () {
            document.getElementById('db-new-table-modal').style.display = 'flex';
        },

        hideNewTableModal: function () {
            document.getElementById('db-new-table-modal').style.display = 'none';
        },

        addColumnRow: function () {
            var container = document.getElementById('new-table-columns');
            var row = document.createElement('div');
            row.className = 'db-column-row';
            row.innerHTML = '<input type="text" placeholder="column_name" class="db-input">'
                + '<select class="db-select"><option>VARCHAR(255)</option><option>TEXT</option><option>INTEGER</option>'
                + '<option>BIGINT</option><option>DECIMAL</option><option>BOOLEAN</option>'
                + '<option>DATE</option><option>TIMESTAMPTZ</option><option>JSONB</option></select>'
                + '<label class="db-checkbox"><input type="checkbox"> PK</label>'
                + '<label class="db-checkbox"><input type="checkbox" checked> NOT NULL</label>'
                + '<button class="db-btn-icon" onclick="this.closest(\'.db-column-row\').remove()">&#10005;</button>';
            container.appendChild(row);
        },

        createTable: async function () {
            var name = document.getElementById('new-table-name').value.trim();
            if (!name) return;
            var colRows = document.querySelectorAll('#new-table-columns .db-column-row');
            var cols = [];
            colRows.forEach(function (row) {
                var inputs = row.querySelectorAll('input');
                var sel = row.querySelector('select');
                var colName = inputs[0].value.trim();
                var colType = sel.value;
                var pk = inputs[1] && inputs[1].checked;
                var nn = inputs[2] && inputs[2].checked;
                if (colName) cols.push({ name: colName, data_type: colType, primary_key: pk, not_null: nn, nullable: !nn });
            });
            if (!cols.length) return;
            try {
                await DB.fetchJson(DB.API + '/table', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ name: name, columns: cols })
                });
                DBApp.hideNewTableModal();
                DB.loadSchema();
            } catch (e) { alert('Error creating table: ' + e.message); }
        }
    });
})();
