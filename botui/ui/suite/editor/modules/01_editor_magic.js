/* Editor app — module 1/3: save notification, magic panel and shared escapeHtml (split from editor-inline-panels.js for #1731) */
// Minimal JS for notification display (could be replaced with htmx extension)

    // #2093 — translated text with an authored English fallback so a missing
    // catalogue entry never leaks a raw key into the editor UI.
    function t(key, fallback, params) {
        return window.gbT ? window.gbT(key, fallback, params) : fallback;
    }
            function showSaveNotification(event) {
                const notification = document.getElementById('notification');
                if (event.detail.successful) {
                    notification.textContent = '✓ ' + t('editor-saved', 'File saved');
                    notification.className = 'notification success show';
                    document.getElementById('dirty-indicator').style.display = 'none';
                } else {
                    notification.textContent = '✗ ' + t('editor-save-failed', 'Save failed');
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
                    content.innerHTML = '<p style="color:var(--error);padding:20px;">' + t('editor-magic-failed', 'Failed to analyze. Try again.') + '</p>';
                }
            } catch (e) {
                content.innerHTML = '<p style="color:var(--error);padding:20px;">' + t('editor-magic-connection-error', 'Error connecting to AI service.') + '</p>';
            }
        }

        function renderMagicResult(result) {
            const content = document.getElementById('magic-content');
            if (result.improved_code) {
                content.innerHTML = `
                    <div class="magic-result">
                        <p><strong>${t('editor-magic-suggested-improvements', 'Suggested improvements:')}</strong></p>
                        <p style="color:var(--text-secondary, #94a3b8);margin:8px 0;">${result.explanation || t('editor-magic-default-explanation', 'Improved code structure and patterns.')}</p>
                        <pre>${escapeHtml(result.improved_code)}</pre>
                        <button class="magic-apply-btn" onclick="applyMagicCode()">${t('editor-magic-apply', 'Apply Changes')}</button>
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
                content.innerHTML = '<p style="padding:20px;">' + t('editor-magic-no-suggestions', 'Your code looks good! No suggestions at this time.') + '</p>';
            }
        }

        function applyMagicCode() {
            if (window.magicImprovedCode) {
                document.getElementById('text-editor').value = window.magicImprovedCode;
                hideMagicPanel();
            }
        }

        function escapeHtml(value) {
          if (window.GBShared && window.GBShared.html) return window.GBShared.html.escape(value);
          return String(value == null ? "" : value)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#39;");
        }

        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey && e.key === 'm') {
                e.preventDefault();
                showMagicPanel();
            }
        });
