let currentSessionId = null;
let isRecording = false;
let recordedActions = [];

async function initBrowser() {
    try {
        const resp = await fetch('/api/browser/session', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ headless: false })
        });
        const data = await resp.json();
        currentSessionId = data.id;
        document.getElementById('browserCanvas').innerHTML = `
            <div style="width:100%; height:100%; display:flex; align-items:center; justify-content:center; color: var(--text-muted);">
                Browser session ${currentSessionId.substring(0,8)} active.
            </div>`;
    } catch(e) {
        alert("Failed to initialize browser");
    }
}

async function executeAction(actionType, payload) {
    if (!currentSessionId) return;
    try {
        await fetch(`/api/browser/session/${currentSessionId}/execute`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ action_type: actionType, payload })
        });
        updateTimeline();
        captureScreenshot();
    } catch(e) {
        console.error(e);
    }
}

async function toggleRecording() {
    if (!currentSessionId) {
        alert("Please initialize browser first");
        return;
    }

    const btn = document.getElementById('recordBtn');
    
    if (isRecording) {
        // Stop recording
        try {
            await fetch(`/api/browser/session/${currentSessionId}/record/stop`, { method: 'POST' });
            isRecording = false;
            btn.textContent = '⏺ Record';
            btn.classList.remove('recording-active');
        } catch(e) {
            console.error(e);
        }
    } else {
        // Start recording
        try {
            await fetch(`/api/browser/session/${currentSessionId}/record/start`, { method: 'POST' });
            isRecording = true;
            btn.textContent = '⏹ Stop Recording';
            btn.classList.add('recording-active');
        } catch(e) {
            console.error(e);
        }
    }
}

async function browserNavigate(url) {
    if (!url) return;
    if (isRecording) {
        recordedActions.push({
            type: 'navigate',
            value: url,
            timestamp: Date.now()
        });
    }
    await executeAction('navigate', { url });
}

async function browserClick(selector) {
    if (isRecording) {
        recordedActions.push({
            type: 'click',
            selector: selector,
            timestamp: Date.now()
        });
    }
    await executeAction('click', { selector });
}

async function captureScreenshot() {
    if (!currentSessionId) return;
    try {
        await fetch(`/api/browser/session/${currentSessionId}/screenshot`);
        // We'd render this to the gallery
        const gallery = document.getElementById('browserGallery');
        if (gallery) {
            gallery.innerHTML = '<div class="screenshot-thumb">Capture</div>' + gallery.innerHTML;
        }
    } catch(e) {}
}

async function exportTest() {
    if (!currentSessionId) {
        alert("No active session");
        return;
    }
    try {
        const resp = await fetch(`/api/browser/session/${currentSessionId}/record/export`);
        const data = await resp.json();

        // Download test file
        const blob = new Blob([data.script], { type: 'text/javascript' });
        const a = document.createElement('a');
        a.href = URL.createObjectURL(blob);
        a.download = `test-${Date.now()}.spec.js`;
        a.click();
    } catch(e) {
        alert("Export failed");
    }
}

function updateTimeline() {
    const timeline = document.getElementById('browserTimeline');
    if (!timeline) return;
    
    if (recordedActions.length === 0) {
        timeline.innerHTML = '<div class="botcoder-empty">No actions recorded</div>';
        return;
    }

    timeline.innerHTML = recordedActions.map(action => `
        <div class="timeline-action">
            <span class="action-type">[${action.type}]</span> 
            <span class="action-details">${action.selector || action.value || ''}</span>
        </div>
    `).join('');
}
