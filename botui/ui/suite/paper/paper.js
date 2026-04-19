/**
 * Paper Module JavaScript
 * AI Writing & Notes Application
 */
(function() {
    'use strict';

    // DOM Elements
    const elements = {
        editor: document.getElementById('editor-content'),
        title: document.getElementById('paper-title'),
        slashMenu: document.getElementById('slash-menu'),
        aiPanel: document.getElementById('ai-panel'),
        sidebar: document.getElementById('paper-sidebar'),
        wordCount: document.getElementById('word-count'),
        charCount: document.getElementById('char-count'),
        saveStatus: document.getElementById('save-status')
    };

    // State
    let slashPosition = null;
    let autoSaveTimer = null;

    /**
     * Initialize the Paper module
     */
    function init() {
        if (!elements.editor) return;

        setupEditorEvents();
        setupToolbarCommands();
        setupSlashMenu();
        setupAIPanel();
        setupSidebar();
        setupModals();
        setupKeyboardShortcuts();
        updateWordCount();
    }

    /**
     * Setup editor input and keydown events
     */
    function setupEditorEvents() {
        elements.editor.addEventListener('input', function() {
            updateWordCount();
            scheduleAutoSave();
            checkSlashCommand();
        });

        elements.editor.addEventListener('keydown', function(e) {
            // Handle slash menu navigation
            if (elements.slashMenu && !elements.slashMenu.classList.contains('hidden')) {
                if (e.key === 'Escape') {
                    hideSlashMenu();
                    e.preventDefault();
                } else if (e.key === 'Enter') {
                    const selected = elements.slashMenu.querySelector('.slash-item.selected') ||
                                     elements.slashMenu.querySelector('.slash-item');
                    if (selected) {
                        executeSlashCommand(selected.dataset.cmd);
                        e.preventDefault();
                    }
                } else if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
                    navigateSlashMenu(e.key === 'ArrowDown' ? 1 : -1);
                    e.preventDefault();
                }
            }
        });
    }

    /**
     * Check for slash command trigger
     */
    function checkSlashCommand() {
        const selection = window.getSelection();
        if (!selection.rangeCount) return;

        const range = selection.getRangeAt(0);
        const text = range.startContainer.textContent || '';
        const cursorPos = range.startOffset;

        // Check for slash command
        if (text[cursorPos - 1] === '/') {
            showSlashMenu(range);
        } else if (elements.slashMenu && !elements.slashMenu.classList.contains('hidden')) {
            // Filter slash menu based on input after /
            const slashIndex = text.lastIndexOf('/');
            if (slashIndex >= 0 && cursorPos > slashIndex) {
                const filter = text.substring(slashIndex + 1, cursorPos).toLowerCase();
                filterSlashMenu(filter);
            }
        }
    }

    /**
     * Setup keyboard shortcuts
     */
    function setupKeyboardShortcuts() {
        document.addEventListener('keydown', function(e) {
            if (e.ctrlKey || e.metaKey) {
                switch(e.key.toLowerCase()) {
                    case 'b':
                        e.preventDefault();
                        document.execCommand('bold');
                        break;
                    case 'i':
                        e.preventDefault();
                        document.execCommand('italic');
                        break;
                    case 'u':
                        e.preventDefault();
                        document.execCommand('underline');
                        break;
                    case 's':
                        e.preventDefault();
                        saveDocument();
                        break;
                    case 'k':
                        e.preventDefault();
                        insertLink();
                        break;
                    case '/':
                        e.preventDefault();
                        toggleFocusMode();
                        break;
                }
            }

            // Escape to close panels
            if (e.key === 'Escape') {
                hideSlashMenu();
                elements.aiPanel?.classList.add('hidden');
                closeModals();
            }
        });
    }

    /**
     * Setup toolbar command buttons
     */
    function setupToolbarCommands() {
        document.querySelectorAll('[data-cmd]').forEach(btn => {
            btn.addEventListener('click', function() {
                const cmd = this.dataset.cmd;
                executeToolbarCommand(cmd);
                elements.editor?.focus();
            });
        });

        // Heading select
        const headingSelect = document.getElementById('heading-select');
        if (headingSelect) {
            headingSelect.addEventListener('change', function() {
                const value = this.value;
                document.execCommand('formatBlock', false, value === 'p' ? 'p' : value);
                elements.editor?.focus();
            });
        }

        // Text color picker
        const colorPicker = document.getElementById('text-color');
        if (colorPicker) {
            colorPicker.addEventListener('change', function() {
                document.execCommand('foreColor', false, this.value);
                elements.editor?.focus();
            });
        }
    }

    /**
     * Execute toolbar command
     */
    function executeToolbarCommand(cmd) {
        switch(cmd) {
            case 'bold':
                document.execCommand('bold');
                break;
            case 'italic':
                document.execCommand('italic');
                break;
            case 'underline':
                document.execCommand('underline');
                break;
            case 'strikethrough':
                document.execCommand('strikeThrough');
                break;
            case 'highlight':
                document.execCommand('hiliteColor', false, '#ffff00');
                break;
            case 'alignLeft':
                document.execCommand('justifyLeft');
                break;
            case 'alignCenter':
                document.execCommand('justifyCenter');
                break;
            case 'alignRight':
                document.execCommand('justifyRight');
                break;
            case 'bulletList':
                document.execCommand('insertUnorderedList');
                break;
            case 'numberedList':
                document.execCommand('insertOrderedList');
                break;
            case 'todoList':
                insertTodo();
                break;
            case 'link':
                insertLink();
                break;
            case 'image':
                insertImage();
                break;
            case 'table':
                insertTable();
                break;
            case 'code':
                document.execCommand('formatBlock', false, 'pre');
                break;
            case 'quote':
                document.execCommand('formatBlock', false, 'blockquote');
                break;
            case 'undo':
                document.execCommand('undo');
                break;
            case 'redo':
                document.execCommand('redo');
                break;
        }
    }

    /**
     * Show slash command menu
     */
    function showSlashMenu(range) {
        if (!elements.slashMenu || !elements.editor) return;

        const rect = range.getBoundingClientRect();
        const editorRect = elements.editor.getBoundingClientRect();

        elements.slashMenu.style.top = (rect.bottom - editorRect.top + elements.editor.scrollTop + 8) + 'px';
        elements.slashMenu.style.left = (rect.left - editorRect.left) + 'px';
        elements.slashMenu.classList.remove('hidden');
        slashPosition = range.startOffset;

        // Reset filter
        filterSlashMenu('');
    }

    /**
     * Hide slash command menu
     */
    function hideSlashMenu() {
        if (elements.slashMenu) {
            elements.slashMenu.classList.add('hidden');
        }
        slashPosition = null;
    }

    /**
     * Filter slash menu items
     */
    function filterSlashMenu(filter) {
        if (!elements.slashMenu) return;

        const items = elements.slashMenu.querySelectorAll('.slash-item');
        let firstVisible = null;

        items.forEach(item => {
            const label = item.querySelector('.slash-label')?.textContent.toLowerCase() || '';
            const matches = label.includes(filter);
            item.style.display = matches ? 'flex' : 'none';
            if (matches && !firstVisible) firstVisible = item;
        });

        // Select first visible
        items.forEach(item => item.classList.remove('selected'));
        if (firstVisible) firstVisible.classList.add('selected');
    }

    /**
     * Navigate slash menu with arrow keys
     */
    function navigateSlashMenu(direction) {
        if (!elements.slashMenu) return;

        const items = Array.from(elements.slashMenu.querySelectorAll('.slash-item'))
                          .filter(i => i.style.display !== 'none');
        const current = items.findIndex(i => i.classList.contains('selected'));

        items.forEach(i => i.classList.remove('selected'));

        let next = current + direction;
        if (next < 0) next = items.length - 1;
        if (next >= items.length) next = 0;

        items[next]?.classList.add('selected');
        items[next]?.scrollIntoView({ block: 'nearest' });
    }

    /**
     * Execute slash command
     */
    function executeSlashCommand(cmd) {
        hideSlashMenu();

        // Remove the slash character and any filter text
        const selection = window.getSelection();
        if (selection.rangeCount) {
            const range = selection.getRangeAt(0);
            const text = range.startContainer.textContent || '';
            const slashIndex = text.lastIndexOf('/');

            if (slashIndex >= 0) {
                range.startContainer.textContent = text.substring(0, slashIndex) + text.substring(range.startOffset);
                range.setStart(range.startContainer, slashIndex);
                range.collapse(true);
                selection.removeAllRanges();
                selection.addRange(range);
            }
        }

        // Execute command
        switch(cmd) {
            case 'h1':
                document.execCommand('formatBlock', false, 'h1');
                break;
            case 'h2':
                document.execCommand('formatBlock', false, 'h2');
                break;
            case 'h3':
                document.execCommand('formatBlock', false, 'h3');
                break;
            case 'bullet':
                document.execCommand('insertUnorderedList');
                break;
            case 'number':
                document.execCommand('insertOrderedList');
                break;
            case 'todo':
                insertTodo();
                break;
            case 'quote':
                document.execCommand('formatBlock', false, 'blockquote');
                break;
            case 'code':
                document.execCommand('formatBlock', false, 'pre');
                break;
            case 'divider':
                document.execCommand('insertHTML', false, '<hr>');
                break;
            case 'callout':
                document.execCommand('insertHTML', false, '<div class="callout">💡 </div>');
                break;
            case 'table':
                insertTable();
                break;
            case 'image':
                insertImage();
                break;
            case 'ai-write':
            case 'ai-summarize':
            case 'ai-expand':
            case 'ai-improve':
            case 'ai-translate':
            case 'ai-extract':
                openAIPanel(cmd);
                break;
        }
    }

    /**
     * Setup slash menu click handlers
     */
    function setupSlashMenu() {
        if (!elements.slashMenu) return;

        elements.slashMenu.querySelectorAll('.slash-item').forEach(item => {
            item.addEventListener('click', function() {
                executeSlashCommand(this.dataset.cmd);
            });
        });

        // Close on click outside
        document.addEventListener('click', function(e) {
            if (elements.slashMenu && 
                !elements.slashMenu.contains(e.target) && 
                !elements.editor?.contains(e.target)) {
                hideSlashMenu();
            }
        });
    }

    /**
     * Insert todo checkbox
     */
    function insertTodo() {
        const html = '<div class="todo-item"><input type="checkbox" class="todo-checkbox"><span></span></div>';
        document.execCommand('insertHTML', false, html);
    }

    /**
     * Insert table
     */
    function insertTable() {
        const html = `
            <table>
                <tr>
                    <td></td>
                    <td></td>
                    <td></td>
                </tr>
                <tr>
                    <td></td>
                    <td></td>
                    <td></td>
                </tr>
            </table>
        `;
        document.execCommand('insertHTML', false, html);
    }

    /**
     * Insert image
     */
    function insertImage() {
        const url = prompt('Enter image URL:');
        if (url) {
            document.execCommand('insertHTML', false, `<img src="${escapeHtml(url)}" alt="Image">`);
        }
    }

    /**
     * Insert link
     */
    function insertLink() {
        const url = prompt('Enter URL:');
        if (url) {
            document.execCommand('createLink', false, url);
        }
    }

    /**
     * Setup AI panel
     */
    function setupAIPanel() {
        // AI button
        const aiBtn = document.getElementById('ai-assist-btn');
        if (aiBtn) {
            aiBtn.addEventListener('click', function() {
                const selectedText = window.getSelection().toString();
                const input = document.getElementById('selected-text-input');
                if (input) input.value = selectedText;
                elements.aiPanel?.classList.toggle('hidden');
            });
        }

        // Close AI panel
        const closeBtn = document.getElementById('close-ai-panel');
        if (closeBtn) {
            closeBtn.addEventListener('click', function() {
                elements.aiPanel?.classList.add('hidden');
            });
        }

        // Tone buttons
        document.querySelectorAll('.tone-btn').forEach(btn => {
            btn.addEventListener('click', function() {
                document.querySelectorAll('.tone-btn').forEach(b => b.classList.remove('active'));
                this.classList.add('active');

                const tone = this.dataset.tone;
                const selectedText = document.getElementById('selected-text-input')?.value || '';

                if (typeof htmx !== 'undefined') {
                    htmx.ajax('POST', '/api/ui/paper/ai/tone', {
                        target: '#ai-response-content',
                        values: { tone, text: selectedText }
                    }).then(() => {
                        document.getElementById('ai-response')?.classList.remove('hidden');
                    });
                }
            });
        });

        // AI response actions
        const copyBtn = document.getElementById('copy-ai-response');
        if (copyBtn) {
            copyBtn.addEventListener('click', function() {
                const content = document.getElementById('ai-response-content')?.innerText || '';
                navigator.clipboard.writeText(content);
            });
        }

        const insertBtn = document.getElementById('insert-ai-response');
        if (insertBtn) {
            insertBtn.addEventListener('click', function() {
                const content = document.getElementById('ai-response-content')?.innerHTML || '';
                elements.editor?.focus();
                document.execCommand('insertHTML', false, content);
            });
        }

        const replaceBtn = document.getElementById('replace-ai-response');
        if (replaceBtn) {
            replaceBtn.addEventListener('click', function() {
                const content = document.getElementById('ai-response-content')?.innerHTML || '';
                document.execCommand('insertHTML', false, content);
            });
        }
    }

    /**
     * Open AI panel with specific action
     */
    function openAIPanel(action) {
        const selectedText = window.getSelection().toString();
        const input = document.getElementById('selected-text-input');
        if (input) input.value = selectedText;
        elements.aiPanel?.classList.remove('hidden');
    }

    /**
     * Setup sidebar toggle
     */
    function setupSidebar() {
        const toggleBtn = document.getElementById('toggle-sidebar');
        if (toggleBtn) {
            toggleBtn.addEventListener('click', function() {
                elements.sidebar?.classList.toggle('collapsed');
            });
        }
    }

    /**
     * Setup modal dialogs
     */
    function setupModals() {
        // Export button
        const exportBtn = document.getElementById('export-btn');
        if (exportBtn) {
            exportBtn.addEventListener('click', function() {
                document.getElementById('export-modal')?.classList.remove('hidden');
            });
        }

        // Close modal buttons
        document.querySelectorAll('.close-modal').forEach(btn => {
            btn.addEventListener('click', function() {
                this.closest('.modal')?.classList.add('hidden');
            });
        });

        // Click outside modal to close
        document.querySelectorAll('.modal').forEach(modal => {
            modal.addEventListener('click', function(e) {
                if (e.target === this) {
                    this.classList.add('hidden');
                }
            });
        });
    }

    /**
     * Close all modals
     */
    function closeModals() {
        document.querySelectorAll('.modal').forEach(modal => {
            modal.classList.add('hidden');
        });
    }

    /**
     * Update word and character count
     */
    function updateWordCount() {
        const text = elements.editor?.innerText || '';
        const words = text.trim().split(/\s+/).filter(w => w.length > 0).length;
        const chars = text.length;

        if (elements.wordCount) elements.wordCount.textContent = words + ' words';
        if (elements.charCount) elements.charCount.textContent = chars + ' characters';
    }

    /**
     * Schedule auto-save
     */
    function scheduleAutoSave() {
        if (autoSaveTimer) clearTimeout(autoSaveTimer);

        if (elements.saveStatus) {
            elements.saveStatus.textContent = 'Unsaved';
            elements.saveStatus.className = 'status-item save-status';
        }

        autoSaveTimer = setTimeout(saveDocument, 2000);
    }

    /**
     * Save document
     */
    function saveDocument() {
        if (elements.saveStatus) {
            elements.saveStatus.textContent = 'Saving...';
            elements.saveStatus.className = 'status-item save-status saving';
        }

        const title = elements.title?.innerText || 'Untitled';
        const content = elements.editor?.innerHTML || '';

        if (typeof htmx !== 'undefined') {
            htmx.ajax('POST', '/api/ui/paper/save', {
                swap: 'none',
                values: { title, content }
            }).then(() => {
                if (elements.saveStatus) {
                    elements.saveStatus.textContent = 'Saved';
                    elements.saveStatus.className = 'status-item save-status saved';
                }
                updateLastEdited();
            }).catch(() => {
                if (elements.saveStatus) {
                    elements.saveStatus.textContent = 'Save failed';
                    elements.saveStatus.className = 'status-item save-status';
                }
            });
        } else {
            // Fallback for when HTMX is not available
            if (elements.saveStatus) {
                elements.saveStatus.textContent = 'Saved';
                elements.saveStatus.className = 'status-item save-status saved';
            }
        }
    }

    /**
     * Update last edited timestamp
     */
    function updateLastEdited() {
        const lastEdited = document.getElementById('last-edited');
        if (lastEdited) {
            lastEdited.textContent = 'Last edited: Just now';
        }
    }

    /**
     * Toggle focus mode
     */
    function toggleFocusMode() {
        document.querySelector('.paper-container')?.classList.toggle('focus-mode');
    }

    /**
     * Escape HTML to prevent XSS
     */
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // Initialize on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    // Expose for external use
    window.Paper = {
        saveDocument,
        insertTodo,
        insertTable,
        insertImage,
        insertLink,
        toggleFocusMode
    };
})();
