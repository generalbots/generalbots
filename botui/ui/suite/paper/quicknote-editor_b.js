/* Split from quicknote-editor.js for #2417 (450-line limit). */
function clearNote() {
    const message = window.gbT ? window.gbT("paper-clear-confirm", "Clear every note in this document?") : "Clear every note in this document?";
    const wipe = () => {
        noteContent.innerHTML = '';
        updateStats();
        saveNote();
    };
    if (!noteContent.innerText.trim()) { wipe(); return; }
    // #2315 — destructive bulk action through the themed danger dialog, so it
    // works in the desktop shell and reads as dangerous.
    GBModal.confirm(message, "Clear notes", wipe);
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
        GBModal.alert(window.gbT ? window.gbT("paper-empty-warning", "Write some notes first.") : "Write some notes first.");
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
    if (!summaryEl || !listEl) return;

    const total = results.tasks.length + results.events.length + results.emails.length + results.files.length;
    summaryEl.textContent = '';
    listEl.textContent = '';

    if (total === 0) {
        const item = document.createElement('div');
        item.className = 'summary-item';
        item.textContent = 'No specific actions detected. Try being more explicit with tasks, dates, or email mentions.';
        summaryEl.appendChild(item);
    } else {
        const addSummary = (count, label) => {
            const item = document.createElement('div');
            item.className = 'summary-item';
            const num = document.createElement('span');
            num.className = 'count';
            num.textContent = String(count);
            item.appendChild(num);
            item.appendChild(document.createTextNode(` ${label}`));
            summaryEl.appendChild(item);
        };
        if (results.tasks.length) addSummary(results.tasks.length, 'Tasks');
        if (results.events.length) addSummary(results.events.length, 'Events');
        if (results.emails.length) addSummary(results.emails.length, 'Emails');
        if (results.files.length) addSummary(results.files.length, 'Files');

        const appendItems = (items, type, defaultMeta) => {
            items.forEach((entry, i) => {
                listEl.appendChild(createResultItem(type, entry.title, entry.time || defaultMeta, `${type}-${i}`));
            });
        };
        appendItems(results.tasks, 'task', 'Add to task list');
        appendItems(results.events, 'event', 'Add to calendar');
        appendItems(results.emails, 'email', 'Draft email');
        appendItems(results.files, 'file', 'Create document');
    }

    if (typeof showModal === 'function') showModal('results-modal');
}

function createResultItem(type, title, meta, id) {
    const icons = {
        task: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>',
        event: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>',
        email: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/></svg>',
        file: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>'
    };

    const row = document.createElement('div');
    row.className = 'result-item';

    const icon = document.createElement('div');
    icon.className = `result-icon ${type}`;
    icon.innerHTML = icons[type] || '';

    const content = document.createElement('div');
    content.className = 'result-content';
    const titleEl = document.createElement('div');
    titleEl.className = 'result-title';
    titleEl.textContent = String(title || '');
    const metaEl = document.createElement('div');
    metaEl.className = 'result-meta';
    metaEl.textContent = String(meta || '');
    content.appendChild(titleEl);
    content.appendChild(metaEl);

    const checkbox = document.createElement('input');
    checkbox.type = 'checkbox';
    checkbox.className = 'result-checkbox';
    checkbox.id = id;
    checkbox.checked = true;

    row.appendChild(icon);
    row.appendChild(content);
    row.appendChild(checkbox);
    return row;
}

function executeAllActions() {
    const checkboxes = document.querySelectorAll('.result-checkbox:checked');

    if (checkboxes.length === 0) {
        if (window.GBModal && typeof window.GBModal.alert === 'function') {
            GBModal.alert(window.gbT ? window.gbT("paper-no-actions", "No actions selected.") : "No actions selected.");
        } else if (window.GBToasts && typeof window.GBToasts.show === 'function') {
            window.GBToasts.show("Paper", window.gbT ? window.gbT("paper-no-actions", "No actions selected.") : "No actions selected.", "warn");
        }
        return;
    }

    if (typeof hideModal === 'function') hideModal('results-modal');
    updateSyncStatus('syncing');

    setTimeout(() => {
        updateSyncStatus('synced');
        if (window.GBModal && typeof window.GBModal.alert === 'function') {
            GBModal.alert(`${checkboxes.length} action(s) processed.`);
        } else if (window.GBToasts && typeof window.GBToasts.show === 'function') {
            window.GBToasts.show("Paper", `${checkboxes.length} action(s) processed.`, "success");
        }
    }, 1500);
}

window.paperSetNoteHtml = paperSetNoteHtml;

(function () {
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", init);
    } else {
        init();
    }
})();
