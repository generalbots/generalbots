(function () {
  "use strict";

  const CONFIG = {
    CANVAS_WIDTH: 960,
    CANVAS_HEIGHT: 540,
    MAX_HISTORY: 50,
    AUTOSAVE_DELAY: 3000,
    WS_RECONNECT_DELAY: 5000,
    MIN_ELEMENT_SIZE: 20,
  };

  const state = {
    presentationId: null,
    presentationName: "Untitled Presentation",
    slides: [],
    currentSlideIndex: 0,
    selectedElement: null,
    clipboard: null,
    history: [],
    historyIndex: -1,
    zoom: 100,
    collaborators: [],
    ws: null,
    isDragging: false,
    isResizing: false,
    isRotating: false,
    dragStart: null,
    resizeHandle: null,
    isDirty: false,
    autoSaveTimer: null,
    isPresenting: false,
    theme: null,
    driveSource: null,
    chatPanelOpen: true,
  };

  const elements = {};

  function init() {
    cacheElements();
    bindEvents();
    createNewPresentation();
    loadFromUrlParams();
    renderThumbnails();
    renderCurrentSlide();
    updateSlideCounter();
  }

  function cacheElements() {
    elements.app = document.getElementById("slides-app");
    elements.presentationName = document.getElementById("presentationName");
    elements.thumbnailsPanel = document.getElementById("thumbnailsPanel");
    elements.thumbnails = document.getElementById("thumbnails");
    elements.canvasContainer = document.getElementById("canvasContainer");
    elements.slideCanvas = document.getElementById("slideCanvas");
    elements.canvasContent = document.getElementById("canvasContent");
    elements.selectionHandles = document.getElementById("selectionHandles");
    elements.cursorIndicators = document.getElementById("cursorIndicators");
    elements.collaborators = document.getElementById("collaborators");
    elements.slideInfo = document.getElementById("slideInfo");
    elements.saveStatus = document.getElementById("saveStatus");
    elements.zoomLevel = document.getElementById("zoomLevel");
    elements.chatPanel = document.getElementById("chatPanel");
    elements.chatMessages = document.getElementById("chatMessages");
    elements.chatInput = document.getElementById("chatInput");
    elements.chatForm = document.getElementById("chatForm");
    elements.contextMenu = document.getElementById("contextMenu");
    elements.slideContextMenu = document.getElementById("slideContextMenu");
    elements.presenterModal = document.getElementById("presenterModal");
  }

  function bindEvents() {
    if (elements.presentationName) {
      elements.presentationName.addEventListener("change", (e) => {
        state.presentationName = e.target.value || "Untitled Presentation";
        state.isDirty = true;
        scheduleAutoSave();
      });
    }

    document.getElementById("undoBtn")?.addEventListener("click", undo);
    document.getElementById("redoBtn")?.addEventListener("click", redo);

    document
      .getElementById("addTextBtn")
      ?.addEventListener("click", addTextBox);
    document
      .getElementById("addImageBtn")
      ?.addEventListener("click", () => showModal("imageModal"));
    document
      .getElementById("addShapeBtn")
      ?.addEventListener("click", () => showModal("shapeModal"));
    document.getElementById("addTableBtn")?.addEventListener("click", addTable);
    document
      .getElementById("addSlideBtn")
      ?.addEventListener("click", () => addSlide());

    document.getElementById("boldBtn")?.addEventListener("click", toggleBold);
    document
      .getElementById("italicBtn")
      ?.addEventListener("click", toggleItalic);
    document
      .getElementById("underlineBtn")
      ?.addEventListener("click", toggleUnderline);

    document
      .getElementById("fontFamily")
      ?.addEventListener("change", (e) => setFontFamily(e.target.value));
    document
      .getElementById("fontSize")
      ?.addEventListener("change", (e) => setFontSize(e.target.value));

    document.getElementById("textColorBtn")?.addEventListener("click", () => {
      document.getElementById("textColorPicker")?.click();
    });
    document
      .getElementById("textColorPicker")
      ?.addEventListener("input", (e) => setTextColor(e.target.value));
    document.getElementById("fillColorBtn")?.addEventListener("click", () => {
      document.getElementById("fillColorPicker")?.click();
    });
    document
      .getElementById("fillColorPicker")
      ?.addEventListener("input", (e) => setFillColor(e.target.value));

    document
      .getElementById("alignLeftBtn")
      ?.addEventListener("click", () => setTextAlign("left"));
    document
      .getElementById("alignCenterBtn")
      ?.addEventListener("click", () => setTextAlign("center"));
    document
      .getElementById("alignRightBtn")
      ?.addEventListener("click", () => setTextAlign("right"));

    document
      .getElementById("presentBtn")
      ?.addEventListener("click", startPresentation);
    document
      .getElementById("shareBtn")
      ?.addEventListener("click", () => showModal("shareModal"));

    document
      .getElementById("transitionsBtn")
      ?.addEventListener("click", showTransitionsModal);
    document
      .getElementById("closeTransitionsModal")
      ?.addEventListener("click", () => hideModal("transitionsModal"));
    document
      .getElementById("applyTransitionsBtn")
      ?.addEventListener("click", applyTransition);
    document
      .getElementById("cancelTransitionsBtn")
      ?.addEventListener("click", () => hideModal("transitionsModal"));
    document
      .getElementById("transitionDuration")
      ?.addEventListener("input", updateDurationDisplay);
    document.querySelectorAll(".transition-btn").forEach((btn) => {
      btn.addEventListener("click", () =>
        selectTransition(btn.dataset.transition),
      );
    });

    document
      .getElementById("animationsBtn")
      ?.addEventListener("click", showAnimationsModal);
    document
      .getElementById("closeAnimationsModal")
      ?.addEventListener("click", () => hideModal("animationsModal"));
    document
      .getElementById("applyAnimationsBtn")
      ?.addEventListener("click", applyAnimation);
    document
      .getElementById("cancelAnimationsBtn")
      ?.addEventListener("click", () => hideModal("animationsModal"));
    document
      .getElementById("previewAnimationBtn")
      ?.addEventListener("click", previewAnimation);

    document
      .getElementById("slideSorterBtn")
      ?.addEventListener("click", showSlideSorter);
    document
      .getElementById("closeSlideSorterModal")
      ?.addEventListener("click", () => hideModal("slideSorterModal"));
    document
      .getElementById("applySorterBtn")
      ?.addEventListener("click", applySorterChanges);
    document
      .getElementById("cancelSorterBtn")
      ?.addEventListener("click", () => hideModal("slideSorterModal"));
    document
      .getElementById("sorterAddSlide")
      ?.addEventListener("click", sorterAddSlide);
    document
      .getElementById("sorterDuplicateSlide")
      ?.addEventListener("click", sorterDuplicateSlide);
    document
      .getElementById("sorterDeleteSlide")
      ?.addEventListener("click", sorterDeleteSlide);

    document
      .getElementById("masterSlideBtn")
      ?.addEventListener("click", showMasterSlideModal);
    document
      .getElementById("closeMasterSlideModal")
      ?.addEventListener("click", () => hideModal("masterSlideModal"));
    document
      .getElementById("applyMasterBtn")
      ?.addEventListener("click", applyMasterSlide);
    document
      .getElementById("cancelMasterBtn")
      ?.addEventListener("click", () => hideModal("masterSlideModal"));
    document
      .getElementById("resetMasterBtn")
      ?.addEventListener("click", resetMasterSlide);
    document.querySelectorAll(".master-layout-item").forEach((item) => {
      item.addEventListener("click", () =>
        selectMasterLayout(item.dataset.layout),
      );
    });
    document
      .getElementById("masterPrimaryColor")
      ?.addEventListener("input", updateMasterPreview);
    document
      .getElementById("masterSecondaryColor")
      ?.addEventListener("input", updateMasterPreview);
    document
      .getElementById("masterAccentColor")
      ?.addEventListener("input", updateMasterPreview);
    document
      .getElementById("masterBgColor")
      ?.addEventListener("input", updateMasterPreview);
    document
      .getElementById("masterTextColor")
      ?.addEventListener("input", updateMasterPreview);
    document
      .getElementById("masterTextLightColor")
      ?.addEventListener("input", updateMasterPreview);
    document
      .getElementById("masterHeadingFont")
      ?.addEventListener("change", updateMasterPreview);
    document
      .getElementById("masterBodyFont")
      ?.addEventListener("change", updateMasterPreview);

    document
      .getElementById("exportPdfBtn")
      ?.addEventListener("click", showExportPdfModal);
    document
      .getElementById("closeExportPdfModal")
      ?.addEventListener("click", () => hideModal("exportPdfModal"));
    document
      .getElementById("exportPdfBtnConfirm")
      ?.addEventListener("click", exportToPdf);
    document
      .getElementById("cancelExportPdfBtn")
      ?.addEventListener("click", () => hideModal("exportPdfModal"));

    document.getElementById("zoomInBtn")?.addEventListener("click", zoomIn);
    document.getElementById("zoomOutBtn")?.addEventListener("click", zoomOut);

    document
      .getElementById("chatToggle")
      ?.addEventListener("click", toggleChatPanel);
    document
      .getElementById("chatClose")
      ?.addEventListener("click", toggleChatPanel);
    elements.chatForm?.addEventListener("submit", handleChatSubmit);

    document.querySelectorAll(".suggestion-btn").forEach((btn) => {
      btn.addEventListener("click", () =>
        handleSuggestionClick(btn.dataset.action),
      );
    });

    document.querySelectorAll(".btn-close").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        const modal = e.target.closest(".modal");
        if (modal) modal.classList.add("hidden");
      });
    });

    document
      .getElementById("closeShareModal")
      ?.addEventListener("click", () => hideModal("shareModal"));
    document
      .getElementById("closeImageModal")
      ?.addEventListener("click", () => hideModal("imageModal"));
    document
      .getElementById("closeShapeModal")
      ?.addEventListener("click", () => hideModal("shapeModal"));
    document
      .getElementById("closeNotesModal")
      ?.addEventListener("click", () => hideModal("notesModal"));
    document
      .getElementById("closeBackgroundModal")
      ?.addEventListener("click", () => hideModal("backgroundModal"));

    document
      .getElementById("insertImageBtn")
      ?.addEventListener("click", insertImage);
    document
      .getElementById("saveNotesBtn")
      ?.addEventListener("click", saveNotes);
    document
      .getElementById("applyBgBtn")
      ?.addEventListener("click", applyBackground);
    document
      .getElementById("copyLinkBtn")
      ?.addEventListener("click", copyShareLink);

    document.querySelectorAll(".shape-btn").forEach((btn) => {
      btn.addEventListener("click", () => {
        addShape(btn.dataset.shape);
        hideModal("shapeModal");
      });
    });

    if (elements.canvasContent) {
      elements.canvasContent.addEventListener(
        "mousedown",
        handleCanvasMouseDown,
      );
      elements.canvasContent.addEventListener(
        "dblclick",
        handleCanvasDoubleClick,
      );
    }

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("contextmenu", handleContextMenu);
    document.addEventListener("click", handleDocumentClick);

    document.querySelectorAll(".context-item").forEach((item) => {
      item.addEventListener("click", () =>
        handleContextAction(item.dataset.action),
      );
    });

    document
      .getElementById("prevSlideBtn")
      ?.addEventListener("click", () => navigatePresentation(-1));
    document
      .getElementById("nextSlideBtn")
      ?.addEventListener("click", () => navigatePresentation(1));
    document
      .getElementById("exitPresenterBtn")
      ?.addEventListener("click", exitPresentation);

    window.addEventListener("beforeunload", handleBeforeUnload);
  }

  function handleBeforeUnload(e) {
    if (state.isDirty) {
      e.preventDefault();
      e.returnValue = "";
    }
  }

  async function loadFromUrlParams() {
    const urlParams = new URLSearchParams(window.location.search);
    const hash = window.location.hash;
    let presentationId = urlParams.get("id");
    let bucket = urlParams.get("bucket");
    let path = urlParams.get("path");

    if (hash) {
      const hashQueryIndex = hash.indexOf("?");
      if (hashQueryIndex > -1) {
        const hashParams = new URLSearchParams(hash.slice(hashQueryIndex + 1));
        presentationId = presentationId || hashParams.get("id");
        bucket = bucket || hashParams.get("bucket");
        path = path || hashParams.get("path");
      } else if (hash.startsWith("#id=")) {
        presentationId = hash.slice(4);
      }
    }

    if (bucket && path) {
      await loadFromDrive(bucket, path);
    } else if (presentationId) {
      try {
        const response = await fetch(`/api/slides/${presentationId}`);
        if (response.ok) {
          const data = await response.json();
          state.presentationId = presentationId;
          state.presentationName = data.name || "Untitled Presentation";
          state.slides = data.slides || [];

          if (elements.presentationName) {
            elements.presentationName.value = state.presentationName;
          }

          renderThumbnails();
          renderCurrentSlide();
          updateSlideCounter();
        }
      } catch (e) {
        console.error("Load failed:", e);
        createNewPresentation();
      }
    } else {
      createNewPresentation();
    }
  }

  async function loadFromDrive(bucket, path) {
    const fileName = path.split("/").pop() || "presentation";

    state.driveSource = { bucket, path };
    state.presentationName = fileName;

    if (elements.presentationName) {
      elements.presentationName.value = fileName;
    }

    try {
      const response = await fetch("/api/files/read", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ bucket, path }),
      });

      if (!response.ok) {
        throw new Error(`Failed to load file: ${response.status}`);
      }

      const data = await response.json();
      const content = data.content || "";

      createNewPresentation();
      if (state.slides.length > 0 && state.slides[0].elements) {
        const titleElement = state.slides[0].elements.find(
          (el) => el.element_type === "text" && el.style?.fontSize >= 32,
        );
        if (titleElement) {
          titleElement.content = fileName.replace(/\.[^/.]+$/, "");
        }
      }

      renderThumbnails();
      renderCurrentSlide();
      updateSlideCounter();
      state.isDirty = false;
    } catch (err) {
      console.error("Failed to load file from drive:", err);
      createNewPresentation();
    }
  }

  function createNewPresentation() {
    const titleSlide = createSlide("title");
    state.slides = [titleSlide];
    state.currentSlideIndex = 0;
    state.theme = createDefaultTheme();
    renderThumbnails();
    renderCurrentSlide();
    updateSlideCounter();
  }

  function createSlide(layout) {
    const slide = {
      id: generateId(),
      layout: layout,
      elements: [],
      background: {
        bg_type: "solid",
        color: "#ffffff",
      },
      notes: null,
      transition: {
        transition_type: "fade",
        duration: 0.5,
      },
    };

    switch (layout) {
      case "title":
        slide.elements.push(
          createTextElement(100, 200, 760, 100, "Presentation Title", {
            fontSize: 48,
            fontWeight: "bold",
            textAlign: "center",
            color: "#1e293b",
          }),
        );
        slide.elements.push(
          createTextElement(100, 320, 760, 50, "Subtitle or Author Name", {
            fontSize: 24,
            textAlign: "center",
            color: "#64748b",
          }),
        );
        break;
      case "title-content":
        slide.elements.push(
          createTextElement(50, 40, 860, 60, "Slide Title", {
            fontSize: 36,
            fontWeight: "bold",
            color: "#1e293b",
          }),
        );
        slide.elements.push(
          createTextElement(
            50,
            120,
            860,
            400,
            "• Click to add content\n• Add your bullet points here",
            {
              fontSize: 20,
              color: "#374151",
              lineHeight: 1.6,
            },
          ),
        );
        break;
      case "two-column":
        slide.elements.push(
          createTextElement(50, 40, 860, 60, "Slide Title", {
            fontSize: 36,
            fontWeight: "bold",
            color: "#1e293b",
          }),
        );
        slide.elements.push(
          createTextElement(50, 120, 410, 400, "Left column content", {
            fontSize: 18,
            color: "#374151",
          }),
        );
        slide.elements.push(
          createTextElement(500, 120, 410, 400, "Right column content", {
            fontSize: 18,
            color: "#374151",
          }),
        );
        break;
      case "section":
        slide.elements.push(
          createTextElement(100, 220, 760, 100, "Section Title", {
            fontSize: 48,
            fontWeight: "bold",
            textAlign: "center",
            color: "#1e293b",
          }),
        );
        break;
      case "blank":
      default:
        break;
    }

    return slide;
  }

  function createTextElement(x, y, width, height, text, style) {
    return {
      id: generateId(),
      element_type: "text",
      x: x,
      y: y,
      width: width,
      height: height,
      rotation: 0,
      content: { text: text },
      style: {
        fontFamily: style.fontFamily || "Inter",
        fontSize: style.fontSize || 16,
        fontWeight: style.fontWeight || "normal",
        fontStyle: style.fontStyle || "normal",
        textAlign: style.textAlign || "left",
        verticalAlign: style.verticalAlign || "top",
        color: style.color || "#000000",
        lineHeight: style.lineHeight || 1.4,
        ...style,
      },
      animations: [],
      z_index: 1,
      locked: false,
    };
  }

  function createShapeElement(x, y, width, height, shapeType, style) {
    return {
      id: generateId(),
      element_type: "shape",
      x: x,
      y: y,
      width: width,
      height: height,
      rotation: 0,
      content: { shape_type: shapeType },
      style: {
        fill: style.fill || "#3b82f6",
        stroke: style.stroke || "none",
        strokeWidth: style.strokeWidth || 0,
        opacity: style.opacity || 1,
        borderRadius: style.borderRadius || 0,
        ...style,
      },
      animations: [],
      z_index: 1,
      locked: false,
    };
  }

  function createImageElement(x, y, width, height, src) {
    return {
      id: generateId(),
      element_type: "image",
      x: x,
      y: y,
      width: width,
      height: height,
      rotation: 0,
      content: { src: src },
      style: {
        opacity: 1,
        borderRadius: 0,
      },
      animations: [],
      z_index: 1,
      locked: false,
    };
  }

  function createDefaultTheme() {
    return {
      name: "Default",
      colors: {
        primary: "#3b82f6",
        secondary: "#64748b",
        accent: "#f59e0b",
        background: "#ffffff",
        text: "#1e293b",
        text_light: "#64748b",
      },
      fonts: {
        heading: "Inter",
        body: "Inter",
      },
    };
  }

  function renderThumbnails() {
    if (!elements.thumbnails) return;

    elements.thumbnails.innerHTML = state.slides
      .map(
        (slide, index) => `
      <div class="slide-thumbnail ${index === state.currentSlideIndex ? "active" : ""}"
           data-index="${index}"
           onclick="window.slidesApp.goToSlide(${index})"
           oncontextmenu="window.slidesApp.showSlideContextMenu(event, ${index})">
        <div class="slide-thumbnail-preview" id="thumbnail-${index}">
          ${renderSlideThumbnailContent(slide)}
        </div>
        <span class="slide-thumbnail-number">${index + 1}</span>
      </div>
    `,
      )
      .join("");
  }

  function renderSlideThumbnailContent(slide) {
    const scale = 0.15;
    let html = `<div style="transform: scale(${scale}); transform-origin: top left; width: ${CONFIG.CANVAS_WIDTH}px; height: ${CONFIG.CANVAS_HEIGHT}px; background: ${slide.background.color || "#ffffff"}; position: relative;">`;

    slide.elements.forEach((element) => {
      html += renderElementHTML(element, true);
    });

    html += "</div>";
    return html;
  }

  function renderCurrentSlide() {
    if (!elements.canvas) return;

    const slide = state.slides[state.currentSlideIndex];
    if (!slide) return;

    elements.canvas.style.background = slide.background.color || "#ffffff";
    elements.canvas.innerHTML = "";

    slide.elements.forEach((element) => {
      const el = document.createElement("div");
      el.innerHTML = renderElementHTML(element);
      const elementNode = el.firstElementChild;
      if (elementNode) {
        elements.canvas.appendChild(elementNode);
        bindElementEvents(elementNode, element);
      }
    });

    clearSelection();
    updateSlideCounter();
  }

  function renderElementHTML(element, isThumbnail = false) {
    const style = buildElementStyle(element);
    const classes = ["slide-element"];

    if (
      state.selectedElement &&
      state.selectedElement.id === element.id &&
      !isThumbnail
    ) {
      classes.push("selected");
    }
    if (element.locked) {
      classes.push("locked");
    }

    let content = "";

    switch (element.element_type) {
      case "text":
        classes.push("slide-element-text");
        content = escapeHtml(element.content.text || "").replace(/\n/g, "<br>");
        break;
      case "image":
        classes.push("slide-element-image");
        content = `<img src="${element.content.src}" alt="" draggable="false">`;
        break;
      case "shape":
        classes.push("slide-element-shape");
        content = renderShapeSVG(element);
        break;
      case "chart":
        classes.push("slide-element-chart");
        content = renderChartContent(element);
        break;
    }

    return `
      <div class="${classes.join(" ")}"
           data-id="${element.id}"
           style="${style}">
        ${content}
      </div>
    `;
  }

  function buildElementStyle(element) {
    const styles = [
      `left: ${element.x}px`,
      `top: ${element.y}px`,
      `width: ${element.width}px`,
      `height: ${element.height}px`,
      `transform: rotate(${element.rotation || 0}deg)`,
      `z-index: ${element.z_index || 1}`,
    ];

    const s = element.style || {};

    if (element.element_type === "text") {
      if (s.fontFamily) styles.push(`font-family: ${s.fontFamily}`);
      if (s.fontSize) styles.push(`font-size: ${s.fontSize}px`);
      if (s.fontWeight) styles.push(`font-weight: ${s.fontWeight}`);
      if (s.fontStyle) styles.push(`font-style: ${s.fontStyle}`);
      if (s.textAlign) styles.push(`text-align: ${s.textAlign}`);
      if (s.color) styles.push(`color: ${s.color}`);
      if (s.lineHeight) styles.push(`line-height: ${s.lineHeight}`);
      if (s.fill) styles.push(`background: ${s.fill}`);
    }

    if (element.element_type === "shape") {
      if (s.opacity) styles.push(`opacity: ${s.opacity}`);
    }

    return styles.join("; ");
  }

  function renderShapeSVG(element) {
    const shapeType = element.content.shape_type || "rectangle";
    const fill = element.style.fill || "#3b82f6";
    const stroke = element.style.stroke || "none";
    const strokeWidth = element.style.strokeWidth || 0;

    let path = "";
    switch (shapeType) {
      case "rectangle":
        path = `<rect x="0" y="0" width="100%" height="100%" rx="${element.style.borderRadius || 0}"/>`;
        break;
      case "rounded-rectangle":
        path = `<rect x="0" y="0" width="100%" height="100%" rx="12"/>`;
        break;
      case "ellipse":
        path = `<ellipse cx="50%" cy="50%" rx="50%" ry="50%"/>`;
        break;
      case "triangle":
        path = `<polygon points="50,0 100,100 0,100"/>`;
        break;
      case "diamond":
        path = `<polygon points="50,0 100,50 50,100 0,50"/>`;
        break;
      case "star":
        path = `<polygon points="50,0 61,35 98,35 68,57 79,91 50,70 21,91 32,57 2,35 39,35"/>`;
        break;
      case "arrow-right":
        path = `<polygon points="0,25 60,25 60,0 100,50 60,100 60,75 0,75"/>`;
        break;
      case "callout":
        path = `<path d="M0,0 L100,0 L100,70 L40,70 L20,100 L20,70 L0,70 Z"/>`;
        break;
      default:
        path = `<rect x="0" y="0" width="100%" height="100%"/>`;
    }

    return `
      <svg viewBox="0 0 100 100" preserveAspectRatio="none" style="fill: ${fill}; stroke: ${stroke}; stroke-width: ${strokeWidth};">
        ${path}
      </svg>
    `;
  }

  function renderChartContent(element) {
    return '<div style="display:flex;align-items:center;justify-content:center;height:100%;color:#999;">Chart</div>';
  }

  function bindElementEvents(node, element) {
    node.addEventListener("mousedown", (e) =>
      handleElementMouseDown(e, element),
    );
    node.addEventListener("dblclick", (e) =>
      handleElementDoubleClick(e, element),
    );
  }

  function handleCanvasMouseDown(e) {
    if (e.target === elements.canvas) {
      clearSelection();
    }
  }

  function handleCanvasDoubleClick(e) {
    if (e.target === elements.canvas) {
      const rect = elements.canvas.getBoundingClientRect();
      const x = (e.clientX - rect.left) / state.zoom;
      const y = (e.clientY - rect.top) / state.zoom;
      addTextBoxAt(x - 100, y - 25);
    }
  }

  function addTextBox() {
    const slide = state.slides[state.currentSlideIndex];
    const centerX = CONFIG.CANVAS_WIDTH / 2 - 150;
    const centerY = CONFIG.CANVAS_HEIGHT / 2 - 30;
    addTextBoxAt(centerX, centerY);
  }

  function addTextBoxAt(x, y) {
    const slide = state.slides[state.currentSlideIndex];
    const textElement = createTextElement(x, y, 300, 60, "Click to edit text", {
      fontSize: 24,
      color: "#1e293b",
    });
    slide.elements.push(textElement);
    saveToHistory();
    renderCurrentSlide();
    selectElement(textElement);
    scheduleAutoSave();
    broadcastChange("elementAdded", { element: textElement });
  }

  function handleElementMouseDown(e, element) {
    e.stopPropagation();

    if (element.locked) return;

    selectElement(element);

    if (e.button === 0) {
      state.isDragging = true;
      state.dragStart = {
        x: e.clientX,
        y: e.clientY,
        elementX: element.x,
        elementY: element.y,
      };
    }
  }

  function handleElementDoubleClick(e, element) {
    e.stopPropagation();

    if (element.element_type === "text") {
      startTextEditing(element);
    }
  }

  function handleResizeStart(e) {
    e.stopPropagation();

    if (!state.selectedElement) return;

    const handle = e.target.dataset.handle;
    if (handle === "rotate") {
      state.isRotating = true;
    } else {
      state.isResizing = true;
      state.resizeHandle = handle;
    }

    state.dragStart = {
      x: e.clientX,
      y: e.clientY,
      elementX: state.selectedElement.x,
      elementY: state.selectedElement.y,
      elementWidth: state.selectedElement.width,
      elementHeight: state.selectedElement.height,
      elementRotation: state.selectedElement.rotation || 0,
    };
  }

  function handleMouseMove(e) {
    if (state.isDragging && state.selectedElement && state.dragStart) {
      const dx = (e.clientX - state.dragStart.x) / state.zoom;
      const dy = (e.clientY - state.dragStart.y) / state.zoom;

      state.selectedElement.x = state.dragStart.elementX + dx;
      state.selectedElement.y = state.dragStart.elementY + dy;

      updateElementPosition(state.selectedElement);
      updateSelectionHandles();
      broadcastChange("elementMove", state.selectedElement);
    } else if (state.isResizing && state.selectedElement && state.dragStart) {
      const dx = (e.clientX - state.dragStart.x) / state.zoom;
      const dy = (e.clientY - state.dragStart.y) / state.zoom;

      resizeElement(dx, dy);
      updateElementPosition(state.selectedElement);
      updateSelectionHandles();
      broadcastChange("elementResize", state.selectedElement);
    } else if (state.isRotating && state.selectedElement) {
      const rect = elements.canvas.getBoundingClientRect();
      const centerX = state.selectedElement.x + state.selectedElement.width / 2;
      const centerY =
        state.selectedElement.y + state.selectedElement.height / 2;
      const mouseX = (e.clientX - rect.left) / state.zoom;
      const mouseY = (e.clientY - rect.top) / state.zoom;

      const angle =
        Math.atan2(mouseY - centerY, mouseX - centerX) * (180 / Math.PI) + 90;
      state.selectedElement.rotation = Math.round(angle);

      updateElementPosition(state.selectedElement);
      updateSelectionHandles();
      updatePropertiesPanel();
      broadcastChange("elementRotate", state.selectedElement);
    }

    broadcastCursor(e);
  }

  function resizeElement(dx, dy) {
    const el = state.selectedElement;
    const s = state.dragStart;

    switch (state.resizeHandle) {
      case "se":
        el.width = Math.max(CONFIG.MIN_ELEMENT_SIZE, s.elementWidth + dx);
        el.height = Math.max(CONFIG.MIN_ELEMENT_SIZE, s.elementHeight + dy);
        break;
      case "sw":
        el.x = s.elementX + dx;
        el.width = Math.max(CONFIG.MIN_ELEMENT_SIZE, s.elementWidth - dx);
        el.height = Math.max(CONFIG.MIN_ELEMENT_SIZE, s.elementHeight + dy);
        break;
      case "ne":
        el.y = s.elementY + dy;
        el.width = Math.max(CONFIG.MIN_ELEMENT_SIZE, s.elementWidth + dx);
        el.height = Math.max(CONFIG.MIN_ELEMENT_SIZE, s.elementHeight - dy);
        break;
      case "nw":
        el.x = s.elementX + dx;
        el.y = s.elementY + dy;
        el.width = Math.max(CONFIG.MIN_ELEMENT_SIZE, s.elementWidth - dx);
        el.height = Math.max(CONFIG.MIN_ELEMENT_SIZE, s.elementHeight - dy);
        break;
      case "n":
        el.y = s.elementY + dy;
        el.height = Math.max(CONFIG.MIN_ELEMENT_SIZE, s.elementHeight - dy);
        break;
      case "s":
        el.height = Math.max(CONFIG.MIN_ELEMENT_SIZE, s.elementHeight + dy);
        break;
      case "e":
        el.width = Math.max(CONFIG.MIN_ELEMENT_SIZE, s.elementWidth + dx);
        break;
      case "w":
        el.x = s.elementX + dx;
        el.width = Math.max(CONFIG.MIN_ELEMENT_SIZE, s.elementWidth - dx);
        break;
    }
  }

  function handleMouseUp() {
    if (state.isDragging || state.isResizing || state.isRotating) {
      saveToHistory();
      scheduleAutoSave();
    }

    state.isDragging = false;
    state.isResizing = false;
    state.isRotating = false;
    state.dragStart = null;
    state.resizeHandle = null;
  }

  function handleKeyDown(e) {
    if (
      e.target.tagName === "INPUT" ||
      e.target.tagName === "TEXTAREA" ||
      e.target.isContentEditable
    ) {
      return;
    }

    const isMod = e.ctrlKey || e.metaKey;

    if (isMod && e.key === "z") {
      e.preventDefault();
      if (e.shiftKey) {
        redo();
      } else {
        undo();
      }
    } else if (isMod && e.key === "y") {
      e.preventDefault();
      redo();
    } else if (isMod && e.key === "c") {
      e.preventDefault();
      copyElement();
    } else if (isMod && e.key === "x") {
      e.preventDefault();
      cutElement();
    } else if (isMod && e.key === "v") {
      e.preventDefault();
      pasteElement();
    } else if (isMod && e.key === "d") {
      e.preventDefault();
      duplicateElement();
    } else if (isMod && e.key === "s") {
      e.preventDefault();
      savePresentation();
    } else if (isMod && e.key === "a") {
      e.preventDefault();
      selectAll();
    } else if (e.key === "Delete" || e.key === "Backspace") {
      if (state.selectedElement) {
        e.preventDefault();
        deleteElement();
      }
    } else if (e.key === "Escape") {
      clearSelection();
      hideAllContextMenus();
      if (state.isPresenting) {
        exitPresentation();
      }
    } else if (e.key === "ArrowUp" && state.selectedElement) {
      e.preventDefault();
      state.selectedElement.y -= e.shiftKey ? 10 : 1;
      updateElementPosition(state.selectedElement);
      updateSelectionHandles();
    } else if (e.key === "ArrowDown" && state.selectedElement) {
      e.preventDefault();
      state.selectedElement.y += e.shiftKey ? 10 : 1;
      updateElementPosition(state.selectedElement);
      updateSelectionHandles();
    } else if (e.key === "ArrowLeft" && state.selectedElement) {
      e.preventDefault();
      state.selectedElement.x -= e.shiftKey ? 10 : 1;
      updateElementPosition(state.selectedElement);
      updateSelectionHandles();
    } else if (e.key === "ArrowRight" && state.selectedElement) {
      e.preventDefault();
      state.selectedElement.x += e.shiftKey ? 10 : 1;
      updateElementPosition(state.selectedElement);
      updateSelectionHandles();
    } else if (e.key === "F5") {
      e.preventDefault();
      startPresentation();
    } else if (
      e.key === "PageDown" ||
      (e.key === "ArrowRight" && !state.selectedElement)
    ) {
      e.preventDefault();
      goToSlide(state.currentSlideIndex + 1);
    } else if (
      e.key === "PageUp" ||
      (e.key === "ArrowLeft" && !state.selectedElement)
    ) {
      e.preventDefault();
      goToSlide(state.currentSlideIndex - 1);
    }
  }

  function selectElement(element) {
    state.selectedElement = element;

    document.querySelectorAll(".slide-element.selected").forEach((el) => {
      el.classList.remove("selected");
    });

    const node = document.querySelector(`[data-id="${element.id}"]`);
    if (node) {
      node.classList.add("selected");
    }

    updateSelectionHandles();
    updatePropertiesPanel();
    showPropertiesPanel();
  }

  function clearSelection() {
    state.selectedElement = null;

    document.querySelectorAll(".slide-element.selected").forEach((el) => {
      el.classList.remove("selected");
    });

    hideSelectionHandles();
    updatePropertiesPanel();
  }

  function updateSelectionHandles() {
    if (!state.selectedElement || !elements.selectionHandles) {
      hideSelectionHandles();
      return;
    }

    const el = state.selectedElement;
    elements.selectionHandles.classList.remove("hidden");
    elements.selectionHandles.style.left = `${el.x}px`;
    elements.selectionHandles.style.top = `${el.y}px`;
    elements.selectionHandles.style.width = `${el.width}px`;
    elements.selectionHandles.style.height = `${el.height}px`;
    elements.selectionHandles.style.transform = `rotate(${el.rotation || 0}deg)`;
  }

  function hideSelectionHandles() {
    if (elements.selectionHandles) {
      elements.selectionHandles.classList.add("hidden");
    }
  }

  function updateElementPosition(element) {
    const node = document.querySelector(`[data-id="${element.id}"]`);
    if (node) {
      node.style.left = `${element.x}px`;
      node.style.top = `${element.y}px`;
      node.style.width = `${element.width}px`;
      node.style.height = `${element.height}px`;
      node.style.transform = `rotate(${element.rotation || 0}deg)`;
    }
    state.isDirty = true;
  }

  function updatePropertiesPanel() {
    if (!state.selectedElement) {
      document.getElementById("prop-x").value = "";
      document.getElementById("prop-y").value = "";
      document.getElementById("prop-width").value = "";
      document.getElementById("prop-height").value = "";
      document.getElementById("prop-rotation").value = 0;
      document.getElementById("rotation-value").textContent = "0°";
      document.getElementById("prop-opacity").value = 100;
      document.getElementById("opacity-value").textContent = "100%";
      return;
    }

    const el = state.selectedElement;
    document.getElementById("prop-x").value = Math.round(el.x);
    document.getElementById("prop-y").value = Math.round(el.y);
    document.getElementById("prop-width").value = Math.round(el.width);
    document.getElementById("prop-height").value = Math.round(el.height);
    document.getElementById("prop-rotation").value = el.rotation || 0;
    document.getElementById("rotation-value").textContent =
      `${el.rotation || 0}°`;

    const opacity = (el.style.opacity || 1) * 100;
    document.getElementById("prop-opacity").value = opacity;
    document.getElementById("opacity-value").textContent =
      `${Math.round(opacity)}%`;
  }

  function showPropertiesPanel() {
    if (elements.propertiesPanel) {
      elements.propertiesPanel.classList.remove("collapsed");
    }
  }

  function startTextEditing(element) {
    const node = document.querySelector(`[data-id="${element.id}"]`);
    if (!node) return;

    node.contentEditable = true;
    node.classList.add("editing");
    node.focus();

    const range = document.createRange();
    range.selectNodeContents(node);
    const sel = window.getSelection();
    sel.removeAllRanges();
    sel.addRange(range);

    node.addEventListener(
      "blur",
      () => {
        node.contentEditable = false;
        node.classList.remove("editing");
        element.content.text = node.innerText;
        saveToHistory();
        scheduleAutoSave();
        renderThumbnails();
      },
      { once: true },
    );
  }

  function goToSlide(index) {
    if (index < 0 || index >= state.slides.length) return;

    state.currentSlideIndex = index;
    renderCurrentSlide();
    renderThumbnails();
    updateSlideCounter();
    broadcastChange("slideChange", { slideIndex: index });
  }

  function addSlide(layout = "title-content") {
    const newSlide = createSlide(layout);
    state.slides.splice(state.currentSlideIndex + 1, 0, newSlide);
    state.currentSlideIndex++;
    saveToHistory();
    renderThumbnails();
    renderCurrentSlide();
    updateSlideCounter();
    scheduleAutoSave();
    broadcastChange("slideAdded", { slideIndex: state.currentSlideIndex });
  }

  function duplicateSlide() {
    const currentSlide = state.slides[state.currentSlideIndex];
    const duplicated = JSON.parse(JSON.stringify(currentSlide));
    duplicated.id = generateId();
    duplicated.elements.forEach((el) => {
      el.id = generateId();
    });
    state.slides.splice(state.currentSlideIndex + 1, 0, duplicated);
    state.currentSlideIndex++;
    saveToHistory();
    renderThumbnails();
    renderCurrentSlide();
    updateSlideCounter();
    scheduleAutoSave();
  }

  function deleteSlide() {
    if (state.slides.length <= 1) return;

    state.slides.splice(state.currentSlideIndex, 1);
    if (state.currentSlideIndex >= state.slides.length) {
      state.currentSlideIndex = state.slides.length - 1;
    }
    saveToHistory();
    renderThumbnails();
    renderCurrentSlide();
    updateSlideCounter();
    scheduleAutoSave();
    broadcastChange("slideDeleted", { slideIndex: state.currentSlideIndex });
  }

  function updateSlideCounter() {
    const currentEl = document.getElementById("current-slide-num");
    const totalEl = document.getElementById("total-slides-num");
    if (currentEl) currentEl.textContent = state.currentSlideIndex + 1;
    if (totalEl) totalEl.textContent = state.slides.length;
  }

  function showImageModal() {
    const url = prompt("Enter image URL:");
    if (url) {
      addImage(url);
    }
  }

  function addImage(url) {
    const slide = state.slides[state.currentSlideIndex];
    const imageElement = createImageElement(100, 100, 400, 300, url);
    slide.elements.push(imageElement);
    saveToHistory();
    renderCurrentSlide();
    selectElement(imageElement);
    scheduleAutoSave();
  }

  function showShapeModal() {
    addShape("rectangle");
  }

  function addShape(shapeType) {
    const slide = state.slides[state.currentSlideIndex];
    const shapeElement = createShapeElement(100, 100, 200, 150, shapeType, {
      fill: "#3b82f6",
    });
    slide.elements.push(shapeElement);
    saveToHistory();
    renderCurrentSlide();
    selectElement(shapeElement);
    scheduleAutoSave();
  }

  function showChartModal() {
    alert("Chart insertion coming soon!");
  }

  function addTable() {
    alert("Table insertion coming soon!");
  }

  function setFontFamily(family) {
    if (
      state.selectedElement &&
      state.selectedElement.element_type === "text"
    ) {
      state.selectedElement.style.fontFamily = family;
      renderCurrentSlide();
      scheduleAutoSave();
    }
  }

  function setFontSize(size) {
    if (
      state.selectedElement &&
      state.selectedElement.element_type === "text"
    ) {
      state.selectedElement.style.fontSize = parseInt(size, 10);
      renderCurrentSlide();
      scheduleAutoSave();
    }
  }

  function toggleBold() {
    if (
      state.selectedElement &&
      state.selectedElement.element_type === "text"
    ) {
      state.selectedElement.style.fontWeight =
        state.selectedElement.style.fontWeight === "bold" ? "normal" : "bold";
      renderCurrentSlide();
      scheduleAutoSave();
    }
  }

  function toggleItalic() {
    if (
      state.selectedElement &&
      state.selectedElement.element_type === "text"
    ) {
      state.selectedElement.style.fontStyle =
        state.selectedElement.style.fontStyle === "italic"
          ? "normal"
          : "italic";
      renderCurrentSlide();
      scheduleAutoSave();
    }
  }

  function toggleUnderline() {
    if (
      state.selectedElement &&
      state.selectedElement.element_type === "text"
    ) {
      state.selectedElement.style.textDecoration =
        state.selectedElement.style.textDecoration === "underline"
          ? "none"
          : "underline";
      renderCurrentSlide();
      scheduleAutoSave();
    }
  }

  function startPresentation() {
    state.isPresenting = true;
    if (elements.presenterModal) {
      elements.presenterModal.classList.remove("hidden");
      renderPresenterSlide();
    }
    document.addEventListener("keydown", handlePresenterKeyDown);
  }

  function exitPresentation() {
    state.isPresenting = false;
    if (elements.presenterModal) {
      elements.presenterModal.classList.add("hidden");
    }
    document.removeEventListener("keydown", handlePresenterKeyDown);
  }

  function handlePresenterKeyDown(e) {
    if (e.key === "Escape") {
      exitPresentation();
    } else if (e.key === "ArrowRight" || e.key === " ") {
      navigatePresentation(1);
    } else if (e.key === "ArrowLeft") {
      navigatePresentation(-1);
    }
  }

  function navigatePresentation(direction) {
    const newIndex = state.currentSlideIndex + direction;
    if (newIndex >= 0 && newIndex < state.slides.length) {
      goToSlide(newIndex);
      if (state.isPresenting) {
        renderPresenterSlide();
      }
    }
  }

  function renderPresenterSlide() {
    const presenterSlide = document.getElementById("presenterSlide");
    const presenterSlideNumber = document.getElementById(
      "presenterSlideNumber",
    );
    if (presenterSlide && state.slides[state.currentSlideIndex]) {
      presenterSlide.innerHTML = renderSlideContent(
        state.slides[state.currentSlideIndex],
      );
    }
    if (presenterSlideNumber) {
      presenterSlideNumber.textContent = `${state.currentSlideIndex + 1} / ${state.slides.length}`;
    }
  }

  function renderSlideContent(slide) {
    let html = "";
    if (slide.elements) {
      slide.elements.forEach((el) => {
        html += renderElementHTML(el);
      });
    }
    return html;
  }

  function zoomIn() {
    if (state.zoom < 200) {
      state.zoom += 10;
      applyZoom();
    }
  }

  function zoomOut() {
    if (state.zoom > 50) {
      state.zoom -= 10;
      applyZoom();
    }
  }

  function applyZoom() {
    if (elements.slideCanvas) {
      elements.slideCanvas.style.transform = `scale(${state.zoom / 100})`;
    }
    if (elements.zoomLevel) {
      elements.zoomLevel.textContent = `${state.zoom}%`;
    }
  }

  function toggleChatPanel() {
    state.chatPanelOpen = !state.chatPanelOpen;
    elements.chatPanel?.classList.toggle("collapsed", !state.chatPanelOpen);
  }

  function handleChatSubmit(e) {
    e.preventDefault();
    const message = elements.chatInput?.value.trim();
    if (!message) return;

    addChatMessage("user", message);
    if (elements.chatInput) elements.chatInput.value = "";

    processAICommand(message);
  }

  function handleSuggestionClick(action) {
    const commands = {
      title: "Add a title slide",
      image: "Insert an image",
      duplicate: "Duplicate this slide",
      notes: "Add speaker notes",
    };

    const message = commands[action] || action;
    addChatMessage("user", message);
    processAICommand(message);
  }

  function addChatMessage(role, content) {
    if (!elements.chatMessages) return;
    const div = document.createElement("div");
    div.className = `chat-message ${role}`;
    div.innerHTML = `<div class="message-bubble">${escapeHtml(content)}</div>`;
    elements.chatMessages.appendChild(div);
    elements.chatMessages.scrollTop = elements.chatMessages.scrollHeight;
  }

  async function processAICommand(command) {
    const lower = command.toLowerCase();
    let response = "";

    if (lower.includes("title") && lower.includes("slide")) {
      addSlide("title");
      response = "Added a new title slide!";
    } else if (lower.includes("add") && lower.includes("slide")) {
      addSlide();
      response = "Added a new blank slide!";
    } else if (lower.includes("duplicate")) {
      duplicateSlide();
      response = "Duplicated the current slide!";
    } else if (lower.includes("delete") && lower.includes("slide")) {
      if (state.slides.length > 1) {
        deleteSlide();
        response = "Deleted the current slide!";
      } else {
        response = "Cannot delete the only slide in the presentation.";
      }
    } else if (lower.includes("image") || lower.includes("picture")) {
      showModal("imageModal");
      response = "Opening image dialog. Enter the image URL to insert.";
    } else if (lower.includes("shape")) {
      showModal("shapeModal");
      response = "Opening shape picker. Choose a shape to insert.";
    } else if (lower.includes("text") || lower.includes("text box")) {
      addTextBox();
      response = "Added a text box! Double-click to edit the text.";
    } else if (lower.includes("background")) {
      showModal("backgroundModal");
      response = "Opening background settings. Choose a color or image.";
    } else if (lower.includes("notes") || lower.includes("speaker")) {
      showModal("notesModal");
      const currentSlide = state.slides[state.currentSlideIndex];
      const notesInput = document.getElementById("speakerNotes");
      if (notesInput && currentSlide) {
        notesInput.value = currentSlide.notes || "";
      }
      response = "Opening speaker notes. Add notes for this slide.";
    } else if (lower.includes("present") || lower.includes("start")) {
      startPresentation();
      response = "Starting presentation mode! Press Esc to exit.";
    } else if (lower.includes("bigger") || lower.includes("larger")) {
      if (state.selectedElement) {
        state.selectedElement.width =
          (state.selectedElement.width || 200) * 1.2;
        state.selectedElement.height =
          (state.selectedElement.height || 100) * 1.2;
        renderCurrentSlide();
        response = "Made the selected element larger!";
      } else {
        response = "Please select an element first.";
      }
    } else if (lower.includes("smaller")) {
      if (state.selectedElement) {
        state.selectedElement.width =
          (state.selectedElement.width || 200) * 0.8;
        state.selectedElement.height =
          (state.selectedElement.height || 100) * 0.8;
        renderCurrentSlide();
        response = "Made the selected element smaller!";
      } else {
        response = "Please select an element first.";
      }
    } else if (lower.includes("center")) {
      if (state.selectedElement) {
        state.selectedElement.x =
          (CONFIG.CANVAS_WIDTH - (state.selectedElement.width || 200)) / 2;
        state.selectedElement.y =
          (CONFIG.CANVAS_HEIGHT - (state.selectedElement.height || 100)) / 2;
        renderCurrentSlide();
        response = "Centered the selected element!";
      } else {
        response = "Please select an element first.";
      }
    } else if (lower.includes("bold")) {
      toggleBold();
      response = "Toggled bold formatting!";
    } else if (lower.includes("italic")) {
      toggleItalic();
      response = "Toggled italic formatting!";
    } else {
      try {
        const res = await fetch("/api/slides/ai", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            command,
            slideIndex: state.currentSlideIndex,
            presentationId: state.presentationId,
          }),
        });
        const data = await res.json();
        response = data.response || "I processed your request.";
      } catch {
        response =
          "I can help you with:\n• Add/duplicate/delete slides\n• Insert text, images, shapes\n• Change slide background\n• Add speaker notes\n• Make elements bigger/smaller\n• Center elements\n• Start presentation";
      }
    }

    addChatMessage("assistant", response);
  }

  function showModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) modal.classList.remove("hidden");
  }

  function hideModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) modal.classList.add("hidden");
  }

  function insertImage() {
    const url = document.getElementById("imageUrl")?.value;
    const alt = document.getElementById("imageAlt")?.value || "Image";
    if (url) {
      const slide = state.slides[state.currentSlideIndex];
      if (slide) {
        const imageElement = createImageElement(url, 100, 100, 400, 300);
        slide.elements.push(imageElement);
        renderCurrentSlide();
        renderThumbnails();
        state.isDirty = true;
        scheduleAutoSave();
      }
      hideModal("imageModal");
    }
  }

  function saveNotes() {
    const notes = document.getElementById("speakerNotes")?.value || "";
    const slide = state.slides[state.currentSlideIndex];
    if (slide) {
      slide.notes = notes;
      state.isDirty = true;
      scheduleAutoSave();
    }
    hideModal("notesModal");
    addChatMessage("assistant", "Speaker notes saved!");
  }

  function applyBackground() {
    const color = document.getElementById("bgColor")?.value;
    const imageUrl = document.getElementById("bgImageUrl")?.value;
    const slide = state.slides[state.currentSlideIndex];

    if (slide) {
      if (imageUrl) {
        slide.background = { bg_type: "image", url: imageUrl };
      } else if (color) {
        slide.background = { bg_type: "solid", color };
      }
      renderCurrentSlide();
      renderThumbnails();
      state.isDirty = true;
      scheduleAutoSave();
    }
    hideModal("backgroundModal");
    addChatMessage("assistant", "Slide background updated!");
  }

  function copyShareLink() {
    const linkInput = document.getElementById("shareLink");
    if (linkInput) {
      const shareUrl = `${window.location.origin}${window.location.pathname}#id=${state.presentationId || "new"}`;
      linkInput.value = shareUrl;
      linkInput.select();
      navigator.clipboard.writeText(shareUrl);
      addChatMessage("assistant", "Share link copied to clipboard!");
    }
  }

  function handleContextMenu(e) {
    e.preventDefault();
    const target = e.target.closest(".slide-element");
    const thumbnail = e.target.closest(".slide-thumbnail");

    hideAllContextMenus();

    if (target) {
      const elementId = target.dataset.id;
      selectElement(elementId);
      showContextMenu(elements.contextMenu, e.clientX, e.clientY);
    } else if (thumbnail) {
      showContextMenu(elements.slideContextMenu, e.clientX, e.clientY);
    }
  }

  function hideAllContextMenus() {
    elements.contextMenu?.classList.add("hidden");
    elements.slideContextMenu?.classList.add("hidden");
  }

  function showSlideContextMenu(e, slideIndex) {
    e.preventDefault();
    e.stopPropagation();
    state.currentSlideIndex = slideIndex;
    hideAllContextMenus();
    showContextMenu(elements.slideContextMenu, e.clientX, e.clientY);
  }

  function showContextMenu(menu, x, y) {
    if (!menu) return;
    menu.style.left = `${x}px`;
    menu.style.top = `${y}px`;
    menu.classList.remove("hidden");
  }

  function handleDocumentClick(e) {
    if (!e.target.closest(".context-menu")) {
      hideAllContextMenus();
    }
  }

  function handleContextAction(action) {
    hideAllContextMenus();

    switch (action) {
      case "cut":
        cutElement();
        break;
      case "copy":
        copyElement();
        break;
      case "paste":
        pasteElement();
        break;
      case "duplicate":
        duplicateElement();
        break;
      case "delete":
        deleteElement();
        break;
      case "bringFront":
        bringToFront();
        break;
      case "sendBack":
        sendToBack();
        break;
      case "newSlide":
        addSlide();
        break;
      case "duplicateSlide":
        duplicateSlide();
        break;
      case "deleteSlide":
        deleteSlide();
        break;
      case "slideBackground":
        showModal("backgroundModal");
        break;
      case "slideNotes":
        showModal("notesModal");
        break;
    }
  }

  function cutElement() {
    if (state.selectedElement) {
      state.clipboard = JSON.parse(JSON.stringify(state.selectedElement));
      deleteElement();
    }
  }

  function copyElement() {
    if (state.selectedElement) {
      state.clipboard = JSON.parse(JSON.stringify(state.selectedElement));
      addChatMessage("assistant", "Element copied!");
    }
  }

  function pasteElement() {
    if (state.clipboard) {
      const slide = state.slides[state.currentSlideIndex];
      if (slide) {
        const newElement = JSON.parse(JSON.stringify(state.clipboard));
        newElement.id = generateId();
        newElement.x += 20;
        newElement.y += 20;
        slide.elements.push(newElement);
        renderCurrentSlide();
        renderThumbnails();
        selectElement(newElement.id);
        state.isDirty = true;
        scheduleAutoSave();
      }
    }
  }

  function duplicateElement() {
    if (state.selectedElement) {
      const slide = state.slides[state.currentSlideIndex];
      if (slide) {
        const newElement = JSON.parse(JSON.stringify(state.selectedElement));
        newElement.id = generateId();
        newElement.x += 20;
        newElement.y += 20;
        slide.elements.push(newElement);
        renderCurrentSlide();
        renderThumbnails();
        selectElement(newElement.id);
        state.isDirty = true;
        scheduleAutoSave();
      }
    }
  }

  function deleteElement() {
    if (state.selectedElement) {
      const slide = state.slides[state.currentSlideIndex];
      if (slide) {
        slide.elements = slide.elements.filter(
          (el) => el.id !== state.selectedElement.id,
        );
        clearSelection();
        renderCurrentSlide();
        renderThumbnails();
        state.isDirty = true;
        scheduleAutoSave();
      }
    }
  }

  function bringToFront() {
    if (state.selectedElement) {
      const slide = state.slides[state.currentSlideIndex];
      if (slide) {
        const index = slide.elements.findIndex(
          (el) => el.id === state.selectedElement.id,
        );
        if (index > -1) {
          const [element] = slide.elements.splice(index, 1);
          slide.elements.push(element);
          renderCurrentSlide();
          state.isDirty = true;
        }
      }
    }
  }

  function sendToBack() {
    if (state.selectedElement) {
      const slide = state.slides[state.currentSlideIndex];
      if (slide) {
        const index = slide.elements.findIndex(
          (el) => el.id === state.selectedElement.id,
        );
        if (index > -1) {
          const [element] = slide.elements.splice(index, 1);
          slide.elements.unshift(element);
          renderCurrentSlide();
          state.isDirty = true;
        }
      }
    }
  }

  function setTextColor(color) {
    if (
      state.selectedElement &&
      state.selectedElement.element_type === "text"
    ) {
      state.selectedElement.style.color = color;
      renderCurrentSlide();
      state.isDirty = true;
      scheduleAutoSave();
    }
    const indicator = document.querySelector("#textColorBtn .color-indicator");
    if (indicator) indicator.style.background = color;
  }

  function setFillColor(color) {
    if (state.selectedElement) {
      if (state.selectedElement.element_type === "shape") {
        state.selectedElement.style.fill = color;
      } else if (state.selectedElement.element_type === "text") {
        state.selectedElement.style.background = color;
      }
      renderCurrentSlide();
      state.isDirty = true;
      scheduleAutoSave();
    }
    const indicator = document.querySelector("#fillColorBtn .fill-indicator");
    if (indicator) indicator.style.background = color;
  }

  function setTextAlign(align) {
    if (
      state.selectedElement &&
      state.selectedElement.element_type === "text"
    ) {
      state.selectedElement.style.textAlign = align;
      renderCurrentSlide();
      state.isDirty = true;
      scheduleAutoSave();
    }
  }

  function undo() {
    if (state.historyIndex > 0) {
      state.historyIndex--;
      restoreFromHistory();
    }
  }

  function redo() {
    if (state.historyIndex < state.history.length - 1) {
      state.historyIndex++;
      restoreFromHistory();
    }
  }

  function saveToHistory() {
    const snapshot = JSON.stringify(state.slides);
    if (state.history[state.historyIndex] === snapshot) return;

    state.history = state.history.slice(0, state.historyIndex + 1);
    state.history.push(snapshot);
    if (state.history.length > CONFIG.MAX_HISTORY) {
      state.history.shift();
    } else {
      state.historyIndex++;
    }
  }

  function restoreFromHistory() {
    if (state.history[state.historyIndex]) {
      state.slides = JSON.parse(state.history[state.historyIndex]);
      renderThumbnails();
      renderCurrentSlide();
      updateSlideCounter();
    }
  }

  function generateId() {
    return "el-" + Math.random().toString(36).substr(2, 9);
  }

  function escapeHtml(str) {
    if (!str) return "";
    const div = document.createElement("div");
    div.textContent = str;
    return div.innerHTML;
  }

  function scheduleAutoSave() {
    if (state.autoSaveTimer) {
      clearTimeout(state.autoSaveTimer);
    }
    state.autoSaveTimer = setTimeout(savePresentation, CONFIG.AUTOSAVE_DELAY);
    if (elements.saveStatus) {
      elements.saveStatus.textContent = "Saving...";
    }
  }

  async function savePresentation() {
    if (!state.isDirty) return;

    try {
      const response = await fetch("/api/slides/save", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          id: state.presentationId,
          name: state.presentationName,
          slides: state.slides,
          theme: state.theme,
          driveSource: state.driveSource,
        }),
      });

      if (response.ok) {
        const result = await response.json();
        if (result.id) {
          state.presentationId = result.id;
          window.history.replaceState({}, "", `#id=${state.presentationId}`);
        }
        state.isDirty = false;
        if (elements.saveStatus) {
          elements.saveStatus.textContent = "Saved";
        }
      } else {
        if (elements.saveStatus) {
          elements.saveStatus.textContent = "Save failed";
        }
      }
    } catch (e) {
      console.error("Save error:", e);
      if (elements.saveStatus) {
        elements.saveStatus.textContent = "Save failed";
      }
    }
  }

  function connectWebSocket() {
    if (!state.presentationId) return;

    try {
      const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
      const wsUrl = `${protocol}//${window.location.host}/api/slides/ws/${state.presentationId}`;
      state.ws = new WebSocket(wsUrl);

      state.ws.onopen = () => {
        state.ws.send(
          JSON.stringify({
            type: "join",
            userId: getUserId(),
            userName: getUserName(),
          }),
        );
      };

      state.ws.onmessage = (e) => {
        try {
          const msg = JSON.parse(e.data);
          handleWebSocketMessage(msg);
        } catch (err) {
          console.error("WS message error:", err);
        }
      };

      state.ws.onclose = () => {
        setTimeout(connectWebSocket, CONFIG.WS_RECONNECT_DELAY);
      };
    } catch (e) {
      console.error("WebSocket failed:", e);
    }
  }

  function handleWebSocketMessage(msg) {
    switch (msg.type) {
      case "user_joined":
        addCollaborator(msg.user);
        break;
      case "user_left":
        removeCollaborator(msg.userId);
        break;
      case "slide_update":
        if (msg.userId !== getUserId()) {
          state.slides = msg.slides;
          renderThumbnails();
          renderCurrentSlide();
        }
        break;
    }
  }

  function addCollaborator(user) {
    if (!state.collaborators.find((u) => u.id === user.id)) {
      state.collaborators.push(user);
      renderCollaborators();
    }
  }

  function removeCollaborator(userId) {
    state.collaborators = state.collaborators.filter((u) => u.id !== userId);
    renderCollaborators();
  }

  function renderCollaborators() {
    if (!elements.collaborators) return;
    elements.collaborators.innerHTML = state.collaborators
      .slice(0, 4)
      .map(
        (u) => `
        <div class="collaborator-avatar" style="background:${u.color || "#4285f4"}" title="${escapeHtml(u.name)}">
          ${u.name.charAt(0).toUpperCase()}
        </div>
      `,
      )
      .join("");
  }

  function getUserId() {
    let id = localStorage.getItem("gb-user-id");
    if (!id) {
      id = "user-" + Math.random().toString(36).substr(2, 9);
      localStorage.setItem("gb-user-id", id);
    }
    return id;
  }

  function getUserName() {
    return localStorage.getItem("gb-user-name") || "Anonymous";
  }

  function showTransitionsModal() {
    showModal("transitionsModal");
    const currentSlide = state.slides[state.currentSlideIndex];
    if (currentSlide?.transition?.transition_type) {
      selectTransition(currentSlide.transition.transition_type);
    }
    if (currentSlide?.transition?.duration) {
      const durationInput = document.getElementById("transitionDuration");
      const durationValue = document.getElementById("durationValue");
      if (durationInput) durationInput.value = currentSlide.transition.duration;
      if (durationValue)
        durationValue.textContent = `${currentSlide.transition.duration}s`;
    }
  }

  function selectTransition(transitionType) {
    document.querySelectorAll(".transition-btn").forEach((btn) => {
      btn.classList.toggle("active", btn.dataset.transition === transitionType);
    });
  }

  function updateDurationDisplay() {
    const durationInput = document.getElementById("transitionDuration");
    const durationValue = document.getElementById("durationValue");
    if (durationInput && durationValue) {
      durationValue.textContent = `${durationInput.value}s`;
    }
  }

  function applyTransition() {
    const activeBtn = document.querySelector(".transition-btn.active");
    const transitionType = activeBtn?.dataset.transition || "none";
    const duration = parseFloat(
      document.getElementById("transitionDuration")?.value || 0.5,
    );
    const applyToAll = document.getElementById("applyToAllSlides")?.checked;

    saveToHistory();

    const transition = {
      transition_type: transitionType,
      duration: duration,
    };

    if (applyToAll) {
      state.slides.forEach((slide) => {
        slide.transition = { ...transition };
      });
      addChatMessage(
        "assistant",
        `Applied ${transitionType} transition to all slides.`,
      );
    } else {
      const currentSlide = state.slides[state.currentSlideIndex];
      if (currentSlide) {
        currentSlide.transition = transition;
      }
      addChatMessage(
        "assistant",
        `Applied ${transitionType} transition to current slide.`,
      );
    }

    hideModal("transitionsModal");
    state.isDirty = true;
    scheduleAutoSave();
  }

  function showAnimationsModal() {
    showModal("animationsModal");
    updateSelectedElementInfo();
    updateAnimationOrderList();
  }

  function updateSelectedElementInfo() {
    const infoEl = document.getElementById("selectedElementInfo");
    if (!infoEl) return;

    if (state.selectedElement) {
      const slide = state.slides[state.currentSlideIndex];
      const element = slide?.elements?.find(
        (el) => el.id === state.selectedElement,
      );
      if (element) {
        const type = element.element_type || "Unknown";
        const content =
          element.content?.text?.substring(0, 30) ||
          element.content?.shape_type ||
          "";
        infoEl.textContent = `${type}: ${content}${content.length > 30 ? "..." : ""}`;
        return;
      }
    }
    infoEl.textContent = "No element selected";
  }

  function updateAnimationOrderList() {
    const listEl = document.getElementById("animationOrderList");
    if (!listEl) return;

    const slide = state.slides[state.currentSlideIndex];
    const animations = [];

    slide?.elements?.forEach((element) => {
      if (element.animations?.length > 0) {
        element.animations.forEach((anim) => {
          animations.push({
            elementId: element.id,
            elementType: element.element_type,
            animation: anim,
          });
        });
      }
    });

    if (animations.length === 0) {
      listEl.innerHTML = '<p class="no-animations">No animations added yet</p>';
      return;
    }

    listEl.innerHTML = animations
      .map(
        (item, index) => `
        <div class="animation-item" data-index="${index}">
          <div>
            <div class="animation-name">${item.animation.type || "Animation"}</div>
            <div class="animation-element">${item.elementType}</div>
          </div>
          <button class="animation-remove" data-element="${item.elementId}">×</button>
        </div>
      `,
      )
      .join("");

    listEl.querySelectorAll(".animation-remove").forEach((btn) => {
      btn.addEventListener("click", () => removeAnimation(btn.dataset.element));
    });
  }

  function applyAnimation() {
    if (!state.selectedElement) {
      addChatMessage(
        "assistant",
        "Please select an element on the slide first.",
      );
      return;
    }

    const entrance = document.getElementById("entranceAnimation")?.value;
    const emphasis = document.getElementById("emphasisAnimation")?.value;
    const exit = document.getElementById("exitAnimation")?.value;
    const start =
      document.getElementById("animationStart")?.value || "on-click";
    const duration = parseFloat(
      document.getElementById("animationDuration")?.value || 0.5,
    );
    const delay = parseFloat(
      document.getElementById("animationDelay")?.value || 0,
    );

    const slide = state.slides[state.currentSlideIndex];
    const element = slide?.elements?.find(
      (el) => el.id === state.selectedElement,
    );

    if (!element) return;

    saveToHistory();

    element.animations = [];

    if (entrance && entrance !== "none") {
      element.animations.push({
        type: entrance,
        category: "entrance",
        start,
        duration,
        delay,
      });
    }

    if (emphasis && emphasis !== "none") {
      element.animations.push({
        type: emphasis,
        category: "emphasis",
        start: "after-previous",
        duration,
        delay: 0,
      });
    }

    if (exit && exit !== "none") {
      element.animations.push({
        type: exit,
        category: "exit",
        start: "after-previous",
        duration,
        delay: 0,
      });
    }

    updateAnimationOrderList();
    hideModal("animationsModal");
    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Animation applied to selected element.");
  }

  function removeAnimation(elementId) {
    const slide = state.slides[state.currentSlideIndex];
    const element = slide?.elements?.find((el) => el.id === elementId);
    if (element) {
      element.animations = [];
      updateAnimationOrderList();
      state.isDirty = true;
      scheduleAutoSave();
    }
  }

  function previewAnimation() {
    if (!state.selectedElement) {
      addChatMessage(
        "assistant",
        "Select an element to preview its animation.",
      );
      return;
    }

    const entrance = document.getElementById("entranceAnimation")?.value;
    const node = document.querySelector(
      `[data-element-id="${state.selectedElement}"]`,
    );

    if (!node || !entrance || entrance === "none") return;

    node.style.animation = "none";
    node.offsetHeight;

    const animationName = entrance.replace(/-/g, "");
    node.style.animation = `${animationName} 0.5s ease`;

    setTimeout(() => {
      node.style.animation = "";
    }, 600);
  }

  let sorterSlideOrder = [];
  let sorterSelectedSlide = null;

  function showSlideSorter() {
    showModal("slideSorterModal");
    sorterSlideOrder = state.slides.map((_, i) => i);
    sorterSelectedSlide = null;
    renderSorterGrid();
  }

  function renderSorterGrid() {
    const grid = document.getElementById("sorterGrid");
    if (!grid) return;

    grid.innerHTML = sorterSlideOrder
      .map((slideIndex, position) => {
        const slide = state.slides[slideIndex];
        if (!slide) return "";

        const isSelected = sorterSelectedSlide === position;
        return `
          <div class="sorter-slide ${isSelected ? "selected" : ""}"
               data-position="${position}"
               data-slide-index="${slideIndex}"
               draggable="true">
            <div class="sorter-slide-content">
              ${renderSorterSlidePreview(slide)}
            </div>
            <div class="sorter-slide-number">${position + 1}</div>
            <div class="sorter-slide-actions">
              <button data-action="duplicate" title="Duplicate">⎘</button>
              <button data-action="delete" title="Delete">×</button>
            </div>
          </div>
        `;
      })
      .join("");

    grid.querySelectorAll(".sorter-slide").forEach((el) => {
      el.addEventListener("click", (e) => {
        if (e.target.closest(".sorter-slide-actions")) return;
        sorterSelectSlide(parseInt(el.dataset.position));
      });

      el.addEventListener("dragstart", handleSorterDragStart);
      el.addEventListener("dragover", handleSorterDragOver);
      el.addEventListener("drop", handleSorterDrop);
      el.addEventListener("dragend", handleSorterDragEnd);

      el.querySelectorAll(".sorter-slide-actions button").forEach((btn) => {
        btn.addEventListener("click", (e) => {
          e.stopPropagation();
          const action = btn.dataset.action;
          const position = parseInt(el.dataset.position);
          if (action === "duplicate") {
            sorterDuplicateAt(position);
          } else if (action === "delete") {
            sorterDeleteAt(position);
          }
        });
      });
    });
  }

  function renderSorterSlidePreview(slide) {
    const bgColor = slide.background?.color || "#ffffff";
    let html = `<div style="width:100%;height:100%;background:${bgColor};padding:8px;font-size:6px;">`;

    if (slide.elements) {
      slide.elements.slice(0, 3).forEach((el) => {
        if (el.element_type === "text" && el.content?.text) {
          const text = el.content.text.substring(0, 50);
          html += `<div style="margin-bottom:4px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">${escapeHtml(text)}</div>`;
        }
      });
    }

    html += "</div>";
    return html;
  }

  function sorterSelectSlide(position) {
    sorterSelectedSlide = position;
    document.querySelectorAll(".sorter-slide").forEach((el) => {
      el.classList.toggle(
        "selected",
        parseInt(el.dataset.position) === position,
      );
    });
  }

  let draggedPosition = null;

  function handleSorterDragStart(e) {
    draggedPosition = parseInt(e.currentTarget.dataset.position);
    e.currentTarget.classList.add("dragging");
    e.dataTransfer.effectAllowed = "move";
  }

  function handleSorterDragOver(e) {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    e.currentTarget.classList.add("drag-over");
  }

  function handleSorterDrop(e) {
    e.preventDefault();
    const targetPosition = parseInt(e.currentTarget.dataset.position);

    if (draggedPosition !== null && draggedPosition !== targetPosition) {
      const draggedIndex = sorterSlideOrder[draggedPosition];
      sorterSlideOrder.splice(draggedPosition, 1);
      sorterSlideOrder.splice(targetPosition, 0, draggedIndex);
      renderSorterGrid();
    }

    e.currentTarget.classList.remove("drag-over");
  }

  function handleSorterDragEnd(e) {
    e.currentTarget.classList.remove("dragging");
    document.querySelectorAll(".sorter-slide").forEach((el) => {
      el.classList.remove("drag-over");
    });
    draggedPosition = null;
  }

  function sorterAddSlide() {
    const newSlide = createSlide("blank");
    state.slides.push(newSlide);
    sorterSlideOrder.push(state.slides.length - 1);
    renderSorterGrid();
  }

  function sorterDuplicateSlide() {
    if (sorterSelectedSlide === null) {
      addChatMessage("assistant", "Select a slide to duplicate.");
      return;
    }
    sorterDuplicateAt(sorterSelectedSlide);
  }

  function sorterDuplicateAt(position) {
    const originalIndex = sorterSlideOrder[position];
    const original = state.slides[originalIndex];
    if (!original) return;

    const duplicated = JSON.parse(JSON.stringify(original));
    duplicated.id = generateId();
    state.slides.push(duplicated);
    sorterSlideOrder.splice(position + 1, 0, state.slides.length - 1);
    renderSorterGrid();
  }

  function sorterDeleteSlide() {
    if (sorterSelectedSlide === null) {
      addChatMessage("assistant", "Select a slide to delete.");
      return;
    }
    sorterDeleteAt(sorterSelectedSlide);
  }

  function sorterDeleteAt(position) {
    if (sorterSlideOrder.length <= 1) {
      addChatMessage("assistant", "Cannot delete the last slide.");
      return;
    }
    sorterSlideOrder.splice(position, 1);
    if (sorterSelectedSlide >= sorterSlideOrder.length) {
      sorterSelectedSlide = sorterSlideOrder.length - 1;
    }
    renderSorterGrid();
  }

  function applySorterChanges() {
    const reorderedSlides = sorterSlideOrder.map((i) => state.slides[i]);
    state.slides = reorderedSlides;
    state.currentSlideIndex = 0;

    hideModal("slideSorterModal");
    renderThumbnails();
    renderCurrentSlide();
    updateSlideCounter();

    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Slide order updated!");
  }

  function showExportPdfModal() {
    showModal("exportPdfModal");
  }

  function exportToPdf() {
    const rangeType = document.querySelector(
      'input[name="slideRange"]:checked',
    )?.value;
    const layout = document.getElementById("pdfLayout")?.value || "full";
    const orientation =
      document.getElementById("pdfOrientation")?.value || "landscape";

    let slidesToExport = [];

    switch (rangeType) {
      case "all":
        slidesToExport = state.slides.map((_, i) => i);
        break;
      case "current":
        slidesToExport = [state.currentSlideIndex];
        break;
      case "custom":
        const customRange = document.getElementById("customRange")?.value || "";
        slidesToExport = parseSlideRange(customRange);
        break;
      default:
        slidesToExport = state.slides.map((_, i) => i);
    }

    if (slidesToExport.length === 0) {
      addChatMessage("assistant", "No slides to export.");
      return;
    }

    const printWindow = window.open("", "_blank");
    const slidesPerPage = getLayoutSlidesPerPage(layout);

    let htmlContent = `
      <!DOCTYPE html>
      <html>
      <head>
        <title>${state.presentationName} - PDF Export</title>
        <style>
          @page { size: ${orientation}; margin: 0.5in; }
          @media print {
            .page-break { page-break-after: always; }
          }
          body { font-family: Arial, sans-serif; margin: 0; padding: 0; }
          .slide-container {
            display: flex;
            flex-wrap: wrap;
            justify-content: center;
            gap: 20px;
            padding: 20px;
          }
          .slide {
            background: white;
            border: 1px solid #ccc;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            overflow: hidden;
          }
          .slide-full { width: 100%; aspect-ratio: 16/9; }
          .slide-2 { width: 45%; aspect-ratio: 16/9; }
          .slide-4 { width: 45%; aspect-ratio: 16/9; }
          .slide-6 { width: 30%; aspect-ratio: 16/9; }
          .slide-content { padding: 20px; height: 100%; box-sizing: border-box; }
          .slide-number { text-align: center; font-size: 12px; color: #666; margin-top: 8px; }
          .notes-section { padding: 10px; font-size: 11px; border-top: 1px solid #ccc; }
        </style>
      </head>
      <body>
    `;

    let slideCount = 0;
    slidesToExport.forEach((slideIndex, i) => {
      const slide = state.slides[slideIndex];
      if (!slide) return;

      if (slideCount > 0 && slideCount % slidesPerPage === 0) {
        htmlContent += '<div class="page-break"></div>';
      }

      if (slideCount % slidesPerPage === 0) {
        htmlContent += '<div class="slide-container">';
      }

      const slideClass =
        slidesPerPage === 1
          ? "slide-full"
          : slidesPerPage === 2
            ? "slide-2"
            : slidesPerPage === 4
              ? "slide-4"
              : "slide-6";
      const bgColor = slide.background?.color || "#ffffff";

      htmlContent += `
        <div class="slide ${slideClass}" style="background:${bgColor};">
          <div class="slide-content">
            ${renderSlideContentForExport(slide)}
          </div>
          <div class="slide-number">Slide ${slideIndex + 1}</div>
          ${layout === "notes" && slide.notes ? `<div class="notes-section">${escapeHtml(slide.notes)}</div>` : ""}
        </div>
      `;

      slideCount++;
      if (slideCount % slidesPerPage === 0 || i === slidesToExport.length - 1) {
        htmlContent += "</div>";
      }
    });

    htmlContent += "</body></html>";

    printWindow.document.write(htmlContent);
    printWindow.document.close();
    printWindow.focus();

    setTimeout(() => {
      printWindow.print();
    }, 500);

    hideModal("exportPdfModal");
    addChatMessage(
      "assistant",
      `Exporting ${slidesToExport.length} slide(s) to PDF...`,
    );
  }

  function parseSlideRange(rangeStr) {
    const slides = [];
    const parts = rangeStr.split(",");

    parts.forEach((part) => {
      part = part.trim();
      if (part.includes("-")) {
        const [start, end] = part.split("-").map((n) => parseInt(n.trim()) - 1);
        for (
          let i = Math.max(0, start);
          i <= Math.min(state.slides.length - 1, end);
          i++
        ) {
          if (!slides.includes(i)) slides.push(i);
        }
      } else {
        const num = parseInt(part) - 1;
        if (num >= 0 && num < state.slides.length && !slides.includes(num)) {
          slides.push(num);
        }
      }
    });

    return slides.sort((a, b) => a - b);
  }

  function getLayoutSlidesPerPage(layout) {
    switch (layout) {
      case "full":
      case "notes":
        return 1;
      case "handout-2":
        return 2;
      case "handout-4":
        return 4;
      case "handout-6":
        return 6;
      default:
        return 1;
    }
  }

  function renderSlideContentForExport(slide) {
    let html = "";
    if (slide.elements) {
      slide.elements.forEach((el) => {
        if (el.element_type === "text" && el.content?.text) {
          const fontSize = el.style?.fontSize || 16;
          const fontWeight = el.style?.fontWeight || "normal";
          const color = el.style?.color || "#000";
          html += `<div style="font-size:${fontSize}px;font-weight:${fontWeight};color:${color};margin-bottom:8px;">${escapeHtml(el.content.text)}</div>`;
        }
      });
    }
    return html || "<p>Empty slide</p>";
  }

  let selectedMasterLayout = "title";

  function showMasterSlideModal() {
    showModal("masterSlideModal");
    selectedMasterLayout = "title";

    if (state.theme) {
      const colors = state.theme.colors || {};
      const fonts = state.theme.fonts || {};

      setColorInput("masterPrimaryColor", colors.primary || "#4285f4");
      setColorInput("masterSecondaryColor", colors.secondary || "#34a853");
      setColorInput("masterAccentColor", colors.accent || "#fbbc04");
      setColorInput("masterBgColor", colors.background || "#ffffff");
      setColorInput("masterTextColor", colors.text || "#212121");
      setColorInput("masterTextLightColor", colors.text_light || "#666666");

      setSelectValue("masterHeadingFont", fonts.heading || "Arial");
      setSelectValue("masterBodyFont", fonts.body || "Arial");
    }

    updateMasterPreview();
    updateMasterLayoutSelection();
  }

  function setColorInput(id, value) {
    const el = document.getElementById(id);
    if (el) el.value = value;
  }

  function setSelectValue(id, value) {
    const el = document.getElementById(id);
    if (el) el.value = value;
  }

  function selectMasterLayout(layout) {
    selectedMasterLayout = layout;
    updateMasterLayoutSelection();
  }

  function updateMasterLayoutSelection() {
    document.querySelectorAll(".master-layout-item").forEach((item) => {
      item.classList.toggle(
        "active",
        item.dataset.layout === selectedMasterLayout,
      );
    });
  }

  function updateMasterPreview() {
    const bgColor =
      document.getElementById("masterBgColor")?.value || "#ffffff";
    const textColor =
      document.getElementById("masterTextColor")?.value || "#212121";
    const textLightColor =
      document.getElementById("masterTextLightColor")?.value || "#666666";
    const headingFont =
      document.getElementById("masterHeadingFont")?.value || "Arial";
    const bodyFont =
      document.getElementById("masterBodyFont")?.value || "Arial";

    const previewSlide = document.querySelector(".preview-slide");
    const previewHeading = document.getElementById("previewHeading");
    const previewBody = document.getElementById("previewBody");

    if (previewSlide) {
      previewSlide.style.background = bgColor;
    }
    if (previewHeading) {
      previewHeading.style.color = textColor;
      previewHeading.style.fontFamily = headingFont;
    }
    if (previewBody) {
      previewBody.style.color = textLightColor;
      previewBody.style.fontFamily = bodyFont;
    }
  }

  function applyMasterSlide() {
    const primaryColor =
      document.getElementById("masterPrimaryColor")?.value || "#4285f4";
    const secondaryColor =
      document.getElementById("masterSecondaryColor")?.value || "#34a853";
    const accentColor =
      document.getElementById("masterAccentColor")?.value || "#fbbc04";
    const bgColor =
      document.getElementById("masterBgColor")?.value || "#ffffff";
    const textColor =
      document.getElementById("masterTextColor")?.value || "#212121";
    const textLightColor =
      document.getElementById("masterTextLightColor")?.value || "#666666";
    const headingFont =
      document.getElementById("masterHeadingFont")?.value || "Arial";
    const bodyFont =
      document.getElementById("masterBodyFont")?.value || "Arial";

    saveToHistory();

    state.theme = {
      name: "Custom",
      colors: {
        primary: primaryColor,
        secondary: secondaryColor,
        accent: accentColor,
        background: bgColor,
        text: textColor,
        text_light: textLightColor,
      },
      fonts: {
        heading: headingFont,
        body: bodyFont,
      },
    };

    state.slides.forEach((slide) => {
      slide.background = slide.background || {};
      slide.background.color = bgColor;

      if (slide.elements) {
        slide.elements.forEach((el) => {
          if (el.element_type === "text") {
            el.style = el.style || {};
            const isHeading =
              el.style.fontSize >= 24 || el.style.fontWeight === "bold";
            el.style.fontFamily = isHeading ? headingFont : bodyFont;
            el.style.color = isHeading ? textColor : textLightColor;
          }
        });
      }
    });

    hideModal("masterSlideModal");
    renderThumbnails();
    renderCurrentSlide();

    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Master slide theme applied to all slides!");
  }

  function resetMasterSlide() {
    setColorInput("masterPrimaryColor", "#4285f4");
    setColorInput("masterSecondaryColor", "#34a853");
    setColorInput("masterAccentColor", "#fbbc04");
    setColorInput("masterBgColor", "#ffffff");
    setColorInput("masterTextColor", "#212121");
    setColorInput("masterTextLightColor", "#666666");
    setSelectValue("masterHeadingFont", "Arial");
    setSelectValue("masterBodyFont", "Arial");

    updateMasterPreview();
  }

  window.slidesApp = {
    init,
    addSlide,
    addTextBox,
    addShape,
    addImage,
    duplicateSlide,
    deleteSlide,
    goToSlide,
    startPresentation,
    exitPresentation,
    showModal,
    hideModal,
    toggleChatPanel,
    savePresentation,
    showTransitionsModal,
    showAnimationsModal,
    showSlideSorter,
    exportToPdf,
    showMasterSlideModal,
    showSlideContextMenu,
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
