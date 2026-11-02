
        function initMonaco() {
            var fileName = '';
            var fnEl = document.getElementById('editor-filename');
            if (fnEl) fileName = fnEl.textContent || fnEl.innerText || '';
            // Skip Monaco for CSV files — use CSV table editor instead
            if (fileName.toLowerCase().endsWith('.csv') || window.__CSV_ACTIVE) return;
            const container = document.getElementById('monaco-editor');
            if (!container) return;
            
            const textarea = document.getElementById('text-editor');
            if (textarea) textarea.style.display = 'none';
            container.style.display = 'block';
            
            const filenameElement = document.getElementById('editor-filename');
            const filepath = filenameElement ? (filenameElement.innerText || filenameElement.textContent).trim() : '';
            let language = 'plaintext';
            if (filepath.endsWith('.js')) language = 'javascript';
            else if (filepath.endsWith('.html')) language = 'html';
            else if (filepath.endsWith('.css')) language = 'css';
            else if (filepath.endsWith('.rs')) language = 'rust';
            else if (filepath.endsWith('.json')) language = 'json';
            else if (filepath.endsWith('.md')) language = 'markdown';
            
            document.getElementById('file-type').textContent = '📄 ' + language.toUpperCase();
            
            window.__MONACO_CREATE_STARTED = true;
            window.monacoEditorInstance = monaco.editor.create(container, {
                value: textarea.value,
                language: language,
                theme: 'vs-dark',
                automaticLayout: true,
                minimap: { enabled: false },
                fontSize: 13,
                fontFamily: "'JetBrains Mono','Fira Code','Cascadia Code','DejaVu Sans Mono','Ubuntu Mono',monospace",
                scrollBeyondLastLine: false
            });

            window.monacoEditorInstance.onDidChangeModelContent(() => {
                textarea.value = window.monacoEditorInstance.getValue();
                document.getElementById('dirty-indicator').style.display = 'inline-block';

                const pos = window.monacoEditorInstance.getPosition();
                const statusPos = document.getElementById('cursor-position');
                if (statusPos) {
                    statusPos.textContent = `Ln ${pos.lineNumber}, Col ${pos.column}`;
                }
            });

            window.monacoEditorInstance.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
                htmx.trigger(document.querySelector('[hx-post="/api/editor/save"]'), 'click');
            });
            
            window.__MONACO_INIT_COMPLETE = true;
            // Apply magic code hook using Monaco
            window.applyMagicCode = function() {
                if (window.magicImprovedCode && window.monacoEditorInstance) {
                    window.monacoEditorInstance.setValue(window.magicImprovedCode);
                    hideMagicPanel();
                }
            };
        }
