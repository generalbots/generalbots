/**
 * Dev Chat Widget - Injectable Script (loader)
 *
 * Add to any page: <script src="/_assets/dev-chat.js"></script>
 * Or inject dynamically in dev mode
 *
 * The widget itself lives in ordered modules under ./dev-chat/ (#1731):
 *   01_styles.js   injected stylesheet
 *   02_dom.js      configuration, DOM and message rendering
 *   03_messaging.js event wiring, sending and WebSocket
 *   04_storage.js  user_data persistence and window.gbDevChat
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

    const current = document.currentScript;
    const entry = (current && current.src) ? current.src : '/_assets/dev-chat.js';

    [
        'dev-chat/01_styles.js',
        'dev-chat/02_dom.js',
        'dev-chat/03_messaging.js',
        'dev-chat/04_storage.js'
    ].forEach((name) => {
        const script = document.createElement('script');
        script.src = new URL(name, entry).href;
        script.async = false;
        document.head.appendChild(script);
    });
})();
