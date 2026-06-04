(function(){
var meetings=[];
var transcripts=[];
var documents=[];
var signDocs=[];
var currentTab='upcoming';
var selectedDocId=null;
var selectedSignId=null;
var signDrawing=false;
var signLastX=0;
var signLastY=0;
var filteredTranscripts=[];

function showFeedback(msg,type){
    var el=document.getElementById('min-feedback');
    if(!el){el=document.createElement('div');el.id='min-feedback';el.style.cssText='position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;transition:opacity .3s';document.body.appendChild(el)}
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
    document.querySelectorAll('.min-tab').forEach(function(t){t.classList.remove('active')});
    document.querySelectorAll('.min-panel').forEach(function(p){p.classList.remove('active')});
    if(btn)btn.classList.add('active');
    var panel=document.getElementById('min-panel-'+tab);
    if(panel)panel.classList.add('active');
}

function updateStats(){
    var el1=document.getElementById('min-stat-upcoming');
    var el2=document.getElementById('min-stat-transcripts');
    var el3=document.getElementById('min-stat-documents');
    var el4=document.getElementById('min-stat-pending-sign');
    if(el1)el1.textContent=meetings.length;
    if(el2)el2.textContent=transcripts.length;
    if(el3)el3.textContent=documents.length;
    if(el4)el4.textContent=signDocs.filter(function(s){return s.status==='pending'}).length;
}

function renderMeetings(){
    var grid=document.getElementById('min-upcoming-grid');
    if(!grid)return;
    if(meetings.length===0){grid.innerHTML='<div class="min-empty"><div class="min-empty-icon">📅</div><h3>No Upcoming Meetings</h3><p>Scheduled meetings will appear here</p></div>';return}
    grid.innerHTML=meetings.map(function(m){
        var participants=m.participants||[];
        return '<div class="min-card">'
            +'<div class="min-card-header"><span class="min-card-title">'+(m.title||'')+'</span><span class="min-badge scheduled">'+(m.status||'scheduled')+'</span></div>'
            +'<div class="min-card-body">'
            +'<div class="min-card-row"><span class="label">Date</span><span>'+(m.date||'')+'</span></div>'
            +'<div class="min-card-row"><span class="label">Time</span><span>'+(m.time||'')+'</span></div>'
            +'<div class="min-card-row"><span class="label">Duration</span><span>'+(m.duration||'')+'</span></div>'
            +'<div class="min-card-row"><span class="label">Location</span><span>'+(m.location||'Online')+'</span></div>'
            +'</div>'
            +'<div style="margin-top:8px"><div class="min-participant-list">'+participants.map(function(p){return '<span class="min-participant">'+(p.name||p)+'</span>'}).join('')+'</div></div>'
            +'<div class="min-card-actions"><button class="min-btn min-btn-primary" onclick="window._minutes.startMeeting(\''+m.id+'\')">Start Meeting</button></div>'
            +'</div>';
    }).join('');
}

function renderTranscripts(){
    var grid=document.getElementById('min-transcripts-grid');
    if(!grid)return;
    var list=filteredTranscripts.length>0?filteredTranscripts:transcripts;
    if(list.length===0){grid.innerHTML='<div class="min-empty"><div class="min-empty-icon">📝</div><h3>No Transcripts</h3><p>Meeting transcripts will appear here</p></div>';return}
    grid.innerHTML=list.map(function(t){
        return '<div class="min-card">'
            +'<div class="min-card-header"><span class="min-card-title">'+(t.meeting_title||'')+'</span><span class="min-badge active">'+(t.status||'')+'</span></div>'
            +'<div class="min-card-body">'
            +'<div class="min-card-row"><span class="label">Date</span><span>'+(t.date||'')+'</span></div>'
            +'<div class="min-card-row"><span class="label">Duration</span><span>'+(t.duration||'')+'</span></div>'
            +'<div class="min-card-row"><span class="label">Speakers</span><span>'+(t.speakers||0)+'</span></div>'
            +'</div>'
            +'<div class="min-transcript">'+(t.text||'No transcript available')+'</div>'
            +'</div>';
    }).join('');
}

function renderDocuments(){
    var grid=document.getElementById('min-documents-grid');
    if(!grid)return;
    if(documents.length===0){grid.innerHTML='<div class="min-empty"><div class="min-empty-icon">📋</div><h3>No Minutes Documents</h3><p>Generated minutes will appear here</p></div>';return}
    grid.innerHTML=documents.map(function(d){
        var statusClass=d.status==='approved'?'approved':d.status==='signed'?'signed':'draft';
        return '<div class="min-card">'
            +'<div class="min-card-header"><span class="min-card-title">'+(d.title||'')+'</span><span class="min-badge '+statusClass+'">'+(d.status||'draft')+'</span></div>'
            +'<div class="min-card-body">'
            +'<div class="min-card-row"><span class="label">Meeting</span><span>'+(d.meeting_title||'')+'</span></div>'
            +'<div class="min-card-row"><span class="label">Date</span><span>'+(d.date||'')+'</span></div>'
            +'<div class="min-card-row"><span class="label">Version</span><span>v'+(d.version||1)+'</span></div>'
            +'</div>'
            +'<div class="min-card-actions">'
            +'<button class="min-btn" onclick="window._minutes.editMinutes(\''+d.id+'\')">Edit</button>'
            +(d.status!=='approved'?'<button class="min-btn min-btn-success" onclick="window._minutes.approveDoc(\''+d.id+'\')">Approve</button>':'')
            +'</div></div>';
    }).join('');
}

function renderSignDocs(){
    var tbody=document.getElementById('min-sign-tbody');
    if(!tbody)return;
    if(signDocs.length===0){tbody.innerHTML='<tr><td colspan="5" class="min-empty">No documents awaiting signature</td></tr>';return}
    tbody.innerHTML=signDocs.map(function(s){
        var statusClass=s.status==='signed'?'signed':'pending';
        return '<tr>'
            +'<td>'+(s.document_name||'')+'</td>'
            +'<td>'+(s.meeting_title||'')+'</td>'
            +'<td>'+(s.required_signers||[])+'</td>'
            +'<td><span class="min-badge '+statusClass+'">'+s.status+'</span></td>'
            +'<td>'+(s.status==='pending'?'<button class="min-btn min-btn-primary" onclick="window._minutes.openSignPad(\''+s.id+'\')">Sign</button>':'<span style="color:#22c55e">Signed</span>')+'</td>'
            +'</tr>';
    }).join('');
}

async function loadMeetings(){
    var data=await apiCall('/api/minutes/meetings');
    if(data){meetings=Array.isArray(data)?data:[];updateStats();renderMeetings()}
}

async function loadTranscripts(){
    var data=await apiCall('/api/minutes/transcripts');
    if(data){transcripts=Array.isArray(data)?data:[];filteredTranscripts=transcripts;updateStats();renderTranscripts()}
}

async function loadDocuments(){
    var data=await apiCall('/api/minutes/documents');
    if(data){
        var docs=Array.isArray(data)?data:[];
        documents=docs.filter(function(d){return d.type!=='sign'});
        signDocs=docs.filter(function(d){return d.type==='sign'});
        if(signDocs.length===0){
            var signData=await apiCall('/api/minutes/signatures');
            if(signData)signDocs=Array.isArray(signData)?signData:[];
        }
        updateStats();renderDocuments();renderSignDocs();
    }
}

function searchTranscripts(query){
    if(!query){filteredTranscripts=transcripts;renderTranscripts();return}
    var q=query.toLowerCase();
    filteredTranscripts=transcripts.filter(function(t){
        return(t.meeting_title||'').toLowerCase().includes(q)||(t.text||'').toLowerCase().includes(q)
    });
    renderTranscripts();
}

async function startMeeting(meetingId){
    showFeedback('Starting meeting...','success');
    var data=await apiCall('/api/minutes/meetings/'+meetingId+'/start',{method:'POST'});
    if(data){showFeedback('Meeting started','success');loadMeetings()}
}

function editMinutes(docId){
    selectedDocId=docId;
    var doc=documents.find(function(d){return d.id===docId});
    if(!doc)return;
    var title=document.getElementById('min-edit-title');
    var content=document.getElementById('min-edit-content');
    if(title)title.textContent='Edit: '+(doc.title||'');
    if(content)content.value=doc.content||'';
    showModal('min-edit-modal');
}

async function saveMinutes(){
    if(!selectedDocId)return;
    var content=document.getElementById('min-edit-content').value;
    var data=await apiCall('/api/minutes/documents/'+selectedDocId,{method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify({content:content})});
    if(data){hideModal('min-edit-modal');showFeedback('Minutes saved','success');loadDocuments()}
}

async function approveMinutes(){
    if(!selectedDocId)return;
    var data=await apiCall('/api/minutes/documents/'+selectedDocId+'/approve',{method:'POST'});
    if(data){hideModal('min-edit-modal');showFeedback('Minutes approved','success');loadDocuments()}
}

async function approveDoc(docId){
    var data=await apiCall('/api/minutes/documents/'+docId+'/approve',{method:'POST'});
    if(data){showFeedback('Document approved','success');loadDocuments()}
}

function openSignPad(signId){
    selectedSignId=signId;
    var area=document.getElementById('min-sign-pad-area');
    if(area)area.style.display='block';
    var canvas=document.getElementById('min-sign-canvas');
    if(!canvas)return;
    var ctx=canvas.getContext('2d');
    ctx.fillStyle='#0f172a';
    ctx.fillRect(0,0,canvas.width,canvas.height);
    ctx.strokeStyle='#f8fafc';
    ctx.lineWidth=2;
    ctx.lineCap='round';
    canvas.onmousedown=function(e){signDrawing=true;signLastX=e.offsetX;signLastY=e.offsetY};
    canvas.onmousemove=function(e){if(!signDrawing)return;ctx.beginPath();ctx.moveTo(signLastX,signLastY);ctx.lineTo(e.offsetX,e.offsetY);ctx.stroke();signLastX=e.offsetX;signLastY=e.offsetY};
    canvas.onmouseup=function(){signDrawing=false};
    canvas.onmouseleave=function(){signDrawing=false};
}

function clearSignPad(){
    var canvas=document.getElementById('min-sign-canvas');
    if(!canvas)return;
    var ctx=canvas.getContext('2d');
    ctx.fillStyle='#0f172a';
    ctx.fillRect(0,0,canvas.width,canvas.height);
}

async function submitSign(){
    if(!selectedSignId)return;
    var canvas=document.getElementById('min-sign-canvas');
    var sigData=canvas?canvas.toDataURL('image/png'):'';
    var data=await apiCall('/api/minutes/signatures/'+selectedSignId+'/sign',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({signature:sigData})});
    if(data){
        showFeedback('Document signed','success');
        selectedSignId=null;
        var area=document.getElementById('min-sign-pad-area');
        if(area)area.style.display='none';
        loadDocuments();
    }
}

function showModal(id){var el=document.getElementById(id);if(el)el.style.display='flex'}
function hideModal(id){var el=document.getElementById(id);if(el)el.style.display='none'}

function loadAll(){loadMeetings();loadTranscripts();loadDocuments()}

window._minutes={switchTab:switchTab,searchTranscripts:searchTranscripts,startMeeting:startMeeting,editMinutes:editMinutes,saveMinutes:saveMinutes,approveMinutes:approveMinutes,approveDoc:approveDoc,openSignPad:openSignPad,clearSignPad:clearSignPad,submitSign:submitSign,showModal:showModal,hideModal:hideModal,loadAll:loadAll};
loadAll();
})();
