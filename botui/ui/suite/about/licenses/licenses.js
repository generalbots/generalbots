"use strict";

/**
 * #2480 — Settings → About → Licenses, with no terminal needed.
 *
 * The appliance ships the authoritative record, the NOTICE and the MIT text
 * under `/usr/share/doc/general-bots/licenses/`. The same record is committed
 * under `/suite/about/licenses/` so the Suite can read it over HTTP in every
 * deployment (desktop, cloud image and dev). `check_third_party_licenses.py`
 * fails CI if the two copies drift, so there is one source of truth.
 *
 * Plain script, loaded after `about-inline.js`; no modules, no CDN.
 */
(function () {
    const BASE = "/suite/about/licenses/";
    const NOTICE_URL = BASE + "NOTICE";
    const RECORD_URL = BASE + "LICENSES-desktop.md";

    function t(key, fallback) {
        return window.gbT ? window.gbT(key, fallback) : fallback;
    }

    function panel() {
        return document.getElementById("gb-licenses-panel");
    }

    function setStatus(message, isError) {
        const status = document.getElementById("gb-licenses-status");
        if (!status) return;
        status.textContent = message;
        status.dataset.state = isError ? "error" : "ready";
    }

    async function readText(url) {
        const response = await fetch(url, { cache: "no-store" });
        if (!response.ok) {
            throw new Error("HTTP " + response.status + " for " + url);
        }
        return response.text();
    }

    function fill(targetId, text) {
        const target = document.getElementById(targetId);
        if (target) target.textContent = text;
    }

    async function load() {
        const view = panel();
        if (!view || view.dataset.loaded === "1") return;
        setStatus(t("about-licenses-loading", "Loading license record..."), false);
        try {
            const [notice, record] = await Promise.all([
                readText(NOTICE_URL),
                readText(RECORD_URL)
            ]);
            fill("gb-licenses-notice", notice);
            fill("gb-licenses-record", record);
            view.dataset.loaded = "1";
            setStatus("", false);
        } catch (error) {
            console.warn("licenses: unable to load the shipped record", error);
            setStatus(t("about-licenses-unavailable", "The license record is not available in this build."), true);
        }
    }

    /** Open the Licenses panel and load the record once. Safe to call twice. */
    window.gbOpenLicenses = function openLicenses() {
        const view = panel();
        if (!view) return Promise.resolve(false);
        view.hidden = false;
        const pending = load();
        const heading = document.getElementById("gb-licenses-heading");
        if (heading) {
            heading.setAttribute("tabindex", "-1");
            heading.focus({ preventScroll: false });
        }
        return pending.then(() => true);
    };

    function fromLocation() {
        let tab = "";
        try {
            tab = new URLSearchParams(window.location.search).get("tab") || "";
        } catch (error) {
            console.warn("licenses: malformed query string", error);
        }
        if (tab.toLowerCase() === "licenses") {
            window.gbOpenLicenses();
        }
    }

    fromLocation();
    window.addEventListener("localeChanged", function () {
        const view = panel();
        if (view && view.dataset.loaded === "1") setStatus("", false);
    });
})();
