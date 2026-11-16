function showHistory() {
    const historyList = document.getElementById('history-list');
    const history = typeof gbStoreGet === 'function' ? gbStoreGet('paper-history', []) : [];

    if (!historyList) return;
    historyList.textContent = '';

    if (!Array.isArray(history) || history.length === 0) {
        const empty = document.createElement('div');
        empty.style.textAlign = 'center';
        empty.style.color = 'var(--text-secondary)';
        empty.style.padding = '20px';
        empty.textContent = 'No history yet';
        historyList.appendChild(empty);
    } else {
        history.forEach((item, index) => {
            const row = document.createElement('button');
            row.type = 'button';
            row.className = 'history-item';
            row.setAttribute('data-history-index', String(index));

            const preview = document.createElement('span');
            preview.className = 'history-item-preview';
            preview.textContent = (item && item.preview) || 'Empty note';

            const date = document.createElement('span');
            date.className = 'history-item-date';
            try {
                date.textContent = typeof gbFormatDate === 'function' ? gbFormatDate(new Date(item && item.timestamp)) : String((item && item.timestamp) || '');
            } catch (e) {
                date.textContent = String((item && item.timestamp) || '');
            }

            row.appendChild(preview);
            row.appendChild(date);
            historyList.appendChild(row);
        });
    }

    if (typeof showModal === 'function') showModal('history-modal');
}

function restoreFromHistory(index) {
    const history = typeof gbStoreGet === 'function' ? gbStoreGet('paper-history', []) : [];
    if (history[index] && typeof window.paperSetNoteHtml === 'function') {
        window.paperSetNoteHtml(history[index].content || '');
        if (typeof window.updateStats === 'function') window.updateStats();
        if (typeof hideModal === 'function') hideModal('history-modal');
    }
}

function showSettings() {
    if (typeof showModal === 'function') showModal('settings-modal');
}

function loadSettings() {
    const saved = localStorage.getItem('paper-settings');
    if (saved) {
        try {
            settings = JSON.parse(saved) || settings;
        } catch (e) {
            /* keep defaults */
        }

        const autosave = document.getElementById('setting-autosave');
        const calendar = document.getElementById('setting-calendar');
        const tasklist = document.getElementById('setting-tasklist');
        if (autosave) autosave.checked = !!settings.autosave;
        if (calendar) calendar.value = settings.calendar || 'default';
        if (tasklist) tasklist.value = settings.tasklist || 'default';
    }
}

function updateSetting(key, value) {
    if (typeof settings === 'undefined') {
        window.settings = window.settings || { autosave: true, calendar: 'default', tasklist: 'default' };
    }
    settings[key] = value;
    localStorage.setItem('paper-settings', JSON.stringify(settings));
}

function showModal(id) {
    const el = document.getElementById(id);
    if (el) el.classList.remove('hidden');
}

function hideModal(id) {
    const el = document.getElementById(id);
    if (el) el.classList.add('hidden');
}

function hideAllModals() {
    document.querySelectorAll('.modal').forEach((modal) => {
        modal.classList.add('hidden');
    });
}
