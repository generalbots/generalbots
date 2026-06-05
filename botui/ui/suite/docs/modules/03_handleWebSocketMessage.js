// docs/modules/03_handleWebSocketMessage.js
"use strict";

// Functions: handleWebSocketMessage, broadcastChange, addCollaborator, removeCollaborator, renderCollaborators, getUserId, getUserName, toggleChatPanel, handleChatSubmit, handleSuggestionClick, addChatMessage, processAICommand, callAI, escapeHtml, showFindReplaceModal, performFind, highlightAllMatches, updateCurrentHighlight, clearFindHighlights, updateFindResults, scrollToMatch, findNext, findPrev, replaceOne, replaceAll

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

