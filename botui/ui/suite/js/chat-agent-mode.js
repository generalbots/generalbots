/**
 * Chat Agent Mode — handles toggling between Agent and Chat mode,
 * multi-panel layout management, and WebSocket message routing
 * for thought process, terminal output, browser preview, and step tracking.
 */
(function () {
    "use strict";

    var agentMode = false;
    var currentStep = 0;
    var totalSteps = 0;
    var terminalLineCount = 0;

    function initAgentMode() {
        setupModeToggle();
        setupToggleSwitches();
        setupStepNavigation();
        setupQuickActions();
        setupSidebarItems();
    }

    function setupModeToggle() {
        var agentBtn = document.getElementById("modeAgentBtn");
        var chatBtn = document.getElementById("modeChatBtn");
        if (!agentBtn || !chatBtn) return;

        agentBtn.addEventListener("click", function () {
            setMode("agent");
        });
        chatBtn.addEventListener("click", function () {
            setMode("chat");
        });
    }

    function setMode(mode) {
        var chatApp = document.getElementById("chat-app");
        var agentBtn = document.getElementById("modeAgentBtn");
        var chatBtn = document.getElementById("modeChatBtn");
        var quickActions = document.getElementById("quickActions");

        if (!chatApp || !agentBtn || !chatBtn) return;

        agentMode = mode === "agent";

        agentBtn.classList.toggle("active", agentMode);
        chatBtn.classList.toggle("active", !agentMode);

        if (agentMode) {
            chatApp.classList.add("agent-mode");
            if (quickActions) quickActions.style.display = "none";
        } else {
            chatApp.classList.remove("agent-mode");
            if (quickActions) quickActions.style.display = "";
        }
    }

    function setupToggleSwitches() {
        var planToggle = document.getElementById("togglePlan");
        var yoloToggle = document.getElementById("toggleYolo");

        if (planToggle) {
            planToggle.addEventListener("click", function () {
                this.classList.toggle("on");
                emitModeChange();
            });
        }
        if (yoloToggle) {
            yoloToggle.addEventListener("click", function () {
                this.classList.toggle("on");
                emitModeChange();
            });
        }
    }

    function emitModeChange() {
        var planOn = document.getElementById("togglePlan");
        var yoloOn = document.getElementById("toggleYolo");
        var mode = "plan";
        if (yoloOn && yoloOn.classList.contains("on")) {
            mode = "yolo";
        }
        if (window.ws && window.ws.readyState === WebSocket.OPEN) {
            window.ws.send(JSON.stringify({
                type: "toggle_mode",
                mode: mode
            }));
        }
    }

    function setupStepNavigation() {
        var prevBtn = document.getElementById("stepPrev");
        var nextBtn = document.getElementById("stepNext");

        if (prevBtn) {
            prevBtn.addEventListener("click", function () {
                if (currentStep > 1) {
                    currentStep--;
                    updateStepCounter();
                }
            });
        }
        if (nextBtn) {
            nextBtn.addEventListener("click", function () {
                if (currentStep < totalSteps) {
                    currentStep++;
                    updateStepCounter();
                }
            });
        }
    }

    function updateStepCounter() {
        var display = document.getElementById("stepCounterText");
        if (display) {
            display.textContent = currentStep + " / " + totalSteps;
        }
    }

    function setupQuickActions() {
        var chips = document.querySelectorAll(".quick-action-chip");
        chips.forEach(function (chip) {
            chip.addEventListener("click", function () {
                var action = this.getAttribute("data-action");
                var prompts = {
                    "full-stack": "Create a full-stack web application",
                    "writing": "Help me write ",
                    "data-insight": "Analyze data and provide insights",
                    "magic-design": "Design a beautiful UI for "
                };
                var input = document.getElementById("messageInput");
                if (input && prompts[action]) {
                    input.value = prompts[action];
                    input.focus();
                }
            });
        });
    }

    function setupSidebarItems() {
        var items = document.querySelectorAll(".agent-sidebar-item");
        items.forEach(function (item) {
            item.addEventListener("click", function () {
                items.forEach(function (i) { i.classList.remove("active"); });
                this.classList.add("active");
            });
        });
    }

    /* ===========================================
       Agent Mode WebSocket Message Handlers
       =========================================== */

    function handleAgentMessage(data) {
        if (!agentMode) return;

        switch (data.type) {
            case "thought_process":
                renderThoughtProcess(data.content);
                break;
            case "terminal_output":
                appendTerminalLine(data.line, data.stream);
                break;
            case "browser_ready":
                showBrowserPreview(data.url);
                break;
            case "step_progress":
                currentStep = data.current;
                totalSteps = data.total;
                updateStepCounter();
                break;
            case "step_complete":
                break;
            case "todo_update":
                renderTodoList(data.todos);
                break;
            case "agent_status":
                updateAgentInfo(data);
                break;
            case "file_created":
                incrementBadge("explorerBadge");
                break;
        }
    }

    function renderThoughtProcess(content) {
        var messages = document.getElementById("messages");
        if (!messages) return;

        var block = document.createElement("div");
        block.className = "thought-process";
        block.innerHTML =
            '<button class="thought-process-header">' +
            '<span class="thought-process-toggle">▶</span>' +
            '<span>Thought Process</span>' +
            "</button>" +
            '<div class="thought-process-body">' + escapeForHtml(content) + "</div>";

        var header = block.querySelector(".thought-process-header");
        header.addEventListener("click", function () {
            block.classList.toggle("expanded");
        });

        messages.appendChild(block);
    }

    function appendTerminalLine(text, stream) {
        var terminal = document.getElementById("terminalPanelContent");
        if (!terminal) return;

        var line = document.createElement("div");
        line.className = "terminal-line " + (stream || "stdout");
        line.textContent = text;
        terminal.appendChild(line);
        terminal.scrollTop = terminal.scrollHeight;

        terminalLineCount++;
        incrementBadge("terminalBadge");
    }

    function showBrowserPreview(url) {
        var content = document.getElementById("browserPanelContent");
        var urlBar = document.getElementById("browserUrlBar");
        if (!content || !urlBar) return;

        urlBar.value = url;
        content.innerHTML = '<iframe src="' + url + '" sandbox="allow-scripts allow-same-origin"></iframe>';
    }

    function renderTodoList(todos) {
        var messages = document.getElementById("messages");
        if (!messages) return;

        var existing = messages.querySelector(".agent-todo-list:last-child");
        if (existing) existing.remove();

        var list = document.createElement("div");
        list.className = "agent-todo-list";

        var headerHtml = '<div class="agent-todo-header">' +
            '<span>📋 Todos</span>' +
            '<span class="agent-todo-count">' + todos.length + "</span>" +
            "</div>";

        var itemsHtml = todos.map(function (todo) {
            var doneClass = todo.done ? " done" : "";
            var checkMark = todo.done ? "✓" : "";
            return '<div class="agent-todo-item' + doneClass + '">' +
                '<span class="agent-todo-check">' + checkMark + "</span>" +
                '<span>' + escapeForHtml(todo.text) + "</span>" +
                "</div>";
        }).join("");

        list.innerHTML = headerHtml + itemsHtml;
        messages.appendChild(list);
    }

    function updateAgentInfo(data) {
        var nameEl = document.getElementById("agentNameDisplay");
        var levelEl = document.getElementById("agentLevelBadge");
        var modelEl = document.getElementById("agentModelDisplay");

        if (nameEl && data.name) nameEl.textContent = data.name;
        if (levelEl && data.level) {
            levelEl.textContent = data.level;
            levelEl.className = "agent-level-badge badge-" + data.level.toLowerCase();
        }
        if (modelEl && data.model) {
            modelEl.textContent = data.model + " — " + (data.usage || 0) + "%";
        }
    }

    function incrementBadge(badgeId) {
        var badge = document.getElementById(badgeId);
        if (!badge) return;
        var count = parseInt(badge.textContent, 10) || 0;
        badge.textContent = count + 1;
        badge.style.display = "";
    }

    function escapeForHtml(text) {
        var div = document.createElement("div");
        div.textContent = text || "";
        return div.innerHTML;
    }

    /* ===========================================
       Expose to global scope
       =========================================== */

    window.AgentMode = {
        init: initAgentMode,
        handleMessage: handleAgentMessage,
        setMode: setMode,
        isActive: function () { return agentMode; }
    };

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", initAgentMode);
    } else {
        initAgentMode();
    }
})();
