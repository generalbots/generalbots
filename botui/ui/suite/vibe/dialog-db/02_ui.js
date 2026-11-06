/* Vibe Schema dialog — split from vibe/vibe-dialog-db.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__db;
    var D = window.VibeDialogs;
    function sidebar() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-sidebar";

        // #1386 — environment selector (production database vs `_dev` twin).
        var envWrap = D.el("div", "vibe-sql-bar");
        envWrap.style.padding = "6px";
        var envSel = D.el("select", "vibe-input");
        envSel.id = "vibeDbEnvSel";
        envSel.style.width = "100%";
        envSel.innerHTML =
            '<option value="production">🏭 production</option>' +
            '<option value="dev">🧪 dev (_dev twin)</option>';
        // #1448 — persist the environment choice so a reopen keeps the twin
        // the user was working in, and the active database is obvious from
        // the table label ("dev database (_dev twin)" vs "production database").
        try {
            var savedEnv = localStorage.getItem("gb-vibe-db-env");
            if (savedEnv === "dev" || savedEnv === "production") {
                M.state.env = savedEnv;
                envSel.value = savedEnv;
            }
        } catch (e) { }
        envSel.addEventListener("change", function () {
            M.state.env = envSel.value;
            M.state.table = null;
            M.state.page = 1;
            try { localStorage.setItem("gb-vibe-db-env", M.state.env); } catch (e) { }
            M.refreshCtxChip();
            var label = document.getElementById("vibeDbTableLabel");
            if (label) label.textContent = M.state.env === "dev" ? "dev database (_dev twin)" : "production database";
            M.loadSchema(false);
        });
        envWrap.appendChild(envSel);

        var list = D.el("div", "vibe-list");
        list.id = "vibeDbTableList";
        list.innerHTML = '<div class="vibe-empty">Loading schema...</div>';

        var foot = D.el("div", "vibe-sql-bar");
        var sql = D.el("textarea", "vibe-textarea");
        sql.id = "vibeDbSql";
        sql.placeholder = "SELECT * FROM table ...";
        sql.rows = 3;
        var run = D.el("button", "vibe-btn primary", "Run");
        run.addEventListener("click", M.runQuery);
        foot.appendChild(sql);
        foot.appendChild(run);

        box.appendChild(envWrap);
        box.appendChild(list);
        box.appendChild(foot);
        return box;
    }

    function main() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-main";

        var toolbar = D.el("div", "vibe-dialog-toolbar");
        var label = D.el("span", "vibe-status info", "no table selected");
        label.id = "vibeDbTableLabel";
        // #1386b — persistent context chip: the connected database is always
        // visible, so grid edits can never be mistaken for another project's.
        var ctx = D.el("span", "vibe-status info");
        ctx.id = "vibeDbCtx";
        ctx.style.marginLeft = "10px";
        ctx.textContent = "⛔ no project selected";
        var refresh = D.el("button", "vibe-btn", "↻ Refresh");
        refresh.addEventListener("click", function () { M.loadSchema(true); });
        var insert = D.el("button", "vibe-btn", "➕ Insert Row");
        insert.addEventListener("click", M.insertRow);
        var spacer = D.el("span");
        spacer.style.flex = "1";
        var exportBtn = D.el("button", "vibe-btn", "Export CSV");
        exportBtn.addEventListener("click", M.exportCsv);
        toolbar.appendChild(label);
        toolbar.appendChild(ctx);
        toolbar.appendChild(spacer);
        toolbar.appendChild(insert);
        toolbar.appendChild(refresh);
        toolbar.appendChild(exportBtn);

        var gridWrap = D.el("div", "vibe-grid");
        gridWrap.id = "vibeDbGrid";
        gridWrap.innerHTML = '<div class="vibe-empty">Select a table from the schema list.</div>';

        var pager = D.el("div", "vibe-browser-status");
        pager.id = "vibeDbPager";
        pager.innerHTML = '<span id="vibeDbRowCount">—</span><span style="flex:1"></span>' +
            '<button class="vibe-btn" id="vibeDbPrev">◀ Prev</button>' +
            '<span id="vibeDbPageInfo">Page 1</span>' +
            '<button class="vibe-btn" id="vibeDbNext">Next ▶</button>';

        box.appendChild(toolbar);
        box.appendChild(gridWrap);
        box.appendChild(pager);
        return box;
    }

    M.sidebar = sidebar;
    M.main = main;
})(window.VibeDialogs);
