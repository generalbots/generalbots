"use strict";
/* Settings > Idioma and Settings > Atalhos de teclado. */

(function () {
    var A = window.GBAssistant;
    function tr(pt, en) { return A.settingsText(pt, en); }

    A.renderLanguage = function (panel) {
        var esc = A.settingsEsc;
        var langs = [["pt-BR", "PT-BR", "Português (Brasil)"], ["en", "EN", "English"]];
        var portuguese = A.locale.startsWith("pt");
        panel.innerHTML = A.settingsHeading(tr("Idioma", "Language"), tr("Idioma do aplicativo.", "App language.")) +
            '<div class="gb-sp-langs" role="group" aria-label="' + esc(tr("Idioma", "Language")) + '">' + langs.map(function (lang) {
                var on = lang[0] === "pt-BR" ? portuguese : !portuguese;
                return '<button type="button" class="gb-sp-lang" data-lang="' + lang[0] + '" aria-pressed="' + on + '"><code>' +
                    lang[1] + '</code><span>' + esc(lang[2]) + '</span>' + (on ? A.settingsIcon("M20 6 9 17l-5-5", 16, 2.2) : "") + '</button>';
            }).join("") + '</div>';
        panel.querySelectorAll("[data-lang]").forEach(function (button) {
            button.onclick = function () {
                var code = button.dataset.lang;
                A.locale = code;
                try { localStorage.setItem("gb-locale", code); } catch (e) { /* storage unavailable */ }
                if (typeof A.translatePage === "function") { A.translatePage(); }
                if (typeof A.renderStatus === "function") { A.renderStatus(); }
                if (typeof A.renderGreeting === "function") { A.renderGreeting(); }
                if (typeof A.renderUser === "function") { A.renderUser(); }
                A.renderSettings();
            };
        });
    };

    A.renderShortcuts = function (panel) {
        var esc = A.settingsEsc;
        var mod = /mac/i.test(navigator.platform || "") ? "⌘" : "Ctrl";
        var rows = [
            [tr("Nova conversa", "New conversation"), [mod, "N"]],
            [tr("Pesquisar conversas", "Search conversations"), [mod, "K"]],
            [tr("Enviar mensagem", "Send message"), ["Enter"]],
            [tr("Quebrar linha", "New line"), ["Shift", "Enter"]],
            [tr("Aumentar o texto", "Larger text"), [mod, "="]],
            [tr("Diminuir o texto", "Smaller text"), [mod, "−"]],
            [tr("Tamanho padrão do texto", "Default text size"), [mod, "0"]]
        ];
        panel.innerHTML = A.settingsHeading(tr("Atalhos de teclado", "Keyboard shortcuts"),
            tr("Mais rápido pelo teclado.", "Faster from the keyboard.")) +
            '<div class="gb-sp-card gb-sp-clip"><ul class="gb-sp-keys">' + rows.map(function (row) {
                return '<li><span>' + esc(row[0]) + '</span><span>' + row[1].map(function (key) {
                    return '<kbd>' + esc(key) + '</kbd>';
                }).join("") + '</span></li>';
            }).join("") + '</ul></div>';
    };
})();
