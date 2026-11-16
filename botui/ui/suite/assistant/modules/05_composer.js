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

    /* ---- Model picker (button + menu) ---- */
    A.shortModelName = function (model) {
        if (!model) { return ""; }
        var s = model;
        var slash = s.lastIndexOf("/");
        if (slash >= 0) { s = s.substring(slash + 1); }
        var colon = s.indexOf(":");
        if (colon > 0) { s = s.substring(0, colon); }
        return s;
    };
    var shortModelName = A.shortModelName;

    A.renderModelPickers = function () {
        var pickers = document.querySelectorAll(".gb-model-picker");
        for (var p = 0; p < pickers.length; p++) { replaceWithButton(pickers[p]); }
        /* Also update any already-converted buttons */
        var btns = document.querySelectorAll(".gb-model-btn");
        for (var b = 0; b < btns.length; b++) { updateBtnLabel(btns[b]); }
    };

    function updateBtnLabel(btn) {
        var label = btn.querySelector(".gb-model-btn-label");
        if (label) { label.textContent = shortModelName(A.selectedModel || A.defaultModel || ""); }
    }

    function replaceWithButton(select) {
        if (select.dataset.replaced) { return; }
        select.dataset.replaced = "1";
        var btn = document.createElement("button");
        btn.className = "gb-model-btn";
        btn.type = "button";
        var short = shortModelName(A.selectedModel || A.defaultModel || "");
        btn.innerHTML = '<span class="gb-model-btn-label">' + (short || "Model") + '</span>' +
            '<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="6 9 12 15 18 9"/></svg>';
        btn.addEventListener("click", function (e) { e.stopPropagation(); toggleModelMenu(btn); });
        select.style.display = "none";
        select.parentNode.insertBefore(btn, select.nextSibling);
    }

    var activeModelMenu = null;
    function toggleModelMenu(anchor) {
        if (activeModelMenu) { activeModelMenu.remove(); activeModelMenu = null; return; }
        var menu = document.createElement("div");
        menu.className = "gb-model-menu";
        activeModelMenu = menu;

        var dest = A.destination || {};
        var scopeLabel = "";
        if (dest.scope === "machine") { scopeLabel = A.t("assistant.on-this-computer", "on this computer"); }
        else if (dest.scope === "network") { scopeLabel = A.t("assistant.on-your-network", "on your network") + (dest.host ? " (" + dest.host + ")" : ""); }
        else if (dest.scope === "external") { scopeLabel = A.t("assistant.external", "external") + (dest.host ? ": " + dest.host : ""); }

        /* Current/default model */
        var currentShort = shortModelName(A.defaultModel || "");
        var isSelected = !A.selectedModel || A.selectedModel === A.defaultModel;
        var item0 = document.createElement("button");
        item0.className = "gb-model-menu-item" + (isSelected ? " active" : "");
        item0.innerHTML = '<span class="gb-model-menu-name">' + esc(currentShort) + '</span>' +
            (scopeLabel ? '<span class="gb-model-menu-meta">' + esc(scopeLabel) + '</span>' : '');
        item0.addEventListener("click", function () { selectModel(A.defaultModel); });
        menu.appendChild(item0);

        /* Other tiers with different models */
        var distinctModels = [];
        var seen = {};
        if (A.defaultModel) { seen[A.defaultModel] = true; }
        for (var i = 0; i < A.tiers.length; i++) {
            var m = A.tiers[i].model;
            if (!seen[m] && !A.tiers[i].inherits_default) { seen[m] = true; distinctModels.push(A.tiers[i]); }
        }
        for (var j = 0; j < distinctModels.length; j++) {
            var tier = distinctModels[j];
            var isAct = A.selectedModel === tier.model;
            var item = document.createElement("button");
            item.className = "gb-model-menu-item" + (isAct ? " active" : "");
            item.innerHTML = '<span class="gb-model-menu-name">' + esc(shortModelName(tier.model)) + '</span>';
            (function (model) { item.addEventListener("click", function () { selectModel(model); }); })(tier.model);
            menu.appendChild(item);
        }

        /* Divider + settings link */
        var div = document.createElement("div");
        div.className = "gb-model-menu-divider";
        menu.appendChild(div);
        var action = document.createElement("button");
        action.className = "gb-model-menu-action";
        action.innerHTML = '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09"/></svg>' +
            '<span>' + esc(A.t("settings.model-switch", "Trocar modelo…")) + '</span>';
        action.addEventListener("click", function () {
            closeModelMenu();
            A.switchView("settings");
            /* Open the model section after settings renders */
            setTimeout(function () {
                var tab = document.querySelector('.gb-set-tab[data-section="model"]');
                if (tab) { tab.click(); }
            }, 80);
        });
        menu.appendChild(action);

        anchor.style.position = "relative";
        anchor.appendChild(menu);

        document.addEventListener("click", closeModelMenu, { once: true });
    }
    function closeModelMenu() {
        if (activeModelMenu) { activeModelMenu.remove(); activeModelMenu = null; }
    }
    function selectModel(model) {
        A.selectedModel = model;
        try { localStorage.setItem("gb-model-preference", model); } catch (e) { /* */ }
        A.renderModelPickers();
        A.renderStatus();
        closeModelMenu();
    }

    function esc(s) { return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;"); }

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
        renderSuggestions("gbSuggestionsDock", suggestions);
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
        messageType = messageType || 1;
        var tooBig = files.filter(function (f) { return f.size > MAX_ATTACH_BYTES; });
        if (tooBig.length) {
            A.renderError(A.t("attach.too-big", "File too large (8 MB limit per file): ") +
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
            if (typeof A.attachCard === "function") {
                (function (idx) {
                    var card = A.attachCard(A.pendingFiles[idx], {
                        removable: true,
                        onRemove: function () { A.pendingFiles.splice(idx, 1); renderFileChips(); }
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
                    for (var d = 0; d < files.length; d++) A.pendingFiles.push(files[d]);
                    renderFileChips();
                }
            });
        }
        /* Paste images/files */
        document.addEventListener("paste", function (e) {
            var items = e.clipboardData && e.clipboardData.items;
            if (!items) return;
            for (var j = 0; j < items.length; j++) {
                if (items[j].kind === "file") {
                    var f = items[j].getAsFile();
                    if (f) { A.pendingFiles.push(f); renderFileChips(); }
                }
            }
        });

        var newChat = document.getElementById("gbNewChat");
        if (newChat) {
            newChat.addEventListener("click", function () {
                A.clearSessionDismissals();
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
