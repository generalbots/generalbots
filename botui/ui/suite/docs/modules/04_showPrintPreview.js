// docs/modules/04_showPrintPreview.js
"use strict";

// Functions: showPrintPreview, updatePrintPreview, printDocument, insertPageBreak, showHeaderFooterModal, switchHfTab, insertHfField, applyHeaderFooter, removeHeaderFooter, handleHeaderFooterInput, createNewDocument

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
