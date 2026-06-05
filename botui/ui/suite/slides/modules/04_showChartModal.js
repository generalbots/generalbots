
"use strict";

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
        const imageElement = createImageElement(100, 100, 400, 300, url);
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
      selectElement(target);
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
