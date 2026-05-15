/**
 * Dev Chat Widget - Injectable Script
 *
 * Add to any page: <script src="/_assets/dev-chat.js"></script>
 * Or inject dynamically in dev mode
 *
 * Uses user_data virtual table for storage (one table for all)
 */

(function() {
    'use strict';

    // Only run in dev mode
    const isDevMode = window.location.search.includes('dev=1') ||
                      document.cookie.includes('dev_mode=1') ||
                      window.location.hostname === 'localhost' ||
                      window.location.hostname === '127.0.0.1';

    if (!isDevMode) return;

    const CONFIG = {
        apiEndpoint: '/api/chat/dev',
        wsEndpoint: '/ws/dev',
        userDataEndpoint: '/api/db/user_data',
        maxHistory: 50
    };

    let isOpen = false;
    let ws = null;
    let isTyping = false;

    // Inject styles
    const styles = `
        #gb-dev-chat-btn {
            position: fixed;
            bottom: 20px;
            right: 20px;
            width: 56px;
            height: 56px;
            border-radius: 50%;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            border: none;
            cursor: pointer;
            box-shadow: 0 4px 16px rgba(102, 126, 234, 0.4);
            z-index: 99999;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: transform 0.2s, box-shadow 0.2s;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }
        #gb-dev-chat-btn:hover {
            transform: scale(1.1);
            box-shadow: 0 6px 24px rgba(102, 126, 234, 0.6);
        }
        #gb-dev-chat-btn svg { width: 28px; height: 28px; fill: white; }
        #gb-dev-chat-btn .badge {
            position: absolute;
            top: -4px;
            right: -4px;
            background: #ef4444;
            color: white;
            font-size: 10px;
            font-weight: bold;
            padding: 2px 6px;
            border-radius: 10px;
            display: none;
        }
        #gb-dev-panel {
            position: fixed;
            bottom: 90px;
            right: 20px;
            width: 380px;
            height: 520px;
            background: #0f172a;
            border-radius: 16px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
            z-index: 99998;
            display: none;
            flex-direction: column;
            overflow: hidden;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }
        #gb-dev-panel.open {
            display: flex;
            animation: gbDevSlideUp 0.3s ease;
        }
        @keyframes gbDevSlideUp {
            from { opacity: 0; transform: translateY(20px); }
            to { opacity: 1; transform: translateY(0); }
        }
        #gb-dev-header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 16px;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }
        #gb-dev-header h3 {
            margin: 0;
            color: white;
            font-size: 16px;
            font-weight: 600;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        #gb-dev-header .dev-badge {
            background: rgba(255,255,255,0.2);
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 10px;
            text-transform: uppercase;
        }
        #gb-dev-header button {
            background: rgba(255,255,255,0.2);
            border: none;
            color: white;
            width: 28px;
            height: 28px;
            border-radius: 6px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        #gb-dev-header button:hover { background: rgba(255,255,255,0.3); }
        #gb-dev-actions {
            padding: 8px 16px;
            background: #1e293b;
            display: flex;
            gap: 6px;
            flex-wrap: wrap;
        }
        .gb-dev-action {
            background: #334155;
            border: none;
            color: #94a3b8;
            padding: 4px 10px;
            border-radius: 4px;
            font-size: 11px;
            cursor: pointer;
        }
        .gb-dev-action:hover { background: #475569; color: white; }
        #gb-dev-messages {
            flex: 1;
            overflow-y: auto;
            padding: 16px;
            display: flex;
            flex-direction: column;
            gap: 12px;
        }
        #gb-dev-messages::-webkit-scrollbar { width: 6px; }
        #gb-dev-messages::-webkit-scrollbar-track { background: transparent; }
        #gb-dev-messages::-webkit-scrollbar-thumb { background: #334155; border-radius: 3px; }
        .gb-dev-msg {
            max-width: 85%;
            padding: 10px 14px;
            border-radius: 12px;
            font-size: 14px;
            line-height: 1.4;
            word-wrap: break-word;
        }
        .gb-dev-msg.user {
            background: #667eea;
            color: white;
            align-self: flex-end;
            border-bottom-right-radius: 4px;
        }
        .gb-dev-msg.bot {
            background: #1e293b;
            color: #e2e8f0;
            align-self: flex-start;
            border-bottom-left-radius: 4px;
        }
        .gb-dev-msg.system {
            background: #064e3b;
            color: #6ee7b7;
            align-self: center;
            font-size: 12px;
            padding: 6px 12px;
        }
        .gb-dev-msg.error {
            background: #7f1d1d;
            color: #fca5a5;
        }
        .gb-dev-msg pre {
            background: #000;
            padding: 8px;
            border-radius: 6px;
            overflow-x: auto;
            margin: 8px 0 0 0;
            font-size: 12px;
        }
        .gb-dev-file {
            background: #1e293b;
            border-left: 3px solid #22c55e;
            padding: 8px 12px;
            margin: 4px 0;
            border-radius: 0 8px 8px 0;
            font-size: 12px;
            color: #94a3b8;
        }
        .gb-dev-file.modified { border-color: #f59e0b; }
        .gb-dev-file.deleted { border-color: #ef4444; }
        .gb-dev-typing {
            display: flex;
            gap: 4px;
            padding: 12px 16px;
            background: #1e293b;
            border-radius: 12px;
            align-self: flex-start;
        }
        .gb-dev-typing span {
            width: 8px;
            height: 8px;
            background: #64748b;
            border-radius: 50%;
            animation: gbDevTyping 1.4s infinite ease-in-out;
        }
        .gb-dev-typing span:nth-child(2) { animation-delay: 0.2s; }
        .gb-dev-typing span:nth-child(3) { animation-delay: 0.4s; }
        @keyframes gbDevTyping {
            0%, 60%, 100% { transform: translateY(0); }
            30% { transform: translateY(-6px); }
        }
        #gb-dev-input-area {
            padding: 12px 16px;
            background: #1e293b;
            border-top: 1px solid #334155;
            display: flex;
            gap: 8px;
        }
        #gb-dev-input {
            flex: 1;
            background: #0f172a;
            border: 1px solid #334155;
            border-radius: 8px;
            padding: 10px 14px;
            color: #e2e8f0;
            font-size: 14px;
            outline: none;
            resize: none;
            min-height: 20px;
            max-height: 100px;
            font-family: inherit;
        }
        #gb-dev-input:focus { border-color: #667eea; }
        #gb-dev-input::placeholder { color: #64748b; }
        #gb-dev-send {
            background: #667eea;
            border: none;
            border-radius: 8px;
            width: 40px;
            height: 40px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        #gb-dev-send:hover { background: #5a67d8; }
        #gb-dev-send:disabled { background: #334155; cursor: not-allowed; }
        #gb-dev-send svg { width: 20px; height: 20px; fill: white; }
    `;

    // Inject stylesheet
    const styleEl = document.createElement('style');
    styleEl.textContent = styles;
    document.head.appendChild(styleEl);

    // Create HTML
    const html = `
        <button id="gb-dev-chat-btn" title="Dev Chat (Ctrl+Shift+D)">
            <svg viewBox="0 0 24 24">
                <path d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H6l-2 2V4h16v12z"/>
                <circle cx="12" cy="10" r="1.5"/>
                <circle cx="8" cy="10" r="1.5"/>
                <circle cx="16" cy="10" r="1.5"/>
            </svg>
            <span class="badge" id="gb-dev-badge">0</span>
        </button>
        <div id="gb-dev-panel">
            <div id="gb-dev-header">
                <h3>
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="white">
                        <path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z"/>
                    </svg>
                    Dev Chat
                    <span class="dev-badge">DEV</span>
                </h3>
                <button id="gb-dev-close" title="Close">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                    </svg>
                </button>
            </div>
            <div id="gb-dev-actions">
                <button class="gb-dev-action" data-cmd="show tables">📋 Tables</button>
                <button class="gb-dev-action" data-cmd="list files">📁 Files</button>
                <button class="gb-dev-action" data-cmd="reload app">🔄 Reload</button>
                <button class="gb-dev-action" data-cmd="show errors">⚠️ Errors</button>
                <button class="gb-dev-action" data-cmd="clear">🗑️ Clear</button>
            </div>
            <div id="gb-dev-messages">
                <div class="gb-dev-msg system">Dev mode active. Talk to test your app.</div>
            </div>
            <div id="gb-dev-input-area">
                <textarea id="gb-dev-input" placeholder="Ask anything or describe changes..." rows="1"></textarea>
                <button id="gb-dev-send">
                    <svg viewBox="0 0 24 24"><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/></svg>
                </button>
            </div>
        </div>
    `;

    // Insert into DOM
    const container = document.createElement('div');
    container.id = 'gb-dev-chat-container';
    container.innerHTML = html;
    document.body.appendChild(container);

    // Elements
    const btn = document.getElementById('gb-dev-chat-btn');
    const panel = document.getElementById('gb-dev-panel');
    const closeBtn = document.getElementById('gb-dev-close');
    const messages = document.getElementById('gb-dev-messages');
    const input = document.getElementById('gb-dev-input');
    const sendBtn = document.getElementById('gb-dev-send');
    const actions = document.querySelectorAll('.gb-dev-action');

    // Toggle panel
    function toggle() {
        isOpen = !isOpen;
        panel.classList.toggle('open', isOpen);
        if (isOpen) {
            input.focus();
            connectWS();
        }
    }

    btn.addEventListener('click', toggle);
    closeBtn.addEventListener('click', toggle);

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        if (e.ctrlKey && e.shiftKey && e.key === 'D') {
            e.preventDefault();
            toggle();
        }
        if (e.key === 'Escape' && isOpen) {
            toggle();
        }
    });

    // Input handling
    input.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            send();
        }
    });

    sendBtn.addEventListener('click', send);

    // Quick actions
    actions.forEach(btn => {
        btn.addEventListener('click', () => {
            const cmd = btn.dataset.cmd;
            if (cmd === 'clear') {
                clearChat();
            } else {
                input.value = cmd;
                send();
            }
        });
    });

    // Get app context
    function getContext() {
        const match = location.pathname.match(/\/apps\/([^\/]+)/);
        return {
            app: match ? match[1] : 'default',
            url: location.href,
            path: location.pathname
        };
    }

    // Add message
    function addMsg(text, type = 'bot') {
        const msg = document.createElement('div');
        msg.className = `gb-dev-msg ${type}`;

        if (type === 'bot' && text.includes('```')) {
            msg.innerHTML = text.replace(/```(\w*)\n?([\s\S]*?)```/g, (_, lang, code) => {
                return `<pre><code>${escapeHtml(code.trim())}</code></pre>`;
            }).replace(/\n/g, '<br>');
        } else {
            msg.textContent = text;
        }

        messages.appendChild(msg);
        messages.scrollTop = messages.scrollHeight;
        saveToUserData();
    }

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // File change indicator
    function showFileChange(path, type = 'modified') {
        const el = document.createElement('div');
        el.className = `gb-dev-file ${type}`;
        const icon = type === 'created' ? '➕' : type === 'deleted' ? '➖' : '✏️';
        el.innerHTML = `${icon} <code>${path}</code>`;
        messages.appendChild(el);
        messages.scrollTop = messages.scrollHeight;
    }

    // Typing indicator
    function showTyping() {
        if (isTyping) return;
        isTyping = true;
        const el = document.createElement('div');
        el.id = 'gb-dev-typing';
        el.className = 'gb-dev-typing';
        el.innerHTML = '<span></span><span></span><span></span>';
        messages.appendChild(el);
        messages.scrollTop = messages.scrollHeight;
    }

    function hideTyping() {
        isTyping = false;
        const el = document.getElementById('gb-dev-typing');
        if (el) el.remove();
    }

    // Send message
    async function send() {
        const text = input.value.trim();
        if (!text) return;

        addMsg(text, 'user');
        input.value = '';
        showTyping();

        try {
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(JSON.stringify({
                    type: 'dev_message',
                    content: text,
                    context: getContext()
                }));
            } else {
                const res = await fetch(CONFIG.apiEndpoint, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        message: text,
                        context: getContext()
                    })
                });
                hideTyping();
                if (res.ok) {
                    handleResponse(await res.json());
                } else {
                    addMsg('Error: ' + res.statusText, 'error');
                }
            }
        } catch (err) {
            hideTyping();
            addMsg('Connection error: ' + err.message, 'error');
        }
    }

    // Handle response
    function handleResponse(data) {
        hideTyping();
        if (data.message) addMsg(data.message, 'bot');
        if (data.files_changed) {
            data.files_changed.forEach(f => showFileChange(f.path, f.type));
        }
        if (data.reload) {
            addMsg('Reloading...', 'system');
            setTimeout(() => location.reload(), 1000);
        }
    }

    // WebSocket
    function connectWS() {
        if (ws && ws.readyState === WebSocket.OPEN) return;

        try {
            const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
            ws = new WebSocket(`${proto}//${location.host}${CONFIG.wsEndpoint}`);

            ws.onopen = () => addMsg('Connected', 'system');

            ws.onmessage = (e) => {
                const data = JSON.parse(e.data);
                switch (data.type) {
                    case 'message':
                        hideTyping();
                        addMsg(data.content, 'bot');
                        break;
                    case 'file_changed':
                        showFileChange(data.path, data.change_type);
                        break;
                    case 'reload':
                        addMsg('Files changed. Reloading...', 'system');
                        setTimeout(() => location.reload(), 500);
                        break;
                    case 'error':
                        hideTyping();
                        addMsg(data.content, 'error');
                        break;
                }
            };

            ws.onclose = () => setTimeout(connectWS, 3000);
        } catch (err) {
            console.error('Dev chat WS error:', err);
        }
    }

    // Clear chat
    function clearChat() {
        messages.innerHTML = '<div class="gb-dev-msg system">Chat cleared.</div>';
        clearUserData();
    }

    // user_data virtual table storage (one table for all)
    function saveToUserData() {
        const history = Array.from(messages.children).map(m => ({
            type: m.classList[1],
            html: m.innerHTML
        })).slice(-CONFIG.maxHistory);

        fetch(CONFIG.userDataEndpoint, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                namespace: 'dev_chat',
                key: getContext().app + '_history',
                value: JSON.stringify(history)
            })
        }).catch(() => {});
    }

    function loadFromUserData() {
        fetch(`${CONFIG.userDataEndpoint}?namespace=dev_chat&key=${getContext().app}_history`)
            .then(r => r.json())
            .then(data => {
                if (data && data.value) {
                    const history = JSON.parse(data.value);
                    history.forEach(m => {
                        const el = document.createElement('div');
                        el.className = `gb-dev-msg ${m.type}`;
                        el.innerHTML = m.html;
                        messages.appendChild(el);
                    });
                }
            })
            .catch(() => {});
    }

    function clearUserData() {
        fetch(`${CONFIG.userDataEndpoint}?namespace=dev_chat&key=${getContext().app}_history`, {
            method: 'DELETE'
        }).catch(() => {});
    }

    // Load history on init
    loadFromUserData();

    // Expose API for external use
    window.gbDevChat = {
        open: () => { if (!isOpen) toggle(); },
        close: () => { if (isOpen) toggle(); },
        send: (msg) => { input.value = msg; send(); },
        addMessage: addMsg,
        showFileChange
    };

})();
