// docs/modules/01_init.js
"use strict";

// Functions: init, cacheElements, bindEvents, handleEditorInput, handleDocNameChange, handleEditorKeydown, handlePaste, handleBeforeUnload, setupToolbar, updateToolbarState, setupKeyboardShortcuts, execCommand, handleHeadingChange, handleFontFamilyChange, handleFontSizeChange, handleTextColorChange, handleHighlightChange, saveToHistory


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

