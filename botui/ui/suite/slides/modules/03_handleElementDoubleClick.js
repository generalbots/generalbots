// slides/modules/03_handleElementDoubleClick.js
"use strict";

// Functions: handleElementDoubleClick, handleResizeStart, handleMouseMove, resizeElement, handleMouseUp, handleKeyDown, selectElement, clearSelection, updateSelectionHandles, hideSelectionHandles, updateElementPosition, updatePropertiesPanel, showPropertiesPanel, startTextEditing, goToSlide, addSlide, duplicateSlide, deleteSlide, updateSlideCounter, showImageModal, addImage, showShapeModal, addShape

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

