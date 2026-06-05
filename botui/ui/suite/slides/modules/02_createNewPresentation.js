// slides/modules/02_createNewPresentation.js
"use strict";

// Functions: createNewPresentation, createSlide, createTextElement, createShapeElement, createImageElement, createDefaultTheme, renderThumbnails, renderSlideThumbnailContent, renderCurrentSlide, renderElementHTML, buildElementStyle, renderShapeSVG, renderChartContent, bindElementEvents, handleCanvasMouseDown, handleCanvasDoubleClick, addTextBox, addTextBoxAt, handleElementMouseDown

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

