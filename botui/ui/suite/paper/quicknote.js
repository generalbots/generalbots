const noteContent = document.getElementById('note-content');
const formatPopup = document.getElementById('format-popup');
const paperDate = document.getElementById('paper-date');
const wordCountEl = document.getElementById('word-count');
const charCountEl = document.getElementById('char-count');
const lastSavedEl = document.getElementById('last-saved');
const syncStatus = document.getElementById('sync-status');

let autoSaveTimer = null;
let settings = {
    autosave: true,
    calendar: 'default',
    tasklist: 'default'
};

function init() {
    updateDate();
    loadNote();
    loadSettings();
    setupEventListeners();
    updateStats();
}

function updateDate() {
    const now = new Date();
    const options = { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' };
    paperDate.textContent = now.toLocaleDateString(undefined, options);
}

function setupEventListeners() {
    document.addEventListener('selectionchange', handleSelectionChange);

    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            hideFormatPopup();
            hideAllModals();
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

    noteContent.addEventListener('blur', () => {
        setTimeout(hideFormatPopup, 200);
    });
}

function handleNoteInput() {
    updateStats();

    if (settings.autosave) {
        clearTimeout(autoSaveTimer);
        autoSaveTimer = setTimeout(saveNote, 2000);
        updateSyncStatus('syncing');
    }
}

function handlePaste(e) {
    e.preventDefault();
    const text = e.clipboardData.getData('text/plain');
    document.execCommand('insertText', false, text);
}

function updateStats() {
    const text = noteContent.innerText || '';
    const words = text.trim() ? text.trim().split(/\s+/).length : 0;
    const chars = text.length;

    wordCountEl.textContent = words;
    charCountEl.textContent = chars;
}

function handleSelectionChange() {
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
    const rect = range.getBoundingClientRect();
    const containerRect = noteContent.closest('.paper-container').getBoundingClientRect();

    formatPopup.style.top = `${rect.top - containerRect.top - 50}px`;
    formatPopup.style.left = `${rect.left - containerRect.left + (rect.width / 2) - 100}px`;
    formatPopup.classList.remove('hidden');
}

function hideFormatPopup() {
    formatPopup.classList.add('hidden');
}

function formatText(command) {
    document.execCommand(command, false, null);
    noteContent.focus();
}

function insertCheckbox() {
    const checkbox = document.createElement('div');
    checkbox.className = 'checkbox-item';
    checkbox.innerHTML = `
        <input type="checkbox" onchange="toggleCheckbox(this)">
        <span contenteditable="true">New task</span>
    `;

    const selection = window.getSelection();
    if (selection.rangeCount) {
        const range = selection.getRangeAt(0);
        range.deleteContents();
        range.insertNode(checkbox);
        range.setStartAfter(checkbox);
        selection.removeAllRanges();
        selection.addRange(range);
    }

    hideFormatPopup();
}

function toggleCheckbox(checkbox) {
    const item = checkbox.closest('.checkbox-item');
    if (checkbox.checked) {
        item.classList.add('checked');
    } else {
        item.classList.remove('checked');
    }
    handleNoteInput();
}

function saveNote() {
    const content = noteContent.innerHTML;
    const timestamp = new Date().toISOString();

    localStorage.setItem('paper-note', content);
    localStorage.setItem('paper-note-timestamp', timestamp);

    saveToHistory(content, timestamp);

    updateSyncStatus('synced');
    updateLastSaved();
}

function loadNote() {
    const content = localStorage.getItem('paper-note');
    if (content) {
        noteContent.innerHTML = content;
        updateLastSaved();
    }
}

function saveToHistory(content, timestamp) {
    const history = JSON.parse(localStorage.getItem('paper-history') || '[]');
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
            text = `Saved ${date.toLocaleDateString()}`;
        }

        lastSavedEl.textContent = text;
    }
}

function updateSyncStatus(status) {
    syncStatus.className = 'sync-status';
    const statusText = syncStatus.querySelector('.status-text');

    if (status === 'syncing') {
        syncStatus.classList.add('syncing');
        statusText.textContent = 'Saving...';
    } else if (status === 'synced') {
        statusText.textContent = 'Synced';
    } else if (status === 'error') {
        syncStatus.classList.add('error');
        statusText.textContent = 'Error';
    }
}

function clearNote() {
    if (noteContent.innerText.trim() && !confirm('Are you sure you want to clear all notes?')) {
        return;
    }

    noteContent.innerHTML = '';
    updateStats();
    saveNote();
}

function exportNote() {
    const content = noteContent.innerText;
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `paper-note-${new Date().toISOString().split('T')[0]}.txt`;
    a.click();
    URL.revokeObjectURL(url);
}

function processNotes() {
    const content = noteContent.innerText.trim();

    if (!content) {
        alert('Please write some notes first!');
        return;
    }

    showModal('processing-modal');

    const statusEl = document.getElementById('processing-status');
    const statuses = [
        'Identifying actionable items...',
        'Detecting tasks and deadlines...',
        'Finding meeting requests...',
        'Analyzing email drafts...',
        'Preparing actions...'
    ];

    let statusIndex = 0;
    const statusInterval = setInterval(() => {
        statusIndex = (statusIndex + 1) % statuses.length;
        statusEl.textContent = statuses[statusIndex];
    }, 800);

    setTimeout(() => {
        clearInterval(statusInterval);
        hideModal('processing-modal');
        showResults(analyzeContent(content));
    }, 3000);
}

function analyzeContent(content) {
    const lines = content.split('\n').filter(line => line.trim());
    const results = {
        tasks: [],
        events: [],
        emails: [],
        files: []
    };

    const taskPatterns = [
        /(?:todo|task|need to|must|should|have to|don't forget to|remember to)\s*[:\-]?\s*(.+)/i,
        /\[\s*\]\s*(.+)/,
        /^[-•*]\s*(.+)/
    ];

    const eventPatterns = [
        /(?:meeting|call|appointment|schedule|on)\s+(?:with\s+)?(.+?)(?:\s+(?:at|on)\s+(.+))?$/i,
        /(\d{1,2}[\/\-]\d{1,2}(?:[\/\-]\d{2,4})?)\s*[:\-]?\s*(.+)/
    ];

    const emailPatterns = [
        /(?:email|send|write to|reply to|message)\s+(.+)/i
    ];

    lines.forEach(line => {
        for (const pattern of taskPatterns) {
            const match = line.match(pattern);
            if (match) {
                results.tasks.push({
                    title: match[1] || line,
                    original: line
                });
                return;
            }
        }

        for (const pattern of eventPatterns) {
            const match = line.match(pattern);
            if (match) {
                results.events.push({
                    title: match[2] || match[1],
                    time: match[2] ? match[1] : null,
                    original: line
                });
                return;
            }
        }

        for (const pattern of emailPatterns) {
            const match = line.match(pattern);
            if (match) {
                results.emails.push({
                    title: match[1],
                    original: line
                });
                return;
            }
        }
    });

    if (results.tasks.length === 0 && results.events.length === 0 && results.emails.length === 0) {
        if (lines.length > 3) {
            results.files.push({
                title: lines[0].substring(0, 50),
                type: 'document'
            });
        }
    }

    return results;
}

function showResults(results) {
    const summaryEl = document.getElementById('results-summary');
    const listEl = document.getElementById('results-list');

    const total = results.tasks.length + results.events.length + results.emails.length + results.files.length;

    if (total === 0) {
        summaryEl.innerHTML = '<div class="summary-item">No specific actions detected. Try being more explicit with tasks, dates, or email mentions.</div>';
        listEl.innerHTML = '';
    } else {
        let summaryHtml = '';
        if (results.tasks.length) summaryHtml += `<div class="summary-item"><span class="count">${results.tasks.length}</span> Tasks</div>`;
        if (results.events.length) summaryHtml += `<div class="summary-item"><span class="count">${results.events.length}</span> Events</div>`;
        if (results.emails.length) summaryHtml += `<div class="summary-item"><span class="count">${results.emails.length}</span> Emails</div>`;
        if (results.files.length) summaryHtml += `<div class="summary-item"><span class="count">${results.files.length}</span> Files</div>`;
        summaryEl.innerHTML = summaryHtml;

        let listHtml = '';

        results.tasks.forEach((task, i) => {
            listHtml += createResultItem('task', task.title, 'Add to task list', `task-${i}`);
        });

        results.events.forEach((event, i) => {
            listHtml += createResultItem('event', event.title, event.time || 'Add to calendar', `event-${i}`);
        });

        results.emails.forEach((email, i) => {
            listHtml += createResultItem('email', email.title, 'Draft email', `email-${i}`);
        });

        results.files.forEach((file, i) => {
            listHtml += createResultItem('file', file.title, 'Create document', `file-${i}`);
        });

        listEl.innerHTML = listHtml;
    }

    showModal('results-modal');
}

function createResultItem(type, title, meta, id) {
    const icons = {
        task: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>',
        event: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>',
        email: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/></svg>',
        file: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>'
    };

    return `
        <div class="result-item">
            <div class="result-icon ${type}">${icons[type]}</div>
            <div class="result-content">
                <div class="result-title">${title}</div>
                <div class="result-meta">${meta}</div>
            </div>
            <input type="checkbox" class="result-checkbox" id="${id}" checked>
        </div>
    `;
}

function executeAllActions() {
    const checkboxes = document.querySelectorAll('.result-checkbox:checked');

    if (checkboxes.length === 0) {
        alert('No actions selected!');
        return;
    }

    hideModal('results-modal');
    updateSyncStatus('syncing');

    setTimeout(() => {
        updateSyncStatus('synced');
        alert(`${checkboxes.length} action(s) have been processed successfully!`);
    }, 1500);
}

function showHistory() {
    const historyList = document.getElementById('history-list');
    const history = JSON.parse(localStorage.getItem('paper-history') || '[]');

    if (history.length === 0) {
        historyList.innerHTML = '<div style="text-align: center; color: var(--text-secondary); padding: 20px;">No history yet</div>';
    } else {
        historyList.innerHTML = history.map((item, index) => `
            <div class="history-item" onclick="restoreFromHistory(${index})">
                <span class="history-item-preview">${item.preview || 'Empty note'}</span>
                <span class="history-item-date">${new Date(item.timestamp).toLocaleDateString()}</span>
            </div>
        `).join('');
    }

    showModal('history-modal');
}

function restoreFromHistory(index) {
    const history = JSON.parse(localStorage.getItem('paper-history') || '[]');
    if (history[index]) {
        noteContent.innerHTML = history[index].content;
        updateStats();
        hideModal('history-modal');
    }
}

function showSettings() {
    showModal('settings-modal');
}

function loadSettings() {
    const saved = localStorage.getItem('paper-settings');
    if (saved) {
        settings = JSON.parse(saved);

        document.getElementById('setting-autosave').checked = settings.autosave;
        document.getElementById('setting-calendar').value = settings.calendar;
        document.getElementById('setting-tasklist').value = settings.tasklist;
    }
}

function updateSetting(key, value) {
    settings[key] = value;
    localStorage.setItem('paper-settings', JSON.stringify(settings));
}

function showModal(id) {
    document.getElementById(id).classList.remove('hidden');
}

function hideModal(id) {
    document.getElementById(id).classList.add('hidden');
}

function hideAllModals() {
    document.querySelectorAll('.modal').forEach(modal => {
        modal.classList.add('hidden');
    });
}

document.addEventListener('DOMContentLoaded', init);
