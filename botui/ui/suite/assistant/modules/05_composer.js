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

    /* ---- Model picker ---- */
    A.renderModelPickers = function () {
        var pickers = document.querySelectorAll(".gb-model-picker");
        for (var p = 0; p < pickers.length; p++) { renderPicker(pickers[p]); }
    };

    function shortModelName(model) {
        if (!model) { return ""; }
        var s = model;
        var slash = s.lastIndexOf("/");
        if (slash >= 0) { s = s.substring(slash + 1); }
        var colon = s.indexOf(":");
        if (colon > 0) { s = s.substring(0, colon); }
        return s;
    }

    function renderPicker(select) {
        select.innerHTML = "";
        /* Determine distinct models among tiers */
        var distinctModels = [];
        var seen = {};
        for (var i = 0; i < A.tiers.length; i++) {
            var m = A.tiers[i].model;
            if (!seen[m]) { seen[m] = true; distinctModels.push(m); }
        }
        var singleModel = distinctModels.length <= 1;

        if (singleModel) {
            /* Show model name as plain text, disable select */
            var name = shortModelName(A.defaultModel || (A.tiers[0] && A.tiers[0].model) || "");
            var opt = document.createElement("option");
            opt.value = A.defaultModel || "";
            opt.textContent = name;
            select.appendChild(opt);
            select.disabled = true;
            return;
        }

        /* Multiple models: "Automatico . <model>" as default, then distinct tiers */
        var autoLabel = A.t("assistant.auto", "Automatic") +
            " · " + shortModelName(A.defaultModel);
        var autoOpt = document.createElement("option");
        autoOpt.value = A.defaultModel || "";
        autoOpt.textContent = autoLabel;
        if (!A.selectedModel || A.selectedModel === A.defaultModel) {
            autoOpt.selected = true;
        }
        select.appendChild(autoOpt);

        for (var j = 0; j < A.tiers.length; j++) {
            var t = A.tiers[j];
            if (t.model === A.defaultModel && t.id === "auto") { continue; }
            if (t.inherits_default) { continue; }
            var o = document.createElement("option");
            o.value = t.model;
            o.textContent = shortModelName(t.model);
            if (t.model === A.selectedModel) { o.selected = true; }
            select.appendChild(o);
        }

        select.disabled = false;
        select.addEventListener("change", function () {
            A.selectedModel = select.value;
            try { localStorage.setItem("gb-model-preference", A.selectedModel); }
            catch (e) { /* */ }
            syncPickers(A.selectedModel);
            A.renderStatus();
        });
    }

    function syncPickers(model) {
        var pickers = document.querySelectorAll(".gb-model-picker");
        for (var i = 0; i < pickers.length; i++) { pickers[i].value = model; }
    }

    /* ---- Suggestions ---- */
    A.onSuggestions = function (suggestions) {
        A.suggestions = suggestions;
        renderSuggestions("gbSuggestionsEmpty", suggestions);
        renderSuggestions("gbSuggestionsDock", suggestions);
    };

    function renderSuggestions(containerId, suggestions) {
        var container = document.getElementById(containerId);
        if (!container) { return; }
        container.innerHTML = "";
        if (!suggestions || suggestions.length === 0) { return; }
        for (var i = 0; i < suggestions.length; i++) {
            var s = suggestions[i];
            var chip = document.createElement("button");
            chip.className = "gb-suggestion-chip";
            chip.textContent = s.text || (typeof s === "string" ? s : "");
            (function (suggestion) {
                chip.addEventListener("click", function () {
                    var text = suggestion.text || (typeof suggestion === "string" ? suggestion : "");
                    var msgType = 1;
                    /* Tool exec: message_type 6 */
                    if (suggestion.action) {
                        try {
                            var actionData = typeof suggestion.action === "string"
                                ? JSON.parse(suggestion.action) : suggestion.action;
                            if (actionData.type === "invoke_tool" &&
                                !actionData.prompt_for_params &&
                                !(Array.isArray(actionData.params) && actionData.params.length)) {
                                msgType = 6;
                                text = actionData.tool || text;
                            }
                        } catch (e) { /* parse error, send as text */ }
                    }
                    doSend(text, msgType);
                });
            })(s);
            container.appendChild(chip);
        }
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
        messageType = messageType || 1;
        var tooBig = files.filter(function (f) { return f.size > MAX_ATTACH_BYTES; });
        if (tooBig.length) {
            A.renderError(A.t("attach.too-big", "Arquivo grande demais (limite de 8 MB por arquivo): ") +
                tooBig.map(function (f) { return f.name; }).join(", "));
            return;
        }
        A.renderUserMessage(text, files);
        clearInputs();
        if (files.length > 0) {
            A.pendingFiles = [];
            clearFileChips();
            encodeAndSend(text, files, messageType);
            return;
        }
        A.sendMessage(text, null, messageType);
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
                    if (remaining === 0) { A.sendMessage(text, encoded, messageType); }
                };
                reader.onerror = function () {
                    remaining--;
                    if (remaining === 0) { A.sendMessage(text, encoded.length ? encoded : null, messageType); }
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
    function handleFileSelect(input) {
        if (!input.files || input.files.length === 0) { return; }
        for (var i = 0; i < input.files.length; i++) {
            A.pendingFiles.push(input.files[i]);
        }
        renderFileChips();
        input.value = "";
    }

    function renderFileChips() {
        var container = document.getElementById("gbFileChips");
        if (!container) { return; }
        container.innerHTML = "";
        for (var i = 0; i < A.pendingFiles.length; i++) {
            var chip = document.createElement("span");
            chip.className = "gb-file-chip";
            chip.textContent = A.pendingFiles[i].name;
            var removeBtn = document.createElement("button");
            removeBtn.className = "gb-file-chip-remove";
            removeBtn.innerHTML = "&times;";
            removeBtn.setAttribute("aria-label", A.t("assistant.remove-file", "Remove file"));
            (function (idx) {
                removeBtn.addEventListener("click", function () {
                    A.pendingFiles.splice(idx, 1); renderFileChips();
                });
            })(i);
            chip.appendChild(removeBtn);
            container.appendChild(chip);
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

        var newChat = document.getElementById("gbNewChat");
        if (newChat) {
            newChat.addEventListener("click", function () {
                A.showEmptyState();
                A.renderGreeting();
                A.onSuggestions(A.suggestions);
                A.newSession().then(function () {
                    A.reconnectWs();
                    A.refreshSidebar();
                });
                if (inputEmpty) { inputEmpty.focus(); }
            });
        }
    };
})();
