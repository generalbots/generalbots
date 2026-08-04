(function () {
    'use strict';

    var DB = window.DB;

    var NUMERIC_TYPES = ['int', 'numeric', 'decimal', 'float', 'double', 'real', 'serial', 'money'];

    DB.typeIcon = function (type) {
        var t = String(type || '').toUpperCase();
        if (t.indexOf('UUID') !== -1) return '<span class="db-ico">&#128273;</span>';
        if (t.indexOf('JSON') !== -1) return '<span class="db-ico">&#128206;</span>';
        if (t.indexOf('BOOL') !== -1) return '<span class="db-ico">&#9745;</span>';
        if (t.indexOf('DATE') !== -1 || t.indexOf('TIME') !== -1) return '<span class="db-ico">&#128197;</span>';
        if (t.indexOf('INT') !== -1 || t.indexOf('NUMERIC') !== -1 || t.indexOf('DECIMAL') !== -1) return '<span class="db-ico">&#128290;</span>';
        return '<span class="db-ico">&#128196;</span>';
    };

    DB.typeBadge = function (type) {
        var t = String(type || '').toUpperCase();
        var label;
        if (t === 'UUID') label = 'uuid';
        else if (t.indexOf('TIMESTAMP') !== -1) label = 'datetime';
        else if (t.indexOf('INT') !== -1 || t.indexOf('BIGINT') !== -1 || t.indexOf('SERIAL') !== -1) label = 'number';
        else if (t.indexOf('DECIMAL') !== -1 || t.indexOf('NUMERIC') !== -1 || t.indexOf('FLOAT') !== -1) label = 'decimal';
        else if (t.indexOf('BOOL') !== -1) label = 'bool';
        else if (t.indexOf('JSON') !== -1) label = 'json';
        else if (t.indexOf('DATE') !== -1) label = 'date';
        else if (t.indexOf('CHAR') !== -1 || t.indexOf('TEXT') !== -1) label = 'text';
        else label = (t.split(' ')[0] || 'any').toLowerCase();
        return '<span class="db-type-badge db-type-' + label + '">' + label + '</span>';
    };

    DB.isNumericType = function (colName) {
        var t = DB.typeOf(colName);
        for (var i = 0; i < NUMERIC_TYPES.length; i++) {
            if (t.indexOf(NUMERIC_TYPES[i]) !== -1) return true;
        }
        return false;
    };

    DB.formatCell = function (colName, raw) {
        var t = DB.typeOf(colName);
        if (raw === null || raw === undefined) {
            return '<span class="db-null">&#8709; NULL</span>';
        }
        if (typeof raw === 'boolean') {
            return '<span class="db-bool ' + (raw ? 'db-bool-on' : 'db-bool-off') + '">'
                + (raw ? '&#10003;' : '&#10005;') + '</span>';
        }
        var s = String(raw);
        if (DB.isNumericType(colName)) {
            return '<span class="db-num">' + DB.sanitize(s) + '</span>';
        }
        if (t.indexOf('timestamp') !== -1 || t.indexOf('date') !== -1 || t.indexOf('time') !== -1) {
            var d = s.replace('T', ' ').split('.')[0];
            if (d.length > 19) d = d.slice(0, 19);
            return '<span class="db-dt">' + DB.sanitize(d) + '</span>';
        }
        if (/^https?:\/\//.test(s)) {
            return '<a class="db-link" href="' + DB.sanitize(s) + '" target="_blank" rel="noopener">' + DB.sanitize(s) + '</a>';
        }
        if (t.indexOf('json') !== -1) {
            var preview = s.length > 64 ? s.slice(0, 64) + '&hellip;' : s;
            return '<span class="db-json">' + DB.sanitize(preview) + '</span>';
        }
        if (s.length > 160) s = s.slice(0, 160) + '&hellip;';
        return '<span class="db-str">' + DB.sanitize(s) + '</span>';
    };

    DB.renderStartup = function () {
        var container = document.getElementById('db-grid-container');
        if (!container || DB.currentTable) return;
        var tables = DB.tables;
        var html = '<div class="db-startup">'
            + '<div class="db-startup-hero">'
            + '<div class="db-startup-logo">'
            + '<svg width="34" height="34" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">'
            + '<ellipse cx="12" cy="5" rx="8" ry="3"></ellipse>'
            + '<path d="M4 5v14c0 1.66 3.58 3 8 3s8-1.34 8-3V5"></path>'
            + '<path d="M4 12c0 1.66 3.58 3 8 3s8-1.34 8-3"></path>'
            + '</svg></div>'
            + '<h2 class="db-startup-title">Schema Browser</h2>'
            + '<p class="db-startup-sub">Explore and manage your database with a modern spreadsheet-style grid. Click a table to open it.</p>'
            + '<div class="db-startup-actions">'
            + '<button class="db-btn db-btn-primary" data-start="new">+ New Table</button>'
            + '<button class="db-btn" data-start="import">Import CSV</button>'
            + '<button class="db-btn" data-start="sql">SQL Console</button>'
            + '</div></div>';

        var totalCols = 0;
        var totalRows = 0;
        tables.forEach(function (t) {
            totalCols += t.column_count || 0;
            totalRows += t.row_count > 0 ? t.row_count : 0;
        });
        html += '<div class="db-startup-stats">'
            + '<div class="db-stat"><span class="db-stat-value">' + tables.length + '</span><span class="db-stat-label">tables</span></div>'
            + '<div class="db-stat"><span class="db-stat-value">' + totalCols + '</span><span class="db-stat-label">columns</span></div>'
            + '<div class="db-stat"><span class="db-stat-value">' + totalRows + '</span><span class="db-stat-label">records</span></div>'
            + '</div>';

        var recent = tables.slice(0, 8);
        if (recent.length) {
            html += '<div class="db-startup-recent">'
                + '<div class="db-startup-recent-head"><span>Quick open</span></div>'
                + '<div class="db-startup-table-grid">';
            recent.forEach(function (t) {
                var name = t.name || '';
                var cols = t.column_count || 0;
                var rows = t.row_count != null ? t.row_count : 0;
                var size = t.table_size || '';
                html += '<div class="db-startup-table-card" data-table="' + DB.sanitize(name) + '">'
                    + '<span class="db-table-icon">' + DB.typeIcon('table') + '</span>'
                    + '<div class="db-startup-table-info">'
                    + '<span class="db-startup-table-name">' + DB.sanitize(name) + '</span>'
                    + '<span class="db-startup-table-meta">' + cols + ' cols &middot; ' + rows + ' rows' + (size ? ' &middot; ' + DB.sanitize(size) : '') + '</span>'
                    + '</div></div>';
            });
            html += '</div></div>';
        }
        html += '</div>';
        container.innerHTML = html;
    };

    DB.renderTableList = function (list) {
        var el = document.getElementById('db-table-list');
        if (!list) list = DB.tables;
        if (!list.length) {
            el.innerHTML = '<div class="db-empty">No tables found</div>';
            return;
        }
        el.innerHTML = list.map(function (t) {
            var name = t.name || t.table_name || '';
            var cols = t.column_count || t.columns ? (t.column_count || t.columns.length) : '?';
            var rows = t.row_count != null ? t.row_count : '?';
            var size = t.table_size ? '<span class="db-table-size">' + DB.sanitize(t.table_size) + '</span>' : '';
            var active = DB.currentTable === name ? ' active' : '';
            return '<div class="db-table-item' + active + '" data-table="' + DB.sanitize(name) + '">'
                + '<span class="db-table-icon">' + DB.typeIcon('table') + '</span>'
                + '<div class="db-table-info">'
                + '<span class="db-table-name">' + DB.sanitize(name) + '</span>'
                + '<span class="db-table-meta">' + cols + ' cols &middot; ' + rows + ' rows</span>'
                + '</div>'
                + size
                + '</div>';
        }).join('');
    };

    DB.renderGrid = function () {
        var container = document.getElementById('db-grid-container');
        if (!DB.columns.length) {
            container.innerHTML = '<div class="db-empty-state"><p>No columns found</p></div>';
            return;
        }
        var rows = DB.rows;
        var html = '<div class="db-grid-wrap"><table class="db-grid"><thead><tr>';
        html += '<th class="db-corner">&nbsp;</th>';
        html += '<th class="db-th-check"><input type="checkbox" id="db-select-all" title="Select all rows"></th>';
        DB.columns.forEach(function (col) {
            var name = col.name;
            var w = DB.widthFor(name);
            var arrow = DB.sortColumn === name
                ? '<span class="db-sort-arrow">' + (DB.sortOrder === 'asc' ? '&#9650;' : '&#9660;') + '</span>'
                : '';
            var pk = col.is_pk ? '<span class="db-pk-badge" title="Primary key">PK</span>' : '';
            html += '<th class="db-th" data-col="' + DB.sanitize(name) + '" style="width:' + w + 'px;min-width:' + w + 'px">'
                + '<div class="db-th-inner" data-act="sort">'
                + '<span class="db-th-label">' + DB.sanitize(name) + '</span>'
                + arrow + pk
                + '</div>'
                + '<span class="db-resize-handle" data-act="resize" data-col="' + DB.sanitize(name) + '" title="Drag to resize"></span>'
                + '</th>';
        });
        html += '<th class="db-th-actions">&nbsp;</th>';
        html += '</tr></thead><tbody>';

        if (!rows.length) {
            html += '<tr><td colspan="' + (DB.columns.length + 3) + '" class="db-empty">'
                + '<div class="db-add-first">'
                + '<div class="db-empty-icon db-empty-icon-sm">'
                + '<svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M12 5v14M5 12h14"></path></svg>'
                + '</div>'
                + '<p class="db-add-first-title">No rows yet</p>'
                + '<p class="db-add-first-sub">This table has no data. Add the first row to get started.</p>'
                + '<button class="db-btn db-btn-primary" data-act="add">+ Add row</button>'
                + '</div></td></tr>';
        } else {
            rows.forEach(function (row, i) {
                var sel = DB.selected.has(i) ? ' db-selected' : '';
                html += '<tr class="db-tr' + sel + '" data-row="' + i + '">';
                html += '<td class="db-rownum">' + (i + 1) + '</td>';
                html += '<td class="db-td-check"><input type="checkbox" data-row="' + i + '"' + (DB.selected.has(i) ? ' checked' : '') + '></td>';
                DB.columns.forEach(function (col) {
                    var name = col.name;
                    var w = DB.widthFor(name);
                    var val = row[name];
                    html += '<td class="db-td" data-col="' + DB.sanitize(name) + '" data-row="' + i + '" style="width:' + w + 'px;min-width:' + w + 'px" title="' + DB.sanitize(val === null || val === undefined ? 'NULL' : String(val)) + '">'
                        + DB.formatCell(name, val) + '</td>';
                });
                html += '<td class="db-td-actions"><span class="db-row-actions">'
                    + '<button class="db-icon-btn" data-act="edit" data-row="' + i + '" title="Edit row">&#9998;</button>'
                    + '<button class="db-icon-btn db-danger" data-act="del" data-row="' + i + '" title="Delete row">&#128465;</button>'
                    + '</span></td>';
                html += '</tr>';
            });
        }
        html += '</tbody></table></div>';
        container.innerHTML = html;
        DB.updateSelectionBar();
        DB.highlightActiveCell();
    };

    DB.renderFieldsView = function () {
        var container = document.getElementById('db-grid-container');
        if (!DB.columns.length) {
            container.innerHTML = '<div class="db-empty-state"><p>No fields</p></div>';
            return;
        }
        var html = '<div class="db-fields-view"><div class="db-fields-header">'
            + '<span class="db-fields-count">' + DB.columns.length + ' fields in ' + DB.sanitize(DB.currentTable) + '</span>'
            + '</div><div class="db-fields-grid">';
        DB.columns.forEach(function (col) {
            var name = col.name;
            var type = col.data_type || '';
            var pk = col.is_pk ? '<span class="db-pk-badge">PK</span>' : '';
            var fk = col.is_fk ? '<span class="db-fk-badge" title="Foreign key">FK</span>' : '';
            html += '<div class="db-field-card">'
                + '<div class="db-field-icon">' + DB.typeIcon(type) + '</div>'
                + '<div class="db-field-info">'
                + '<span class="db-field-name">' + DB.sanitize(name) + '</span>'
                + '<span class="db-field-badges">' + DB.typeBadge(type) + pk + fk + '</span>'
                + '<span class="db-field-type">' + DB.sanitize(type) + (col.nullable ? ' &middot; nullable' : ' &middot; NOT NULL') + '</span>'
                + '</div></div>';
        });
        html += '</div></div>';
        container.innerHTML = html;
    };

    DB.renderPagination = function () {
        var totalPages = Math.max(1, Math.ceil(DB.totalCount / DB.pageSize));
        var info = document.getElementById('db-page-info');
        if (info) info.textContent = 'Page ' + (DB.currentPage + 1) + ' of ' + totalPages;
        var prev = document.getElementById('db-prev-page');
        var next = document.getElementById('db-next-page');
        if (prev) prev.disabled = DB.currentPage <= 0;
        if (next) next.disabled = DB.currentPage >= totalPages - 1;
    };

    DB.updateSelectionBar = function () {
        var bar = document.getElementById('db-selection-bar');
        if (!bar) return;
        if (DB.selected.size === 0) {
            bar.hidden = true;
            return;
        }
        bar.hidden = false;
        var count = bar.querySelector('.db-sel-count');
        if (count) count.textContent = DB.selected.size + ' selected';
    };

    DB.highlightActiveCell = function () {
        var container = document.getElementById('db-grid-container');
        var old = container.querySelector('.db-td.active-cell');
        if (old) old.classList.remove('active-cell');
        if (!DB.activeCell || DB.currentView !== 'grid') return;
        var td = container.querySelector('td[data-row="' + DB.activeCell.row + '"][data-col="' + CSS.escape(DB.activeCell.col) + '"]');
        if (td) td.classList.add('active-cell');
    };

    DB.scrollActiveIntoView = function () {
        var container = document.getElementById('db-grid-container');
        if (!DB.activeCell) return;
        var td = container.querySelector('td[data-row="' + DB.activeCell.row + '"][data-col="' + CSS.escape(DB.activeCell.col) + '"]');
        if (td) td.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    };
})();
