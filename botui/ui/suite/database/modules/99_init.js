(function () {
    'use strict';
if (window.GBAppLifecycle) GBAppLifecycle.begin("database");

    var DB = window.DB;

    function ready(fn) {
        if (document.readyState !== 'loading') {
            fn();
        } else {
            document.addEventListener('DOMContentLoaded', fn);
        }
    }

    ready(function () {
        DB.bindGrid();

        var tableList = document.getElementById('db-table-list');
        if (tableList) {
            tableList.addEventListener('click', function (e) {
                var item = e.target.closest('.db-table-item');
                if (item) window.DBApp.selectTable(item.getAttribute('data-table'));
            });
        }

        var toolbar = document.getElementById('db-toolbar');
        if (toolbar) {
            toolbar.addEventListener('click', function (e) {
                var btn = e.target.closest('[data-tool]');
                if (!btn) return;
                var act = btn.getAttribute('data-tool');
                if (act === 'sql') window.DBApp.showQueryBuilder();
                else if (act === 'export') window.DBApp.exportTableCSV();
                else if (act === 'import') window.DBApp.importCSV();
                else if (act === 'add') window.DBApp.addNewRow();
            });
        }

        var viewTabs = document.getElementById('db-view-tabs');
        if (viewTabs) {
            viewTabs.addEventListener('click', function (e) {
                var tab = e.target.closest('.db-view-tab');
                if (tab) window.DBApp.switchView(tab.getAttribute('data-view'));
            });
        }

        var selBar = document.getElementById('db-selection-bar');
        if (selBar) {
            selBar.addEventListener('click', function (e) {
                var btn = e.target.closest('[data-sel]');
                if (!btn) return;
                if (btn.getAttribute('data-sel') === 'delete') window.DBApp.batchDeleteSelected();
                else if (btn.getAttribute('data-sel') === 'clear') window.DBApp.clearSelection();
            });
        }

        var pag = document.getElementById('db-pagination');
        if (pag) {
            pag.addEventListener('click', function (e) {
                var btn = e.target.closest('button');
                if (!btn) return;
                if (btn.id === 'db-prev-page') window.DBApp.prevPage();
                else if (btn.id === 'db-next-page') window.DBApp.nextPage();
            });
        }

        var sqlInput = document.getElementById('db-sql-input');
        if (sqlInput) {
            sqlInput.addEventListener('keydown', function (e) {
                if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
                    e.preventDefault();
                    window.DBApp.executeQuery();
                }
            });
        }

        var editModal = document.getElementById('db-cell-edit-modal');
        if (editModal) {
            editModal.querySelector('.db-save-btn').addEventListener('click', function () {
                var modal = document.getElementById('db-cell-edit-modal');
                if (modal.hasAttribute('data-new')) window.DBApp.saveNewRow();
                else window.DBApp.saveRowEdit();
            });
        }

        var pageSize = document.getElementById('db-page-size');
        if (pageSize) {
            pageSize.addEventListener('change', function () {
                window.DBApp.setPageSize(this.value);
            });
        }

        var newTableModal = document.getElementById('db-new-table-modal');
        if (newTableModal) {
            newTableModal.querySelector('.db-create-btn').addEventListener('click', function () {
                window.DBApp.createTable();
            });
            newTableModal.querySelector('.db-cancel-btn').addEventListener('click', function () {
                window.DBApp.hideNewTableModal();
            });
        }

        var importModal = document.getElementById('db-import-modal');
        if (importModal) {
            importModal.querySelector('.db-import-cancel').addEventListener('click', function () {
                window.DBApp.hideImportModal();
            });
            importModal.querySelector('.db-import-run').addEventListener('click', function () {
                window.DBApp.processImport();
            });
        }

        DB.loadSchema();
    });
})();
