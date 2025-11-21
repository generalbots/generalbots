window.driveApp = function driveApp() {
  return {
    currentView: "all",
    viewMode: "tree",
    sortBy: "name",
    searchQuery: "",
    selectedItem: null,
    currentPath: "/",
    currentBucket: null,
    showUploadDialog: false,

    showEditor: false,
    editorContent: "",
    editorFilePath: "",
    editorFileName: "",
    editorLoading: false,
    editorSaving: false,

    quickAccess: [
      { id: "all", label: "All Files", icon: "📁", count: null },
      { id: "recent", label: "Recent", icon: "🕐", count: null },
      { id: "starred", label: "Starred", icon: "⭐", count: 3 },
      { id: "shared", label: "Shared", icon: "👥", count: 5 },
      { id: "trash", label: "Trash", icon: "🗑️", count: 0 },
    ],

    storageUsed: "12.3 GB",
    storageTotal: "50 GB",
    storagePercent: 25,

    fileTree: [],
    loading: false,
    error: null,

    get allItems() {
      const flatten = (items) => {
        let result = [];
        items.forEach((item) => {
          result.push(item);
          if (item.children && item.expanded) {
            result = result.concat(flatten(item.children));
          }
        });
        return result;
      };
      return flatten(this.fileTree);
    },

    get filteredItems() {
      let items = this.allItems;

      if (this.searchQuery.trim()) {
        const query = this.searchQuery.toLowerCase();
        items = items.filter((item) => item.name.toLowerCase().includes(query));
      }

      items = [...items].sort((a, b) => {
        if (a.type === "folder" && b.type !== "folder") return -1;
        if (a.type !== "folder" && b.type === "folder") return 1;

        switch (this.sortBy) {
          case "name":
            return a.name.localeCompare(b.name);
          case "modified":
            return new Date(b.modified) - new Date(a.modified);
          case "size":
            return (
              this.sizeToBytes(b.size || "0") - this.sizeToBytes(a.size || "0")
            );
          case "type":
            return (a.type || "").localeCompare(b.type || "");
          default:
            return 0;
        }
      });

      return items;
    },

    get breadcrumbs() {
      const crumbs = [{ name: "Home", path: "/" }];

      if (this.currentBucket) {
        crumbs.push({
          name: this.currentBucket,
          path: `/${this.currentBucket}`,
        });

        if (this.currentPath && this.currentPath !== "/") {
          const parts = this.currentPath.split("/").filter(Boolean);
          let currentPath = `/${this.currentBucket}`;
          parts.forEach((part) => {
            currentPath += `/${part}`;
            crumbs.push({ name: part, path: currentPath });
          });
        }
      }

      return crumbs;
    },

    async loadFiles(bucket = null, path = null) {
      this.loading = true;
      this.error = null;

      try {
        const params = new URLSearchParams();
        if (bucket) params.append("bucket", bucket);
        if (path) params.append("path", path);

        const response = await fetch(`/files/list?${params.toString()}`);

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const files = await response.json();
        this.fileTree = this.convertToTree(files, bucket, path);
        this.currentBucket = bucket;
        this.currentPath = path || "/";
      } catch (err) {
        console.error("Error loading files:", err);
        this.error = err.toString();
        this.fileTree = this.getMockData();
      } finally {
        this.loading = false;
      }
    },

    convertToTree(files, bucket, basePath) {
      return files.map((file) => {
        const depth = basePath ? basePath.split("/").filter(Boolean).length : 0;

        return {
          id: file.path,
          name: file.name,
          type: file.is_dir ? "folder" : this.getFileTypeFromName(file.name),
          path: file.path,
          bucket: bucket,
          depth: depth,
          expanded: false,
          modified: new Date().toISOString().split("T")[0],
          created: new Date().toISOString().split("T")[0],
          size: file.is_dir ? null : "0 KB",
          children: file.is_dir ? [] : undefined,
          isDir: file.is_dir,
          icon: file.icon,
        };
      });
    },

    getFileTypeFromName(filename) {
      const ext = filename.split(".").pop().toLowerCase();
      const typeMap = {
        pdf: "pdf",
        doc: "document",
        docx: "document",
        txt: "text",
        md: "text",
        bas: "code",
        ast: "code",
        xls: "spreadsheet",
        xlsx: "spreadsheet",
        csv: "spreadsheet",
        ppt: "presentation",
        pptx: "presentation",
        jpg: "image",
        jpeg: "image",
        png: "image",
        gif: "image",
        svg: "image",
        mp4: "video",
        avi: "video",
        mov: "video",
        mp3: "audio",
        wav: "audio",
        zip: "archive",
        rar: "archive",
        tar: "archive",
        gz: "archive",
        js: "code",
        ts: "code",
        py: "code",
        java: "code",
        cpp: "code",
        rs: "code",
        go: "code",
        html: "code",
        css: "code",
        json: "code",
        xml: "code",
        gbkb: "knowledge",
        exe: "executable",
      };
      return typeMap[ext] || "file";
    },

    getMockData() {
      return [
        {
          id: 1,
          name: "Documents",
          type: "folder",
          path: "/Documents",
          depth: 0,
          expanded: true,
          modified: "2024-01-15",
          created: "2024-01-01",
          isDir: true,
          icon: "📁",
          children: [
            {
              id: 2,
              name: "notes.txt",
              type: "text",
              path: "/Documents/notes.txt",
              depth: 1,
              size: "4 KB",
              modified: "2024-01-14",
              created: "2024-01-13",
              icon: "📃",
            },
          ],
        },
      ];
    },

    getFileIcon(item) {
      if (item.icon) return item.icon;

      const iconMap = {
        folder: "📁",
        pdf: "📄",
        document: "📝",
        text: "📃",
        spreadsheet: "📊",
        presentation: "📽️",
        image: "🖼️",
        video: "🎬",
        audio: "🎵",
        archive: "📦",
        code: "💻",
        knowledge: "📚",
        executable: "⚙️",
      };
      return iconMap[item.type] || "📄";
    },

    async toggleFolder(item) {
      if (item.type === "folder") {
        item.expanded = !item.expanded;

        if (item.expanded && item.children.length === 0) {
          try {
            const params = new URLSearchParams();
            params.append("bucket", item.bucket || item.name);
            if (item.path !== item.name) {
              params.append("path", item.path);
            }

            const response = await fetch(`/files/list?${params.toString()}`);
            if (response.ok) {
              const files = await response.json();
              item.children = this.convertToTree(
                files,
                item.bucket || item.name,
                item.path,
              );
            }
          } catch (err) {
            console.error("Error loading folder contents:", err);
          }
        }
      }
    },

    openFolder(item) {
      if (item.type === "folder") {
        this.loadFiles(item.bucket || item.name, item.path);
      }
    },

    selectItem(item) {
      this.selectedItem = item;
    },

    navigateToPath(path) {
      if (path === "/") {
        this.loadFiles(null, null);
      } else {
        const parts = path.split("/").filter(Boolean);
        const bucket = parts[0];
        const filePath = parts.slice(1).join("/");
        this.loadFiles(bucket, filePath || "/");
      }
    },

    isEditableFile(item) {
      if (item.type === "folder") return false;
      const editableTypes = ["text", "code"];
      const editableExtensions = [
        "txt",
        "md",
        "js",
        "ts",
        "json",
        "html",
        "css",
        "xml",
        "csv",
        "log",
        "yml",
        "yaml",
        "ini",
        "conf",
        "sh",
        "bat",
        "bas",
        "ast",
        "gbkb",
      ];

      if (editableTypes.includes(item.type)) return true;

      const ext = item.name.split(".").pop().toLowerCase();
      return editableExtensions.includes(ext);
    },

    async editFile(item) {
      if (!this.isEditableFile(item)) {
        alert(`Cannot edit ${item.type} files. Only text files can be edited.`);
        return;
      }

      this.editorLoading = true;
      this.showEditor = true;
      this.editorFileName = item.name;
      this.editorFilePath = item.path;

      try {
        const response = await fetch("/files/read", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            bucket: item.bucket || this.currentBucket,
            path: item.path,
          }),
        });

        if (!response.ok) {
          const error = await response.json();
          throw new Error(error.error || "Failed to read file");
        }

        const data = await response.json();
        this.editorContent = data.content;
      } catch (err) {
        console.error("Error reading file:", err);
        alert(`Error opening file: ${err.message}`);
        this.showEditor = false;
      } finally {
        this.editorLoading = false;
      }
    },

    async saveFile() {
      if (!this.editorFilePath) return;

      this.editorSaving = true;

      try {
        const response = await fetch("/files/write", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            bucket: this.currentBucket,
            path: this.editorFilePath,
            content: this.editorContent,
          }),
        });

        if (!response.ok) {
          const error = await response.json();
          throw new Error(error.error || "Failed to save file");
        }

        alert("File saved successfully!");
      } catch (err) {
        console.error("Error saving file:", err);
        alert(`Error saving file: ${err.message}`);
      } finally {
        this.editorSaving = false;
      }
    },

    closeEditor() {
      if (
        this.editorContent &&
        confirm("Close editor? Unsaved changes will be lost.")
      ) {
        this.showEditor = false;
        this.editorContent = "";
        this.editorFilePath = "";
        this.editorFileName = "";
      } else if (!this.editorContent) {
        this.showEditor = false;
      }
    },

    async downloadItem(item) {
      window.open(
        `/files/download?bucket=${item.bucket}&path=${item.path}`,
        "_blank",
      );
    },

    shareItem(item) {
      const shareUrl = `${window.location.origin}/files/share?bucket=${item.bucket}&path=${item.path}`;
      prompt("Share link:", shareUrl);
    },

    async deleteItem(item) {
      if (!confirm(`Are you sure you want to delete "${item.name}"?`)) return;

      try {
        const response = await fetch("/files/delete", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            bucket: item.bucket || this.currentBucket,
            path: item.path,
          }),
        });

        if (!response.ok) {
          const error = await response.json();
          throw new Error(error.error || "Failed to delete");
        }

        alert("Deleted successfully!");
        this.loadFiles(this.currentBucket, this.currentPath);
        this.selectedItem = null;
      } catch (err) {
        console.error("Error deleting:", err);
        alert(`Error: ${err.message}`);
      }
    },

    async createFolder() {
      const name = prompt("Enter folder name:");
      if (!name) return;

      try {
        const response = await fetch("/files/create-folder", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            bucket: this.currentBucket,
            path: this.currentPath === "/" ? "" : this.currentPath,
            name: name,
          }),
        });

        if (!response.ok) {
          const error = await response.json();
          throw new Error(error.error || "Failed to create folder");
        }

        alert("Folder created!");
        this.loadFiles(this.currentBucket, this.currentPath);
      } catch (err) {
        console.error("Error creating folder:", err);
        alert(`Error: ${err.message}`);
      }
    },

    sizeToBytes(sizeStr) {
      if (!sizeStr || sizeStr === "—") return 0;

      const units = {
        B: 1,
        KB: 1024,
        MB: 1024 * 1024,
        GB: 1024 * 1024 * 1024,
        TB: 1024 * 1024 * 1024 * 1024,
      };

      const match = sizeStr.match(/^([\d.]+)\s*([A-Z]+)$/i);
      if (!match) return 0;

      const value = parseFloat(match[1]);
      const unit = match[2].toUpperCase();

      return value * (units[unit] || 1);
    },

    renderChildren(item) {
      return "";
    },

    init() {
      console.log("✓ Drive component initialized");
      this.loadFiles(null, null);

      const section = document.querySelector("#section-drive");
      if (section) {
        section.addEventListener("section-shown", () => {
          console.log("Drive section shown");
          this.loadFiles(this.currentBucket, this.currentPath);
        });

        section.addEventListener("section-hidden", () => {
          console.log("Drive section hidden");
        });
      }
    },
  };
};

console.log("✓ Drive app function registered");
