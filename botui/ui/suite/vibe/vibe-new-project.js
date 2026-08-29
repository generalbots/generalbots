/**
 * Vibe New-Project dialog (#767) — exactly 3 kinds (Bot / Website / Custom)
 * and an env tier (small/medium/large). Environment, deploy trigger and hooks
 * were removed (2026-08-27): a project is created in development (Run tests
 * dev) and published to production via the toolbar Deploy button — there is
 * no staging and no external hooks.
 */
(function () {
    "use strict";

    var KINDS = [
        { id: "bot", name: "Bot", sub: "Conversational bot (BASIC)", project_type: "bot" },
        { id: "website", name: "Website", sub: "Static site, Caddy-served", project_type: "website" },
        { id: "custom", name: "Custom", sub: "Runtime VM + runner", project_type: "custom" }
    ];
    var TIERS = ["small", "medium", "large"];
    var FRAMEWORKS = {
        // Web is htmx-only: the website template ships as an htmx page, so the
        // framework picker offers nothing else (html/css are htmx's building
        // blocks, not standalone templates).
        website: ["htmx"],
        custom: ["node", "python", "htmx", "html"]
    };

    // name kept in state so Kind/Tier re-renders do not discard the user's
    // typed value (fixes #821).
    var state = { kind: 0, tier: 0, name: "" };

    // A project is created in development; production is reached only via the
    // toolbar Deploy button (pipeline_mode=deploy). No staging, no env picker.
    var CREATE_ENV = "development";

    function esc(s) {
        var d = document.createElement("div");
        d.textContent = s == null ? "" : String(s);
        return d.innerHTML;
    }

    function kindHtml() {
        return KINDS.map(function (k, i) {
            return '<button type="button" class="vibe-np-kind"' +
                (i === state.kind ? ' data-active="1"' : "") +
                ' data-kind="' + i + '">' +
                '<span class="vibe-np-kind-name">' + esc(k.name) + "</span>" +
                '<span class="vibe-np-kind-sub">' + esc(k.sub) + "</span></button>";
        }).join("");
    }

    function optionHtml(items, key) {
        return items.map(function (item, i) {
            return '<label class="vibe-np-opt' + (i === state[key] ? " active" : "") + '">' +
                '<input type="radio" name="vnp-' + key + '" value="' + item + '"' +
                (i === state[key] ? " checked" : "") + "> " + item +
                "</label>";
        }).join("");
    }

    function render() {
        var el = document.getElementById("vibeNewProjectModal");
        if (!el) return;
        var kind = KINDS[state.kind];
        el.innerHTML =
            '<div class="vibe-np-overlay">' +
            '<div class="vibe-np-dialog">' +
            '<div class="vibe-np-head"><span>New Project</span>' +
            '<button type="button" class="vibe-np-close" onclick="window.VibeNewProject.close()">&times;</button></div>' +
            '<div class="vibe-np-body">' +
            '<label class="vibe-np-label">Name</label>' +
            '<input type="text" id="vnpName" class="vibe-np-input" placeholder="e.g. my-project" value="' + esc(state.name) + '">' +
            '<label class="vibe-np-label">Kind</label>' +
            '<div class="vibe-np-kinds">' + kindHtml() + "</div>" +
            '<label class="vibe-np-label">Env tier</label>' +
            '<div class="vibe-np-opts">' + optionHtml(TIERS, "tier") + "</div>" +
            (FRAMEWORKS[kind.id] ?
                '<label class="vibe-np-label">Framework</label>' +
                '<select id="vnpFramework" class="vibe-np-input">' +
                FRAMEWORKS[kind.id].map(function (f) { return '<option value="' + esc(f) + '">' + esc(f) + "</option>"; }).join("") +
                "</select>"
                : "") +
            '<div class="vibe-np-hint">Created in <b>development</b> — Run tests it here; the <b>Deploy</b> button publishes to production.</div>' +
            '<div class="vibe-np-err" id="vnpErr"></div>' +
            "</div>" +
            '<div class="vibe-np-foot">' +
            '<button type="button" class="vibe-np-cancel" onclick="window.VibeNewProject.close()">Cancel</button>' +
            '<button type="button" class="vibe-np-create" id="vnpCreateBtn">Create project</button>' +
            "</div></div></div>";

        var kinds = el.querySelectorAll(".vibe-np-kind");
        kinds.forEach(function (btn) {
            btn.addEventListener("click", function () {
                state.kind = parseInt(this.dataset.kind, 10);
                render();
            });
        });
        var tierInputs = el.querySelectorAll('input[name="vnp-tier"]');
        tierInputs.forEach(function (input) {
            input.addEventListener("change", function () {
                state.tier = TIERS.indexOf(this.value);
                render();
            });
        });
        // Persist the typed name in state so re-renders (Kind/Tier/Env
        // changes) do not discard it (fixes #821).
        var nameInput = document.getElementById("vnpName");
        if (nameInput) {
            nameInput.addEventListener("input", function () {
                state.name = this.value;
            });
        }
        renderKindActive();
        var create = document.getElementById("vnpCreateBtn");
        if (create) create.addEventListener("click", submitCreate);
    }

    function renderKindActive() {
        var kinds = document.querySelectorAll(".vibe-np-kind");
        kinds.forEach(function (btn, i) {
            btn.classList.toggle("active", i === state.kind);
        });
    }

    async function submitCreate() {
        var err = document.getElementById("vnpErr");
        var nameInput = document.getElementById("vnpName");
        var name = nameInput ? nameInput.value.trim() : "";
        if (name) state.name = name;
        var kind = KINDS[state.kind];
        var tier = TIERS[state.tier];
        var env = CREATE_ENV;
        if (!name) {
            if (err) err.textContent = "Project name is required.";
            return;
        }
        var frameworkEl = document.getElementById("vnpFramework");
        if (err) err.textContent = "";

        var payload = {
            name: name,
            project_type: kind.project_type,
            environment: env,
            framework: frameworkEl ? frameworkEl.value : null,
            payload: {
                env_tier: tier,
                creator: "vibe-new"
            }
        };
        try {
            var resp = await vibeAuthFetch("/api/vibe/projects", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
            var data = await resp.json();
            if (!resp.ok || !data.success) {
                if (err) err.textContent = data.error || ("HTTP " + resp.status);
                return;
            }
            var project = data.project;
            await raiseDevVm(project, env, kind.id, tier);
            // Seed project.draw at the ROOT of the project workspace so the
            // Canvas app opens a ready architecture diagram on first click
            // (no generation round-trip needed). The canvas app reads and
            // edits exactly this file (project root /project.draw).
            seedProjectDraw(project.id, name, kind.project_type || kind.id);
            if (typeof currentProject !== "undefined") currentProject = name;
            if (typeof currentProjectId !== "undefined") currentProjectId = project.id;
            state.name = "";
            close();
            document.dispatchEvent(new CustomEvent("gb:vibe-project", {
                detail: { project: name, id: project.id }
            }));
            document.dispatchEvent(new CustomEvent("gb:vibe-project-created", {
                detail: { project: name, id: project.id }
            }));
            if (typeof vibeAddMsg === "function") {
                vibeAddMsg("system", "Project '" + name + "' (" + kind.name + ", " + tier + ") created — dev VM raised.");
            }
        } catch (e) {
            if (err) err.textContent = "Create failed: " + e.message;
        }
    }

    // Seed a baseline project.draw (vibe-design v2 format, matching the
    // canvas app's loader) into the project workspace root at creation so
    // the Canvas button opens a rendered architecture immediately.
    function seedProjectDraw(pid, name, projectType) {
        var draw = {
            kind: "vibe-design",
            version: 2,
            elements: [],
            connectors: [],
            generated: {
                version: 1,
                generatedAt: new Date().toISOString(),
                source: "project-creation",
                title: name,
                svg: buildSeedSvg(name, projectType)
            }
        };
        vibeAuthFetch("/api/vibe/projects/" + encodeURIComponent(pid) + "/files", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ path: "project.draw", content: JSON.stringify(draw) })
        }).catch(function (e) {
            console.warn("seed project.draw failed (canvas will generate on open):", e);
        });
    }

    // Minimal green/blue zone diagram matching the canvas app's "generated"
    // field shape — the canvas renders it as the starting architecture.
    function buildSeedSvg(name, projectType) {
        var title = String(name || "project");
        var kind = String(projectType || "website");
        function esc(s) { return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;"); }
        return "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 920 520\" aria-label=\"Generated project design\">" +
            "<defs><linearGradient id=\"vbSeedGrad\" x1=\"0\" x2=\"1\"><stop stop-color=\"#dbeafe\"/><stop offset=\"1\" stop-color=\"#dcfce7\"/></linearGradient></defs>" +
            "<rect x=\"36\" y=\"40\" width=\"848\" height=\"440\" rx=\"18\" fill=\"url(#vbSeedGrad)\" stroke=\"#93c5fd\" stroke-width=\"2\"/>" +
            "<rect x=\"76\" y=\"88\" width=\"230\" height=\"320\" rx=\"12\" fill=\"#fff\" stroke=\"#60a5fa\"/>" +
            "<text x=\"142\" y=\"134\" font-family=\"system-ui\" font-size=\"18\" font-weight=\"700\" fill=\"#0f172a\">" + esc(title) + "</text>" +
            "<text x=\"100\" y=\"184\" font-family=\"system-ui\" font-size=\"13\" fill=\"#475569\">" + esc(kind) + " application</text>" +
            "<rect x=\"336\" y=\"88\" width=\"508\" height=\"74\" rx=\"12\" fill=\"#fff\" stroke=\"#86efac\"/>" +
            "<text x=\"356\" y=\"118\" font-family=\"system-ui\" font-size=\"14\" font-weight=\"700\" fill=\"#15803d\">Frontend</text>" +
            "<rect x=\"336\" y=\"188\" width=\"242\" height=\"220\" rx=\"12\" fill=\"#fff\" stroke=\"#c4b5fd\"/>" +
            "<text x=\"356\" y=\"218\" font-family=\"system-ui\" font-size=\"14\" font-weight=\"700\" fill=\"#7c3aed\">Backend</text>" +
            "<rect x=\"602\" y=\"188\" width=\"242\" height=\"220\" rx=\"12\" fill=\"#fff\" stroke=\"#fdba74\"/>" +
            "<text x=\"622\" y=\"218\" font-family=\"system-ui\" font-size=\"14\" font-weight=\"700\" fill=\"#c2410c\">Data</text>" +
            "<path d=\"M306 248H336M578 298H602\" stroke=\"#64748b\" stroke-width=\"3\" stroke-dasharray=\"7 7\"/>" +
            "<text x=\"100\" y=\"360\" font-family=\"system-ui\" font-size=\"12\" fill=\"#64748b\">Architecture seeded at project creation</text>" +
            "</svg>";
    }

    async function raiseDevVm(project, env, kindId, tier) {
        var payload = {
            env: env,
            tier: tier,
            runner_enabled: kindId === "custom" && env === "development"
        };
        try {
            await vibeAuthFetch("/api/vibe/projects/" + project.id + "/vms", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });
        } catch (e) {
            console.warn("dev VM raise failed:", e);
        }
    }

    function close() {
        var el = document.getElementById("vibeNewProjectModal");
        if (el) el.style.display = "none";
        var wm = window.WindowManager;
        if (wm && wm.getWindow("vibe-newproject")) wm.close("vibe-newproject");
    }

    function open() {
        var el = document.getElementById("vibeNewProjectModal");
        if (!el) return;
        // Floating tool window (VB6-style); falls back to in-window display
        // when the desktop shell is absent (isolated run).
        if (window.VibeWindows) window.VibeWindows.openNewProject();
        var wm = window.WindowManager;
        if (wm && !/[?&]isolated=1/.test(window.location.search)) {
            el.style.display = "flex";
            render();
            wm.focusWindow("vibe-newproject");
            return;
        }
        el.style.display = "flex";
        render();
    }

    window.VibeNewProject = { open: open, close: close, submit: submitCreate };
})();