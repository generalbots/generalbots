(function() {
    'use strict';

    var API = '/api/database';
    var currentTable = null;
    var currentPage = 0;
    var pageSize = 50;
    var totalCount = 0;
    var tables = [];
    var columns = [];
    var editState = null;
    var pkColumn = null;
    var rowPks = [];
    var sortColumn = null;
    var sortOrder = 'asc';

    function sanitize(str) {
        var d = document.createElement('div');
        d.textContent = str;
        return d.innerHTML;
    }

    function typeIcon(type) {
        if (!type) return '';
        var t = type.toUpperCase();
        if (t === 'UUID') return '&#128273;';
        if (t.includes('INT') || t.includes('DECIMAL') || t.includes('NUMERIC')) return '&#128290;';
        if (t.includes('BOOL')) return '&#9745;';
        if (t.includes('DATE') || t.includes('TIME')) return '&#128197;';
        if (t.includes('JSON')) return '&#128206;';
        return '&#128221;';
    }

    async function loadSchema() {
        var list = document.getElementById('db-table-list');
        try {
            var resp = await fetch(API + '/schema');
            if (!resp.ok) throw new Error('Failed to load schema');
            var data = await resp.json();
            tables = data.tables || data || [];
            if (!Array.isArray(tables)) tables = [];
            renderTableList(tables);
        } catch (e) {
            list.innerHTML = '<div class="db-error">Error loading schema: ' + sanitize(e.message) + '</div>';
        }
    }

    function renderTableList(list) {
        var el = document.getElementById('db-table-list');
        if (list.length === 0) {
            el.innerHTML = '<div class="db-empty">No tables found</div>';
            return;
        }
        el.innerHTML = list.map(function(t) {
            var name = t.table_name || t.name || t;
            var cols = t.column_count || '?';
            var rows = t.row_count || '?';
            var active = currentTable === name ? ' active' : '';
            return '<div class="db-table-item' + active + '" onclick="DBApp.selectTable(\'' + sanitize(name) + '\')">'
                + '<span class="db-table-icon">' + typeIcon('table') + '</span>'
                + '<div class="db-table-info">'
                + '<span class="db-table-name">' + sanitize(name) + '</span>'
                + '<span class="db-table-meta">' + cols + ' cols &middot; ' + rows + ' rows</span>'
                + '</div>'
                + '</div>';
        }).join('');
    }

    async function loadTableData(tableName) {
        currentTable = tableName;
        document.getElementById('db-current-table').textContent = tableName;
        var container = document.getElementById('db-grid-container');

        var url = API + '/table/' + encodeURIComponent(tableName) + '/data?page=' + (currentPage + 1) + '&page_size=' + pageSize;
        if (sortColumn) url += '&sort=' + encodeURIComponent(sortColumn) + '&sort_order=' + sortOrder;

        try {
            var resp = await fetch(url);
            if (!resp.ok) throw new Error('Failed to load data');
            var data = await resp.json();

            columns = data.columns || [];
            var rows = data.rows || [];
            totalCount = data.total !== undefined ? data.total : (data.total_rows || rows.length);
            pkColumn = data.pk_column || null;
            rowPks = rows.map(function(row, i) {
                return pkColumn && row[pkColumn] !== undefined && row[pkColumn] !== null ? String(row[pkColumn]) : String(i);
            });

            document.getElementById('db-row-count').textContent = totalCount + ' rows';
            document.getElementById('db-pagination').style.display = 'flex';

            renderGrid(columns, rows);
            renderPagination();
            renderFilters(columns);
        } catch (e) {
            container.innerHTML = '<div class="db-empty-state"><p>Error: ' + sanitize(e.message) + '</p></div>';
        }
    }

    function renderGrid(cols, rows) {
        var container = document.getElementById('db-grid-container');
        if (cols.length === 0) {
            container.innerHTML = '<div class="db-empty-state"><p>No columns found</p></div>';
            return;
        }

        var html = '<table class="db-grid"><thead><tr>';
        html += '<th class="db-th-check"><input type="checkbox" onchange="DBApp.toggleSelectAll(this)"></th>';
        cols.forEach(function(col) {
            var name = col.column_name || col.name || col;
            var icon = typeIcon(col.data_type || col.type || '');
            html += '<th class="db-th" onclick="DBApp.sortBy(\'' + sanitize(name) + '\')">' + icon + ' ' + sanitize(name) + ' <span class="db-sort-arrow"></span></th>';
        });
        html += '<th class="db-th-actions">Actions</th></tr></thead><tbody>';

        if (rows.length === 0) {
            html += '<tr><td colspan="' + (cols.length + 2) + '" class="db-empty">No data</td></tr>';
        }

        rows.forEach(function(row, ri) {
            html += '<tr>';
            html += '<td class="db-td-check"><input type="checkbox" data-row="' + ri + '"></td>';
            cols.forEach(function(col) {
                var name = col.column_name || col.name || col;
                var val = row[name];
                var display = val === null ? '<span class="db-null">NULL</span>' : sanitize(String(val));
                html += '<td class="db-td" ondblclick="DBApp.editCell(\'' + sanitize(name) + '\',' + ri + ')" title="' + display + '">' + display + '</td>';
            });
            html += '<td class="db-td-actions">';
            html += '<button class="db-btn-icon" onclick="DBApp.editRow(' + ri + ')" title="Edit">&#9998;</button>';
            html += '<button class="db-btn-icon db-btn-danger" onclick="DBApp.deleteRow(' + ri + ')" title="Delete">&#128465;</button>';
            html += '</td></tr>';
        });

        html += '</tbody></table>';
        container.innerHTML = html;
    }

    function renderFilters(cols) {
        var el = document.getElementById('db-filters');
        if (cols.length === 0) { el.style.display = 'none'; return; }
        el.style.display = 'flex';
        el.innerHTML = '<span class="db-filter-label">Filter:</span>'
            + cols.map(function(col) {
                var name = col.column_name || col.name || col;
                return '<input type="text" class="db-filter-input" placeholder="' + sanitize(name) + '" data-col="' + sanitize(name) + '">';
            }).join('');
    }

    function renderPagination() {
        var totalPages = Math.max(1, Math.ceil(totalCount / pageSize));
        document.getElementById('db-page-info').textContent = 'Page ' + (currentPage + 1) + ' of ' + totalPages;
        document.getElementById('db-prev-page').disabled = currentPage <= 0;
        document.getElementById('db-next-page').disabled = currentPage >= totalPages - 1;
    }

    window.DBApp = {
        refreshSchema: loadSchema,

        filterTables: function(q) {
            var filtered = tables.filter(function(t) {
                var name = t.table_name || t.name || t;
                return name.toLowerCase().includes(q.toLowerCase());
            });
            renderTableList(filtered);
        },

        selectTable: function(name) {
            currentPage = 0;
            loadTableData(name);
            document.querySelectorAll('.db-table-item').forEach(function(el) {
                el.classList.toggle('active', el.textContent.includes(name));
            });
        },

        prevPage: function() { if (currentPage > 0) { currentPage--; loadTableData(currentTable); } },
        nextPage: function() { currentPage++; loadTableData(currentTable); },
        setPageSize: function(s) { pageSize = parseInt(s) || 50; currentPage = 0; if (currentTable) loadTableData(currentTable); },

        sortBy: function(col) {
            if (!currentTable) return;
            if (sortColumn === col) {
                sortOrder = (sortOrder === 'asc') ? 'desc' : 'asc';
            } else {
                sortColumn = col;
                sortOrder = 'asc';
            }
            currentPage = 0;
            loadTableData(currentTable);
        },

        toggleSelectAll: function(checkbox) {
            document.querySelectorAll('.db-td-check input[type="checkbox"]').forEach(function(cb) {
                cb.checked = checkbox.checked;
            });
        },

        editCell: function(col, row) {
            editState = { col: col, row: row };
            document.getElementById('db-cell-edit-label').textContent = 'Edit: ' + col;
            document.getElementById('db-cell-edit-value').value = '';
            document.getElementById('db-cell-edit-modal').style.display = 'flex';
            document.getElementById('db-cell-edit-value').focus();
        },

        hideCellEdit: function() {
            document.getElementById('db-cell-edit-modal').style.display = 'none';
            editState = null;
        },

        saveCellEdit: async function() {
            if (!editState || !currentTable) return;
            var val = document.getElementById('db-cell-edit-value').value;

            if (editState.col === '_new_row') {
                try {
                    var data;
                    try { data = JSON.parse(val); } catch (e) { alert('Invalid JSON for new row'); return; }
                    await fetch(API + '/table/' + encodeURIComponent(currentTable) + '/row', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(data)
                    });
                    DBApp.hideCellEdit();
                    loadTableData(currentTable);
                } catch (e) { alert('Error creating row: ' + e.message); }
                return;
            }

            var pk = rowPks[editState.row];
            if (pk === undefined) { alert('No primary key available for this row'); return; }

            try {
                var resp = await fetch(API + '/table/' + encodeURIComponent(currentTable) + '/row/' + encodeURIComponent(pk), {
                    method: 'PUT',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ column: editState.col, value: val })
                });
                if (!resp.ok) {
                    var err = await resp.json().catch(function() { return {}; });
                    throw new Error(err.error || ('HTTP ' + resp.status));
                }
                DBApp.hideCellEdit();
                loadTableData(currentTable);
            } catch (e) { alert('Error saving: ' + e.message); }
        },

        editRow: function(row) {
            document.getElementById('db-cell-edit-label').textContent = 'Edit Row ' + row;
            document.getElementById('db-cell-edit-value').value = 'Use column editors for individual cells';
            document.getElementById('db-cell-edit-modal').style.display = 'flex';
        },

        deleteRow: async function(row) {
            if (!confirm('Delete this row?')) return;
            var pk = rowPks[row];
            if (pk === undefined) { alert('No primary key available for this row'); return; }
            try {
                await fetch(API + '/table/' + encodeURIComponent(currentTable) + '/row/' + encodeURIComponent(pk), { method: 'DELETE' });
                loadTableData(currentTable);
            } catch (e) { alert('Error deleting: ' + e.message); }
        },

        addNewRow: function() {
            if (!currentTable) return;
            editState = { col: '_new_row', row: -1 };
            document.getElementById('db-cell-edit-label').textContent = 'New Row';
            document.getElementById('db-cell-edit-value').value = '{}';
            document.getElementById('db-cell-edit-modal').style.display = 'flex';
        },

        showQueryBuilder: function() {
            document.getElementById('db-sql-console').style.display = 'flex';
            document.getElementById('db-sql-input').focus();
        },

        hideQueryBuilder: function() {
            document.getElementById('db-sql-console').style.display = 'none';
        },

        executeQuery: async function() {
            var sql = document.getElementById('db-sql-input').value.trim();
            if (!sql) return;
            var result = document.getElementById('db-sql-result');
            result.innerHTML = '<div class="db-loading">Executing...</div>';
            try {
                var resp = await fetch(API + '/query', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ query: sql })
                });
                var data = await resp.json();
                if (data.columns && data.rows) {
                    var html = '<table class="db-grid"><thead><tr>';
                    data.columns.forEach(function(c) { html += '<th>' + sanitize(c) + '</th>'; });
                    html += '</tr></thead><tbody>';
                    data.rows.forEach(function(row) {
                        html += '<tr>';
                        data.columns.forEach(function(c) {
                            var v = row[c];
                            html += '<td>' + (v === null ? '<span class="db-null">NULL</span>' : sanitize(String(v))) + '</td>';
                        });
                        html += '</tr>';
                    });
                    html += '</tbody></table>';
                    result.innerHTML = html;
                } else if (data.error) {
                    result.innerHTML = '<div class="db-error">' + sanitize(data.error) + '</div>';
                } else {
                    result.innerHTML = '<div class="db-success">Query executed successfully</div>';
                }
            } catch (e) {
                result.innerHTML = '<div class="db-error">' + sanitize(e.message) + '</div>';
            }
        },

        explainQuery: async function() {
            var sql = document.getElementById('db-sql-input').value.trim();
            if (!sql) return;
            document.getElementById('db-sql-input').value = 'EXPLAIN ANALYZE ' + sql;
            DBApp.executeQuery();
        },

        exportTableCSV: function() {
            if (!currentTable) return;
            window.open(API + '/table/' + encodeURIComponent(currentTable) + '/export?format=csv');
        },

        importCSV: function() {
            document.getElementById('db-import-modal').style.display = 'flex';
        },

        hideImportModal: function() {
            document.getElementById('db-import-modal').style.display = 'none';
        },

        processImport: async function() {
            var file = document.getElementById('db-import-file').files[0];
            if (!file || !currentTable) return;
            var reader = new FileReader();
            reader.onload = async function(e) {
                var lines = e.target.result.split('\n').filter(function(l) { return l.trim(); });
                if (lines.length < 2) return;
                var headers = lines[0].split(',').map(function(h) { return h.replace(/"/g, '').trim(); });
                for (var i = 1; i < lines.length; i++) {
                    var cols = lines[i].split(',').map(function(c) { return c.replace(/"/g, '').trim(); });
                    var row = {};
                    headers.forEach(function(h, idx) { row[h] = cols[idx]; });
                    await fetch(API + '/table/' + encodeURIComponent(currentTable) + '/row', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(row)
                    });
                }
                DBApp.hideImportModal();
                loadTableData(currentTable);
            };
            reader.readAsText(file);
        },

        showNewTableModal: function() {
            document.getElementById('db-new-table-modal').style.display = 'flex';
        },

        hideNewTableModal: function() {
            document.getElementById('db-new-table-modal').style.display = 'none';
        },

        addColumnRow: function() {
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

        createTable: async function() {
            var name = document.getElementById('new-table-name').value.trim();
            if (!name) return;
            var colRows = document.querySelectorAll('#new-table-columns .db-column-row');
            var cols = [];
            colRows.forEach(function(row) {
                var inputs = row.querySelectorAll('input');
                var sel = row.querySelector('select');
                var colName = inputs[0].value.trim();
                var colType = sel.value;
                var pk = inputs[1] && inputs[1].checked;
                var nn = inputs[2] && inputs[2].checked;
                if (colName) cols.push({ name: colName, data_type: colType, primary_key: pk, not_null: nn, nullable: !nn });
            });
            if (cols.length === 0) return;
            try {
                await fetch(API + '/table', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ name: name, columns: cols })
                });
                DBApp.hideNewTableModal();
                loadSchema();
            } catch (e) { alert('Error creating table: ' + e.message); }
        }
    };

    document.getElementById('db-sql-input').addEventListener('keydown', function(e) {
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
            e.preventDefault();
            DBApp.executeQuery();
        }
    });

    loadSchema();
})();
