if (window.GBAppLifecycle) GBAppLifecycle.begin("o365");
(function(){
var state={sharepointSites:[],calendarEvents:[],onedriveFiles:[],teams:[],currentTab:'sharepoint',currentFolder:'root',folderStack:[{id:'root',name:'Root'}]};

// #2521 — the shared skeleton is js/gb-app-kit.js; this app keeps no copy of it.
var kit = window.GBAppKit;
function escapeHtml(value){ return kit.escapeHtml(value); }
function showFeedback(msg,type){ return kit.showFeedback('o365', msg, type); }
function apiCall(url,options){ return kit.apiCall('o365', url, options); }

function switchTab(tab,btn){
    state.currentTab=tab;
    kit.switchTab(tab, btn, {
        tabSelector: '.o365-tabs [role="tab"], .o365-tabs .o365-tab',
        panelSelector: '.o365-panel'
    });
}

function updateStats(){
    var values={
        'stat-sites': state.sharepointSites.length,
        'stat-events': state.calendarEvents.length,
        'stat-files': state.onedriveFiles.length,
        'stat-teams': state.teams.length
    };
    if (window.GBAppKit && window.GBAppKit.setStatValues) return window.GBAppKit.setStatValues(values);
    Object.keys(values).forEach(function(id){ var el=document.getElementById(id); if(el) el.textContent=String(values[id]); });
}

function renderSharePoint(){
    var grid=document.getElementById('sharepoint-grid');
    if(!grid)return;
    if(state.sharepointSites.length===0){grid.innerHTML='<div class="o365-empty"><div class="o365-empty-icon">📁</div><h3>No SP Sites</h3><p>Connect your o365 account to access SP</p></div>';return}
    grid.innerHTML=state.sharepointSites.map(function(s){
        return '<div class="o365-card">'
            +'<div class="o365-card-header"><span class="o365-card-icon">📁</span><span class="o365-card-title">'+escapeHtml(s.site_name||s.name||'')+'</span></div>'
            +'<div class="o365-card-body">'
            +'<div class="o365-card-row"><span class="label">List</span><span style="font-size:12px">'+escapeHtml(s.list_name||'')+'</span></div>'
            +'<div class="o365-card-row"><span class="label">Items</span><span>'+escapeHtml(String(s.item_count||0))+'</span></div>'
            +'<div class="o365-card-row"><span class="label">Last Modified</span><span>'+escapeHtml(s.last_modified||'')+'</span></div>'
            +'</div>'
            +'</div>';
    }).join('');
}

function renderCalendar(){
    var grid=document.getElementById('calendar-grid');
    if(!grid)return;
    if(state.calendarEvents.length===0){grid.innerHTML='<div class="o365-empty"><div class="o365-empty-icon">📅</div><h3>No Upcoming Events</h3><p>No meetings scheduled</p></div>';return}
    grid.innerHTML=state.calendarEvents.map(function(e){
        return '<div class="o365-card">'
            +'<div class="o365-card-header"><span class="o365-card-icon">📅</span><span class="o365-card-title">'+escapeHtml(e.subject||'')+'</span></div>'
            +'<div class="o365-card-body">'
            +'<div class="o365-card-row"><span class="label">Start</span><span>'+escapeHtml(e.start||'')+'</span></div>'
            +'<div class="o365-card-row"><span class="label">End</span><span>'+escapeHtml(e.end||'')+'</span></div>'
            +'<div class="o365-card-row"><span class="label">Location</span><span>'+escapeHtml(e.location||'Online')+'</span></div>'
            +'<div class="o365-card-row"><span class="label">Attendees</span><span>'+escapeHtml(String(e.attendees||0))+'</span></div>'
            +'</div></div>';
    }).join('');
}

function renderOneDrive(){
    var tbody=document.getElementById('onedrive-tbody');
    if(!tbody)return;
    if(state.onedriveFiles.length===0){tbody.innerHTML='<tr><td colspan="5" class="o365-empty">No files found</td></tr>';return}
    tbody.innerHTML=state.onedriveFiles.map(function(f){
        var isFolder=(f.type==='folder');
        var safeId = String(f.id == null ? '' : f.id);
        var safeName = String(f.name == null ? '' : f.name);
        var safeType = String(f.type == null ? 'file' : f.type).replace(/[^a-zA-Z0-9_-]/g,'');
        var nameCell = isFolder
            ? '<button type="button" class="o365-link-btn" data-o365-nav-id="'+escapeHtml(safeId)+'" data-o365-nav-name="'+escapeHtml(safeName)+'">'+escapeHtml(safeName)+'</button>'
            : '<span>'+escapeHtml(safeName)+'</span>';
        var actionCell = isFolder
            ? '<button class="o365-btn" type="button" data-o365-nav-id="'+escapeHtml(safeId)+'" data-o365-nav-name="'+escapeHtml(safeName)+'">Open</button>'
            : '<button class="o365-btn" type="button" data-o365-dl-id="'+escapeHtml(safeId)+'" data-o365-dl-name="'+escapeHtml(safeName)+'">Download</button>';
        return '<tr>'
            +'<td>'+(isFolder?'📁 ':'📄 ')+nameCell+'</td>'
            +'<td><span class="o365-badge '+escapeHtml(safeType)+'">'+escapeHtml(safeType)+'</span></td>'
            +'<td>'+escapeHtml(String(f.modified||f.last_modified||''))+'</td>'
            +'<td>'+escapeHtml(String(f.size||f.size_bytes||'--'))+'</td>'
            +'<td>'+actionCell+'</td>'
            +'</tr>';
    }).join('');
    bindOneDriveActions(tbody);
    renderBreadcrumb();
}

function bindOneDriveActions(root){
    if (!root) return;
    root.querySelectorAll('[data-o365-nav-id]').forEach(function(btn){
        btn.addEventListener('click', function(){
            navigateFolder(this.getAttribute('data-o365-nav-id') || '', this.getAttribute('data-o365-nav-name') || '');
        });
    });
    root.querySelectorAll('[data-o365-dl-id]').forEach(function(btn){
        btn.addEventListener('click', function(){
            downloadFile(this.getAttribute('data-o365-dl-id') || '', this.getAttribute('data-o365-dl-name') || '');
        });
    });
}

function renderBreadcrumb(){
    var el=document.getElementById('onedrive-breadcrumb');
    if(!el)return;
    el.innerHTML=state.folderStack.map(function(f,i){
        var name = (typeof f === 'object' && f) ? (f.name || f.id || 'Folder') : String(f);
        if(i===state.folderStack.length-1)return '<span class="o365-breadcrumb-current">'+escapeHtml(String(name))+'</span>';
        return '<button type="button" class="o365-breadcrumb-item" data-o365-bc-index="'+i+'">'+escapeHtml(String(name))+'</button><span class="o365-breadcrumb-sep">/</span>';
    }).join('');
    el.querySelectorAll('[data-o365-bc-index]').forEach(function(btn){
        btn.addEventListener('click', function(){
            var idx = parseInt(this.getAttribute('data-o365-bc-index'), 10);
            if (!isNaN(idx)) navigateTo(idx);
        });
    });
}

async function loadSharePoint(){
    var data=await apiCall('/api/o365/sharepoint');
    if(data){state.sharepointSites=Array.isArray(data)?data:[];updateStats();renderSharePoint()}
}

async function loadTeams(){
    var data=await apiCall('/api/o365/teams');
    if(data){state.teams=Array.isArray(data)?data:[];updateStats()}
}

async function loadCalendar(){
    var data=await apiCall('/api/o365/calendar');
    if(data){state.calendarEvents=Array.isArray(data)?data:[];updateStats();renderCalendar()}
}

async function loadOneDrive(folderId){
    var url='/api/o365/onedrive';
    if(folderId&&folderId!=='root')url+='?folder='+folderId;
    var data=await apiCall(url);
    if(data){state.onedriveFiles=Array.isArray(data)?data:[];state.currentFolder=folderId||'root';updateStats();renderOneDrive()}
}

async function loadSettings(){
    var data=await apiCall('/api/o365/settings');
    var accountEl=document.getElementById('settings-account');
    var badgeEl=document.getElementById('settings-status');
    var syncEl=document.getElementById('settings-sync');
    if(!data) return;
    if(data.connected){
        if(accountEl)accountEl.textContent=data.account||'o365';
        if(badgeEl){badgeEl.textContent='Connected';badgeEl.className='o365-badge o365-badge-connected';badgeEl.style.display='inline-block'}
        if(syncEl)syncEl.textContent=(data.last_sync?gbFormatDateTime(new Date(data.last_sync)):'Never');
    }else{
        if(accountEl)accountEl.textContent='Not connected';
        if(badgeEl){badgeEl.textContent='Not connected';badgeEl.className='o365-badge o365-badge-disconnected';badgeEl.style.display='inline-block'}
        if(syncEl&&data.last_sync)syncEl.textContent=gbFormatDateTime(new Date(data.last_sync));
        else if(syncEl)syncEl.textContent='Never';
    }
}

function navigateFolder(folderId, folderName){
    var name = folderName || folderId;
    var exists = state.folderStack.find(function(x){ return (typeof x==="object"?x.id:x)===folderId; });
    if(!exists) state.folderStack.push({id: folderId, name: name});
    loadOneDrive(folderId);
}

function navigateTo(index){
    state.folderStack = state.folderStack.slice(0, index+1);
    var cur = state.folderStack[state.folderStack.length-1];
    var id = (typeof cur==="object") ? cur.id : cur;
    loadOneDrive(id);
}

function downloadFile(fileId, fileName){
    showFeedback('Starting download...','success');
    var token = (window.GBAppKit && window.GBAppKit.getAuthToken) ? window.GBAppKit.getAuthToken() : (localStorage.getItem('gb_token') || '');
    var headers = {};
    if (token) headers.Authorization = 'Bearer ' + token;
    return fetch('/api/o365/onedrive/download/'+fileId, { headers: headers }).then(function(r){
        if(!r.ok) throw new Error('HTTP '+r.status);
        return r.blob();
    }).then(function(blob){
        var url=URL.createObjectURL(blob);
        var a=document.createElement('a');
        a.href=url;
        a.download=fileName || ('o365-file-'+fileId);
        document.body.appendChild(a);
        a.click();
        a.remove();
        setTimeout(function(){URL.revokeObjectURL(url);},2000);
        showFeedback('Download complete','success');
    }).catch(function(e){
        showFeedback('Download failed: '+e.message,'error');
    });
}

function showCreateMeeting(){
    var modal=document.getElementById('meeting-modal');
    if(modal)modal.style.display='flex';
}

async function createMeeting(){
    var subject=document.getElementById('meeting-subject').value;
    var start=document.getElementById('meeting-start').value;
    var end=document.getElementById('meeting-end').value;
    var attendees=document.getElementById('meeting-attendees').value;
    if(!subject||!start){showFeedback('Subject and start time required','error');return}
    var data=await apiCall('/api/o365/calendar',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({subject:subject,start:start,end:end,attendees:attendees.split(',').map(function(e){return e.trim()})})});
    if(data){hideModal('meeting-modal');showFeedback('Meeting created','success');loadCalendar()}
}

async function syncNow(){
    showFeedback('Syncing...','success');
    var syncEl=document.getElementById('settings-sync');
    if(syncEl)syncEl.textContent='Syncing...';
    var data=await apiCall('/api/o365/sync',{method:'POST'});
    if(!data){
        if(syncEl)syncEl.textContent='Never';
        return;
    }
    loadAll();
}

async function updateSync(type,frequency){
    var data=await apiCall('/api/o365/settings/sync',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({type:type,frequency:parseInt(frequency)})});
    if(!data) return;
    showFeedback('Sync frequency updated','success');
}

async function connectAccount(){
    showFeedback('Connecting to o365...','success');
    var data=await apiCall('/api/o365/connect',{method:'POST'});
    if(!data) return;
    loadAll();
}

async function disconnectAccount(){
    if(!confirm('Disconnect o365 account?'))return;
    var data=await apiCall('/api/o365/disconnect',{method:'POST'});
    if(!data) return;
    showFeedback('Account disconnected','success');
    loadAll();
}

function showModal(id){ return kit.showModal(id); }
function hideModal(id){ return kit.hideModal(id); }

function bindDelegated(){
    var root = document.querySelector('[data-gb-app="o365"]') || document;
    if (root.__o365Bound) return;
    root.__o365Bound = true;
    root.addEventListener('click', function(e){
        var tab = e.target.closest('.o365-tabs [data-tab]');
        if (tab) { switchTab(tab.getAttribute('data-tab'), tab); return; }
        var btn = e.target.closest('[data-o365-action]');
        if (!btn) return;
        var action = btn.getAttribute('data-o365-action');
        if (action === 'load-all') loadAll();
        else if (action === 'show-create-meeting') showCreateMeeting();
        else if (action === 'sync-now') syncNow();
        else if (action === 'hide-modal') hideModal(btn.getAttribute('data-o365-modal') || 'meeting-modal');
        else if (action === 'create-meeting') createMeeting();
    });
    root.addEventListener('change', function(e){
        var sel = e.target.closest('[data-o365-sync]');
        if (sel) updateSync(sel.getAttribute('data-o365-sync'), sel.value);
    });
    document.addEventListener('keydown', function(e){
        var tab = e.target.closest('.o365-tabs [data-tab]');
        if (!tab) return;
        var tabs = Array.prototype.slice.call(document.querySelectorAll('.o365-tabs [data-tab]'));
        var idx = tabs.indexOf(tab);
        if (idx === -1) return;
        var next = null;
        if (e.key === 'ArrowRight') next = tabs[(idx + 1) % tabs.length];
        else if (e.key === 'ArrowLeft') next = tabs[(idx - 1 + tabs.length) % tabs.length];
        else if (e.key === 'Home') next = tabs[0];
        else if (e.key === 'End') next = tabs[tabs.length - 1];
        if (next) {
            e.preventDefault();
            switchTab(next.getAttribute('data-tab'), next);
            next.focus();
        }
    });
}

function loadAll(){loadSharePoint();loadCalendar();loadOneDrive('root');loadTeams();loadSettings()}

function init(){
    bindDelegated();
    var tabs = Array.prototype.slice.call(document.querySelectorAll('.o365-tabs [data-tab]'));
    var active = tabs.filter(function(t){ return t.classList.contains('active') || t.getAttribute('aria-selected') === 'true'; })[0] || tabs[0];
    if (active) switchTab(active.getAttribute('data-tab') || 'sharepoint', active);
    loadAll();
}

window._o365={switchTab:switchTab,navigateFolder:navigateFolder,navigateTo:navigateTo,downloadFile:downloadFile,showCreateMeeting:showCreateMeeting,createMeeting:createMeeting,syncNow:syncNow,updateSync:updateSync,connectAccount:connectAccount,disconnectAccount:disconnectAccount,showModal:showModal,hideModal:hideModal,loadAll:loadAll};
init();
})();
