function paperEscape(value) {
    if (window.GBShared && window.GBShared.html && typeof window.GBShared.html.escape === 'function') {
        return window.GBShared.html.escape(value);
    }
    return String(value == null ? '' : value)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

function sanitizePastedHtml(html) {
    if (window.GBSanitize && typeof window.GBSanitize.html === 'function') {
        return window.GBSanitize.html(String(html || ''));
    }
    return paperEscape(html);
}

function sanitizeNoteHtml(html) {
    const allowedTags = new Set(['DIV', 'P', 'BR', 'SPAN', 'STRONG', 'B', 'EM', 'I', 'U', 'UL', 'OL', 'LI', 'INPUT']);
    const allowedAttrs = {
        DIV: new Set(['class']),
        SPAN: new Set(['contenteditable']),
        INPUT: new Set(['type', 'class']),
        LI: new Set([]),
        P: new Set([]),
        UL: new Set([]),
        OL: new Set([]),
        STRONG: new Set([]),
        B: new Set([]),
        EM: new Set([]),
        I: new Set([]),
        U: new Set([]),
        BR: new Set([])
    };
    const tpl = document.createElement('template');
    tpl.innerHTML = String(html || '');
    const walk = (node) => {
        const children = Array.from(node.childNodes || []);
        for (const child of children) {
            if (child.nodeType === Node.ELEMENT_NODE) {
                const tag = child.tagName;
                if (!allowedTags.has(tag)) {
                    const text = document.createTextNode(child.textContent || '');
                    child.replaceWith(text);
                    continue;
                }
                const attrs = Array.from(child.attributes || []);
                for (const attr of attrs) {
                    const name = attr.name.toLowerCase();
                    const allowed = allowedAttrs[tag] || new Set();
                    if (name.startsWith('on') || !allowed.has(name)) {
                        child.removeAttribute(attr.name);
                        continue;
                    }
                    if (tag === 'INPUT' && name === 'type' && String(attr.value).toLowerCase() !== 'checkbox') {
                        child.setAttribute('type', 'checkbox');
                    }
                    if (tag === 'SPAN' && name === 'contenteditable') {
                        child.setAttribute('contenteditable', 'true');
                    }
                    if (name === 'class') {
                        const safe = String(attr.value || '')
                            .split(/\s+/)
                            .filter((c) => c === 'checkbox-item' || c === 'checked' || c === 'todo-checkbox')
                            .join(' ');
                        if (safe) child.setAttribute('class', safe);
                        else child.removeAttribute('class');
                    }
                }
                if (tag === 'INPUT') {
                    child.setAttribute('type', 'checkbox');
                    if (!child.classList.contains('todo-checkbox')) child.classList.add('todo-checkbox');
                    child.removeAttribute('checked');
                }
                walk(child);
            } else if (child.nodeType === Node.COMMENT_NODE) {
                child.remove();
            }
        }
    };
    walk(tpl.content);
    return tpl.innerHTML;
}

function paperSetNoteHtml(html) {
    if (!noteContent) return;
    noteContent.innerHTML = sanitizeNoteHtml(html || '');
    noteContent.querySelectorAll('input[type="checkbox"]').forEach((cb) => {
        const item = cb.closest('.checkbox-item');
        if (item) item.classList.toggle('checked', !!cb.checked);
    });
}

let noteContent;
let formatPopup;
let paperDate;
let wordCountEl;
let charCountEl;
let lastSavedEl;
let syncStatus;

let autoSaveTimer = null;
let settings = {
    autosave: true,
    calendar: 'default',
    tasklist: 'default'
};

function init() {
    noteContent = document.getElementById('note-content');
    formatPopup = document.getElementById('format-popup');
    paperDate = document.getElementById('paper-date');
    wordCountEl = document.getElementById('word-count');
    charCountEl = document.getElementById('char-count');
    lastSavedEl = document.getElementById('last-saved');
    syncStatus = document.getElementById('sync-status');

    if (!document.getElementById('paper-app')) return;

    updateDate();
    if (typeof loadSettings === 'function') loadSettings();
    loadNote();
    setupEventListeners();
    updateStats();
}

function updateDate() {
    if (!paperDate) return;
    const now = new Date();
    const options = { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' };
    paperDate.textContent = gbFormatDate(now, options);
}

function setupEventListeners() {
    if (window.__paperQuicknoteBound) return;
    window.__paperQuicknoteBound = true;

    document.addEventListener('selectionchange', handleSelectionChange);

    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            hideFormatPopup();
            if (typeof hideAllModals === 'function') hideAllModals();
        }
        if ((e.ctrlKey || e.metaKey) && e.key === 's') {
            e.preventDefault();
            saveNote();
        }
        if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
            e.preventDefault();
            formatText('bold');
        }
        if ((e.ctrlKey || e.metaKey) && e.key === 'i') {
            e.preventDefault();
            formatText('italic');
        }
        if ((e.ctrlKey || e.metaKey) && e.key === 'u') {
            e.preventDefault();
            formatText('underline');
        }
    });

    const app = document.getElementById('paper-app') || document;

    if (noteContent) {
        noteContent.addEventListener('input', handleNoteInput);
        noteContent.addEventListener('paste', handlePaste);
        noteContent.addEventListener('blur', () => {
            setTimeout(hideFormatPopup, 200);
        });
    }

    app.addEventListener('click', (e) => {
        const target = e.target;
        if (!(target instanceof Element)) return;

        const formatBtn = target.closest('[data-format]');
        if (formatBtn) {
            e.preventDefault();
            formatText(formatBtn.getAttribute('data-format'));
            return;
        }

        const closeBtn = target.closest('[data-close-modal]');
        if (closeBtn && typeof hideModal === 'function') {
            e.preventDefault();
            hideModal(closeBtn.getAttribute('data-close-modal'));
            return;
        }

        const actionBtn = target.closest('[data-action]');
        if (actionBtn) {
            e.preventDefault();
            const action = actionBtn.getAttribute('data-action');
            if (action === 'show-history' && typeof showHistory === 'function') showHistory();
            else if (action === 'show-settings' && typeof showSettings === 'function') showSettings();
            else if (action === 'insert-checkbox') insertCheckbox();
            else if (action === 'process-notes') processNotes();
            else if (action === 'clear-note') clearNote();
            else if (action === 'export-note') exportNote();
            else if (action === 'execute-all-actions') executeAllActions();
            return;
        }

        const historyItem = target.closest('[data-history-index]');
        if (historyItem && typeof restoreFromHistory === 'function') {
            e.preventDefault();
            restoreFromHistory(parseInt(historyItem.getAttribute('data-history-index'), 10));
            return;
        }

        if (target.classList && target.classList.contains('modal')) {
            target.classList.add('hidden');
        }
    });

    app.addEventListener('change', (e) => {
        const target = e.target;
        if (!(target instanceof Element)) return;
        if (target.matches('input[type="checkbox"]') && target.closest('.checkbox-item')) {
            const item = target.closest('.checkbox-item');
            if (item) item.classList.toggle('checked', target.checked);
            handleNoteInput();
            return;
        }
        const setting = target.getAttribute('data-setting');
        if (setting && typeof updateSetting === 'function') {
            updateSetting(setting, target.type === 'checkbox' ? target.checked : target.value);
        }
    });
}

function handleNoteInput() {
    if (!noteContent) return;
    updateStats();

    if (settings.autosave) {
        clearTimeout(autoSaveTimer);
        autoSaveTimer = setTimeout(saveNote, 2000);
        updateSyncStatus('syncing');
    }
}

function handlePaste(e) {
    e.preventDefault();
    if (e.clipboardData && e.clipboardData.getData('text/html')) {
        const html = sanitizePastedHtml(e.clipboardData.getData('text/html'));
        if (html) {
            gbFormat('insertHTML', sanitizeNoteHtml(html));
            return;
        }
    }
    const text = (e.clipboardData && e.clipboardData.getData('text/plain')) || '';
    gbFormat('insertText', text);
}

function updateStats() {
    if (!noteContent) return;
    const text = noteContent.innerText || '';
    const words = text.trim() ? text.trim().split(/\s+/).length : 0;
    const chars = text.length;

    if (wordCountEl) wordCountEl.textContent = String(words);
    if (charCountEl) charCountEl.textContent = String(chars);
}
window.updateStats = updateStats;

function handleSelectionChange() {
    if (!noteContent || !formatPopup) return;
    const selection = window.getSelection();

    if (!selection.rangeCount || selection.isCollapsed) {
        hideFormatPopup();
        return;
    }

    const range = selection.getRangeAt(0);
    const selectedText = range.toString().trim();

    if (!selectedText || !noteContent.contains(range.commonAncestorContainer)) {
        hideFormatPopup();
        return;
    }

    showFormatPopup(range);
}

function showFormatPopup(range) {
    if (!noteContent || !formatPopup) return;
    const rect = range.getBoundingClientRect();
    const container = noteContent.closest('.paper-container');
    const containerRect = (container || noteContent).getBoundingClientRect();

    formatPopup.style.top = `${rect.top - containerRect.top - 50}px`;
    formatPopup.style.left = `${rect.left - containerRect.left + (rect.width / 2) - 100}px`;
    formatPopup.classList.remove('hidden');
}

function hideFormatPopup() {
    if (formatPopup) formatPopup.classList.add('hidden');
}

function formatText(command) {
    gbFormat(command);
    if (noteContent) noteContent.focus();
}

function insertCheckbox() {
    if (!noteContent) return;
    const checkbox = document.createElement('div');
    checkbox.className = 'checkbox-item';

    const input = document.createElement('input');
    input.type = 'checkbox';
    input.className = 'todo-checkbox';

    const span = document.createElement('span');
    span.setAttribute('contenteditable', 'true');
    span.textContent = (typeof window.gbT === "function" ? window.gbT("paper-quicknote-new-task", "New task") : "New task");

    checkbox.appendChild(input);
    checkbox.appendChild(document.createTextNode(' '));
    checkbox.appendChild(span);

    const selection = window.getSelection();
    if (selection && selection.rangeCount) {
        const range = selection.getRangeAt(0);
        range.deleteContents();
        range.insertNode(checkbox);
        range.setStartAfter(checkbox);
        selection.removeAllRanges();
        selection.addRange(range);
    } else {
        noteContent.appendChild(checkbox);
    }

    hideFormatPopup();
}

function saveNote() {
    if (!noteContent) return;
    const content = sanitizeNoteHtml(noteContent.innerHTML);
    const timestamp = new Date().toISOString();

    localStorage.setItem('paper-note', content);
    localStorage.setItem('paper-note-timestamp', timestamp);

    saveToHistory(content, timestamp);

    updateSyncStatus('local');
    updateLastSaved();
}

function loadNote() {
    if (!noteContent) return;
    const content = localStorage.getItem('paper-note');
    if (content) {
        paperSetNoteHtml(content);
        updateLastSaved();
    }
}

function saveToHistory(content, timestamp) {
    const history = gbStoreGet('paper-history', []);
    const preview = noteContent.innerText.substring(0, 100);

    history.unshift({
        content,
        timestamp,
        preview
    });

    if (history.length > 50) {
        history.pop();
    }

    localStorage.setItem('paper-history', JSON.stringify(history));
}

function updateLastSaved() {
    const timestamp = localStorage.getItem('paper-note-timestamp');
    if (timestamp) {
        const date = new Date(timestamp);
        const now = new Date();
        const diff = now - date;

        let text;
        if (diff < 60000) {
            text = 'Saved just now';
        } else if (diff < 3600000) {
            const mins = Math.floor(diff / 60000);
            text = `Saved ${mins} minute${mins > 1 ? 's' : ''} ago`;
        } else if (diff < 86400000) {
            const hours = Math.floor(diff / 3600000);
            text = `Saved ${hours} hour${hours > 1 ? 's' : ''} ago`;
        } else {
            text = `Saved ${gbFormatDate(date)}`;
        }

        lastSavedEl.textContent = text;
    }
}

function syncLabel(status) {
    if (status === 'syncing') return 'Saving…';
    if (status === 'error') return 'Not saved — retry';
    if (status === 'local') return 'Saved on this device';
    return 'Saved';
}

// #2315 — state used to be thrown away here (`if (!syncStatus) return;` in a
// function whose only job was to change that element). Now the chip is
// mounted, labelled and announced.
function updateSyncStatus(status) {
    if (!syncStatus) return;
    syncStatus.setAttribute('data-state', status);
    syncStatus.classList.toggle('syncing', status === 'syncing');
    const text = syncStatus.querySelector('.status-text');
    if (text) text.textContent = syncLabel(status);
}

