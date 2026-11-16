/* Split from database.js for #2305 (450-line limit). */
Object.assign(DB, {
    clearSql: function () {
        const editor = document.getElementById('dbSqlEditor');
        const results = document.getElementById('dbSqlResults');
        const duration = document.getElementById('dbQueryDuration');
        if (editor) editor.value = '';
        if (results) results.innerHTML = '<div class="db-empty">Execute a query to see results</div>';
        if (duration) duration.textContent = '';
    },

    showInsertDialog: function () {
        if (!this.currentTable) return;

        const table = this.schema.find(t => t.name === this.currentTable);
        if (!table) return;

        const title = document.getElementById('dbInsertTitle');
        const form = document.getElementById('dbInsertForm');
        const modal = document.getElementById('dbInsertModal');
        if (!title || !form || !modal) {
            if (window.GBModal && window.GBModal.alert) window.GBModal.alert('Insert dialog is unavailable in this view.', 'Database');
            else if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Database', 'Insert dialog is unavailable in this view.', 'error');
            return;
        }
        title.textContent = `Insert into ${this.currentTable}`;

        let html = '';
        table.columns.forEach(col => {
            const required = !col.nullable && !col.default_value;
            html += `<div class="db-form-row">
                <label>${this._esc(col.name)} <span class="db-type">(${this._esc(col.data_type)})</span>${required ? ' <span class="db-required">*</span>' : ''}</label>
                <input type="text" data-col="${this._esc(col.name)}" placeholder="${this._esc(col.default_value || 'NULL')}" class="db-input">
            </div>`;
        });

        form.innerHTML = html;
        modal.style.display = 'flex';
    },

    closeInsertDialog: function () {
        const modal = document.getElementById('dbInsertModal');
        if (modal) modal.style.display = 'none';
    },

    submitInsert: async function () {
        if (!this.currentTable) return;

        const form = document.getElementById('dbInsertForm');
        if (!form) return;
        const inputs = form.querySelectorAll('.db-input');
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
            if (window.GBModal && window.GBModal.alert) window.GBModal.alert('Provide at least one value', 'Database');
            else if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Database', 'Provide at least one value', 'error');
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
                const msg = `Insert failed: ${err.error}`;
                if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Database', msg, 'error');
                else if (window.GBModal && window.GBModal.alert) window.GBModal.alert(msg, 'Database');
                return;
            }

            this.closeInsertDialog();
            await this.loadData();
            this.refreshSchema();
        } catch (e) {
            const msg = `Insert failed: ${e.message}`;
            if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Database', msg, 'error');
            else if (window.GBModal && window.GBModal.alert) window.GBModal.alert(msg, 'Database');
        }
    },

    confirmDelete: function (pk) {
        if (!this.currentTable) return;
        const run = async () => {
            try {
                const resp = await fetch(
                    `/api/database/table/${encodeURIComponent(this.currentTable)}/row/${encodeURIComponent(pk)}`,
                    { method: 'DELETE', headers: this._apiHeaders() }
                );

                if (!resp.ok) {
                    const err = await resp.json().catch(() => ({ error: resp.statusText }));
                    const msg = `Delete failed: ${err.error}`;
                    if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Database', msg, 'error');
                    else if (window.GBModal && window.GBModal.alert) window.GBModal.alert(msg, 'Database');
                    return;
                }

                await this.loadData();
                this.refreshSchema();
            } catch (e) {
                const msg = `Delete failed: ${e.message}`;
                if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Database', msg, 'error');
                else if (window.GBModal && window.GBModal.alert) window.GBModal.alert(msg, 'Database');
            }
        };
        if (window.GBModal && window.GBModal.confirm) {
            window.GBModal.confirm(`Delete row with id ${pk}?`, 'Confirm', () => { run(); });
            return;
        }
    },

    confirmDropTable: function (name) {
        const run = async () => {
            try {
                const resp = await fetch(
                    `/api/database/table/${encodeURIComponent(name)}`,
                    { method: 'DELETE', headers: this._apiHeaders() }
                );

                if (!resp.ok) {
                    const err = await resp.json().catch(() => ({ error: resp.statusText }));
                    const msg = `Drop failed: ${err.error}`;
                    if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Database', msg, 'error');
                    else if (window.GBModal && window.GBModal.alert) window.GBModal.alert(msg, 'Database');
                    return;
                }

                this.currentTable = null;
                const tableName = document.getElementById('dbTableName');
                const grid = document.getElementById('dbDataGrid');
                if (tableName) tableName.textContent = '';
                if (grid) grid.innerHTML = '<div class="db-empty">Select a table</div>';
                await this.refreshSchema();
            } catch (e) {
                const msg = `Drop failed: ${e.message}`;
                if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Database', msg, 'error');
                else if (window.GBModal && window.GBModal.alert) window.GBModal.alert(msg, 'Database');
            }
        };
        if (window.GBModal && window.GBModal.confirm) {
            window.GBModal.confirm(`DROP TABLE "${name}"? This cannot be undone.`, 'Confirm', () => { run(); });
            return;
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
});


(function(){ var __cb = () => DB.init(); if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
