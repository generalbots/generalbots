/* Editor app — module 3/3: file bootstrap, Vibe workspace explorer and Monaco loader (split from editor-inline-panels.js for #1731) */

        // #2282 — boot scoped to this window so two Drive-opened editors keep
        // independent file contexts (page globals clobbered by the next open).
        var __editorBootRoot = (document.currentScript && document.currentScript.closest('[id^="window-body-"]')) || null;
        if (!__editorBootRoot) {
            var __ebs = document.querySelectorAll('[id^="window-body-editor"]');
            if (__ebs.length === 1) __editorBootRoot = __ebs[__ebs.length - 1];
        }
        function __editorBootCtx() {
            if (typeof gbEditorFileCtx === 'function') return gbEditorFileCtx(__editorBootRoot || document);
            var ds = (__editorBootRoot && __editorBootRoot.dataset) || {};
            var params = new URLSearchParams(window.location.search);
            return {
                bucket: ds.gbFileBucket || window.__EDITOR_FILE_BUCKET || params.get('bucket') || '',
                path: ds.gbFilePath || window.__EDITOR_FILE_PATH || params.get('file') || '',
                scope: ds.gbFileScope || window.__EDITOR_FILE_SCOPE || 'bot',
            };
        }
        if (__editorBootRoot) __editorBootRoot.__monacoEditorInstance = null;

        (function() {
            var ctx = __editorBootCtx();
            var filePath = ctx.path || '';
            if (filePath) {
                var fileName = filePath.split('/').pop();
                var root = __editorBootRoot || document;
                var fnEl = root.querySelector ? root.querySelector('#editor-filename') : document.getElementById('editor-filename');
                var fpEl = root.querySelector ? root.querySelector('#editor-filepath') : document.getElementById('editor-filepath');
                if (fnEl) fnEl.textContent = fileName;
                if (fpEl) fpEl.textContent = filePath;
            }
        })();

        var editorBootPromise = window.__EDITOR_BOOT || Promise.resolve();
        editorBootPromise.then(function() {
            var ctx = __editorBootCtx();
            var filePath = ctx.path || '';
            var bucket = ctx.bucket || '';
            var scope = ctx.scope || 'bot';
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
                            switchToCsvMode(decoded, __editorBootRoot);
                        } else {
                            var root = __editorBootRoot || document;
                            var ta = root.querySelector ? root.querySelector('#text-editor') : document.getElementById('text-editor');
                            if (ta) {
                                ta.value = decoded;
                                var mi = (__editorBootRoot && __editorBootRoot.__monacoEditorInstance) || window.monacoEditorInstance;
                                if (mi) mi.setValue(decoded);
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
                    // Open the requested file when the caller named one (an
                    // AutoTask row's "edit source" action passes ?path=),
                    // otherwise the entry point so the editor is never blank.
                    var requested = window.__EDITOR_VIBE_PATH
                        || new URLSearchParams(window.location.search).get('path')
                        || '';
                    var first = requested && files.indexOf(requested) !== -1 ? requested
                        : (files.indexOf('index.html') !== -1 ? 'index.html'
                        : (files.find(function (f) { return /index\.js$/.test(f); }) || files[0]));
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
            // #1408 — when a Vibe agent run finishes, reload the workspace
            // tree so files written by the run appear in the open Editor.
            document.addEventListener('gb:vibe-app-refresh', function () {
                if (typeof vibeLoadTree === 'function') vibeLoadTree();
            });
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
