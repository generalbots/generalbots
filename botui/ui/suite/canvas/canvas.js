/* =============================================================================
   CANVAS MODULE - Whiteboard/Drawing Application
   ============================================================================= */

(function () {
  "use strict";

  // =============================================================================
  // STATE
  // =============================================================================

  const state = {
    canvasId: null,
    canvasName: "Untitled Canvas",
    tool: "select",
    color: "#000000",
    strokeWidth: 2,
    fillColor: "transparent",
    fontSize: 16,
    fontFamily: "Inter",
    zoom: 1,
    panX: 0,
    panY: 0,
    isDrawing: false,
    isPanning: false,
    startX: 0,
    startY: 0,
    elements: [],
    selectedElement: null,
    clipboard: null,
    history: [],
    historyIndex: -1,
    gridEnabled: true,
    snapToGrid: true,
    gridSize: 20,
  };

  let canvas = null;
  let ctx = null;

  // =============================================================================
  // INITIALIZATION
  // =============================================================================

  function init() {
    canvas = document.getElementById("canvas");
    if (!canvas) {
      console.warn("Canvas element not found");
      return;
    }
    ctx = canvas.getContext("2d");

    resizeCanvas();
    bindEvents();
    loadFromUrl();
    render();

    console.log("Canvas module initialized");
  }

  function resizeCanvas() {
    if (!canvas) return;
    const container = canvas.parentElement;
    if (container) {
      canvas.width = container.clientWidth || 1200;
      canvas.height = container.clientHeight || 800;
    }
  }

  function bindEvents() {
    if (!canvas) return;

    canvas.addEventListener("mousedown", handleMouseDown);
    canvas.addEventListener("mousemove", handleMouseMove);
    canvas.addEventListener("mouseup", handleMouseUp);
    canvas.addEventListener("mouseleave", handleMouseUp);
    canvas.addEventListener("wheel", handleWheel);
    canvas.addEventListener("dblclick", handleDoubleClick);

    document.addEventListener("keydown", handleKeyDown);
    window.addEventListener("resize", () => {
      resizeCanvas();
      render();
    });

    // Touch support
    canvas.addEventListener("touchstart", handleTouchStart);
    canvas.addEventListener("touchmove", handleTouchMove);
    canvas.addEventListener("touchend", handleTouchEnd);
  }

  // =============================================================================
  // TOOL SELECTION
  // =============================================================================

  function selectTool(tool) {
    state.tool = tool;

    // Update UI
    document.querySelectorAll(".tool-btn").forEach((btn) => {
      btn.classList.toggle("active", btn.dataset.tool === tool);
    });

    // Update cursor
    const cursors = {
      select: "default",
      pan: "grab",
      pencil: "crosshair",
      brush: "crosshair",
      eraser: "crosshair",
      rectangle: "crosshair",
      ellipse: "crosshair",
      line: "crosshair",
      arrow: "crosshair",
      text: "text",
      sticky: "crosshair",
      image: "crosshair",
      connector: "crosshair",
      frame: "crosshair",
    };
    canvas.style.cursor = cursors[tool] || "default";
  }

  // =============================================================================
  // MOUSE HANDLERS
  // =============================================================================

  function handleMouseDown(e) {
    const rect = canvas.getBoundingClientRect();
    const x = (e.clientX - rect.left - state.panX) / state.zoom;
    const y = (e.clientY - rect.top - state.panY) / state.zoom;

    state.startX = x;
    state.startY = y;

    if (state.tool === "pan") {
      state.isPanning = true;
      canvas.style.cursor = "grabbing";
      return;
    }

    if (state.tool === "select") {
      const element = findElementAt(x, y);
      selectElement(element);
      if (element) {
        state.isDrawing = true; // For dragging
      }
      return;
    }

    state.isDrawing = true;

    if (state.tool === "text") {
      createTextElement(x, y);
      state.isDrawing = false;
      return;
    }

    if (state.tool === "sticky") {
      createStickyNote(x, y);
      state.isDrawing = false;
      return;
    }

    if (state.tool === "pencil" || state.tool === "brush") {
      const element = {
        id: generateId(),
        type: "path",
        points: [{ x, y }],
        color: state.color,
        strokeWidth:
          state.tool === "brush" ? state.strokeWidth * 3 : state.strokeWidth,
      };
      state.elements.push(element);
      state.selectedElement = element;
    }
  }

  function handleMouseMove(e) {
    const rect = canvas.getBoundingClientRect();
    const x = (e.clientX - rect.left - state.panX) / state.zoom;
    const y = (e.clientY - rect.top - state.panY) / state.zoom;

    if (state.isPanning) {
      state.panX += e.movementX;
      state.panY += e.movementY;
      render();
      return;
    }

    if (!state.isDrawing) return;

    if (state.tool === "pencil" || state.tool === "brush") {
      if (state.selectedElement && state.selectedElement.points) {
        state.selectedElement.points.push({ x, y });
        render();
      }
      return;
    }

    if (state.tool === "eraser") {
      const element = findElementAt(x, y);
      if (element) {
        state.elements = state.elements.filter((el) => el.id !== element.id);
        render();
      }
      return;
    }

    if (state.tool === "select" && state.selectedElement) {
      const dx = x - state.startX;
      const dy = y - state.startY;
      state.selectedElement.x += dx;
      state.selectedElement.y += dy;
      state.startX = x;
      state.startY = y;
      render();
      return;
    }

    // Preview shape while drawing
    render();
    drawPreviewShape(state.startX, state.startY, x, y);
  }

  function handleMouseUp(e) {
    if (state.isPanning) {
      state.isPanning = false;
      canvas.style.cursor = state.tool === "pan" ? "grab" : "default";
      return;
    }

    if (!state.isDrawing) return;

    const rect = canvas.getBoundingClientRect();
    const x = (e.clientX - rect.left - state.panX) / state.zoom;
    const y = (e.clientY - rect.top - state.panY) / state.zoom;

    if (
      ["rectangle", "ellipse", "line", "arrow", "frame"].includes(state.tool)
    ) {
      const element = createShapeElement(
        state.tool,
        state.startX,
        state.startY,
        x,
        y,
      );
      state.elements.push(element);
      saveToHistory();
    }

    if (state.tool === "pencil" || state.tool === "brush") {
      saveToHistory();
    }

    state.isDrawing = false;
    render();
  }

  function handleWheel(e) {
    e.preventDefault();
    const delta = e.deltaY > 0 ? -0.1 : 0.1;
    const newZoom = Math.max(0.1, Math.min(5, state.zoom + delta));
    state.zoom = newZoom;
    updateZoomDisplay();
    render();
  }

  function handleDoubleClick(e) {
    const rect = canvas.getBoundingClientRect();
    const x = (e.clientX - rect.left - state.panX) / state.zoom;
    const y = (e.clientY - rect.top - state.panY) / state.zoom;

    const element = findElementAt(x, y);
    if (element && element.type === "text") {
      editTextElement(element);
    }
  }

  // =============================================================================
  // TOUCH HANDLERS
  // =============================================================================

  function handleTouchStart(e) {
    e.preventDefault();
    const touch = e.touches[0];
    handleMouseDown({ clientX: touch.clientX, clientY: touch.clientY });
  }

  function handleTouchMove(e) {
    e.preventDefault();
    const touch = e.touches[0];
    handleMouseMove({
      clientX: touch.clientX,
      clientY: touch.clientY,
      movementX: 0,
      movementY: 0,
    });
  }

  function handleTouchEnd(e) {
    e.preventDefault();
    const touch = e.changedTouches[0];
    handleMouseUp({ clientX: touch.clientX, clientY: touch.clientY });
  }

  // =============================================================================
  // KEYBOARD HANDLERS
  // =============================================================================

  function handleKeyDown(e) {
    const isMod = e.ctrlKey || e.metaKey;

    // Tool shortcuts
    if (!isMod && !e.target.matches("input, textarea")) {
      const toolKeys = {
        v: "select",
        h: "pan",
        p: "pencil",
        b: "brush",
        e: "eraser",
        r: "rectangle",
        o: "ellipse",
        l: "line",
        a: "arrow",
        t: "text",
        s: "sticky",
        i: "image",
        c: "connector",
        f: "frame",
      };
      if (toolKeys[e.key.toLowerCase()]) {
        selectTool(toolKeys[e.key.toLowerCase()]);
        return;
      }
    }

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
    } else if (isMod && e.key === "v") {
      e.preventDefault();
      pasteElement();
    } else if (isMod && e.key === "x") {
      e.preventDefault();
      cutElement();
    } else if (isMod && e.key === "a") {
      e.preventDefault();
      selectAll();
    } else if (e.key === "Delete" || e.key === "Backspace") {
      if (state.selectedElement && !e.target.matches("input, textarea")) {
        e.preventDefault();
        deleteSelected();
      }
    } else if (e.key === "Escape") {
      selectElement(null);
    } else if (e.key === "+" || e.key === "=") {
      if (isMod) {
        e.preventDefault();
        zoomIn();
      }
    } else if (e.key === "-") {
      if (isMod) {
        e.preventDefault();
        zoomOut();
      }
    } else if (e.key === "0" && isMod) {
      e.preventDefault();
      resetZoom();
    }
  }

  // =============================================================================
  // ELEMENT CREATION
  // =============================================================================

  function createShapeElement(type, x1, y1, x2, y2) {
    const minX = Math.min(x1, x2);
    const minY = Math.min(y1, y2);
    const width = Math.abs(x2 - x1);
    const height = Math.abs(y2 - y1);

    return {
      id: generateId(),
      type: type,
      x: minX,
      y: minY,
      width: width,
      height: height,
      x1: x1,
      y1: y1,
      x2: x2,
      y2: y2,
      color: state.color,
      fillColor: state.fillColor,
      strokeWidth: state.strokeWidth,
    };
  }

  function createTextElement(x, y) {
    const text = prompt("Enter text:");
    if (!text) return;

    const element = {
      id: generateId(),
      type: "text",
      x: x,
      y: y,
      text: text,
      color: state.color,
      fontSize: state.fontSize,
      fontFamily: state.fontFamily,
    };
    state.elements.push(element);
    saveToHistory();
    render();
  }

  function createStickyNote(x, y) {
    const element = {
      id: generateId(),
      type: "sticky",
      x: x,
      y: y,
      width: 200,
      height: 200,
      text: "Double-click to edit",
      color: "#ffeb3b",
    };
    state.elements.push(element);
    saveToHistory();
    render();
  }

  function editTextElement(element) {
    const newText = prompt("Edit text:", element.text);
    if (newText !== null) {
      element.text = newText;
      saveToHistory();
      render();
    }
  }

  // =============================================================================
  // ELEMENT SELECTION & MANIPULATION
  // =============================================================================

  function findElementAt(x, y) {
    for (let i = state.elements.length - 1; i >= 0; i--) {
      const el = state.elements[i];
      if (isPointInElement(x, y, el)) {
        return el;
      }
    }
    return null;
  }

  function isPointInElement(x, y, el) {
    const margin = 5;
    switch (el.type) {
      case "rectangle":
      case "frame":
      case "sticky":
        return (
          x >= el.x - margin &&
          x <= el.x + el.width + margin &&
          y >= el.y - margin &&
          y <= el.y + el.height + margin
        );
      case "ellipse":
        const cx = el.x + el.width / 2;
        const cy = el.y + el.height / 2;
        const rx = el.width / 2 + margin;
        const ry = el.height / 2 + margin;
        return (x - cx) ** 2 / rx ** 2 + (y - cy) ** 2 / ry ** 2 <= 1;
      case "text":
        return (
          x >= el.x - margin &&
          x <= el.x + 200 &&
          y >= el.y - el.fontSize &&
          y <= el.y + margin
        );
      case "line":
      case "arrow":
        return (
          distanceToLine(x, y, el.x1, el.y1, el.x2, el.y2) <=
          margin + el.strokeWidth
        );
      case "path":
        if (!el.points) return false;
        for (const pt of el.points) {
          if (Math.abs(pt.x - x) < margin && Math.abs(pt.y - y) < margin) {
            return true;
          }
        }
        return false;
      default:
        return false;
    }
  }

  function distanceToLine(x, y, x1, y1, x2, y2) {
    const A = x - x1;
    const B = y - y1;
    const C = x2 - x1;
    const D = y2 - y1;
    const dot = A * C + B * D;
    const lenSq = C * C + D * D;
    let param = lenSq !== 0 ? dot / lenSq : -1;
    let xx, yy;
    if (param < 0) {
      xx = x1;
      yy = y1;
    } else if (param > 1) {
      xx = x2;
      yy = y2;
    } else {
      xx = x1 + param * C;
      yy = y1 + param * D;
    }
    const dx = x - xx;
    const dy = y - yy;
    return Math.sqrt(dx * dx + dy * dy);
  }

  function selectElement(element) {
    state.selectedElement = element;
    render();
  }

  function selectAll() {
    // Select all - for now just render all as selected
    render();
  }

  function deleteSelected() {
    if (!state.selectedElement) return;
    state.elements = state.elements.filter(
      (el) => el.id !== state.selectedElement.id,
    );
    state.selectedElement = null;
    saveToHistory();
    render();
  }

  function copyElement() {
    if (state.selectedElement) {
      state.clipboard = JSON.parse(JSON.stringify(state.selectedElement));
    }
  }

  function cutElement() {
    copyElement();
    deleteSelected();
  }

  function pasteElement() {
    if (!state.clipboard) return;
    const newElement = JSON.parse(JSON.stringify(state.clipboard));
    newElement.id = generateId();
    newElement.x = (newElement.x || 0) + 20;
    newElement.y = (newElement.y || 0) + 20;
    if (newElement.x1 !== undefined) {
      newElement.x1 += 20;
      newElement.y1 += 20;
      newElement.x2 += 20;
      newElement.y2 += 20;
    }
    state.elements.push(newElement);
    state.selectedElement = newElement;
    saveToHistory();
    render();
  }

  // =============================================================================
  // RENDERING
  // =============================================================================

  function render() {
    if (!ctx || !canvas) return;

    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.save();
    ctx.translate(state.panX, state.panY);
    ctx.scale(state.zoom, state.zoom);

    // Draw grid
    if (state.gridEnabled) {
      drawGrid();
    }

    // Draw elements
    for (const element of state.elements) {
      drawElement(element);
    }

    // Draw selection
    if (state.selectedElement) {
      drawSelection(state.selectedElement);
    }

    ctx.restore();
  }

  function drawGrid() {
    const gridSize = state.gridSize;
    const width = canvas.width / state.zoom;
    const height = canvas.height / state.zoom;

    ctx.strokeStyle = "#e0e0e0";
    ctx.lineWidth = 0.5;

    for (let x = 0; x < width; x += gridSize) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }

    for (let y = 0; y < height; y += gridSize) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }
  }

  function drawElement(el) {
    ctx.strokeStyle = el.color || state.color;
    ctx.fillStyle = el.fillColor || "transparent";
    ctx.lineWidth = el.strokeWidth || state.strokeWidth;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";

    switch (el.type) {
      case "rectangle":
      case "frame":
        ctx.beginPath();
        ctx.rect(el.x, el.y, el.width, el.height);
        if (el.fillColor && el.fillColor !== "transparent") {
          ctx.fill();
        }
        ctx.stroke();
        break;

      case "ellipse":
        ctx.beginPath();
        ctx.ellipse(
          el.x + el.width / 2,
          el.y + el.height / 2,
          el.width / 2,
          el.height / 2,
          0,
          0,
          Math.PI * 2,
        );
        if (el.fillColor && el.fillColor !== "transparent") {
          ctx.fill();
        }
        ctx.stroke();
        break;

      case "line":
        ctx.beginPath();
        ctx.moveTo(el.x1, el.y1);
        ctx.lineTo(el.x2, el.y2);
        ctx.stroke();
        break;

      case "arrow":
        drawArrow(el.x1, el.y1, el.x2, el.y2);
        break;

      case "path":
        if (el.points && el.points.length > 0) {
          ctx.beginPath();
          ctx.moveTo(el.points[0].x, el.points[0].y);
          for (let i = 1; i < el.points.length; i++) {
            ctx.lineTo(el.points[i].x, el.points[i].y);
          }
          ctx.stroke();
        }
        break;

      case "text":
        ctx.font = `${el.fontSize || 16}px ${el.fontFamily || "Inter"}`;
        ctx.fillStyle = el.color || "#000000";
        ctx.fillText(el.text, el.x, el.y);
        break;

      case "sticky":
        ctx.fillStyle = el.color || "#ffeb3b";
        ctx.fillRect(el.x, el.y, el.width, el.height);
        ctx.strokeStyle = "#c0a000";
        ctx.strokeRect(el.x, el.y, el.width, el.height);
        ctx.fillStyle = "#000000";
        ctx.font = "14px Inter";
        wrapText(el.text, el.x + 10, el.y + 25, el.width - 20, 18);
        break;
    }
  }

  function drawArrow(x1, y1, x2, y2) {
    const headLength = 15;
    const angle = Math.atan2(y2 - y1, x2 - x1);

    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(x2, y2);
    ctx.lineTo(
      x2 - headLength * Math.cos(angle - Math.PI / 6),
      y2 - headLength * Math.sin(angle - Math.PI / 6),
    );
    ctx.moveTo(x2, y2);
    ctx.lineTo(
      x2 - headLength * Math.cos(angle + Math.PI / 6),
      y2 - headLength * Math.sin(angle + Math.PI / 6),
    );
    ctx.stroke();
  }

  function drawPreviewShape(x1, y1, x2, y2) {
    ctx.save();
    ctx.translate(state.panX, state.panY);
    ctx.scale(state.zoom, state.zoom);
    ctx.strokeStyle = state.color;
    ctx.lineWidth = state.strokeWidth;
    ctx.setLineDash([5, 5]);

    switch (state.tool) {
      case "rectangle":
      case "frame":
        ctx.strokeRect(
          Math.min(x1, x2),
          Math.min(y1, y2),
          Math.abs(x2 - x1),
          Math.abs(y2 - y1),
        );
        break;
      case "ellipse":
        ctx.beginPath();
        ctx.ellipse(
          (x1 + x2) / 2,
          (y1 + y2) / 2,
          Math.abs(x2 - x1) / 2,
          Math.abs(y2 - y1) / 2,
          0,
          0,
          Math.PI * 2,
        );
        ctx.stroke();
        break;
      case "line":
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.stroke();
        break;
      case "arrow":
        drawArrow(x1, y1, x2, y2);
        break;
    }

    ctx.restore();
  }

  function drawSelection(el) {
    ctx.strokeStyle = "#2196f3";
    ctx.lineWidth = 2 / state.zoom;
    ctx.setLineDash([5 / state.zoom, 5 / state.zoom]);

    let x, y, w, h;
    if (el.type === "line" || el.type === "arrow") {
      x = Math.min(el.x1, el.x2) - 5;
      y = Math.min(el.y1, el.y2) - 5;
      w = Math.abs(el.x2 - el.x1) + 10;
      h = Math.abs(el.y2 - el.y1) + 10;
    } else if (el.type === "path") {
      const bounds = getPathBounds(el.points);
      x = bounds.minX - 5;
      y = bounds.minY - 5;
      w = bounds.maxX - bounds.minX + 10;
      h = bounds.maxY - bounds.minY + 10;
    } else {
      x = el.x - 5;
      y = el.y - 5;
      w = (el.width || 100) + 10;
      h = (el.height || 20) + 10;
    }

    ctx.strokeRect(x, y, w, h);
    ctx.setLineDash([]);
  }

  function getPathBounds(points) {
    if (!points || points.length === 0) {
      return { minX: 0, minY: 0, maxX: 0, maxY: 0 };
    }
    let minX = Infinity,
      minY = Infinity,
      maxX = -Infinity,
      maxY = -Infinity;
    for (const pt of points) {
      minX = Math.min(minX, pt.x);
      minY = Math.min(minY, pt.y);
      maxX = Math.max(maxX, pt.x);
      maxY = Math.max(maxY, pt.y);
    }
    return { minX, minY, maxX, maxY };
  }

  function wrapText(text, x, y, maxWidth, lineHeight) {
    const words = text.split(" ");
    let line = "";
    for (const word of words) {
      const testLine = line + word + " ";
      const metrics = ctx.measureText(testLine);
      if (metrics.width > maxWidth && line !== "") {
        ctx.fillText(line, x, y);
        line = word + " ";
        y += lineHeight;
      } else {
        line = testLine;
      }
    }
    ctx.fillText(line, x, y);
  }

  // =============================================================================
  // ZOOM CONTROLS
  // =============================================================================

  function zoomIn() {
    state.zoom = Math.min(5, state.zoom + 0.1);
    updateZoomDisplay();
    render();
  }

  function zoomOut() {
    state.zoom = Math.max(0.1, state.zoom - 0.1);
    updateZoomDisplay();
    render();
  }

  function resetZoom() {
    state.zoom = 1;
    state.panX = 0;
    state.panY = 0;
    updateZoomDisplay();
    render();
  }

  function fitToScreen() {
    // Calculate bounds of all elements
    if (state.elements.length === 0) {
      resetZoom();
      return;
    }

    let minX = Infinity,
      minY = Infinity,
      maxX = -Infinity,
      maxY = -Infinity;
    for (const el of state.elements) {
      if (el.x !== undefined) {
        minX = Math.min(minX, el.x);
        minY = Math.min(minY, el.y);
        maxX = Math.max(maxX, el.x + (el.width || 100));
        maxY = Math.max(maxY, el.y + (el.height || 50));
      }
    }

    const contentWidth = maxX - minX + 100;
    const contentHeight = maxY - minY + 100;
    const scaleX = canvas.width / contentWidth;
    const scaleY = canvas.height / contentHeight;
    state.zoom = Math.min(scaleX, scaleY, 1);
    state.panX = -minX * state.zoom + 50;
    state.panY = -minY * state.zoom + 50;

    updateZoomDisplay();
    render();
  }

  function updateZoomDisplay() {
    const el = document.getElementById("zoom-level");
    if (el) {
      el.textContent = Math.round(state.zoom * 100) + "%";
    }
  }

  // =============================================================================
  // HISTORY (UNDO/REDO)
  // =============================================================================

  function saveToHistory() {
    // Remove any redo states
    state.history = state.history.slice(0, state.historyIndex + 1);
    // Save current state
    state.history.push(JSON.stringify(state.elements));
    state.historyIndex = state.history.length - 1;
    // Limit history size
    if (state.history.length > 50) {
      state.history.shift();
      state.historyIndex--;
    }
  }

  function undo() {
    if (state.historyIndex > 0) {
      state.historyIndex--;
      state.elements = JSON.parse(state.history[state.historyIndex]);
      state.selectedElement = null;
      render();
    }
  }

  function redo() {
    if (state.historyIndex < state.history.length - 1) {
      state.historyIndex++;
      state.elements = JSON.parse(state.history[state.historyIndex]);
      state.selectedElement = null;
      render();
    }
  }

  // =============================================================================
  // CLEAR CANVAS
  // =============================================================================

  function clearCanvas() {
    if (!confirm("Clear the entire canvas? This cannot be undone.")) return;
    state.elements = [];
    state.selectedElement = null;
    saveToHistory();
    render();
  }

  // =============================================================================
  // COLOR & STYLE
  // =============================================================================

  function setColor(color) {
    state.color = color;
    if (state.selectedElement) {
      state.selectedElement.color = color;
      saveToHistory();
      render();
    }
  }

  function setFillColor(color) {
    state.fillColor = color;
    if (state.selectedElement) {
      state.selectedElement.fillColor = color;
      saveToHistory();
      render();
    }
  }

  function setStrokeWidth(width) {
    state.strokeWidth = parseInt(width);
    if (state.selectedElement) {
      state.selectedElement.strokeWidth = state.strokeWidth;
      saveToHistory();
      render();
    }
  }

  function toggleGrid() {
    state.gridEnabled = !state.gridEnabled;
    render();
  }

  // =============================================================================
  // SAVE/LOAD
  // =============================================================================

  function loadFromUrl() {
    const urlParams = new URLSearchParams(window.location.search);
    const canvasId = urlParams.get("id");
    if (canvasId) {
      loadCanvas(canvasId);
    }
  }

  async function loadCanvas(canvasId) {
    try {
      const response = await fetch(`/api/canvas/${canvasId}`);
      if (response.ok) {
        const data = await response.json();
        state.canvasId = canvasId;
        state.canvasName = data.name || "Untitled Canvas";
        state.elements = data.elements || [];
        saveToHistory();
        render();
      }
    } catch (e) {
      console.error("Failed to load canvas:", e);
    }
  }

  async function saveCanvas() {
    try {
      const response = await fetch(
        "/api/canvas" + (state.canvasId ? `/${state.canvasId}` : ""),
        {
          method: state.canvasId ? "PUT" : "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            name: state.canvasName,
            elements: state.elements,
          }),
        },
      );
      if (response.ok) {
        const data = await response.json();
        if (data.id) {
          state.canvasId = data.id;
          window.history.replaceState({}, "", `?id=${state.canvasId}`);
        }
        showNotification("Canvas saved", "success");
      }
    } catch (e) {
      console.error("Failed to save canvas:", e);
      showNotification("Failed to save canvas", "error");
    }
  }

  function exportCanvas(format) {
    if (format === "png" || format === "jpg") {
      const dataUrl = canvas.toDataURL(`image/${format}`);
      const link = document.createElement("a");
      link.download = `${state.canvasName}.${format}`;
      link.href = dataUrl;
      link.click();
    } else if (format === "json") {
      const data = JSON.stringify(
        { name: state.canvasName, elements: state.elements },
        null,
        2,
      );
      const blob = new Blob([data], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.download = `${state.canvasName}.json`;
      link.href = url;
      link.click();
      URL.revokeObjectURL(url);
    }
  }

  // =============================================================================
  // SHARING & COLLABORATION
  // =============================================================================

  function shareCanvas() {
    if (!state.canvasId) {
      // Save canvas first if not saved
      saveCanvas().then(() => {
        showShareDialog();
      });
    } else {
      showShareDialog();
    }
  }

  function showShareDialog() {
    const modal = document.getElementById("share-modal");
    if (modal) {
      if (modal.showModal) {
        modal.showModal();
      } else {
        modal.classList.add("open");
        modal.style.display = "flex";
      }
      // Generate share link
      const shareUrl = `${window.location.origin}/canvas?id=${state.canvasId}`;
      const shareLinkInput = document.getElementById("share-link");
      if (shareLinkInput) {
        shareLinkInput.value = shareUrl;
      }
    } else {
      // Fallback: copy link to clipboard
      const shareUrl = `${window.location.origin}/canvas?id=${state.canvasId || "new"}`;
      navigator.clipboard
        .writeText(shareUrl)
        .then(() => {
          showNotification("Share link copied to clipboard", "success");
        })
        .catch(() => {
          showNotification(
            "Canvas ID: " + (state.canvasId || "unsaved"),
            "info",
          );
        });
    }
  }

  // =============================================================================
  // PROPERTIES PANEL
  // =============================================================================

  function togglePropertiesPanel() {
    const panel = document.getElementById("properties-panel");
    if (panel) {
      panel.classList.toggle("collapsed");
      const isCollapsed = panel.classList.contains("collapsed");
      // Update toggle button icon if needed
      const toggleBtn = panel.querySelector(".panel-toggle span");
      if (toggleBtn) {
        toggleBtn.textContent = isCollapsed ? "⚙️" : "✕";
      }
    }
  }

  // =============================================================================
  // LAYERS MANAGEMENT
  // =============================================================================

  let layers = [
    { id: "layer_1", name: "Layer 1", visible: true, locked: false },
  ];
  let activeLayerId = "layer_1";

  function addLayer() {
    const newId = "layer_" + (layers.length + 1);
    const newLayer = {
      id: newId,
      name: "Layer " + (layers.length + 1),
      visible: true,
      locked: false,
    };
    layers.push(newLayer);
    activeLayerId = newId;
    renderLayers();
    showNotification("Layer added", "success");
  }

  function renderLayers() {
    const layersList = document.getElementById("layers-list");
    if (!layersList) return;

    layersList.innerHTML = layers
      .map(
        (layer) => `
        <div class="layer-item ${layer.id === activeLayerId ? "active" : ""}"
             data-layer-id="${layer.id}"
             onclick="selectLayer('${layer.id}')">
            <span class="layer-visibility" onclick="event.stopPropagation(); toggleLayerVisibility('${layer.id}')">${layer.visible ? "👁️" : "👁️‍🗨️"}</span>
            <span class="layer-name">${layer.name}</span>
            <span class="layer-lock" onclick="event.stopPropagation(); toggleLayerLock('${layer.id}')">${layer.locked ? "🔒" : "🔓"}</span>
        </div>
      `,
      )
      .join("");
  }

  function selectLayer(layerId) {
    activeLayerId = layerId;
    renderLayers();
  }

  function toggleLayerVisibility(layerId) {
    const layer = layers.find((l) => l.id === layerId);
    if (layer) {
      layer.visible = !layer.visible;
      renderLayers();
      render();
    }
  }

  function toggleLayerLock(layerId) {
    const layer = layers.find((l) => l.id === layerId);
    if (layer) {
      layer.locked = !layer.locked;
      renderLayers();
    }
  }

  // =============================================================================
  // CLIPBOARD & DUPLICATE
  // =============================================================================

  function duplicateSelected() {
    if (!state.selectedElement) {
      showNotification("No element selected", "warning");
      return;
    }

    const original = state.selectedElement;
    const duplicate = JSON.parse(JSON.stringify(original));
    duplicate.id = generateId();
    // Offset the duplicate slightly
    if (duplicate.x !== undefined) duplicate.x += 20;
    if (duplicate.y !== undefined) duplicate.y += 20;

    state.elements.push(duplicate);
    state.selectedElement = duplicate;
    saveToHistory();
    render();
    showNotification("Element duplicated", "success");
  }

  function copySelected() {
    if (!state.selectedElement) {
      showNotification("No element selected", "warning");
      return;
    }
    state.clipboard = JSON.parse(JSON.stringify(state.selectedElement));
    showNotification("Element copied", "success");
  }

  function pasteClipboard() {
    if (!state.clipboard) {
      showNotification("Nothing to paste", "warning");
      return;
    }

    const pasted = JSON.parse(JSON.stringify(state.clipboard));
    pasted.id = generateId();
    // Offset the pasted element
    if (pasted.x !== undefined) pasted.x += 20;
    if (pasted.y !== undefined) pasted.y += 20;

    state.elements.push(pasted);
    state.selectedElement = pasted;
    saveToHistory();
    render();
    showNotification("Element pasted", "success");
  }

  // =============================================================================
  // ELEMENT ORDERING
  // =============================================================================

  function bringToFront() {
    if (!state.selectedElement) return;
    const index = state.elements.findIndex(
      (e) => e.id === state.selectedElement.id,
    );
    if (index !== -1 && index < state.elements.length - 1) {
      state.elements.splice(index, 1);
      state.elements.push(state.selectedElement);
      saveToHistory();
      render();
    }
  }

  function sendToBack() {
    if (!state.selectedElement) return;
    const index = state.elements.findIndex(
      (e) => e.id === state.selectedElement.id,
    );
    if (index > 0) {
      state.elements.splice(index, 1);
      state.elements.unshift(state.selectedElement);
      saveToHistory();
      render();
    }
  }

  // =============================================================================
  // EXPORT MODAL
  // =============================================================================

  function showExportModal() {
    const modal = document.getElementById("export-modal");
    if (modal) {
      if (modal.showModal) {
        modal.showModal();
      } else {
        modal.classList.add("open");
        modal.style.display = "flex";
      }
    }
  }

  function closeExportModal() {
    const modal = document.getElementById("export-modal");
    if (modal) {
      if (modal.close) {
        modal.close();
      } else {
        modal.classList.remove("open");
        modal.style.display = "none";
      }
    }
  }

  function doExport() {
    const formatSelect = document.getElementById("export-format");
    const format = formatSelect ? formatSelect.value : "png";
    exportCanvas(format);
    closeExportModal();
  }

  // =============================================================================
  // UTILITIES
  // =============================================================================

  function generateId() {
    return "el_" + Math.random().toString(36).substr(2, 9);
  }

  function showNotification(message, type) {
    if (typeof window.showNotification === "function") {
      window.showNotification(message, type);
    } else if (typeof window.GBAlerts !== "undefined") {
      if (type === "success") window.GBAlerts.success("Canvas", message);
      else if (type === "error") window.GBAlerts.error("Canvas", message);
      else window.GBAlerts.info("Canvas", message);
    } else {
      console.log(`[${type}] ${message}`);
    }
  }

  // =============================================================================
  // EXPORT TO WINDOW
  // =============================================================================

  window.selectTool = selectTool;
  window.zoomIn = zoomIn;
  window.zoomOut = zoomOut;
  window.resetZoom = resetZoom;
  window.fitToScreen = fitToScreen;
  window.undo = undo;
  window.redo = redo;
  window.clearCanvas = clearCanvas;
  window.setColor = setColor;
  window.setFillColor = setFillColor;
  window.setStrokeWidth = setStrokeWidth;
  window.toggleGrid = toggleGrid;
  window.saveCanvas = saveCanvas;
  window.exportCanvas = exportCanvas;
  window.deleteSelected = deleteSelected;
  window.copyElement = copyElement;
  window.cutElement = cutElement;
  window.pasteElement = pasteElement;

  // Sharing & Collaboration
  window.shareCanvas = shareCanvas;

  // Properties Panel
  window.togglePropertiesPanel = togglePropertiesPanel;

  // Layers
  window.addLayer = addLayer;
  window.selectLayer = selectLayer;
  window.toggleLayerVisibility = toggleLayerVisibility;
  window.toggleLayerLock = toggleLayerLock;

  // Clipboard & Duplicate
  window.duplicateSelected = duplicateSelected;
  window.copySelected = copySelected;
  window.pasteClipboard = pasteClipboard;

  // Element Ordering
  window.bringToFront = bringToFront;
  window.sendToBack = sendToBack;

  // Export Modal
  window.showExportModal = showExportModal;
  window.closeExportModal = closeExportModal;
  window.doExport = doExport;

  // =============================================================================
  // INITIALIZE
  // =============================================================================

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
