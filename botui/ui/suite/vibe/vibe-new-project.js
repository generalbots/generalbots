/**
 * Vibe New-Project dialog (#767) — exactly 3 kinds (Bot / Website / Custom),
 * env tier (small/medium/large), env picker, deploy trigger and hooks editor.
 * Creates the project through the registry API, then raises the dev VM
 * (runner enabled for custom dev), matching the #744 lifecycle.
 */
(function () {
    "use strict";

    var KINDS = [
        { id: "bot", name: "Bot", sub: "Conversational bot (BASIC)", project_type: "bot" },
        { id: "website", name: "Website", sub: "Static site, Caddy-served", project_type: "website" },
        { id: "custom", name: "Custom", sub: "Runtime VM + runner", project_type: "custom" }
    ];
    var TIERS = ["small", "medium", "large"];
    var ENVS = ["development", "staging", "production"];
    var FRAMEWORKS = {
        website: ["htmx", "html", "css"],
        custom: ["node", "python", "htmx", "html"]
    };

    var state = { kind: 0, tier: 0, env: 0 };

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
            '<input type="text" id="vnpName" class="vibe-np-input" placeholder="e.g. autoremAppes-app" value="' + esc(currentProject) + '">' +
            '<label class="vibe-np-label">Kind</label>' +
            '<div class="vibe-np-kinds">' + kindHtml() + "</div>" +
            '<label class="vibe-np-label">Env tier</label>' +
            '<div class="vibe-np-opts">' + optionHtml(TIERS, "tier") + "</div>" +
            '<label class="vibe-np-label">Environment</label>' +
            '<div class="vibe-np-opts">' + optionHtml(ENVS, "env") + "</div>" +
            (FRAMEWORKS[kind.id] ?
                '<label class="vibe-np-label">Framework</label>' +
                '<select id="vnpFramework" class="vibe-np-input">' +
                FRAMEWORKS[kind.id].map(function (f) { return '<option value="' + esc(f) + '">' + esc(f) + "</option>"; }).join("") +
                "</select>"
                : "") +
            '<label class="vibe-np-label">Deploy trigger</label>' +
            '<select id="vnpTrigger" class="vibe-np-input">' +
            '<option value="manual">manual</option>' +
            '<option value="on-commit">on commit</option>' +
            "</select>" +
            '<label class="vibe-np-label">Hooks</label>' +
            '<textarea id="vnpHooks" class="vibe-np-hooks" placeholder="name=url per line&#10;deploy=https://example.com/hooks/deploy"></textarea>' +
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
        ["tier", "env"].forEach(function (key) {
            var inputs = el.querySelectorAll('input[name="vnp-' + key + '"]');
            inputs.forEach(function (input) {
                input.addEventListener("change", function () {
                    state[key] = key === "tier" ? TIERS.indexOf(this.value) : ENVS.indexOf(this.value);
                    render();
                });
            });
        });
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

    function hooksValue() {
        var ta = document.getElementById("vnpHooks");
        var out = {};
        if (!ta) return out;
        ta.value.split("\n").forEach(function (line) {
            var idx = line.indexOf("=");
            if (idx > 0) {
                out[line.slice(0, idx).trim()] = line.slice(idx + 1).trim();
            }
        });
        return out;
    }

    async function submitCreate() {
        var err = document.getElementById("vnpErr");
        var nameInput = document.getElementById("vnpName");
        var name = nameInput ? nameInput.value.trim() : "";
        var kind = KINDS[state.kind];
        var tier = TIERS[state.tier];
        var env = ENVS[state.env];
        if (!name) {
            if (err) err.textContent = "Project name is required.";
            return;
        }
        var frameworkEl = document.getElementById("vnpFramework");
        var triggerEl = document.getElementById("vnpTrigger");
        if (err) err.textContent = "";

        var payload = {
            name: name,
            project_type: kind.project_type,
            environment: env,
            framework: frameworkEl ? frameworkEl.value : null,
            payload: {
                env_tier: tier,
                deploy_trigger: triggerEl ? triggerEl.value : "manual",
                hooks: hooksValue(),
                creator: "vibe-new"
            }
        };
        try {
            var resp = await fetch("/api/vibe/projects", {
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
            if (typeof currentProject !== "undefined") currentProject = name;
            close();
            document.dispatchEvent(new CustomEvent("gb:vibe-project", {
                detail: { project: name, id: project.id }
            }));
            if (typeof vibeAddMsg === "function") {
                vibeAddMsg("system", "Project '" + name + "' (" + kind.name + ", " + tier + ") created — dev VM raised.");
            }
        } catch (e) {
            if (err) err.textContent = "Create failed: " + e.message;
        }
    }

    async function raiseDevVm(project, env, kindId, tier) {
        var payload = {
            env: env,
            tier: tier,
            runner_enabled: kindId === "custom" && env === "development"
        };
        try {
            await fetch("/api/vibe/projects/" + project.id + "/vms", {
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
    }

    function open() {
        var el = document.getElementById("vibeNewProjectModal");
        if (!el) return;
        el.style.display = "flex";
        render();
    }

    window.VibeNewProject = { open: open, close: close, submit: submitCreate };
})();