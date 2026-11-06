/* Dev Chat Widget — module 3/4: event wiring, sending and WebSocket
   Split from dev-chat.js for #1731. Loaded third by the dev-chat.js loader. */
(function () {
    'use strict';

    const GB = window.__gbDevChat;
    if (!GB) return;

    const CONFIG = GB.CONFIG;
    const state = GB.state;
    const { btn, closeBtn, messages, input, sendBtn, actions } = GB.els;
    const toggle = GB.toggle;
    const getContext = GB.getContext;
    const addMsg = GB.addMsg;
    const showFileChange = GB.showFileChange;
    const showTyping = GB.showTyping;
    const hideTyping = GB.hideTyping;

    btn.addEventListener('click', toggle);
    closeBtn.addEventListener('click', toggle);

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        if (e.ctrlKey && e.shiftKey && e.key === 'D') {
            e.preventDefault();
            toggle();
        }
        if (e.key === 'Escape' && state.isOpen) {
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

    // Send message
    async function send() {
        const text = input.value.trim();
        if (!text) return;

        addMsg(text, 'user');
        input.value = '';
        showTyping();

        try {
            if (state.ws && state.ws.readyState === WebSocket.OPEN) {
                state.ws.send(JSON.stringify({
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
        if (state.ws && state.ws.readyState === WebSocket.OPEN) return;

        try {
            const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
            state.ws = new WebSocket(`${proto}//${location.host}${CONFIG.wsEndpoint}`);

            state.ws.onopen = () => addMsg('Connected', 'system');

            state.ws.onmessage = (e) => {
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

            state.ws.onclose = () => setTimeout(connectWS, 3000);
        } catch (err) {
            console.error('Dev chat WS error:', err);
        }
    }

    // Clear chat
    function clearChat() {
        messages.innerHTML = '<div class="gb-dev-msg system">Chat cleared.</div>';
        GB.clearUserData();
    }

    GB.send = send;
    GB.connectWS = connectWS;
    GB.clearChat = clearChat;
})();
