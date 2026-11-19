/* Split from database.js for #2305 (450-line limit).
   #2307 — the insert modal markup was never shipped with an opener, and the
   dialog helpers assumed ids that do not exist in every host page. Everything
   below is null-safe and creates its own UI when the host page omits it. */
Object.assign(DB, {
    clearSql: function () {
        const editor = document.getElementById('dbSqlEditor');
        const results = document.getElementById('dbSqlResults');
        const duration = document.getElementById('dbQueryDuration');
        if (editor) editor.value = '';
        if (results) results.innerHTML = '<div class="db-empty">' + this._esc(this.t('db-empty-query', 'Execute a query to see results')) + '</div>';
        if (duration) duration.textContent = '';
    },

    _notify: function (message, isError) {
        var title = this.t(isError ? 'db-error' : 'db-database', isError ? 'Error' : 'Database');
        if (window.GBToasts && typeof window.GBToasts.show === 'function') {
            window.GBToasts.show(title, message, isError ? 'error' : 'info');
            return;
        }
        if (window.GBModal && typeof window.GBModal.alert === 'function') {
            window.GBModal.alert(message, title);
        }
    },

    _setInsertStatus: function (message, isError) {
        const el = document.getElementById('dbInsertStatus');
        if (!el) return;
        el.textContent = message || '';
        el.setAttribute('role', isError ? 'alert' : 'status');
        el.setAttribute('aria-live', isError ? 'assertive' : 'polite');
    },

    // The legacy partial ships #dbInsertModal without an opener; when the
    // partial is absent (e.g. an HTMX host page) build the whole dialog here.
    _ensureInsertModal: function () {
        const existing = document.getElementById('dbInsertModal');
        if (existing) return existing;
        if (!document.body) return null;
        const modal = document.createElement('div');
        modal.id = 'dbInsertModal';
        modal.className = 'botcoder-modal';
        modal.setAttribute('role', 'dialog');
        modal.setAttribute('aria-modal', 'true');
        modal.setAttribute('aria-labelledby', 'dbInsertTitle');
        modal.style.cssText = 'display:none;position:fixed;inset:0;background:rgba(0,0,0,.45);align-items:center;justify-content:center;z-index:var(--z-dropdown);';
        modal.innerHTML = '<div class="botcoder-modal-content" style="background:#fff;color:#111;border-radius:8px;padding:16px;width:min(720px,92vw);max-height:80vh;overflow:auto;">' +
            '<div style="display:flex;align-items:center;justify-content:space-between;gap:12px;margin-bottom:12px;">' +
            '<h3 id="dbInsertTitle" style="margin:0;" data-i18n="db-insert-row"></h3>' +
            '<button class="botcoder-btn" type="button" onclick="DB.closeInsertDialog()" data-i18n="action-close"></button>' +
            '</div>' +
            '<div id="dbInsertForm"></div>' +
            '<div style="display:flex;justify-content:flex-end;gap:8px;margin-top:12px;">' +
            '<button class="botcoder-btn" type="button" onclick="DB.closeInsertDialog()" data-i18n="action-cancel"></button>' +
            '<button class="botcoder-btn botcoder-btn-primary" type="button" onclick="DB.submitInsert()" data-i18n="db-insert"></button>' +
            '</div></div>';
        document.body.appendChild(modal);
        const title = document.getElementById('dbInsertTitle');
        if (title) title.textContent = this.t('db-insert-row', 'Insert row');
        const buttons = modal.querySelectorAll('button[data-i18n]');
        buttons.forEach(button => {
            const key = button.getAttribute('data-i18n');
            const fallback = key === 'action-close'
                ? 'Close'
                : (key === 'action-cancel' ? 'Cancel' : (key === 'db-insert' ? 'Insert' : 'Save'));
            button.textContent = this.t(key, fallback);
        });
        return modal;
    },

    showInsertDialog: function () {
        if (!this.currentTable) return;
        const table = this.schema.find(t => t.name === this.currentTable);
        if (!table || !Array.isArray(table.columns)) return;
        const modal = this._ensureInsertModal();
        if (!modal) return;
        const submit = modal.querySelector('[onclick="DB.submitInsert()"]');
        if (submit) {
            submit.setAttribute('data-i18n', 'db-insert');
            submit.textContent = this.t('db-insert', 'Insert');
        }
        const title = document.getElementById('dbInsertTitle');
        if (title) {
            title.textContent = this.t('db-insert-into', 'Insert into') + ' ' + this.currentTable;
        }
        const form = document.getElementById('dbInsertForm');
        if (!form) return;

        let html = '';
        table.columns.forEach(col => {
            const required = !col.nullable && !col.default_value;
            html += '<div class="db-form-row">' +
                '<label>' + this._esc(col.name) + ' <span class="db-type">(' +
                this._esc(col.data_type) + ')</span>' +
                (required ? ' <span class="db-required">*</span>' : '') + '</label>' +
                '<input type="text" data-col="' + this._esc(col.name) + '" placeholder="' +
                this._esc(col.default_value || 'NULL') + '" class="db-input"></div>';
        });
        html += '<div id="dbInsertStatus" class="db-form-status" role="status" aria-live="polite"></div>';
        form.innerHTML = html;
        modal.style.display = 'flex';
        const first = form.querySelector('.db-input');
        if (first) first.focus();
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
            if (!col || val === '') return;
            if (/^\d+$/.test(val)) data[col] = parseInt(val, 10);
            else if (/^\d+\.\d+$/.test(val)) data[col] = parseFloat(val);
            else if (val.toLowerCase() === 'true') data[col] = true;
            else if (val.toLowerCase() === 'false') data[col] = false;
            else data[col] = val;
        });

        if (Object.keys(data).length === 0) {
            this._setInsertStatus(this.t('db-provide-value', 'Provide at least one value'), true);
            return;
        }

        try {
            const resp = await fetch('/api/database/table/' + encodeURIComponent(this.currentTable) + '/row', {
                method: 'POST',
                headers: this._apiHeaders(),
                body: JSON.stringify({ data })
            });
            if (!resp.ok) {
                const err = await resp.json().catch(() => ({ error: resp.statusText }));
                this._setInsertStatus(this.t('db-insert-failed', 'Insert failed') + ': ' + (err.error || ''), true);
                return;
            }
            this.closeInsertDialog();
            await this.loadData();
            this.refreshSchema();
        } catch (e) {
            this._setInsertStatus(this.t('db-insert-failed', 'Insert failed') + ': ' + e.message, true);
        }
    },

    // #2307 — no native confirm(). Prefer the shared modal, otherwise render a
    // local one so the action is never silently skipped or auto-approved.
    _confirm: function (message, onYes) {
        if (window.GBModal && typeof window.GBModal.confirm === 'function') {
            window.GBModal.confirm(message, this.t('db-confirm-title', 'Confirm'), onYes);
            return;
        }
        let dialog = document.getElementById('dbConfirmDialog');
        if (!dialog) {
            if (!document.body) return;
            dialog = document.createElement('dialog');
            dialog.id = 'dbConfirmDialog';
            dialog.className = 'botcoder-modal';
            dialog.setAttribute('role', 'alertdialog');
            dialog.setAttribute('aria-labelledby', 'dbConfirmTitle');
            dialog.innerHTML = '<div class="botcoder-modal-content" style="background:#fff;color:#111;border-radius:8px;padding:16px;width:min(480px,92vw);">' +
                '<h3 id="dbConfirmTitle" style="margin:0 0 12px;" data-i18n="db-confirm-title"></h3>' +
                '<p class="db-confirm-message" data-db-confirm-message role="status" aria-live="polite"></p>' +
                '<div style="display:flex;justify-content:flex-end;gap:8px;margin-top:12px;">' +
                '<button class="botcoder-btn" type="button" data-db-confirm-cancel data-i18n="action-cancel"></button>' +
                '<button class="botcoder-btn botcoder-btn-primary" type="button" data-db-confirm-accept data-i18n="db-confirm-accept"></button>' +
                '</div></div>';
            document.body.appendChild(dialog);
            const heading = document.getElementById('dbConfirmTitle');
            if (heading) heading.textContent = this.t('db-confirm-title', 'Confirm');
            const cancel = dialog.querySelector('[data-db-confirm-cancel]');
            const accept = dialog.querySelector('[data-db-confirm-accept]');
            if (cancel) {
                cancel.textContent = this.t('action-cancel', 'Cancel');
                cancel.addEventListener('click', () => {
                    dialog.__dbAction = null;
                    if (dialog.open) dialog.close();
                });
            }
            if (accept) {
                accept.textContent = this.t('db-confirm-accept', 'Confirm');
                accept.addEventListener('click', () => {
                    const action = dialog.__dbAction;
                    dialog.__dbAction = null;
                    if (dialog.open) dialog.close();
                    if (typeof action === 'function') action();
                });
            }
        }
        const messageEl = dialog.querySelector('[data-db-confirm-message]');
        if (messageEl) messageEl.textContent = message;
        dialog.__dbAction = onYes;
        if (!dialog.open && typeof dialog.showModal === 'function') dialog.showModal();
    },

    confirmDelete: function (pk) {
        if (!this.currentTable) return;
        const run = async () => {
            try {
                const resp = await fetch(
                    '/api/database/table/' + encodeURIComponent(this.currentTable) + '/row/' + encodeURIComponent(pk),
                    { method: 'DELETE', headers: this._apiHeaders() }
                );
                if (!resp.ok) {
                    const err = await resp.json().catch(() => ({ error: resp.statusText }));
                    this._notify(this.t('db-delete-failed', 'Delete failed') + ': ' + (err.error || ''), true);
                    return;
                }
                await this.loadData();
                this.refreshSchema();
            } catch (e) {
                this._notify(this.t('db-delete-failed', 'Delete failed') + ': ' + e.message, true);
            }
        };
        this._confirm(this.t('db-confirm-delete', 'Delete this row?'), run);
    },

    confirmDropTable: function (name) {
        const run = async () => {
            try {
                const resp = await fetch('/api/database/table/' + encodeURIComponent(name), {
                    method: 'DELETE',
                    headers: this._apiHeaders()
                });
                if (!resp.ok) {
                    const err = await resp.json().catch(() => ({ error: resp.statusText }));
                    this._notify(this.t('db-drop-failed', 'Drop failed') + ': ' + (err.error || ''), true);
                    return;
                }
                this.currentTable = null;
                const tableName = document.getElementById('dbTableName');
                const grid = document.getElementById('dbDataGrid');
                if (tableName) tableName.textContent = '';
                if (grid) grid.innerHTML = '<div class="db-empty">' + this._esc(this.t('db-select-table', 'Select a table')) + '</div>';
                await this.refreshSchema();
            } catch (e) {
                this._notify(this.t('db-drop-failed', 'Drop failed') + ': ' + e.message, true);
            }
        };
        this._confirm(this.t('db-confirm-drop', 'Drop table') + ' "' + name + '"?', run);
    },

    exportCsv: function () {
        const grid = document.querySelector('#dbDataGrid table');
        if (!grid) return;
        const rows = Array.from(grid.querySelectorAll('tr'));
        const csv = rows.map(row =>
            Array.from(row.querySelectorAll('th, td'))
                .slice(0, -1)
                .map(cell => '"' + cell.textContent.replace(/"/g, '""') + '"')
                .join(',')
        ).join('\n');
        const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = (this.currentTable || 'export') + '.csv';
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
