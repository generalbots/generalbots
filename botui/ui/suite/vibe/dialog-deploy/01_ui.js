/* Vibe Deploy dialog — split from vibe/vibe-dialog-deploy.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__deploy = D.__deploy || {};
    var D = window.VibeDialogs;
    M.state = { projects: [], projectId: null, history: [] };

    function sidebar() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-sidebar";

        var head = D.el("div", "vibe-dialog-title");
        head.textContent = "DEPLOYMENTS";
        head.style.padding = "10px";

        var projBar = D.el("div", "vibe-commit-box");
        var sel = D.el("select", "vibe-select");
        sel.id = "vibeDeployProject";
        sel.innerHTML = "<option>no project</option>";
        sel.style.width = "100%";
        sel.addEventListener("change", function () {
            M.state.projectId = sel.value || null;
            M.loadHistory();
        });
        projBar.innerHTML = "<div style='font-size:10px;color:var(--text-muted);margin-bottom:6px'>project</div>";
        projBar.appendChild(sel);

        var list = D.el("div", "vibe-list");
        list.id = "vibeDeployList";
        list.innerHTML = '<div class="vibe-empty">Select a project to see deployments.</div>';

        box.appendChild(head);
        box.appendChild(projBar);
        box.appendChild(list);
        return box;
    }

    function main() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-main";

        var toolbar = D.el("div", "vibe-dialog-toolbar");
        var label = D.el("span", "vibe-status info", "no project");
        label.id = "vibeDeployLabel";
        var spacer = D.el("span");
        spacer.style.flex = "1";
        var probe = D.el("button", "vibe-btn", "🔍 Probe env");
        probe.addEventListener("click", M.probeEnv);
        var preview = D.el("button", "vibe-btn", "🌐 Preview App");
        preview.addEventListener("click", M.previewApp);
        var promote = D.el("button", "vibe-btn", "⬆ Promote test → public");
        promote.title = "Copy the tested release ({slug}-test) to the public site ({slug})";
        promote.addEventListener("click", M.promoteSite);
        var deployBtn = D.el("button", "vibe-btn primary", "🚀 Deploy pipeline");
        deployBtn.addEventListener("click", M.deployPipeline);
        toolbar.appendChild(label);
        toolbar.appendChild(spacer);
        toolbar.appendChild(probe);
        toolbar.appendChild(promote);
        toolbar.appendChild(preview);
        toolbar.appendChild(deployBtn);

        var grid = D.el("div", "vibe-grid");
        grid.id = "vibeDeployMain";
        grid.innerHTML = '<div class="vibe-empty">Pick a project. Deploy runs the intents→build→test→' +
            "publish pipeline (fully automated) on the backend.</div>";

        // ── App Security (access policy per bound domain) ──
        var secBox = D.el("div", "vibe-card");
        secBox.style.margin = "12px 0 0";
        var secHead = D.el("div", "vibe-dialog-title");
        secHead.textContent = "🔐 APP SECURITY";
        secHead.style.padding = "10px";
        var secHint = D.el("div");
        secHint.style.cssText = "font-size:11px;color:var(--text-muted);padding:0 10px 8px;";
        secHint.textContent = "Who can open each domain. Public = anyone · Account = any signed-in user · Allowlist = only listed emails (JWT gate via Caddy).";
        var secBody = D.el("div");
        secBody.id = "vibeSecurityBody";
        secBody.innerHTML = '<div class="vibe-empty">Pick a project to manage its domains.</div>';
        secBox.appendChild(secHead);
        secBox.appendChild(secHint);
        secBox.appendChild(secBody);

        box.appendChild(toolbar);
        box.appendChild(grid);
        box.appendChild(secBox);
        return box;
    }

    M.sidebar = sidebar;
    M.main = main;
})(window.VibeDialogs);
