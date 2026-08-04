(function () {
    'use strict';

    window.DB = window.DB || {};

    var DB = window.DB;

    DB.API = '/api/database';
    DB.currentTable = null;
    DB.currentPage = 0;
    DB.pageSize = 50;
    DB.totalCount = 0;
    DB.tables = [];
    DB.columns = [];
    DB.typeMap = {};
    DB.rows = [];
    DB.pkColumn = null;
    DB.rowPks = [];
    DB.sortColumn = null;
    DB.sortOrder = 'asc';
    DB.currentView = 'grid';
    DB.selected = new Set();
    DB.activeCell = null;
    DB.editing = false;
    DB.columnWidths = {};
    DB.widthKey = '';

    DB.sanitize = function (str) {
        var d = document.createElement('div');
        d.textContent = str == null ? '' : String(str);
        return d.innerHTML;
    };

    DB.typeOf = function (colName) {
        return (DB.typeMap[colName] || '').toLowerCase();
    };

    DB.isPK = function (colName) {
        for (var i = 0; i < DB.columns.length; i++) {
            if (DB.columns[i].name === colName) return !!DB.columns[i].is_pk;
        }
        return false;
    };

    DB.isNullable = function (colName) {
        for (var i = 0; i < DB.columns.length; i++) {
            if (DB.columns[i].name === colName) return DB.columns[i].nullable !== false;
        }
        return true;
    };

    DB.widthFor = function (colName) {
        return DB.columnWidths[colName] || 160;
    };

    DB.applyWidths = function () {
        try {
            var raw = localStorage.getItem(DB.widthKey);
            if (raw) {
                DB.columnWidths = JSON.parse(raw);
            }
        } catch (e) {
            DB.columnWidths = {};
        }
    };

    DB.persistWidths = function () {
        try {
            localStorage.setItem(DB.widthKey, JSON.stringify(DB.columnWidths));
        } catch (e) {
            /* storage unavailable — widths are session-scoped only */
        }
    };
})();
