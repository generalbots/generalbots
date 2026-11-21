"use strict";

(function () {
    var A = window.GBAssistant;
    var endpoint = "/api/settings/custom-instructions";
    var maxLength = 1500;
    var icons = [
        ["robot", "Robô", "Robot", "M12 8V4H8M4 8h16v12H4zM2 14h2M20 14h2M9 13v2M15 13v2"],
        ["spark", "Brilho", "Spark", "M12 3l1.9 5.8L20 10l-6.1 1.2L12 17l-1.9-5.8L4 10l6.1-1.2z"],
        ["building", "Prédio público", "Public building", "M3 22h18M6 18v-7M10 18v-7M14 18v-7M18 18v-7M12 2l8 5H4z"],
        ["book", "Livro", "Book", "M2 4h6a4 4 0 0 1 4 4v13a3 3 0 0 0-3-3H2zM22 4h-6a4 4 0 0 0-4 4v13a3 3 0 0 1 3-3h7z"],
        ["shield", "Escudo", "Shield", "M12 2 4 6v6c0 5 3.4 8.6 8 10 4.6-1.4 8-5 8-10V6z"],
        ["feather", "Pena", "Feather", "M20.24 12.24a6 6 0 0 0-8.49-8.49L5 10.5V19h8.5zM16 8 2 22M17.5 15H9"],
        ["leaf", "Folha", "Leaf", "M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10zM2 21c0-3 1.85-5.36 5.08-6"],
        ["compass", "Bússola", "Compass", "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zM16.24 7.76l-2.12 6.36-6.36 2.12 2.12-6.36z"]
    ];
    var styles = [
        ["Profissional", "Professional", "M3 7h18v13H3zM8 7V4h8v3",
            "Responda em português formal e objetivo, com tópicos quando ajudar. Cite a fonte quando usar um documento.",
            "Answer in formal, objective English, with bullet points when they help. Cite the source when you use a document."],
        ["Amigável", "Friendly", "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zM8 14s1.5 2 4 2 4-2 4-2M9 9h.01M15 9h.01",
            "Seja cordial e próximo, com tom de conversa e atitude positiva, sem perder a utilidade.",
            "Be warm and approachable, with a conversational, positive tone, without losing usefulness."],
        ["Direto", "Direct", "M5 12h14M13 6l6 6-6 6",
            "Vá direto ao ponto. Respostas curtas, sem rodeios e sem repetir a pergunta.",
            "Get straight to the point. Short answers, no padding, no repeating the question."],
        ["Didático", "Teaching", "M2 4h6a4 4 0 0 1 4 4v13a3 3 0 0 0-3-3H2zM22 4h-6a4 4 0 0 0-4 4v13a3 3 0 0 1 3-3h7z",
            "Explique passo a passo, com exemplos simples, como para quem está aprendendo o assunto.",
            "Explain step by step, with simple examples, as if to someone learning the subject."],
        ["Técnico", "Technical", "M16 18l6-6-6-6M8 6l-6 6 6 6",
            "Use termos técnicos com precisão e mostre o raciocínio quando a resposta depender de cálculo ou norma.",
            "Use technical terms precisely and show the reasoning when the answer depends on a calculation or a rule."]
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
                '</p><button type="button" class="gb-sp-btn">' + tr("Tentar novamente", "Try again") + '</button>';
            panel.querySelector("button").onclick = function () { A.openSettings("instructions"); };
        }
    };

    function field(id, number, label, value, placeholder, extra) {
        return '<div class="gb-sp-field"><label class="gb-sp-q" for="' + id + '"><b>' + number + '</b>' + esc(label) + '</label>' + (extra || "") +
            '<textarea class="gb-sp-area" id="' + id + '" maxlength="' + maxLength + '" placeholder="' + esc(placeholder) + '">' + esc(value) +
            '</textarea><output class="gb-sp-count" for="' + id + '"></output></div>';
    }

    function draw(panel, data) {
        var profile = Object.assign({}, A.assistantProfile);
        var enabled = data.enabled !== false;
        var photoPath = "M3 5h18v14H3zM8.5 10a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3zM21 15l-5-5L5 19";
        panel.innerHTML = A.settingsHeading("Persona", tr("Quem é o seu assistente.", "Who your assistant is."),
            tr("Nome, rosto e jeito de responder. Vale em todas as conversas novas.",
                "Name, face and way of answering. Applies to every new conversation.")) +
            '<form id="gbPersonaForm" class="gb-sp-form"><section class="gb-sp-card"><div class="gb-sp-identity">' +
            '<div class="gb-sp-face" id="gbPersonaAvatar"></div><label><span class="gb-sp-label">' + tr("Nome", "Name") +
            '</span><input class="gb-sp-input" id="gbPersonaName" maxlength="80" value="' + esc(profile.name) + '" placeholder="' +
            tr("Ex.: Assistente da Secretaria", "E.g. Department assistant") + '"></label></div><div class="gb-sp-stack">' +
            '<span class="gb-sp-label" id="gbPersonaIconLabel">' + tr("Ícone ou foto", "Icon or photo") + '</span>' +
            '<div class="gb-sp-row" role="group" aria-labelledby="gbPersonaIconLabel">' + icons.map(function (icon) {
                var label = esc(tr(icon[1], icon[2]));
                return '<button type="button" class="gb-sp-icon" data-icon="' + icon[0] + '" aria-label="' + label + '" title="' + label + '">' +
                    iconMarkup(icon, 18) + '</button>';
            }).join("") + '<i class="gb-sp-sep" aria-hidden="true"></i><label class="gb-sp-btn gb-sp-upload">' + A.settingsIcon(photoPath, 15) +
            '<span id="gbPersonaPhotoLabel"></span><input id="gbPersonaPhoto" type="file" accept="image/png,image/jpeg,image/webp"></label></div></div></section>' +
            '<section class="gb-sp-card gb-sp-clip"><button type="button" class="gb-sp-switchrow" role="switch" id="gbInstrEnabled" aria-checked="' + enabled +
            '"><span>' + tr("Usar estas instruções", "Use these instructions") + '</span><span class="gb-sp-state" id="gbInstrState"></span>' +
            '<i class="gb-sp-toggle" aria-hidden="true"></i></button><fieldset class="gb-sp-fields">' +
            field("gbInstrAbout", "01", tr("O que o assistente deve saber sobre você?", "What should the assistant know about you?"), data.about,
                tr("Ex.: Sou analista na Secretaria de Saúde. Escrevo ofícios, relatórios e respostas a pedidos de informação.",
                    "E.g. I am an analyst at the Health Department. I write letters, reports and answers to information requests.")) +
            field("gbInstrResponse", "02", tr("Como o assistente deve responder?", "How should the assistant answer?"), data.response,
                tr("Ex.: Português formal, respostas curtas, com tópicos quando ajudar. Cite a fonte quando usar um documento.",
                    "E.g. Formal English, short answers, bullet points when they help. Cite the source when you use a document."),
                '<div class="gb-sp-row gb-sp-styles"><span class="gb-sp-label">' + tr("Comece por um estilo", "Start from a style") + '</span>' +
                styles.map(function (style, index) {
                    return '<button type="button" class="gb-sp-chip" data-style="' + index + '" aria-pressed="false">' +
                        A.settingsIcon(style[2], 13, 1.8) + esc(tr(style[0], style[1])) + '</button>';
                }).join("") + '</div>') +
            '</fieldset><div class="gb-sp-foot"><span class="gb-sp-foot-status" role="status"></span><button type="submit" class="gb-sp-btn gb-sp-btn-pri" disabled>' +
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

        function dirty() {
            save.disabled = false;
            status.textContent = tr("Alterações não salvas", "Unsaved changes");
        }
        function renderAvatar() {
            avatar.innerHTML = profile.photo ? '<img alt="" src="' + esc(profile.photo) + '">' : iconMarkup(iconFor(profile.icon), 28);
            iconButtons.forEach(function (button) {
                button.setAttribute("aria-pressed", !profile.photo && button.dataset.icon === profile.icon);
            });
            form.querySelector(".gb-sp-upload").classList.toggle("gb-sp-on", !!profile.photo);
            form.querySelector("#gbPersonaPhotoLabel").textContent = profile.photo ? tr("Trocar foto", "Change photo") : tr("Enviar foto", "Upload photo");
        }
        function renderToggle() {
            var on = toggle.getAttribute("aria-checked") === "true";
            fields.disabled = !on;
            form.querySelector("#gbInstrState").textContent = on ? tr("Ligadas", "On") : tr("Desligadas", "Off");
        }
        function count(input) {
            form.querySelector('output[for="' + input.id + '"]').textContent = input.value.length + " / " + maxLength;
        }
        renderAvatar();
        renderToggle();
        toggle.onclick = function () {
            toggle.setAttribute("aria-checked", String(toggle.getAttribute("aria-checked") !== "true"));
            renderToggle();
            dirty();
        };
        form.querySelector("#gbPersonaName").oninput = dirty;
        [about, response].forEach(function (input) { count(input); input.oninput = function () { count(input); dirty(); }; });
        iconButtons.forEach(function (button) {
            button.onclick = function () {
                photoRevision++;
                profile.icon = button.dataset.icon;
                profile.photo = "";
                renderAvatar();
                dirty();
            };
        });
        form.querySelectorAll("[data-style]").forEach(function (button) {
            button.onclick = function () {
                var style = styles[Number(button.dataset.style)];
                response.value = tr(style[3], style[4]);
                form.querySelectorAll("[data-style]").forEach(function (other) { other.setAttribute("aria-pressed", String(other === button)); });
                count(response);
                dirty();
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
                renderAvatar();
                dirty();
            } catch (error) {
                if (revision === photoRevision) {
                    status.textContent = tr("Escolha uma imagem PNG, JPEG ou WebP de até 10 MB.", "Choose a PNG, JPEG or WebP image up to 10 MB.");
                    save.disabled = false;
                }
            } finally {
                picker.value = "";
            }
        };
        form.onsubmit = async function (event) {
            event.preventDefault();
            if (save.disabled) { return; }
            var next = Object.assign({}, profile, {
                name: form.querySelector("#gbPersonaName").value.trim(),
                about: about.value, response: response.value, enabled: toggle.getAttribute("aria-checked") === "true"
            });
            form.inert = true;
            save.disabled = true;
            status.textContent = "";
            try {
                var result = await A.apiFetch(endpoint, { method: "PUT", body: JSON.stringify(next) });
                if (!result.ok) { throw new Error(); }
                apply(next);
                status.textContent = "✓ " + tr("Salvo", "Saved");
            } catch (error) {
                status.textContent = tr("Não foi possível salvar a persona.", "Could not save your assistant profile.");
                save.disabled = false;
            } finally {
                form.inert = false;
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
