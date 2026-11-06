/* Dev Chat Widget — module 2/4: configuration, DOM and message rendering
   Split from dev-chat.js for #1731. Loaded second by the dev-chat.js loader. */
(function () {
    'use strict';

    const GB = window.__gbDevChat;
    if (!GB) return;

    const CONFIG = {
        apiEndpoint: '/api/chat/dev',
        wsEndpoint: '/ws/dev',
        userDataEndpoint: '/api/db/user_data',
        maxHistory: 50
    };

    const state = {
        isOpen: false,
        ws: null,
        isTyping: false
    };

    GB.CONFIG = CONFIG;
    GB.state = state;

    // Inject stylesheet
    const styleEl = document.createElement('style');
    styleEl.textContent = GB.STYLES;
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

    const els = { btn, panel, closeBtn, messages, input, sendBtn, actions };
    GB.els = els;

    // Toggle panel
    function toggle() {
        state.isOpen = !state.isOpen;
        panel.classList.toggle('open', state.isOpen);
        if (state.isOpen) {
            input.focus();
            GB.connectWS();
        }
    }

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
        GB.saveToUserData();
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
        if (state.isTyping) return;
        state.isTyping = true;
        const el = document.createElement('div');
        el.id = 'gb-dev-typing';
        el.className = 'gb-dev-typing';
        el.innerHTML = '<span></span><span></span><span></span>';
        messages.appendChild(el);
        messages.scrollTop = messages.scrollHeight;
    }

    function hideTyping() {
        state.isTyping = false;
        const el = document.getElementById('gb-dev-typing');
        if (el) el.remove();
    }

    GB.toggle = toggle;
    GB.getContext = getContext;
    GB.addMsg = addMsg;
    GB.showFileChange = showFileChange;
    GB.showTyping = showTyping;
    GB.hideTyping = hideTyping;
})();
