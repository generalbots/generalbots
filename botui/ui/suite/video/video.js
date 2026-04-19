class VideoEditor {
  constructor() {
    this.projectId = null;
    this.project = null;
    this.clips = [];
    this.layers = [];
    this.audioTracks = [];
    this.playheadMs = 0;
    this.totalDurationMs = 0;
    this.isPlaying = false;
    this.selection = { type: "none" };
    this.zoomLevel = 5;
    this.pixelsPerMs = 0.1;
    this.undoStack = [];
    this.redoStack = [];
    this.driveSource = null;

    this.init();
  }

  async init() {
    this.bindEvents();
    this.updateTimeRuler();
    await this.loadFromUrlParams();
    await this.loadProjects();
  }

  async loadFromUrlParams() {
    const urlParams = new URLSearchParams(window.location.search);
    const hash = window.location.hash;
    let bucket = urlParams.get("bucket");
    let path = urlParams.get("path");

    if (hash) {
      const hashQueryIndex = hash.indexOf("?");
      if (hashQueryIndex !== -1) {
        const hashParams = new URLSearchParams(
          hash.substring(hashQueryIndex + 1),
        );
        bucket = bucket || hashParams.get("bucket");
        path = path || hashParams.get("path");
      }
    }

    if (bucket && path) {
      await this.loadFromDrive(bucket, path);
    }
  }

  async loadFromDrive(bucket, path) {
    const fileName = path.split("/").pop() || "media";
    const ext = fileName.split(".").pop().toLowerCase();

    this.driveSource = { bucket, path };

    try {
      const response = await fetch("/api/files/download", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ bucket, path }),
      });

      if (!response.ok) {
        throw new Error(`Failed to load file: ${response.status}`);
      }

      const blob = await response.blob();
      const url = URL.createObjectURL(blob);

      const isImage = [
        "png",
        "jpg",
        "jpeg",
        "gif",
        "webp",
        "svg",
        "bmp",
        "ico",
        "tiff",
        "tif",
        "heic",
        "heif",
      ].includes(ext);
      const isVideo = [
        "mp4",
        "webm",
        "mov",
        "avi",
        "mkv",
        "wmv",
        "flv",
        "m4v",
      ].includes(ext);

      const previewEl = document.getElementById("preview-video");
      if (previewEl) {
        if (isImage) {
          previewEl.outerHTML = `<img id="preview-video" src="${url}" style="max-width:100%;max-height:100%;object-fit:contain;" alt="${fileName}">`;
        } else if (isVideo) {
          previewEl.src = url;
          previewEl.load();
        }
      }

      const projectName = document.getElementById("current-project-name");
      if (projectName) {
        projectName.textContent = fileName;
      }
    } catch (err) {
      console.error("Failed to load file from drive:", err);
    }
  }

  bindEvents() {
    document
      .getElementById("new-project-btn")
      ?.addEventListener("click", () => this.showNewProjectModal());
    document
      .getElementById("create-project")
      ?.addEventListener("click", () => this.createProject());
    document
      .getElementById("cancel-new-project")
      ?.addEventListener("click", () => this.hideModal("new-project-modal"));
    document
      .getElementById("close-new-project-modal")
      ?.addEventListener("click", () => this.hideModal("new-project-modal"));

    document
      .getElementById("btn-export")
      ?.addEventListener("click", () => this.showExportModal());
    document
      .getElementById("start-export")
      ?.addEventListener("click", () => this.startExport());
    document
      .getElementById("cancel-export")
      ?.addEventListener("click", () => this.hideModal("export-modal"));
    document
      .getElementById("close-export-modal")
      ?.addEventListener("click", () => this.hideModal("export-modal"));

    document
      .getElementById("btn-undo")
      ?.addEventListener("click", () => this.undo());
    document
      .getElementById("btn-redo")
      ?.addEventListener("click", () => this.redo());
    document
      .getElementById("btn-delete")
      ?.addEventListener("click", () => this.deleteSelected());
    document
      .getElementById("btn-split")
      ?.addEventListener("click", () => this.splitAtPlayhead());

    document
      .getElementById("btn-play-pause")
      ?.addEventListener("click", () => this.togglePlayback());
    document
      .getElementById("btn-preview")
      ?.addEventListener("click", () => this.togglePlayback());

    document
      .getElementById("zoom-slider")
      ?.addEventListener("input", (e) =>
        this.setZoom(parseInt(e.target.value)),
      );
    document
      .getElementById("btn-zoom-in")
      ?.addEventListener("click", () => this.setZoom(this.zoomLevel + 1));
    document
      .getElementById("btn-zoom-out")
      ?.addEventListener("click", () => this.setZoom(this.zoomLevel - 1));

    document
      .getElementById("volume-slider")
      ?.addEventListener("input", (e) =>
        this.setVolume(parseInt(e.target.value)),
      );

    document
      .getElementById("project-name")
      ?.addEventListener("change", (e) =>
        this.updateProjectName(e.target.value),
      );

    document.querySelectorAll(".element-btn").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        const action = e.currentTarget.dataset.action;
        if (e.currentTarget.classList.contains("ai-btn")) {
          this.handleAITool(action);
        } else {
          this.handleAddElement(action);
        }
      });
    });

    document
      .getElementById("confirm-add-text")
      ?.addEventListener("click", () => this.addTextLayer());
    document
      .getElementById("cancel-add-text")
      ?.addEventListener("click", () => this.hideModal("add-text-modal"));
    document
      .getElementById("close-add-text-modal")
      ?.addEventListener("click", () => this.hideModal("add-text-modal"));

    document
      .getElementById("btn-send-chat")
      ?.addEventListener("click", () => this.sendChatMessage());
    document.getElementById("chat-input")?.addEventListener("keypress", (e) => {
      if (e.key === "Enter") this.sendChatMessage();
    });

    document
      .getElementById("toggle-chat")
      ?.addEventListener("click", () => this.toggleChatPanel());

    document
      .getElementById("timeline-body")
      ?.addEventListener("click", (e) => this.handleTimelineClick(e));

    document.querySelectorAll(".aspect-btn").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        document
          .querySelectorAll(".aspect-btn")
          .forEach((b) => b.classList.remove("active"));
        e.currentTarget.classList.add("active");
      });
    });

    document.addEventListener("keydown", (e) => this.handleKeyboard(e));
  }

  handleKeyboard(e) {
    if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA") return;

    if (e.ctrlKey || e.metaKey) {
      switch (e.key.toLowerCase()) {
        case "z":
          e.preventDefault();
          if (e.shiftKey) this.redo();
          else this.undo();
          break;
        case "y":
          e.preventDefault();
          this.redo();
          break;
        case "s":
          e.preventDefault();
          this.saveProject();
          break;
      }
    } else {
      switch (e.key) {
        case " ":
          e.preventDefault();
          this.togglePlayback();
          break;
        case "Delete":
        case "Backspace":
          if (this.selection.type !== "none") {
            e.preventDefault();
            this.deleteSelected();
          }
          break;
        case "s":
        case "S":
          this.splitAtPlayhead();
          break;
        case "ArrowLeft":
          this.movePlayhead(-1000);
          break;
        case "ArrowRight":
          this.movePlayhead(1000);
          break;
      }
    }
  }

  async loadProjects() {
    try {
      const response = await fetch("/api/video/projects");
      const data = await response.json();
      this.renderProjectList(data.projects || []);
    } catch (error) {
      console.error("Failed to load projects:", error);
    }
  }

  renderProjectList(projects) {
    const container = document.getElementById("project-list");
    if (!container) return;

    if (projects.length === 0) {
      container.innerHTML = `
                <div class="empty-state">
                    <span data-i18n="video.no_projects">No projects yet</span>
                    <p data-i18n="video.create_first">Create your first video project</p>
                </div>
            `;
      return;
    }

    container.innerHTML = projects
      .map(
        (p) => `
            <div class="project-item" data-id="${p.id}">
                <div class="project-thumbnail">
                    ${p.thumbnail_url ? `<img src="${p.thumbnail_url}" alt="" />` : "<span>🎬</span>"}
                </div>
                <div class="project-info">
                    <span class="project-title">${this.escapeHtml(p.name)}</span>
                    <span class="project-meta">${this.formatDuration(p.total_duration_ms)} • ${p.clips_count} clips</span>
                </div>
            </div>
        `,
      )
      .join("");

    container.querySelectorAll(".project-item").forEach((item) => {
      item.addEventListener("click", () => this.loadProject(item.dataset.id));
    });
  }

  async loadProject(projectId) {
    try {
      const response = await fetch(`/api/video/projects/${projectId}`);
      const data = await response.json();

      this.projectId = projectId;
      this.project = data.project;
      this.clips = data.clips || [];
      this.layers = data.layers || [];
      this.audioTracks = data.audio_tracks || [];
      this.playheadMs = data.project.playhead_ms || 0;
      this.totalDurationMs = data.project.total_duration_ms || 0;

      this.updateUI();
      this.renderTimeline();
      this.renderPreview();
    } catch (error) {
      console.error("Failed to load project:", error);
      this.showNotification("Failed to load project", "error");
    }
  }

  updateUI() {
    if (!this.project) return;

    document.getElementById("project-name").value = this.project.name;
    document.getElementById("project-status").textContent = this.project.status;
    document.getElementById("stat-duration").textContent = this.formatDuration(
      this.totalDurationMs,
    );
    document.getElementById("stat-clips").textContent = this.clips.length;
    document.getElementById("stat-layers").textContent = this.layers.length;
    document.getElementById("stat-resolution").textContent =
      `${this.project.resolution_width}x${this.project.resolution_height}`;

    this.updateTimeDisplay();
    this.updateContextDisplay();
  }

  updateTimeDisplay() {
    document.getElementById("current-time").textContent = this.formatTime(
      this.playheadMs,
    );
    document.getElementById("total-time").textContent = this.formatTime(
      this.totalDurationMs,
    );
    document.getElementById("context-playhead").textContent =
      this.formatDuration(this.playheadMs);
  }

  updateContextDisplay() {
    const selectionContainer = document.getElementById(
      "context-selection-container",
    );
    const selectionValue = document.getElementById("context-selection");

    if (this.selection.type === "none") {
      selectionContainer.style.display = "none";
    } else {
      selectionContainer.style.display = "flex";
      switch (this.selection.type) {
        case "clip":
          const clip = this.clips.find((c) => c.id === this.selection.id);
          selectionValue.textContent = clip ? clip.name : "Clip";
          break;
        case "layer":
          const layer = this.layers.find((l) => l.id === this.selection.id);
          selectionValue.textContent = layer ? layer.name : "Layer";
          break;
        default:
          selectionValue.textContent = this.selection.type;
      }
    }
  }

  renderTimeline() {
    this.renderVideoTrack();
    this.renderLayersTrack();
    this.renderAudioTrack();
    this.updatePlayheadPosition();
  }

  renderVideoTrack() {
    const track = document.getElementById("video-track");
    if (!track) return;

    track.innerHTML = this.clips
      .map(
        (clip) => `
            <div class="timeline-block clip-block ${this.selection.type === "clip" && this.selection.id === clip.id ? "selected" : ""}"
                 data-id="${clip.id}"
                 data-type="clip"
                 style="left: ${this.msToPixels(clip.start_ms)}px; width: ${this.msToPixels(clip.duration_ms)}px;">
                <div class="resize-handle left"></div>
                <span class="block-label">${this.escapeHtml(clip.name)}</span>
                <div class="resize-handle right"></div>
            </div>
        `,
      )
      .join("");

    this.bindBlockEvents(track);
  }

  renderLayersTrack() {
    const track = document.getElementById("layers-track");
    if (!track) return;

    track.innerHTML = this.layers
      .map(
        (layer) => `
            <div class="timeline-block layer-block ${this.selection.type === "layer" && this.selection.id === layer.id ? "selected" : ""}"
                 data-id="${layer.id}"
                 data-type="layer"
                 style="left: ${this.msToPixels(layer.start_ms)}px; width: ${this.msToPixels(layer.end_ms - layer.start_ms)}px; background: ${this.getLayerColor(layer.layer_type)};">
                <div class="resize-handle left"></div>
                <span class="block-label">${this.escapeHtml(layer.name)}</span>
                <div class="resize-handle right"></div>
            </div>
        `,
      )
      .join("");

    this.bindBlockEvents(track);
  }

  renderAudioTrack() {
    const track = document.getElementById("audio-track");
    if (!track) return;

    track.innerHTML = this.audioTracks
      .map(
        (audio) => `
            <div class="timeline-block audio-block"
                 data-id="${audio.id}"
                 data-type="audio"
                 style="left: ${this.msToPixels(audio.start_ms)}px; width: ${this.msToPixels(audio.duration_ms)}px; background: #9b59b6;">
                <div class="resize-handle left"></div>
                <span class="block-label">${this.escapeHtml(audio.name)}</span>
                <div class="resize-handle right"></div>
            </div>
        `,
      )
      .join("");

    this.bindBlockEvents(track);
  }

  bindBlockEvents(track) {
    track.querySelectorAll(".timeline-block").forEach((block) => {
      block.addEventListener("click", (e) => {
        e.stopPropagation();
        this.selectItem(block.dataset.type, block.dataset.id);
      });

      block.addEventListener("mousedown", (e) => {
        if (e.target.classList.contains("resize-handle")) {
          this.startResize(
            block,
            e.target.classList.contains("left") ? "left" : "right",
            e,
          );
        } else {
          this.startDrag(block, e);
        }
      });
    });
  }

  startDrag(block, e) {
    const startX = e.clientX;
    const startLeft = parseInt(block.style.left) || 0;

    const onMouseMove = (moveEvent) => {
      const deltaX = moveEvent.clientX - startX;
      const newLeft = Math.max(0, startLeft + deltaX);
      block.style.left = `${newLeft}px`;
    };

    const onMouseUp = async () => {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);

      const newStartMs = this.pixelsToMs(parseInt(block.style.left));
      await this.updateItemPosition(
        block.dataset.type,
        block.dataset.id,
        newStartMs,
      );
    };

    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
  }

  startResize(block, side, e) {
    const startX = e.clientX;
    const startLeft = parseInt(block.style.left) || 0;
    const startWidth = parseInt(block.style.width) || 100;

    const onMouseMove = (moveEvent) => {
      const deltaX = moveEvent.clientX - startX;

      if (side === "left") {
        const newLeft = Math.max(0, startLeft + deltaX);
        const newWidth = startWidth - deltaX;
        if (newWidth > 20) {
          block.style.left = `${newLeft}px`;
          block.style.width = `${newWidth}px`;
        }
      } else {
        const newWidth = Math.max(20, startWidth + deltaX);
        block.style.width = `${newWidth}px`;
      }
    };

    const onMouseUp = async () => {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);

      const newStartMs = this.pixelsToMs(parseInt(block.style.left));
      const newDurationMs = this.pixelsToMs(parseInt(block.style.width));
      await this.updateItemTiming(
        block.dataset.type,
        block.dataset.id,
        newStartMs,
        newDurationMs,
      );
    };

    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
    e.stopPropagation();
  }

  async updateItemPosition(type, id, startMs) {
    if (!this.projectId) return;

    try {
      if (type === "clip") {
        await fetch(`/api/video/clips/${id}`, {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ start_ms: startMs }),
        });
      } else if (type === "layer") {
        const layer = this.layers.find((l) => l.id === id);
        const duration = layer ? layer.end_ms - layer.start_ms : 5000;
        await fetch(`/api/video/layers/${id}`, {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            start_ms: startMs,
            end_ms: startMs + duration,
          }),
        });
      }
      await this.loadProject(this.projectId);
    } catch (error) {
      console.error("Failed to update position:", error);
    }
  }

  async updateItemTiming(type, id, startMs, durationMs) {
    if (!this.projectId) return;

    try {
      if (type === "clip") {
        await fetch(`/api/video/clips/${id}`, {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ start_ms: startMs, duration_ms: durationMs }),
        });
      } else if (type === "layer") {
        await fetch(`/api/video/layers/${id}`, {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            start_ms: startMs,
            end_ms: startMs + durationMs,
          }),
        });
      }
      await this.loadProject(this.projectId);
    } catch (error) {
      console.error("Failed to update timing:", error);
    }
  }

  selectItem(type, id) {
    this.selection = { type, id };
    this.updateContextDisplay();
    this.renderTimeline();
    this.showPropertiesPanel(type, id);
  }

  clearSelection() {
    this.selection = { type: "none" };
    this.updateContextDisplay();
    this.renderTimeline();
    document.getElementById("properties-panel").style.display = "none";
  }

  showPropertiesPanel(type, id) {
    const panel = document.getElementById("properties-panel");
    const content = document.getElementById("properties-content");
    if (!panel || !content) return;

    let item;
    if (type === "clip") {
      item = this.clips.find((c) => c.id === id);
    } else if (type === "layer") {
      item = this.layers.find((l) => l.id === id);
    }

    if (!item) return;

    panel.style.display = "block";
    content.innerHTML = this.renderPropertiesForm(type, item);

    content.querySelectorAll("input, select").forEach((input) => {
      input.addEventListener("change", () => this.saveProperties(type, id));
    });
  }

  renderPropertiesForm(type, item) {
    if (type === "clip") {
      return `
                <div class="form-group">
                    <label>Name</label>
                    <input type="text" id="prop-name" value="${this.escapeHtml(item.name)}" />
                </div>
                <div class="form-group">
                    <label>Volume</label>
                    <input type="range" id="prop-volume" min="0" max="100" value="${item.volume * 100}" />
                </div>
                <div class="form-group">
                    <label>Transition In</label>
                    <select id="prop-transition-in">
                        <option value="">None</option>
                        <option value="fade" ${item.transition_in === "fade" ? "selected" : ""}>Fade</option>
                        <option value="dissolve" ${item.transition_in === "dissolve" ? "selected" : ""}>Dissolve</option>
                        <option value="wipe" ${item.transition_in === "wipe" ? "selected" : ""}>Wipe</option>
                    </select>
                </div>
            `;
    } else if (type === "layer") {
      return `
                <div class="form-group">
                    <label>Name</label>
                    <input type="text" id="prop-name" value="${this.escapeHtml(item.name)}" />
                </div>
                <div class="form-group">
                    <label>Opacity</label>
                    <input type="range" id="prop-opacity" min="0" max="100" value="${item.opacity * 100}" />
                </div>
                <div class="form-group">
                    <label>Position X</label>
                    <input type="range" id="prop-x" min="0" max="100" value="${item.x * 100}" />
                </div>
                <div class="form-group">
                    <label>Position Y</label>
                    <input type="range" id="prop-y" min="0" max="100" value="${item.y * 100}" />
                </div>
                <div class="form-group">
                    <label>Rotation</label>
                    <input type="range" id="prop-rotation" min="-180" max="180" value="${item.rotation}" />
                </div>
            `;
    }
    return "";
  }

  async saveProperties(type, id) {
    if (!this.projectId) return;

    try {
      if (type === "clip") {
        await fetch(`/api/video/clips/${id}`, {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            name: document.getElementById("prop-name")?.value,
            volume:
              parseFloat(document.getElementById("prop-volume")?.value || 100) /
              100,
            transition_in:
              document.getElementById("prop-transition-in")?.value || null,
          }),
        });
      } else if (type === "layer") {
        await fetch(`/api/video/layers/${id}`, {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            name: document.getElementById("prop-name")?.value,
            opacity:
              parseFloat(
                document.getElementById("prop-opacity")?.value || 100,
              ) / 100,
            x: parseFloat(document.getElementById("prop-x")?.value || 50) / 100,
            y: parseFloat(document.getElementById("prop-y")?.value || 50) / 100,
            rotation: parseFloat(
              document.getElementById("prop-rotation")?.value || 0,
            ),
          }),
        });
      }
      await this.loadProject(this.projectId);
    } catch (error) {
      console.error("Failed to save properties:", error);
    }
  }

  renderPreview() {
    const overlays = document.getElementById("layer-overlays");
    if (!overlays) return;

    if (this.clips.length > 0 && !this.isPlaying) {
      this.loadPreviewFrame();
    }

    const visibleLayers = this.layers.filter(
      (l) => this.playheadMs >= l.start_ms && this.playheadMs < l.end_ms,
    );

    overlays.innerHTML = visibleLayers
      .map((layer) => {
        const style = `
                left: ${layer.x * 100}%;
                top: ${layer.y * 100}%;
                width: ${layer.width * 100}%;
                height: ${layer.height * 100}%;
                opacity: ${layer.opacity};
                transform: translate(-50%, -50%) rotate(${layer.rotation}deg);
            `;

        if (layer.layer_type === "text") {
          const props = layer.properties_json || {};
          return `
                    <div class="layer-item ${this.selection.type === "layer" && this.selection.id === layer.id ? "selected" : ""}"
                         data-id="${layer.id}"
                         style="${style}; color: ${props.color || "#fff"}; font-size: ${props.font_size || 48}px; font-family: ${props.font_family || "Arial"};">
                        ${this.escapeHtml(props.content || "")}
                    </div>
                `;
        }

        return `<div class="layer-item" data-id="${layer.id}" style="${style}"></div>`;
      })
      .join("");
  }

  handleTimelineClick(e) {
    const track = e.target.closest(".track-content");
    if (!track) return;

    if (e.target === track) {
      const rect = track.getBoundingClientRect();
      const x = e.clientX - rect.left;
      this.setPlayhead(this.pixelsToMs(x));
      this.clearSelection();
    }
  }

  setPlayhead(ms) {
    this.playheadMs = Math.max(0, Math.min(ms, this.totalDurationMs));
    this.updatePlayheadPosition();
    this.updateTimeDisplay();
    this.renderPreview();
  }

  movePlayhead(deltaMs) {
    this.setPlayhead(this.playheadMs + deltaMs);
  }

  updatePlayheadPosition() {
    const playhead = document.getElementById("playhead");
    if (playhead) {
      playhead.style.left = `${140 + this.msToPixels(this.playheadMs)}px`;
    }
  }

  togglePlayback() {
    if (this.isPlaying) {
      this.stopPlayback();
    } else {
      this.startPlayback();
    }
  }

  startPlayback() {
    this.isPlaying = true;
    this.updatePlayButton();

    const startTime = Date.now();
    const startMs = this.playheadMs;

    this.playbackInterval = setInterval(() => {
      const elapsed = Date.now() - startTime;
      const newMs = startMs + elapsed;

      if (newMs >= this.totalDurationMs) {
        this.stopPlayback();
        this.setPlayhead(0);
      } else {
        this.setPlayhead(newMs);
      }
    }, 33);
  }

  stopPlayback() {
    this.isPlaying = false;
    this.updatePlayButton();

    if (this.playbackInterval) {
      clearInterval(this.playbackInterval);
      this.playbackInterval = null;
    }
  }

  updatePlayButton() {
    const btn = document.getElementById("btn-play-pause");
    if (btn) {
      btn.innerHTML = this.isPlaying
        ? '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"></rect><rect x="14" y="4" width="4" height="16"></rect></svg>'
        : '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>';
    }
  }

  setZoom(level) {
    this.zoomLevel = Math.max(1, Math.min(10, level));
    this.pixelsPerMs = 0.02 * this.zoomLevel;

    document.getElementById("zoom-slider").value = this.zoomLevel;

    this.updateTimeRuler();
    this.renderTimeline();
  }

  updateTimeRuler() {
    const ruler = document.getElementById("time-ruler");
    if (!ruler) return;

    const duration = Math.max(this.totalDurationMs, 60000);
    const width = this.msToPixels(duration);
    const interval = this.getTimeInterval();

    let html = "";
    for (let ms = 0; ms <= duration; ms += interval) {
      const x = this.msToPixels(ms);
      html += `<div class="time-marker" style="left: ${x}px;">${this.formatDuration(ms)}</div>`;
    }

    ruler.innerHTML = html;
    ruler.style.width = `${width}px`;
  }

  getTimeInterval() {
    if (this.pixelsPerMs > 0.5) return 1000;
    if (this.pixelsPerMs > 0.2) return 5000;
    if (this.pixelsPerMs > 0.1) return 10000;
    return 30000;
  }

  setVolume(volume) {
    console.log("Volume set to:", volume);
  }

  showNewProjectModal() {
    this.showModal("new-project-modal");
    document.getElementById("new-project-name").value = "";
    document.getElementById("new-project-name").focus();
  }

  async createProject() {
    const name =
      document.getElementById("new-project-name")?.value || "Untitled Project";
    const aspectBtn = document.querySelector(".aspect-btn.active");
    const fps = parseInt(
      document.getElementById("new-project-fps")?.value || "30",
    );

    const resolution_width = parseInt(aspectBtn?.dataset.width || "1920");
    const resolution_height = parseInt(aspectBtn?.dataset.height || "1080");

    try {
      const response = await fetch("/api/video/projects", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name,
          resolution_width,
          resolution_height,
          fps,
        }),
      });

      const data = await response.json();
      this.hideModal("new-project-modal");
      await this.loadProject(data.project.id);
      this.showNotification("Project created", "success");
    } catch (error) {
      console.error("Failed to create project:", error);
      this.showNotification("Failed to create project", "error");
    }
  }

  async updateProjectName(name) {
    if (!this.projectId) return;

    try {
      await fetch(`/api/video/projects/${this.projectId}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name }),
      });
    } catch (error) {
      console.error("Failed to update project name:", error);
    }
  }

  handleAddElement(action) {
    switch (action) {
      case "add-clip":
        this.showUploadDialog();
        break;
      case "add-text":
        this.showModal("add-text-modal");
        document.getElementById("text-content").value = "";
        document.getElementById("text-content").focus();
        break;
      case "add-image":
        this.showAddImageDialog();
        break;
      case "add-shape":
        this.addShape();
        break;
      case "add-audio":
        this.showAddAudioDialog();
        break;
      case "add-narration":
        this.showNarrationDialog();
        break;
    }
  }

  showAddClipDialog() {
    const url = prompt("Enter video URL:");
    if (url) {
      this.addClip(url);
    }
  }

  showUploadDialog() {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "video/*";
    input.onchange = async (e) => {
      const file = e.target.files[0];
      if (file) {
        await this.uploadAndAddClip(file);
      }
    };
    input.click();
  }

  async uploadAndAddClip(file) {
    if (!this.projectId) {
      this.showNotification("Please create a project first", "warning");
      return;
    }

    this.showNotification("Uploading video...", "info");

    try {
      const formData = new FormData();
      formData.append("file", file);

      const response = await fetch(
        `/api/video/projects/${this.projectId}/upload`,
        {
          method: "POST",
          body: formData,
        },
      );

      const data = await response.json();

      if (data.file_url) {
        await this.addClip(data.file_url, file.name);
        this.showNotification("Video uploaded and added", "success");
      } else {
        this.showNotification(
          "Upload failed: " + (data.error || "Unknown error"),
          "error",
        );
      }
    } catch (error) {
      console.error("Failed to upload:", error);
      this.showNotification("Failed to upload video", "error");
    }
  }

  async addClip(sourceUrl, name = null) {
    if (!this.projectId) {
      this.showNotification("Please create a project first", "warning");
      return;
    }

    try {
      await fetch(`/api/video/projects/${this.projectId}/clips`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name: name || "Clip",
          source_url: sourceUrl,
          at_ms: this.playheadMs,
          duration_ms: 5000,
        }),
      });
      await this.loadProject(this.projectId);
      this.showNotification("Clip added", "success");
    } catch (error) {
      console.error("Failed to add clip:", error);
      this.showNotification("Failed to add clip", "error");
    }
  }

  async addTextLayer() {
    if (!this.projectId) {
      this.showNotification("Please create a project first", "warning");
      return;
    }

    const content = document.getElementById("text-content")?.value || "Text";
    const fontFamily = document.getElementById("text-font")?.value || "Arial";
    const fontSize = parseInt(
      document.getElementById("text-size")?.value || "48",
    );
    const color = document.getElementById("text-color")?.value || "#FFFFFF";
    const durationSec = parseInt(
      document.getElementById("text-duration")?.value || "5",
    );

    try {
      await fetch(`/api/video/projects/${this.projectId}/layers`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name: content.substring(0, 20),
          layer_type: "text",
          start_ms: this.playheadMs,
          end_ms: this.playheadMs + durationSec * 1000,
          x: 0.5,
          y: 0.8,
          width: 0.8,
          height: 0.1,
          properties: {
            content,
            font_family: fontFamily,
            font_size: fontSize,
            color,
          },
        }),
      });
      this.hideModal("add-text-modal");
      await this.loadProject(this.projectId);
      this.showNotification("Text layer added", "success");
    } catch (error) {
      console.error("Failed to add text layer:", error);
      this.showNotification("Failed to add text layer", "error");
    }
  }

  showAddImageDialog() {
    const url = prompt("Enter image URL:");
    if (url) {
      this.addImageLayer(url);
    }
  }

  async addImageLayer(sourceUrl) {
    if (!this.projectId) return;

    try {
      await fetch(`/api/video/projects/${this.projectId}/layers`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name: "Image",
          layer_type: "image",
          start_ms: this.playheadMs,
          end_ms: this.playheadMs + 5000,
          x: 0.5,
          y: 0.5,
          width: 0.5,
          height: 0.5,
          properties: { source_url: sourceUrl },
        }),
      });
      await this.loadProject(this.projectId);
      this.showNotification("Image added", "success");
    } catch (error) {
      console.error("Failed to add image:", error);
    }
  }

  async addShape() {
    if (!this.projectId) return;

    try {
      await fetch(`/api/video/projects/${this.projectId}/layers`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name: "Rectangle",
          layer_type: "shape",
          start_ms: this.playheadMs,
          end_ms: this.playheadMs + 5000,
          x: 0.5,
          y: 0.5,
          width: 0.3,
          height: 0.2,
          properties: {
            shape_type: "rectangle",
            fill_color: "#3498db",
            stroke_color: "#2980b9",
            stroke_width: 2,
          },
        }),
      });
      await this.loadProject(this.projectId);
      this.showNotification("Shape added", "success");
    } catch (error) {
      console.error("Failed to add shape:", error);
    }
  }

  showAddAudioDialog() {
    const url = prompt("Enter audio URL:");
    if (url) {
      this.addAudioTrack(url, "music");
    }
  }

  showNarrationDialog() {
    const text = prompt("Enter narration text:");
    if (text) {
      this.generateNarration(text);
    }
  }

  async addAudioTrack(sourceUrl, trackType) {
    if (!this.projectId) return;

    try {
      await fetch(`/api/video/projects/${this.projectId}/audio`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name: trackType === "narration" ? "Narration" : "Audio",
          source_url: sourceUrl,
          track_type: trackType,
          start_ms: this.playheadMs,
          duration_ms: 10000,
          volume: 1.0,
        }),
      });
      await this.loadProject(this.projectId);
      this.showNotification("Audio track added", "success");
    } catch (error) {
      console.error("Failed to add audio:", error);
    }
  }

  async generateNarration(text) {
    if (!this.projectId) {
      this.showNotification("Please create a project first", "warning");
      return;
    }

    this.showNotification("Generating narration...", "info");

    try {
      const response = await fetch(
        `/api/video/projects/${this.projectId}/tts`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            text,
            voice: "alloy",
            speed: 1.0,
            language: "en",
          }),
        },
      );

      const data = await response.json();

      if (data.audio_url) {
        await this.loadProject(this.projectId);
        this.showNotification("Narration added!", "success");
      } else {
        this.showNotification(data.error || "TTS failed", "error");
      }
    } catch (error) {
      console.error("TTS error:", error);
      this.showNotification("Failed to generate narration", "error");
    }
  }

  handleAITool(action) {
    switch (action) {
      case "auto-captions":
        this.generateAutoCaptions();
        break;
      case "tts":
        this.showTTSDialog();
        break;
      case "detect-scenes":
        this.detectScenes();
        break;
      case "templates":
        this.showTemplatesDialog();
        break;
      case "reframe":
        this.showReframeDialog();
        break;
      case "transitions":
        this.showTransitionsDialog();
        break;
    }
  }

  async generateAutoCaptions() {
    if (!this.projectId) {
      this.showNotification("Please create a project first", "warning");
      return;
    }

    if (this.clips.length === 0) {
      this.showNotification("Add a video clip first", "warning");
      return;
    }

    this.showNotification(
      "Generating captions... This may take a moment.",
      "info",
    );

    try {
      const response = await fetch(
        `/api/video/projects/${this.projectId}/captions`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            style: "default",
            max_chars_per_line: 40,
            font_size: 32,
            color: "#FFFFFF",
            background: true,
          }),
        },
      );

      const data = await response.json();

      if (data.captions_count) {
        await this.loadProject(this.projectId);
        this.showNotification(
          `Added ${data.captions_count} caption layers!`,
          "success",
        );
      } else {
        this.showNotification(
          data.error || "Caption generation failed",
          "error",
        );
      }
    } catch (error) {
      console.error("Caption error:", error);
      this.showNotification("Failed to generate captions", "error");
    }
  }

  showTTSDialog() {
    const text = prompt("Enter text for narration:");
    if (text && text.trim()) {
      this.generateNarration(text.trim());
    }
  }

  async detectScenes() {
    if (!this.projectId || this.clips.length === 0) {
      this.showNotification("Add a video clip first", "warning");
      return;
    }

    this.showNotification("Detecting scenes...", "info");

    try {
      const response = await fetch(
        `/api/video/projects/${this.projectId}/scenes`,
        { method: "POST" },
      );

      const data = await response.json();

      if (data.scenes && data.scenes.length > 0) {
        this.showNotification(`Found ${data.scenes.length} scenes!`, "success");
        this.showScenesPanel(data.scenes);
      } else {
        this.showNotification("No scene changes detected", "info");
      }
    } catch (error) {
      console.error("Scene detection error:", error);
      this.showNotification("Failed to detect scenes", "error");
    }
  }

  showScenesPanel(scenes) {
    let html = "<h4>Detected Scenes</h4><div class='scenes-list'>";
    scenes.forEach((scene, i) => {
      html += `
        <div class="scene-item" onclick="videoEditor.setPlayhead(${scene.start_ms})">
          <span class="scene-num">${i + 1}</span>
          <span class="scene-time">${this.formatDuration(scene.start_ms)} - ${this.formatDuration(scene.end_ms)}</span>
        </div>
      `;
    });
    html += "</div>";

    const panel = document.getElementById("properties-panel");
    const content = document.getElementById("properties-content");
    if (panel && content) {
      panel.style.display = "block";
      content.innerHTML = html;
    }
  }

  async showTemplatesDialog() {
    try {
      const response = await fetch("/api/video/templates");
      const data = await response.json();

      if (data.templates && data.templates.length > 0) {
        const template = data.templates.find((t) =>
          confirm(`Apply template: ${t.name}?\n${t.description}`),
        );
        if (template) {
          await this.applyTemplate(template.id);
        }
      }
    } catch (error) {
      console.error("Templates error:", error);
      this.showNotification("Failed to load templates", "error");
    }
  }

  async applyTemplate(templateId) {
    if (!this.projectId) {
      this.showNotification("Please create a project first", "warning");
      return;
    }

    const title = prompt("Enter title for template:") || "Title";
    const subtitle = prompt("Enter subtitle (optional):") || "";

    try {
      const response = await fetch(
        `/api/video/projects/${this.projectId}/template`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            template_id: templateId,
            customizations: { title, subtitle },
          }),
        },
      );

      const data = await response.json();

      if (data.success) {
        await this.loadProject(this.projectId);
        this.showNotification("Template applied!", "success");
      } else {
        this.showNotification(
          data.error || "Failed to apply template",
          "error",
        );
      }
    } catch (error) {
      console.error("Template error:", error);
      this.showNotification("Failed to apply template", "error");
    }
  }

  showReframeDialog() {
    if (!this.projectId || this.clips.length === 0) {
      this.showNotification("Add a video clip first", "warning");
      return;
    }

    const aspectRatio = prompt(
      "Select aspect ratio:\n1. 16:9 (Landscape)\n2. 9:16 (Portrait/TikTok)\n3. 1:1 (Square)\n4. 4:5 (Instagram)",
      "2",
    );

    let width, height;
    switch (aspectRatio) {
      case "1":
        width = 1920;
        height = 1080;
        break;
      case "2":
        width = 1080;
        height = 1920;
        break;
      case "3":
        width = 1080;
        height = 1080;
        break;
      case "4":
        width = 1080;
        height = 1350;
        break;
      default:
        return;
    }

    this.autoReframe(width, height);
  }

  async autoReframe(targetWidth, targetHeight) {
    this.showNotification("Auto-reframing video...", "info");

    try {
      const response = await fetch(
        `/api/video/projects/${this.projectId}/reframe`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            target_width: targetWidth,
            target_height: targetHeight,
          }),
        },
      );

      const data = await response.json();

      if (data.reframed_url) {
        this.showNotification(
          "Video reframed! Adding as new clip...",
          "success",
        );
        await this.addClip(data.reframed_url, "Reframed");
      } else {
        this.showNotification(data.error || "Reframe failed", "error");
      }
    } catch (error) {
      console.error("Reframe error:", error);
      this.showNotification("Failed to reframe video", "error");
    }
  }

  showTransitionsDialog() {
    if (this.clips.length < 2) {
      this.showNotification("Need at least 2 clips for transitions", "warning");
      return;
    }

    const transitionType = prompt(
      "Select transition:\n1. fade\n2. dissolve\n3. wipe\n4. slide",
      "1",
    );

    const types = { 1: "fade", 2: "dissolve", 3: "wipe", 4: "slide" };
    const type = types[transitionType];

    if (type) {
      this.addTransitionBetweenClips(type);
    }
  }

  async addTransitionBetweenClips(transitionType) {
    if (this.clips.length < 2) return;

    this.showNotification("Adding transitions...", "info");

    try {
      for (let i = 0; i < this.clips.length - 1; i++) {
        await fetch(
          `/api/video/clips/${this.clips[i].id}/transition/${this.clips[i + 1].id}`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              transition_type: transitionType,
              duration_ms: 500,
            }),
          },
        );
      }

      await this.loadProject(this.projectId);
      this.showNotification("Transitions added!", "success");
    } catch (error) {
      console.error("Transition error:", error);
      this.showNotification("Failed to add transitions", "error");
    }
  }

  async deleteSelected() {
    if (this.selection.type === "none") return;

    try {
      if (this.selection.type === "clip") {
        await fetch(`/api/video/clips/${this.selection.id}`, {
          method: "DELETE",
        });
      } else if (this.selection.type === "layer") {
        await fetch(`/api/video/layers/${this.selection.id}`, {
          method: "DELETE",
        });
      } else if (this.selection.type === "audio") {
        await fetch(`/api/video/audio/${this.selection.id}`, {
          method: "DELETE",
        });
      }

      this.clearSelection();
      await this.loadProject(this.projectId);
      this.showNotification("Item deleted", "success");
    } catch (error) {
      console.error("Failed to delete:", error);
      this.showNotification("Failed to delete", "error");
    }
  }

  async splitAtPlayhead() {
    if (this.selection.type !== "clip") {
      this.showNotification("Select a clip to split", "info");
      return;
    }

    const clip = this.clips.find((c) => c.id === this.selection.id);
    if (!clip) return;

    if (
      this.playheadMs <= clip.start_ms ||
      this.playheadMs >= clip.start_ms + clip.duration_ms
    ) {
      this.showNotification(
        "Playhead must be within the selected clip",
        "warning",
      );
      return;
    }

    try {
      const response = await fetch(
        `/api/video/clips/${this.selection.id}/split`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ at_ms: this.playheadMs }),
        },
      );

      const data = await response.json();

      if (data.first_clip && data.second_clip) {
        this.clearSelection();
        await this.loadProject(this.projectId);
        this.showNotification("Clip split successfully", "success");
      } else {
        this.showNotification(data.error || "Failed to split clip", "error");
      }
    } catch (error) {
      console.error("Failed to split clip:", error);
      this.showNotification("Failed to split clip", "error");
    }
  }

  async loadPreviewFrame() {
    if (!this.projectId || this.clips.length === 0) return;

    try {
      const response = await fetch(
        `/api/video/projects/${this.projectId}/preview?at_ms=${this.playheadMs}&width=640&height=360`,
      );
      const data = await response.json();

      if (data.preview_url) {
        const canvas = document.getElementById("preview-canvas");
        const placeholder = canvas?.querySelector(".preview-placeholder");
        if (placeholder) {
          placeholder.innerHTML = `<img src="${data.preview_url}" alt="Preview" style="max-width: 100%; max-height: 100%;" />`;
        }
      }
    } catch (error) {
      console.log("Preview not available:", error);
    }
  }

  undo() {
    if (this.undoStack.length === 0) return;
    const state = this.undoStack.pop();
    this.redoStack.push(this.getCurrentState());
    this.restoreState(state);
  }

  redo() {
    if (this.redoStack.length === 0) return;
    const state = this.redoStack.pop();
    this.undoStack.push(this.getCurrentState());
    this.restoreState(state);
  }

  getCurrentState() {
    return {
      clips: [...this.clips],
      layers: [...this.layers],
      audioTracks: [...this.audioTracks],
    };
  }

  restoreState(state) {
    this.clips = state.clips;
    this.layers = state.layers;
    this.audioTracks = state.audioTracks;
    this.renderTimeline();
    this.renderPreview();
  }

  saveState() {
    this.undoStack.push(this.getCurrentState());
    this.redoStack = [];
  }

  async saveProject() {
    if (!this.projectId) return;
    this.showNotification("Project saved", "success");
  }

  showExportModal() {
    if (!this.projectId) {
      this.showNotification("Please create a project first", "warning");
      return;
    }
    this.showModal("export-modal");
    document.getElementById("export-progress").style.display = "none";
  }

  async startExport() {
    const format = document.getElementById("export-format")?.value || "mp4";
    const quality = document.getElementById("export-quality")?.value || "high";

    try {
      document.getElementById("export-progress").style.display = "block";
      document.getElementById("export-progress-text").textContent =
        "Starting export...";

      const response = await fetch(
        `/api/video/projects/${this.projectId}/export`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ format, quality }),
        },
      );

      const data = await response.json();
      this.pollExportStatus(data.export.id);
    } catch (error) {
      console.error("Failed to start export:", error);
      this.showNotification("Failed to start export", "error");
      document.getElementById("export-progress").style.display = "none";
    }
  }

  async pollExportStatus(exportId) {
    const poll = async () => {
      try {
        const response = await fetch(`/api/video/exports/${exportId}/status`);
        const data = await response.json();

        document.getElementById("export-progress-fill").style.width =
          `${data.progress}%`;
        document.getElementById("export-progress-text").textContent =
          `${data.status}: ${data.progress}%`;

        if (data.status === "completed") {
          this.showNotification("Export complete!", "success");
          if (data.output_url) {
            window.open(data.output_url, "_blank");
          }
          this.hideModal("export-modal");
        } else if (data.status === "failed") {
          this.showNotification(
            "Export failed: " + (data.error_message || "Unknown error"),
            "error",
          );
          document.getElementById("export-progress").style.display = "none";
        } else {
          setTimeout(poll, 2000);
        }
      } catch (error) {
        console.error("Failed to poll export status:", error);
      }
    };

    poll();
  }

  async sendChatMessage() {
    const input = document.getElementById("chat-input");
    const message = input?.value?.trim();
    if (!message || !this.projectId) return;

    input.value = "";

    this.addChatMessage(message, "user");

    try {
      const response = await fetch(
        `/api/video/projects/${this.projectId}/chat`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            message,
            playhead_ms: this.playheadMs,
            selection: this.selection,
          }),
        },
      );

      const data = await response.json();

      if (data.success) {
        this.addChatMessage(data.message, "assistant");
        if (data.project) {
          await this.loadProject(this.projectId);
        }
      } else {
        this.addChatMessage(
          "Sorry, I could not process that request.",
          "assistant",
        );
      }
    } catch (error) {
      console.error("Chat error:", error);
      this.addChatMessage("Sorry, something went wrong.", "assistant");
    }
  }

  addChatMessage(text, role) {
    const container = document.getElementById("chat-messages");
    if (!container) return;

    const msg = document.createElement("div");
    msg.className = `chat-message ${role}`;
    msg.innerHTML = `<p>${this.escapeHtml(text)}</p>`;
    container.appendChild(msg);
    container.scrollTop = container.scrollHeight;
  }

  toggleChatPanel() {
    const panel = document.getElementById("chat-panel");
    const messages = document.getElementById("chat-messages");
    const inputContainer = panel?.querySelector(".chat-input-container");

    if (messages && inputContainer) {
      const isCollapsed = messages.style.display === "none";
      messages.style.display = isCollapsed ? "flex" : "none";
      inputContainer.style.display = isCollapsed ? "flex" : "none";
    }
  }

  showModal(id) {
    const modal = document.getElementById(id);
    if (modal) modal.style.display = "flex";
  }

  hideModal(id) {
    const modal = document.getElementById(id);
    if (modal) modal.style.display = "none";
  }

  showNotification(message, type = "info") {
    console.log(`[${type.toUpperCase()}] ${message}`);

    const existing = document.querySelector(".notification");
    if (existing) existing.remove();

    const notification = document.createElement("div");
    notification.className = `notification notification-${type}`;
    notification.textContent = message;
    notification.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            padding: 12px 20px;
            border-radius: 8px;
            background: ${type === "error" ? "#e74c3c" : type === "success" ? "#27ae60" : type === "warning" ? "#f39c12" : "#3498db"};
            color: white;
            font-size: 14px;
            z-index: 10000;
            animation: slideIn 0.3s ease;
        `;
    document.body.appendChild(notification);

    setTimeout(() => notification.remove(), 3000);
  }

  msToPixels(ms) {
    return ms * this.pixelsPerMs;
  }

  pixelsToMs(pixels) {
    return Math.round(pixels / this.pixelsPerMs);
  }

  formatTime(ms) {
    const totalSeconds = Math.floor(ms / 1000);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;
    return `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
  }

  formatDuration(ms) {
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
  }

  getLayerColor(type) {
    switch (type) {
      case "text":
        return "#e74c3c";
      case "image":
        return "#27ae60";
      case "shape":
        return "#3498db";
      default:
        return "#9b59b6";
    }
  }

  escapeHtml(text) {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }
}

document.addEventListener("DOMContentLoaded", () => {
  window.videoEditor = new VideoEditor();
});
