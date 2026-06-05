// docs/modules/02_undo.js
"use strict";

// Functions: undo, redo, updateWordCount, zoomIn, zoomOut, applyZoom, scheduleAutoSave, saveDocument, loadFromUrlParams, loadFromDrive, markdownToHtml, showModal, hideModal, closeModals, insertLink, insertImage, insertTable, copyShareLink, exportDocument, exportAsPDF, exportAsDocx, exportAsHTML, exportAsTxt, exportAsMarkdown, downloadFile, connectWebSocket

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

  async function exportAsDocx(title, content) {
    try {
      addChatMessage("assistant", "Generating DOCX...");
      const response = await fetch("/api/docs/export/docx", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ title, content, documentId: state.documentId }),
      });
      if (response.ok) {
        const blob = await response.blob();
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = (title || "document") + ".docx";
        a.click();
        URL.revokeObjectURL(url);
        addChatMessage("assistant", "DOCX exported successfully!");
      } else {
        addChatMessage("assistant", "DOCX export failed. Try again later.");
      }
    } catch (e) {
      addChatMessage("assistant", "DOCX export failed: " + e.message);
    }
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

