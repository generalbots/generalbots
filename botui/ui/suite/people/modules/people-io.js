"use strict";

(function () {
    const scope = window.__peopleScope || {};
    const state = scope.state;
    const API_BASE = scope.API_BASE;
    const loadContacts = scope.loadContacts;
    const showContact = scope.showContact;
    const openAddContact = scope.openAddContact;
    const editContact = scope.editContact;
    const closeContactPanel = scope.closeContactPanel;
    const closeModal = scope.closeModal;
    const saveContact = scope.saveContact;
    const deleteContact = scope.deleteContact;

        window.exportCsv = function() {
            const rows = state.contacts.map((c) => [
                c.firstName || "",
                c.lastName || "",
                c.email || "",
                c.phone || "",
                c.company || "",
                c.title || "",
            ]);
            const header = ["first_name", "last_name", "email", "phone", "company", "title"];
            const csv = [header, ...rows]
                .map((r) => r.map((v) => `"${String(v).replace(/"/g, '""')}"`).join(","))
                .join("\n");
            const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = "contacts.csv";
            a.click();
            URL.revokeObjectURL(url);
        }

        window.importCsv = async function(event) {
            const file = event.target.files && event.target.files[0];
            if (!file) return;
            const reader = new FileReader();
            reader.onload = async (e) => {
                const text = String(e.target.result || "");
                const lines = text.split(/\r?\n/).filter((l) => l.trim());
                if (lines.length < 2) return;
                const headers = parseCsvLine(lines[0]).map((h) => h.toLowerCase().trim());
                let imported = 0;
                for (let i = 1; i < lines.length; i++) {
                    const cols = parseCsvLine(lines[i]);
                    const row = {};
                    headers.forEach((h, idx) => {
                        row[h] = cols[idx];
                    });
                    const data = {
                        firstName: row["first name"] || row["firstname"] || row["first_name"] || "",
                        lastName: row["last name"] || row["lastname"] || row["last_name"] || "",
                        email: row["email"] || "",
                        phone: row["phone"] || "",
                        company: row["company"] || "",
                        title: row["title"] || row["job title"] || "",
                    };
                    if (!data.firstName && !data.email) continue;
                    try {
                        await apiFetch("", {
                            method: "POST",
                            body: JSON.stringify(toSnakeBody(data)),
                        });
                        imported++;
                    } catch (err) {
                        console.error("CSV import row failed", err);
                    }
                }
                alert(`Imported ${imported} contacts`);
                await loadContacts();
            };
            reader.readAsText(file);
        }

        window.parseCsvLine = function(line) {
            const out = [];
            let cur = "";
            let inQ = false;
            for (let i = 0; i < line.length; i++) {
                const ch = line[i];
                if (inQ) {
                    if (ch === '"' && line[i + 1] === '"') {
                        cur += '"';
                        i++;
                    } else if (ch === '"') {
                        inQ = false;
                    } else {
                        cur += ch;
                    }
                } else if (ch === '"') {
                    inQ = true;
                } else if (ch === ",") {
                    out.push(cur);
                    cur = "";
                } else {
                    cur += ch;
                }
            }
            out.push(cur);
            return out;
        }

        window.advancedSearch = function() {
            const f = state.filters;
            const q = (document.getElementById("people-search") || {}).value || "";
            f.name = q.toLowerCase();
            loadContacts();
        }

        window.mergeContacts = async function(id1, id2) {
            if (!confirm("Merge these contacts? The first contact will be kept.")) return;
            try {
                await apiFetch(`/${id1}`, {
                    method: "PUT",
                    body: JSON.stringify(toSnakeBody({
                        mergeInto: id2,
                        mergedAt: new Date().toISOString(),
                    })),
                });
                await loadContacts();
            } catch (err) {
                console.error("Merge failed", err);
                alert("Merge failed: " + err.message);
            }
        }

        window.showContact = showContact;
        window.openAddContact = openAddContact;
        window.editContact = editContact;
        window.closeContactPanel = closeContactPanel;
        window.closeModal = closeModal;
        window.saveContact = saveContact;
        window.deleteContact = deleteContact;
        window.startChat = startChat;
        window.sendEmail = sendEmail;
        window.scheduleMeeting = scheduleMeeting;
        window.advancedSearch = advancedSearch;
        window.exportCsv = exportCsv;
        window.importCsv = importCsv;
        window.mergeContacts = mergeContacts;
})();
