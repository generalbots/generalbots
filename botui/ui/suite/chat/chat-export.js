"use strict";
/**
 * Chat — conversation export (Markdown download) and print / save-as-PDF.
 *
 * The transcript on screen is the source: exporting reads `#messages` so what
 * is saved is exactly what the person read, including replayed history.
 */

(function () {
  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function plusButton(className, label, svgPath) {
    var button = document.createElement("button");
    button.type = "button";
    button.className = className;
    button.title = label;
    button.setAttribute("aria-label", label);
    button.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" ' +
      'stroke="currentColor" stroke-width="2" aria-hidden="true">' + svgPath + "</svg>";
    return button;
  }

  function conversationTitle() {
    var bot = (window.ChatState && (ChatState.currentBotId || ChatState.botName)) ||
      window.__BOT_NAME__ || "";
    var name = bot ? String(bot) : t("chat-export-untitled", "Conversation");
    var stamp = new Date().toISOString().slice(0, 10);
    return name + " — " + stamp;
  }

  function messageText(bubble) {
    // Prefer the stored source: it keeps the original markdown (fences,
    // tables, emphasis) that the rendered DOM has already flattened.
    var raw = bubble.dataset ? bubble.dataset.originalContent : "";
    if (typeof raw === "string" && raw.trim()) {
      return raw.replace(/(?:^|\n)📎\s+[^\n]+$/gm, "").trim();
    }
    var body = bubble.querySelector(".message-content") || bubble;
    var clone = body.cloneNode(true);
    var strip = clone.querySelectorAll(
      ".message-actions, .gb-feedback-wrap, .gb-message-time, .gb-retry-btn, " +
      ".gb-artifact-btn, .gb-attachment-btn, .gb-message-toggle"
    );
    for (var index = 0; index < strip.length; index++) strip[index].remove();
    return (clone.innerText || clone.textContent || "").trim();
  }

  function buildMarkdown() {
    var transcript = document.getElementById("messages");
    if (!transcript) return "";
    var lines = ["# " + conversationTitle(), ""];
    var bubbles = transcript.querySelectorAll(".message");
    for (var index = 0; index < bubbles.length; index++) {
      var bubble = bubbles[index];
      var text = messageText(bubble);
      if (!text) continue;
      var role = bubble.classList.contains("user")
        ? t("chat-export-user", "You")
        : t("chat-export-assistant", "Assistant");
      lines.push("**" + role + "**");
      lines.push("");
      lines.push(text);
      lines.push("");
    }
    if (lines.length <= 2) return "";
    return lines.join("\n");
  }

  function slug(text) {
    return String(text)
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 60) || "conversation";
  }

  function download(markdown, filename) {
    var blob = new Blob([markdown], { type: "text/markdown;charset=utf-8" });
    var url = URL.createObjectURL(blob);
    var link = document.createElement("a");
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    link.remove();
    setTimeout(function () {
      try { URL.revokeObjectURL(url); } catch (error) { /* already gone */ }
    }, 200);
  }

  function announce(key, fallback) {
    var message = t(key, fallback);
    if (typeof window.chatNotice === "function") {
      window.chatNotice(message, "success");
      return;
    }
    if (typeof window.showToast === "function") window.showToast(message);
  }

  function exportConversation() {
    var markdown = buildMarkdown();
    if (!markdown) {
      announce("chat-export-empty", "Nothing to export yet");
      return "";
    }
    download(markdown, slug(conversationTitle()) + ".md");
    announce("chat-export-done", "Conversation exported");
    return markdown;
  }

  function printConversation() {
    var layout = document.getElementById("chat-app");
    if (layout) layout.classList.add("gb-printing");
    try {
      window.print();
    } finally {
      setTimeout(function () {
        if (layout) layout.classList.remove("gb-printing");
      }, 400);
    }
  }

  function mount() {
    var host = document.getElementById("chatHeaderActions");
    if (!host || host.dataset.gbExportBound === "1") return;
    host.dataset.gbExportBound = "1";

    var exportBtn = plusButton(
      "gb-chat-header-btn",
      t("chat-export-md", "Export conversation as Markdown"),
      '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>' +
      '<polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>'
    );
    exportBtn.addEventListener("click", exportConversation);

    var printBtn = plusButton(
      "gb-chat-header-btn",
      t("chat-export-print", "Print or save as PDF"),
      '<polyline points="6 9 6 2 18 2 18 9"/>' +
      '<path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"/>' +
      '<rect x="6" y="14" width="12" height="8"/>'
    );
    printBtn.addEventListener("click", printConversation);

    host.appendChild(exportBtn);
    host.appendChild(printBtn);
  }

  window.GBChatExport = {
    buildMarkdown: buildMarkdown,
    exportConversation: exportConversation,
    printConversation: printConversation
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount);
  } else {
    mount();
  }
  window.addEventListener("gb-chat-ready", mount);
})();
