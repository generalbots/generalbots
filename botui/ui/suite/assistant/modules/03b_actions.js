"use strict";
/**
 * 03b_actions.js - Per-message action toolbars.
 *
 * Assistant messages: Copy (markdown source), Regenerate.
 * User messages: Copy, Edit (inline editor, resend as new turn).
 *
 * Server limitation: the WS protocol has no "regenerate" or "edit-and-
 * resend-from-here" command. Regenerate resends the same user text as
 * a new turn and removes the old answer from the DOM (the history still
 * holds it). Edit sends the edited text as a new turn.
 */

(function () {
    var A = window.GBAssistant;
    var I = window.GBIcon || function () { return ""; };
    function pt() { return A.locale && A.locale.startsWith("pt"); }
    function t(ptT, enT) { return pt() ? ptT : enT; }

    /* ================================================================
       Assistant toolbar: Copy + Regenerate
       On hover / focus, and always visible on the LAST assistant msg.
       ================================================================ */
    A.addAssistantToolbar = function (el, content, retryable) {
        var tb = document.createElement("div");
        tb.className = "gb-msg-toolbar";

        /* Copy */
        if (content) {
            var cp = document.createElement("button");
            cp.className = "gb-msg-toolbar-btn";
            cp.title = t("Copiar", "Copy");
            cp.setAttribute("aria-label", t("Copiar", "Copy"));
            cp.innerHTML = I("copy", 14) + '<span class="gb-toolbar-label">' +
                A.escapeHtml(t("Copiar", "Copy")) + '</span>';
            (function (txt, btn) {
                btn.addEventListener("click", function () {
                    navigator.clipboard.writeText(txt).then(function () {
                        var lbl = btn.querySelector(".gb-toolbar-label");
                        if (lbl) {
                            lbl.textContent = t("Copiado", "Copied");
                            setTimeout(function () { lbl.textContent = t("Copiar", "Copy"); }, 1500);
                        }
                    }).catch(function () { /* clipboard denied */ });
                });
            })(content, cp);
            tb.appendChild(cp);
        }

        /* Regenerate */
        var regen = document.createElement("button");
        regen.className = "gb-msg-toolbar-btn";
        regen.title = t("Regenerar", "Regenerate");
        regen.setAttribute("aria-label", t("Regenerar", "Regenerate"));
        regen.innerHTML = I("arrows-clockwise", 14) + '<span class="gb-toolbar-label">' +
            A.escapeHtml(t("Regenerar", "Regenerate")) + '</span>';
        regen.addEventListener("click", function () {
            if (A.isStreaming) { return; }
            /* Find the last user message before this assistant message */
            var userMsgs = document.querySelectorAll("#gbMessages .gb-msg-user");
            var lastUser = userMsgs.length ? userMsgs[userMsgs.length - 1] : null;
            if (!lastUser) { return; }
            var userText = (lastUser.dataset.originalContent || lastUser.textContent || "").trim();
            if (!userText) { return; }
            /* Remove this assistant message from DOM */
            el.remove();
            /* Resend as a new turn */
            if (typeof A.sendMessage === "function") {
                A.sendMessage(userText);
            }
        });
        tb.appendChild(regen);

        el.appendChild(tb);
    };

    /* ================================================================
       User toolbar: Copy + Edit
       ================================================================ */
    A.addUserToolbar = function (el, content) {
        var tb = document.createElement("div");
        tb.className = "gb-msg-toolbar gb-msg-toolbar-user";

        /* Copy */
        if (content) {
            var cp = document.createElement("button");
            cp.className = "gb-msg-toolbar-btn";
            cp.title = t("Copiar", "Copy");
            cp.setAttribute("aria-label", t("Copiar", "Copy"));
            cp.innerHTML = I("copy", 14);
            (function (txt, btn) {
                btn.addEventListener("click", function () {
                    navigator.clipboard.writeText(txt).then(function () {
                        btn.title = t("Copiado", "Copied");
                        setTimeout(function () { btn.title = t("Copiar", "Copy"); }, 1500);
                    }).catch(function () { /* clipboard denied */ });
                });
            })(content, cp);
            tb.appendChild(cp);
        }

        /* Edit */
        var ed = document.createElement("button");
        ed.className = "gb-msg-toolbar-btn";
        ed.title = t("Editar", "Edit");
        ed.setAttribute("aria-label", t("Editar", "Edit"));
        ed.innerHTML = I("pencil", 14);
        (function (msgEl) {
            ed.addEventListener("click", function () {
                if (A.isStreaming) { return; }
                startEdit(msgEl);
            });
        })(el);
        tb.appendChild(ed);

        el.appendChild(tb);
    };

    /* ================================================================
       Inline edit
       ================================================================ */
    function startEdit(msgEl) {
        if (msgEl.querySelector(".gb-edit-area")) { return; }
        var bubble = msgEl.querySelector(".gb-msg-user-bubble");
        if (!bubble) { return; }
        var original = msgEl.dataset.originalContent || bubble.textContent || "";
        var savedHtml = bubble.innerHTML;

        bubble.innerHTML = "";
        bubble.classList.add("gb-editing");

        var area = document.createElement("textarea");
        area.className = "gb-edit-area";
        area.value = original;
        area.rows = Math.min(Math.max(original.split("\n").length, 2), 8);
        bubble.appendChild(area);

        var bar = document.createElement("div");
        bar.className = "gb-edit-bar";

        var saveBtn = document.createElement("button");
        saveBtn.className = "gb-edit-save";
        saveBtn.textContent = t("Salvar e enviar", "Save & send");
        bar.appendChild(saveBtn);

        var cancelBtn = document.createElement("button");
        cancelBtn.className = "gb-edit-cancel";
        cancelBtn.textContent = t("Cancelar", "Cancel");
        bar.appendChild(cancelBtn);

        bubble.appendChild(bar);
        area.focus();
        area.setSelectionRange(area.value.length, area.value.length);

        function cancel() {
            bubble.classList.remove("gb-editing");
            bubble.innerHTML = "";
            bubble.textContent = original;
        }
        function save() {
            var edited = area.value.trim();
            if (!edited) { cancel(); return; }
            bubble.classList.remove("gb-editing");
            bubble.innerHTML = "";
            bubble.textContent = edited;
            msgEl.dataset.originalContent = edited;
            /* Send as a new turn (server has no edit-and-resend) */
            if (typeof A.sendMessage === "function") {
                A.renderUserMessage(edited, []);
                A.sendMessage(edited);
            }
        }

        saveBtn.addEventListener("click", save);
        cancelBtn.addEventListener("click", cancel);
        area.addEventListener("keydown", function (e) {
            if (e.key === "Escape") { e.preventDefault(); cancel(); }
            if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); save(); }
        });
    }
})();
