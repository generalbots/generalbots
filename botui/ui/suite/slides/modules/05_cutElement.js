// slides/modules/05_cutElement.js
"use strict";

// Functions: cutElement, copyElement, pasteElement, duplicateElement, deleteElement, bringToFront, sendToBack, setTextColor, setFillColor, setTextAlign, undo, redo, saveToHistory, restoreFromHistory, generateId, escapeHtml, scheduleAutoSave, savePresentation, broadcastChange, broadcastCursor, connectWebSocket, handleWebSocketMessage, addCollaborator, removeCollaborator, updateRemoteCursor, renderCollaborators, getUserId, getUserName, showTransitionsModal, selectTransition, updateDurationDisplay, applyTransition

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

  function broadcastChange(type, data) {
    if (!state.ws || state.ws.readyState !== WebSocket.OPEN) return;
    try {
      state.ws.send(JSON.stringify({ type, userId: getUserId(), ...data }));
    } catch (e) {
      console.error("broadcastChange error:", e);
    }
  }

  function broadcastCursor(e) {
    if (!state.ws || state.ws.readyState !== WebSocket.OPEN) return;
    try {
      const rect = document.getElementById("slideCanvas").getBoundingClientRect();
      state.ws.send(JSON.stringify({
        type: "cursor",
        userId: getUserId(),
        userName: getUserName(),
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
      }));
    } catch (err) {
      console.error("broadcastCursor error:", err);
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
      case "cursor":
        if (msg.userId !== getUserId()) {
          updateRemoteCursor(msg);
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
    const indicator = document.getElementById("cursor-" + userId);
    if (indicator) indicator.remove();
  }

  function updateRemoteCursor(msg) {
    let indicator = document.getElementById("cursor-" + msg.userId);
    if (!indicator) {
      indicator = document.createElement("div");
      indicator.id = "cursor-" + msg.userId;
      indicator.className = "remote-cursor";
      indicator.style.cssText = "position:absolute;pointer-events:none;z-index:9999;transition:left 0.1s,top 0.1s;";
      indicator.innerHTML = `<svg width="16" height="16" viewBox="0 0 16 16"><path d="M0 0 L16 6 L6 8 L8 16 Z" fill="${msg.color || "#4285f4"}"/></svg><span style="font-size:10px;background:${msg.color || "#4285f4"};color:#fff;padding:1px 4px;border-radius:3px;margin-left:2px;white-space:nowrap;">${escapeHtml(msg.userName || "")}</span>`;
      const canvas = document.getElementById("slideCanvas");
      if (canvas) canvas.appendChild(indicator);
    }
    indicator.style.left = (msg.x || 0) + "px";
    indicator.style.top = (msg.y || 0) + "px";
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

