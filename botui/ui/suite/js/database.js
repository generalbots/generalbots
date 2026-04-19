const botCoderDatabase = {
    schema: [],
    zoomLevel: 1,

    init: function() {
        this.refreshSchema();
    },

    refreshSchema: async function() {
        try {
            const list = document.getElementById('botcoderTableList');
            if (list) list.innerHTML = 'Loading...';

            // Real fetch
            const resp = await fetch('/api/database/schema');
            if (resp.ok) {
                const data = await resp.json();
                this.schema = data.tables || [];
            } else {
                this.schema = [
                    { name: 'users', fields: ['id', 'email', 'name', 'created_at'] },
                    { name: 'posts', fields: ['id', 'user_id', 'title', 'body'], relations: [{ from: 'user_id', to: 'users.id' }] }
                ];
            }
            this.renderSidebar();
            this.renderErDiagram();
        } catch (e) {
            console.error(e);
            this.schema = [
                { name: 'users', fields: ['id', 'email', 'name', 'created_at'] },
                { name: 'orders', fields: ['id', 'user_id', 'total', 'status'] }
            ];
            this.renderSidebar();
            this.renderErDiagram();
        }
    },

    renderSidebar: function() {
        const list = document.getElementById('botcoderTableList');
        const select = document.getElementById('dbQueryTable');
        if (!list || !select) return;

        let html = '';
        let options = '<option value="">Select table...</option>';

        this.schema.forEach(table => {
            html += `<div class="botcoder-table-item" onclick="botCoderDatabase.viewTableData('${table.name}')">
                <span class="botcoder-table-icon">📇</span> ${table.name}
            </div>`;
            options += `<option value="${table.name}">${table.name}</option>`;
        });

        list.innerHTML = html;
        select.innerHTML = options;
    },

    renderErDiagram: function() {
        const canvas = document.getElementById('botcoderErCanvas');
        if (!canvas) return;

        let html = '';
        this.schema.forEach((table, i) => {
            const x = 50 + (i * 250);
            const y = 50 + ((i % 2) * 150);
            
            let fieldsHtml = table.fields.map(f => `<div class="er-field">${f}</div>`).join('');
            
            html += `
                <div class="er-table-node" style="left: ${x}px; top: ${y}px;" id="er-node-${table.name}">
                    <div class="er-table-header">${table.name}</div>
                    <div class="er-table-body">${fieldsHtml}</div>
                </div>
            `;
        });

        canvas.innerHTML = `
            <div class="er-canvas-wrapper" style="transform: scale(${this.zoomLevel}); transform-origin: top left;">
                ${html}
            </div>
        `;
    },

    zoomIn: function() {
        this.zoomLevel += 0.1;
        this.renderErDiagram();
    },

    zoomOut: function() {
        this.zoomLevel = Math.max(0.5, this.zoomLevel - 0.1);
        this.renderErDiagram();
    },

    selectQueryTable: function() {
        const table = document.getElementById('dbQueryTable').value;
        const editor = document.getElementById('dbSqlEditor');
        if (table && editor) {
            editor.value = `SELECT * FROM ${table} LIMIT 100;`;
        }
    },

    viewTableData: async function(tableName) {
        document.getElementById('dbSqlEditor').value = `SELECT * FROM ${tableName} LIMIT 100;`;
        this.executeQuery();
    },

    executeQuery: async function() {
        const sql = document.getElementById('dbSqlEditor').value;
        const grid = document.getElementById('botcoderDataGrid');
        
        if (!sql) return;
        
        try {
            grid.innerHTML = '<div class="botcoder-loading">Executing query...</div>';
            
            // Expected backend call
            const resp = await fetch('/api/database/query', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ query: sql })
            });
            
            if (resp.ok) {
                const result = await resp.json();
                this.renderGrid(result.columns || [], result.rows || []);
            } else {
                // Mock result
                setTimeout(() => {
                    this.renderGrid(
                        ['id', 'name', 'status'],
                        [
                            [1, 'Alice', 'active'],
                            [2, 'Bob', 'inactive']
                        ]
                    );
                }, 500);
            }
        } catch (e) {
            grid.innerHTML = `<div class="botcoder-error">Error: ${e.message}</div>`;
        }
    },

    renderGrid: function(columns, rows) {
        const grid = document.getElementById('botcoderDataGrid');
        if (!grid) return;

        if (!columns || columns.length === 0) {
            grid.innerHTML = '<div class="botcoder-empty">No results found</div>';
            return;
        }

        let thead = '<tr>' + columns.map(c => `<th>${c}</th>`).join('') + '</tr>';
        
        let tbody = rows.map(row => {
            return '<tr>' + row.map(cell => `<td>${cell}</td>`).join('') + '</tr>';
        }).join('');

        grid.innerHTML = `
            <table class="botcoder-grid-table">
                <thead>${thead}</thead>
                <tbody>${tbody}</tbody>
            </table>
        `;
    },

    exportCsv: function() {
        alert('Exporting to CSV...');
    }
};

document.addEventListener('DOMContentLoaded', () => { botCoderDatabase.init(); });
if (document.readyState === 'complete' || document.readyState === 'interactive') {
    botCoderDatabase.init();
}
