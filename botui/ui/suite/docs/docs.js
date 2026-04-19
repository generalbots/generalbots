(function () {
  "use strict";

  const CONFIG = {
    AUTOSAVE_DELAY: 3000,
    MAX_HISTORY: 50,
    WS_RECONNECT_DELAY: 5000,
  };

  const state = {
    docId: null,
    docTitle: "Untitled Document",
    content: "",
    history: [],
    historyIndex: -1,
    isDirty: false,
    autoSaveTimer: null,
    ws: null,
    collaborators: [],

    driveSource: null,
    zoom: 100,
    findMatches: [],
    findMatchIndex: -1,
  };

  const elements = {};

  function init() {
    cacheElements();
    bindEvents();
    loadFromUrlParams();
    setupToolbar();
    setupKeyboardShortcuts();
    updateWordCount();
    connectWebSocket();
  }

  function cacheElements() {
    elements.app = document.getElementById("docs-app");
    elements.docName = document.getElementById("docName");
    elements.editorContent = document.getElementById("editorContent");
    elements.editorPage = document.getElementById("editorPage");
    elements.collaborators = document.getElementById("collaborators");
    elements.pageInfo = document.getElementById("pageInfo");
    elements.wordCount = document.getElementById("wordCount");
    elements.charCount = document.getElementById("charCount");
    elements.saveStatus = document.getElementById("saveStatus");
    elements.zoomLevel = document.getElementById("zoomLevel");

    elements.shareModal = document.getElementById("shareModal");
    elements.linkModal = document.getElementById("linkModal");
    elements.imageModal = document.getElementById("imageModal");
    elements.tableModal = document.getElementById("tableModal");
    elements.exportModal = document.getElementById("exportModal");
    elements.findReplaceModal = document.getElementById("findReplaceModal");
    elements.printPreviewModal = document.getElementById("printPreviewModal");
    elements.headerFooterModal = document.getElementById("headerFooterModal");
    elements.editorHeader = document.getElementById("editorHeader");
    elements.editorFooter = document.getElementById("editorFooter");
  }

  function bindEvents() {
    if (elements.editorContent) {
      elements.editorContent.addEventListener("input", handleEditorInput);
      elements.editorContent.addEventListener("keydown", handleEditorKeydown);
      elements.editorContent.addEventListener("paste", handlePaste);
    }

    if (elements.docName) {
      elements.docName.addEventListener("change", handleDocNameChange);
      elements.docName.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          elements.editorContent?.focus();
        }
      });
    }

    document.getElementById("undoBtn")?.addEventListener("click", undo);
    document.getElementById("redoBtn")?.addEventListener("click", redo);
    document
      .getElementById("boldBtn")
      ?.addEventListener("click", () => execCommand("bold"));
    document
      .getElementById("italicBtn")
      ?.addEventListener("click", () => execCommand("italic"));
    document
      .getElementById("underlineBtn")
      ?.addEventListener("click", () => execCommand("underline"));
    document
      .getElementById("strikeBtn")
      ?.addEventListener("click", () => execCommand("strikeThrough"));

    document
      .getElementById("alignLeftBtn")
      ?.addEventListener("click", () => execCommand("justifyLeft"));
    document
      .getElementById("alignCenterBtn")
      ?.addEventListener("click", () => execCommand("justifyCenter"));
    document
      .getElementById("alignRightBtn")
      ?.addEventListener("click", () => execCommand("justifyRight"));
    document
      .getElementById("alignJustifyBtn")
      ?.addEventListener("click", () => execCommand("justifyFull"));

    document
      .getElementById("bulletListBtn")
      ?.addEventListener("click", () => execCommand("insertUnorderedList"));
    document
      .getElementById("numberListBtn")
      ?.addEventListener("click", () => execCommand("insertOrderedList"));
    document
      .getElementById("indentBtn")
      ?.addEventListener("click", () => execCommand("indent"));
    document
      .getElementById("outdentBtn")
      ?.addEventListener("click", () => execCommand("outdent"));

    document
      .getElementById("linkBtn")
      ?.addEventListener("click", () => showModal("linkModal"));
    document
      .getElementById("imageBtn")
      ?.addEventListener("click", () => showModal("imageModal"));
    document
      .getElementById("tableBtn")
      ?.addEventListener("click", () => showModal("tableModal"));

    document
      .getElementById("shareBtn")
      ?.addEventListener("click", () => showModal("shareModal"));

    document
      .getElementById("headingSelect")
      ?.addEventListener("change", handleHeadingChange);
    document
      .getElementById("fontFamily")
      ?.addEventListener("change", handleFontFamilyChange);
    document
      .getElementById("fontSize")
      ?.addEventListener("change", handleFontSizeChange);

    document.getElementById("textColorBtn")?.addEventListener("click", () => {
      document.getElementById("textColorPicker")?.click();
    });
    document
      .getElementById("textColorPicker")
      ?.addEventListener("input", handleTextColorChange);
    document.getElementById("highlightBtn")?.addEventListener("click", () => {
      document.getElementById("highlightPicker")?.click();
    });
    document
      .getElementById("highlightPicker")
      ?.addEventListener("input", handleHighlightChange);

    document.getElementById("zoomInBtn")?.addEventListener("click", zoomIn);
    document.getElementById("zoomOutBtn")?.addEventListener("click", zoomOut);



    document.querySelectorAll(".btn-close, .modal").forEach((el) => {
      el.addEventListener("click", (e) => {
        if (e.target === el) closeModals();
      });
    });

    document
      .getElementById("closeShareModal")
      ?.addEventListener("click", () => hideModal("shareModal"));
    document
      .getElementById("closeLinkModal")
      ?.addEventListener("click", () => hideModal("linkModal"));
    document
      .getElementById("closeImageModal")
      ?.addEventListener("click", () => hideModal("imageModal"));
    document
      .getElementById("closeTableModal")
      ?.addEventListener("click", () => hideModal("tableModal"));
    document
      .getElementById("closeExportModal")
      ?.addEventListener("click", () => hideModal("exportModal"));

    document
      .getElementById("insertLinkBtn")
      ?.addEventListener("click", insertLink);
    document
      .getElementById("insertImageBtn")
      ?.addEventListener("click", insertImage);
    document
      .getElementById("insertTableBtn")
      ?.addEventListener("click", insertTable);
    document
      .getElementById("copyLinkBtn")
      ?.addEventListener("click", copyShareLink);

    document.querySelectorAll(".export-option").forEach((btn) => {
      btn.addEventListener("click", () => exportDocument(btn.dataset.format));
    });

    document
      .getElementById("findReplaceBtn")
      ?.addEventListener("click", showFindReplaceModal);
    document
      .getElementById("closeFindReplaceModal")
      ?.addEventListener("click", () => hideModal("findReplaceModal"));
    document.getElementById("findNextBtn")?.addEventListener("click", findNext);
    document.getElementById("findPrevBtn")?.addEventListener("click", findPrev);
    document
      .getElementById("replaceBtn")
      ?.addEventListener("click", replaceOne);
    document
      .getElementById("replaceAllBtn")
      ?.addEventListener("click", replaceAll);
    document
      .getElementById("findInput")
      ?.addEventListener("input", performFind);

    document
      .getElementById("printPreviewBtn")
      ?.addEventListener("click", showPrintPreview);
    document
      .getElementById("closePrintPreviewModal")
      ?.addEventListener("click", () => hideModal("printPreviewModal"));
    document
      .getElementById("printBtn")
      ?.addEventListener("click", printDocument);
    document
      .getElementById("cancelPrintBtn")
      ?.addEventListener("click", () => hideModal("printPreviewModal"));
    document
      .getElementById("printOrientation")
      ?.addEventListener("change", updatePrintPreview);
    document
      .getElementById("printPaperSize")
      ?.addEventListener("change", updatePrintPreview);
    document
      .getElementById("printHeaders")
      ?.addEventListener("change", updatePrintPreview);

    document
      .getElementById("pageBreakBtn")
      ?.addEventListener("click", insertPageBreak);

    document
      .getElementById("headerFooterBtn")
      ?.addEventListener("click", showHeaderFooterModal);
    document
      .getElementById("closeHeaderFooterModal")
      ?.addEventListener("click", () => hideModal("headerFooterModal"));
    document
      .getElementById("applyHeaderFooterBtn")
      ?.addEventListener("click", applyHeaderFooter);
    document
      .getElementById("cancelHeaderFooterBtn")
      ?.addEventListener("click", () => hideModal("headerFooterModal"));
    document
      .getElementById("removeHeaderFooterBtn")
      ?.addEventListener("click", removeHeaderFooter);
    document.querySelectorAll(".hf-tab").forEach((tab) => {
      tab.addEventListener("click", () => switchHfTab(tab.dataset.tab));
    });
    document
      .getElementById("insertPageNum")
      ?.addEventListener("click", () => insertHfField("header", "pageNum"));
    document
      .getElementById("insertDate")
      ?.addEventListener("click", () => insertHfField("header", "date"));
    document
      .getElementById("insertDocTitle")
      ?.addEventListener("click", () => insertHfField("header", "title"));
    document
      .getElementById("insertFooterPageNum")
      ?.addEventListener("click", () => insertHfField("footer", "pageNum"));
    document
      .getElementById("insertFooterDate")
      ?.addEventListener("click", () => insertHfField("footer", "date"));
    document
      .getElementById("insertFooterDocTitle")
      ?.addEventListener("click", () => insertHfField("footer", "title"));

    if (elements.editorHeader) {
      elements.editorHeader.addEventListener("input", handleHeaderFooterInput);
    }
    if (elements.editorFooter) {
      elements.editorFooter.addEventListener("input", handleHeaderFooterInput);
    }

    window.addEventListener("beforeunload", handleBeforeUnload);
  }

  function handleEditorInput() {
    saveToHistory();
    state.isDirty = true;
    updateWordCount();
    scheduleAutoSave();
    broadcastChange();
  }

  function handleDocNameChange() {
    state.docTitle = elements.docName.value || "Untitled Document";
    state.isDirty = true;
    scheduleAutoSave();
  }

  function handleEditorKeydown(e) {
    if (e.ctrlKey || e.metaKey) {
      switch (e.key.toLowerCase()) {
        case "b":
          e.preventDefault();
          execCommand("bold");
          break;
        case "i":
          e.preventDefault();
          execCommand("italic");
          break;
        case "u":
          e.preventDefault();
          execCommand("underline");
          break;
        case "z":
          e.preventDefault();
          if (e.shiftKey) {
            redo();
          } else {
            undo();
          }
          break;
        case "y":
          e.preventDefault();
          redo();
          break;
        case "s":
          e.preventDefault();
          saveDocument();
          break;
      }
    }
  }

  function handlePaste(e) {
    e.preventDefault();
    const text = e.clipboardData.getData("text/plain");
    document.execCommand("insertText", false, text);
  }

  function handleBeforeUnload(e) {
    if (state.isDirty) {
      e.preventDefault();
      e.returnValue = "";
    }
  }

  function setupToolbar() {
    updateToolbarState();
    if (elements.editorContent) {
      elements.editorContent.addEventListener("mouseup", updateToolbarState);
      elements.editorContent.addEventListener("keyup", updateToolbarState);
    }
  }

  function updateToolbarState() {
    document
      .getElementById("boldBtn")
      ?.classList.toggle("active", document.queryCommandState("bold"));
    document
      .getElementById("italicBtn")
      ?.classList.toggle("active", document.queryCommandState("italic"));
    document
      .getElementById("underlineBtn")
      ?.classList.toggle("active", document.queryCommandState("underline"));
    document
      .getElementById("strikeBtn")
      ?.classList.toggle("active", document.queryCommandState("strikeThrough"));
  }

  function setupKeyboardShortcuts() {
    document.addEventListener("keydown", (e) => {
      if (e.target.closest(".chat-input, .modal input")) return;

      if (e.key === "Escape") {
        closeModals();
      }
    });
  }

  function execCommand(command, value = null) {
    elements.editorContent?.focus();
    document.execCommand(command, false, value);
    saveToHistory();
    state.isDirty = true;
    scheduleAutoSave();
    updateToolbarState();
  }

  function handleHeadingChange(e) {
    const value = e.target.value;
    execCommand("formatBlock", value);
  }

  function handleFontFamilyChange(e) {
    execCommand("fontName", e.target.value);
  }

  function handleFontSizeChange(e) {
    execCommand("fontSize", e.target.value);
  }

  function handleTextColorChange(e) {
    execCommand("foreColor", e.target.value);
    const indicator = document.querySelector("#textColorBtn .color-indicator");
    if (indicator) indicator.style.background = e.target.value;
  }

  function handleHighlightChange(e) {
    execCommand("hiliteColor", e.target.value);
    const indicator = document.querySelector("#highlightBtn .color-indicator");
    if (indicator) indicator.style.background = e.target.value;
  }

  function saveToHistory() {
    if (!elements.editorContent) return;
    const content = elements.editorContent.innerHTML;
    if (state.history[state.historyIndex] === content) return;

    state.history = state.history.slice(0, state.historyIndex + 1);
    state.history.push(content);
    if (state.history.length > CONFIG.MAX_HISTORY) {
      state.history.shift();
    } else {
      state.historyIndex++;
    }
  }

  function undo() {
    if (state.historyIndex > 0) {
      state.historyIndex--;
      if (elements.editorContent) {
        elements.editorContent.innerHTML = state.history[state.historyIndex];
      }
      state.isDirty = true;
      updateWordCount();
    }
  }

  function redo() {
    if (state.historyIndex < state.history.length - 1) {
      state.historyIndex++;
      if (elements.editorContent) {
        elements.editorContent.innerHTML = state.history[state.historyIndex];
      }
      state.isDirty = true;
      updateWordCount();
    }
  }

  function updateWordCount() {
    if (!elements.editorContent) return;
    const text = elements.editorContent.innerText || "";
    const words = text
      .trim()
      .split(/\s+/)
      .filter((w) => w.length > 0);
    const chars = text.length;

    if (elements.wordCount) {
      elements.wordCount.textContent = `${words.length} word${words.length !== 1 ? "s" : ""}`;
    }
    if (elements.charCount) {
      elements.charCount.textContent = `${chars} character${chars !== 1 ? "s" : ""}`;
    }

    const pageHeight = 1056;
    const contentHeight = elements.editorContent.scrollHeight || pageHeight;
    const pages = Math.max(1, Math.ceil(contentHeight / pageHeight));
    if (elements.pageInfo) {
      elements.pageInfo.textContent = `Page 1 of ${pages}`;
    }
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
    if (elements.editorPage) {
      elements.editorPage.style.transform = `scale(${state.zoom / 100})`;
      elements.editorPage.style.transformOrigin = "top center";
    }
    if (elements.zoomLevel) {
      elements.zoomLevel.textContent = `${state.zoom}%`;
    }
  }

  function scheduleAutoSave() {
    if (state.autoSaveTimer) {
      clearTimeout(state.autoSaveTimer);
    }
    state.autoSaveTimer = setTimeout(saveDocument, CONFIG.AUTOSAVE_DELAY);
    if (elements.saveStatus) {
      elements.saveStatus.textContent = "Saving...";
    }
  }

  async function saveDocument() {
    if (!state.isDirty) return;

    const content = elements.editorContent?.innerHTML || "";
    const title = state.docTitle;

    try {
      const response = await fetch("/api/docs/save", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          id: state.docId,
          title,
          content,
          driveSource: state.driveSource,
        }),
      });

      if (response.ok) {
        const result = await response.json();
        if (result.id) {
          state.docId = result.id;
          window.history.replaceState({}, "", `#id=${state.docId}`);
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

  async function loadFromUrlParams() {
    const urlParams = new URLSearchParams(window.location.search);
    const hash = window.location.hash;
    let docId = urlParams.get("id");
    let bucket = urlParams.get("bucket");
    let path = urlParams.get("path");

    if (hash) {
      const hashQueryIndex = hash.indexOf("?");
      if (hashQueryIndex > -1) {
        const hashParams = new URLSearchParams(hash.slice(hashQueryIndex + 1));
        docId = docId || hashParams.get("id");
        bucket = bucket || hashParams.get("bucket");
        path = path || hashParams.get("path");
      } else if (hash.startsWith("#id=")) {
        docId = hash.slice(4);
      }
    }

    if (bucket && path) {
      state.driveSource = { bucket, path };
      await loadFromDrive(bucket, path);
    } else if (docId) {
      try {
        const response = await fetch(`/api/docs/${docId}`);
        if (response.ok) {
          const data = await response.json();
          state.docId = docId;
          state.docTitle = data.title || "Untitled Document";
          if (elements.docName) elements.docName.value = state.docTitle;
          if (elements.editorContent)
            elements.editorContent.innerHTML = data.content || "";
          saveToHistory();
          updateWordCount();
        }
      } catch (e) {
        console.error("Load failed:", e);
      }
    } else {
      saveToHistory();
    }
  }

  async function loadFromDrive(bucket, path) {
    const fileName = path.split("/").pop() || "Document";
    const ext = fileName.split(".").pop()?.toLowerCase();

    try {
      const response = await fetch("/api/drive/content", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ bucket, path }),
      });

      if (response.ok) {
        const data = await response.json();
        const content = data.content || "";

        state.docTitle = fileName.replace(/\.[^.]+$/, "");
        if (elements.docName) elements.docName.value = state.docTitle;

        if (ext === "md") {
          if (elements.editorContent) {
            elements.editorContent.innerHTML = markdownToHtml(content);
          }
        } else if (ext === "txt") {
          if (elements.editorContent) {
            elements.editorContent.innerHTML = `<p>${escapeHtml(content).replace(/\n/g, "</p><p>")}</p>`;
          }
        } else {
          if (elements.editorContent) {
            elements.editorContent.innerHTML = content;
          }
        }

        saveToHistory();
        updateWordCount();
      }
    } catch (e) {
      console.error("Drive load failed:", e);
    }
  }

  function markdownToHtml(md) {
    return md
      .replace(/^### (.+)$/gm, "<h3>$1</h3>")
      .replace(/^## (.+)$/gm, "<h2>$1</h2>")
      .replace(/^# (.+)$/gm, "<h1>$1</h1>")
      .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
      .replace(/\*(.+?)\*/g, "<em>$1</em>")
      .replace(/`(.+?)`/g, "<code>$1</code>")
      .replace(/\n/g, "<br>");
  }

  function showModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) modal.classList.remove("hidden");
  }

  function hideModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) modal.classList.add("hidden");
  }

  function closeModals() {
    document
      .querySelectorAll(".modal")
      .forEach((m) => m.classList.add("hidden"));
  }

  function insertLink() {
    const url = document.getElementById("linkUrl")?.value;
    const text = document.getElementById("linkText")?.value || url;
    if (url) {
      elements.editorContent?.focus();
      document.execCommand(
        "insertHTML",
        false,
        `<a href="${escapeHtml(url)}" target="_blank">${escapeHtml(text)}</a>`,
      );
      hideModal("linkModal");
      saveToHistory();
      state.isDirty = true;
    }
  }

  function insertImage() {
    const url = document.getElementById("imageUrl")?.value;
    const alt = document.getElementById("imageAlt")?.value || "Image";
    if (url) {
      elements.editorContent?.focus();
      document.execCommand(
        "insertHTML",
        false,
        `<img src="${escapeHtml(url)}" alt="${escapeHtml(alt)}" style="max-width:100%">`,
      );
      hideModal("imageModal");
      saveToHistory();
      state.isDirty = true;
    }
  }

  function insertTable() {
    const rows = parseInt(document.getElementById("tableRows")?.value, 10) || 3;
    const cols = parseInt(document.getElementById("tableCols")?.value, 10) || 3;

    let html = '<table style="border-collapse:collapse;width:100%">';
    for (let r = 0; r < rows; r++) {
      html += "<tr>";
      for (let c = 0; c < cols; c++) {
        const cell = r === 0 ? "th" : "td";
        html += `<${cell} style="border:1px solid var(--sentient-border,#e0e0e0);padding:8px">${r === 0 ? "Header" : ""}</${cell}>`;
      }
      html += "</tr>";
    }
    html += "</table><p></p>";

    elements.editorContent?.focus();
    document.execCommand("insertHTML", false, html);
    hideModal("tableModal");
    saveToHistory();
    state.isDirty = true;
  }

  function copyShareLink() {
    const linkInput = document.getElementById("shareLink");
    if (linkInput) {
      const shareUrl = `${window.location.origin}${window.location.pathname}#id=${state.docId || "new"}`;
      linkInput.value = shareUrl;
      linkInput.select();
      navigator.clipboard.writeText(shareUrl);
    }
  }

  function exportDocument(format) {
    const title = state.docTitle || "document";
    const content = elements.editorContent?.innerHTML || "";

    switch (format) {
      case "pdf":
        exportAsPDF(title, content);
        break;
      case "docx":
        exportAsDocx(title, content);
        break;
      case "html":
        exportAsHTML(title, content);
        break;
      case "txt":
        exportAsTxt(title);
        break;
      case "md":
        exportAsMarkdown(title);
        break;
    }
    hideModal("exportModal");
  }

  function exportAsPDF(title, content) {
    const printWindow = window.open("", "_blank");
    if (printWindow) {
      printWindow.document.write(`
        <!DOCTYPE html>
        <html>
        <head>
          <title>${escapeHtml(title)}</title>
          <style>
            body { font-family: Arial, sans-serif; padding: 40px; max-width: 800px; margin: 0 auto; }
            h1, h2, h3 { margin-top: 1em; }
            p { line-height: 1.6; }
            table { border-collapse: collapse; width: 100%; }
            th, td { border: 1px solid #ccc; padding: 8px; }
          </style>
        </head>
        <body>${content}</body>
        </html>
      `);
      printWindow.document.close();
      printWindow.print();
    }
  }

  function exportAsDocx(title, content) {
    addChatMessage(
      "assistant",
      "DOCX export requires server-side processing. Feature coming soon!",
    );
  }

  function exportAsHTML(title, content) {
    const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>${escapeHtml(title)}</title>
  <style>
    body { font-family: Arial, sans-serif; padding: 40px; max-width: 800px; margin: 0 auto; }
    h1, h2, h3 { margin-top: 1em; }
    p { line-height: 1.6; }
    table { border-collapse: collapse; width: 100%; }
    th, td { border: 1px solid #ccc; padding: 8px; }
  </style>
</head>
<body>
${content}
</body>
</html>`;
    downloadFile(html, `${title}.html`, "text/html");
  }

  function exportAsTxt(title) {
    const text = elements.editorContent?.innerText || "";
    downloadFile(text, `${title}.txt`, "text/plain");
  }

  function exportAsMarkdown(title) {
    const text = elements.editorContent?.innerText || "";
    const md = `# ${title}\n\n${text}`;
    downloadFile(md, `${title}.md`, "text/markdown");
  }

  function downloadFile(content, filename, mimeType) {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  function connectWebSocket() {
    if (!state.docId) return;

    try {
      const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
      const wsUrl = `${protocol}//${window.location.host}/api/docs/ws/${state.docId}`;
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
      case "content_update":
        if (msg.userId !== getUserId() && elements.editorContent) {
          const selection = window.getSelection();
          const range =
            selection?.rangeCount > 0 ? selection.getRangeAt(0) : null;
          elements.editorContent.innerHTML = msg.content;
          if (range) {
            try {
              selection?.removeAllRanges();
              selection?.addRange(range);
            } catch (e) {
              // Ignore selection restoration errors
            }
          }
        }
        break;
    }
  }

  function broadcastChange() {
    if (state.ws && state.ws.readyState === WebSocket.OPEN) {
      state.ws.send(
        JSON.stringify({
          type: "content_update",
          userId: getUserId(),
          content: elements.editorContent?.innerHTML || "",
        }),
      );
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
      shorter: "Make the selected text shorter",
      grammar: "Fix grammar and spelling in the document",
      formal: "Make the text more formal",
      summarize: "Summarize this document",
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
    const selectedText = window.getSelection()?.toString() || "";
    let response = "";

    if (lower.includes("shorter") || lower.includes("concise")) {
      if (selectedText) {
        response = await callAI("shorten", selectedText);
      } else {
        response =
          "Please select some text first, then ask me to make it shorter.";
      }
    } else if (
      lower.includes("grammar") ||
      lower.includes("spelling") ||
      lower.includes("fix")
    ) {
      const text = selectedText || elements.editorContent?.innerText || "";
      response = await callAI("grammar", text);
    } else if (lower.includes("formal")) {
      if (selectedText) {
        response = await callAI("formal", selectedText);
      } else {
        response =
          "Please select some text first, then ask me to make it formal.";
      }
    } else if (lower.includes("casual") || lower.includes("informal")) {
      if (selectedText) {
        response = await callAI("casual", selectedText);
      } else {
        response =
          "Please select some text first, then ask me to make it casual.";
      }
    } else if (lower.includes("summarize") || lower.includes("summary")) {
      const text = selectedText || elements.editorContent?.innerText || "";
      response = await callAI("summarize", text);
    } else if (lower.includes("translate")) {
      const langMatch = lower.match(/to (\w+)/);
      const lang = langMatch ? langMatch[1] : "Spanish";
      const text = selectedText || elements.editorContent?.innerText || "";
      response = await callAI("translate", text, lang);
    } else if (lower.includes("expand") || lower.includes("longer")) {
      if (selectedText) {
        response = await callAI("expand", selectedText);
      } else {
        response = "Please select some text first, then ask me to expand it.";
      }
    } else if (lower.includes("heading") || lower.includes("title")) {
      execCommand("formatBlock", "h1");
      response = "Applied heading format to selected text.";
    } else if (lower.includes("bullet") || lower.includes("list")) {
      execCommand("insertUnorderedList");
      response = "Created a bullet list.";
    } else if (lower.includes("number") && lower.includes("list")) {
      execCommand("insertOrderedList");
      response = "Created a numbered list.";
    } else if (lower.includes("bold")) {
      execCommand("bold");
      response = "Applied bold formatting.";
    } else if (lower.includes("italic")) {
      execCommand("italic");
      response = "Applied italic formatting.";
    } else if (lower.includes("underline")) {
      execCommand("underline");
      response = "Applied underline formatting.";
    } else {
      try {
        const res = await fetch("/api/docs/ai", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            command,
            selectedText,
            docId: state.docId,
          }),
        });
        const data = await res.json();
        response = data.response || "I processed your request.";
      } catch {
        response =
          "I can help you with:\n• Make text shorter or longer\n• Fix grammar and spelling\n• Translate to another language\n• Change tone (formal/casual)\n• Summarize the document\n• Format as heading, list, etc.";
      }
    }

    addChatMessage("assistant", response);
  }

  async function callAI(action, text, extra = "") {
    try {
      const res = await fetch("/api/docs/ai", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ action, text, extra, docId: state.docId }),
      });
      if (res.ok) {
        const data = await res.json();
        return data.result || data.response || "Done!";
      }
      return "AI processing failed. Please try again.";
    } catch {
      return "Unable to connect to AI service. Please try again later.";
    }
  }

  function escapeHtml(str) {
    if (!str) return "";
    const div = document.createElement("div");
    div.textContent = str;
    return div.innerHTML;
  }

  function showFindReplaceModal() {
    showModal("findReplaceModal");
    document.getElementById("findInput")?.focus();
    state.findMatches = [];
    state.findMatchIndex = -1;
    clearFindHighlights();
  }

  function performFind() {
    const searchText = document.getElementById("findInput")?.value || "";
    const matchCase = document.getElementById("findMatchCase")?.checked;
    const wholeWord = document.getElementById("findWholeWord")?.checked;

    clearFindHighlights();
    state.findMatches = [];
    state.findMatchIndex = -1;

    if (!searchText || !elements.editorContent) {
      updateFindResults();
      return;
    }

    const content = elements.editorContent.innerHTML;
    let flags = "g";
    if (!matchCase) flags += "i";

    let searchPattern = searchText.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    if (wholeWord) {
      searchPattern = `\\b${searchPattern}\\b`;
    }

    const regex = new RegExp(searchPattern, flags);
    const textContent = elements.editorContent.textContent;
    let match;

    while ((match = regex.exec(textContent)) !== null) {
      state.findMatches.push({
        index: match.index,
        length: match[0].length,
        text: match[0],
      });
    }

    if (state.findMatches.length > 0) {
      state.findMatchIndex = 0;
      highlightAllMatches(searchText, matchCase, wholeWord);
      scrollToMatch();
    }

    updateFindResults();
  }

  function highlightAllMatches(searchText, matchCase, wholeWord) {
    if (!elements.editorContent) return;

    const walker = document.createTreeWalker(
      elements.editorContent,
      NodeFilter.SHOW_TEXT,
      null,
      false,
    );

    const textNodes = [];
    let node;
    while ((node = walker.nextNode())) {
      textNodes.push(node);
    }

    let flags = "g";
    if (!matchCase) flags += "i";
    let searchPattern = searchText.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    if (wholeWord) {
      searchPattern = `\\b${searchPattern}\\b`;
    }
    const regex = new RegExp(`(${searchPattern})`, flags);

    textNodes.forEach((textNode) => {
      const text = textNode.textContent;
      if (regex.test(text)) {
        const span = document.createElement("span");
        span.innerHTML = text.replace(
          regex,
          '<mark class="find-highlight">$1</mark>',
        );
        textNode.parentNode.replaceChild(span, textNode);
      }
    });

    updateCurrentHighlight();
  }

  function updateCurrentHighlight() {
    const highlights =
      elements.editorContent?.querySelectorAll(".find-highlight");
    if (!highlights) return;

    highlights.forEach((el, index) => {
      el.classList.toggle("current", index === state.findMatchIndex);
    });
  }

  function clearFindHighlights() {
    if (!elements.editorContent) return;

    const highlights =
      elements.editorContent.querySelectorAll(".find-highlight");
    highlights.forEach((el) => {
      const parent = el.parentNode;
      parent.replaceChild(document.createTextNode(el.textContent), el);
      parent.normalize();
    });

    const wrapperSpans = elements.editorContent.querySelectorAll("span:empty");
    wrapperSpans.forEach((span) => {
      if (span.childNodes.length === 0) {
        span.remove();
      }
    });
  }

  function updateFindResults() {
    const resultsEl = document.getElementById("findResults");
    if (resultsEl) {
      const count = state.findMatches.length;
      const span = resultsEl.querySelector("span");
      if (span) {
        span.textContent =
          count === 0
            ? "0 matches found"
            : `${state.findMatchIndex + 1} of ${count} matches`;
      }
    }
  }

  function scrollToMatch() {
    const highlights =
      elements.editorContent?.querySelectorAll(".find-highlight");
    if (highlights && highlights[state.findMatchIndex]) {
      highlights[state.findMatchIndex].scrollIntoView({
        behavior: "smooth",
        block: "center",
      });
    }
  }

  function findNext() {
    if (state.findMatches.length === 0) return;
    state.findMatchIndex =
      (state.findMatchIndex + 1) % state.findMatches.length;
    updateCurrentHighlight();
    scrollToMatch();
    updateFindResults();
  }

  function findPrev() {
    if (state.findMatches.length === 0) return;
    state.findMatchIndex =
      (state.findMatchIndex - 1 + state.findMatches.length) %
      state.findMatches.length;
    updateCurrentHighlight();
    scrollToMatch();
    updateFindResults();
  }

  function replaceOne() {
    if (state.findMatches.length === 0 || state.findMatchIndex < 0) return;

    const replaceText = document.getElementById("replaceInput")?.value || "";
    const highlights =
      elements.editorContent?.querySelectorAll(".find-highlight");

    if (highlights && highlights[state.findMatchIndex]) {
      const highlight = highlights[state.findMatchIndex];
      highlight.replaceWith(document.createTextNode(replaceText));
      elements.editorContent.normalize();

      state.findMatches.splice(state.findMatchIndex, 1);
      if (state.findMatches.length > 0) {
        state.findMatchIndex = state.findMatchIndex % state.findMatches.length;
        updateCurrentHighlight();
        scrollToMatch();
      } else {
        state.findMatchIndex = -1;
      }
      updateFindResults();

      state.isDirty = true;
      scheduleAutoSave();
    }
  }

  function replaceAll() {
    if (state.findMatches.length === 0) return;

    const replaceText = document.getElementById("replaceInput")?.value || "";
    const highlights =
      elements.editorContent?.querySelectorAll(".find-highlight");

    if (highlights) {
      const count = highlights.length;
      highlights.forEach((highlight) => {
        highlight.replaceWith(document.createTextNode(replaceText));
      });
      elements.editorContent.normalize();

      state.findMatches = [];
      state.findMatchIndex = -1;
      updateFindResults();

      state.isDirty = true;
      scheduleAutoSave();
      addChatMessage("assistant", `Replaced ${count} occurrences.`);
    }
  }

  function showPrintPreview() {
    showModal("printPreviewModal");
    updatePrintPreview();
  }

  function updatePrintPreview() {
    const orientation =
      document.getElementById("printOrientation")?.value || "portrait";
    const showHeaders = document.getElementById("printHeaders")?.checked;
    const printPage = document.getElementById("printPage");
    const printContent = document.getElementById("printContent");
    const printHeader = document.getElementById("printHeader");
    const printFooter = document.getElementById("printFooter");

    if (printPage) {
      printPage.className = `print-page ${orientation}`;
    }

    if (printHeader) {
      printHeader.innerHTML = showHeaders ? state.docTitle : "";
      printHeader.style.display = showHeaders ? "block" : "none";
    }

    if (printFooter) {
      printFooter.innerHTML = showHeaders ? "Page 1" : "";
      printFooter.style.display = showHeaders ? "block" : "none";
    }

    if (printContent && elements.editorContent) {
      printContent.innerHTML = elements.editorContent.innerHTML;
    }
  }

  function printDocument() {
    const orientation =
      document.getElementById("printOrientation")?.value || "portrait";
    const showHeaders = document.getElementById("printHeaders")?.checked;
    const content = elements.editorContent?.innerHTML || "";

    const printWindow = window.open("", "_blank");

    printWindow.document.write(`
      <!DOCTYPE html>
      <html>
      <head>
        <title>${state.docTitle}</title>
        <style>
          @page { size: ${orientation}; margin: 1in; }
          body {
            font-family: Arial, sans-serif;
            font-size: 12pt;
            line-height: 1.6;
            color: #000;
          }
          h1 { font-size: 24pt; margin-bottom: 12pt; }
          h2 { font-size: 18pt; margin-bottom: 10pt; }
          h3 { font-size: 14pt; margin-bottom: 8pt; }
          p { margin-bottom: 12pt; }
          table { border-collapse: collapse; width: 100%; margin: 12pt 0; }
          td, th { border: 1px solid #ccc; padding: 8px; }
          .page-break { page-break-after: always; }
          ${showHeaders ? `.header { text-align: center; font-size: 10pt; color: #666; margin-bottom: 24pt; }` : ""}
        </style>
      </head>
      <body>
        ${showHeaders ? `<div class="header">${state.docTitle}</div>` : ""}
        ${content}
      </body>
      </html>
    `);

    printWindow.document.close();
    printWindow.focus();
    setTimeout(() => {
      printWindow.print();
      printWindow.close();
    }, 250);

    hideModal("printPreviewModal");
  }

  function insertPageBreak() {
    if (!elements.editorContent) return;

    const pageBreak = document.createElement("div");
    pageBreak.className = "page-break";
    pageBreak.contentEditable = "false";

    const selection = window.getSelection();
    if (selection.rangeCount > 0) {
      const range = selection.getRangeAt(0);
      range.deleteContents();
      range.insertNode(pageBreak);

      const newParagraph = document.createElement("p");
      newParagraph.innerHTML = "<br>";
      pageBreak.after(newParagraph);

      range.setStartAfter(newParagraph);
      range.collapse(true);
      selection.removeAllRanges();
      selection.addRange(range);
    } else {
      elements.editorContent.appendChild(pageBreak);
    }

    state.isDirty = true;
    scheduleAutoSave();
  }

  function showHeaderFooterModal() {
    showModal("headerFooterModal");

    const headerEditor = document.getElementById("headerEditor");
    const footerEditor = document.getElementById("footerEditor");

    if (headerEditor && elements.editorHeader) {
      headerEditor.innerHTML = elements.editorHeader.innerHTML;
    }
    if (footerEditor && elements.editorFooter) {
      footerEditor.innerHTML = elements.editorFooter.innerHTML;
    }
  }

  function switchHfTab(tabName) {
    document.querySelectorAll(".hf-tab").forEach((tab) => {
      tab.classList.toggle("active", tab.dataset.tab === tabName);
    });
    document
      .getElementById("hfHeaderTab")
      ?.classList.toggle("active", tabName === "header");
    document
      .getElementById("hfFooterTab")
      ?.classList.toggle("active", tabName === "footer");
  }

  function insertHfField(type, field) {
    const editorId = type === "header" ? "headerEditor" : "footerEditor";
    const editor = document.getElementById(editorId);
    if (!editor) return;

    let fieldContent = "";
    switch (field) {
      case "pageNum":
        fieldContent =
          '<span class="hf-field" data-field="pageNum">[Page #]</span>';
        break;
      case "date":
        fieldContent = `<span class="hf-field" data-field="date">${new Date().toLocaleDateString()}</span>`;
        break;
      case "title":
        fieldContent = `<span class="hf-field" data-field="title">${state.docTitle}</span>`;
        break;
    }

    editor.focus();
    document.execCommand("insertHTML", false, fieldContent);
  }

  function applyHeaderFooter() {
    const headerEditor = document.getElementById("headerEditor");
    const footerEditor = document.getElementById("footerEditor");

    if (elements.editorHeader && headerEditor) {
      elements.editorHeader.innerHTML = headerEditor.innerHTML;
    }
    if (elements.editorFooter && footerEditor) {
      elements.editorFooter.innerHTML = footerEditor.innerHTML;
    }

    hideModal("headerFooterModal");
    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Header and footer updated!");
  }

  function removeHeaderFooter() {
    if (elements.editorHeader) {
      elements.editorHeader.innerHTML = "";
    }
    if (elements.editorFooter) {
      elements.editorFooter.innerHTML = "";
    }

    const headerEditor = document.getElementById("headerEditor");
    const footerEditor = document.getElementById("footerEditor");
    if (headerEditor) headerEditor.innerHTML = "";
    if (footerEditor) footerEditor.innerHTML = "";

    hideModal("headerFooterModal");
    state.isDirty = true;
    scheduleAutoSave();
    addChatMessage("assistant", "Header and footer removed.");
  }

  function handleHeaderFooterInput() {
    state.isDirty = true;
    scheduleAutoSave();
  }

  function createNewDocument() {
    state.docId = null;
    state.docTitle = "Untitled Document";
    state.isDirty = false;
    state.history = [];
    state.historyIndex = -1;

    if (elements.docName) elements.docName.value = state.docTitle;
    if (elements.editorContent) elements.editorContent.innerHTML = "";

    window.history.replaceState({}, "", window.location.pathname);
    saveToHistory();
    updateWordCount();
    elements.editorContent?.focus();
  }

  window.gbDocs = {
    init,
    createNewDocument,
    saveDocument,
    exportDocument,
    showModal,
    hideModal,
    closeModals,
    toggleChatPanel,
    execCommand,
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
