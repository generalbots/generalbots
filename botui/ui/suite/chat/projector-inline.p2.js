
    function seekTo(percent) {
        const media = getMediaElement();
        if (media && media.duration) {
            media.currentTime = (percent / 100) * media.duration;
        }
    }

    function setVolume(value) {
        const media = getMediaElement();
        if (media) {
            media.volume = value / 100;
            projectorState.isMuted = value === 0;
            document.getElementById('mute-btn').textContent = value === 0 ? '🔇' : '🔊';
        }
    }

    function toggleMute() {
        const media = getMediaElement();
        if (media) {
            media.muted = !media.muted;
            projectorState.isMuted = media.muted;
            document.getElementById('mute-btn').textContent = media.muted ? '🔇' : '🔊';
        }
    }

    function toggleLoop() {
        const media = getMediaElement();
        if (media) {
            media.loop = !media.loop;
            projectorState.isLooping = media.loop;
            document.getElementById('loop-btn').classList.toggle('active', media.loop);
        }
    }

    function setPlaybackSpeed(speed) {
        const media = getMediaElement();
        if (media) {
            media.playbackRate = parseFloat(speed);
        }
    }

    function updateProgress() {
        const media = getMediaElement();
        if (media && media.duration) {
            const progress = (media.currentTime / media.duration) * 100;
            document.getElementById('progress-bar').value = progress;
        }
    }

    function updateTimeDisplay() {
        const media = getMediaElement();
        if (media) {
            const current = formatTime(media.currentTime);
            const duration = formatTime(media.duration || 0);
            document.getElementById('time-display').textContent = `${current} / ${duration}`;
        }
    }

    function formatTime(seconds) {
        if (isNaN(seconds)) return '0:00';
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    }

    // Slide/Page Controls
    function prevSlide() {
        if (projectorState.currentSlide > 1) {
            projectorState.currentSlide--;
            updateSlide();
        }
    }

    function nextSlide() {
        if (projectorState.currentSlide < projectorState.totalSlides) {
            projectorState.currentSlide++;
            updateSlide();
        }
    }

    function goToSlide(num) {
        const slide = parseInt(num);
        if (slide >= 1 && slide <= projectorState.totalSlides) {
            projectorState.currentSlide = slide;
            updateSlide();
        }
    }

    function updateSlide() {
        const slideContent = document.getElementById('slide-content');
        if (slideContent) {
            slideContent.src = `${projectorState.source}?slide=${projectorState.currentSlide}`;
        }
        updateSlideInfo();
    }

    function updateSlideInfo() {
        document.getElementById('slide-info').textContent =
            `Slide ${projectorState.currentSlide} of ${projectorState.totalSlides}`;
        document.getElementById('slide-input').value = projectorState.currentSlide;
    }

    // Image Controls
    function prevImage() {
        if (projectorState.currentImage > 0) {
            projectorState.currentImage--;
            updateImage();
        }
    }

    function nextImage() {
        if (projectorState.currentImage < projectorState.totalImages - 1) {
            projectorState.currentImage++;
            updateImage();
        }
    }

    function updateImage() {
        // Implementation for image galleries
        updateImageInfo();
    }

    function updateImageInfo() {
        document.getElementById('image-info').textContent =
            `${projectorState.currentImage + 1} of ${projectorState.totalImages}`;
    }

    function rotateImage() {
        projectorState.rotation = (projectorState.rotation + 90) % 360;
        const img = document.getElementById('projector-img');
        if (img) {
            img.style.transform = `rotate(${projectorState.rotation}deg) scale(${projectorState.zoom / 100})`;
        }
    }

    function fitToScreen() {
        projectorState.zoom = 100;
        projectorState.rotation = 0;
        const img = document.getElementById('projector-img');
        if (img) {
            img.style.transform = 'none';
        }
        document.getElementById('zoom-level').textContent = '100%';
    }

    // Zoom Controls
    function zoomIn() {
        projectorState.zoom = Math.min(300, projectorState.zoom + 25);
        applyZoom();
    }

    function zoomOut() {
        projectorState.zoom = Math.max(25, projectorState.zoom - 25);
        applyZoom();
    }

    function applyZoom() {
        const img = document.getElementById('projector-img');
        const slideContainer = document.getElementById('slide-container');

        if (img) {
            img.style.transform = `rotate(${projectorState.rotation}deg) scale(${projectorState.zoom / 100})`;
        }
        if (slideContainer) {
            slideContainer.style.transform = `scale(${projectorState.zoom / 100})`;
        }

        document.getElementById('zoom-level').textContent = `${projectorState.zoom}%`;
    }

    // Code Controls
    function toggleLineNumbers() {
        projectorState.lineNumbers = !projectorState.lineNumbers;
        const container = document.getElementById('code-container');
        if (container) {
            container.classList.toggle('line-numbers', projectorState.lineNumbers);
        }
    }

    function toggleWordWrap() {
        projectorState.wordWrap = !projectorState.wordWrap;
        const container = document.getElementById('code-container');
        if (container) {
            container.style.whiteSpace = projectorState.wordWrap ? 'pre-wrap' : 'pre';
        }
    }

    function setCodeTheme(theme) {
        const container = document.getElementById('code-container');
        if (container) {
            container.className = `projector-code ${projectorState.lineNumbers ? 'line-numbers' : ''} theme-${theme}`;
        }
    }

    function copyCode() {
        const code = document.querySelector('.projector-code code');
        if (code) {
            navigator.clipboard.writeText(code.textContent).then(() => {
                // Show feedback
                const btn = document.querySelector('.code-controls .control-btn:last-child');
                const originalText = btn.textContent;
                btn.textContent = '✅';
                setTimeout(() => btn.textContent = originalText, 2000);
            });
        }
    }

    // Fullscreen
    function toggleFullscreen() {
        const container = document.querySelector('.projector-container');
        const icon = document.getElementById('fullscreen-icon');

        if (!document.fullscreenElement) {
            container.requestFullscreen().then(() => {
                container.classList.add('fullscreen');
                icon.textContent = '⛶';
            }).catch(() => { });
        } else {
            document.exitFullscreen().then(() => {
                container.classList.remove('fullscreen');
                icon.textContent = '⛶';
            }).catch(() => { });
        }
    }

    // Download
    function downloadContent() {
        const link = document.createElement('a');
        link.href = projectorState.source;
        link.download = '';
        link.click();
    }

    // Share
    function shareContent() {
        if (navigator.share) {
            navigator.share({
                title: document.getElementById('projector-title').textContent,
                url: projectorState.source
            }).catch(() => { });
        } else {
            navigator.clipboard.writeText(window.location.origin + projectorState.source).then(() => {
                alert('Link copied to clipboard!');
            });
        }
    }

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        if (!projectorState.isOpen) return;

        switch (e.key) {
            case 'Escape':
                closeProjector();
                break;
            case ' ':
                e.preventDefault();
                togglePlayPause();
                break;
            case 'ArrowLeft':
                if (projectorState.contentType === 'Video' || projectorState.contentType === 'Audio') {
                    mediaSeekBack();
                } else {
                    prevSlide();
                }
                break;
            case 'ArrowRight':
                if (projectorState.contentType === 'Video' || projectorState.contentType === 'Audio') {
                    mediaSeekForward();
                } else {
                    nextSlide();
                }
                break;
            case 'f':
                toggleFullscreen();
                break;
            case 'm':
                toggleMute();
                break;
            case '+':
            case '=':
                zoomIn();
                break;
            case '-':
                zoomOut();
                break;
        }
    });

    // Helper Functions
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    function parseMarkdown(text) {
        // Simple markdown parsing - use marked.js for full support
        return text
            .replace(/^### (.*$)/gim, '<h3>$1</h3>')
            .replace(/^## (.*$)/gim, '<h2>$1</h2>')
            .replace(/^# (.*$)/gim, '<h1>$1</h1>')
            .replace(/\*\*(.*)\*\*/gim, '<strong>$1</strong>')
            .replace(/\*(.*)\*/gim, '<em>$1</em>')
            .replace(/`([^`]+)`/gim, '<code>$1</code>')
            .replace(/\n/gim, '<br>');
    }

    // Listen for play messages from WebSocket
    if (window.htmx) {
        htmx.on('htmx:wsMessage', function (event) {
            try {
                const data = JSON.parse(event.detail.message);
                if (data.type === 'play') {
                    openProjector(data.data);
                } else if (data.type === 'player_command') {
                    switch (data.command) {
                        case 'stop':
                            closeProjector();
                            break;
                        case 'pause':
                            const media = getMediaElement();
                            if (media) media.pause();
                            break;
                        case 'resume':
                            const mediaR = getMediaElement();
                            if (mediaR) mediaR.play();
                            break;
                    }
                }
            } catch (e) {
                // Not a projector message
            }
        });
    }
