(function() {
'use strict';
if (window.GBAppLifecycle) GBAppLifecycle.begin("lists");
function escapeHtml(str) {
    if (str == null) return '';
    if (window.GBShared && window.GBShared.html && window.GBShared.html.escape) return window.GBShared.html.escape(str);
    return String(str).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;').replace(/'/g,'&#39;');
}
function csvCell(v) {
    var s = String(v == null ? '' : v);
    if (/^[=+\-@\t\r]/.test(s)) s = "'" + s;
    if (/[",\n\r]/.test(s)) return '"' + s.replace(/"/g,'""') + '"';
    return '"' + s.replace(/"/g,'""') + '"';
}
let allLists = [];
let currentFilter = 'all';
let editingListId = null;
let membersListId = null;

function showListModal(listId) {
    const modal = document.getElementById('list-modal');
    const title = document.getElementById('list-modal-title');
    const submit = document.getElementById('list-form-submit');
    editingListId = listId || null;

    if (listId) {
        title.textContent = 'Edit List';
        submit.textContent = 'Save Changes';
        const list = allLists.find(l => l.id === listId);
        if (list) {
            document.getElementById('list-form-id').value = list.id;
            document.getElementById('list-form-name').value = list.name;
            document.getElementById('list-form-type').value = list.list_type || 'static';
            document.getElementById('list-form-query').value = list.query_text || '';
        }
    } else {
        title.textContent = 'Create List';
        submit.textContent = 'Create List';
        document.getElementById('list-form').reset();
        document.getElementById('list-form-id').value = '';
    }
    modal.style.display = 'flex';
}

function hideListModal() {
    document.getElementById('list-modal').style.display = 'none';
    editingListId = null;
}

function showMembersModal(listId) {
    const modal = document.getElementById('members-modal');
    const list = allLists.find(l => l.id === listId);
    membersListId = listId || null;
    document.getElementById('members-modal-title').textContent = (list ? list.name : 'List') + ' Members';
    modal.style.display = 'flex';
    loadMembers(listId);
}

function hideMembersModal() { membersListId = null; document.getElementById('members-modal').style.display = 'none'; }

function showDeleteModal(listId) {
    const list = allLists.find(l => l.id === listId);
    document.getElementById('delete-list-name').textContent = list ? list.name : '';
    document.getElementById('delete-modal').style.display = 'flex';
    document.getElementById('btn-confirm-delete').onclick = function() { deleteList(listId); };
}

function hideDeleteModal() { document.getElementById('delete-modal').style.display = 'none'; }

function showImportModal() {
    const sel = document.getElementById('import-target-list');
    sel.innerHTML = allLists.map(l => '<option value="' + escapeHtml(String(l.id || '')) + '">' + escapeHtml(l.name || '') + '</option>').join('');
    document.getElementById('import-modal').style.display = 'flex';
}

function hideImportModal() { document.getElementById('import-modal').style.display = 'none'; }

function filterLists(view, btn) {
    currentFilter = view;
    document.querySelectorAll('.crm-tab').forEach(t => t.classList.remove('active'));
    if (btn) btn.classList.add('active');
    renderLists();
}

function searchLists(q) {
    renderLists(q.toLowerCase());
}

function renderLists(searchQuery) {
    const grid = document.getElementById('listsList');
    let filtered = allLists;
    if (currentFilter !== 'all') {
        filtered = filtered.filter(l => l.list_type === currentFilter);
    }
    if (searchQuery) {
        filtered = filtered.filter(l => l.name.toLowerCase().includes(searchQuery));
    }
    if (filtered.length === 0) {
        grid.innerHTML = '<div style="grid-column:1/-1;padding:40px;text-align:center;color:var(--text-secondary)">No lists found.</div>';
        return;
    }
    grid.innerHTML = filtered.map(list => {
        const id = String(list.id || '');
        const name = list.name || 'Unnamed';
        const type = list.list_type || 'static';
        return '<div class="list-card" data-action="open" data-id="' + escapeHtml(id) + '">'
        + '<div class="list-card-header">'
        + '<h3 class="list-card-title">' + escapeHtml(name) + '</h3>'
        + '<span class="list-type ' + escapeHtml(type) + '">' + escapeHtml(type) + '</span>'
        + '</div>'
        + '<div class="list-stats">'
        + '<div class="list-stat"><span class="list-stat-value">' + escapeHtml(String(list.member_count || 0)) + '</span><span class="list-stat-label">Members</span></div>'
        + '</div>'
        + '<div class="list-card-actions" data-stop="1">'
        + '<button type="button" data-action="edit" data-id="' + escapeHtml(id) + '">Edit</button>'
        + '<button type="button" data-action="export" data-id="' + escapeHtml(id) + '">Export</button>'
        + '<button type="button" class="danger" data-action="delete" data-id="' + escapeHtml(id) + '">Delete</button>'
        + '</div>'
        + '</div>';
    }).join('');
    grid.querySelectorAll('[data-action="open"]').forEach(function(card){
        card.addEventListener('click', function(){ showMembersModal(this.getAttribute('data-id')); });
    });
    grid.querySelectorAll('[data-action="edit"]').forEach(function(btn){
        btn.addEventListener('click', function(e){ e.stopPropagation(); showListModal(this.getAttribute('data-id')); });
    });
    grid.querySelectorAll('[data-action="export"]').forEach(function(btn){
        btn.addEventListener('click', function(e){ e.stopPropagation(); exportSingleList(this.getAttribute('data-id')); });
    });
    grid.querySelectorAll('[data-action="delete"]').forEach(function(btn){
        btn.addEventListener('click', function(e){ e.stopPropagation(); showDeleteModal(this.getAttribute('data-id')); });
    });
}

async function loadLists() {
    const grid = document.getElementById('listsList');
    try {
        const resp = await fetch('/api/crm/lists');
        if (resp.ok) {
            allLists = await resp.json();
            if (!Array.isArray(allLists)) allLists = [];
            renderLists();
        } else {
            if (grid) grid.innerHTML = '<div style="grid-column:1/-1;padding:40px;text-align:center;color:var(--id-danger-icon)">' + (typeof window.gbT === "function" ? window.gbT("lists-load-failed", "Failed to load lists.") : "Failed to load lists.") + ' <button type="button" id="lists-retry-btn" style="margin-left:8px;">' + (typeof window.gbT === "function" ? window.gbT("action-retry", "Retry") : "Retry") + '</button></div>';
            var btn = document.getElementById('lists-retry-btn');
            if (btn) btn.addEventListener('click', loadLists);
        }
    } catch (e) {
        console.error('loadLists:', e);
        if (grid) grid.innerHTML = '<div style="grid-column:1/-1;padding:40px;text-align:center;color:var(--id-danger-icon)">' + (typeof window.gbT === "function" ? window.gbT("lists-load-failed", "Failed to load lists.") : "Failed to load lists.") + ' <button type="button" id="lists-retry-btn" style="margin-left:8px;">' + (typeof window.gbT === "function" ? window.gbT("action-retry", "Retry") : "Retry") + '</button></div>';
        var btn2 = document.getElementById('lists-retry-btn');
        if (btn2) btn2.addEventListener('click', loadLists);
    }
}

async function loadMembers(listId) {
    const content = document.getElementById('members-content');
    try {
        const resp = await fetch('/api/crm/lists/' + listId + '/members');
        if (resp.ok) {
            const members = await resp.json();
            if (!Array.isArray(members) || members.length === 0) {
                content.innerHTML = '<div style="text-align:center;color:var(--text-secondary);padding:20px">No members yet.</div>';
                return;
            }
            content.innerHTML = members.map(m => '<div class="member-row">'
                + '<div><div class="member-name">' + escapeHtml(m.name || m.email || 'Unknown') + '</div>'
                + '<div class="member-email">' + escapeHtml(m.email || '') + '</div></div>'
                + '<button class="btn-secondary" style="padding:4px 8px;font-size:11px" data-remove-list="' + escapeHtml(String(listId||'')) + '" data-remove-id="' + escapeHtml(String(m.id||'')) + '">Remove</button>'
                + '</div>').join('');
            content.querySelectorAll('[data-remove-list]').forEach(function(btn){
                btn.addEventListener('click', function(){
                    removeMember(this.getAttribute('data-remove-list'), this.getAttribute('data-remove-id'));
                });
            });
        }
    } catch (e) { content.innerHTML = '<div style="color:var(--id-danger-icon)">Error loading members.</div>'; }
}

async function deleteList(listId) {
    try {
        const resp = await fetch('/api/crm/lists/' + listId, { method: 'DELETE' });
        if (resp.ok) {
            allLists = allLists.filter(l => l.id !== listId);
            renderLists();
            hideDeleteModal();
        } else {
            const msg = await resp.text().catch(function(){return '';});
            if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Lists', msg || ('Delete failed (' + resp.status + ')'), 'error');
        }
    } catch (e) { console.error('deleteList:', e); if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Lists', 'Delete failed', 'error'); }
}

async function removeMember(listId, contactId) {
    try {
        const resp = await fetch('/api/crm/lists/' + listId + '/members/' + contactId, { method: 'DELETE' });
        if (!resp.ok) {
            const msg = await resp.text().catch(function(){return '';});
            if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Lists', msg || ('Request failed (' + resp.status + ')'), 'error');
            return;
        }
        loadMembers(listId);
    } catch (e) { console.error('removeMember:', e); if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Lists', 'Remove failed', 'error'); }
}

async function addMemberToList() {
    const input = document.getElementById('add-member-search');
    const email = input.value.trim();
    if (!email) return;
    const listId = membersListId;
    if (!listId) { if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Lists', 'No list selected', 'error'); return; }
    try {
        const resp = await fetch('/api/crm/lists/' + listId + '/members', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email: email })
        });
        if (!resp.ok) {
            const msg = await resp.text().catch(function(){return '';});
            if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Lists', msg || ('Request failed (' + resp.status + ')'), 'error');
            return;
        }
        input.value = '';
        loadMembers(listId);
    } catch (e) { console.error('addMember:', e); if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Lists', 'Add failed', 'error'); }
}

function exportLists() {
    const csv = 'Name,Type,Members\n' + allLists.map(l => csvCell(l.name||'') + ',' + csvCell(l.list_type||'') + ',' + csvCell(l.member_count||0)).join('\n');
    downloadFile(csv, 'lists-export.csv', 'text/csv');
}

function exportSingleList(listId) {
    const list = allLists.find(l => l.id === listId);
    if (!list) return;
    const csv = 'Name,Type,Members\n' + csvCell(list.name||'') + ',' + csvCell(list.list_type||'') + ',' + csvCell(list.member_count||0);
    var fname = String(list.name || 'list').replace(/[^a-zA-Z0-9._-]+/g, '_');
    downloadFile(csv, fname + '.csv', 'text/csv');
}

function downloadFile(content, filename, mime) {
    const blob = new Blob([content], { type: mime });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = filename;
    a.click();
    URL.revokeObjectURL(a.href);
}

function importLists() { showImportModal(); }

function processImport() {
    const file = document.getElementById('import-csv-file').files[0];
    const listId = document.getElementById('import-target-list').value;
    if (!file || !listId) return;
    const reader = new FileReader();
    reader.onload = async function(e) {
        const lines = e.target.result.split('\n').filter(l => l.trim());
        var added = 0, failed = 0;
        for (let i = 1; i < lines.length; i++) {
            const cols = lines[i].split(',').map(c => c.replace(/"/g, '').trim());
            if (cols[0]) {
                try {
                    const resp = await fetch('/api/crm/lists/' + listId + '/members', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ name: cols[0], email: cols[1] || '' })
                    });
                    if (resp.ok) added++; else failed++;
                } catch (err) { failed++; }
            }
        }
        hideImportModal();
        if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Lists', 'Imported ' + added + (failed ? ', failed ' + failed : ''), failed ? 'error' : 'success');
        loadMembers(listId);
    };
    reader.readAsText(file);
}

document.getElementById('list-form').addEventListener('submit', async function(e) {
    e.preventDefault();
    const formData = new FormData(this);
    const data = Object.fromEntries(formData);
    const id = data.id;
    const url = id ? '/api/crm/lists/' + id : '/api/crm/lists';
    const method = id ? 'PUT' : 'POST';
    try {
        const resp = await fetch(url, {
            method: method,
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });
        if (resp.ok) {
            hideListModal();
            loadLists();
        }
    } catch (err) { console.error('saveList:', err); }
});

document.getElementById('list-form-type').addEventListener('change', function() {
    document.getElementById('query-group').style.display = this.value === 'dynamic' ? 'block' : 'none';
});

// #2537: minimal hook — the segment builder refreshes this grid after save.
window.GBListsRefresh = loadLists;
loadLists();
})();
