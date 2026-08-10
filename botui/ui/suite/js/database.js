const DB = {
    schema: [],
    currentTable: null,
    currentPage: 1,
    pageSize: 100,
    totalRows: 0,
    activeTab: 'data',

    init: function () {
        this.refreshSchema();
        this._bindKeyboard();
    },

    _bindKeyboard: function () {
        document.addEventListener('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
                if (this.activeTab === 'sql') this.executeSql();
            }
        });
    },

    _apiHeaders: function () {
        const headers = { 'Content-Type': 'application/json' };
        const botId = this._getBotId();
        if (botId) headers['X-Bot-Id'] = botId;
        return headers;
    },

    _getBotId: function () {
        const meta = document.querySelector('meta[name="bot-id"]');
        if (meta) return meta.getAttribute('content');
        const url = new URL(window.location.href);
        const parts = url.pathname.split('/').filter(Boolean);
        if (parts.length >= 2 && parts[0] === 'suite') return null;
        if (parts.length >= 1) return parts[0];
        return null;
    },

    toggleSidebar: function () {
        const sidebar = document.getElementById('dbSidebar');
        sidebar.classList.toggle('collapsed');
    },

    switchTab: function (tab) {
        if (!document.getElementById('dbTabData') || !document.getElementById('dbTabStructure') || !document.getElementById('dbTabSql')) return;
        this.activeTab = tab;
        document.querySelectorAll('.db-tab').forEach(t => t.classList.toggle('active', t.dataset.tab === tab));
        document.getElementById('dbTabData').style.display = tab === 'data' ? '' : 'none';
        document.getElementById('dbTabStructure').style.display = tab === 'structure' ? '' : 'none';
        document.getElementById('dbTabSql').style.display = tab === 'sql' ? '' : 'none';
    },

    refreshSchema: async function () {
        const list = document.getElementById('dbTableList');
        if (!list) return;
        list.innerHTML = '<div class="db-loading">Loading…</div>';

        try {
            const resp = await fetch('/api/database/schema', { headers: this._apiHeaders() });
            if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
            const data = await resp.json();
            this.schema = data.tables || [];
        } catch (e) {
            console.error('Schema load failed:', e);
            this.schema = [];
        }

        this._renderTableList();
    },

    _renderTableList: function () {
        const list = document.getElementById('dbTableList');
        if (!this.schema.length) {
            list.innerHTML = '<div class="db-empty">No tables found</div>';
            return;
        }

        let html = '';
        this.schema.forEach(table => {
            const active = this.currentTable === table.name ? ' active' : '';
            html += `<div class="db-table-item${active}" onclick="DB.selectTable('${this._esc(table.name)}')">
                <span class="db-table-icon">📋</span>
                <div class="db-table-info">
                    <span class="db-table-name">${this._esc(table.name)}</span>
                    <span class="db-table-meta">${table.row_count} rows · ${this._esc(table.table_size)}</span>
                </div>
            </div>`;
        });

        list.innerHTML = html;
    },

    selectTable: async function (name) {
        this.currentTable = name;
        this.currentPage = 1;
        const tableName = document.getElementById('dbTableName');
        if (tableName) tableName.textContent = name;
        this._renderTableList();
        this._renderStructure(name);
        this.switchTab('data');
        await this.loadData();
    },

    loadData: async function () {
        if (!this.currentTable) return;

        const grid = document.getElementById('dbDataGrid');
        grid.innerHTML = '<div class="db-loading">Loading data…</div>';

        try {
            const url = `/api/database/table/${encodeURIComponent(this.currentTable)}/data?page=${this.currentPage}&page_size=${this.pageSize}`;
            const resp = await fetch(url, { headers: this._apiHeaders() });
            if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
            const data = await resp.json();

            this.totalRows = data.total_rows || 0;
            this._renderGrid(data.columns || [], data.rows || []);
            this._updatePagination();
        } catch (e) {
            grid.innerHTML = `<div class="db-error">Failed to load data: ${this._esc(e.message)}</div>`;
        }
    },

    _renderGrid: function (columns, rows) {
        const grid = document.getElementById('dbDataGrid');
        if (!columns.length) {
            grid.innerHTML = '<div class="db-empty">No columns</div>';
            return;
        }
        if (!rows.length) {
            grid.innerHTML = '<div class="db-empty">Table is empty</div>';
            return;
        }

        let html = '<table class="db-grid"><thead><tr>';
        html += '<th class="db-row-num">#</th>';
        columns.forEach(c => { html += `<th>${this._esc(c)}</th>`; });
        html += '<th class="db-actions-col">Actions</th></tr></thead><tbody>';

        rows.forEach((row, i) => {
            const rowNum = (this.currentPage - 1) * this.pageSize + i + 1;
            html += '<tr>';
            html += `<td class="db-row-num">${rowNum}</td>`;
            row.forEach(cell => {
                const display = cell === null ? '<span class="db-null">NULL</span>' : this._esc(String(cell));
                html += `<td title="${this._esc(String(cell))}">${display}</td>`;
            });
            const pk = row[0];
            html += `<td class="db-actions-cell">
                <button class="db-btn-icon db-btn-danger" onclick="DB.confirmDelete('${this._esc(String(pk))}')" title="Delete">🗑</button>
            </td>`;
            html += '</tr>';
        });

        html += '</tbody></table>';
        grid.innerHTML = html;
    },

    _updatePagination: function () {
        const totalPages = Math.max(1, Math.ceil(this.totalRows / this.pageSize));
        document.getElementById('dbPageInfo').textContent = `Page ${this.currentPage} of ${totalPages}`;
        document.getElementById('dbRowCount').textContent = `${this.totalRows} rows`;
        document.getElementById('dbPrevBtn').disabled = this.currentPage <= 1;
        document.getElementById('dbNextBtn').disabled = this.currentPage >= totalPages;
    },

    prevPage: function () {
        if (this.currentPage > 1) {
            this.currentPage--;
            this.loadData();
        }
    },

    nextPage: function () {
        const totalPages = Math.max(1, Math.ceil(this.totalRows / this.pageSize));
        if (this.currentPage < totalPages) {
            this.currentPage++;
            this.loadData();
        }
    },

    _renderStructure: function (tableName) {
        const container = document.getElementById('dbStructure');
        if (!container) return;
        const table = this.schema.find(t => t.name === tableName);

        if (!table) {
            container.innerHTML = '<div class="db-empty">Table not found in schema</div>';
            return;
        }

        let html = '<table class="db-grid db-structure-table">';
        html += '<thead><tr><th>Column</th><th>Type</th><th>Nullable</th><th>Default</th><th>Key</th></tr></thead><tbody>';

        table.columns.forEach(col => {
            const keys = [];
            if (col.is_pk) keys.push('<span class="db-badge db-badge-pk">PK</span>');
            if (col.is_fk) keys.push('<span class="db-badge db-badge-fk">FK</span>');

            html += '<tr>';
            html += `<td><strong>${this._esc(col.name)}</strong></td>`;
            html += `<td><span class="db-type">${this._esc(col.data_type)}</span></td>`;
            html += `<td>${col.nullable ? 'YES' : '<strong>NO</strong>'}</td>`;
            html += `<td class="db-null">${col.default_value ? this._esc(col.default_value) : '—'}</td>`;
            html += `<td>${keys.join(' ')}</td>`;
            html += '</tr>';
        });

        html += '</tbody></table>';

        html += `<div class="db-table-actions">
            <button class="db-btn db-btn-danger" onclick="DB.confirmDropTable('${this._esc(tableName)}')">Drop Table</button>
        </div>`;

        container.innerHTML = html;
    },

    executeSql: async function () {
        const editor = document.getElementById('dbSqlEditor');
        const query = editor.value.trim();
        if (!query) return;

        const results = document.getElementById('dbSqlResults');
        const durationEl = document.getElementById('dbQueryDuration');
        results.innerHTML = '<div class="db-loading">Executing…</div>';

        const start = performance.now();

        try {
            const resp = await fetch('/api/database/query', {
                method: 'POST',
                headers: this._apiHeaders(),
                body: JSON.stringify({ query })
            });

            const elapsed = Math.round(performance.now() - start);
            durationEl.textContent = `${elapsed}ms`;

            if (!resp.ok) {
                const err = await resp.json().catch(() => ({ error: resp.statusText }));
                results.innerHTML = `<div class="db-error">Error: ${this._esc(err.error || 'Unknown error')}</div>`;
                return;
            }

            const data = await resp.json();

            if (data.is_mutation) {
                results.innerHTML = `<div class="db-success">Executed. ${data.row_count} row(s) affected. Duration: ${data.duration_ms}ms</div>`;
                this.loadData();
                this.refreshSchema();
                return;
            }

            if (!data.columns || !data.columns.length) {
                results.innerHTML = '<div class="db-empty">No results</div>';
                return;
            }

            this._renderGridTo(results, data.columns, data.rows || []);
        } catch (e) {
            results.innerHTML = `<div class="db-error">Error: ${this._esc(e.message)}</div>`;
        }
    },

    _renderGridTo: function (container, columns, rows) {
        let html = '<table class="db-grid"><thead><tr>';
        columns.forEach(c => { html += `<th>${this._esc(c)}</th>`; });
        html += '</tr></thead><tbody>';

        rows.forEach(row => {
            html += '<tr>';
            row.forEach(cell => {
                const display = cell === null ? '<span class="db-null">NULL</span>' : this._esc(String(cell));
                html += `<td title="${this._esc(String(cell))}">${display}</td>`;
            });
            html += '</tr>';
        });

        html += '</tbody></table>';
        container.innerHTML = html;
    },

    clearSql: function () {
        document.getElementById('dbSqlEditor').value = '';
        document.getElementById('dbSqlResults').innerHTML = '<div class="db-empty">Execute a query to see results</div>';
        document.getElementById('dbQueryDuration').textContent = '';
    },

    showInsertDialog: function () {
        if (!this.currentTable) return;

        const table = this.schema.find(t => t.name === this.currentTable);
        if (!table) return;

        document.getElementById('dbInsertTitle').textContent = `Insert into ${this.currentTable}`;

        let html = '';
        table.columns.forEach(col => {
            const required = !col.nullable && !col.default_value;
            html += `<div class="db-form-row">
                <label>${this._esc(col.name)} <span class="db-type">(${this._esc(col.data_type)})</span>${required ? ' <span class="db-required">*</span>' : ''}</label>
                <input type="text" data-col="${this._esc(col.name)}" placeholder="${this._esc(col.default_value || 'NULL')}" class="db-input">
            </div>`;
        });

        document.getElementById('dbInsertForm').innerHTML = html;
        document.getElementById('dbInsertModal').style.display = 'flex';
    },

    closeInsertDialog: function () {
        document.getElementById('dbInsertModal').style.display = 'none';
    },

    submitInsert: async function () {
        if (!this.currentTable) return;

        const inputs = document.querySelectorAll('#dbInsertForm .db-input');
        const data = {};

        inputs.forEach(input => {
            const col = input.dataset.col;
            const val = input.value.trim();
            if (val !== '') {
                if (/^\d+$/.test(val)) data[col] = parseInt(val, 10);
                else if (/^\d+\.\d+$/.test(val)) data[col] = parseFloat(val);
                else if (val.toLowerCase() === 'true') data[col] = true;
                else if (val.toLowerCase() === 'false') data[col] = false;
                else data[col] = val;
            }
        });

        if (Object.keys(data).length === 0) {
            alert('Provide at least one value');
            return;
        }

        try {
            const resp = await fetch(`/api/database/table/${encodeURIComponent(this.currentTable)}/row`, {
                method: 'POST',
                headers: this._apiHeaders(),
                body: JSON.stringify({ data })
            });

            if (!resp.ok) {
                const err = await resp.json().catch(() => ({ error: resp.statusText }));
                alert(`Insert failed: ${err.error}`);
                return;
            }

            this.closeInsertDialog();
            await this.loadData();
            this.refreshSchema();
        } catch (e) {
            alert(`Insert failed: ${e.message}`);
        }
    },

    confirmDelete: async function (pk) {
        if (!this.currentTable) return;
        if (!confirm(`Delete row with id ${pk}?`)) return;

        try {
            const resp = await fetch(
                `/api/database/table/${encodeURIComponent(this.currentTable)}/row/${encodeURIComponent(pk)}`,
                { method: 'DELETE', headers: this._apiHeaders() }
            );

            if (!resp.ok) {
                const err = await resp.json().catch(() => ({ error: resp.statusText }));
                alert(`Delete failed: ${err.error}`);
                return;
            }

            await this.loadData();
            this.refreshSchema();
        } catch (e) {
            alert(`Delete failed: ${e.message}`);
        }
    },

    confirmDropTable: async function (name) {
        if (!confirm(`DROP TABLE "${name}"? This cannot be undone.`)) return;

        try {
            const resp = await fetch(
                `/api/database/table/${encodeURIComponent(name)}`,
                { method: 'DELETE', headers: this._apiHeaders() }
            );

            if (!resp.ok) {
                const err = await resp.json().catch(() => ({ error: resp.statusText }));
                alert(`Drop failed: ${err.error}`);
                return;
            }

            this.currentTable = null;
            document.getElementById('dbTableName').textContent = '';
            document.getElementById('dbDataGrid').innerHTML = '<div class="db-empty">Select a table</div>';
            await this.refreshSchema();
        } catch (e) {
            alert(`Drop failed: ${e.message}`);
        }
    },

    exportCsv: function () {
        const grid = document.querySelector('#dbDataGrid table');
        if (!grid) return;

        const rows = Array.from(grid.querySelectorAll('tr'));
        const csv = rows.map(row =>
            Array.from(row.querySelectorAll('th, td'))
                .slice(0, -1)
                .map(cell => `"${cell.textContent.replace(/"/g, '""')}"`)
                .join(',')
        ).join('\n');

        const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `${this.currentTable || 'export'}.csv`;
        a.click();
        URL.revokeObjectURL(url);
    },

    _esc: function (str) {
        if (str === null || str === undefined) return '';
        const div = document.createElement('div');
        div.textContent = String(str);
        return div.innerHTML;
    }
};

(function(){ var __cb = () => DB.init(); if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
