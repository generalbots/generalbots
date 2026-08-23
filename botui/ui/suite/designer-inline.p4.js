
        function showSearchPanel() {
            let panel = document.getElementById('search-panel');
            if (!panel) {
                panel = document.createElement('div');
                panel.id = 'search-panel';
                panel.className = 'magic-panel visible';
                panel.style.right = '300px';
                panel.innerHTML = '<div class="magic-header"><span>Search Nodes</span><button class="magic-close" onclick="hideSearchPanel()">&times;</button></div>'
                    + '<div style="padding:12px"><input type="text" id="node-search-input" placeholder="Search by type or content..." oninput="searchNodes(this.value)" style="width:100%;padding:8px;border:1px solid var(--border);border-radius:4px;background:var(--surface);color:var(--text)"></div>'
                    + '<div id="search-results" style="padding:0 12px 12px;max-height:300px;overflow-y:auto"></div>';
                document.body.appendChild(panel);
            } else {
                panel.classList.add('visible');
            }
            document.getElementById('node-search-input').focus();
        }

        function hideSearchPanel() {
            var panel = document.getElementById('search-panel');
            if (panel) panel.classList.remove('visible');
        }

        function searchNodes(query) {
            var results = document.getElementById('search-results');
            if (!results) return;
            if (!query.trim()) { results.innerHTML = ''; return; }
            var q = query.toLowerCase();
            var found = [];
            state.nodes.forEach(function(node) {
                var template = nodeTemplates[node.type];
                var label = node.type;
                var fieldText = Object.values(node.fields).join(' ').toLowerCase();
                if (label.toLowerCase().includes(q) || fieldText.includes(q)) {
                    found.push(node);
                }
            });
            if (found.length === 0) {
                results.innerHTML = '<div style="text-align:center;color:var(--text-secondary);padding:16px">No nodes found</div>';
                return;
            }
            results.innerHTML = found.map(function(node) {
                return '<div style="padding:8px;border:1px solid var(--border);border-radius:4px;margin-bottom:6px;cursor:pointer;display:flex;justify-content:space-between;align-items:center" onclick="selectAndPan(\'' + node.id + '\')">'
                    + '<span><strong>' + node.type + '</strong> - ' + Object.values(node.fields).join(', ').substring(0, 60) + '</span>'
                    + '<span style="color:var(--text-secondary);font-size:11px">(' + Math.round(node.x) + ',' + Math.round(node.y) + ')</span>'
                    + '</div>';
            }).join('');
        }

        function selectAndPan(nodeId) {
            selectNode(nodeId);
            var node = state.nodes.get(nodeId);
            if (node) {
                state.pan.x = -node.x * state.zoom + 400;
                state.pan.y = -node.y * state.zoom + 300;
                updateCanvasTransform();
            }
        }

        async function deployFlow() {
            var filename = document.getElementById('current-filename').value;
            if (!filename && !state.driveSource) {
                alert('Save the flow first before deploying.');
                return;
            }
            var basCode = generateBasCode();
            var path = state.driveSource ? state.driveSource.path : filename;
            var bucket = state.driveSource ? state.driveSource.bucket : null;

            if (!bucket) {
                bucket = prompt('Enter bot bucket name (e.g., mybot.gbai):');
                if (!bucket) return;
            }
            if (!path) path = 'dialog.bas';

            try {
                var resp = await fetch('/api/files/write', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ bucket: bucket, path: path, content: basCode })
                });
                if (resp.ok) {
                    alert('Flow deployed to ' + bucket + '/' + path + ' successfully!');
                } else {
                    var err = await resp.json();
                    alert('Deploy failed: ' + (err.error || 'Unknown error'));
                }
            } catch (e) {
                alert('Deploy failed: ' + e.message);
            }
        }

        function showVersionsPanel() {
            var panel = document.getElementById('versions-panel');
            if (!panel) {
                panel = document.createElement('div');
                panel.id = 'versions-panel';
                panel.className = 'magic-panel visible';
                panel.style.right = '300px';
                panel.innerHTML = '<div class="magic-header"><span>Version History</span><button class="magic-close" onclick="hideVersionsPanel()">&times;</button></div>'
                    + '<div id="versions-content" style="padding:12px;max-height:400px;overflow-y:auto"></div>';
                document.body.appendChild(panel);
            } else {
                panel.classList.add('visible');
            }
            loadVersions();
        }

        function hideVersionsPanel() {
            var panel = document.getElementById('versions-panel');
            if (panel) panel.classList.remove('visible');
        }

        async function loadVersions() {
            var content = document.getElementById('versions-content');
            if (!content) return;
            content.innerHTML = '<div style="text-align:center;color:var(--text-secondary)">Loading versions...</div>';
            var filename = document.getElementById('current-filename').value;
            if (!filename && !state.driveSource) {
                content.innerHTML = '<div style="text-align:center;color:var(--text-secondary);padding:20px">No file saved yet. Save your flow first.</div>';
                return;
            }
            var path = state.driveSource ? state.driveSource.path : filename;
            var bucket = state.driveSource ? state.driveSource.bucket : 'default';
            try {
                var resp = await fetch('/api/files/versions?bucket=' + encodeURIComponent(bucket) + '&path=' + encodeURIComponent(path));
                if (resp.ok) {
                    var versions = await resp.json();
                    if (!Array.isArray(versions) || versions.length === 0) {
                        content.innerHTML = '<div style="text-align:center;color:var(--text-secondary);padding:20px">No version history available.</div>';
                        return;
                    }
                    content.innerHTML = versions.map(function(v, i) {
                        return '<div style="padding:8px;border:1px solid var(--border);border-radius:4px;margin-bottom:6px;cursor:pointer;display:flex;justify-content:space-between" onclick="restoreVersion(\'' + (v.id || i) + '\')">'
                            + '<span>' + (v.label || 'Version ' + (versions.length - i)) + '</span>'
                            + '<span style="color:var(--text-secondary);font-size:11px">' + (v.date || '') + '</span>'
                            + '</div>';
                    }).join('');
                } else {
                    content.innerHTML = '<div style="text-align:center;color:var(--text-secondary);padding:20px">Could not load versions.</div>';
                }
            } catch (e) {
                content.innerHTML = '<div style="text-align:center;color:var(--text-secondary);padding:20px">Error loading versions.</div>';
            }
        }

        async function restoreVersion(versionId) {
            if (!confirm('Restore this version? Current unsaved changes will be lost.')) return;
            var filename = document.getElementById('current-filename').value;
            var path = state.driveSource ? state.driveSource.path : filename;
            var bucket = state.driveSource ? state.driveSource.bucket : 'default';
            try {
                var resp = await fetch('/api/files/read', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ bucket: bucket, path: path, version: versionId })
                });
                if (resp.ok) {
                    var data = await resp.json();
                    if (data.content) {
                        document.getElementById('canvas-inner').innerHTML = '';
                        state.nodes.clear();
                        state.connections = [];
                        state.selectedNode = null;
                        state.history = [];
                        state.historyIndex = -1;
                        state.nextNodeId = 1;
                        parseBasicCodeToNodes(data.content);
                        hideVersionsPanel();
                    }
                }
            } catch (e) {
                alert('Error restoring version: ' + e.message);
            }
        }
