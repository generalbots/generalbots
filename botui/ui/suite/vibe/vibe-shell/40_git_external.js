"use strict";
/**
 * Vibe Shell — external git controls (issue #1177).
 * Mounted into the toolbar and reuses the endpoints the Source Control
 * dialog already consumes (botgit crate routes):
 *   GET  /api/git/status?repo=        · GET /api/git/branches?repo=
 *   POST /api/git/commit               · POST /api/git/push?repo=
 *   POST /api/git/branch/:name
 * There is no pull-request endpoint registered on the backend, so the
 * Push & PR button performs the push and renders response.pr_url only
 * when the backend provides it. Export stays disabled: no export or
 * download endpoint exists for Vibe projects (verified in
 * endpoint_inventory.rs).
 */
(function () {
    "use strict";

    var S = window.VibeShell;

    function el(tag, cls, text) {
        var node = document.createElement(tag);
        if (cls) node.className = cls;
        if (text != null) node.textContent = text;
        return node;
    }

    function repo() {
        return encodeURIComponent(S.projectName());
    }

    function api(path, options) {
        return vibeAuthFetch(path, options).then(function (resp) {
            return resp.json().catch(function () {
                return { status: "failure", error: "HTTP " + resp.status };
            });
        });
    }

    function setStatus(text, cls) {
        var out = document.getElementById("vibeShellGitStatus");
        if (!out) return;
        out.textContent = text;
        out.className = "vibe-shell-git-status" + (cls ? " " + cls : "");
    }

    function showPrCard(url) {
        var host = document.getElementById("vibeShellPrCard");
        if (!host) return;
        if (!url) { host.innerHTML = ""; return; }
        var link = document.createElement("a");
        link.href = url;
        link.target = "_blank";
        link.rel = "noopener";
        link.className = "vibe-shell-pr-link";
        link.textContent = url;
        host.innerHTML = "";
        var card = el("div", "vibe-shell-pr-card");
        card.appendChild(el("span", null, "PR: "));
        card.appendChild(link);
        host.appendChild(card);
    }

    function loadBranches() {
        var input = document.getElementById("vibeShellBranchInput");
        if (!input) return;
        api("/api/git/branches?repo=" + repo()).then(function (data) {
            var branches = (data && data.branches) || [];
            var list = document.getElementById("vibeShellBranchList");
            if (!list || !Array.isArray(branches)) return;
            list.innerHTML = "";
            branches.forEach(function (b) {
                var name = typeof b === "string" ? b : (b.name || "");
                if (!name) return;
                var opt = document.createElement("option");
                opt.value = name;
                list.appendChild(opt);
            });
            var current = branches.filter(function (b) { return typeof b === "object" && b.current; })[0];
            if (current && current.name && !input.value) input.value = current.name;
        }).catch(function () { });
    }

    function useBranch() {
        var input = document.getElementById("vibeShellBranchInput");
        if (!input || !input.value.trim()) { setStatus("Enter a branch name first.", "err"); return; }
        var name = input.value.trim();
        setStatus("Switching to " + name + "…");
        api("/api/git/branch/" + encodeURIComponent(name) + "?repo=" + repo(), { method: "POST" })
            .then(function (data) {
                if (data && data.status === "failure") throw new Error(data.error || "branch failed");
                setStatus("On branch " + name + ".", "ok");
                loadBranches();
            })
            .catch(function (err) { setStatus("Branch error: " + err.message, "err"); });
    }

    function commit() {
        var msg = document.getElementById("vibeShellCommitMsg");
        if (!msg || !msg.value.trim()) { setStatus("Enter a commit message first.", "err"); return; }
        setStatus("Committing…");
        api("/api/git/commit", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ message: msg.value.trim(), repo: S.projectName() }),
        }).then(function (data) {
            if (data && data.success) {
                msg.value = "";
                setStatus("Committed.", "ok");
            } else {
                setStatus("Commit failed: " + ((data && data.error) || "unknown"), "err");
            }
        }).catch(function () { setStatus("Commit request failed.", "err"); });
    }

    function pushAndPr() {
        setStatus("Pushing…");
        showPrCard(null);
        api("/api/git/push?repo=" + repo(), { method: "POST" })
            .then(function (data) {
                if (data && data.status === "failure") {
                    throw new Error(data.error || "push rejected");
                }
                if (data && data.pr_url) {
                    showPrCard(data.pr_url);
                    setStatus("Pushed — pull request ready.", "ok");
                } else {
                    /* No PR endpoint is registered backend-side; report the
                       push result without inventing a PR. */
                    setStatus(data && data.message ? String(data.message) : "Pushed.", "ok");
                }
            })
            .catch(function (err) { setStatus("Push failed: " + err.message, "err"); });
    }

    function buildButton(label, handler, extraCls, disabledTitle) {
        var btn = el("button", "vibe-btn vibe-shell-git-btn" + (extraCls ? " " + extraCls : ""), label);
        btn.type = "button";
        if (disabledTitle) {
            btn.disabled = true;
            btn.title = disabledTitle;
        } else {
            btn.addEventListener("click", handler);
        }
        return btn;
    }

    function mount() {
        var host = document.getElementById("vibeShellGitSection");
        if (!host || host.childElementCount) return;

        var branch = el("input", "vibe-shell-git-input");
        branch.id = "vibeShellBranchInput";
        branch.type = "text";
        branch.placeholder = "branch";
        branch.setAttribute("list", "vibeShellBranchList");
        var dataList = el("datalist");
        dataList.id = "vibeShellBranchList";
        host.appendChild(branch);
        host.appendChild(dataList);
        host.appendChild(buildButton("Use branch", useBranch));

        var msg = el("input", "vibe-shell-git-input vibe-shell-git-msg");
        msg.id = "vibeShellCommitMsg";
        msg.type = "text";
        msg.placeholder = "commit message";
        msg.addEventListener("keydown", function (e) {
            if (e.key === "Enter") commit();
        });
        host.appendChild(msg);
        host.appendChild(buildButton("Commit", commit));

        host.appendChild(buildButton("Push & PR", pushAndPr, "primary"));
        host.appendChild(buildButton("Export Project", null, null,
            "No export endpoint is registered for Vibe projects."));

        var status = el("span", "vibe-shell-git-status");
        status.id = "vibeShellGitStatus";
        host.appendChild(status);

        var prHost = el("span");
        prHost.id = "vibeShellPrCard";
        host.appendChild(prHost);

        loadBranches();
    }

    window.VibeShell.git = { mount: mount };
})();
