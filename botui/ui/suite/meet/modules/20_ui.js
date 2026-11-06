/* Meet — in-room chat, reactions and the call timer. */
(function () {
    "use strict";
    var M = window.GBMeetInternal;
    var escapeHtml = M.escapeHtml;

    function addChatMessage(sender, content) {
        var container = document.getElementById('chat-messages');
        if (!container) return;

        var msg = document.createElement('div');
        msg.className = 'chat-message';
        msg.innerHTML =
            '<div class="chat-sender">' + escapeHtml(sender) + '</div>' +
            '<div class="chat-text">' + escapeHtml(content) + '</div>';
        container.appendChild(msg);
        container.scrollTop = container.scrollHeight;
    }

    function showFloatingReaction(emoji) {
        var el = document.createElement('div');
        el.textContent = emoji;
        el.style.cssText = 'position:fixed;bottom:120px;font-size:2rem;pointer-events:none;z-index:999;transition:all 2s ease-out;opacity:1;';
        el.style.left = (Math.random() * 60 + 20) + '%';
        document.body.appendChild(el);

        setTimeout(function() {
            el.style.opacity = '0';
            el.style.transform = 'translateY(-200px)';
        }, 50);

        setTimeout(function() { el.remove(); }, 2100);
    }

    function startTimer() {
        var state = window._meetState;
        state.timerInterval = setInterval(function() {
            state.timerSeconds++;
            var h = Math.floor(state.timerSeconds / 3600);
            var m = Math.floor((state.timerSeconds % 3600) / 60);
            var s = state.timerSeconds % 60;
            var display = [h, m, s].map(function(v) { return v.toString().padStart(2, '0'); }).join(':');
            var el = document.getElementById('room-timer');
            if (el) el.textContent = display;
        }, 1000);
    }

    function stopTimer() {
        var state = window._meetState;
        if (state.timerInterval) {
            clearInterval(state.timerInterval);
            state.timerInterval = null;
        }
        state.timerSeconds = 0;
    }
    M.addChatMessage = addChatMessage;
    M.showFloatingReaction = showFloatingReaction;
    M.startTimer = startTimer;
    M.stopTimer = stopTimer;
})();
