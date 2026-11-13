/* Paper app — module 2/2: toolbar, panel and modal bindings (split from paper/paper-boot.js for #1731) */

(function (app) {
    app.bindToolbar = function (ctx) {
        const { editor, sidebar, aiPanel, slashMenu, hideSlashMenu, executeSlashCommand, insertTodo, insertTable, insertImage } = ctx;

        // Toolbar commands
        document.querySelectorAll("[data-cmd]").forEach((btn) => {
            btn.addEventListener("click", function () {
                const cmd = this.dataset.cmd;
                switch (cmd) {
                    case "bold":
                        gbFormat("bold");
                        break;
                    case "italic":
                        gbFormat("italic");
                        break;
                    case "underline":
                        gbFormat("underline");
                        break;
                    case "strikethrough":
                        gbFormat("strikeThrough");
                        break;
                    case "highlight":
                        gbFormat("hiliteColor", "#ffff00");
                        break;
                    case "alignLeft":
                        gbFormat("justifyLeft");
                        break;
                    case "alignCenter":
                        gbFormat("justifyCenter");
                        break;
                    case "alignRight":
                        gbFormat("justifyRight");
                        break;
                    case "bulletList":
                        gbFormat("insertUnorderedList");
                        break;
                    case "numberedList":
                        gbFormat("insertOrderedList");
                        break;
                    case "todoList":
                        insertTodo();
                        break;
                    case "link":
                        const url = prompt("Enter URL:");
                        if (url) gbFormat("createLink", url);
                        break;
                    case "image":
                        insertImage();
                        break;
                    case "table":
                        insertTable();
                        break;
                    case "code":
                        gbFormat("formatBlock", "pre");
                        break;
                    case "quote":
                        gbFormat("formatBlock", "blockquote",);
                        break;
                    case "undo":
                        gbFormat("undo");
                        break;
                    case "redo":
                        gbFormat("redo");
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
                    gbFormat("formatBlock", "p");
                } else {
                    gbFormat("formatBlock", value);
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
                gbFormat("insertHTML", content);
            });

        document
            .getElementById("replace-ai-response")
            ?.addEventListener("click", function () {
                const content = document.getElementById(
                    "ai-response-content",
                ).innerHTML;
                gbFormat("insertHTML", content);
            });

    };

    app.boot();
})(window.__gbPaper);
