(function(){
var sharepointSites=[];
var calendarEvents=[];
var onedriveFiles=[];
var currentTab='sharepoint';
var currentFolder='root';
var folderStack=['root'];

function showFeedback(msg,type){
    var el=document.getElementById('m365-feedback');
    if(!el){el=document.createElement('div');el.id='m365-feedback';el.style.cssText='position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;transition:opacity .3s';document.body.appendChild(el)}
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
    currentTab=tab;
    document.querySelectorAll('.m365-tab').forEach(function(t){t.classList.remove('active')});
    document.querySelectorAll('.m365-panel').forEach(function(p){p.classList.remove('active')});
    if(btn)btn.classList.add('active');
    var panel=document.getElementById('panel-'+tab);
    if(panel)panel.classList.add('active');
}

function updateStats(){
    var el1=document.getElementById('stat-sites');
    var el2=document.getElementById('stat-events');
    var el3=document.getElementById('stat-files');
    if(el1)el1.textContent=sharepointSites.length;
    if(el2)el2.textContent=calendarEvents.length;
    if(el3)el3.textContent=onedriveFiles.length;
}

function renderSharePoint(){
    var grid=document.getElementById('sharepoint-grid');
    if(!grid)return;
    if(sharepointSites.length===0){grid.innerHTML='<div class="m365-empty"><div class="m365-empty-icon">📁</div><h3>No SharePoint Sites</h3><p>Connect your M365 account to access SharePoint</p></div>';return}
    grid.innerHTML=sharepointSites.map(function(s){
        return '<div class="m365-card">'
            +'<div class="m365-card-header"><span class="m365-card-icon">📁</span><span class="m365-card-title">'+(s.name||'')+'</span></div>'
            +'<div class="m365-card-body">'
            +'<div class="m365-card-row"><span class="label">URL</span><span style="font-size:12px">'+(s.url||'')+'</span></div>'
            +'<div class="m365-card-row"><span class="label">Documents</span><span>'+(s.document_count||0)+'</span></div>'
            +'</div>'
            +'<div class="m365-card-actions"><button class="m365-btn" onclick="window._m365.viewSiteLibraries(\''+s.id+'\')">View Libraries</button></div>'
            +'</div>';
    }).join('');
}

function renderCalendar(){
    var grid=document.getElementById('calendar-grid');
    if(!grid)return;
    if(calendarEvents.length===0){grid.innerHTML='<div class="m365-empty"><div class="m365-empty-icon">📅</div><h3>No Upcoming Events</h3><p>No meetings scheduled</p></div>';return}
    grid.innerHTML=calendarEvents.map(function(e){
        return '<div class="m365-card">'
            +'<div class="m365-card-header"><span class="m365-card-icon">📅</span><span class="m365-card-title">'+(e.subject||'')+'</span></div>'
            +'<div class="m365-card-body">'
            +'<div class="m365-card-row"><span class="label">Start</span><span>'+(e.start||'')+'</span></div>'
            +'<div class="m365-card-row"><span class="label">End</span><span>'+(e.end||'')+'</span></div>'
            +'<div class="m365-card-row"><span class="label">Location</span><span>'+(e.location||'Online')+'</span></div>'
            +'<div class="m365-card-row"><span class="label">Attendees</span><span>'+(e.attendees||0)+'</span></div>'
            +'</div></div>';
    }).join('');
}

function renderOneDrive(){
    var tbody=document.getElementById('onedrive-tbody');
    if(!tbody)return;
    if(onedriveFiles.length===0){tbody.innerHTML='<tr><td colspan="5" class="m365-empty">No files found</td></tr>';return}
    tbody.innerHTML=onedriveFiles.map(function(f){
        var isFolder=f.type==='folder';
        return '<tr>'
            +'<td>'+(isFolder?'📁 ':'📄 ')+'<span style="cursor:pointer;color:'+(isFolder?'var(--primary-color,#3b82f6)':'inherit')+'" '+(isFolder?'onclick="window._m365.navigateFolder(\''+f.id+'\')"':'')+'>'+(f.name||'')+'</span></td>'
            +'<td><span class="m365-badge '+f.type+'">'+f.type+'</span></td>'
            +'<td>'+(f.modified||'')+'</td>'
            +'<td>'+(f.size||'--')+'</td>'
            +'<td>'+(isFolder?'<button class="m365-btn" onclick="window._m365.navigateFolder(\''+f.id+'\')">Open</button>':'<button class="m365-btn" onclick="window._m365.downloadFile(\''+f.id+'\')">Download</button>')+'</td>'
            +'</tr>';
    }).join('');
    renderBreadcrumb();
}

function renderBreadcrumb(){
    var el=document.getElementById('onedrive-breadcrumb');
    if(!el)return;
    el.innerHTML=folderStack.map(function(f,i){
        if(i===folderStack.length-1)return '<span class="m365-breadcrumb-current">'+f+'</span>';
        return '<button class="m365-breadcrumb-item" onclick="window._m365.navigateTo(\''+f+'\','+i+')">'+f+'</button><span class="m365-breadcrumb-sep">/</span>';
    }).join('');
}

function renderSettings(){
    var el4=document.getElementById('stat-connection');
    var accountEl=document.getElementById('settings-account');
    var statusEl=document.getElementById('settings-status');
    var syncEl=document.getElementById('settings-sync');
    if(el4){el4.textContent='Connected';el4.style.color='#22c55e'}
    if(accountEl)accountEl.textContent='Connected to Microsoft 365';
    if(statusEl){statusEl.textContent='Connected';statusEl.className='m365-badge connected'}
    if(syncEl)syncEl.textContent=new Date().toLocaleString();
}

async function loadSharePoint(){
    var data=await apiCall('/api/m365/sharepoint');
    if(data){sharepointSites=Array.isArray(data)?data:[];updateStats();renderSharePoint()}
}

async function loadCalendar(){
    var data=await apiCall('/api/m365/calendar');
    if(data){calendarEvents=Array.isArray(data)?data:[];updateStats();renderCalendar()}
}

async function loadOneDrive(folderId){
    var url='/api/m365/onedrive';
    if(folderId&&folderId!=='root')url+='?folder='+folderId;
    var data=await apiCall(url);
    if(data){onedriveFiles=Array.isArray(data)?data:[];currentFolder=folderId||'root';updateStats();renderOneDrive()}
}

async function loadSettings(){
    var data=await apiCall('/api/m365/settings');
    if(data){
        if(data.connected!==undefined){
            var el4=document.getElementById('stat-connection');
            var accountEl=document.getElementById('settings-account');
            var statusEl=document.getElementById('settings-status');
            if(el4){el4.textContent=data.connected?'Connected':'Disconnected';el4.style.color=data.connected?'#22c55e':'#ef4444'}
            if(accountEl)accountEl.textContent=data.account||'Not connected';
            if(statusEl){statusEl.textContent=data.connected?'Connected':'Disconnected';statusEl.className='m365-badge '+(data.connected?'connected':'disconnected')}
        }
    }else{renderSettings()}
}

function navigateFolder(folderId){
    if(!folderStack.includes(folderId))folderStack.push(folderId);
    loadOneDrive(folderId);
}

function navigateTo(folderId,index){
    folderStack=folderStack.slice(0,index+1);
    loadOneDrive(folderId);
}

async function viewSiteLibraries(siteId){
    showFeedback('Loading libraries...','success');
}

async function downloadFile(fileId){
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
    var data=await apiCall('/api/m365/calendar',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({subject:subject,start:start,end:end,attendees:attendees.split(',').map(function(e){return e.trim()})})});
    if(data){hideModal('meeting-modal');showFeedback('Meeting created','success');loadCalendar()}
}

async function syncNow(){
    showFeedback('Syncing...','success');
    var syncEl=document.getElementById('settings-sync');
    if(syncEl)syncEl.textContent='Syncing...';
    await apiCall('/api/m365/sync',{method:'POST'});
    if(syncEl)syncEl.textContent=new Date().toLocaleString();
    loadAll();
}

async function updateSync(type,frequency){
    await apiCall('/api/m365/settings/sync',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({type:type,frequency:parseInt(frequency)})});
    showFeedback('Sync frequency updated','success');
}

async function connectAccount(){
    showFeedback('Connecting to Microsoft 365...','success');
    await apiCall('/api/m365/connect',{method:'POST'});
    loadAll();
}

async function disconnectAccount(){
    if(!confirm('Disconnect Microsoft 365 account?'))return;
    await apiCall('/api/m365/disconnect',{method:'POST'});
    showFeedback('Account disconnected','success');
    loadAll();
}

function showModal(id){var el=document.getElementById(id);if(el)el.style.display='flex'}
function hideModal(id){var el=document.getElementById(id);if(el)el.style.display='none'}

function loadAll(){loadSharePoint();loadCalendar();loadOneDrive('root');loadSettings()}

window._m365={switchTab:switchTab,navigateFolder:navigateFolder,navigateTo:navigateTo,viewSiteLibraries:viewSiteLibraries,downloadFile:downloadFile,showCreateMeeting:showCreateMeeting,createMeeting:createMeeting,syncNow:syncNow,updateSync:updateSync,connectAccount:connectAccount,disconnectAccount:disconnectAccount,showModal:showModal,hideModal:hideModal,loadSharePoint:loadSharePoint,loadCalendar:loadCalendar,loadOneDrive:loadOneDrive,loadAll:loadAll};
loadAll();
})();
