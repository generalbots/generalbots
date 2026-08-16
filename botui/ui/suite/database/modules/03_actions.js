(function () {
    'use strict';

    var DB = window.DB;

    DB.fetchJson = async function (url, opts) {
        var resp = await fetch(url, opts);
        if (!resp.ok) {
            var body = await resp.json().catch(function () { return {}; });
            throw new Error(body.error || ('HTTP ' + resp.status));
        }
        return resp.json();
    }

    DB.loadSchema = async function () {
        var list = document.getElementById('db-table-list');
        try {
            var data = await DB.fetchJson(DB.API + '/schema');
            DB.tables = (data.tables || []).map(function (t) {
                return {
                    name: t.name || t.table_name || '',
                    columns: (t.columns || []).map(function (c) {
                        return {
                            name: c.name || c.column_name || '',
                            data_type: c.data_type || '',
                            nullable: c.nullable !== false,
                            is_pk: !!c.is_pk,
                            is_fk: !!c.is_fk,
                            default_value: c.default_value || null
                        };
                    }),
                    column_count: t.column_count || (t.columns ? t.columns.length : 0),
                    row_count: t.row_count != null ? t.row_count : null,
                    table_size: t.table_size || ''
                };
            });
            DB.renderTableList();
            DB.renderStartup();
        } catch (e) {
            list.innerHTML = '<div class="db-error">Error loading schema: ' + DB.sanitize(e.message) + '</div>';
        }
    };

    function enrichColumns() {
        DB.columns = [];
        DB.typeMap = {};
        for (var i = 0; i < DB.tables.length; i++) {
            if (DB.tables[i].name === DB.currentTable) {
                DB.columns = DB.tables[i].columns || [];
                break;
            }
        }
        DB.columns.forEach(function (c) {
            DB.typeMap[c.name] = c.data_type || '';
        });
    }

    DB.loadTableData = async function (tableName) {
        DB.currentTable = tableName;
        DB.currentView = 'grid';
        DB.selected.clear();
        DB.activeCell = null;
        DB.widthKey = 'db-colwidths:' + tableName;
        DB.applyWidths();

        var tbl = document.getElementById('db-current-table');
        if (tbl) tbl.textContent = tableName;
        var tabs = document.getElementById('db-view-tabs');
        if (tabs) tabs.style.display = 'flex';
        var pag = document.getElementById('db-pagination');
        if (pag) pag.style.display = 'flex';

        var url = DB.API + '/table/' + encodeURIComponent(tableName) + '/data?page=' + (DB.currentPage + 1) + '&page_size=' + DB.pageSize;
        if (DB.sortColumn) url += '&sort=' + encodeURIComponent(DB.sortColumn) + '&sort_order=' + DB.sortOrder;

        var container = document.getElementById('db-grid-container');
        try {
            var data = await DB.fetchJson(url);
            DB.columns = [];
            DB.rows = data.rows || [];
            DB.totalCount = data.total != null ? data.total : (data.total_rows != null ? data.total_rows : DB.rows.length);
            DB.pkColumn = data.pk_column || null;
            enrichColumns();
            DB.rowPks = DB.rows.map(function (row) {
                return DB.pkColumn && row[DB.pkColumn] != null ? String(row[DB.pkColumn]) : null;
            });

            var count = document.getElementById('db-row-count');
            if (count) count.textContent = DB.totalCount + ' rows';
            DB.renderGrid();
            DB.renderPagination();
            DB.renderFilters();
        } catch (e) {
            container.innerHTML = '<div class="db-empty-state"><p>Error: ' + DB.sanitize(e.message) + '</p></div>';
        }
    };

    DB.renderFilters = function () {
        var el = document.getElementById('db-filters');
        if (!el) return;
        if (!DB.columns.length) { el.style.display = 'none'; return; }
        el.style.display = 'flex';
        el.innerHTML = '<span class="db-filter-label">Filter:</span>'
            + DB.columns.map(function (col) {
                var name = col.name;
                return '<input type="text" class="db-filter-input" placeholder="' + DB.sanitize(name) + '" data-col="' + DB.sanitize(name) + '" oninput="DBApp.applyFilter(this)">';
            }).join('');
    };

    DB.filterAndReload = async function (col, val) {
        var url = DB.API + '/table/' + encodeURIComponent(DB.currentTable) + '/data?page=1&page_size=' + DB.pageSize
            + '&filter_col=' + encodeURIComponent(col) + '&filter_op=ilike&filter_val=' + encodeURIComponent(val);
        try {
            var data = await DB.fetchJson(url);
            DB.rows = data.rows || [];
            DB.totalCount = data.total != null ? data.total : DB.rows.length;
            DB.currentPage = 0;
            DB.rowPks = DB.rows.map(function (row) {
                return DB.pkColumn && row[DB.pkColumn] != null ? String(row[DB.pkColumn]) : null;
            });
            var count = document.getElementById('db-row-count');
            if (count) count.textContent = DB.totalCount + ' rows';
            DB.renderGrid();
            DB.renderPagination();
        } catch (e) {
            console.error('Filter failed:', e);
        }
    };

    DB.deleteRows = async function (indices) {
        var ids = [];
        indices.forEach(function (i) {
            if (DB.rowPks[i]) ids.push(DB.rowPks[i]);
        });
        if (!ids.length) return;
        try {
            await DB.fetchJson(DB.API + '/table/' + encodeURIComponent(DB.currentTable) + '/rows/batch-delete', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ids: ids })
            });
            DB.selected.clear();
            DB.loadTableData(DB.currentTable);
        } catch (e) {
            alert('Error deleting rows: ' + e.message);
        }
    };

    DB.saveCellValue = async function (col, rowIdx, rawValue) {
        var pk = DB.rowPks[rowIdx];
        if (!pk) {
            alert('No primary key available for this row');
            return;
        }
        var value = rawValue;
        if (DB.isNumericType(col)) {
            if (value === '' || value == null) {
                value = DB.isNullable(col) ? null : 0;
            } else {
                var n = Number(value);
                value = isNaN(n) ? value : n;
            }
        } else if (typeof value === 'string' && (value === 'true' || value === 'false') && DB.typeOf(col).indexOf('bool') !== -1) {
            value = value === 'true';
        }
        try {
            await DB.fetchJson(DB.API + '/table/' + encodeURIComponent(DB.currentTable) + '/row/' + encodeURIComponent(pk), {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ column: col, value: value })
            });
            DB.loadTableData(DB.currentTable);
        } catch (e) {
            alert('Error saving: ' + e.message);
            DB.loadTableData(DB.currentTable);
        }
    };

    DB.beginCellEdit = function (rowIdx, colName) {
        if (DB.isPK(colName)) return;
        var container = document.getElementById('db-grid-container');
        var td = container.querySelector('td[data-row="' + rowIdx + '"][data-col="' + CSS.escape(colName) + '"]');
        if (!td) return;
        var raw = DB.rows[rowIdx] ? DB.rows[rowIdx][colName] : null;
        DB.editing = true;
        td.classList.add('db-editing');

        var isBool = DB.typeOf(colName).indexOf('bool') !== -1;
        var input;
        if (isBool) {
            input = document.createElement('input');
            input.type = 'checkbox';
            input.checked = !!raw;
            input.className = 'db-inline-check';
        } else {
            input = document.createElement('input');
            input.type = DB.isNumericType(colName) ? 'number' : 'text';
            input.className = 'db-inline-edit';
            input.value = raw == null ? '' : String(raw);
        }
        td.innerHTML = '';
        td.appendChild(input);
        input.focus();
        if (input.select) input.select();

        var finished = false;
        var commit = function (save) {
            if (finished) return;
            finished = true;
            DB.editing = false;
            var newVal = save ? (isBool ? input.checked : input.value) : raw;
            if (save && newVal !== raw) {
                DB.saveCellValue(colName, rowIdx, isBool ? newVal : (newVal == null ? null : String(newVal)));
            } else if (!save) {
                DB.loadTableData(DB.currentTable);
            } else {
                td.innerHTML = DB.formatCell(colName, newVal);
                td.classList.remove('db-editing');
            }
            DB.activeCell = { row: rowIdx, col: colName };
        };
        input.addEventListener('keydown', function (e) {
            e.stopPropagation();
            if (e.key === 'Enter') { e.preventDefault(); commit(true); }
            else if (e.key === 'Escape') { e.preventDefault(); commit(false); }
        });
        input.addEventListener('blur', function () { commit(true); });
    };

    DB.navigate = function (dr, dc) {
        if (!DB.activeCell || DB.currentView !== 'grid') return;
        var r = Math.max(0, Math.min(DB.rows.length - 1, DB.activeCell.row + dr));
        var c = Math.max(0, Math.min(DB.columns.length - 1, DB.activeCell.colIdx + dc));
        DB.activeCell = { row: r, colIdx: c, col: DB.columns[c].name };
        DB.highlightActiveCell();
        DB.scrollActiveIntoView();
    };

    DB.bindGrid = function () {
        var container = document.getElementById('db-grid-container');
        container.setAttribute('tabindex', '0');

        container.addEventListener('click', function (e) {
            var target = e.target;
            var act = target.getAttribute && target.getAttribute('data-act');
            if (!act) {
                var btn = target.closest('[data-act]');
                if (btn) { act = btn.getAttribute('data-act'); target = btn; }
            }
            if (act === 'sort') {
                var th = target.closest('.db-th');
                DBApp.sortBy(th.getAttribute('data-col'));
            } else if (act === 'edit') {
                DBApp.openRowEditor(parseInt(target.getAttribute('data-row'), 10));
            } else if (act === 'del') {
                DBApp.deleteRow(parseInt(target.getAttribute('data-row'), 10));
            } else if (act === 'add') {
                DBApp.addNewRow();
            } else {
                var startBtn = target.closest('[data-start]');
                if (startBtn) {
                    var s = startBtn.getAttribute('data-start');
                    if (s === 'new') DBApp.showNewTableModal();
                    else if (s === 'import') DBApp.importCSV();
                    else if (s === 'sql') DBApp.showQueryBuilder();
                    return;
                }
                var card = target.closest('.db-startup-table-card');
                if (card) {
                    DBApp.selectTable(card.getAttribute('data-table'));
                    return;
                }
                var cell = target.closest('td[data-col]');
                if (cell && !cell.classList.contains('db-td-actions')) {
                    var ci = Array.prototype.indexOf.call(cell.parentElement.children, cell);
                    DB.activeCell = {
                        row: parseInt(cell.getAttribute('data-row'), 10),
                        colIdx: ci,
                        col: cell.getAttribute('data-col')
                    };
                    DB.highlightActiveCell();
                }
            }
        });

        container.addEventListener('change', function (e) {
            if (e.target && e.target.type === 'checkbox') {
                var selAll = document.getElementById('db-select-all');
                if (e.target === selAll) {
                    DBApp.toggleSelectAll(selAll);
                } else if (e.target.hasAttribute('data-row')) {
                    DBApp.toggleRowSelect(parseInt(e.target.getAttribute('data-row'), 10), e.target.checked);
                }
            }
        });

        container.addEventListener('dblclick', function (e) {
            var td = e.target.closest('td[data-col]');
            if (!td || DB.isPK(td.getAttribute('data-col'))) return;
            if (td.classList.contains('db-td-actions')) return;
            var ci = Array.prototype.indexOf.call(td.parentElement.children, td);
            var colName = td.getAttribute('data-col');
            DB.activeCell = { row: parseInt(td.getAttribute('data-row'), 10), colIdx: ci, col: colName };
            DB.highlightActiveCell();
            DB.beginCellEdit(DB.activeCell.row, colName);
        });

        container.addEventListener('keydown', function (e) {
            if (DB.editing) return;
            var k = e.key;
            if (k === 'ArrowDown') { e.preventDefault(); DB.navigate(1, 0); }
            else if (k === 'ArrowUp') { e.preventDefault(); DB.navigate(-1, 0); }
            else if (k === 'ArrowRight') { e.preventDefault(); DB.navigate(0, 1); }
            else if (k === 'ArrowLeft') { e.preventDefault(); DB.navigate(0, -1); }
            else if (k === 'Enter' && DB.activeCell) {
                e.preventDefault();
                DB.beginCellEdit(DB.activeCell.row, DB.activeCell.col);
            } else if (k === 'Escape' && DB.activeCell) {
                e.preventDefault();
                DB.activeCell = null;
                DB.highlightActiveCell();
            }
        });

        container.addEventListener('mousedown', function (e) {
            var handle = e.target.closest('[data-act="resize"]');
            if (!handle) return;
            e.preventDefault();
            var col = handle.getAttribute('data-col');
            var startX = e.clientX;
            var startW = DB.widthFor(col);
            var onMove = function (ev) {
                var w = Math.max(60, Math.min(800, startW + (ev.clientX - startX)));
                DB.columnWidths[col] = w;
                var ths = document.querySelectorAll('th[data-col="' + CSS.escape(col) + '"]');
                var tds = document.querySelectorAll('td[data-col="' + CSS.escape(col) + '"]');
                ths.forEach(function (el) { el.style.width = w + 'px'; el.style.minWidth = w + 'px'; });
                tds.forEach(function (el) { el.style.width = w + 'px'; el.style.minWidth = w + 'px'; });
            };
            var onUp = function () {
                document.removeEventListener('mousemove', onMove);
                document.removeEventListener('mouseup', onUp);
                DB.persistWidths();
            };
            document.addEventListener('mousemove', onMove);
            document.addEventListener('mouseup', onUp);
        });
    };
})();
