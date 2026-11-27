"use strict";
function gt(key, en) { return window.gbT ? window.gbT(key, en) : en; }

/* Settings > Sobre: the "Licenças de terceiros" card. It shows the NOTICE and
 * the component record that also ship in the install folder (`licenses\`).
 * The suite serves byte-identical copies from /suite/about/licenses/, kept in
 * step by scripts/check_third_party_licenses.py. Nothing is fetched from the
 * internet. */

(function () {
    var A = window.GBAssistant;
    var BASE = "/suite/about/licenses/";
    var texts = null;
    var failed = false;
    var open = false;
    var loading = null;

    function tr(pt, en) { return A.settingsText(pt, en); }

    async function readText(name) {
        var response = await fetch(BASE + name, { cache: "no-store", credentials: "same-origin" });
        var type = response.headers.get("content-type") || "";
        if (!response.ok || type.indexOf("text/html") === 0) {
            throw new Error("licenses: " + name + " is not in this build");
        }
        return response.text();
    }

    function load() {
        if (!loading) {
            loading = Promise.all([readText("NOTICE"), readText("LICENSES-desktop.md")]).then(function (parts) {
                texts = { notice: parts[0], record: parts[1] };
                failed = false;
            }, function (error) {
                console.warn(error);
                failed = true;
                loading = null;
            });
        }
        return loading;
    }

    function body() {
        var esc = A.settingsEsc;
        if (failed) {
            return '<p class="gb-sp-lic-note" role="status">' + esc(tr("Não foi possível abrir as licenças nesta versão. Os textos estão na pasta “licenses”, dentro da pasta de instalação.",
                "The licenses could not be opened in this build. The texts are in the “licenses” folder, inside the install folder.")) + '</p>';
        }
        if (!texts) {
            return '<p class="gb-sp-lic-note" role="status">' + esc(gt("assistant-loading", "Loading…")) + '</p>';
        }
        return '<h3 class="gb-sp-lic-sub">' + esc(tr("Avisos", "Notices")) + '</h3><pre class="gb-sp-lic-text" tabindex="0">' + esc(texts.notice) +
            '</pre><h3 class="gb-sp-lic-sub">' + esc(tr("Lista de componentes", "Component record")) + '</h3><pre class="gb-sp-lic-text" tabindex="0">' +
            esc(texts.record) + '</pre>';
    }

    function toggleLabel() {
        return open ? tr("Ocultar licenças", "Hide licenses") : tr("Mostrar licenças", "Show licenses");
    }

    function card() {
        var esc = A.settingsEsc;
        return '<section class="gb-sp-card gb-sp-clip" aria-labelledby="gbLicTitle"><div class="gb-sp-head"><strong id="gbLicTitle">' +
            esc(tr("Licenças de terceiros", "Third-party licenses")) + '</strong></div><div class="gb-sp-lic"><p>' +
            esc(tr("O aplicativo usa programas de outros autores, cada um com a sua licença. Os textos completos e onde obter o código-fonte também ficam na pasta “licenses”, dentro da pasta de instalação.",
                "The app uses programs by other authors, each under its own license. The full texts, and where to get the source code, are also in the “licenses” folder, inside the install folder.")) +
            '</p><button type="button" class="gb-sp-btn" id="gbLicToggle" aria-controls="gbLicBody" aria-expanded="' + open + '">' + esc(toggleLabel()) +
            '</button><div id="gbLicBody"' + (open ? "" : " hidden") + '>' + (open ? body() : "") + '</div></div></section>';
    }

    function bind(panel) {
        var toggle = panel.querySelector("#gbLicToggle");
        var target = panel.querySelector("#gbLicBody");
        if (!toggle || !target) { return; }
        function paint() {
            target.hidden = !open;
            target.innerHTML = open ? body() : "";
            toggle.setAttribute("aria-expanded", String(open));
            toggle.textContent = toggleLabel();
        }
        toggle.onclick = function () {
            open = !open;
            paint();
            if (open && !texts) {
                load().then(function () { if (target.isConnected && open) { paint(); } });
            }
        };
    }

    var renderAboutBase = A.renderAbout;
    if (typeof renderAboutBase !== "function") { return; }
    A.renderAbout = function (panel) {
        var result = renderAboutBase.apply(this, arguments);
        panel.insertAdjacentHTML("beforeend", card());
        bind(panel);
        return result;
    };
})();
