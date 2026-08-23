if (window.GBAppLifecycle) GBAppLifecycle.begin("paper");

    (function () {
        const editor = document.getElementById("editor-content");
        const title = document.getElementById("paper-title");
        const slashMenu = document.getElementById("slash-menu");
        const aiPanel = document.getElementById("ai-panel");
        const sidebar = document.getElementById("paper-sidebar");

        // Slash command handling
        let slashPosition = null;

        editor.addEventListener("input", function (e) {
            updateWordCount();

            const selection = window.getSelection();
            const range = selection.getRangeAt(0);
            const text = range.startContainer.textContent || "";
            const cursorPos = range.startOffset;

            // Check for slash command
            if (text[cursorPos - 1] === "/") {
                showSlashMenu(range);
            } else if (slashMenu && !slashMenu.classList.contains("hidden")) {
                // Filter slash menu based on input after /
                const slashIndex = text.lastIndexOf("/");
                if (slashIndex >= 0 && cursorPos > slashIndex) {
                    const filter = text
                        .substring(slashIndex + 1, cursorPos)
                        .toLowerCase();
                    filterSlashMenu(filter);
                }
            }
        });

        editor.addEventListener("keydown", function (e) {
            // Handle slash menu navigation
            if (!slashMenu.classList.contains("hidden")) {
                if (e.key === "Escape") {
                    hideSlashMenu();
                    e.preventDefault();
                } else if (e.key === "Enter") {
                    const selected =
                        slashMenu.querySelector(".slash-item.selected") ||
                        slashMenu.querySelector(".slash-item");
                    if (selected) {
                        executeSlashCommand(selected.dataset.cmd);
                        e.preventDefault();
                    }
                } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
                    navigateSlashMenu(e.key === "ArrowDown" ? 1 : -1);
                    e.preventDefault();
                }
            }

            // Keyboard shortcuts
            if (e.ctrlKey || e.metaKey) {
                switch (e.key.toLowerCase()) {
                    case "b":
                        e.preventDefault();
                        document.execCommand("bold");
                        break;
                    case "i":
                        e.preventDefault();
                        document.execCommand("italic");
                        break;
                    case "u":
                        e.preventDefault();
                        document.execCommand("underline");
                        break;
                    case "s":
                        e.preventDefault();
                        saveDocument();
                        break;
                }
            }
        });

        function showSlashMenu(range) {
            const rect = range.getBoundingClientRect();
            const editorRect = editor.getBoundingClientRect();

            slashMenu.style.top =
                rect.bottom - editorRect.top + editor.scrollTop + 8 + "px";
            slashMenu.style.left = rect.left - editorRect.left + "px";
            slashMenu.classList.remove("hidden");
            slashPosition = range.startOffset;
        }

        function hideSlashMenu() {
            slashMenu.classList.add("hidden");
            slashPosition = null;
        }

        function filterSlashMenu(filter) {
            const items = slashMenu.querySelectorAll(".slash-item");
            let firstVisible = null;

            items.forEach((item) => {
                const label = item
                    .querySelector(".slash-label")
                    .textContent.toLowerCase();
                const matches = label.includes(filter);
                item.style.display = matches ? "flex" : "none";
                if (matches && !firstVisible) firstVisible = item;
            });

            // Select first visible
            items.forEach((item) => item.classList.remove("selected"));
            if (firstVisible) firstVisible.classList.add("selected");
        }

        function navigateSlashMenu(direction) {
            const items = Array.from(
                slashMenu.querySelectorAll(".slash-item"),
            ).filter((i) => i.style.display !== "none");
            const current = items.findIndex((i) =>
                i.classList.contains("selected"),
            );

            items.forEach((i) => i.classList.remove("selected"));

            let next = current + direction;
            if (next < 0) next = items.length - 1;
            if (next >= items.length) next = 0;

            items[next]?.classList.add("selected");
            items[next]?.scrollIntoView({ block: "nearest" });
        }

        function executeSlashCommand(cmd) {
            hideSlashMenu();

            // Remove the slash character
            const selection = window.getSelection();
            const range = selection.getRangeAt(0);
            const text = range.startContainer.textContent;
            const slashIndex = text.lastIndexOf("/");

            if (slashIndex >= 0) {
                range.startContainer.textContent =
                    text.substring(0, slashIndex) +
                    text.substring(range.startOffset);
                range.setStart(range.startContainer, slashIndex);
                range.collapse(true);
                selection.removeAllRanges();
                selection.addRange(range);
            }

            // Execute command
            switch (cmd) {
                case "h1":
                    document.execCommand("formatBlock", false, "h1");
                    break;
                case "h2":
                    document.execCommand("formatBlock", false, "h2");
                    break;
                case "h3":
                    document.execCommand("formatBlock", false, "h3");
                    break;
                case "bullet":
                    document.execCommand("insertUnorderedList");
                    break;
                case "number":
                    document.execCommand("insertOrderedList");
                    break;
                case "todo":
                    insertTodo();
                    break;
                case "quote":
                    document.execCommand("formatBlock", false, "blockquote");
                    break;
                case "code":
                    document.execCommand("formatBlock", false, "pre");
                    break;
                case "divider":
                    document.execCommand("insertHTML", false, "<hr>");
                    break;
                case "callout":
                    document.execCommand(
                        "insertHTML",
                        false,
                        '<div class="callout">💡 </div>',
                    );
                    break;
                case "table":
                    insertTable();
                    break;
                case "image":
                    insertImage();
                    break;
                case "ai-write":
                case "ai-summarize":
                case "ai-expand":
                case "ai-improve":
                case "ai-translate":
                case "ai-extract":
                    openAIPanel(cmd);
                    break;
            }
        }

        function insertTodo() {
            const html =
                '<div class="todo-item"><input type="checkbox" class="todo-checkbox"><span></span></div>';
            document.execCommand("insertHTML", false, html);
        }

        function insertTable() {
            const html = `
            <table style="width: 100%; border-collapse: collapse;">
                <tr>
                    <td style="border: 1px solid var(--border); padding: 8px;"></td>
                    <td style="border: 1px solid var(--border); padding: 8px;"></td>
                    <td style="border: 1px solid var(--border); padding: 8px;"></td>
                </tr>
                <tr>
                    <td style="border: 1px solid var(--border); padding: 8px;"></td>
                    <td style="border: 1px solid var(--border); padding: 8px;"></td>
                    <td style="border: 1px solid var(--border); padding: 8px;"></td>
                </tr>
            </table>
        `;
            document.execCommand("insertHTML", false, html);
        }

        function insertImage() {
            const url = prompt("Enter image URL:");
            if (url) {
                document.execCommand(
                    "insertHTML",
                    false,
                    `<img src="${url}" style="max-width: 100%;">`,
                );
            }
        }

        function openAIPanel(action) {
            const selectedText = window.getSelection().toString();
            document.getElementById("selected-text-input").value = selectedText;
            aiPanel.classList.remove("hidden");
        }

        function updateWordCount() {
            const text = editor.innerText || "";
            const words = text
                .trim()
                .split(/\s+/)
                .filter((w) => w.length > 0).length;
            const chars = text.length;

            const wordEl = document.getElementById("word-count");
            const charEl = document.getElementById("char-count");

            if (window.i18n && window.i18n.loaded) {
                wordEl.textContent = window.i18n.t("paper-word-count", {
                    count: words,
                });
                charEl.textContent = window.i18n.t("paper-char-count", {
                    count: chars,
                });
            } else {
                wordEl.textContent = words + " words";
                charEl.textContent = chars + " characters";
            }
        }

        function saveDocument() {
            const status = document.getElementById("save-status");
            const savingText =
                window.i18n && window.i18n.loaded
                    ? window.i18n.t("paper-saving")
                    : "Saving...";
            const savedText =
                window.i18n && window.i18n.loaded
                    ? window.i18n.t("paper-saved")
                    : "Saved";

            status.textContent = savingText;
            status.className = "status-item save-status saving";

            htmx.ajax("POST", "/api/ui/paper/save", {
                target: "none",
                values: {
                    title: title.innerText,
                    content: editor.innerHTML,
                },
            }).then(() => {
                status.textContent = savedText;
                status.className = "status-item save-status saved";
            });
        }

        // Toolbar commands
        document.querySelectorAll("[data-cmd]").forEach((btn) => {
            btn.addEventListener("click", function () {
                const cmd = this.dataset.cmd;
                switch (cmd) {
                    case "bold":
                        document.execCommand("bold");
                        break;
                    case "italic":
                        document.execCommand("italic");
                        break;
                    case "underline":
                        document.execCommand("underline");
                        break;
                    case "strikethrough":
                        document.execCommand("strikeThrough");
                        break;
                    case "highlight":
                        document.execCommand("hiliteColor", false, "#ffff00");
                        break;
                    case "alignLeft":
                        document.execCommand("justifyLeft");
                        break;
                    case "alignCenter":
                        document.execCommand("justifyCenter");
                        break;
                    case "alignRight":
                        document.execCommand("justifyRight");
                        break;
                    case "bulletList":
                        document.execCommand("insertUnorderedList");
                        break;
                    case "numberedList":
                        document.execCommand("insertOrderedList");
                        break;
                    case "todoList":
                        insertTodo();
                        break;
                    case "link":
                        const url = prompt("Enter URL:");
                        if (url) document.execCommand("createLink", false, url);
                        break;
                    case "image":
                        insertImage();
                        break;
                    case "table":
                        insertTable();
                        break;
                    case "code":
                        document.execCommand("formatBlock", false, "pre");
                        break;
                    case "quote":
                        document.execCommand(
                            "formatBlock",
                            false,
                            "blockquote",
                        );
                        break;
                    case "undo":
                        document.execCommand("undo");
                        break;
                    case "redo":
                        document.execCommand("redo");
                        break;
                }
                editor.focus();
            });
        });

        // Heading select
        document
            .getElementById("heading-select")
            .addEventListener("change", function () {
                const value = this.value;
                if (value === "p") {
                    document.execCommand("formatBlock", false, "p");
                } else {
                    document.execCommand("formatBlock", false, value);
                }
                editor.focus();
            });

        // Toggle sidebar
        document
            .getElementById("toggle-sidebar")
            .addEventListener("click", function () {
                sidebar.classList.toggle("collapsed");
            });

        // AI Panel
        document
            .getElementById("ai-assist-btn")
            .addEventListener("click", function () {
                const selectedText = window.getSelection().toString();
                document.getElementById("selected-text-input").value =
                    selectedText;
                aiPanel.classList.toggle("hidden");
            });

        document
            .getElementById("close-ai-panel")
            .addEventListener("click", function () {
                aiPanel.classList.add("hidden");
            });

        // Export modal
        document
            .getElementById("export-btn")
            .addEventListener("click", function () {
                document
                    .getElementById("export-modal")
                    .classList.remove("hidden");
            });

        document.querySelectorAll(".close-modal").forEach((btn) => {
            btn.addEventListener("click", function () {
                this.closest(".modal").classList.add("hidden");
            });
        });

        // Click outside modal to close
        document.querySelectorAll(".modal").forEach((modal) => {
            modal.addEventListener("click", function (e) {
                if (e.target === this) {
                    this.classList.add("hidden");
                }
            });
        });

        // Click outside slash menu to close
        document.addEventListener("click", function (e) {
            if (!slashMenu.contains(e.target) && !editor.contains(e.target)) {
                hideSlashMenu();
            }
        });

        // Slash menu item click
        slashMenu.querySelectorAll(".slash-item").forEach((item) => {
            item.addEventListener("click", function () {
                executeSlashCommand(this.dataset.cmd);
            });
        });

        // Tone buttons
        document.querySelectorAll(".tone-btn").forEach((btn) => {
            btn.addEventListener("click", function () {
                document
                    .querySelectorAll(".tone-btn")
                    .forEach((b) => b.classList.remove("active"));
                this.classList.add("active");

                const tone = this.dataset.tone;
                const selectedText = document.getElementById(
                    "selected-text-input",
                ).value;

                htmx.ajax("POST", "/api/ui/paper/ai/tone", {
                    target: "#ai-response-content",
                    values: {
                        tone: tone,
                        text: selectedText,
                    },
                }).then(() => {
                    document
                        .getElementById("ai-response")
                        .classList.remove("hidden");
                });
            });
        });

        // AI response actions
        document
            .getElementById("copy-ai-response")
            ?.addEventListener("click", function () {
                const content = document.getElementById(
                    "ai-response-content",
                ).innerText;
                navigator.clipboard.writeText(content);
            });

        document
            .getElementById("insert-ai-response")
            ?.addEventListener("click", function () {
                const content = document.getElementById(
                    "ai-response-content",
                ).innerHTML;
                editor.focus();
                document.execCommand("insertHTML", false, content);
            });

        document
            .getElementById("replace-ai-response")
            ?.addEventListener("click", function () {
                const content = document.getElementById(
                    "ai-response-content",
                ).innerHTML;
                document.execCommand("insertHTML", false, content);
            });

        // Initial word count
        updateWordCount();
    })();
