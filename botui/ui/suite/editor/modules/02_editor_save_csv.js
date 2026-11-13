/* Editor app — module 2/3: auth fetch, save and CSV mode (split from editor-inline-panels.js for #1731) */

    // #2093 — translated text with an authored English fallback so a missing
    // catalogue entry never leaks a raw key into the editor UI.
    function t(key, fallback, params) {
        return window.gbT ? window.gbT(key, fallback, params) : fallback;
    }

        // Add auth token to HTMX requests
        var gbToken = localStorage.getItem('gb-access-token') || '';
        if (gbToken && window.htmx) {
            document.body.addEventListener('htmx:configRequest', function(e) {
                e.detail.headers['Authorization'] = 'Bearer ' + gbToken;
            });
        }

        // Helper to make authenticated fetch
        function authFetch(url, options) {
            options = options || {};
            options.headers = options.headers || {};
            if (gbToken) options.headers['Authorization'] = 'Bearer ' + gbToken;
            return fetch(url, options);
        }

        // #2282 — resolve the editor window root and its Drive file context so
        // two editor windows keep independent file contexts.
        function gbEditorResolveRoot(el) {
            var node = el && el.nodeType ? el : (el && el.target ? el.target : null);
            if (node && node.closest) {
                var w = node.closest('[id^="window-body-"]');
                if (w) return w;
            }
            var ae = document.activeElement;
            if (ae && ae.closest) {
                var aw = ae.closest('[id^="window-body-"]');
                if (aw) return aw;
            }
            return document;
        }
        function gbEditorFileCtx(root) {
            root = root || document;
            var ds = root.dataset || {};
            if (ds.gbFilePath) {
                return { bucket: ds.gbFileBucket || '', path: ds.gbFilePath, scope: ds.gbFileScope || 'bot' };
            }
            var wid = ds.windowId || (root.id || '').replace('window-body-', '');
            var map = (window.__EDITOR_FILE_CTX && window.__EDITOR_FILE_CTX[wid]) ||
                (window.__GB_WINDOW_FILE_CTX && window.__GB_WINDOW_FILE_CTX[wid]);
            if (map && map.path) return { bucket: map.bucket || '', path: map.path, scope: map.scope || 'bot' };
            var params = new URLSearchParams(window.location.search);
            return {
                bucket: window.__EDITOR_FILE_BUCKET || params.get('bucket') || '',
                path: window.__EDITOR_FILE_PATH || params.get('file') || '',
                scope: window.__EDITOR_FILE_SCOPE || 'bot',
            };
        }

        function closeEditorWindow(el) {
            var root = gbEditorResolveRoot(el);
            var w = root && root.id ? document.getElementById(root.id.replace('window-body-', 'window-')) : null;
            if (!w && root && root.closest) w = root.closest('[id^="window-"]');
            if (w) {
                var id = w.id.replace('window-', '');
                if (window.WindowManager) window.WindowManager.close(id);
                return;
            }
            if (window.htmx) htmx.ajax('GET', '/api/files/list', { target: '#main-content', pushUrl: true });
            else window.location.href = '/suite/drive/drive.html';
        }

        // Save file content back to Drive (scoped to the invoking window)
        function saveEditorFile(el) {
            var root = gbEditorResolveRoot(el);
            var ctx = gbEditorFileCtx(root);
            var filePath = ctx.path || '';
            var bucket = ctx.bucket || '';
            var scope = ctx.scope || 'bot';
            if (!filePath) { showSaveNotification({detail:{successful:false}}); return; }
            var content = '';
            var csvActive = root && root.__csvActive;
            if (csvActive) {
                var table = root.querySelector('#csv-editor table.csv-table') || root.querySelector('.csv-table');
                if (table) {
                    var rows = Array.from(table.querySelectorAll('tbody tr'));
                    var headers = Array.from(table.querySelectorAll('thead th')).slice(1).map(function(th) { return th.textContent; });
                    var lines = [headers.join(',')];
                    rows.forEach(function(tr) {
                        var inputs = Array.from(tr.querySelectorAll('.csv-input'));
                        var vals = inputs.map(function(inp) { return '"' + inp.value.replace(/"/g, '""') + '"'; });
                        lines.push(vals.join(','));
                    });
                    content = lines.join('\n');
                }
            } else {
                var ta = root.querySelector ? root.querySelector('#text-editor') : document.getElementById('text-editor');
                content = ta ? ta.value : '';
                if (root && root.__monacoEditorInstance) content = root.__monacoEditorInstance.getValue();
                else if (window.monacoEditorInstance && !root.__vibeProjectId) content = window.monacoEditorInstance.getValue();
            }
            var spinner = root.querySelector ? root.querySelector('#save-spinner') : document.getElementById('save-spinner');
            if (spinner) spinner.style.display = 'inline-block';
            var pathParts = filePath.split('/');
            var relativePath = pathParts.slice(2).join('/');
            if (!relativePath) relativePath = filePath;
            authFetch('/api/files/write', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    bucket: bucket,
                    path: relativePath,
                    content: btoa(content),
                    scope: scope
                })
            }).then(function(r) {
                if (spinner) spinner.style.display = 'none';
                if (r.ok) {
                    showSaveNotification({detail:{successful:true}});
                    var dirty = root.querySelector ? root.querySelector('#dirty-indicator') : document.getElementById('dirty-indicator');
                    if (dirty) dirty.style.display = 'none';
                } else {
                    showSaveNotification({detail:{successful:false}});
                }
            }).catch(function() {
                if (spinner) spinner.style.display = 'none';
                showSaveNotification({detail:{successful:false}});
            });
        }

        function switchToCsvMode(csvData, root) {
            root = gbEditorResolveRoot(root);
            var textWrapper = root.querySelector ? root.querySelector('#text-editor-wrapper') : document.getElementById('text-editor-wrapper');
            var csvSection = root.querySelector ? root.querySelector('#csv-editor') : document.getElementById('csv-editor');
            var textTools = root.querySelector ? root.querySelector('#text-tools') : document.getElementById('text-tools');
            var csvTools = root.querySelector ? root.querySelector('#csv-tools') : document.getElementById('csv-tools');
            if (textWrapper) textWrapper.style.display = 'none';
            if (csvSection) { csvSection.style.display = 'block'; csvSection.innerHTML = ''; }
            if (csvTools) csvTools.style.display = 'flex';
            if (textTools) textTools.style.display = 'none';
            var monacoContainer = root.querySelector ? root.querySelector('#monaco-editor') : document.getElementById('monaco-editor');
            if (monacoContainer) monacoContainer.style.display = 'none';
            if (!csvData) return;
            var lines = csvData.split('\n').filter(function(l) { return l.trim(); });
            if (lines.length === 0) return;
            var headers = lines[0].split(',').map(function(h) { return h.trim().replace(/^"|"$/g, ''); });
            var table = document.createElement('table');
            table.className = 'csv-table';
            var thead = document.createElement('thead');
            var headRow = document.createElement('tr');
            var cornerTh = document.createElement('th');
            cornerTh.className = 'row-num';
            cornerTh.textContent = '#';
            headRow.appendChild(cornerTh);
            headers.forEach(function(h) {
                var th = document.createElement('th');
                th.textContent = h;
                headRow.appendChild(th);
            });
            thead.appendChild(headRow);
            table.appendChild(thead);
            var tbody = document.createElement('tbody');
            for (var ri = 1; ri < lines.length; ri++) {
                var vals = parseCsvLine(lines[ri]);
                var tr = document.createElement('tr');
                var tdNum = document.createElement('td');
                tdNum.className = 'row-num';
                tdNum.textContent = ri;
                tr.appendChild(tdNum);
                vals.forEach(function(v) {
                    var td = document.createElement('td');
                    var input = document.createElement('input');
                    input.type = 'text';
                    input.className = 'csv-input';
                    input.value = v;
                    input.dataset.row = ri - 1;
                    input.dataset.col = headers.indexOf(v);
                    tr.appendChild(td);
                    td.appendChild(input);
                });
                tbody.appendChild(tr);
            }
            table.appendChild(tbody);
            csvSection = root.querySelector ? root.querySelector('#csv-editor') : document.getElementById('csv-editor');
            if (csvSection) { csvSection.innerHTML = ''; csvSection.appendChild(table); }
            if (root && root.dataset) root.__csvActive = true;
            window.__CSV_DATA = csvData;
            window.__CSV_ACTIVE = true;
        }

        function csvAddRow(el) {
            var root = gbEditorResolveRoot(el);
            var table = root.querySelector ? root.querySelector('#csv-editor table.csv-table') : document.querySelector('#csv-editor table.csv-table');
            if (!table) return;
            var tbody = table.querySelector('tbody');
            if (!tbody) { tbody = document.createElement('tbody'); table.appendChild(tbody); }
            var cols = table.querySelectorAll('thead th').length - 1;
            var rowNum = tbody.children.length + 1;
            var tr = document.createElement('tr');
            var tdNum = document.createElement('td');
            tdNum.className = 'row-num';
            tdNum.textContent = rowNum;
            tr.appendChild(tdNum);
            for (var c = 0; c < cols; c++) {
                var td = document.createElement('td');
                var input = document.createElement('input');
                input.type = 'text';
                input.className = 'csv-input';
                input.value = '';
                td.appendChild(input);
                tr.appendChild(td);
            }
            tbody.appendChild(tr);
            window.__CSV_DIRTY = true;
            if (root && root.dataset) root.__csvDirty = true;
            var dirty = root.querySelector ? root.querySelector('#dirty-indicator') : document.getElementById('dirty-indicator');
            if (dirty) dirty.style.display = 'inline-block';
        }

        function csvAddColumn(el) {
            var root = gbEditorResolveRoot(el);
            var table = root.querySelector ? root.querySelector('#csv-editor table.csv-table') : document.querySelector('#csv-editor table.csv-table');
            if (!table) return;
            var thead = table.querySelector('thead');
            if (thead) {
                var th = document.createElement('th');
                th.textContent = t('editor-column-number', 'Column ' + thead.querySelectorAll('th').length, { number: thead.querySelectorAll('th').length });
                thead.appendChild(th);
            }
            var rows = table.querySelectorAll('tbody tr');
            rows.forEach(function(tr) {
                var td = document.createElement('td');
                var input = document.createElement('input');
                input.type = 'text';
                input.className = 'csv-input';
                input.value = '';
                td.appendChild(input);
                tr.appendChild(td);
            });
            window.__CSV_DIRTY = true;
            if (root && root.dataset) root.__csvDirty = true;
            var dirty = root.querySelector ? root.querySelector('#dirty-indicator') : document.getElementById('dirty-indicator');
            if (dirty) dirty.style.display = 'inline-block';
        }

        function parseCsvLine(line) {
            var result = [], current = '', inQuotes = false;
            for (var i = 0; i < line.length; i++) {
                var c = line[i];
                if (inQuotes) {
                    if (c === '"' && line[i+1] === '"') { current += '"'; i++; }
                    else if (c === '"') { inQuotes = false; }
                    else { current += c; }
                } else {
                    if (c === '"') { inQuotes = true; }
                    else if (c === ',') { result.push(current.trim()); current = ''; }
                    else { current += c; }
                }
            }
            result.push(current.trim());
            return result;
        }
