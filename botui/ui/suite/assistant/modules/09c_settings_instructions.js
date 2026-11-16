"use strict";
/**
 * 09c_settings_instructions.js - Custom instructions section for settings.
 */

(function () {
    var A = window.GBAssistant;

    function esc(s) {
        return String(s == null ? "" : s)
            .replace(/&/g, "&amp;").replace(/</g, "&lt;")
            .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
    }

    /** Render the custom instructions section in the settings panel. */
    A.renderInstructions = function (panel) {
        var data = { about: "", response: "", enabled: true };
        panel.innerHTML = '<div class="gb-set-loading"><span class="gb-set-loading-dot"></span>' +
            esc(A.t("assistant.connecting", "Loading...")) + '</div>';
        A.apiFetch("/api/settings/custom-instructions").then(function (r) {
            if (!r.ok) { throw new Error("HTTP " + r.status); }
            return r.json();
        }).then(function (json) {
            data.about = (typeof json.about === "string") ? json.about : "";
            data.response = (typeof json.response === "string") ? json.response : "";
            data.enabled = json.enabled !== false;
            drawInstructions(panel, data);
        }).catch(function () {
            drawInstructions(panel, data);
        });
    };

    function drawInstructions(panel, data) {
        var MAX = 1500;
        var html =
            '<div class="gb-set-card">' +
            '<h2 class="gb-set-title">' + esc(A.t("settings.instructions", "Custom instructions")) + '</h2>' +
            '<p class="gb-set-desc">' + esc(A.t("settings.instructions-desc",
                "Tell the assistant about yourself and how you want it to respond.")) + '</p>' +
            '<div class="gb-set-toggle-row">' +
                '<label class="gb-set-label" for="gbInstrEnabled">' +
                    esc(A.t("settings.instructions-enabled", "Enable custom instructions")) + '</label>' +
                '<input type="checkbox" id="gbInstrEnabled" class="gb-set-toggle"' +
                    (data.enabled ? ' checked' : '') + ' />' +
            '</div>' +
            '</div>' +
            '<div class="gb-set-card" id="gbInstrFields"' + (data.enabled ? '' : ' style="opacity:0.5;pointer-events:none"') + '>' +
                '<label class="gb-set-label" for="gbInstrAbout">' +
                    esc(A.t("settings.instructions-about",
                        "What should the assistant know about you?")) + '</label>' +
                '<textarea id="gbInstrAbout" class="gb-set-textarea" maxlength="' + MAX + '" rows="4">' +
                    esc(data.about) + '</textarea>' +
                '<span class="gb-set-counter" id="gbInstrAboutCount">' +
                    data.about.length + ' / ' + MAX + '</span>' +
                '<label class="gb-set-label" for="gbInstrResponse" style="margin-top:12px">' +
                    esc(A.t("settings.instructions-response",
                        "How should the assistant respond?")) + '</label>' +
                '<textarea id="gbInstrResponse" class="gb-set-textarea" maxlength="' + MAX + '" rows="4">' +
                    esc(data.response) + '</textarea>' +
                '<span class="gb-set-counter" id="gbInstrRespCount">' +
                    data.response.length + ' / ' + MAX + '</span>' +
                '<button class="gb-set-btn-sm" id="gbInstrSave" style="margin-top:8px">' +
                    esc(A.t("settings.save", "Save")) + '</button>' +
                '<span class="gb-set-hint" id="gbInstrHint"></span>' +
            '</div>';
        panel.innerHTML = html;

        var toggle = document.getElementById("gbInstrEnabled");
        var fields = document.getElementById("gbInstrFields");
        if (toggle && fields) {
            toggle.addEventListener("change", function () {
                data.enabled = toggle.checked;
                fields.style.opacity = data.enabled ? "" : "0.5";
                fields.style.pointerEvents = data.enabled ? "" : "none";
                saveInstructions(data);
            });
        }

        var aboutTA = document.getElementById("gbInstrAbout");
        var respTA = document.getElementById("gbInstrResponse");
        var aboutCount = document.getElementById("gbInstrAboutCount");
        var respCount = document.getElementById("gbInstrRespCount");
        if (aboutTA && aboutCount) {
            aboutTA.addEventListener("input", function () {
                aboutCount.textContent = aboutTA.value.length + " / " + MAX;
            });
        }
        if (respTA && respCount) {
            respTA.addEventListener("input", function () {
                respCount.textContent = respTA.value.length + " / " + MAX;
            });
        }

        var saveBtn = document.getElementById("gbInstrSave");
        if (saveBtn) {
            saveBtn.addEventListener("click", function () {
                data.about = aboutTA ? aboutTA.value : "";
                data.response = respTA ? respTA.value : "";
                saveInstructions(data);
            });
        }
    }

    function saveInstructions(data) {
        var hint = document.getElementById("gbInstrHint");
        A.apiFetch("/api/settings/custom-instructions", {
            method: "PUT",
            body: JSON.stringify({
                about: data.about,
                response: data.response,
                enabled: data.enabled
            })
        }).then(function (r) {
            if (hint) {
                hint.textContent = r.ok
                    ? A.t("settings.saved-server", "Saved.")
                    : A.t("settings.saved-local", "Saved on this device.");
            }
        }).catch(function () {
            if (hint) { hint.textContent = A.t("settings.saved-local", "Saved on this device."); }
        });
    }
})();
