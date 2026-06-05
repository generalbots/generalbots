// sheet/modules/06_loadFromUrlParams.js
"use strict";

// Functions: loadFromUrlParams, handleBeforeUnload, connectWebSocket, handleWebSocketMessage, broadcastChange, updateRemoteCursor, addCollaborator, removeCollaborator, renderCollaborators, getUserId, getUserName, escapeHtml, toggleChatPanel, handleChatSubmit, handleSuggestionClick, addChatMessage, processAICommand, sortAscending, sortDescending, sortSelection, connectChatWebSocket, handleNumberFormatChange, applyNumberFormat, formatValue

  async function loadFromUrlParams() {
    const hash = window.location.hash;
    if (!hash) return;

    const params = new URLSearchParams(hash.substring(1));
    const sheetId = params.get("id");

    if (sheetId) {
      try {
        const response = await fetch(`/api/sheet/${sheetId}`);
        if (response.ok) {
          const data = await response.json();
          state.sheetId = sheetId;
          state.sheetName = data.name || "Untitled Spreadsheet";
          state.worksheets = data.worksheets || [{ name: "Sheet1", data: {} }];

          if (elements.sheetName) elements.sheetName.value = state.sheetName;

          renderWorksheetTabs();
          renderAllCells();
        }
      } catch (e) {
        console.error("Load failed:", e);
      }
    }
  }

  function handleBeforeUnload(e) {
    if (state.isDirty) {
      e.preventDefault();
      e.returnValue = "";
    }
  }

  function connectWebSocket() {
    if (!state.sheetId) return;

    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${protocol}//${window.location.host}/api/sheet/ws/${state.sheetId}`;

    try {
      state.ws = new WebSocket(wsUrl);

      state.ws.onopen = () => {
        state.ws.send(
          JSON.stringify({
            type: "join",
            sheetId: state.sheetId,
            userId: getUserId(),
            userName: getUserName(),
          }),
        );
      };

      state.ws.onmessage = (event) => {
        const msg = JSON.parse(event.data);
        handleWebSocketMessage(msg);
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
      case "cellChange":
        if (msg.userId !== getUserId()) {
          const ws = state.worksheets[state.activeWorksheet];
          const key = `${msg.row},${msg.col}`;
          if (msg.value) {
            ws.data[key] = { value: msg.value };
          } else {
            delete ws.data[key];
          }
          renderCell(msg.row, msg.col);
        }
        break;
      case "cursor":
        updateRemoteCursor(msg);
        break;
      case "userJoined":
        addCollaborator(msg.user);
        break;
      case "userLeft":
        removeCollaborator(msg.userId);
        break;
    }
  }

  function broadcastChange(type, data) {
    if (state.ws?.readyState === WebSocket.OPEN) {
      state.ws.send(
        JSON.stringify({
          type,
          sheetId: state.sheetId,
          userId: getUserId(),
          ...data,
        }),
      );
    }
  }

  function updateRemoteCursor(msg) {
    let cursor = document.getElementById(`cursor-${msg.userId}`);
    if (!cursor) {
      cursor = document.createElement("div");
      cursor.id = `cursor-${msg.userId}`;
      cursor.className = "cursor-indicator";
      cursor.style.borderColor = msg.color || "#4285f4";
      cursor.innerHTML = `<div class="cursor-label" style="background:${msg.color || "#4285f4"}">${escapeHtml(msg.userName)}</div>`;
      elements.cursorIndicators?.appendChild(cursor);
    }

    const cell = elements.cells.querySelector(
      `[data-row="${msg.row}"][data-col="${msg.col}"]`,
    );
    if (cell) {
      const rect = cell.getBoundingClientRect();
      const container = elements.cellsContainer.getBoundingClientRect();
      cursor.style.left = rect.left - container.left + "px";
      cursor.style.top = rect.top - container.top + "px";
      cursor.style.width = rect.width + "px";
      cursor.style.height = rect.height + "px";
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
    document.getElementById(`cursor-${userId}`)?.remove();
    renderCollaborators();
  }

  function renderCollaborators() {
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

  function escapeHtml(str) {
    if (!str) return "";
    return String(str)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  function toggleChatPanel() {
    state.chatPanelOpen = !state.chatPanelOpen;
    elements.chatPanel.classList.toggle("collapsed", !state.chatPanelOpen);
  }

  function handleChatSubmit(e) {
    e.preventDefault();
    const message = elements.chatInput.value.trim();
    if (!message) return;

    addChatMessage("user", message);
    elements.chatInput.value = "";

    processAICommand(message);
  }

  function handleSuggestionClick(action) {
    const commands = {
      sum: "Sum column B",
      format: "Format selected cells as currency",
      chart: "Create a bar chart from selected data",
      sort: "Sort selected column A to Z",
    };

    const message = commands[action] || action;
    addChatMessage("user", message);
    processAICommand(message);
  }

  function addChatMessage(role, content) {
    const div = document.createElement("div");
    div.className = `chat-message ${role}`;
    div.innerHTML = `<div class="message-bubble">${escapeHtml(content)}</div>`;
    elements.chatMessages.appendChild(div);
    elements.chatMessages.scrollTop = elements.chatMessages.scrollHeight;
  }

  async function processAICommand(command) {
    const lower = command.toLowerCase();
    let response = "";

    if (lower.includes("sum")) {
      const { start, end } = state.selection;
      const colLetter = getColName(start.col);
      const formula = `=SUM(${colLetter}${start.row + 1}:${colLetter}${end.row + 1})`;

      const resultRow = end.row + 1;
      if (resultRow < CONFIG.ROWS) {
        setCellValue(resultRow, start.col, formula);
        renderCell(resultRow, start.col);
        selectCell(resultRow, start.col);
        response = `Done! Added SUM formula in cell ${getColName(start.col)}${resultRow + 1}`;
      } else {
        response = "Cannot add sum - no row available below selection";
      }
    } else if (lower.includes("currency") || lower.includes("$")) {
      formatCells("currency");
      response = "Formatted selected cells as currency";
    } else if (lower.includes("percent") || lower.includes("%")) {
      formatCells("percent");
      response = "Formatted selected cells as percentage";
    } else if (lower.includes("bold")) {
      formatCells("bold");
      response = "Applied bold formatting to selected cells";
    } else if (lower.includes("italic")) {
      formatCells("italic");
      response = "Applied italic formatting to selected cells";
    } else if (lower.includes("sort") && lower.includes("z")) {
      sortDescending();
      response = "Sorted selection Z to A";
    } else if (lower.includes("sort")) {
      sortAscending();
      response = "Sorted selection A to Z";
    } else if (lower.includes("chart")) {
      showModal("chartModal");
      response =
        "Opening chart dialog. Select chart type and configure options.";
    } else if (lower.includes("clear")) {
      clearCells();
      response = "Cleared selected cells";
    } else if (lower.includes("average") || lower.includes("avg")) {
      const { start, end } = state.selection;
      const colLetter = getColName(start.col);
      const formula = `=AVERAGE(${colLetter}${start.row + 1}:${colLetter}${end.row + 1})`;
      const resultRow = end.row + 1;
      if (resultRow < CONFIG.ROWS) {
        setCellValue(resultRow, start.col, formula);
        renderCell(resultRow, start.col);
        selectCell(resultRow, start.col);
        response = `Done! Added AVERAGE formula in cell ${getColName(start.col)}${resultRow + 1}`;
      }
    } else {
      try {
        const res = await fetch("/api/sheet/ai", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            command,
            selection: state.selection,
            activeCell: state.activeCell,
            sheetId: state.sheetId,
          }),
        });
        const data = await res.json();
        response = data.response || "I processed your request";
      } catch {
        response =
          "I can help you with:\n• Sum/Average a column\n• Format as currency or percent\n• Bold/Italic formatting\n• Sort data\n• Create charts\n• Clear cells";
      }
    }

    addChatMessage("assistant", response);
  }

  function sortAscending() {
    sortSelection(true);
  }

  function sortDescending() {
    sortSelection(false);
  }

  function sortSelection(ascending) {
    saveToHistory();
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];

    const rows = [];
    for (let r = start.row; r <= end.row; r++) {
      const rowData = [];
      for (let c = start.col; c <= end.col; c++) {
        rowData.push(getCellData(r, c));
      }
      rows.push({ row: r, data: rowData });
    }

    rows.sort((a, b) => {
      const valA = a.data[0]?.value || a.data[0]?.formula || "";
      const valB = b.data[0]?.value || b.data[0]?.formula || "";
      const numA = parseFloat(valA);
      const numB = parseFloat(valB);

      if (!isNaN(numA) && !isNaN(numB)) {
        return ascending ? numA - numB : numB - numA;
      }
      return ascending
        ? String(valA).localeCompare(String(valB))
        : String(valB).localeCompare(String(valA));
    });

    rows.forEach((rowObj, i) => {
      const targetRow = start.row + i;
      rowObj.data.forEach((cellData, j) => {
        const targetCol = start.col + j;
        const key = `${targetRow},${targetCol}`;
        if (cellData) {
          ws.data[key] = cellData;
        } else {
          delete ws.data[key];
        }
      });
    });

    renderAllCells();
    state.isDirty = true;
    scheduleAutoSave();
  }

  function connectChatWebSocket() {
    // Chat uses main WebSocket connection
  }

  function handleNumberFormatChange(e) {
    const format = e.target.value;
    if (format === "custom") {
      showModal("customNumberFormatModal");
      return;
    }
    applyNumberFormat(format);
  }

  function applyNumberFormat(format) {
    saveToHistory();
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];

    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        const key = `${r},${c}`;
        if (!ws.data[key]) ws.data[key] = { value: "" };
        ws.data[key].format = format;

        const rawValue = ws.data[key].rawValue || ws.data[key].value;
        if (rawValue) {
          ws.data[key].rawValue = rawValue;
          ws.data[key].value = formatValue(rawValue, format);
        }
        renderCell(r, c);
      }
    }

    state.isDirty = true;
    scheduleAutoSave();
  }

  function formatValue(value, format) {
    const num = parseFloat(value);
    if (isNaN(num) && format !== "text") return value;

    switch (format) {
      case "number":
        return num.toLocaleString("en-US", {
          minimumFractionDigits: state.decimalPlaces,
          maximumFractionDigits: state.decimalPlaces,
        });
      case "currency":
        return num.toLocaleString("en-US", {
          style: "currency",
          currency: "USD",
          minimumFractionDigits: state.decimalPlaces,
        });
      case "accounting":
        const formatted = Math.abs(num).toLocaleString("en-US", {
          style: "currency",
          currency: "USD",
        });
        return num < 0 ? `(${formatted})` : formatted;
      case "percent":
        return (num * 100).toFixed(state.decimalPlaces) + "%";
      case "scientific":
        return num.toExponential(state.decimalPlaces);
      case "date_short":
        const d1 = new Date(num);
        return isNaN(d1.getTime()) ? value : d1.toLocaleDateString("en-US");
      case "date_long":
        const d2 = new Date(num);
        return isNaN(d2.getTime())
          ? value
          : d2.toLocaleDateString("en-US", {
            year: "numeric",
            month: "long",
            day: "numeric",
          });
      case "time":
        const d3 = new Date(num);
        return isNaN(d3.getTime())
          ? value
          : d3.toLocaleTimeString("en-US", {
            hour: "numeric",
            minute: "2-digit",
          });
      case "datetime":
        const d4 = new Date(num);
        return isNaN(d4.getTime()) ? value : d4.toLocaleString("en-US");
      case "fraction":
        return toFraction(num);
      case "text":
        return String(value);
      default:
        return value;
    }
  }

