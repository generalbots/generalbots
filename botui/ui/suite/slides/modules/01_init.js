
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
