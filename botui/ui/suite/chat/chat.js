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
  wordWrap: false,
};

function getMediaElement() {
  return document.querySelector(".projector-video, .projector-audio");
}

function openProjector(data) {
  const overlay = document.getElementById("projector-overlay");
  const content = document.getElementById("projector-content");
  const loading = document.getElementById("projector-loading");
  const title = document.getElementById("projector-title");
  const icon = document.getElementById("projector-icon");
  projectorState = {
    ...projectorState,
    isOpen: true,
    contentType: data.content_type,
    source: data.source_url,
    options: data.options || {},
  };
  title.textContent = data.title || "Content Viewer";

  const icons = {
    Video: "🎬",
    Audio: "🎵",
    Image: "🖼️",
    Pdf: "📄",
    Presentation: "📊",
    Code: "💻",
    Spreadsheet: "📈",
    Markdown: "📝",
    Html: "🌐",
    Document: "📃",
  };
  icon.textContent = icons[data.content_type] || "📁";
  loading.classList.remove("hidden");
  hideAllControls();
  overlay.classList.remove("hidden");
  loadContent(data);
}

// Load Content
function loadContent(data) {
  const content = document.getElementById("projector-content");
  const loading = document.getElementById("projector-loading");

  setTimeout(() => {
    loading.classList.add("hidden");

    switch (data.content_type) {
      case "Video":
        loadVideo(content, data);
        break;
      case "Audio":
        loadAudio(content, data);
        break;
      case "Image":
        loadImage(content, data);
        break;
      case "Pdf":
        loadPdf(content, data);
        break;
      case "Presentation":
        loadPresentation(content, data);
        break;
      case "Code":
        loadCode(content, data);
        break;
      case "Markdown":
        loadMarkdown(content, data);
        break;
      case "Iframe":
      case "Html":
        loadIframe(content, data);
        break;
      default:
        loadGeneric(content, data);
    }
  }, 300);
}

// Load Video
function loadVideo(container, data) {
  const loading = document.getElementById("projector-loading");

  const video = document.createElement("video");
  video.className = "projector-video";
  video.src = data.source_url;
  video.controls = false;
  video.autoplay = data.options?.autoplay || false;
  video.loop = data.options?.loop_content || false;
  video.muted = data.options?.muted || false;

  video.addEventListener("loadedmetadata", () => {
    loading.classList.add("hidden");
    updateTimeDisplay();
  });

  video.addEventListener("timeupdate", () => {
    updateProgress();
    updateTimeDisplay();
  });

  video.addEventListener("play", () => {
    projectorState.isPlaying = true;
    document.getElementById("play-pause-btn").textContent = "⏸️";
  });

  video.addEventListener("pause", () => {
    projectorState.isPlaying = false;
    document.getElementById("play-pause-btn").textContent = "▶️";
  });

  video.addEventListener("ended", () => {
    if (!projectorState.isLooping) {
      projectorState.isPlaying = false;
      document.getElementById("play-pause-btn").textContent = "▶️";
    }
  });
  clearContent(container);
  container.appendChild(video);
  showControls("media");
}

// Load Audio
function loadAudio(container, data) {
  const wrapper = document.createElement("div");
  wrapper.style.textAlign = "center";
  wrapper.style.padding = "40px";

  const visualizer = document.createElement("canvas");
  visualizer.className = "audio-visualizer";
  visualizer.id = "audio-visualizer";
  wrapper.appendChild(visualizer);

  const audio = document.createElement("audio");
  audio.className = "projector-audio";
  audio.src = data.source_url;
  audio.autoplay = data.options?.autoplay || false;
  audio.loop = data.options?.loop_content || false;

  audio.addEventListener("loadedmetadata", () => updateTimeDisplay());
  audio.addEventListener("timeupdate", () => {
    updateProgress();
    updateTimeDisplay();
  });
  audio.addEventListener("play", () => {
    projectorState.isPlaying = true;
    document.getElementById("play-pause-btn").textContent = "⏸️";
  });
  audio.addEventListener("pause", () => {
    projectorState.isPlaying = false;
    document.getElementById("play-pause-btn").textContent = "▶️";
  });

  wrapper.appendChild(audio);

  clearContent(container);
  container.appendChild(wrapper);

  showControls("media");
}

// Load Image
function loadImage(container, data) {
  const img = document.createElement("img");
  img.className = "projector-image";
  img.src = data.source_url;
  img.alt = data.title || "Image";
  img.id = "projector-img";

  img.addEventListener("load", () => {
    document.getElementById("projector-loading").classList.add("hidden");
  });

  img.addEventListener("error", () => {
    showError("Failed to load image");
  });

  clearContent(container);
  container.appendChild(img);
  document.getElementById("prev-image-btn").style.display =
    projectorState.totalImages > 1 ? "block" : "none";
  document.getElementById("next-image-btn").style.display =
    projectorState.totalImages > 1 ? "block" : "none";

  showControls("image");
  updateImageInfo();
}

// Load PDF
function loadPdf(container, data) {
  const iframe = document.createElement("iframe");
  iframe.className = "projector-pdf";
  iframe.src = `/static/pdfjs/web/viewer.html?file=${encodeURIComponent(data.source_url)}`;

  clearContent(container);
  container.appendChild(iframe);

  showControls("slide");
}

// Load Presentation
function loadPresentation(container, data) {
  const wrapper = document.createElement("div");
  wrapper.className = "projector-presentation";

  const slideContainer = document.createElement("div");
  slideContainer.className = "slide-container";
  slideContainer.id = "slide-container";

  const slideImg = document.createElement("img");
  slideImg.className = "slide-content";
  slideImg.id = "slide-content";
  slideImg.src = `${data.source_url}?slide=1`;

  slideContainer.appendChild(slideImg);
  wrapper.appendChild(slideContainer);

  clearContent(container);
  container.appendChild(wrapper);

  showControls("slide");
  updateSlideInfo();
}

// Load Code
function loadCode(container, data) {
  const wrapper = document.createElement("div");
  wrapper.className = "projector-code";
  wrapper.id = "code-container";
  if (projectorState.lineNumbers) {
    wrapper.classList.add("line-numbers");
  }

  const pre = document.createElement("pre");
  const code = document.createElement("code");

  fetch(data.source_url)
    .then((res) => res.text())
    .then((text) => {
      const lines = text
        .split("\n")
        .map((line) => `<span class="line">${escapeHtml(line)}</span>`)
        .join("\n");
      code.innerHTML = lines;

      if (window.Prism) {
        Prism.highlightElement(code);
      }
    })
    .catch(() => {
      code.textContent = "Failed to load code";
    });

  pre.appendChild(code);
  wrapper.appendChild(pre);

  clearContent(container);
  container.appendChild(wrapper);
  const filename = data.source_url.split("/").pop();
  document.getElementById("code-info").textContent = filename;

  showControls("code");
}

// Load Markdown
function loadMarkdown(container, data) {
  const wrapper = document.createElement("div");
  wrapper.className = "projector-markdown";

  fetch(data.source_url)
    .then((res) => res.text())
    .then((text) => {
      wrapper.innerHTML = parseMarkdown(text);
    })
    .catch(() => {
      wrapper.innerHTML = "<p>Failed to load markdown</p>";
    });

  clearContent(container);
  container.appendChild(wrapper);

  hideAllControls();
}

// Load Iframe
function loadIframe(container, data) {
  const iframe = document.createElement("iframe");
  iframe.className = "projector-iframe";
  iframe.src = data.source_url;
  iframe.allow =
    "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture";
  iframe.allowFullscreen = true;

  clearContent(container);
  container.appendChild(iframe);

  hideAllControls();
}

// Load Generic
function loadGeneric(container, data) {
  const wrapper = document.createElement("div");
  wrapper.style.textAlign = "center";
  wrapper.style.padding = "40px";
  wrapper.style.color = "#888";

  wrapper.innerHTML = `
        <div style="font-size: 64px; margin-bottom: 20px;">📁</div>
        <div style="font-size: 18px; margin-bottom: 10px;">Cannot preview this file type</div>
        <a href="${data.source_url}" download style="color: #667eea; text-decoration: none;">
            ⬇️ Download File
        </a>
    `;

  clearContent(container);
  container.appendChild(wrapper);

  hideAllControls();
}

// Show Error
function showError(message) {
  const content = document.getElementById("projector-content");
  content.innerHTML = `
        <div class="projector-error">
            <span class="projector-error-icon">❌</span>
            <span class="projector-error-message">${message}</span>
        </div>
    `;
}

// Clear Content
function clearContent(container) {
  const loading = document.getElementById("projector-loading");
  container.innerHTML = "";
  container.appendChild(loading);
}

// Show/Hide Controls
function showControls(type) {
  hideAllControls();
  const controls = document.getElementById(`${type}-controls`);
  if (controls) {
    controls.classList.remove("hidden");
  }
}

function hideAllControls() {
  document.getElementById("media-controls")?.classList.add("hidden");
  document.getElementById("slide-controls")?.classList.add("hidden");
  document.getElementById("image-controls")?.classList.add("hidden");
  document.getElementById("code-controls")?.classList.add("hidden");
}

// Close Projector
function closeProjector() {
  const overlay = document.getElementById("projector-overlay");
  overlay.classList.add("hidden");
  projectorState.isOpen = false;

  const media = getMediaElement();
  if (media) {
    media.pause();
    media.src = "";
  }

  const content = document.getElementById("projector-content");
  const loading = document.getElementById("projector-loading");
  content.innerHTML = "";
  content.appendChild(loading);
}

function closeProjectorOnOverlay(event) {
  if (event.target.id === "projector-overlay") {
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
    document.getElementById("mute-btn").textContent = value === 0 ? "🔇" : "🔊";
  }
}

function toggleMute() {
  const media = getMediaElement();
  if (media) {
    media.muted = !media.muted;
    projectorState.isMuted = media.muted;
    document.getElementById("mute-btn").textContent = media.muted ? "🔇" : "🔊";
  }
}

function toggleLoop() {
  const media = getMediaElement();
  if (media) {
    media.loop = !media.loop;
    projectorState.isLooping = media.loop;
    document.getElementById("loop-btn").classList.toggle("active", media.loop);
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
    document.getElementById("progress-bar").value = progress;
  }
}

function updateTimeDisplay() {
  const media = getMediaElement();
  if (media) {
    const current = formatTime(media.currentTime);
    const duration = formatTime(media.duration || 0);
    document.getElementById("time-display").textContent =
      `${current} / ${duration}`;
  }
}

function formatTime(seconds) {
  if (isNaN(seconds)) return "0:00";
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
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
  const slideContent = document.getElementById("slide-content");
  if (slideContent) {
    slideContent.src = `${projectorState.source}?slide=${projectorState.currentSlide}`;
  }
  updateSlideInfo();
}

function updateSlideInfo() {
  document.getElementById("slide-info").textContent =
    `Slide ${projectorState.currentSlide} of ${projectorState.totalSlides}`;
  document.getElementById("slide-input").value = projectorState.currentSlide;
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
  updateImageInfo();
}

function updateImageInfo() {
  document.getElementById("image-info").textContent =
    `${projectorState.currentImage + 1} of ${projectorState.totalImages}`;
}

function rotateImage() {
  projectorState.rotation = (projectorState.rotation + 90) % 360;
  const img = document.getElementById("projector-img");
  if (img) {
    img.style.transform = `rotate(${projectorState.rotation}deg) scale(${projectorState.zoom / 100})`;
  }
}

function fitToScreen() {
  projectorState.zoom = 100;
  projectorState.rotation = 0;
  const img = document.getElementById("projector-img");
  if (img) {
    img.style.transform = "none";
  }
  document.getElementById("zoom-level").textContent = "100%";
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
  const img = document.getElementById("projector-img");
  const slideContainer = document.getElementById("slide-container");

  if (img) {
    img.style.transform = `rotate(${projectorState.rotation}deg) scale(${projectorState.zoom / 100})`;
  }
  if (slideContainer) {
    slideContainer.style.transform = `scale(${projectorState.zoom / 100})`;
  }

  document.getElementById("zoom-level").textContent = `${projectorState.zoom}%`;
}

// Code Controls
function toggleLineNumbers() {
  projectorState.lineNumbers = !projectorState.lineNumbers;
  const container = document.getElementById("code-container");
  if (container) {
    container.classList.toggle("line-numbers", projectorState.lineNumbers);
  }
}

function toggleWordWrap() {
  projectorState.wordWrap = !projectorState.wordWrap;
  const container = document.getElementById("code-container");
  if (container) {
    container.style.whiteSpace = projectorState.wordWrap ? "pre-wrap" : "pre";
  }
}

function setCodeTheme(theme) {
  const container = document.getElementById("code-container");
  if (container) {
    container.className = `projector-code ${projectorState.lineNumbers ? "line-numbers" : ""} theme-${theme}`;
  }
}

function copyCode() {
  const code = document.querySelector(".projector-code code");
  if (code) {
    navigator.clipboard.writeText(code.textContent).then(() => {
      const btn = document.querySelector(
        ".code-controls .control-btn:last-child",
      );
      const originalText = btn.textContent;
      btn.textContent = "✅";
      setTimeout(() => (btn.textContent = originalText), 2000);
    });
  }
}

// Fullscreen
function toggleFullscreen() {
  const container = document.querySelector(".projector-container");
  const icon = document.getElementById("fullscreen-icon");

  if (!document.fullscreenElement) {
    container
      .requestFullscreen()
      .then(() => {
        container.classList.add("fullscreen");
        icon.textContent = "⛶";
      })
      .catch(() => {});
  } else {
    document
      .exitFullscreen()
      .then(() => {
        container.classList.remove("fullscreen");
        icon.textContent = "⛶";
      })
      .catch(() => {});
  }
}

// Download
function downloadContent() {
  const link = document.createElement("a");
  link.href = projectorState.source;
  link.download = "";
  link.click();
}

// Share
function shareContent() {
  if (navigator.share) {
    navigator
      .share({
        title: document.getElementById("projector-title").textContent,
        url: projectorState.source,
      })
      .catch(() => {});
  } else {
    navigator.clipboard
      .writeText(window.location.origin + projectorState.source)
      .then(() => {
        alert("Link copied to clipboard!");
      });
  }
}

// Keyboard shortcuts for projector
document.addEventListener("keydown", (e) => {
  if (!projectorState.isOpen) return;

  switch (e.key) {
    case "Escape":
      closeProjector();
      break;
    case " ":
      e.preventDefault();
      togglePlayPause();
      break;
    case "ArrowLeft":
      if (
        projectorState.contentType === "Video" ||
        projectorState.contentType === "Audio"
      ) {
        mediaSeekBack();
      } else {
        prevSlide();
      }
      break;
    case "ArrowRight":
      if (
        projectorState.contentType === "Video" ||
        projectorState.contentType === "Audio"
      ) {
        mediaSeekForward();
      } else {
        nextSlide();
      }
      break;
    case "f":
      toggleFullscreen();
      break;
    case "m":
      toggleMute();
      break;
    case "+":
    case "=":
      zoomIn();
      break;
    case "-":
      zoomOut();
      break;
  }
});
function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

function parseMarkdown(text) {
  return text
    .replace(/^### (.*$)/gim, "<h3>$1</h3>")
    .replace(/^## (.*$)/gim, "<h2>$1</h2>")
    .replace(/^# (.*$)/gim, "<h1>$1</h1>")
    .replace(/\*\*(.*)\*\*/gim, "<strong>$1</strong>")
    .replace(/\*(.*)\*/gim, "<em>$1</em>")
    .replace(/`([^`]+)`/gim, "<code>$1</code>")
    .replace(/\n/gim, "<br>");
}
// Export projector functions for onclick handlers in projector.html
window.openProjector = openProjector;
window.closeProjector = closeProjector;
window.closeProjectorOnOverlay = closeProjectorOnOverlay;
window.toggleFullscreen = toggleFullscreen;
window.downloadContent = downloadContent;
window.shareContent = shareContent;
window.togglePlayPause = togglePlayPause;
window.mediaSeekBack = mediaSeekBack;
window.mediaSeekForward = mediaSeekForward;
window.toggleMute = toggleMute;
window.setVolume = setVolume;
window.toggleLoop = toggleLoop;
window.prevSlide = prevSlide;
window.nextSlide = nextSlide;
window.goToSlide = goToSlide;
window.zoomIn = zoomIn;
window.zoomOut = zoomOut;
window.prevImage = prevImage;
window.nextImage = nextImage;
window.rotateImage = rotateImage;
window.fitToScreen = fitToScreen;
window.toggleLineNumbers = toggleLineNumbers;
window.toggleWordWrap = toggleWordWrap;
window.setCodeTheme = setCodeTheme;
window.copyCode = copyCode;

if (window.htmx) {
  htmx.on("htmx:wsMessage", function (event) {
    try {
      const data = JSON.parse(event.detail.message);
      if (data.type === "play") {
        openProjector(data.data);
      } else if (data.type === "player_command") {
        switch (data.command) {
          case "stop":
            closeProjector();
            break;
          case "pause":
            const media = getMediaElement();
            if (media) media.pause();
            break;
          case "resume":
            const mediaR = getMediaElement();
            if (mediaR) mediaR.play();
            break;
        }
      }
    } catch (e) {}
  });
}
