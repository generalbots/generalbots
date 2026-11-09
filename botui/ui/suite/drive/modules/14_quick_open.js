/* Drive quick open (#1973).
 *
 * Ctrl/Cmd+Shift+K used to focus the toolbar search box, which is one keystroke away
 * from just clicking it. It now opens a searchable list of recent files across
 * every Drive location (the same union the Recent tab shows), quick-open style:
 * type to filter, arrows to move, Enter to open, Esc to dismiss.
 *
 * The toolbar search stays as it was for full-content search; the list itself
 * is populated from `fetchRecentEntries()` (05_files.js).
 */
(function (window) {
    "use strict";

    var MAX_ROWS = 50;
    var overlay = null;
    var input = null;
    var list = null;
    var status = null;
    var entries = [];
    var rows = [];
    var active = 0;
    var loading = false;

    function ensureUi() {
        if (overlay) return;
        overlay = document.createElement("div");
        overlay.className = "gb-quickopen";
        overlay.setAttribute("role", "dialog");
        overlay.setAttribute("aria-modal", "true");
        overlay.setAttribute("aria-label", "Recent files");
        overlay.hidden = true;
        overlay.innerHTML =
            '<div class="gb-quickopen-panel">' +
            '<input type="text" id="gb-quickopen-input" class="gb-quickopen-input"' +
            ' placeholder="Type to filter recent files…" autocomplete="off"' +
            ' aria-label="Filter recent files" />' +
            '<div class="gb-quickopen-list" id="gb-quickopen-list" role="listbox"></div>' +
            '<p class="gb-quickopen-status" id="gb-quickopen-status"></p>' +
            "</div>";
        document.body.appendChild(overlay);

        input = overlay.querySelector("#gb-quickopen-input");
        list = overlay.querySelector("#gb-quickopen-list");
        status = overlay.querySelector("#gb-quickopen-status");

        input.addEventListener("input", render);
        input.addEventListener("keydown", onKey);
        overlay.addEventListener("mousedown", function (event) {
            if (event.target === overlay) close();
        });
        list.addEventListener("click", function (event) {
            var row = event.target.closest ? event.target.closest(".gb-quickopen-row") : null;
            if (row) choose(Number(row.getAttribute("data-index")));
        });
    }

    function escapeHtml(value) {
        return String(value == null ? "" : value)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;");
    }

    function isOpen() {
        return !!overlay && !overlay.hidden;
    }

    function open() {
        ensureUi();
        overlay.hidden = false;
        input.value = "";
        active = 0;
        input.focus();
        load();
    }

    function close() {
        if (overlay) overlay.hidden = true;
    }

    function toggle() {
        if (isOpen()) close();
        else open();
    }

    async function load() {
        loading = true;
        list.innerHTML = '<p class="gb-quickopen-empty">Loading recent files…</p>';
        if (status) status.textContent = "";
        try {
            entries = window.fetchRecentEntries ? await window.fetchRecentEntries() : [];
        } catch (error) {
            entries = [];
            if (window.console) console.warn("drive: quick open could not list recents", error);
        }
        loading = false;
        render();
    }

    function filtered() {
        var query = input ? input.value.trim().toLowerCase() : "";
        var matches = entries.filter(function (entry) {
            if (entry.is_dir) return false;
            if (!query) return true;
            var name = (entry.name || "").toLowerCase();
            var path = (entry.path || "").toLowerCase();
            return name.indexOf(query) !== -1 || path.indexOf(query) !== -1;
        });
        return matches.slice(0, MAX_ROWS);
    }

    function render() {
        if (!list) return;
        rows = filtered();
        if (active >= rows.length) active = Math.max(rows.length - 1, 0);
        if (!rows.length) {
            list.innerHTML = loading
                ? '<p class="gb-quickopen-empty">Loading recent files…</p>'
                : '<p class="gb-quickopen-empty">No recent files match. Open a file in Drive and it will appear here.</p>';
            if (status) status.textContent = "";
            return;
        }
        list.innerHTML = rows
            .map(function (entry, index) {
                var scope = entry.scope === "user" ? "My files" : "Bot";
                return (
                    '<button type="button" class="gb-quickopen-row' +
                    (index === active ? " active" : "") +
                    '" data-index="' + index + '" role="option" aria-selected="' +
                    (index === active ? "true" : "false") + '">' +
                    '<span class="gb-quickopen-name">' + escapeHtml(entry.name) + "</span>" +
                    '<span class="gb-quickopen-path">' + escapeHtml(entry.path) + "</span>" +
                    '<span class="gb-quickopen-scope">' + scope + "</span>" +
                    "</button>"
                );
            })
            .join("");
        if (status) {
            status.textContent =
                rows.length + (rows.length === 1 ? " recent file" : " recent files") +
                " — arrows to move, Enter to open, Esc to close";
        }
    }

    function move(delta) {
        if (!rows.length) return;
        active = (active + delta + rows.length) % rows.length;
        render();
        var node = list.querySelector('.gb-quickopen-row[data-index="' + active + '"]');
        if (node && node.scrollIntoView) node.scrollIntoView({ block: "nearest" });
    }

    function choose(index) {
        var entry = rows[index];
        if (!entry) return;
        close();
        if (window.DriveModule && typeof window.DriveModule.openFile === "function") {
            window.DriveModule.openFile(entry.path, entry.bucket || undefined);
        } else if (typeof window.openFile === "function") {
            window.openFile(entry.path, entry.bucket || undefined);
        }
    }

    function onKey(event) {
        if (event.key === "ArrowDown") {
            event.preventDefault();
            move(1);
        } else if (event.key === "ArrowUp") {
            event.preventDefault();
            move(-1);
        } else if (event.key === "Enter") {
            event.preventDefault();
            choose(active);
        } else if (event.key === "Escape") {
            event.preventDefault();
            close();
        }
    }

    window.DriveQuickOpen = { open: open, close: close, toggle: toggle, reload: load };
})(window);
