// Projector State
    let projectorState = {
        isOpen: false,
        contentType: null,
        source: null,
        options: {},
        currentSlide: 1,
        totalSlides: 1,
        currentImage: 0,
        totalImages: 1,
        zoom: 100,
        rotation: 0,
        isPlaying: false,
        isLooping: false,
        isMuted: false,
        lineNumbers: true,
        wordWrap: false
    };

    // Get media element
    function getMediaElement() {
        return document.querySelector('.projector-video, .projector-audio');
    }

    // Open Projector
    function openProjector(data) {
        const overlay = document.getElementById('projector-overlay');
        const content = document.getElementById('projector-content');
        const loading = document.getElementById('projector-loading');
        const title = document.getElementById('projector-title');
        const icon = document.getElementById('projector-icon');

        // Reset state
        projectorState = {
            ...projectorState,
            isOpen: true,
            contentType: data.content_type,
            source: data.source_url,
            options: data.options || {}
        };

        // Set title
        title.textContent = data.title || 'Content Viewer';

        // Set icon based on content type
        const icons = {
            'Video': '🎬',
            'Audio': '🎵',
            'Image': '🖼️',
            'Pdf': '📄',
            'Presentation': '📊',
            'Code': '💻',
            'Spreadsheet': '📈',
            'Markdown': '📝',
            'Html': '🌐',
            'Document': '📃'
        };
        icon.textContent = icons[data.content_type] || '📁';

        // Show loading
        loading.classList.remove('hidden');
        hideAllControls();

        // Show overlay
        overlay.classList.remove('hidden');

        // Load content based on type
        loadContent(data);
    }

    // Load Content
    function loadContent(data) {
        const content = document.getElementById('projector-content');
        const loading = document.getElementById('projector-loading');

        setTimeout(() => {
            loading.classList.add('hidden');

            switch (data.content_type) {
                case 'Video':
                    loadVideo(content, data);
                    break;
                case 'Audio':
                    loadAudio(content, data);
                    break;
                case 'Image':
                    loadImage(content, data);
                    break;
                case 'Pdf':
                    loadPdf(content, data);
                    break;
                case 'Presentation':
                    loadPresentation(content, data);
                    break;
                case 'Code':
                    loadCode(content, data);
                    break;
                case 'Markdown':
                    loadMarkdown(content, data);
                    break;
                case 'Iframe':
                case 'Html':
                    loadIframe(content, data);
                    break;
                default:
                    loadGeneric(content, data);
            }
        }, 300);
    }

    // Load Video
    function loadVideo(container, data) {
        const loading = document.getElementById('projector-loading');

        const video = document.createElement('video');
        video.className = 'projector-video';
        video.src = data.source_url;
        video.controls = false;
        video.autoplay = data.options?.autoplay || false;
        video.loop = data.options?.loop_content || false;
        video.muted = data.options?.muted || false;

        video.addEventListener('loadedmetadata', () => {
            loading.classList.add('hidden');
            updateTimeDisplay();
        });

        video.addEventListener('timeupdate', () => {
            updateProgress();
            updateTimeDisplay();
        });

        video.addEventListener('play', () => {
            projectorState.isPlaying = true;
            document.getElementById('play-pause-btn').textContent = '⏸️';
        });

        video.addEventListener('pause', () => {
            projectorState.isPlaying = false;
            document.getElementById('play-pause-btn').textContent = '▶️';
        });

        video.addEventListener('ended', () => {
            if (!projectorState.isLooping) {
                projectorState.isPlaying = false;
                document.getElementById('play-pause-btn').textContent = '▶️';
            }
        });

        // Clear and add video
        clearContent(container);
        container.appendChild(video);

        // Show media controls
        showControls('media');
    }

    // Load Audio
    function loadAudio(container, data) {
        const wrapper = document.createElement('div');
        wrapper.style.textAlign = 'center';
        wrapper.style.padding = '40px';

        // Visualizer placeholder
        const visualizer = document.createElement('canvas');
        visualizer.className = 'audio-visualizer';
        visualizer.id = 'audio-visualizer';
        wrapper.appendChild(visualizer);

        const audio = document.createElement('audio');
        audio.className = 'projector-audio';
        audio.src = data.source_url;
        audio.autoplay = data.options?.autoplay || false;
        audio.loop = data.options?.loop_content || false;

        audio.addEventListener('loadedmetadata', () => updateTimeDisplay());
        audio.addEventListener('timeupdate', () => {
            updateProgress();
            updateTimeDisplay();
        });
        audio.addEventListener('play', () => {
            projectorState.isPlaying = true;
            document.getElementById('play-pause-btn').textContent = '⏸️';
        });
        audio.addEventListener('pause', () => {
            projectorState.isPlaying = false;
            document.getElementById('play-pause-btn').textContent = '▶️';
        });

        wrapper.appendChild(audio);

        clearContent(container);
        container.appendChild(wrapper);

        showControls('media');
    }

    // Load Image
    function loadImage(container, data) {
        const img = document.createElement('img');
        img.className = 'projector-image';
        img.src = data.source_url;
        img.alt = data.title || 'Image';
        img.id = 'projector-img';

        img.addEventListener('load', () => {
            document.getElementById('projector-loading').classList.add('hidden');
        });

        img.addEventListener('error', () => {
            showError('Failed to load image');
        });

        clearContent(container);
        container.appendChild(img);

        // Hide nav if single image
        document.getElementById('prev-image-btn').style.display =
            projectorState.totalImages > 1 ? 'block' : 'none';
        document.getElementById('next-image-btn').style.display =
            projectorState.totalImages > 1 ? 'block' : 'none';

        showControls('image');
        updateImageInfo();
    }

    // Load PDF
    function loadPdf(container, data) {
        const iframe = document.createElement('iframe');
        iframe.className = 'projector-pdf';
        iframe.src = `/static/pdfjs/web/viewer.html?file=${encodeURIComponent(data.source_url)}`;

        clearContent(container);
        container.appendChild(iframe);

        showControls('slide');
    }

    // Load Presentation
    function loadPresentation(container, data) {
        const wrapper = document.createElement('div');
        wrapper.className = 'projector-presentation';

        const slideContainer = document.createElement('div');
        slideContainer.className = 'slide-container';
        slideContainer.id = 'slide-container';

        // For now, show as images (each slide converted to image)
        const slideImg = document.createElement('img');
        slideImg.className = 'slide-content';
        slideImg.id = 'slide-content';
        slideImg.src = `${data.source_url}?slide=1`;

        slideContainer.appendChild(slideImg);
        wrapper.appendChild(slideContainer);

        clearContent(container);
        container.appendChild(wrapper);

        showControls('slide');
        updateSlideInfo();
    }

    // Load Code
    function loadCode(container, data) {
        const wrapper = document.createElement('div');
        wrapper.className = 'projector-code';
        wrapper.id = 'code-container';
        if (projectorState.lineNumbers) {
            wrapper.classList.add('line-numbers');
        }

        const pre = document.createElement('pre');
        const code = document.createElement('code');

        // Fetch code content
        fetch(data.source_url)
            .then(res => res.text())
            .then(text => {
                // Split into lines for line numbers
                const lines = text.split('\n').map(line =>
                    `<span class="line">${escapeHtml(line)}</span>`
                ).join('\n');
                code.innerHTML = lines;

                // Apply syntax highlighting if Prism is available
                if (window.Prism) {
                    Prism.highlightElement(code);
                }
            })
            .catch(() => {
                code.textContent = 'Failed to load code';
            });

        pre.appendChild(code);
        wrapper.appendChild(pre);

        clearContent(container);
        container.appendChild(wrapper);

        // Update code info
        const filename = data.source_url.split('/').pop();
        document.getElementById('code-info').textContent = filename;

        showControls('code');
    }

    // Load Markdown
    function loadMarkdown(container, data) {
        const wrapper = document.createElement('div');
        wrapper.className = 'projector-markdown';

        fetch(data.source_url)
            .then(res => res.text())
            .then(text => {
                // Simple markdown parsing (use marked.js in production)
                wrapper.innerHTML = parseMarkdown(text);
            })
            .catch(() => {
                wrapper.innerHTML = '<p>Failed to load markdown</p>';
            });

        clearContent(container);
        container.appendChild(wrapper);

        hideAllControls();
    }

    // Load Iframe
    function loadIframe(container, data) {
        const iframe = document.createElement('iframe');
        iframe.className = 'projector-iframe';
        iframe.src = data.source_url;
        iframe.allow = 'accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture';
        iframe.allowFullscreen = true;

        clearContent(container);
        container.appendChild(iframe);

        hideAllControls();
    }

    // Load Generic
    function loadGeneric(container, data) {
        const wrapper = document.createElement('div');
        wrapper.style.textAlign = 'center';
        wrapper.style.padding = '40px';
        wrapper.style.color = 'var(--text-muted, #888)';

        wrapper.innerHTML = `
        <div style="font-size: 64px; margin-bottom: 20px;">📁</div>
        <div style="font-size: 18px; margin-bottom: 10px;">Cannot preview this file type</div>
        <a href="${data.source_url}" download style="color: var(--accent, #667eea); text-decoration: none;">
            ⬇️ Download File
        </a>
    `;

        clearContent(container);
        container.appendChild(wrapper);

        hideAllControls();
    }

    // Show Error
    function showError(message) {
        const content = document.getElementById('projector-content');
        content.innerHTML = `
        <div class="projector-error">
            <span class="projector-error-icon">❌</span>
            <span class="projector-error-message">${message}</span>
        </div>
    `;
    }

    // Clear Content
    function clearContent(container) {
        const loading = document.getElementById('projector-loading');
        container.innerHTML = '';
        container.appendChild(loading);
    }

    // Show/Hide Controls
    function showControls(type) {
        hideAllControls();
        const controls = document.getElementById(`${type}-controls`);
        if (controls) {
            controls.classList.remove('hidden');
        }
    }

    function hideAllControls() {
        document.getElementById('media-controls').classList.add('hidden');
        document.getElementById('slide-controls').classList.add('hidden');
        document.getElementById('image-controls').classList.add('hidden');
        document.getElementById('code-controls').classList.add('hidden');
    }

    // Close Projector
    function closeProjector() {
        const overlay = document.getElementById('projector-overlay');
        overlay.classList.add('hidden');
        projectorState.isOpen = false;

        // Stop any playing media
        const media = getMediaElement();
        if (media) {
            media.pause();
            media.src = '';
        }

        // Clear content
        const content = document.getElementById('projector-content');
        const loading = document.getElementById('projector-loading');
        content.innerHTML = '';
        content.appendChild(loading);
    }

    function closeProjectorOnOverlay(event) {
        if (event.target.id === 'projector-overlay') {
            closeProjector();
        }
    }

    // Media Controls
    function togglePlayPause() {
        const media = getMediaElement();
        if (media) {
            if (media.paused) {
                media.play();
            } else {
                media.pause();
            }
        }
    }

    function mediaSeekBack() {
        const media = getMediaElement();
        if (media) {
            media.currentTime = Math.max(0, media.currentTime - 10);
        }
    }

    function mediaSeekForward() {
        const media = getMediaElement();
        if (media) {
            media.currentTime = Math.min(media.duration, media.currentTime + 10);
        }
    }
