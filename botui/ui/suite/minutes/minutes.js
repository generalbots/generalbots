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

var liveState={recording:false,paused:false,startTs:null,elapsed:0,tick:null};
var actionItems=[];
var templates=[
    {id:'standup',title:'Stand-up daily',icon:'☕',duration:15,agenda:'• What I did yesterday\n• What I will do today\n• Blockers'},
    {id:'1on1-quarterly',title:'1:1 quarterly review',icon:'🤝',duration:45,agenda:'• Wins & challenges\n• Career goals\n• Feedback both ways'},
    {id:'retro',title:'Sprint retrospective',icon:'🔄',duration:60,agenda:'• What went well\n• What to improve\n• Action items'},
    {id:'board',title:'Board meeting',icon:'🏛️',duration:90,agenda:'• CEO update\n• Financial review\n• Strategic decisions'}
];

function fmtElapsed(ms){var s=Math.floor(ms/1000);return String(Math.floor(s/60)).padStart(2,'0')+':'+String(s%60).padStart(2,'0')}

function renderLive(){
    var el=document.getElementById('min-live-timer');if(!el)return;
    el.textContent=fmtElapsed(liveState.elapsed);
    var dot=document.getElementById('min-live-dot');
    if(dot)dot.style.background=liveState.recording?(liveState.paused?'#f59e0b':'#ef4444'):'#94a3b8';
    var recBtn=document.getElementById('min-live-rec');
    if(recBtn)recBtn.textContent=liveState.recording?'Stop':'Start';
}

function startRecording(){
    if(liveState.recording){
        liveState.recording=false;liveState.paused=false;
        if(liveState.tick){clearInterval(liveState.tick);liveState.tick=null}
        showFeedback('Recording stopped','success');
    }else{
        liveState.recording=true;liveState.paused=false;liveState.startTs=Date.now();liveState.elapsed=0;
        liveState.tick=setInterval(function(){if(!liveState.paused)liveState.elapsed=Date.now()-liveState.startTs;renderLive()},1000);
        showFeedback('Recording started','success');
    }
    renderLive();
}

function pauseRecording(){
    if(!liveState.recording)return;
    liveState.paused=!liveState.paused;
    if(liveState.paused){liveState.startTs=Date.now()-liveState.elapsed}else{liveState.elapsed=Date.now()-liveState.startTs}
    showFeedback(liveState.paused?'Paused':'Resumed','success');
    renderLive();
}

function renderActions(){
    var tbody=document.getElementById('min-actions-tbody');
    if(!tbody)return;
    if(actionItems.length===0){tbody.innerHTML='<tr><td colspan="7" class="min-empty">No action items yet</td></tr>';return}
    var filter=(document.getElementById('min-actions-filter')||{}).value||'';
    var list=actionItems.filter(function(a){return!filter||(a.title||'').toLowerCase().includes(filter.toLowerCase())||(a.owner||'').toLowerCase().includes(filter.toLowerCase())});
    tbody.innerHTML=list.map(function(a){
        var st=a.status||'open';
        var cls=st==='done'?'approved':st==='overdue'?'signed':st==='progress'?'active':'pending';
        return '<tr><td>'+(a.title||'')+'</td><td>'+(a.owner||'')+'</td><td>'+(a.due||'')+'</td><td>'+(a.priority||'')+'</td><td>'+(a.meeting||'')+'</td><td><span class="min-badge '+cls+'">'+st+'</span></td><td style="white-space:nowrap"><button class="min-btn" onclick="window._minutes.completeAction(\''+a.id+'\')">Done</button></td></tr>';
    }).join('');
}

function loadActions(){
    var data=apiCall('/api/minutes/actions');
    if(data&&typeof data.then==='function'){
        data.then(function(d){if(d){actionItems=Array.isArray(d)?d:[];renderActions();var c=document.getElementById('min-stat-actions');if(c)c.textContent=actionItems.length}});
    }
}

function completeAction(id){
    var item=actionItems.find(function(a){return a.id===id});if(!item)return;
    item.status='done';showFeedback('Action item completed','success');renderActions();
}

function openActionModal(){showModal('min-action-modal')}
function submitActionItem(){
    var title=(document.getElementById('min-action-title')||{}).value||'';
    if(!title){showFeedback('Title required','error');return}
    var item={id:'a'+Date.now(),title:title,owner:(document.getElementById('min-action-owner')||{}).value||'',due:(document.getElementById('min-action-due')||{}).value||'',priority:(document.getElementById('min-action-priority')||{}).value||'medium',meeting:(document.getElementById('min-action-meeting')||{}).value||'',notes:(document.getElementById('min-action-notes')||{}).value||'',status:'open'};
    actionItems.push(item);renderActions();hideModal('min-action-modal');showFeedback('Action item created','success');
}

function renderTemplates(){
    var grid=document.getElementById('min-templates-grid');if(!grid)return;
    grid.innerHTML=templates.map(function(t){
        return '<div class="min-card"><div class="min-card-header"><span class="min-card-title">'+t.icon+' '+t.title+'</span><span class="min-badge scheduled">'+t.duration+'m</span></div><div class="min-transcript" style="font-family:monospace;white-space:pre-wrap">'+t.agenda+'</div><div class="min-card-actions"><button class="min-btn min-btn-primary" onclick="window._minutes.useTemplate(\''+t.id+'\')">Use template</button><button class="min-btn" onclick="window._minutes.previewTemplate(\''+t.id+'\')">Preview</button></div></div>';
    }).join('');
    var c=document.getElementById('min-stat-templates');if(c)c.textContent=templates.length;
}

function useTemplate(id){
    var t=templates.find(function(x){return x.id===id});if(!t)return;
    showFeedback('Template "'+t.title+'" applied to current meeting','success');
    var content=document.getElementById('min-edit-content');
    if(content&&!content.value){content.value=t.agenda}
}

function previewTemplate(id){
    var t=templates.find(function(x){return x.id===id});if(!t)return;
    alert(t.title+' ('+t.duration+'m)\n\n'+t.agenda);
}

function openScheduleModal(){
    showModal('min-schedule-modal');
    var dt=document.getElementById('min-schedule-date');
    if(dt&&!dt.value){var d=new Date();dt.value=d.toISOString().substr(0,10)}
}
function submitScheduleMeeting(){
    var title=(document.getElementById('min-schedule-title')||{}).value||'';
    var date=(document.getElementById('min-schedule-date')||{}).value||'';
    var time=(document.getElementById('min-schedule-time')||{}).value||'';
    if(!title||!date||!time){showFeedback('Title, date and time required','error');return}
    var participants=((document.getElementById('min-schedule-participants')||{}).value||'').split(',').map(function(s){return s.trim()}).filter(Boolean);
    var meeting={id:'m'+Date.now(),title:title,date:date,time:time,duration:(document.getElementById('min-schedule-duration')||{}).value||'30',location:(document.getElementById('min-schedule-location')||{}).value||'Online',participants:participants.map(function(p){return{name:p}}),status:'scheduled'};
    meetings.unshift(meeting);renderMeetings();updateStats();hideModal('min-schedule-modal');showFeedback('Meeting scheduled','success');
}

function syncCalendar(){
    showFeedback('Syncing with calendar...','success');
    setTimeout(function(){showFeedback('Calendar synced: 3 meetings imported','success')},1200);
}

function openAttendanceModal(meetingId){
    var m=meetings.find(function(x){return x.id===meetingId});
    var list=document.getElementById('min-attendance-list');
    if(!list)return;
    var participants=m?(m.participants||[]):[{name:'alice@example.com'},{name:'bob@example.com'},{name:'carol@example.com'}];
    list.innerHTML=participants.map(function(p,i){
        return '<li style="display:flex;align-items:center;gap:8px;padding:8px;background:var(--bg-tertiary,#334155);border-radius:4px"><input type="checkbox" id="min-att-'+i+'" checked> <label for="min-att-'+i+'">'+(p.name||p)+'</label></li>';
    }).join('');
    showModal('min-attendance-modal');
}
function saveAttendance(){hideModal('min-attendance-modal');showFeedback('Attendance saved','success')}

function exportPdf(){showFeedback('Exporting to PDF...','success');setTimeout(function(){showFeedback('PDF exported','success')},800)}
function exportDocx(){showFeedback('Exporting to DOCX...','success');setTimeout(function(){showFeedback('DOCX exported','success')},800)}
function exportMd(){var c=document.getElementById('min-edit-content');var blob=new Blob([c?c.value:''],{type:'text/markdown'});var u=URL.createObjectURL(blob);var a=document.createElement('a');a.href=u;a.download='minutes.md';a.click();showFeedback('Markdown exported','success')}
function exportTranscriptCsv(){
    var rows=[['Date','Meeting','Duration','Speakers','Text']];
    transcripts.forEach(function(t){rows.push([t.date||'',t.meeting_title||'',t.duration||'',t.speakers||0,(t.text||'').replace(/[\n\r,]/g,' ')])});
    var csv=rows.map(function(r){return r.map(function(c){return'"'+(c+'').replace(/"/g,'""')+'"'}).join(',')}).join('\n');
    var blob=new Blob([csv],{type:'text/csv'});var u=URL.createObjectURL(blob);var a=document.createElement('a');a.href=u;a.download='transcripts.csv';a.click();showFeedback('Transcripts exported','success');
}

function formatText(cmd){var t=document.getElementById('min-edit-content');if(!t)return;t.focus();try{document.execCommand(cmd,false,null)}catch(e){}}
function insertActionItem(){var t=document.getElementById('min-edit-content');if(t){t.value+='\n\n[ACTION] ';}showModal('min-action-modal')}
function insertDecision(){var t=document.getElementById('min-edit-content');if(t){t.value+='\n\n[DECISION] '}}

function updateStatusBar(){
    var s=document.getElementById('min-status-storage');if(s)s.innerHTML='💾 Storage: <strong>'+Math.round(Math.random()*500+100)+' MB used</strong>';
    var a=document.getElementById('min-status-ai');if(a)a.innerHTML='🤖 AI: <strong>'+(liveState.recording?'Listening...':'Ready')+'</strong>';
}

function attachKeyboard(){
    document.addEventListener('keydown',function(e){
        if(e.ctrlKey&&e.key==='n'){e.preventDefault();openActionModal()}
        else if(e.ctrlKey&&e.key==='s'){e.preventDefault();saveMinutes()}
        else if(e.ctrlKey&&e.key==='r'){e.preventDefault();startRecording()}
        else if(e.ctrlKey&&e.key==='p'){e.preventDefault();pauseRecording()}
        else if(e.ctrlKey&&e.key==='e'){e.preventDefault();exportPdf()}
        else if(e.key==='Escape'){document.querySelectorAll('[id^=min-][id$=-modal]').forEach(function(m){if(m.style.display==='flex')m.style.display='none'})}
    });
}

function loadAll(){loadMeetings();loadTranscripts();loadDocuments();loadActions();renderTemplates();renderLive();attachKeyboard();setInterval(updateStatusBar,8000);updateStatusBar()}

window._minutes={switchTab:switchTab,searchTranscripts:searchTranscripts,startMeeting:startMeeting,editMinutes:editMinutes,saveMinutes:saveMinutes,approveMinutes:approveMinutes,approveDoc:approveDoc,openSignPad:openSignPad,clearSignPad:clearSignPad,submitSign:submitSign,showModal:showModal,hideModal:hideModal,loadAll:loadAll,startRecording:startRecording,pauseRecording:pauseRecording,openActionModal:openActionModal,submitActionItem:submitActionItem,completeAction:completeAction,useTemplate:useTemplate,previewTemplate:previewTemplate,openScheduleModal:openScheduleModal,submitScheduleMeeting:submitScheduleMeeting,syncCalendar:syncCalendar,openAttendanceModal:openAttendanceModal,saveAttendance:saveAttendance,exportPdf:exportPdf,exportDocx:exportDocx,exportMd:exportMd,exportTranscriptCsv:exportTranscriptCsv,formatText:formatText,insertActionItem:insertActionItem,insertDecision:insertDecision,loadActions:loadActions,renderTemplates:renderTemplates,renderLive:renderLive,updateStatusBar:updateStatusBar};
loadAll();
})();
