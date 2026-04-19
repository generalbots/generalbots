const botCoderEditor = {
    instances: {},
    currentFile: null,
    tabs: [],
    models: {},
    editor: null,

    init: function() {
        if (!window.require) {
            console.error('Monaco loader not found.');
            return;
        }

        require.config({ paths: { vs: '/suite/js/vendor/vs' } });
        require(['vs/editor/editor.main'], () => {
            this.editor = monaco.editor.create(document.getElementById('monacoEditorContainer'), {
                value: '// Welcome to BotCoder Multi-Agent IDE\n// Select a file from the explorer to begin viewing or editing.',
                language: 'javascript',
                theme: 'vs-dark',
                automaticLayout: true,
                minimap: { enabled: true },
                fontSize: 14,
                fontFamily: 'Consolas, "Courier New", monospace'
            });

            this.setupKeyboardShortcuts();
            this.refreshFiles();

            // Auto-save debouncer could be added here on content change
            this.editor.onDidChangeModelContent(() => {
                const statusSpan = document.getElementById('botcoderFileStatus');
                if (statusSpan && this.currentFile) {
                    statusSpan.textContent = '• Unsaved';
                    this.markTabDirty(this.currentFile, true);
                }
            });
        });
    },

    setupKeyboardShortcuts: function() {
        // Ctrl+S to save
        this.editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
            this.saveCurrentFile();
        });

        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey && e.key === 's') {
                e.preventDefault();
                this.saveCurrentFile();
            }
        });
    },

    getLanguageFromExt: function(fileName) {
        if (fileName.endsWith('.js')) return 'javascript';
        if (fileName.endsWith('.html')) return 'html';
        if (fileName.endsWith('.css')) return 'css';
        if (fileName.endsWith('.json')) return 'json';
        if (fileName.endsWith('.bas')) return 'basic'; // Custom type maybe
        if (fileName.endsWith('.rs')) return 'rust';
        return 'plaintext';
    },

    openFile: async function(filePath) {
        try {
            document.getElementById('botcoderCurrentFile').textContent = 'Loading...';
            
            // Check if already open
            if (this.models[filePath]) {
                this.switchToTab(filePath);
                return;
            }

            const response = await fetch(`/api/editor/file/${encodeURIComponent(filePath)}`);
            if (!response.ok) throw new Error('File not found');

            const result = await response.json();
            const content = result.content || '';
            
            const language = this.getLanguageFromExt(filePath);
            const model = monaco.editor.createModel(content, language, monaco.Uri.file(filePath));
            this.models[filePath] = { model, dirty: false };
            
            this.addTab(filePath);
            this.switchToTab(filePath);

        } catch (err) {
            console.error('Error opening file:', err);
            alert('Failed to open file: ' + err.message);
        }
    },

    addTab: function(filePath) {
        if (!this.tabs.includes(filePath)) {
            this.tabs.push(filePath);
            this.renderTabs();
        }
    },

    switchToTab: function(filePath) {
        if (!this.models[filePath]) return;
        
        this.currentFile = filePath;
        this.editor.setModel(this.models[filePath].model);
        
        const fileName = filePath.split('/').pop() || filePath;
        document.getElementById('botcoderCurrentFile').textContent = fileName;
        
        const isDirty = this.models[filePath].dirty;
        document.getElementById('botcoderFileStatus').textContent = isDirty ? '• Unsaved' : '';
        
        this.renderTabs();
    },

    closeTab: function(filePath, e) {
        if (e) e.stopPropagation();
        
        if (this.models[filePath] && this.models[filePath].dirty) {
            if (!confirm(`Save changes to ${filePath} before closing?`)) {
                return;
            }
        }
        
        if (this.models[filePath]) {
            this.models[filePath].model.dispose();
            delete this.models[filePath];
        }

        this.tabs = this.tabs.filter(t => t !== filePath);
        
        if (this.currentFile === filePath) {
            this.currentFile = this.tabs.length > 0 ? this.tabs[this.tabs.length - 1] : null;
            if (this.currentFile) {
                this.switchToTab(this.currentFile);
            } else {
                this.editor.setValue('// Select a file to edit');
                document.getElementById('botcoderCurrentFile').textContent = 'No file open';
                document.getElementById('botcoderFileStatus').textContent = '';
            }
        }
        
        this.renderTabs();
    },

    renderTabs: function() {
        const tabsContainer = document.getElementById('botcoderTabs');
        if (!tabsContainer) return;

        tabsContainer.innerHTML = this.tabs.map(tab => {
            const fileName = tab.split('/').pop();
            const isActive = tab === this.currentFile;
            const isDirty = this.models[tab]?.dirty;
            
            return `
                <div class="botcoder-tab ${isActive ? 'active' : ''}" onclick="botCoderEditor.switchToTab('${tab}')">
                    <span class="botcoder-tab-name">${fileName}</span>
                    <span class="botcoder-tab-indicator" style="display: ${isDirty ? 'inline-block' : 'none'}">•</span>
                    <span class="botcoder-tab-close" onclick="botCoderEditor.closeTab('${tab}', event)">×</span>
                </div>
            `;
        }).join('');
    },

    markTabDirty: function(filePath, dirty) {
        if (this.models[filePath]) {
            this.models[filePath].dirty = dirty;
            this.renderTabs();
        }
    },

    saveCurrentFile: async function() {
        if (!this.currentFile || !this.models[this.currentFile]) return;
        
        const content = this.models[this.currentFile].model.getValue();
        document.getElementById('botcoderFileStatus').textContent = 'Saving...';
        
        try {
            const response = await fetch(`/api/editor/file/${encodeURIComponent(this.currentFile)}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ content })
            });
            
            if (!response.ok) throw new Error('Save failed');
            
            this.markTabDirty(this.currentFile, false);
            document.getElementById('botcoderFileStatus').textContent = 'Saved';
            setTimeout(() => {
                if (document.getElementById('botcoderFileStatus').textContent === 'Saved') {
                    document.getElementById('botcoderFileStatus').textContent = '';
                }
            }, 2000);
            
            // Sync via websocket theoretically
            this.syncFile(this.currentFile, content);
            
        } catch (err) {
            console.error('Save error:', err);
            document.getElementById('botcoderFileStatus').textContent = 'Error saving';
        }
    },

    syncFile: function(filePath, content) {
        // Implement WebSocket auto-sync logic here
        // if (ws && ws.readyState === WebSocket.OPEN) { ... }
    },

    publishFile: function() {
        if (confirm('Commit and publish your changes?')) {
            alert('File published successfully! (Mock implementation)');
        }
    },

    refreshFiles: async function() {
        try {
            const response = await fetch('/api/editor/files');
            if (response.ok) {
                const data = await response.json();
                this.renderFileTree(data.files || []);
            } else {
                this.mockFileTree(); // fallback to mock
            }
        } catch(e) {
            this.mockFileTree();
        }
    },

    mockFileTree: function() {
        const mockFiles = [
            'src/main.rs',
            'ui/index.html',
            'ui/style.css',
            'bot/logic.bas',
            'package.json'
        ];
        this.renderFileTree(mockFiles);
    },

    renderFileTree: function(files) {
        const container = document.getElementById('botcoderFileTree');
        if (!container) return;

        if (files.length === 0) {
            container.innerHTML = '<div class="botcoder-empty">No files in workspace</div>';
            return;
        }

        container.innerHTML = files.map(f => {
            const isFolder = f.endsWith('/');
            const icon = isFolder ? '📁' : '📄';
            return `
                <div class="botcoder-file-item" onclick="botCoderEditor.openFile('${f}')">
                    <span class="file-icon">${icon}</span>
                    <span class="file-name">${f.replace(/\/$/, '').split('/').pop()}</span>
                </div>
            `;
        }).join('');
    }
};

botCoderEditor.init();
