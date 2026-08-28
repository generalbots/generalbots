// Minimal JS for notification display (could be replaced with htmx extension)
            function showSaveNotification(event) {
                const notification = document.getElementById('notification');
                if (event.detail.successful) {
                    notification.textContent = '✓ File saved';
                    notification.className = 'notification success show';
                    document.getElementById('dirty-indicator').style.display = 'none';
                } else {
                    notification.textContent = '✗ Save failed';
                    notification.className = 'notification error show';
                }
                setTimeout(() => notification.classList.remove('show'), 3000);
            }

            // Mark as dirty on edit
            document.getElementById('text-editor')?.addEventListener('input', function() {
                document.getElementById('dirty-indicator').style.display = 'inline-block';
            });

            // Keyboard shortcuts
            document.addEventListener('keydown', function(e) {
                if ((e.ctrlKey || e.metaKey) && e.key === 's') {
                    e.preventDefault();
                    saveEditorFile();
                }
            });
        function showMagicPanel() {
            document.getElementById('magic-panel').classList.add('visible');
            runMagicAnalysis();
        }

        function hideMagicPanel() {
            document.getElementById('magic-panel').classList.remove('visible');
        }

        async function runMagicAnalysis() {
            const content = document.getElementById('magic-content');
            const editor = document.getElementById('text-editor');

            if (!content || !editor) return;
            const code = editor.value;

            if (!code.trim()) {
                content.innerHTML = '<p style="color:var(--text-secondary, #94a3b8);text-align:center;padding:40px;">No code to analyze. Start typing or open a file.</p>';
                return;
            }

            content.innerHTML = '<div class="magic-loading">✨ Analyzing your code...</div>';

            try {
                const response = await fetch('/api/editor/magic', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ code: code })
                });

                if (response.ok) {
                    const result = await response.json();
                    renderMagicResult(result);
                } else {
                    content.innerHTML = '<p style="color:var(--error);padding:20px;">Failed to analyze. Try again.</p>';
                }
            } catch (e) {
                content.innerHTML = '<p style="color:var(--error);padding:20px;">Error connecting to AI service.</p>';
            }
        }

        function renderMagicResult(result) {
            const content = document.getElementById('magic-content');
            if (result.improved_code) {
                content.innerHTML = `
                    <div class="magic-result">
                        <p><strong>Suggested improvements:</strong></p>
                        <p style="color:var(--text-secondary, #94a3b8);margin:8px 0;">${result.explanation || 'Improved code structure and patterns.'}</p>
                        <pre>${escapeHtml(result.improved_code)}</pre>
                        <button class="magic-apply-btn" onclick="applyMagicCode()">Apply Changes</button>
                    </div>
                `;
                window.magicImprovedCode = result.improved_code;
            } else if (result.suggestions) {
                content.innerHTML = result.suggestions.map(s => `
                    <div class="magic-result">
                        <p><strong>${s.title}</strong></p>
                        <p style="color:var(--text-secondary, #94a3b8);">${s.description}</p>
                    </div>
                `).join('');
            } else {
                content.innerHTML = '<p style="padding:20px;">Your code looks good! No suggestions at this time.</p>';
            }
        }

        function applyMagicCode() {
            if (window.magicImprovedCode) {
                document.getElementById('text-editor').value = window.magicImprovedCode;
                hideMagicPanel();
            }
        }

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }

        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey && e.key === 'm') {
                e.preventDefault();
                showMagicPanel();
            }
        });

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

        function closeEditorWindow() {
            // Find the window element and close via WindowManager
            var el = document.getElementById('editor-filename');
            if (!el) { window.location.href = '/suite/desktop.html#'; return; }
            // Walk up to find window-container
            var w = el.closest('[id^="window-"]');
            if (w) {
                var id = w.id.replace('window-', '');
                if (window.WindowManager) window.WindowManager.close(id);
                return;
            }
            // Fallback: navigate to drive
            if (window.htmx) htmx.ajax('GET', '/api/files/list', { target: '#main-content', pushUrl: true });
            else window.location.href = '/suite/drive/drive.html';
        }

        // Save file content back to Drive
        function saveEditorFile() {
            var filePath = window.__EDITOR_FILE_PATH || '';
            var params = new URLSearchParams(window.location.search);
            if (!filePath) filePath = params.get('file') || '';
            var bucket = window.__EDITOR_FILE_BUCKET || params.get('bucket') || '';
            var scope = window.__EDITOR_FILE_SCOPE || 'bot';
            if (!filePath) { showSaveNotification({detail:{successful:false}}); return; }
            var content = '';
            if (window.__CSV_ACTIVE) {
                // Rebuild CSV from table inputs
                var table = document.querySelector('#csv-editor table.csv-table');
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
                var ta = document.getElementById('text-editor');
                content = ta ? ta.value : '';
                if (window.monacoEditorInstance) content = window.monacoEditorInstance.getValue();

            }
            var spinner = document.getElementById('save-spinner');
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
                    document.getElementById('dirty-indicator').style.display = 'none';
                } else {
                    showSaveNotification({detail:{successful:false}});
                }
            }).catch(function() {
                if (spinner) spinner.style.display = 'none';
                showSaveNotification({detail:{successful:false}});
            });
        }

        function switchToCsvMode(csvData) {
            var textWrapper = document.getElementById('text-editor-wrapper');
            var csvSection = document.getElementById('csv-editor');
            var textTools = document.getElementById('text-tools');
            var csvTools = document.getElementById('csv-tools');
            if (textWrapper) textWrapper.style.display = 'none';
            if (csvSection) { csvSection.style.display = 'block'; csvSection.innerHTML = ''; }
            if (csvTools) csvTools.style.display = 'flex';
            if (textTools) textTools.style.display = 'none';
            var monacoContainer = document.getElementById('monaco-editor');
            if (monacoContainer) monacoContainer.style.display = 'none';
            // Build CSV table
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
            headers.forEach(function(h, ci) {
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
            var csvSection = document.getElementById('csv-editor');
            if (csvSection) { csvSection.innerHTML = ''; csvSection.appendChild(table); }
            window.__CSV_DATA = csvData;
            window.__CSV_ACTIVE = true;
        }

        function csvAddRow() {
            var table = document.querySelector('#csv-editor table.csv-table');
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
            document.getElementById('dirty-indicator').style.display = 'inline-block';
        }

        function csvAddColumn() {
            var table = document.querySelector('#csv-editor table.csv-table');
            if (!table) return;
            var thead = table.querySelector('thead');
            if (thead) {
                var th = document.createElement('th');
                th.textContent = 'Column ' + (thead.querySelectorAll('th').length);
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
            document.getElementById('dirty-indicator').style.display = 'inline-block';
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

        // Set filename SYNCHRONOUSLY — no async microtask delay
        (function() {
            var filePath = window.__EDITOR_FILE_PATH || '';
            var params = new URLSearchParams(window.location.search);
            if (!filePath) filePath = params.get('file') || '';
            if (filePath) {
                var fileName = filePath.split('/').pop();
                var fnEl = document.getElementById('editor-filename');
                var fpEl = document.getElementById('editor-filepath');
                if (fnEl) fnEl.textContent = fileName;
                if (fpEl) fpEl.textContent = filePath;
            }
        })();

        // Load file content asynchronously
        var editorBootPromise = window.__EDITOR_BOOT || Promise.resolve();
        editorBootPromise.then(function() {
            var filePath = window.__EDITOR_FILE_PATH || '';
            var params = new URLSearchParams(window.location.search);
            if (!filePath) filePath = params.get('file') || '';
            var bucket = window.__EDITOR_FILE_BUCKET || params.get('bucket') || '';
            var scope = window.__EDITOR_FILE_SCOPE || 'bot';
            if (filePath) {
                var isCsv = filePath.toLowerCase().endsWith('.csv');
                var pathParts = filePath.split('/');
                var relativePath = pathParts.slice(2).join('/');
                if (!relativePath) relativePath = filePath;
                authFetch('/api/files/download', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ bucket: bucket, path: relativePath, scope: scope })
                }).then(function(r) { return r.json(); })
                .then(function(data) {
                    if (data.content) {
                        var decoded = atob(data.content);
                        if (isCsv) {
                            switchToCsvMode(decoded);
                        } else {
                            var ta = document.getElementById('text-editor');
                            if (ta) {
                                ta.value = decoded;
                                if (window.monacoEditorInstance) {
                                    window.monacoEditorInstance.setValue(decoded);
                                }
                            }
                        }
                    }
                }).catch(function(err) {
                    console.warn('Failed to load file:', err);
                });
            }
        });

        // ================================================================
        // Workspace explorer (#editor-tree) — project VM dev source.
        // When the Editor is opened from Vibe with a project context, the
        // sidebar lists the project workspace files (the same files the LLM
        // edits through chat). The user opens, edits and saves them here
        // BEFORE commit (Source Control) or deploy to production.
        // ================================================================
        var vibeProjectId = null;
        var vibeFilePath = null;

        function vibeProjectContext() {
            var p = window.__gbAppParams__ || {};
            var qs = new URLSearchParams(window.location.search);
            return p.project || qs.get('project') || null;
        }

        function vibeSetEditorValue(content) {
            var ta = document.getElementById('text-editor');
            if (ta) ta.value = content;
            if (window.monacoEditorInstance) {
                window.monacoEditorInstance.setValue(content);
            }
        }

        function vibeEditorContent() {
            if (window.monacoEditorInstance) return window.monacoEditorInstance.getValue();
            var ta = document.getElementById('text-editor');
            return ta ? ta.value : '';
        }

        function vibeSetActiveFile(path) {
            vibeFilePath = path;
            var fileName = path.split('/').pop();
            var fnEl = document.getElementById('editor-filename');
            var fpEl = document.getElementById('editor-filepath');
            if (fnEl) fnEl.textContent = fileName;
            if (fpEl) fpEl.textContent = 'dev VM workspace / ' + path;
            var dirty = document.getElementById('dirty-indicator');
            if (dirty) dirty.style.display = 'none';
            document.querySelectorAll('#editor-tree .editor-tree-file.active').forEach(function (n) {
                n.classList.remove('active');
            });
            var node = document.querySelector('#editor-tree [data-vibe-path="' + CSS.escape(path) + '"]');
            if (node) node.classList.add('active');
        }

        function vibeTreeHtml(files) {
            // Build a nested tree from sorted relative paths.
            var root = {};
            files.forEach(function (p) {
                var parts = p.split('/');
                var node = root;
                for (var i = 0; i < parts.length; i++) {
                    var seg = parts[i];
                    if (i === parts.length - 1) {
                        (node.__files = node.__files || []).push(seg);
                    } else {
                        node = node[seg] = node[seg] || {};
                    }
                }
            });
            function esc(s) {
                return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/"/g, '&quot;');
            }
            function renderFolder(map, prefix) {
                var html = '';
                Object.keys(map).filter(function (k) { return k !== '__files'; }).sort().forEach(function (folder) {
                    html += '<div class="editor-tree-folder" data-vibe-folder>' +
                        '<span class="tw">\u25b8</span>\ud83d\udcc1 ' + esc(folder) + '</div>' +
                        '<div class="editor-tree-children" style="display:none">' +
                        renderFolder(map[folder], prefix + folder + '/') + '</div>';
                });
                (map.__files || []).sort().forEach(function (f) {
                    var full = prefix + f;
                    html += '<div class="editor-tree-file" data-vibe-path="' + esc(full) + '">' +
                        '\ud83d\udcc4 ' + esc(f) + '</div>';
                });
                return html;
            }
            return renderFolder(root, '');
        }

        function vibeLoadTree() {
            var tree = document.getElementById('editor-tree');
            if (!tree || !vibeProjectId) return;
            tree.innerHTML = '<div class="editor-tree-empty">Loading workspace\u2026</div>';
            authFetch('/api/vibe/projects/' + encodeURIComponent(vibeProjectId) + '/files')
                .then(function (r) { return r.json(); })
                .then(function (data) {
                    var files = (data && data.files) || [];
                    if (!files.length) {
                        tree.innerHTML = '<div class="editor-tree-empty">Workspace is empty \u2014 run the agent or create files.</div>';
                        return;
                    }
                    tree.innerHTML = vibeTreeHtml(files);
                    tree.querySelectorAll('[data-vibe-folder]').forEach(function (folder) {
                        folder.addEventListener('click', function () {
                            var children = folder.nextElementSibling;
                            if (!children) return;
                            var open = children.style.display !== 'none';
                            children.style.display = open ? 'none' : 'block';
                            var tw = folder.querySelector('.tw');
                            if (tw) tw.textContent = open ? '\u25b8' : '\u25be';
                        });
                    });
                    tree.querySelectorAll('[data-vibe-path]').forEach(function (node) {
                        node.addEventListener('click', function () {
                            vibeOpenFile(node.getAttribute('data-vibe-path'));
                        });
                    });
                    // Auto-open the entry point so the editor is never blank.
                    var first = files.indexOf('index.html') !== -1 ? 'index.html'
                        : (files.find(function (f) { return /index\.js$/.test(f); }) || files[0]);
                    vibeOpenFile(first);
                })
                .catch(function () {
                    tree.innerHTML = '<div class="editor-tree-empty">Failed to load workspace files.</div>';
                });
        }

        function vibeOpenFile(path) {
            if (!vibeProjectId || !path) return;
            authFetch('/api/vibe/projects/' + encodeURIComponent(vibeProjectId) + '/files/content?path=' + encodeURIComponent(path))
                .then(function (r) { return r.json(); })
                .then(function (data) {
                    if (data && typeof data.content === 'string') {
                        vibeSetEditorValue(data.content);
                        vibeSetActiveFile(path);
                        if (path.toLowerCase().endsWith('.csv')) switchToCsvMode(data.content);
                    } else {
                        vibeSetActiveFile(path);
                        vibeSetEditorValue('');
                    }
                })
                .catch(function (err) {
                    console.warn('workspace file load failed:', err);
                });
        }

        // Save into the project workspace (POST /files {path, content}) when a
        // project context is active; otherwise fall back to the Drive save.
        var __driveSaveEditorFile = saveEditorFile;
        saveEditorFile = function () {
            if (vibeProjectId && vibeFilePath) {
                var spinner = document.getElementById('save-spinner');
                if (spinner) spinner.style.display = 'inline-block';
                var body = {
                    path: vibeFilePath,
                    content: window.__CSV_ACTIVE ? (function () {
                        // Rebuild CSV from the table, mirroring the Drive path.
                        var table = document.querySelector('#csv-editor table.csv-table');
                        if (!table) return vibeEditorContent();
                        var headers = Array.from(table.querySelectorAll('thead th')).slice(1).map(function (th) { return th.textContent; });
                        var lines = [headers.join(',')];
                        table.querySelectorAll('tbody tr').forEach(function (tr) {
                            var vals = Array.from(tr.querySelectorAll('.csv-input')).map(function (inp) {
                                return '"' + inp.value.replace(/"/g, '""') + '"';
                            });
                            lines.push(vals.join(','));
                        });
                        return lines.join('\n');
                    })() : vibeEditorContent(),
                };
                authFetch('/api/vibe/projects/' + encodeURIComponent(vibeProjectId) + '/files', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body),
                }).then(function (r) {
                    if (spinner) spinner.style.display = 'none';
                    showSaveNotification({ detail: { successful: r.ok } });
                }).catch(function () {
                    if (spinner) spinner.style.display = 'none';
                    showSaveNotification({ detail: { successful: false } });
                });
                return;
            }
            __driveSaveEditorFile();
        };

        // Boot the explorer once the app params arrive (deep links set
        // __gbAppParams__ just before the body injection; poll briefly to
        // absorb late shell bootstrap).
        (function vibeExplorerBoot(tries) {
            var pid = vibeProjectContext();
            if (!pid) {
                if (tries < 10) {
                    setTimeout(function () { vibeExplorerBoot(tries + 1); }, 250);
                }
                return;
            }
            vibeProjectId = String(pid);
            var sidebar = document.getElementById('editor-sidebar');
            if (sidebar) sidebar.style.display = 'flex';
            var rootLabel = document.getElementById('editor-tree-root');
            authFetch('/api/vibe/projects/' + encodeURIComponent(vibeProjectId))
                .then(function (r) { return r.json(); })
                .then(function (data) {
                    var name = data && data.project && (data.project.name || data.project.project_id);
                    if (rootLabel && name) rootLabel.textContent = String(name);
                })
                .catch(function () { /* label stays generic */ });
            var refresh = document.getElementById('editor-tree-refresh');
            if (refresh && !refresh.dataset.wired) {
                refresh.dataset.wired = '1';
                refresh.addEventListener('click', vibeLoadTree);
            }
            vibeLoadTree();
        })(0);

        // Fallback: if Monaco doesn't load in 3s, show textarea
        var monacoFallbackTimer = setTimeout(function() {
            var ta = document.getElementById('text-editor');
            var mc = document.getElementById('monaco-editor');
            if (ta && mc && !window.monacoEditorInstance) {
                ta.style.display = 'block';
                mc.style.display = 'none';
            }
        }, 3000);

        let monacoEditorInstance = null;

        // Load Monaco Editor
        if (!document.getElementById('monaco-script')) {
            const script = document.createElement('script');
            script.id = 'monaco-script';
            script.src = '/suite/js/vendor/vs/loader.js';
            script.onload = () => {
                require.config({ paths: { 'vs': '/suite/js/vendor/vs' }});
                require(['vs/editor/editor.main'], function() {
                    clearTimeout(monacoFallbackTimer);
                    initMonaco();
                });
            };
            script.onerror = function() {
                clearTimeout(monacoFallbackTimer);
                var ta = document.getElementById('text-editor');
                var mc = document.getElementById('monaco-editor');
                if (ta && mc) { ta.style.display = 'block'; mc.style.display = 'none'; }
            };
            document.head.appendChild(script);
        } else if (window.monaco) {
            clearTimeout(monacoFallbackTimer);
            initMonaco();
        }
