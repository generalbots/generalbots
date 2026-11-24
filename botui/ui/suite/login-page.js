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
               then hand over. A per-browser shell choice from Settings wins
               over the product default so the preview survives re-login. */
            var shellChoice = "";
            try { shellChoice = localStorage.getItem("gb-shell-override") || ""; } catch (e) { /* storage disabled */ }
            if (!explicitTarget && (shellChoice === "assistant" || shellChoice === "desktop")) {
                target = shellChoice === "assistant"
                    ? "/suite/assistant/assistant.html" : "/suite/desktop.html";
            } else if (p && p.shell === "assistant" && !explicitTarget) {
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
})();
