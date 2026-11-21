"use strict";

(function () {
    var A = window.GBAssistant;
    var endpoint = "/api/settings/custom-instructions";
    var maxLength = 1500;
    var icons = [
        ["robot", "Robô", "Robot", "M12 8V4H8M4 8h16v12H4zM2 14h2M20 14h2M9 13v2M15 13v2"],
        ["spark", "Brilho", "Spark", "M12 3l2 7 7 2-7 2-2 7-2-7-7-2 7-2z"],
        ["building", "Edifício", "Building", "M3 9l9-5 9 5M3 20h18M5 10h14M6 11v7M10 11v7M14 11v7M18 11v7"],
        ["book", "Livro", "Book", "M12 7v13M12 7C9 5 6 5 3 6v13c3-1 6-1 9 1 3-2 6-2 9-1V6c-3-1-6-1-9 1z"],
        ["shield", "Escudo", "Shield", "M12 3l8 3v6c0 4-3 7-8 9-5-2-8-5-8-9V6z"],
        ["feather", "Pena", "Feather", "M5 19c-3-7 6-14 11-15 3 0 5 2 4 5-2 6-8 10-15 10zm-2 2L15 9"],
        ["leaf", "Folha", "Leaf", "M5 18c1-6 5-9 12-10M20 4c3 11-3 17-10 16-4-1-6-4-4-8 2-3 8-2 14-8z"],
        ["compass", "Bússola", "Compass", "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zm4 6-2 6-6 2 2-6z"]
    ];
    var styles = [
        ["Formal", "Formal", "Use linguagem formal e objetiva.", "Use formal, straightforward language."],
        ["Direto", "Concise", "Seja breve e vá direto ao ponto.", "Be brief and get straight to the point."],
        ["Didático", "Explanatory", "Explique passo a passo e inclua exemplos quando ajudarem.", "Explain step by step and include examples when helpful."],
        ["Técnico", "Technical", "Use terminologia técnica precisa e cite fontes.", "Use precise technical terminology and cite sources."],
        ["Amigável", "Friendly", "Responda com um tom acessível e amigável.", "Respond in an approachable, friendly tone."]
    ];

    function tr(pt, en) { return A.settingsText(pt, en); }
    function esc(value) {
        return String(value == null ? "" : value).replace(/[&<>"]/g, function (c) {
            return {"&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;"}[c];
        });
    }
    function iconMarkup(icon, size) {
        return '<svg aria-hidden="true" width="' + size + '" height="' + size +
            '" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="' +
            icon[3] + '"/></svg>';
    }
    function iconFor(id) {
        return icons.find(function (icon) { return icon[0] === id; }) || icons[0];
    }

    function safePhoto(photo) {
        return typeof photo === "string" && photo.length <= 262144 &&
            /^data:image\/(png|jpeg|webp);base64,[a-z0-9+/=]+$/i.test(photo) ? photo : "";
    }

    function apply(data) {
        A.assistantProfile = {
            name: typeof data.name === "string" ? data.name : "",
            icon: iconFor(data.icon)[0],
            photo: safePhoto(data.photo)
        };
        document.dispatchEvent(new Event("gb-assistant-profile"));
        return data;
    }
    A.assistantProfileIcon = function (size) {
        return iconMarkup(iconFor(A.assistantProfile && A.assistantProfile.icon), size);
    };

    A.loadAssistantProfile = async function () {
        var response = await A.apiFetch(endpoint);
        if (!response.ok) { throw new Error("HTTP " + response.status); }
        return apply(await response.json());
    };

    A.renderInstructions = async function (panel) {
        try {
            var data = await A.loadAssistantProfile();
            if (panel.isConnected) { draw(panel, data); }
        } catch (error) {
            if (!panel.isConnected) { return; }
            panel.innerHTML = A.settingsHeading("Persona", tr("Quem é o seu assistente.", "Who your assistant is.")) +
                '<p role="alert">' + tr("Não foi possível carregar a persona.", "Could not load your assistant profile.") +
                '</p><button type="button" class="gb-set-btn-sm">' + tr("Tentar novamente", "Try again") + '</button>';
            panel.querySelector("button").onclick = function () { A.openSettings("instructions"); };
        }
    };

    function field(id, label, value, placeholder) {
        return '<label class="gb-set-instruction-label" for="' + id + '">' + esc(label) +
            '</label><textarea class="gb-set-textarea" id="' + id + '" maxlength="' + maxLength +
            '" rows="3" placeholder="' + esc(placeholder) + '">' + esc(value) +
            '</textarea><output class="gb-set-counter" for="' + id + '"></output>';
    }

    function draw(panel, data) {
        var profile = Object.assign({}, A.assistantProfile);
        var enabled = data.enabled !== false;
        panel.innerHTML = A.settingsHeading("Persona", tr("Quem é o seu assistente.", "Who your assistant is."),
            tr("Nome, rosto e jeito de responder. Vale em todas as conversas novas.",
                "Name, appearance, and response style for new conversations.")) +
            '<form id="gbPersonaForm"><section class="gb-set-slab">' +
            '<div class="gb-set-slab-body gb-set-persona-identity"><div class="gb-set-persona-avatar" id="gbPersonaAvatar"></div>' +
            '<div><label class="gb-set-meta" for="gbPersonaName">' + tr("Nome", "Name") +
            '</label><input class="gb-set-profile-input" id="gbPersonaName" maxlength="80" value="' +
            esc(profile.name) + '" placeholder="' + tr("Assistente", "Assistant") + '"></div></div>' +
            '<div class="gb-set-slab-body"><div class="gb-set-meta" id="gbPersonaIconLabel">' +
            tr("Ícone ou foto", "Icon or photo") + '</div><div class="gb-set-icon-row" role="group" aria-labelledby="gbPersonaIconLabel">' +
            icons.map(function (icon) {
                var label = esc(tr(icon[1], icon[2]));
                return '<button type="button" class="gb-set-btn-sm" data-icon="' + icon[0] + '" aria-label="' +
                    label + '" title="' + label + '">' + iconMarkup(icon, 18) + '</button>';
            }).join("") + '<label class="gb-set-btn-sm gb-set-upload">' + tr("Enviar foto", "Upload photo") +
            '<input id="gbPersonaPhoto" type="file" accept="image/png,image/jpeg,image/webp"></label></div></div></section>' +
            '<section class="gb-set-slab"><label class="gb-set-slab-toggle" for="gbInstrEnabled">' +
            '<span>' + tr("Usar estas instruções", "Use these instructions") + '</span>' +
            '<span class="gb-set-meta" id="gbInstrState"></span>' +
            '<input type="checkbox" role="switch" class="gb-set-toggle" id="gbInstrEnabled"' + (enabled ? " checked" : "") +
            '></label><fieldset class="gb-set-slab-body gb-set-instructions">' +
            field("gbInstrAbout", tr("O que o assistente deve saber sobre você?", "What should the assistant know about you?"),
                data.about, tr("Ex.: Escrevo ofícios e relatórios.", "Example: I write letters and reports.")) +
            '<div class="gb-set-meta">' + tr("Comece por um estilo", "Start with a style") + '</div>' +
            '<div class="gb-set-icon-row">' + styles.map(function (style, index) {
                return '<button type="button" class="gb-set-btn-sm" data-style="' + index + '">' + tr(style[0], style[1]) + '</button>';
            }).join("") + '</div>' +
            field("gbInstrResponse", tr("Como o assistente deve responder?", "How should the assistant respond?"),
                data.response, tr("Ex.: Português formal, respostas curtas. Cite as fontes.", "Example: Use formal English, short answers, and cite sources.")) +
            '</fieldset><div class="gb-set-slab-body gb-set-profile-actions">' +
            '<span class="gb-set-hint" role="status"></span><button type="submit" class="gb-set-btn-sm gb-set-btn-accent">' +
            esc(A.t("settings.save", "Save")) + '</button></div></section></form>';

        var form = panel.querySelector("form");
        var avatar = form.querySelector("#gbPersonaAvatar");
        var picker = form.querySelector("#gbPersonaPhoto");
        var status = form.querySelector("[role=status]");
        var save = form.querySelector("[type=submit]");
        var toggle = form.querySelector("#gbInstrEnabled");
        var fields = form.querySelector("fieldset");
        var about = form.querySelector("#gbInstrAbout");
        var response = form.querySelector("#gbInstrResponse");
        var iconButtons = form.querySelectorAll("[data-icon]");
        var photoRevision = 0;

        function renderAvatar() {
            avatar.innerHTML = profile.photo
                ? '<img alt="" src="' + esc(profile.photo) + '">'
                : iconMarkup(iconFor(profile.icon), 28);
            iconButtons.forEach(function (button) {
                button.setAttribute("aria-pressed", !profile.photo && button.dataset.icon === profile.icon);
            });
        }
        function renderToggle() {
            fields.disabled = !toggle.checked;
            form.querySelector("#gbInstrState").textContent = toggle.checked ? tr("Ligadas", "On") : tr("Desligadas", "Off");
        }
        function count(input) {
            form.querySelector('output[for="' + input.id + '"]').textContent = input.value.length + " / " + maxLength;
        }
        renderAvatar();
        renderToggle();
        toggle.onchange = renderToggle;
        [about, response].forEach(function (input) { count(input); input.oninput = function () { count(input); }; });
        iconButtons.forEach(function (button) {
            button.onclick = function () {
                photoRevision++;
                profile.icon = button.dataset.icon;
                profile.photo = "";
                save.disabled = false;
                renderAvatar();
            };
        });
        form.querySelectorAll("[data-style]").forEach(function (button) {
            button.onclick = function () {
                var style = styles[Number(button.dataset.style)];
                response.value = tr(style[2], style[3]);
                count(response);
            };
        });
        picker.onchange = async function () {
            var file = picker.files[0];
            if (!file) { return; }
            var revision = ++photoRevision;
            save.disabled = true;
            try {
                var photo = await readPhoto(file);
                if (revision !== photoRevision) { return; }
                profile.photo = photo;
                status.textContent = "";
                renderAvatar();
            } catch (error) {
                if (revision === photoRevision) {
                    status.textContent = tr("Escolha uma imagem PNG, JPEG ou WebP de até 10 MB.", "Choose a PNG, JPEG or WebP image up to 10 MB.");
                }
            } finally {
                if (revision === photoRevision) { save.disabled = false; }
                picker.value = "";
            }
        };
        form.onsubmit = async function (event) {
            event.preventDefault();
            if (save.disabled) { return; }
            var next = Object.assign({}, profile, {
                name: form.querySelector("#gbPersonaName").value.trim(),
                about: about.value, response: response.value, enabled: toggle.checked
            });
            form.inert = true;
            save.disabled = true;
            status.textContent = "";
            try {
                var result = await A.apiFetch(endpoint, { method: "PUT", body: JSON.stringify(next) });
                if (!result.ok) { throw new Error(); }
                apply(next);
                status.textContent = tr("Salvo.", "Saved.");
            } catch (error) {
                status.textContent = tr("Não foi possível salvar a persona.", "Could not save your assistant profile.");
            } finally {
                form.inert = false;
                save.disabled = false;
            }
        };
    }

    async function readPhoto(file) {
        if (!/^image\/(png|jpeg|webp)$/.test(file.type) || file.size > 10 * 1024 * 1024) { throw new Error(); }
        var bitmap = await createImageBitmap(file);
        try {
            var edge = Math.min(bitmap.width, bitmap.height);
            var canvas = document.createElement("canvas");
            canvas.width = canvas.height = 192;
            canvas.getContext("2d").drawImage(bitmap, (bitmap.width - edge) / 2, (bitmap.height - edge) / 2,
                edge, edge, 0, 0, canvas.width, canvas.height);
            var photo = safePhoto(canvas.toDataURL("image/webp", 0.86));
            if (!photo) { throw new Error(); }
            return photo;
        } finally {
            bitmap.close();
        }
    }
})();
