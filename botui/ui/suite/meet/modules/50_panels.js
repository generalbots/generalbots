/* Meet — modals, side panels, device tests and the more-options menu. */
(function () {
    "use strict";

    window.showModal = function(id) {
        var modal = document.getElementById(id);
        if (modal) modal.showModal();
    };

    window.hideModal = function(id) {
        var modal = document.getElementById(id);
        if (modal) modal.close();
    };

    window.togglePanel = function(name) {
        var panels = ['participants', 'chat', 'transcription'];
        panels.forEach(function(p) {
            var panel = document.getElementById(p + '-panel');
            if (panel) {
                if (p === name) {
                    panel.classList.toggle('hidden');
                } else {
                    panel.classList.add('hidden');
                }
            }
        });
    };

    window.showReactions = function() {
        var popup = document.getElementById('reactions-popup');
        if (popup) popup.classList.toggle('hidden');
    };

    window.copyMeetingLink = function() {
        var input = document.getElementById('meeting-link');
        if (input) {
            input.select();
            navigator.clipboard.writeText(input.value);
        }
    };

    window.testVideo = async function() {
        try {
            var stream = await navigator.mediaDevices.getUserMedia({ video: true });
            var preview = document.getElementById('preview-video');
            if (preview) preview.srcObject = stream;
        } catch (err) {
            console.error('Error testing video:', err);
        }
    };

    window.testAudio = function() {
        try {
            var AudioCtx = window.AudioContext || window.webkitAudioContext;
            if (!AudioCtx) {
                if (window.showNotification) window.showNotification('Audio not supported');
                return;
            }
            var ctx = new AudioCtx();
            var osc = ctx.createOscillator();
            var gain = ctx.createGain();
            gain.gain.value = 0.05;
            osc.type = 'sine';
            osc.frequency.value = 440;
            osc.connect(gain);
            gain.connect(ctx.destination);
            osc.start();
            setTimeout(function () {
                try {
                    osc.stop();
                    ctx.close();
                } catch (e) {}
            }, 350);
            if (window.showNotification) window.showNotification('Playing test tone...');
        } catch (err) {
            console.error('Audio test failed:', err);
            if (window.showNotification) window.showNotification('Audio test failed');
        }
    };

    window.toggleFullscreen = function() {
        var room = document.getElementById('meeting-room');
        if (!room) return;
        if (document.fullscreenElement) {
            document.exitFullscreen();
        } else if (room.requestFullscreen) {
            room.requestFullscreen();
        }
    };

    window.showMoreOptions = function() {
        var existing = document.getElementById('more-options-menu');
        if (existing) {
            existing.remove();
            return;
        }
        var menu = document.createElement('div');
        menu.id = 'more-options-menu';
        menu.className = 'more-options-menu';
        menu.innerHTML = [
            '<button type="button" onclick="copyMeetingLink()">Copy meeting link</button>',
            '<button type="button" onclick="toggleFullscreen()">Toggle fullscreen</button>',
            '<button type="button" onclick="togglePanel(\'transcription\')">Toggle transcription</button>',
            '<button type="button" onclick="showMoreOptions()">Close</button>',
        ].join('');
        document.body.appendChild(menu);
        var btn = document.getElementById('more-btn');
        if (btn) {
            var r = btn.getBoundingClientRect();
            menu.style.position = 'fixed';
            menu.style.right = window.innerWidth - r.right + 'px';
            menu.style.bottom = window.innerHeight - r.top + 8 + 'px';
        }
    };

    window.showNotification = function(message) {
        console.log('Notification:', message);
    };
})();
