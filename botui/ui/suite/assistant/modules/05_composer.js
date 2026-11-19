"use strict";
/**
 * 05_composer.js - Composer, send, file attach, model picker, suggestions.
 */

(function () {
    var A = window.GBAssistant;

    /* ---- Greeting ---- */
    A.renderGreeting = function () {
        var el = document.getElementById("gbGreeting");
        if (!el) { return; }
        var hour = new Date().getHours();
        var timeKey, timeFallback;
        if (hour < 12) { timeKey = "assistant.good-morning"; timeFallback = "Good morning"; }
        else if (hour < 18) { timeKey = "assistant.good-afternoon"; timeFallback = "Good afternoon"; }
        else { timeKey = "assistant.good-evening"; timeFallback = "Good evening"; }
        var greeting = A.t(timeKey, timeFallback);
        if (A.userName) { greeting += ", " + A.userName; }
        el.textContent = greeting;
    };

    /* Model picker lives in 05b_model_picker.js */

    /* ---- Suggestions (max 3, removable, persisted dismissals) ---- */
    var dismissedKey = "gb-dismissed-suggestions";

    function getDismissed() {
        try {
            var raw = localStorage.getItem(dismissedKey);
            return raw ? JSON.parse(raw) : {};
        } catch (e) { return {}; }
    }
    function setDismissed(map) {
        try { localStorage.setItem(dismissedKey, JSON.stringify(map)); } catch (e) { /* */ }
    }

    /* Session-level dismissals (cleared on new conversation) */
    var sessionDismissed = {};

    A.clearSessionDismissals = function () { sessionDismissed = {}; };

    A.onSuggestions = function (suggestions) {
        A.suggestions = suggestions;
        renderSuggestions("gbSuggestionsEmpty", suggestions);
        /* Dock suggestions belong to the empty state; hide once conversation
           has at least one message. They reappear on new conversation. */
        if (!A.hasUserSent) {
            renderSuggestions("gbSuggestionsDock", suggestions);
        }
    };

    function renderSuggestions(containerId, suggestions) {
        var container = document.getElementById(containerId);
        if (!container) { return; }
        container.innerHTML = "";
        if (!suggestions || suggestions.length === 0) { return; }
        var dismissed = getDismissed();
        var botKey = A.botName || "default";
        var botDismissed = dismissed[botKey] || [];
        var visible = [];
        for (var i = 0; i < suggestions.length; i++) {
            var s = suggestions[i];
            var text = s.text || (typeof s === "string" ? s : "");
            if (botDismissed.indexOf(text) >= 0) { continue; }
            if (sessionDismissed[text]) { continue; }
            visible.push(s);
            if (visible.length >= 3) { break; }
        }
        if (visible.length === 0) { return; }
        for (var v = 0; v < visible.length; v++) {
            container.appendChild(buildSuggestionChip(visible[v], containerId));
        }
    }

    function buildSuggestionChip(suggestion, containerId) {
        var text = suggestion.text || (typeof suggestion === "string" ? suggestion : "");
        var chip = document.createElement("button");
        chip.className = "gb-suggestion-chip";

        var label = document.createElement("span");
        label.textContent = text;
        chip.appendChild(label);

        /* Remove (x) control */
        var rm = document.createElement("span");
        rm.className = "gb-suggestion-remove";
        rm.setAttribute("role", "button");
        rm.setAttribute("aria-label", A.t("suggestion.remove", "Remove suggestion"));
        rm.innerHTML = "&times;";
        rm.addEventListener("click", function (e) {
            e.stopPropagation();
            sessionDismissed[text] = true;
            /* Also persist per-bot */
            var dismissed = getDismissed();
            var botKey = A.botName || "default";
            if (!dismissed[botKey]) { dismissed[botKey] = []; }
            if (dismissed[botKey].indexOf(text) < 0) { dismissed[botKey].push(text); }
            setDismissed(dismissed);
            renderSuggestions("gbSuggestionsEmpty", A.suggestions);
            renderSuggestions("gbSuggestionsDock", A.suggestions);
        });
        chip.appendChild(rm);

        chip.addEventListener("click", function () {
            var msgType = 1;
            var sendText = text;
            if (suggestion.action) {
                try {
                    var actionData = typeof suggestion.action === "string"
                        ? JSON.parse(suggestion.action) : suggestion.action;
                    if (actionData.type === "invoke_tool" &&
                        !actionData.prompt_for_params &&
                        !(Array.isArray(actionData.params) && actionData.params.length)) {
                        msgType = 6;
                        sendText = actionData.tool || text;
                    }
                } catch (e) { /* parse error, send as text */ }
            }
            doSend(sendText, msgType);
        });
        return chip;
    }

    /* ---- Send ---- */
    /* Attachments travel inside the socket frame as base64; a larger file
       fails the frame and the chat would stay silent. Same limit as the server
       expects from the existing client. */
    var MAX_ATTACH_BYTES = 8 * 1024 * 1024;

    function doSend(text, messageType) {
        text = (text || "").trim();
        var files = A.pendingFiles.slice();
        if (!text && files.length === 0) { return; }
        /* Block sends while streaming (message would be lost). */
        if (A.isStreaming) {
            A.announce(A.t("assistant.wait-response",
                "Aguarde a resposta terminar"));
            return;
        }
        messageType = messageType || 1;
        var tooBig = files.filter(function (f) { return f.size > MAX_ATTACH_BYTES; });
        if (tooBig.length) {
            var fmtMB = function (b) { return (b / 1048576).toFixed(1) + " MB"; };
            var names = tooBig.map(function (f) {
                return f.name + " (" + fmtMB(f.size) + ")";
            }).join(", ");
            A.renderError(A.t("attach.too-big",
                "Arquivo grande demais (limite de 8 MB por arquivo): ") + names);
            return;
        }
        var empty = files.filter(function (f) { return f.size === 0; });
        if (empty.length) {
            A.renderError(A.t("attach.empty-file",
                "Arquivo vazio: ") + empty.map(function (f) { return f.name; }).join(", "));
            return;
        }
        A.renderUserMessage(text, files);
        /* The wait shows at once: the server's first frame can take seconds
           (reading an attachment, a web search) and the screen stood still. */
        A.showThinking();
        clearInputs();
        if (files.length > 0) {
            A.pendingFiles = [];
            clearFileChips();
            encodeAndSend(text, files, messageType);
            return;
        }
        sendNow(text, null, messageType);
    }

    /* A message that could not leave (connection closed) must not keep the
       wait on screen. */
    function sendNow(text, files, messageType) {
        if (!A.sendMessage(text, files, messageType)) { A.hideThinking(); }
    }

    function encodeAndSend(text, fileList, messageType) {
        var encoded = [];
        var remaining = fileList.length;
        for (var i = 0; i < fileList.length; i++) {
            (function (file) {
                var reader = new FileReader();
                reader.onload = function () {
                    /* The shape the server reads: name + content_base64. */
                    encoded.push({
                        name: file.name,
                        content_base64: String(reader.result).split(",")[1] || "",
                    });
                    remaining--;
                    if (remaining === 0) { sendNow(text, encoded, messageType); }
                };
                reader.onerror = function () {
                    remaining--;
                    if (remaining === 0) { sendNow(text, encoded.length ? encoded : null, messageType); }
                };
                reader.readAsDataURL(file);
            })(fileList[i]);
        }
    }

    function clearInputs() {
        ["gbInputEmpty", "gbInputDock"].forEach(function (id) {
            var el = document.getElementById(id);
            if (el) { el.value = ""; autoGrow(el); }
        });
    }
    function clearFileChips() {
        var c = document.getElementById("gbFileChips");
        if (c) { c.innerHTML = ""; }
    }

    function autoGrow(el) {
        el.style.height = "auto";
        el.style.height = Math.min(el.scrollHeight, 200) + "px";
    }

    /* ---- File attachment ---- */
    function isDuplicate(file) {
        for (var i = 0; i < A.pendingFiles.length; i++) {
            if (A.pendingFiles[i].name === file.name &&
                A.pendingFiles[i].size === file.size) { return true; }
        }
        return false;
    }

    function addDroppedFiles(fileList) {
        var added = 0;
        for (var d = 0; d < fileList.length; d++) {
            if (!isDuplicate(fileList[d])) {
                A.pendingFiles.push(fileList[d]);
                added++;
            }
        }
        if (added) { renderFileChips(); }
    }

    function setupAreaDrop(el) {
        if (!el) { return; }
        ["dragenter", "dragover", "dragleave", "drop"].forEach(function (ev) {
            el.addEventListener(ev, function (e) { e.preventDefault(); });
        });
        el.addEventListener("dragenter", function () {
            el.classList.add("gb-drop-overlay");
        });
        el.addEventListener("dragleave", function (e) {
            if (!el.contains(e.relatedTarget)) {
                el.classList.remove("gb-drop-overlay");
            }
        });
        el.addEventListener("drop", function (e) {
            el.classList.remove("gb-drop-overlay");
            var files = e.dataTransfer && e.dataTransfer.files;
            if (files && files.length) { addDroppedFiles(files); }
        });
    }

    function handleFileSelect(input) {
        if (!input.files || input.files.length === 0) { return; }
        for (var i = 0; i < input.files.length; i++) {
            if (!isDuplicate(input.files[i])) {
                A.pendingFiles.push(input.files[i]);
            }
        }
        renderFileChips();
        input.value = "";
    }

    function renderFileChips() {
        var container = document.getElementById("gbFileChips");
        if (!container) { return; }
        container.innerHTML = "";
        for (var i = 0; i < A.pendingFiles.length; i++) {
            if (typeof A.attachCard === "function") {
                (function (idx) {
                    var card = A.attachCard(A.pendingFiles[idx], {
                        removable: true,
                        onRemove: function () { A.pendingFiles.splice(idx, 1); renderFileChips(); }
                    });
                    /* Keyboard: Delete/Backspace removes this chip */
                    card.setAttribute("tabindex", "0");
                    card.addEventListener("keydown", function (e) {
                        if (e.key === "Delete" || e.key === "Backspace") {
                            e.preventDefault();
                            A.pendingFiles.splice(idx, 1);
                            renderFileChips();
                            /* Move focus to the next chip or the input. */
                            var next = container.querySelector(".gb-attach-card");
                            if (next) { next.focus(); }
                            else {
                                var inp = document.getElementById("gbInputDock") ||
                                          document.getElementById("gbInputEmpty");
                                if (inp) { inp.focus(); }
                            }
                        }
                    });
                    container.appendChild(card);
                })(i);
            } else {
                var chip = document.createElement("span");
                chip.className = "gb-file-chip";
                chip.textContent = A.pendingFiles[i].name;
                container.appendChild(chip);
            }
        }
    }

    /* ---- Bind events ---- */
    A.initComposer = function () {
        var inputEmpty = document.getElementById("gbInputEmpty");
        var sendEmpty = document.getElementById("gbSendEmpty");
        var fileEmpty = document.getElementById("gbFileEmpty");
        var inputDock = document.getElementById("gbInputDock");
        var sendDock = document.getElementById("gbSendDock");
        var fileDock = document.getElementById("gbFileDock");
        var stopBtn = document.getElementById("gbStopBtn");

        function bindInput(input, sendBtn) {
            if (!input) { return; }
            input.addEventListener("keydown", function (e) {
                if (e.key === "Enter" && !e.shiftKey) {
                    e.preventDefault(); doSend(input.value);
                }
            });
            input.addEventListener("input", function () { autoGrow(input); });
            if (sendBtn) {
                sendBtn.addEventListener("click", function () { doSend(input.value); });
            }
        }
        bindInput(inputEmpty, sendEmpty);
        bindInput(inputDock, sendDock);

        if (fileEmpty) { fileEmpty.addEventListener("change", function () { handleFileSelect(fileEmpty); }); }
        if (fileDock) { fileDock.addEventListener("change", function () { handleFileSelect(fileDock); }); }
        if (stopBtn) { stopBtn.addEventListener("click", function () { A.cancelGeneration(); }); }

        /* Drag-and-drop onto the docked composer */
        var composerInner = document.querySelector(".gb-composer-inner");
        if (composerInner) {
            ["dragenter","dragover","dragleave","drop"].forEach(function (ev) {
                composerInner.addEventListener(ev, function (e) { e.preventDefault(); e.stopPropagation(); });
            });
            composerInner.addEventListener("dragover", function () { composerInner.classList.add("gb-drop-active"); });
            composerInner.addEventListener("dragleave", function (e) {
                if (!composerInner.contains(e.relatedTarget)) composerInner.classList.remove("gb-drop-active");
            });
            composerInner.addEventListener("drop", function (e) {
                composerInner.classList.remove("gb-drop-active");
                var files = e.dataTransfer && e.dataTransfer.files;
                if (files && files.length) {
                    addDroppedFiles(files);
                }
            });
        }

        /* Full-area drag-and-drop: conversation area and empty state
           both accept file drops, forwarding them to the composer. A
           semi-transparent overlay appears so the person sees the target. */
        setupAreaDrop(document.getElementById("gbConversation"));
        setupAreaDrop(document.getElementById("gbEmptyState"));
        setupAreaDrop(document.getElementById("gbChatView"));

        /* Paste images/files + long text → attachment conversion */
        var PASTE_TEXT_THRESHOLD = 4000;
        document.addEventListener("paste", function (e) {
            var items = e.clipboardData && e.clipboardData.items;
            if (!items) { return; }
            var hasFile = false;
            for (var j = 0; j < items.length; j++) {
                if (items[j].kind === "file") {
                    var f = items[j].getAsFile();
                    if (f) { A.pendingFiles.push(f); hasFile = true; }
                }
            }
            if (hasFile) { renderFileChips(); return; }
            /* Long text paste: convert to a text-file chip. */
            var text = e.clipboardData.getData("text/plain");
            if (text && text.length > PASTE_TEXT_THRESHOLD) {
                e.preventDefault();
                var blob = new Blob([text], { type: "text/plain" });
                var name = A.t("attach.pasted-text", "texto-colado") + ".txt";
                var file = new File([blob], name, { type: "text/plain" });
                A.pendingFiles.push(file);
                renderFileChips();
            }
        });

        var newChat = document.getElementById("gbNewChat");
        if (newChat) {
            newChat.addEventListener("click", function () {
                A.clearSessionDismissals();
                A.pendingFiles = [];
                clearFileChips();
                A.showEmptyState();
                A.renderGreeting();
                A.onSuggestions(A.suggestions);
                A.newSession().then(function () {
                    if (typeof A.syncAgentToggle === "function") { A.syncAgentToggle(); }
                    A.reconnectWs();
                    A.refreshSidebar();
                });
                if (inputEmpty) { inputEmpty.focus(); }
            });
        }
    };
})();
