"use strict";

(function () {
    var A = window.GBAssistant;
    var ENDPOINT = "/api/settings/chat-profile";
    var colors = [
        ["#171717", "Preto", "Black"], ["#2453a0", "Azul", "Blue"],
        ["#087956", "Verde", "Green"], ["#7543a2", "Violeta", "Violet"],
        ["#a44b03", "Laranja", "Orange"]
    ];

    function tr(pt, en) { return A.settingsText(pt, en); }
    function esc(value) {
        return String(value == null ? "" : value).replace(/[&<>"]/g, function (c) {
            return {"&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;"}[c];
        });
    }

    function apply(data) {
        var name = typeof data.display_name === "string" ? data.display_name : "";
        var color = /^#[a-f0-9]{6}$/i.test(data.avatar_color || "") ? data.avatar_color : "";
        A.accountProfile = { display_name: name, avatar_color: color };
        A.avatarColor = color;
        if (name) { A.userName = name; }
        if (A.renderUser) { A.renderUser(); }
        if (A.renderGreeting) { A.renderGreeting(); }
        document.dispatchEvent(new Event("gb-assistant-profile"));
        return A.accountProfile;
    }

    A.loadChatProfile = async function () {
        var jobs = [
            A.apiFetch(ENDPOINT).then(function (r) { return r.ok ? r.json().then(apply) : null; }),
            typeof A.loadAssistantProfile === "function" ? A.loadAssistantProfile() : null
        ];
        await Promise.allSettled(jobs);
    };

    A.renderAccount = async function (panel) {
        var data = A.accountProfile;
        if (!data) {
            await A.loadChatProfile();
            data = A.accountProfile;
        }
        if (!panel.isConnected) { return; }
        var accountName = A.loginUserName || A.userName || A.userEmail || "";
        var name = (data && data.display_name) || A.userName || accountName;
        var color = (data && data.avatar_color) || colors[0][0];
        var initials = name.trim().slice(0, 1).toUpperCase();
        panel.innerHTML = A.settingsHeading(tr("Conta", "Account"), tr("Quem está usando.", "Who’s using the assistant.")) +
            '<section class="gb-set-slab"><h2>' + tr("Como você aparece no chat", "How you appear in chat") + '</h2>' +
            '<form id="gbChatProfile" class="gb-set-slab-body gb-set-profile-form">' +
            '<label class="gb-set-meta" for="gbDisplayName">' + tr("Nome de exibição", "Display name") + '</label>' +
            '<input class="gb-set-profile-input" id="gbDisplayName" required maxlength="80" value="' + esc(name) + '">' +
            '<div class="gb-set-meta" id="gbAvatarColorLabel">' + tr("Cor do avatar", "Avatar color") + '</div>' +
            '<div class="gb-set-icon-row" role="group" aria-labelledby="gbAvatarColorLabel">' +
            colors.map(function (swatch) {
                var label = esc(tr(swatch[1], swatch[2]));
                return '<button type="button" class="gb-set-btn-sm" data-color="' + swatch[0] +
                    '" aria-label="' + label + '" title="' + label + '"><i style="background:' +
                    swatch[0] + '"></i></button>';
            }).join("") +
            '</div><div class="gb-set-meta">' + tr("Prévia", "Preview") + '</div>' +
            '<div class="gb-set-profile-preview"><strong id="gbProfilePreviewName">' + esc(name) +
            '</strong><span class="gb-set-profile-avatar" id="gbProfilePreviewAvatar">' + esc(initials) +
            '</span></div><div class="gb-set-profile-actions">' +
            '<span role="status" class="gb-set-hint"></span><button type="submit" class="gb-set-btn-sm gb-set-btn-accent">' +
            esc(A.t("settings.save", "Save")) + '</button></div></form></section>' +
            '<section class="gb-set-slab"><h2>' + esc(accountName) + '</h2>' +
            details([[tr("E-mail", "Email"), A.userEmail || "—"]]) +
            '<div class="gb-set-slab-body"><button type="button" class="gb-set-signout" id="gbSetSignOut">' +
            esc(A.t("assistant.sign-out", "Sign out")) + '</button></div></section>';
        panel.querySelector("#gbSetSignOut").onclick = function () { A.signOut(); };
        var form = panel.querySelector("#gbChatProfile");
        var input = form.querySelector("#gbDisplayName");
        var avatar = form.querySelector("#gbProfilePreviewAvatar");
        var previewName = form.querySelector("#gbProfilePreviewName");
        var status = form.querySelector("[role=status]");
        var submit = form.querySelector("[type=submit]");
        var swatches = form.querySelectorAll("[data-color]");
        function preview() {
            previewName.textContent = input.value;
            avatar.textContent = input.value.trim().slice(0, 1).toUpperCase();
            avatar.style.backgroundColor = color;
            swatches.forEach(function (swatch) { swatch.setAttribute("aria-pressed", swatch.dataset.color === color); });
        }
        preview();
        input.addEventListener("input", preview);
        swatches.forEach(function (swatch) {
            swatch.onclick = function () { color = swatch.dataset.color; preview(); };
        });
        form.onsubmit = async function (event) {
            event.preventDefault();
            var displayName = input.value.trim();
            if (!displayName || submit.disabled) { return; }
            submit.disabled = true;
            status.textContent = "";
            var next = { display_name: displayName, avatar_color: color };
            try {
                var response = await A.apiFetch(ENDPOINT, { method: "PUT", body: JSON.stringify(next) });
                if (!response.ok) { throw new Error(); }
                apply(next);
                status.textContent = tr("Salvo.", "Saved.");
            } catch (error) {
                status.textContent = tr("Não foi possível salvar.", "Could not save your profile.");
            } finally {
                submit.disabled = false;
            }
        };
    };

    function details(rows) {
        return '<dl class="gb-set-details">' + rows.map(function (row) {
            return '<div><dt>' + esc(row[0]) + '</dt><dd>' + esc(row[1]) + '</dd></div>';
        }).join("") + '</dl>';
    }

    var releases = "https://github.com/generalbots/generalbots/releases";
    var lastCheck = null;

    A.renderAbout = function (panel) {
        var product = A.productConfig || {};
        var missing = tr("Não informado", "Unavailable");
        var tauri = window.__TAURI_INTERNALS__ || (window.__TAURI__ && window.__TAURI__.core);
        panel.innerHTML = '<section class="gb-set-slab"><div class="gb-set-about-heading">' +
            '<strong class="gb-set-about-logo" aria-label="General Bots">GB</strong>' +
            '<span class="gb-set-meta">' + tr("Edição Lite", "Lite edition") + '</span>' +
            '<button type="button" class="gb-set-btn-sm" id="gbCheckUpdate">' +
            tr("Procurar atualizações", "Check for updates") + '</button></div>' + details([
                [tr("Produto", "Product"), product.name || "General Bots"],
                [tr("Edição", "Edition"), "Lite"],
                [tr("Versão", "Version"), product.version || missing],
                [tr("Última verificação", "Last checked"), lastCheck ? lastCheck.toLocaleString(A.locale) : missing],
                [tr("Data de compilação", "Build date"), product.build_date || product.built_at || missing],
                ["Commit", String(product.commit || product.git_commit || "").slice(0, 8) || missing],
                [tr("Licença", "License"), "MIT"]
            ]) + '<dl class="gb-set-details gb-set-about-credits"><div><dt>Design System</dt><dd>Mantis · ' +
            tr("criado por", "created by") + ' <a href="https://www.sentient.net.br" target="_blank" rel="noopener noreferrer">Sentient</a>' +
            '</dd></div></dl><div class="gb-set-slab-body gb-set-about-status" role="status" hidden></div></section>';
        var button = panel.querySelector("#gbCheckUpdate");
        button.onclick = async function () {
            if (!tauri || typeof tauri.invoke !== "function") {
                window.open(releases, "_blank", "noopener,noreferrer");
                return;
            }
            button.disabled = true;
            var status = panel.querySelector("[role=status]");
            status.hidden = false;
            status.textContent = tr("Procurando atualizações…", "Checking for updates…");
            try {
                var info = await tauri.invoke("check_release_update");
                lastCheck = new Date();
                if (!panel.isConnected) { return; }
                A.renderAbout(panel);
                status = panel.querySelector("[role=status]");
                status.hidden = false;
                if (info.update_available) {
                    status.textContent = tr("Versão disponível", "Version available") + " " + info.latest_version + " ";
                    var link = document.createElement("button");
                    link.type = "button";
                    link.className = "gb-set-btn-sm";
                    link.textContent = tr("Abrir lançamento", "Open release");
                    link.onclick = function () {
                        tauri.invoke("open_release_url", { url: info.release_url }).catch(function () {
                            status.textContent = tr("Não foi possível abrir o lançamento.", "Could not open the release.");
                        });
                    };
                    status.appendChild(link);
                } else {
                    status.textContent = tr("Você está usando a versão mais recente.", "You’re using the latest version.");
                }
            } catch (error) {
                status.textContent = tr("Não foi possível procurar atualizações.", "Could not check for updates.");
            } finally {
                button.disabled = false;
            }
        };
    };
})();
