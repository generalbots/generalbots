/* Dev Chat Widget — module 4/4: user_data persistence and public API
   Split from dev-chat.js for #1731. Loaded last by the dev-chat.js loader. */
(function () {
    'use strict';

    const GB = window.__gbDevChat;
    if (!GB) return;

    const CONFIG = GB.CONFIG;
    const state = GB.state;
    const { messages, input } = GB.els;
    const toggle = GB.toggle;
    const getContext = GB.getContext;
    const addMsg = GB.addMsg;
    const showFileChange = GB.showFileChange;
    const send = GB.send;

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

    GB.saveToUserData = saveToUserData;
    GB.clearUserData = clearUserData;

    // Load history on init
    loadFromUserData();

    // Expose API for external use
    window.gbDevChat = {
        open: () => { if (!state.isOpen) toggle(); },
        close: () => { if (state.isOpen) toggle(); },
        send: (msg) => { input.value = msg; send(); },
        addMessage: addMsg,
        showFileChange
    };

    // The parts registry is only needed while the modules above are loading.
    delete window.__gbDevChat;
})();
