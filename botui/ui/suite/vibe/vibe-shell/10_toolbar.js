"use strict";
/**
 * Vibe Shell — toolbar mode command bar (issue #1177).
 * Replaces the ribbon with a slim bar exposing the shared desktop apps
 * (Terminal, Browser, Chat). The desktop window manager API actually
 * available is window.WindowManager (js/window-manager.js): open/close/
 * focus/launchFromMenu/openDeepLink/openIsolated/getApp. There is no
 * DesktopWM.open or openCustom global, so app launches go through
 * WindowManager.launchFromMenu and floating previews are plain WM windows
 * hosting an iframe.
 */
(function () {
    "use strict";

    var S = window.VibeShell;

    function wm() {
        return typeof window.WindowManager !== "undefined" ? window.WindowManager : null;
    }

    function el(tag, cls, text) {
        var node = document.createElement(tag);
        if (cls) node.className = cls;
        if (text != null) node.textContent = text;
        return node;
    }

    function openSharedApp(appId, fallbackUrl, title) {
        var mgr = wm();
        var app = mgr ? mgr.getApp(appId) : null;
        if (mgr && app && typeof mgr.launchFromMenu === "function") {
            mgr.launchFromMenu(appId, app.title || title, app.hxGet || fallbackUrl);
            return;
        }
        openIframeWindow("vibe-shell-" + appId, title, fallbackUrl);
    }

    /* Floating iframe window — the "openCustom" fallback implemented on top
       of the real WindowManager.open primitive. */
    function openIframeWindow(id, title, url) {
        var mgr = wm();
        if (!mgr || typeof mgr.open !== "function") {
            window.open(url, "_blank", "noopener");
            return;
        }
        mgr.open(id, title, "");
        var body = document.getElementById("window-body-" + id);
        if (!body) return;
        body.classList.add("vibe-shell-frame-body");
        var frame = el("iframe");
        frame.className = "vibe-shell-frame";
        frame.src = url;
        frame.title = title;
        body.appendChild(frame);
    }

    function openTerminal() {
        openSharedApp("terminal", "/suite/terminal/terminal.html", "Terminal");
    }

    function resolvePreviewUrl(projectId) {
        if (!projectId) return Promise.reject(new Error("Select a project first"));
        return vibeAuthFetch("/api/vibe/projects/" + encodeURIComponent(projectId))
            .then(function (r) { return r.json(); })
            .then(function (projectData) {
                if (projectData && projectData.success === false) throw new Error(projectData.error || "Project lookup failed");
                var project = projectData && projectData.project;
                var env = (project && (project.environment || project.env)) || "development";
                return vibeAuthFetch("/api/vibe/projects/" + encodeURIComponent(projectId) + "/preview?env=" + encodeURIComponent(env));
            })
            .then(function (r) { return r.json(); })
            .then(function (data) {
                var payload = data && data.data ? data.data : data;
                var url = payload && payload.preview_url;
                if (!url || !/^https?:\/\//i.test(String(url))) throw new Error("No live preview — deploy the project first");
                return String(url);
            });
    }

    function openBrowser() {
        var projectId = S.projectId();
        resolvePreviewUrl(projectId)
            .then(function (url) {
                var name = S.projectName();
                openIframeWindow("vibe-shell-browser", "Browser · " + name, url);
            })
            .catch(function () {
                openSharedApp("browser", "/suite/browser/browser.html", "Browser");
            });
    }

    function openChat() {
        document.dispatchEvent(
            new CustomEvent("gb-open-project-chat", {
                detail: { projectId: S.projectId(), projectName: S.projectName() },
                bubbles: true,
            })
        );
    }

    function toggleMode() {
        var next = S.isToolbar() ? "classic" : "toolbar";
        S.setMode(next, true);
        restartVibeWindow();
    }

    /* A mode switch re-renders the whole app surface; the clean path is to
       close and relaunch the Vibe window through the window manager so the
       partial (and its scripts) execute fresh under the new mode. */
    function restartVibeWindow() {
        var mgr = wm();
        if (mgr && typeof mgr.launchFromMenu === "function") {
            try { mgr.close("vibe"); } catch (ignore) { }
            mgr.launchFromMenu("vibe", "Vibe", "/suite/partials/vibe.html");
            return;
        }
        window.location.reload();
    }

    function buildButton(label, icon, handler, extraCls) {
        var btn = el("button", "vibe-shell-tb-btn" + (extraCls ? " " + extraCls : ""));
        btn.type = "button";
        btn.innerHTML = '<span class="vibe-shell-tb-icon">' + icon + "</span>" +
            '<span class="vibe-shell-tb-label">' + label + "</span>";
        btn.addEventListener("click", handler);
        return btn;
    }

    function build() {
        if (document.getElementById("vibeShellToolbar")) return;
        var container = document.getElementById("vibeWindow");
        if (!container) return;
        var bar = el("div", "vibe-shell-toolbar");
        bar.id = "vibeShellToolbar";
        bar.setAttribute("role", "toolbar");
        bar.setAttribute("aria-label", "Vibe shell commands");

        var apps = el("div", "vibe-shell-tb-group");
        apps.appendChild(buildButton("Terminal", "⌨️", openTerminal));
        apps.appendChild(buildButton("Browser", "🌐", openBrowser));
        apps.appendChild(buildButton("Chat", "💬", openChat));
        bar.appendChild(apps);

        var palettes = el("div", "vibe-shell-tb-group");
        palettes.id = "vibeShellPaletteButtons";
        bar.appendChild(palettes);

        var git = el("div", "vibe-shell-tb-group vibe-shell-tb-git");
        git.id = "vibeShellGitSection";
        bar.appendChild(git);

        var spacer = el("span", "vibe-shell-tb-spacer");
        bar.appendChild(spacer);

        var toggle = buildButton("", "⇄", toggleMode, "vibe-shell-tb-mode");
        toggle.id = "vibeShellModeToggle";
        updateToggleLabel(toggle);
        bar.appendChild(toggle);

        var ribbon = container.querySelector(".vibe-ribbon");
        if (ribbon && ribbon.parentNode) {
            ribbon.parentNode.insertBefore(bar, ribbon);
        } else {
            container.insertBefore(bar, container.firstChild);
        }
    }

    function updateToggleLabel(btn) {
        if (!btn) btn = document.getElementById("vibeShellModeToggle");
        if (!btn) return;
        var label = btn.querySelector(".vibe-shell-tb-label");
        if (label) label.textContent = S.isToolbar() ? "Toolbar ⇄ Classic" : "Classic ⇄ Toolbar";
        btn.title = "Switch shell mode (current: " + S.mode + ")";
    }

    /** Discreet mode-switch chip also available in classic mode. */
    function registerClassicToggle() {
        if (document.getElementById("vibeShellClassicToggle")) return;
        var tabs = document.querySelector("#vibeRibbon .vibe-ribbon-tabs");
        if (!tabs) return;
        var chip = el("button", "vibe-ribbon-tab vibe-shell-classic-toggle");
        chip.id = "vibeShellClassicToggle";
        chip.type = "button";
        chip.textContent = "SHELL ⇄";
        chip.title = "Toggle between classic and toolbar shell modes";
        chip.addEventListener("click", function (e) {
            e.stopPropagation();
            toggleMode();
        });
        tabs.appendChild(chip);
    }

    window.VibeShell.toolbar = {
        build: build,
        openTerminal: openTerminal,
        openBrowser: openBrowser,
        openChat: openChat,
        toggleMode: toggleMode,
        registerClassicToggle: registerClassicToggle,
        refreshToggle: function () { updateToggleLabel(null); },
    };
})();
