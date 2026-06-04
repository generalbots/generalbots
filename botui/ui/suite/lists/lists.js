(function() {
'use strict';
let allLists = [];
let currentFilter = 'all';
let editingListId = null;

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
    document.getElementById('members-modal-title').textContent = (list ? list.name : 'List') + ' Members';
    modal.style.display = 'flex';
    loadMembers(listId);
}

function hideMembersModal() { document.getElementById('members-modal').style.display = 'none'; }

function showDeleteModal(listId) {
    const list = allLists.find(l => l.id === listId);
    document.getElementById('delete-list-name').textContent = list ? list.name : '';
    document.getElementById('delete-modal').style.display = 'flex';
    document.getElementById('btn-confirm-delete').onclick = function() { deleteList(listId); };
}

function hideDeleteModal() { document.getElementById('delete-modal').style.display = 'none'; }

function showImportModal() {
    const sel = document.getElementById('import-target-list');
    sel.innerHTML = allLists.map(l => '<option value="' + l.id + '">' + l.name + '</option>').join('');
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
    grid.innerHTML = filtered.map(list => '<div class="list-card" onclick="showMembersModal(\'' + list.id + '\')">'
        + '<div class="list-card-header">'
        + '<h3 class="list-card-title">' + (list.name || 'Unnamed') + '</h3>'
        + '<span class="list-type ' + (list.list_type || 'static') + '">' + (list.list_type || 'static') + '</span>'
        + '</div>'
        + '<div class="list-stats">'
        + '<div class="list-stat"><span class="list-stat-value">' + (list.member_count || 0) + '</span><span class="list-stat-label">Members</span></div>'
        + '</div>'
        + '<div class="list-card-actions" onclick="event.stopPropagation()">'
        + '<button onclick="showListModal(\'' + list.id + '\')">Edit</button>'
        + '<button onclick="exportSingleList(\'' + list.id + '\')">Export</button>'
        + '<button class="danger" onclick="showDeleteModal(\'' + list.id + '\')">Delete</button>'
        + '</div>'
        + '</div>').join('');
}

async function loadLists() {
    try {
        const resp = await fetch('/api/crm/lists');
        if (resp.ok) {
            allLists = await resp.json();
            if (!Array.isArray(allLists)) allLists = [];
            renderLists();
        }
    } catch (e) { console.error('loadLists:', e); }
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
                + '<div><div class="member-name">' + (m.name || m.email || 'Unknown') + '</div>'
                + '<div class="member-email">' + (m.email || '') + '</div></div>'
                + '<button class="btn-secondary" style="padding:4px 8px;font-size:11px" onclick="removeMember(\'' + listId + '\',\'' + m.id + '\')">Remove</button>'
                + '</div>').join('');
        }
    } catch (e) { content.innerHTML = '<div style="color:#ef4444">Error loading members.</div>'; }
}

async function deleteList(listId) {
    try {
        const resp = await fetch('/api/crm/lists/' + listId, { method: 'DELETE' });
        if (resp.ok) {
            allLists = allLists.filter(l => l.id !== listId);
            renderLists();
            hideDeleteModal();
        }
    } catch (e) { console.error('deleteList:', e); }
}

async function removeMember(listId, contactId) {
    try {
        await fetch('/api/crm/lists/' + listId + '/members/' + contactId, { method: 'DELETE' });
        loadMembers(listId);
    } catch (e) { console.error('removeMember:', e); }
}

async function addMemberToList() {
    const input = document.getElementById('add-member-search');
    const email = input.value.trim();
    if (!email) return;
    const modal = document.getElementById('members-modal');
    const title = document.getElementById('members-modal-title').textContent;
    const listId = allLists.find(l => title.includes(l.name))?.id;
    if (!listId) return;
    try {
        await fetch('/api/crm/lists/' + listId + '/members', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email: email })
        });
        input.value = '';
        loadMembers(listId);
    } catch (e) { console.error('addMember:', e); }
}

function exportLists() {
    const csv = 'Name,Type,Members\n' + allLists.map(l => '"' + (l.name||'') + '","' + (l.list_type||'') + '",' + (l.member_count||0)).join('\n');
    downloadFile(csv, 'lists-export.csv', 'text/csv');
}

function exportSingleList(listId) {
    const list = allLists.find(l => l.id === listId);
    if (!list) return;
    const csv = 'Name,Type,Members\n"' + (list.name||'') + '","' + (list.list_type||'') + '",' + (list.member_count||0);
    downloadFile(csv, (list.name || 'list') + '.csv', 'text/csv');
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
        for (let i = 1; i < lines.length; i++) {
            const cols = lines[i].split(',').map(c => c.replace(/"/g, '').trim());
            if (cols[0]) {
                await fetch('/api/crm/lists/' + listId + '/members', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ name: cols[0], email: cols[1] || '' })
                });
            }
        }
        hideImportModal();
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

loadLists();
})();
