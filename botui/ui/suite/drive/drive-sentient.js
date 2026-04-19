(function () {
  "use strict";

  let currentView = "grid";
  let selectedFile = null;
  let aiPanelOpen = true;

  function toggleView(type) {
    currentView = type;
    const fileView = document.getElementById("file-view");
    if (fileView) {
      fileView.classList.remove("grid-view", "list-view");
      fileView.classList.add(type + "-view");
    }
    document.querySelectorAll(".app-btn-secondary").forEach((btn) => {
      btn.classList.remove("active");
    });
    event.target.classList.add("active");
  }

  function openFolder(el) {
    const folderName = el.querySelector(".file-name").textContent;
    const breadcrumb = document.querySelector(".breadcrumb");
    if (breadcrumb) {
      const separator = document.createElement("span");
      separator.className = "breadcrumb-separator";
      separator.textContent = "›";
      breadcrumb.appendChild(separator);

      const item = document.createElement("span");
      item.className = "breadcrumb-item current";
      item.textContent = folderName;
      breadcrumb.appendChild(item);

      breadcrumb.querySelectorAll(".breadcrumb-item").forEach((i) => {
        i.classList.remove("current");
      });
      item.classList.add("current");
    }
    addAIMessage("assistant", `Opened folder: ${folderName}`);
  }

  function selectFile(el) {
    document.querySelectorAll(".file-item").forEach((item) => {
      item.classList.remove("selected");
    });
    el.classList.add("selected");
    selectedFile = {
      name: el.querySelector(".file-name").textContent,
      meta: el.querySelector(".file-meta").textContent,
    };
  }

  function toggleAIPanel() {
    const panel = document.getElementById("ai-panel");
    if (panel) {
      aiPanelOpen = !aiPanelOpen;
      panel.classList.toggle("hidden", !aiPanelOpen);
    }
    const toggle = document.querySelector(".ai-toggle");
    if (toggle) {
      toggle.classList.toggle("active", aiPanelOpen);
    }
  }

  function aiAction(action) {
    const actions = {
      organize: "Analyzing folder structure to suggest organization...",
      find: "What file are you looking for?",
      analyze: "Select a file and I'll analyze its content.",
      share: "Select a file to set up sharing options.",
    };
    addAIMessage("assistant", actions[action] || "How can I help?");
  }

  function sendAIMessage() {
    const input = document.getElementById("ai-input");
    if (!input || !input.value.trim()) return;

    const message = input.value.trim();
    addAIMessage("user", message);
    input.value = "";

    setTimeout(() => {
      processAIQuery(message);
    }, 500);
  }

  function addAIMessage(type, content) {
    const container = document.getElementById("ai-messages");
    if (!container) return;

    const div = document.createElement("div");
    div.className = "ai-message " + type;
    div.innerHTML = '<div class="ai-message-bubble">' + escapeHtml(content) + "</div>";
    container.appendChild(div);
    container.scrollTop = container.scrollHeight;
  }

  function processAIQuery(query) {
    const lowerQuery = query.toLowerCase();
    let response = "I can help you manage your files. Try asking me to find, organize, or analyze files.";

    if (lowerQuery.includes("find") || lowerQuery.includes("search") || lowerQuery.includes("buscar")) {
      response = "I'll search for files matching your query. What type of file are you looking for?";
    } else if (lowerQuery.includes("organize") || lowerQuery.includes("organizar")) {
      response = "I can help organize your files by type, date, or project. Which method would you prefer?";
    } else if (lowerQuery.includes("share") || lowerQuery.includes("compartilhar")) {
      if (selectedFile) {
        response = `Setting up sharing for "${selectedFile.name}". Who would you like to share it with?`;
      } else {
        response = "Please select a file first, then I can help you share it.";
      }
    } else if (lowerQuery.includes("delete") || lowerQuery.includes("excluir")) {
      if (selectedFile) {
        response = `Are you sure you want to delete "${selectedFile.name}"? This action cannot be undone.`;
      } else {
        response = "Please select a file first before deleting.";
      }
    } else if (lowerQuery.includes("storage") || lowerQuery.includes("space") || lowerQuery.includes("espaço")) {
      response = "You're using 12.4 GB of your 50 GB storage. Would you like me to find large files to free up space?";
    }

    addAIMessage("assistant", response);
  }

  function uploadFile() {
    const input = document.createElement("input");
    input.type = "file";
    input.multiple = true;
    input.onchange = function (e) {
      const files = Array.from(e.target.files);
      if (files.length > 0) {
        const names = files.map((f) => f.name).join(", ");
        addAIMessage("assistant", `Uploading ${files.length} file(s): ${names}`);
        simulateUpload(files);
      }
    };
    input.click();
  }

  function simulateUpload(files) {
    setTimeout(() => {
      addAIMessage("assistant", `Successfully uploaded ${files.length} file(s)!`);
      files.forEach((file) => {
        addFileToView(file.name, formatFileSize(file.size));
      });
    }, 1500);
  }

  function addFileToView(name, size) {
    const fileView = document.getElementById("file-view");
    if (!fileView) return;

    const icon = getFileIcon(name);
    const div = document.createElement("div");
    div.className = "file-item";
    div.onclick = function () {
      selectFile(this);
    };
    div.innerHTML = `
      <div class="file-icon">${icon}</div>
      <div class="file-name">${escapeHtml(name)}</div>
      <div class="file-meta">${size}</div>
    `;
    fileView.appendChild(div);
  }

  function getFileIcon(filename) {
    const ext = filename.split(".").pop().toLowerCase();
    const icons = {
      pdf: "📄",
      doc: "📝",
      docx: "📝",
      xls: "📊",
      xlsx: "📊",
      ppt: "📽️",
      pptx: "📽️",
      jpg: "🖼️",
      jpeg: "🖼️",
      png: "🖼️",
      gif: "🖼️",
      mp4: "🎬",
      mov: "🎬",
      mp3: "🎵",
      wav: "🎵",
      zip: "📦",
      rar: "📦",
      txt: "📝",
      md: "📝",
      js: "💻",
      ts: "💻",
      rs: "💻",
      py: "💻",
    };
    return icons[ext] || "📄";
  }

  function formatFileSize(bytes) {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + " MB";
    return (bytes / (1024 * 1024 * 1024)).toFixed(1) + " GB";
  }

  function escapeHtml(text) {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }

  function initKeyboardShortcuts() {
    document.addEventListener("keydown", function (e) {
      if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA") return;

      if (e.key === "Delete" && selectedFile) {
        addAIMessage("assistant", `Delete "${selectedFile.name}"? Press Delete again to confirm.`);
      }
      if (e.ctrlKey && e.key === "u") {
        e.preventDefault();
        uploadFile();
      }
      if (e.key === "Escape") {
        document.querySelectorAll(".file-item").forEach((item) => {
          item.classList.remove("selected");
        });
        selectedFile = null;
      }
    });
  }

  function initSearch() {
    const searchInput = document.querySelector(".search-input");
    if (searchInput) {
      searchInput.addEventListener("input", function (e) {
        const query = e.target.value.toLowerCase();
        document.querySelectorAll(".file-item").forEach((item) => {
          const name = item.querySelector(".file-name").textContent.toLowerCase();
          item.style.display = name.includes(query) ? "" : "none";
        });
      });
    }
  }

  function initTabs() {
    document.querySelectorAll(".topbar-tab").forEach((tab) => {
      tab.addEventListener("click", function () {
        document.querySelectorAll(".topbar-tab").forEach((t) => t.classList.remove("active"));
        this.classList.add("active");
        const tabName = this.textContent.trim();
        addAIMessage("assistant", `Switched to ${tabName} view.`);
      });
    });
  }

  function initAppLauncher() {
    document.querySelectorAll(".app-icon").forEach((icon) => {
      icon.addEventListener("click", function () {
        const app = this.dataset.app;
        if (app && app !== "drive") {
          window.location.href = "/suite/" + app + "/";
        }
      });
    });
  }

  function init() {
    initKeyboardShortcuts();
    initSearch();
    initTabs();
    initAppLauncher();

    const aiInput = document.getElementById("ai-input");
    if (aiInput) {
      aiInput.addEventListener("keypress", function (e) {
        if (e.key === "Enter") {
          sendAIMessage();
        }
      });
    }
  }

  window.toggleView = toggleView;
  window.openFolder = openFolder;
  window.selectFile = selectFile;
  window.toggleAIPanel = toggleAIPanel;
  window.aiAction = aiAction;
  window.sendAIMessage = sendAIMessage;
  window.uploadFile = uploadFile;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
