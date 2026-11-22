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
    };

    var releases = "https://github.com/generalbots/generalbots/releases";
    var lastCheck = null;

    function aboutRow(key, value, missing) {
        return '<div class="gb-sp-k">' + esc(key) + '</div><div class="gb-sp-v' + (missing ? " gb-sp-none" : "") + '">' + esc(value) + '</div>';
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
            A.settingsIcon("M7 17 17 7M8 7h9v9", 11, 3) + '</a></div></section>';
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
