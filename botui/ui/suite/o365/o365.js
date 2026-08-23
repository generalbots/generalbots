if (window.GBAppLifecycle) GBAppLifecycle.begin("o365");
(function(){
var state={sharepointSites:[],calendarEvents:[],onedriveFiles:[],currentTab:'sharepoint',currentFolder:'root',folderStack:['root']};

function showFeedback(msg,type){
    var el=document.getElementById('o365-feedback');
    if(!el){el=document.createElement('div');el.id='o365-feedback';el.style.cssText='position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;transition:opacity .3s';document.body.appendChild(el)}
    el.textContent=msg;
    el.style.background=type==='error'?'#ef4444':'#22c55e';
    el.style.color='#fff';
    el.style.opacity='1';
    setTimeout(function(){el.style.opacity='0'},3000);
}

async function apiCall(url,options){
    try{var r=await fetch(url,options);if(!r.ok)throw new Error('HTTP '+r.status);var t=await r.text();return t?JSON.parse(t):null}catch(e){showFeedback('API error: '+e.message,'error');return null}
}

function switchTab(tab,btn){
    state.currentTab=tab;
    document.querySelectorAll('.o365-tab').forEach(function(t){t.classList.remove('active')});
    document.querySelectorAll('.o365-panel').forEach(function(p){p.classList.remove('active')});
    if(btn)btn.classList.add('active');
    var panel=document.getElementById('panel-'+tab);
    if(panel)panel.classList.add('active');
}

function updateStats(){
    var s1=document.getElementById('stat-sites');
    var s2=document.getElementById('stat-events');
    var s3=document.getElementById('stat-files');
    if(s1)s1.textContent=state.sharepointSites.length;
    if(s2)s2.textContent=state.calendarEvents.length;
    if(s3)s3.textContent=state.onedriveFiles.length;
}

function renderSharePoint(){
    var grid=document.getElementById('sharepoint-grid');
    if(!grid)return;
    if(state.sharepointSites.length===0){grid.innerHTML='<div class="o365-empty"><div class="o365-empty-icon">📁</div><h3>No SP Sites</h3><p>Connect your o365 account to access SP</p></div>';return}
    grid.innerHTML=state.sharepointSites.map(function(s){
        return '<div class="o365-card">'
            +'<div class="o365-card-header"><span class="o365-card-icon">📁</span><span class="o365-card-title">'+(s.site_name||s.name||'')+'</span></div>'
            +'<div class="o365-card-body">'
            +'<div class="o365-card-row"><span class="label">List</span><span style="font-size:12px">'+(s.list_name||'')+'</span></div>'
            +'<div class="o365-card-row"><span class="label">Items</span><span>'+(s.item_count||0)+'</span></div>'
            +'<div class="o365-card-row"><span class="label">Last Modified</span><span>'+(s.last_modified||'')+'</span></div>'
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
            +'<div class="o365-card-header"><span class="o365-card-icon">📅</span><span class="o365-card-title">'+(e.subject||'')+'</span></div>'
            +'<div class="o365-card-body">'
            +'<div class="o365-card-row"><span class="label">Start</span><span>'+(e.start||'')+'</span></div>'
            +'<div class="o365-card-row"><span class="label">End</span><span>'+(e.end||'')+'</span></div>'
            +'<div class="o365-card-row"><span class="label">Location</span><span>'+(e.location||'Online')+'</span></div>'
            +'<div class="o365-card-row"><span class="label">Attendees</span><span>'+(e.attendees||0)+'</span></div>'
            +'</div></div>';
    }).join('');
}

function renderOneDrive(){
    var tbody=document.getElementById('onedrive-tbody');
    if(!tbody)return;
    if(state.onedriveFiles.length===0){tbody.innerHTML='<tr><td colspan="5" class="o365-empty">No files found</td></tr>';return}
    tbody.innerHTML=state.onedriveFiles.map(function(f){
        var isFolder=(f.type==='folder');
        return '<tr>'
            +'<td>'+(isFolder?'📁 ':'📄 ')+'<span style="cursor:pointer;color:'+(isFolder?'var(--primary-color,#3b82f6)':'inherit')+'" '+(isFolder?'onclick="window._o365.navigateFolder(\''+f.id+'\')"':'')+'>'+(f.name||'')+'</span></td>'
            +'<td><span class="o365-badge '+(f.type||'file')+'">'+(f.type||'file')+'</span></td>'
            +'<td>'+(f.modified||f.last_modified||'')+'</td>'
            +'<td>'+(f.size||f.size_bytes||'--')+'</td>'
            +'<td>'+(isFolder?'<button class="o365-btn" onclick="window._o365.navigateFolder(\''+f.id+'\')">Open</button>':'<button class="o365-btn" onclick="window._o365.downloadFile(\''+f.id+'\')">Download</button>')+'</td>'
            +'</tr>';
    }).join('');
    renderBreadcrumb();
}

function renderBreadcrumb(){
    var el=document.getElementById('onedrive-breadcrumb');
    if(!el)return;
    el.innerHTML=state.folderStack.map(function(f,i){
        if(i===state.folderStack.length-1)return '<span class="o365-breadcrumb-current">'+f+'</span>';
        return '<button class="o365-breadcrumb-item" onclick="window._o365.navigateTo(\''+f+'\','+i+')">'+f+'</button><span class="o365-breadcrumb-sep">/</span>';
    }).join('');
}

async function loadSharePoint(){
    var data=await apiCall('/api/o365/sharepoint');
    if(data){state.sharepointSites=Array.isArray(data)?data:[];updateStats();renderSharePoint()}
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
    var statusEl=document.getElementById('o365-connection-status');
    var accountEl=document.getElementById('settings-account');
    var badgeEl=document.getElementById('settings-status');
    var syncEl=document.getElementById('settings-sync');
    if(data&&data.connected){
        if(statusEl){statusEl.textContent='Connected';statusEl.className='o365-status connected'}
        if(accountEl)accountEl.textContent=data.account||'Connected to o365';
        if(badgeEl){badgeEl.textContent='Connected';badgeEl.className='o365-badge connected'}
        if(syncEl)syncEl.textContent=new Date().toLocaleString();
    }else{
        if(statusEl){statusEl.textContent='Disconnected';statusEl.className='o365-status'}
        if(accountEl)accountEl.textContent='Not connected';
        if(badgeEl){badgeEl.textContent='Disconnected';badgeEl.className='o365-badge disconnected'}
    }
}

function navigateFolder(folderId){
    if(!state.folderStack.includes(folderId))state.folderStack.push(folderId);
    loadOneDrive(folderId);
}

function navigateTo(folderId,index){
    state.folderStack=state.folderStack.slice(0,index+1);
    loadOneDrive(folderId);
}

function downloadFile(fileId){
    showFeedback('Starting download...','success');
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
    await apiCall('/api/o365/sync',{method:'POST'});
    if(syncEl)syncEl.textContent=new Date().toLocaleString();
    loadAll();
}

async function updateSync(type,frequency){
    await apiCall('/api/o365/settings/sync',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({type:type,frequency:parseInt(frequency)})});
    showFeedback('Sync frequency updated','success');
}

async function connectAccount(){
    showFeedback('Connecting to o365...','success');
    await apiCall('/api/o365/connect',{method:'POST'});
    loadAll();
}

async function disconnectAccount(){
    if(!confirm('Disconnect o365 account?'))return;
    await apiCall('/api/o365/disconnect',{method:'POST'});
    showFeedback('Account disconnected','success');
    loadAll();
}

function showModal(id){var el=document.getElementById(id);if(el)el.style.display='flex'}
function hideModal(id){var el=document.getElementById(id);if(el)el.style.display='none'}

function loadAll(){loadSharePoint();loadCalendar();loadOneDrive('root');loadSettings()}

window._o365={switchTab:switchTab,navigateFolder:navigateFolder,navigateTo:navigateTo,downloadFile:downloadFile,showCreateMeeting:showCreateMeeting,createMeeting:createMeeting,syncNow:syncNow,updateSync:updateSync,connectAccount:connectAccount,disconnectAccount:disconnectAccount,showModal:showModal,hideModal:hideModal,loadAll:loadAll};
loadAll();
})();
