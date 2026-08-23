
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

(function(){ var __cb = init; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
