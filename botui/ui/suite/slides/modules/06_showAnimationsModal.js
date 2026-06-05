// slides/modules/06_showAnimationsModal.js
"use strict";

// Functions: showAnimationsModal, updateSelectedElementInfo, updateAnimationOrderList, applyAnimation, removeAnimation, previewAnimation, showSlideSorter, renderSorterGrid, renderSorterSlidePreview, sorterSelectSlide, handleSorterDragStart, handleSorterDragOver, handleSorterDrop, handleSorterDragEnd, sorterAddSlide, sorterDuplicateSlide, sorterDuplicateAt, sorterDeleteSlide, sorterDeleteAt, applySorterChanges, showExportPdfModal, exportToPdf

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
