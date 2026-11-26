"use strict";
/* Settings > Conta (chat profile, saved as it changes) and Settings > Sobre. */

(function () {
    var A = window.GBAssistant;
    var ENDPOINT = "/api/settings/chat-profile";
    var ACCENT = "accent";
    var colors = [["#171717", "Grafite", "Graphite"], [ACCENT, "Acento", "Accent"], ["#9cbdfe", "Azul", "Blue"],
        ["#64d7a6", "Verde", "Green"], ["#c0b0fe", "Roxo", "Purple"]];
    var logoutPath = "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4M16 17l5-5-5-5M21 12H9";

    function tr(pt, en) { return A.settingsText(pt, en); }
    function esc(value) { return A.settingsEsc(value); }
    function accentHex() {
        return getComputedStyle(document.documentElement).getPropertyValue("--id-accent").trim().toLowerCase();
    }
    function inkOn(hex) {
        var rgb = hex.slice(1).match(/../g).map(function (part) {
            var c = parseInt(part, 16) / 255;
            return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
        });
        var lum = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
        return 1.05 / (lum + 0.05) >= (lum + 0.05) / 0.0589 ? "#ffffff" : "#171717";
    }
    function initials(name) {
        return (name || "?").trim().split(/\s+/).map(function (word) { return word[0]; }).join("").slice(0, 2).toUpperCase() || "?";
    }

    function apply(data) {
        var name = typeof data.display_name === "string" ? data.display_name : "";
        var color = /^#[a-f0-9]{6}$/i.test(data.avatar_color || "") ? data.avatar_color : "";
        A.accountProfile = { display_name: name, avatar_color: color };
        A.avatarColor = color;
        if (name) { A.userName = name; }
        if (A.renderUser) { A.renderUser(); }
        var railAvatar = document.getElementById("gbUserAvatar");
        var own = color && color.toLowerCase() !== "#171717";
        if (railAvatar) { railAvatar.style.color = own ? inkOn(color) : ""; }
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

    function roleLabel() {
        var roles = A.userRoles || [];
        if (!roles.length) { return ""; }
        return roles.some(function (r) { return /admin/i.test(r); }) ? tr("Administrador", "Administrator") : tr("Usuário", "User");
    }

    /* Change password: the server checks the current one and writes the local
       record and the directory together; other sessions are signed out. */
    function passwordProblem(code) {
        var messages = {
            password_too_short: ["A nova senha precisa ter pelo menos 8 caracteres.", "The new password needs at least 8 characters."],
            password_too_long: ["A nova senha pode ter no máximo 128 caracteres.", "The new password can have at most 128 characters."],
            password_needs_lowercase: ["A nova senha precisa de uma letra minúscula.", "The new password needs a lowercase letter."],
            password_needs_uppercase: ["A nova senha precisa de uma letra maiúscula.", "The new password needs an uppercase letter."],
            password_needs_digit: ["A nova senha precisa de um número.", "The new password needs a number."],
            password_needs_symbol: ["A nova senha precisa de um símbolo, como ! ou #.", "The new password needs a symbol, such as ! or #."],
            password_unchanged: ["A nova senha precisa ser diferente da atual.", "The new password must be different from the current one."],
            current_password_wrong: ["A senha atual não está correta.", "The current password is not correct."],
            password_refused: ["O diretório recusou esta senha. Escolha outra.", "The directory refused this password. Choose a different one."],
            directory_unavailable: ["O serviço de senhas não está disponível agora. Tente de novo em instantes.", "The password service is not available right now. Try again in a moment."],
            password_not_saved: ["Não foi possível salvar a nova senha.", "The new password could not be saved."],
            mismatch: ["A nova senha e a confirmação não são iguais.", "The new password and its confirmation are not the same."]
        };
        var pair = messages[code] || ["Não foi possível alterar a senha. Tente de novo.", "Could not change the password. Try again."];
        return tr(pair[0], pair[1]);
    }

    function renderPasswordCard(panel) {
        var field = function (id, label, autocomplete, last) {
            return '<label class="gb-sp-k' + (last ? ' gb-sp-last' : '') + '" for="' + id + '">' + label + '</label><div class="gb-sp-v' +
                (last ? ' gb-sp-last' : '') + '"><input class="gb-sp-input" id="' + id + '" type="password" autocomplete="' + autocomplete + '" maxlength="128"></div>';
        };
        panel.insertAdjacentHTML("beforeend", '<section class="gb-sp-card" id="gbPasswordCard"><div class="gb-sp-head"><strong>' +
            tr("Alterar senha", "Change password") + '</strong></div><form class="gb-sp-grid" novalidate>' +
            field("gbPwCurrent", tr("Senha atual", "Current password"), "current-password") +
            field("gbPwNew", tr("Nova senha", "New password"), "new-password") +
            field("gbPwConfirm", tr("Confirme a nova senha", "Confirm new password"), "new-password", true) +
            '</form><p class="gb-sp-pw-hint" id="gbPwHint">' + tr("Pelo menos 8 caracteres, com letra maiúscula, letra minúscula, número e símbolo. As outras sessões serão encerradas.",
                "At least 8 characters, with an uppercase letter, a lowercase letter, a number and a symbol. Other sessions will be signed out.") +
            '</p><p class="gb-sp-error" role="alert" hidden></p><p class="gb-sp-pw-ok" role="status" hidden></p>' +
            '<div class="gb-sp-foot"><label class="gb-sp-pw-reveal"><input type="checkbox" id="gbPwReveal"> ' + tr("Mostrar senhas", "Show passwords") +
            '</label><button type="button" class="gb-sp-btn gb-sp-btn-pri" id="gbPwSave">' + tr("Alterar senha", "Change password") + '</button></div></section>');
        var card = panel.querySelector("#gbPasswordCard");
        var inputs = card.querySelectorAll("input[type=password]");
        var current = card.querySelector("#gbPwCurrent");
        var fresh = card.querySelector("#gbPwNew");
        var confirm = card.querySelector("#gbPwConfirm");
        var error = card.querySelector(".gb-sp-error");
        var ok = card.querySelector(".gb-sp-pw-ok");
        var button = card.querySelector("#gbPwSave");
        fresh.setAttribute("aria-describedby", "gbPwHint");
        function say(target, text) { error.hidden = ok.hidden = true; if (text) { target.textContent = text; target.hidden = false; } }
        card.querySelector("#gbPwReveal").onchange = function (event) {
            inputs.forEach(function (input) { input.type = event.target.checked ? "text" : "password"; });
        };
        async function submit() {
            say(error, "");
            if (!current.value) { say(error, passwordProblem("current_password_wrong")); current.focus(); return; }
            if (fresh.value !== confirm.value) { say(error, passwordProblem("mismatch")); confirm.focus(); return; }
            button.disabled = true;
            try {
                var response = await A.apiFetch("/api/auth/password", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ current_password: current.value, new_password: fresh.value })
                });
                var data = await response.json().catch(function () { return {}; });
                if (!response.ok || !data.success) { throw new Error(data.code || ""); }
                current.value = fresh.value = confirm.value = "";
                say(ok, tr("Senha alterada. Use a nova senha na próxima vez que entrar.", "Password changed. Use the new password next time you sign in."));
            } catch (problem) {
                say(error, passwordProblem(problem.message));
            } finally {
                button.disabled = false;
            }
        }
        button.onclick = submit;
        card.querySelector("form").addEventListener("keydown", function (event) {
            if (event.key === "Enter") { event.preventDefault(); submit(); }
        });
    }

    A.renderAccount = async function (panel) {
        if (!A.accountProfile) { await A.loadChatProfile(); }
        if (!panel.isConnected) { return; }
        var data = A.accountProfile || {};
        var accountName = A.loginUserName || A.userName || A.userEmail || "";
        var name = data.display_name || A.userName || accountName;
        var color = (data.avatar_color || colors[0][0]).toLowerCase();
        if (color === accentHex()) { color = ACCENT; }
        var role = roleLabel();
        var admin = (A.userRoles || []).some(function (r) { return /admin/i.test(r); });
        var now = new Date().toLocaleTimeString(A.locale, { hour: "2-digit", minute: "2-digit" });
        panel.innerHTML = A.settingsHeading(tr("Conta", "Account"), tr("Quem está usando.", "Who’s using the assistant.")) +
            '<section class="gb-sp-card"><div class="gb-sp-head"><strong>' + tr("Como você aparece no chat", "How you appear in chat") +
            '</strong></div><div class="gb-sp-grid"><label class="gb-sp-k" for="gbDisplayName">' + tr("Nome de exibição", "Display name") +
            '</label><div class="gb-sp-v"><input class="gb-sp-input" id="gbDisplayName" maxlength="80" value="' + esc(name) +
            '" placeholder="' + tr("Como quer ser chamado", "What should we call you") + '"></div>' +
            '<div class="gb-sp-k" id="gbAvatarColorLabel">' + tr("Cor do avatar", "Avatar colour") + '</div>' +
            '<div class="gb-sp-swatches" role="group" aria-labelledby="gbAvatarColorLabel">' + colors.map(function (swatch) {
                var label = esc(tr(swatch[1], swatch[2]));
                return '<button type="button" class="gb-sp-swatch" data-color="' + swatch[0] + '" aria-label="' + label + '" title="' + label +
                    '"><i style="background:' + (swatch[0] === ACCENT ? "var(--id-accent)" : swatch[0]) + '"></i></button>';
            }).join("") + '</div><div class="gb-sp-k gb-sp-last">' + tr("Prévia", "Preview") + '</div>' +
            '<div class="gb-sp-preview gb-sp-last"><div class="gb-sp-preview-who"><time>' + esc(now) + '</time><b id="gbProfilePreviewName"></b>' +
            '<span class="gb-sp-ava" id="gbProfilePreviewAvatar"></span></div><div class="gb-sp-bubble">' +
            tr("Resuma este ofício em três tópicos.", "Summarise this letter in three points.") + '</div></div></div>' +
            '<p class="gb-sp-error" role="status" hidden></p></section>' +
            '<section class="gb-sp-card gb-sp-clip"><div class="gb-sp-person"><span class="gb-sp-ava">' + esc(initials(accountName).slice(0, 1)) +
            '</span><span class="gb-sp-who"><b>' + esc(accountName) + '</b><small>' + esc(A.userEmail || "") + '</small></span>' +
            (role ? '<span class="gb-sp-badge">' + esc(role) + '</span>' : '') + '</div>' +
            '<div class="gb-sp-grid gb-sp-info"><div class="gb-sp-k">' + tr("Nome", "Name") + '</div><div class="gb-sp-v">' + esc(accountName) +
            '</div><div class="gb-sp-k' + (admin ? '' : ' gb-sp-last') + '">' + tr("E-mail", "Email") + '</div><div class="gb-sp-v gb-sp-mono' +
            (admin ? '' : ' gb-sp-last') + '">' + esc(A.userEmail || "—") + '</div>' + (admin ? '<div class="gb-sp-k gb-sp-last">' + tr("Acesso", "Access") +
            '</div><div class="gb-sp-v gb-sp-last">' + tr("Criado na primeira execução deste computador", "Created on this computer’s first run") + '</div>' : '') + '</div><div class="gb-sp-foot"><button type="button" class="gb-sp-btn gb-sp-btn-danger" id="gbSetSignOut">' +
            A.settingsIcon(logoutPath, 14, 1.8) + esc(A.t("assistant.sign-out", "Sign out")) + '</button></div></section>';
        panel.querySelector("#gbSetSignOut").onclick = function () { A.signOut(); };
        var input = panel.querySelector("#gbDisplayName");
        var avatar = panel.querySelector("#gbProfilePreviewAvatar");
        var previewName = panel.querySelector("#gbProfilePreviewName");
        var status = panel.querySelector("[role=status]");
        var swatches = panel.querySelectorAll("[data-color]");
        var timer = 0;
        function hex() { return color === ACCENT ? accentHex() : color; }
        function preview() {
            previewName.textContent = input.value;
            avatar.textContent = initials(input.value);
            avatar.style.background = color === ACCENT ? "var(--id-accent)" : color;
            avatar.style.color = /^#[0-9a-f]{6}$/i.test(hex()) ? inkOn(hex()) : "";
            swatches.forEach(function (swatch) { swatch.setAttribute("aria-pressed", swatch.dataset.color === color); });
        }
        async function save() {
            clearTimeout(timer);
            var displayName = input.value.trim();
            if (!displayName || !/^#[0-9a-f]{6}$/i.test(hex())) { return; }
            var next = { display_name: displayName, avatar_color: hex() };
            try {
                var response = await A.apiFetch(ENDPOINT, { method: "PUT", body: JSON.stringify(next) });
                if (!response.ok) { throw new Error(); }
                apply(next);
                status.hidden = true;
            } catch (error) {
                status.hidden = false;
                status.textContent = tr("Não foi possível salvar.", "Could not save your profile.");
            }
        }
        preview();
        input.addEventListener("input", function () { preview(); clearTimeout(timer); timer = setTimeout(save, 600); });
        input.addEventListener("change", save);
        swatches.forEach(function (swatch) {
            swatch.onclick = function () { color = swatch.dataset.color; preview(); save(); };
        });
        renderPasswordCard(panel);
    };

    var releases = "https://github.com/generalbots/generalbots/releases";
    var lastCheck = null;

    function aboutRow(key, value, missing) {
        return '<div class="gb-sp-k">' + esc(key) + '</div><div class="gb-sp-v' + (missing ? " gb-sp-none" : "") + '">' + esc(value) + '</div>';
    }

    /* Collaborators of github.com/GeneralBots/generalbots with their commits
     * on the default branch (GitHub, 2026-11-25), most first. Kept in the
     * page: the app works offline, so nothing is fetched and no avatar is
     * loaded from GitHub. Maintainers are the repository admins. */
    var collaborators = [
        ["rodrigorodriguez", 4257, true], ["thipatriota", 242, true], ["jramos-br", 22],
        ["ChristopherCastilho", 11], ["danielolima96", 5], ["ZuilhoSe", 2],
        ["AnaPaulaGil", 0], ["AtyllaL", 0], ["christopherdecastilho", 0], ["Dariojunior3", 0],
        ["flavioandrade91", 0], ["HeraldoAlmeida", 0], ["jonathasc", 0], ["lpicanco", 0],
        ["marcosvelasco", 0], ["oerlabshenrique", 0], ["operations-pragmatismo", 0, true],
        ["othonlima", 0], ["RobsonDantasE", 0]
    ];
    /* How many the card shows before "Mostrar todos"; the stylesheet hides
     * the rest (`.gb-sp-people:not(.is-all) > li:nth-child(n + 7)`). */
    var collaboratorsShown = 6;
    var collaboratorsOpen = false;

    function collaboratorsCard() {
        var items = collaborators.map(function (person) {
            var login = person[0];
            var commits = person[1];
            var facts = [];
            if (commits > 0) {
                facts.push(commits.toLocaleString(A.locale) + " " + (commits === 1 ? "commit" : "commits"));
            }
            if (person[2]) { facts.push(tr("Mantenedor", "Maintainer")); }
            return '<li><a class="gb-sp-person" href="https://github.com/' + encodeURIComponent(login) + '" target="_blank" rel="noopener noreferrer">' +
                '<span class="gb-sp-person-badge" aria-hidden="true">' + esc(login.charAt(0).toUpperCase()) + '</span>' +
                '<span class="gb-sp-person-text"><span class="gb-sp-person-name">' + esc(login) + '</span>' +
                (facts.length ? '<span class="gb-sp-person-facts">' + esc(facts.join(" · ")) + '</span>' : '') + '</span></a></li>';
        }).join("");
        var more = collaborators.length > collaboratorsShown ? '<div class="gb-sp-people-foot"><button type="button" class="gb-sp-btn" id="gbPeopleToggle" aria-controls="gbPeopleList" aria-expanded="' +
            collaboratorsOpen + '">' + esc(collaboratorsOpen ? tr("Mostrar menos", "Show less") : tr("Mostrar todos", "Show all") + " (" + collaborators.length + ")") + '</button></div>' : '';
        return '<section class="gb-sp-card gb-sp-clip" aria-labelledby="gbPeopleTitle"><div class="gb-sp-head"><strong id="gbPeopleTitle">' +
            esc(tr("Colaboradores", "Collaborators")) + '</strong><span class="gb-sp-people-count">' + esc(tr("por contribuição", "by contribution")) +
            ' · github.com/GeneralBots/generalbots</span></div><ul class="gb-sp-people' + (collaboratorsOpen ? ' is-all' : '') +
            '" id="gbPeopleList">' + items + '</ul>' + more + '</section>';
    }

    function bindCollaborators(panel) {
        var toggle = panel.querySelector("#gbPeopleToggle");
        if (!toggle) { return; }
        toggle.onclick = function () {
            collaboratorsOpen = !collaboratorsOpen;
            panel.querySelector("#gbPeopleList").classList.toggle("is-all", collaboratorsOpen);
            toggle.setAttribute("aria-expanded", String(collaboratorsOpen));
            toggle.textContent = collaboratorsOpen ? tr("Mostrar menos", "Show less") : tr("Mostrar todos", "Show all") + " (" + collaborators.length + ")";
        };
    }

    function known(value) { return value && !/^unknown$/i.test(String(value)) ? String(value) : ""; }

    A.renderAbout = function (panel, state) {
        var product = A.productConfig || {};
        var tauri = window.__TAURI_INTERNALS__ || (window.__TAURI__ && window.__TAURI__.core);
        var built = known(product.build_date || product.built_at);
        var commit = known(product.commit || product.git_commit).slice(0, 8);
        var version = known(product.version).replace(/^v/i, "");
        var checked = lastCheck ? tr("Hoje, ", "Today, ") + lastCheck.toLocaleTimeString(A.locale, { hour: "2-digit", minute: "2-digit" }) : "";
        var label = state === "checking" ? tr("Procurando…", "Checking…") :
            state === "latest" ? tr("Você está na versão mais recente", "You’re on the latest version") : tr("Procurar atualizações", "Check for updates");
        panel.innerHTML = '<section class="gb-sp-card gb-sp-clip"><div class="gb-sp-about-head"><span class="id-lockup" role="img" aria-label="General Bots Lite">' +
            '<span class="id-lockup-mark" aria-hidden="true"></span><span class="id-lockup-tag" aria-hidden="true">LITE</span></span>' +
            '<button type="button" class="gb-sp-btn" id="gbCheckUpdate"' +
            (state === "checking" ? ' aria-busy="true" disabled' : '') + '>' + A.settingsIcon("M21 12a9 9 0 1 1-3-6.7L21 8M21 3v5h-5", 13, 2) +
            esc(label) + '</button></div><div class="gb-sp-about-status" role="status" hidden></div><div class="gb-sp-grid gb-sp-about">' +
            aboutRow(tr("Produto", "Product"), product.name || "General Bots") +
            aboutRow(tr("Edição", "Edition"), "Lite") +
            aboutRow(tr("Versão", "Version"), version || tr("Não informada", "Unavailable"), !version) +
            aboutRow(tr("Última verificação", "Last checked"), checked || tr("Não verificada", "Not checked"), !checked) +
            aboutRow(tr("Data de compilação", "Build date"), built || tr("Não informada", "Unavailable"), !built) +
            aboutRow("Commit", commit || tr("Não informado", "Unavailable"), !commit) +
            aboutRow(tr("Licença", "License"), "MIT") +
            '<div class="gb-sp-k">Design System</div><div class="gb-sp-v">Mantis · ' + tr("criado por", "created by") +
            ' <a href="https://www.sentient.net.br" target="_blank" rel="noopener noreferrer">Sentient</a></div></div>' +
            '<div class="gb-sp-about-foot"><span>' + tr("Código aberto, escrito em Rust.", "Open source, written in Rust.") +
            '</span><a href="https://github.com/GeneralBots" target="_blank" rel="noopener noreferrer">' + tr("Ver o código", "View the code") +
            A.settingsIcon("M7 17 17 7M8 7h9v9", 11, 3) + '</a></div></section>' + collaboratorsCard();
        bindCollaborators(panel);
        var button = panel.querySelector("#gbCheckUpdate");
        button.onclick = async function () {
            if (!tauri || typeof tauri.invoke !== "function") {
                window.open(releases, "_blank", "noopener,noreferrer");
                return;
            }
            A.renderAbout(panel, "checking");
            var info = null;
            var failed = false;
            try {
                info = await tauri.invoke("check_release_update");
                lastCheck = new Date();
            } catch (error) {
                failed = true;
            }
            if (!panel.isConnected) { return; }
            A.renderAbout(panel, info && !info.update_available ? "latest" : "");
            if (!failed && !(info && info.update_available)) { return; }
            var status = panel.querySelector("[role=status]");
            status.hidden = false;
            if (failed) {
                status.textContent = tr("Não foi possível procurar atualizações.", "Could not check for updates.");
                return;
            }
            status.textContent = tr("Versão disponível: ", "Version available: ") + info.latest_version;
            var open = document.createElement("button");
            open.type = "button";
            open.className = "gb-sp-btn";
            open.textContent = tr("Abrir lançamento", "Open release");
            open.onclick = function () {
                tauri.invoke("open_release_url", { url: info.release_url }).catch(function () {
                    status.textContent = tr("Não foi possível abrir o lançamento.", "Could not open the release.");
                });
            };
            status.appendChild(open);
        };
    };
})();
