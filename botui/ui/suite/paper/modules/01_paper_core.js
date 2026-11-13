/* Paper app — module 1/2: slash menu, editor commands and AI helpers (split from paper/paper-boot.js for #1731) */

if (window.GBAppLifecycle) GBAppLifecycle.begin("paper");

(function (app) {
    // Shared editor elements, assigned by init() once the HTMX partial lands.
    let editor = null;
    let title = null;
    let slashMenu = null;
    let aiPanel = null;
    let sidebar = null;

    // Slash command handling
    let slashPosition = null;

    // The editor markup arrives via HTMX partials (03_canvas.html), so the
    // element may not exist yet when this script runs. Poll briefly instead
    // of throwing on a null element.
    function boot() {
        const editor = document.getElementById("editor-content");
        if (!editor) { setTimeout(boot, 100); return; }
        init(editor);
    }

    function init(editorArg) {
        editor = editorArg;
        title = document.getElementById("paper-title");
        slashMenu = document.getElementById("slash-menu");
        aiPanel = document.getElementById("ai-panel");
        sidebar = document.getElementById("paper-sidebar");

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
                        gbFormat("bold");
                        break;
                    case "i":
                        e.preventDefault();
                        gbFormat("italic");
                        break;
                    case "u":
                        e.preventDefault();
                        gbFormat("underline");
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
                    gbFormat("formatBlock", "h1");
                    break;
                case "h2":
                    gbFormat("formatBlock", "h2");
                    break;
                case "h3":
                    gbFormat("formatBlock", "h3");
                    break;
                case "bullet":
                    gbFormat("insertUnorderedList");
                    break;
                case "number":
                    gbFormat("insertOrderedList");
                    break;
                case "todo":
                    insertTodo();
                    break;
                case "quote":
                    gbFormat("formatBlock", "blockquote");
                    break;
                case "code":
                    gbFormat("formatBlock", "pre");
                    break;
                case "divider":
                    gbFormat("insertHTML", "<hr>");
                    break;
                case "callout":
                    gbFormat("insertHTML", '<div class="callout">💡 </div>',);
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
            gbFormat("insertHTML", html);
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
            gbFormat("insertHTML", html);
        }

        function insertImage() {
            // #2315 — prompt() is blocked in the desktop shell, so inserting an
            // image was a silent no-op there. GBModal works in both contexts.
            GBModal.prompt("Image URL", "", "Insert image", function (url) {
                if (!url) return;
                const safe = String(url).replace(/"/g, "&quot;").trim();
                if (!/^(https?:|data:image\/|\/)/i.test(safe)) {
                    GBModal.alert("Only http(s), data:image or site-relative URLs are allowed.");
                    return;
                }
                gbFormat("insertHTML", `<img src="${safe}" style="max-width: 100%;">`);
            });
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

        app.bindToolbar({
            editor: editor,
            sidebar: sidebar,
            aiPanel: aiPanel,
            slashMenu: slashMenu,
            hideSlashMenu: hideSlashMenu,
            executeSlashCommand: executeSlashCommand,
            insertTodo: insertTodo,
            insertTable: insertTable,
            insertImage: insertImage,
        });

        // Initial word count
        updateWordCount();
    }

    Object.assign(app, { boot: boot, init: init });
})(window.__gbPaper = window.__gbPaper || {});
