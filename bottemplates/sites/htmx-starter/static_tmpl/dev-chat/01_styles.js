/* Dev Chat Widget — module 1/4: injected stylesheet
   Split from dev-chat.js for #1731. Loaded first by the dev-chat.js loader. */
(function () {
    'use strict';

    const GB = (window.__gbDevChat = window.__gbDevChat || {});

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

    GB.STYLES = styles;
})();
