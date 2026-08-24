/* Memory app — part 2: import/export, report modal, wiring */
"use strict";

function memCloseModals() {
    document.querySelectorAll(".mem-modal.open").forEach(m => m.classList.remove("open"));
}

async function exportMemories() {
    try {
        const data = await memApi("/api/memory/export");
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.href = url;
        link.download = "memories-export-" + new Date().toISOString().slice(0, 10) + ".json";
        document.body.appendChild(link);
        link.click();
        link.remove();
        URL.revokeObjectURL(url);
    } catch (err) {
        memSetState("Export failed: " + err.message, true);
    }
}

function memNormalizeImportItems(parsed) {
    if (Array.isArray(parsed)) return parsed;
    if (parsed && Array.isArray(parsed.items)) return parsed.items;
    if (parsed && Array.isArray(parsed.memories)) return parsed.memories;
    return null;
}

function renderImportReport(report, isDryRun) {
    const box = document.getElementById("mem-import-report");
    if (!box) return;
    box.classList.remove("mem-report-error");
    const created = report.created != null ? report.created : (report.inserted || 0);
    const updated = report.updated != null ? report.updated : 0;
    const skipped = report.skipped != null ? report.skipped : 0;
    const errors = Array.isArray(report.errors) ? report.errors : [];
    let html = "<strong>" + (isDryRun ? "Dry run complete — nothing was written." : "Import complete.") + "</strong>";
    html += '<ul><li>Created: ' + created + "</li><li>Updated: " + updated +
        "</li><li>Skipped: " + skipped + "</li></ul>";
    if (errors.length) {
        html += '<div class="mem-report-error"><strong>Errors (' + errors.length + "):</strong><ul>" +
            errors.slice(0, 10).map(e => "<li>" + memEsc(typeof e === "string" ? e : JSON.stringify(e)) + "</li>").join("") +
            "</ul></div>";
    }
    box.innerHTML = html;
    box.classList.remove("hidden");
}

let memLastImportItems = null;

async function runImport(dryRun) {
    const fileInput = document.getElementById("mem-import-file");
    const reportBox = document.getElementById("mem-import-report");
    const btn = document.getElementById("mem-import-run");
    if (!reportBox || !btn) return;
    if (dryRun || !memLastImportItems) {
        if (!fileInput.files || !fileInput.files[0]) {
            reportBox.textContent = "Select a JSON file first.";
            reportBox.classList.remove("hidden");
            return;
        }
        try {
            const text = await fileInput.files[0].text();
            const parsed = JSON.parse(text);
            const items = memNormalizeImportItems(parsed);
            if (!items) throw new Error("Expected a JSON array or an object with an items array.");
            memLastImportItems = items;
        } catch (err) {
            reportBox.textContent = "Invalid file: " + err.message;
            reportBox.classList.add("mem-report-error");
            reportBox.classList.remove("hidden");
            return;
        }
    }
    btn.disabled = true;
    try {
        const data = await memApi("/api/memory/import", {
            method: "POST",
            body: JSON.stringify({ dry_run: dryRun, items: memLastImportItems })
        });
        const report = data.report || data.result || data;
        renderImportReport(report, dryRun);
        if (!dryRun) {
            memLastImportItems = null;
            await loadItems();
        }
    } catch (err) {
        reportBox.textContent = "Import failed: " + err.message;
        reportBox.classList.add("mem-report-error");
        reportBox.classList.remove("hidden");
    } finally {
        btn.disabled = false;
    }
}

document.addEventListener("click", function (event) {
    const target = event.target.closest("[data-mem-close],[data-mem-pin],[data-mem-edit],[data-mem-delete],[data-mem-kind]");
    if (!target) return;
    if (target.hasAttribute("data-mem-close")) { memCloseModals(); return; }
    if (target.hasAttribute("data-mem-pin")) { togglePin(target.getAttribute("data-mem-pin")); return; }
    if (target.hasAttribute("data-mem-edit")) { startInlineEdit(target.getAttribute("data-mem-edit")); return; }
    if (target.hasAttribute("data-mem-delete")) { deleteItem(target.getAttribute("data-mem-delete")); return; }
    if (target.hasAttribute("data-mem-kind")) {
        MemState.kindFilter = target.getAttribute("data-mem-kind");
        renderGrid();
        return;
    }
});

(function initMemoryApp() {
    const root = document.getElementById("memory-app");
    if (!root) return;
    document.getElementById("mem-add").addEventListener("click", openAddModal);
    document.getElementById("mem-save-item").addEventListener("click", saveNewItem);
    document.getElementById("mem-export").addEventListener("click", exportMemories);
    document.getElementById("mem-import").addEventListener("click", function () {
        document.getElementById("mem-import-report").classList.add("hidden");
        document.getElementById("mem-import-modal").classList.add("open");
    });
    document.getElementById("mem-import-run").addEventListener("click", function () {
        runImport(document.getElementById("mem-dry-run").checked);
    });
    const search = document.getElementById("mem-search");
    search.addEventListener("input", function () {
        clearTimeout(MemState.searchTimer);
        MemState.searchTimer = setTimeout(function () {
            MemState.query = search.value.trim();
            loadItems();
        }, 350);
    });
    loadItems();
})();
