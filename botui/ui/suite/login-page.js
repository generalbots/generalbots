"use strict";
/**
 * login-page.js — Sign-in page logic.
 *
 * Bilingual (pt-BR / en), same language rule as the assistant shell.
 * Keeps the existing auth call, token handling and redirect unchanged.
 */
(function () {
    /* ---- language ---- */
    var lang = "pt";
    try {
        var stored = localStorage.getItem("gb-locale");
        if (stored) { lang = stored.toLowerCase().startsWith("pt") ? "pt" : "en"; }
        else { lang = (navigator.language || "en").toLowerCase().startsWith("pt") ? "pt" : "en"; }
    } catch (e) { /* fallback pt */ }

    var L = {
        pt: {
            title: "Entrar — General Bots",
            subtitle: "Entre com a conta de administrador.",
            email: "E-mail",
            password: "Senha",
            submit: "Entrar",
            busy: "Entrando…",
            showPw: "Mostrar senha",
            hidePw: "Ocultar senha",
            errWrongCreds: "E-mail ou senha incorretos.",
            errTooMany: "Muitas tentativas. Aguarde %s segundos e tente novamente.",
            errNoServer: "Não foi possível conectar ao servidor.",
            errRetry: "Tentar novamente",
            errGeneric: "Não foi possível entrar. Verifique seus dados.",
            errNoToken: "O servidor não retornou um token de sessão."
        },
        en: {
            title: "Sign in — General Bots",
            subtitle: "Sign in with the administrator account.",
            email: "E-mail",
            password: "Password",
            submit: "Sign in",
            busy: "Signing in…",
            showPw: "Show password",
            hidePw: "Hide password",
            errWrongCreds: "Wrong e-mail or password.",
            errTooMany: "Too many attempts. Wait %s seconds and try again.",
            errNoServer: "Could not reach the server.",
            errRetry: "Try again",
            errGeneric: "Could not sign in. Check your credentials.",
            errNoToken: "The server did not return a session token."
        }
    };
    var t = L[lang] || L.pt;

    /* ---- apply labels ---- */
    document.title = t.title;
    var subtitle = document.getElementById("loginSubtitle");
    if (subtitle) { subtitle.textContent = t.subtitle; }
    var labelEmail = document.getElementById("labelEmail");
    if (labelEmail) { labelEmail.textContent = t.email; }
    var labelPw = document.getElementById("labelPassword");
    if (labelPw) { labelPw.textContent = t.password; }
    var btnText = document.getElementById("loginBtnText");
    if (btnText) { btnText.textContent = t.submit; }
    var toggle = document.getElementById("pwToggle");
    if (toggle) { toggle.setAttribute("aria-label", t.showPw); }

    /* ---- session-expired notice from the assistant ---- */
    try {
        if (sessionStorage.getItem("gb-session-expired") === "1") {
            sessionStorage.removeItem("gb-session-expired");
            var expMsg = lang === "pt"
                ? "Sua sessão expirou. Entre novamente."
                : "Your session expired. Please sign in again.";
            if (subtitle) {
                subtitle.textContent = expMsg;
                subtitle.style.color = "var(--id-warn, #92400e)";
            }
        }
    } catch (e) { /* storage disabled */ }

    /* ---- redirect target ---- */
    var params = new URLSearchParams(window.location.search);
    var explicitTarget = params.get("return_to") || params.get("redirect") || "";
    if (/\/(login|signup)(\?|$)/.test(explicitTarget)) { explicitTarget = ""; }
    var target = explicitTarget || "/suite/desktop.html";

    /* ---- accent: restore from localStorage (user choice) or server ---- */
    (function loadAccent() {
        var stored = null;
        try { stored = localStorage.getItem("gb-accent"); } catch (e) { /* */ }
        if (stored && window.GBIdentity) {
            window.GBIdentity.apply(stored);
        }
        var url = (window.__GB_SERVER_ORIGIN__ || "") + "/api/product";
        fetch(url, { credentials: "same-origin" }).then(function (r) {
            return r.ok ? r.json() : null;
        }).then(function (p) {
            if (p && p.accent && window.GBIdentity && !stored) {
                window.GBIdentity.apply(p.accent);
            }
            /* Editions on the assistant shell go straight to it after the
               sign-in: the legacy desktop page would paint first and only
               then hand over. */
            if (p && p.shell === "assistant" && !explicitTarget) {
                target = "/suite/assistant/assistant.html";
            }
        }).catch(function () { /* keep defaults */ });
    })();

    function redirect(token, email) {
        try {
            localStorage.setItem("gb-access-token", token);
            localStorage.setItem("gb-user-data", JSON.stringify({
                id: email || "admin",
                name: (email || "admin").split("@")[0],
                email: email || ""
            }));
            sessionStorage.removeItem("gb-signed-out");
        } catch (e) { /* storage disabled */ }
        var sep = target.indexOf("?") === -1 ? "?" : "&";
        window.location.replace(target + sep + "token=" + encodeURIComponent(token));
    }

    /* ---- auto-login with existing token ---- */
    var existing = "";
    try { existing = localStorage.getItem("gb-access-token") || ""; } catch (e) { /* */ }
    if (existing) {
        fetch("/api/auth/me", { headers: { Authorization: "Bearer " + existing } })
            .then(function (r) { return r.ok ? r.json() : null; })
            .then(function (me) {
                if (me && !me.is_anonymous) { redirect(existing, me.email || ""); }
            })
            .catch(function () { /* show the form */ });
    }

    /* ---- show/hide password ---- */
    var pwInput = document.getElementById("login-password");
    var iconShow = document.getElementById("pwIconShow");
    var iconHide = document.getElementById("pwIconHide");
    if (toggle && pwInput) {
        toggle.addEventListener("click", function () {
            var hidden = pwInput.type === "password";
            pwInput.type = hidden ? "text" : "password";
            if (iconShow) { iconShow.style.display = hidden ? "none" : ""; }
            if (iconHide) { iconHide.style.display = hidden ? "" : "none"; }
            toggle.setAttribute("aria-label", hidden ? t.hidePw : t.showPw);
            pwInput.focus();
        });
    }

    /* ---- error display ---- */
    var errEl = document.getElementById("login-error");
    function showError(html) {
        if (!errEl) { return; }
        errEl.innerHTML = html;
        errEl.style.display = "block";
    }
    function hideError() {
        if (!errEl) { return; }
        errEl.style.display = "none";
        errEl.innerHTML = "";
    }
    function esc(s) {
        return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;")
            .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
    }

    /* ---- form submit ---- */
    var form = document.getElementById("login-form");
    var btn = document.getElementById("login-btn");
    var emailInput = document.getElementById("login-email");

    if (form) {
        form.addEventListener("submit", function (e) {
            e.preventDefault();
            hideError();

            var email = emailInput ? emailInput.value.trim() : "";
            var password = pwInput ? pwInput.value : "";

            /* Clear field-level error marks */
            if (emailInput) { emailInput.classList.remove("gb-login-input-error"); }
            if (pwInput) { pwInput.classList.remove("gb-login-input-error"); }

            /* Client-side validation */
            if (!email) {
                if (emailInput) { emailInput.classList.add("gb-login-input-error"); emailInput.focus(); }
                return;
            }
            if (!password) {
                if (pwInput) { pwInput.classList.add("gb-login-input-error"); pwInput.focus(); }
                return;
            }

            /* Busy state */
            if (btn) { btn.disabled = true; }
            if (btnText) { btnText.textContent = t.busy; }

            fetch("/api/auth/login", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ email: email, password: password })
            }).then(function (response) {
                return response.text().then(function (text) {
                    var data = {};
                    try { data = JSON.parse(text); } catch (ex) { data = { error: text }; }
                    return { status: response.status, ok: response.ok, data: data };
                });
            }).then(function (result) {
                if (!result.ok) {
                    var status = result.status;
                    var data = result.data;
                    if (status === 401 || status === 403) {
                        showError(esc(t.errWrongCreds));
                        if (pwInput) { pwInput.classList.add("gb-login-input-error"); pwInput.focus(); pwInput.select(); }
                    } else if (status === 429) {
                        var wait = data.retry_after || data.wait || 30;
                        showError(esc(t.errTooMany.replace("%s", String(wait))));
                    } else {
                        showError(esc(t.errGeneric));
                    }
                    resetButton();
                    return;
                }
                var token = result.data.access_token || result.data.token;
                if (!token) {
                    showError(esc(t.errNoToken));
                    resetButton();
                    return;
                }
                redirect(token, result.data.email || email);
            }).catch(function () {
                showError(
                    esc(t.errNoServer) +
                    ' <a id="loginRetry">' + esc(t.errRetry) + '</a>'
                );
                var retryLink = document.getElementById("loginRetry");
                if (retryLink) {
                    retryLink.addEventListener("click", function () {
                        hideError();
                        resetButton();
                        if (emailInput) { emailInput.focus(); }
                    });
                }
                resetButton();
            });
        });
    }

    function resetButton() {
        if (btn) { btn.disabled = false; }
        if (btnText) { btnText.textContent = t.submit; }
    }

    /* ---- focus: email first ---- */
    if (emailInput) { emailInput.focus(); }

    /* ---- forgot password: desktop app only, reset through the shell ---- */
    var tauri = window.__TAURI__ && window.__TAURI__.core;
    if (!form || !tauri || typeof tauri.invoke !== "function") { return; }
    var R = {
        pt: {
            link: "Esqueci minha senha", title: "Criar uma nova senha",
            note: "Isto troca a senha do administrador deste computador e encerra as outras sessões. Só funciona aqui, no aplicativo.",
            fresh: "Nova senha", again: "Confirme a nova senha",
            hint: "Pelo menos 8 caracteres, com letra maiúscula, letra minúscula, número e símbolo.",
            save: "Salvar e entrar", saving: "Salvando…", back: "Voltar para entrar",
            mismatch: "As duas senhas não são iguais.",
            password_too_short: "A nova senha precisa ter pelo menos 8 caracteres.",
            password_too_long: "A nova senha pode ter no máximo 128 caracteres.",
            password_needs_lowercase: "A nova senha precisa de uma letra minúscula.",
            password_needs_uppercase: "A nova senha precisa de uma letra maiúscula.",
            password_needs_digit: "A nova senha precisa de um número.",
            password_needs_symbol: "A nova senha precisa de um símbolo, como ! ou #.",
            recovery_unavailable: "Não foi possível falar com o assistente neste computador. Feche e abra o aplicativo e tente de novo.",
            recovery_refused: "A recuperação de senha só funciona pelo aplicativo, neste computador.",
            no_local_administrator: "Este computador não tem uma conta de administrador para recuperar.",
            password_not_saved: "Não foi possível salvar a nova senha.",
            failed: "Não foi possível trocar a senha. Tente de novo."
        },
        en: {
            link: "Forgot password", title: "Create a new password",
            note: "This replaces the password of this computer's administrator and signs out other sessions. It only works here, in the app.",
            fresh: "New password", again: "Confirm the new password",
            hint: "At least 8 characters, with an uppercase letter, a lowercase letter, a number and a symbol.",
            save: "Save and sign in", saving: "Saving…", back: "Back to sign in",
            mismatch: "The two passwords are not the same.",
            password_too_short: "The new password needs at least 8 characters.",
            password_too_long: "The new password can have at most 128 characters.",
            password_needs_lowercase: "The new password needs a lowercase letter.",
            password_needs_uppercase: "The new password needs an uppercase letter.",
            password_needs_digit: "The new password needs a number.",
            password_needs_symbol: "The new password needs a symbol, such as ! or #.",
            recovery_unavailable: "Could not reach the assistant on this computer. Close and reopen the app, then try again.",
            recovery_refused: "Password recovery only works from the app, on this computer.",
            no_local_administrator: "This computer has no administrator account to recover.",
            password_not_saved: "The new password could not be saved.",
            failed: "Could not change the password. Try again."
        }
    };
    var r = R[lang] || R.pt;
    var forgot = document.createElement("button");
    forgot.type = "button";
    forgot.className = "gb-login-forgot";
    forgot.textContent = r.link;
    form.appendChild(forgot);
    var box = document.createElement("form");
    box.className = "gb-login-recovery";
    box.noValidate = true;
    box.hidden = true;
    box.innerHTML = '<p class="gb-login-recovery-title">' + esc(r.title) + '</p>' +
        '<p class="gb-login-note">' + esc(r.note) + '</p>' +
        '<div class="gb-login-field"><label for="recovery-new">' + esc(r.fresh) + '</label>' +
        '<input id="recovery-new" type="password" autocomplete="new-password" aria-describedby="recovery-hint" required /></div>' +
        '<div class="gb-login-field"><label for="recovery-again">' + esc(r.again) + '</label>' +
        '<input id="recovery-again" type="password" autocomplete="new-password" required /></div>' +
        '<label class="gb-login-reveal"><input type="checkbox" id="recovery-reveal" /> ' + esc(t.showPw) + '</label>' +
        '<p class="gb-login-note" id="recovery-hint">' + esc(r.hint) + '</p>' +
        '<button type="submit" class="gb-login-submit" id="recovery-save">' + esc(r.save) + '</button>' +
        '<div class="gb-login-error" id="recovery-error" role="alert" aria-live="polite"></div>' +
        '<button type="button" class="gb-login-forgot" id="recovery-back">' + esc(r.back) + '</button>';
    form.parentNode.insertBefore(box, form.nextSibling);
    var fresh = box.querySelector("#recovery-new");
    var again = box.querySelector("#recovery-again");
    var save = box.querySelector("#recovery-save");
    var recErr = box.querySelector("#recovery-error");
    function recoveryError(text) {
        recErr.textContent = text;
        recErr.style.display = text ? "block" : "none";
    }
    function showRecovery(open) {
        form.hidden = open;
        box.hidden = !open;
        recoveryError("");
        (open ? fresh : emailInput || forgot).focus();
    }
    forgot.addEventListener("click", function () { showRecovery(true); });
    box.querySelector("#recovery-back").addEventListener("click", function () { showRecovery(false); });
    box.querySelector("#recovery-reveal").addEventListener("change", function (e) {
        fresh.type = again.type = e.target.checked ? "text" : "password";
    });
    box.addEventListener("submit", function (e) {
        e.preventDefault();
        recoveryError("");
        if (fresh.value !== again.value) { recoveryError(r.mismatch); again.focus(); return; }
        save.disabled = true;
        save.textContent = r.saving;
        tauri.invoke("local_password_reset", { newPassword: fresh.value }).then(function (result) {
            redirect(result.access_token, result.email || "");
        }).catch(function (code) {
            if (String(code) === "recovery_cancelled") { recoveryError(""); }
            else { recoveryError(r[String(code)] || r.failed); }
            save.disabled = false;
            save.textContent = r.save;
            fresh.focus();
        });
    });
})();
