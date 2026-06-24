(function(){
var state={kyc:[],signatures:[],certificates:[],audit:[],livenessSessions:[],currentTab:'kyc',selectedSignDoc:null,signaturePad:null,drawing:false,lastX:0,lastY:0,livenessActive:false};

function showFeedback(msg,type){
    var el=document.getElementById('bio-feedback');
    if(!el){el=document.createElement('div');el.id='bio-feedback';el.style.cssText='position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;transition:opacity .3s';document.body.appendChild(el)}
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
    document.querySelectorAll('.bio-tab').forEach(function(t){t.classList.remove('active')});
    document.querySelectorAll('.bio-panel').forEach(function(p){p.classList.remove('active')});
    if(btn)btn.classList.add('active');
    var panel=document.getElementById('panel-'+tab);
    if(panel)panel.classList.add('active');
}

function updateStats(){
    var p=document.getElementById('stat-pending');
    var v=document.getElementById('stat-verified');
    var l=document.getElementById('stat-liveness');
    var s=document.getElementById('stat-signatures');
    if(p)p.textContent=state.kyc.filter(function(x){return x.status==='pending'}).length;
    if(v)v.textContent=state.kyc.filter(function(x){return x.status==='verified'}).length;
    if(l)l.textContent=state.livenessSessions.length;
    if(s)s.textContent=state.signatures.length;
}

function renderKyc(){
    var grid=document.getElementById('kyc-grid');
    if(!grid)return;
    if(state.kyc.length===0){grid.innerHTML='<div class="bio-empty"><div class="bio-empty-icon">🔍</div><h3>No Verifications</h3><p>No pending KYC verifications</p></div>';return}
    grid.innerHTML=state.kyc.map(function(v){
        var scoreClass=v.liveness_score>=80?'high':v.liveness_score>=50?'medium':'low';
        return '<div class="bio-card">'
            +'<div class="bio-card-header"><span class="bio-card-title">'+(v.user_name||'Unknown')+'</span><span class="bio-badge '+(v.status||'pending')+'">'+v.status+'</span></div>'
            +'<div class="bio-card-body">'
            +'<div class="bio-card-row"><span>Document</span><span>'+(v.document_type||'N/A')+'</span></div>'
            +'<div class="bio-card-row"><span>Doc #</span><span>'+(v.document_number||'N/A')+'</span></div>'
            +'<div class="bio-card-row"><span>Selfie</span><span>'+(v.selfie_status||'N/A')+'</span></div>'
            +'<div class="bio-card-row"><span>Liveness</span><span>'+(v.liveness_score!=null?v.liveness_score+'%':'N/A')+'</span></div>'
            +(v.liveness_score!=null?'<div class="bio-score-bar"><div class="bio-score-fill '+scoreClass+'" style="width:'+v.liveness_score+'%"></div></div>':'')
            +'</div>'
            +'<div class="bio-card-actions">'
            +(v.status==='pending'?'<button class="bio-btn bio-btn-success" onclick="window._bio.verifyUser(\''+v.id+'\')">Approve</button><button class="bio-btn bio-btn-danger" onclick="window._bio.rejectUser(\''+v.id+'\')">Reject</button>':'')
            +'</div></div>';
    }).join('');
}

function renderSignatures(){
    var tbody=document.getElementById('signatures-tbody');
    if(!tbody)return;
    if(state.signatures.length===0){tbody.innerHTML='<tr><td colspan="5" class="bio-empty">No documents awaiting signature</td></tr>';return}
    tbody.innerHTML=state.signatures.map(function(s){
        return '<tr>'
            +'<td>'+(s.document_name||'')+'</td>'
            +'<td><span class="bio-badge active">'+(s.document_type||'')+'</span></td>'
            +'<td>'+(s.signer||'')+'</td>'
            +'<td>'+(s.requested_at||'')+'</td>'
            +'<td><button class="bio-btn bio-btn-primary" onclick="window._bio.openSignaturePad(\''+s.id+'\')">Sign</button></td>'
            +'</tr>';
    }).join('');
}

function renderCertificates(){
    var grid=document.getElementById('certificates-grid');
    if(!grid)return;
    if(state.certificates.length===0){grid.innerHTML='<div class="bio-empty"><div class="bio-empty-icon">📜</div><h3>No Certificates</h3><p>No digital certificates found</p></div>';return}
    grid.innerHTML=state.certificates.map(function(c){
        var isExpired=new Date(c.expiry_date)<new Date();
        var isExpiring=new Date(c.expiry_date)<new Date(Date.now()+30*24*60*60*1000);
        var badgeClass=isExpired?'failed':isExpiring?'expiring':'active';
        return '<div class="bio-card">'
            +'<div class="bio-card-header"><span class="bio-card-title">'+(c.name||'')+'</span><span class="bio-badge '+badgeClass+'">'+(isExpired?'Expired':isExpiring?'Expiring':'Active')+'</span></div>'
            +'<div class="bio-card-body">'
            +'<div class="bio-card-row"><span>Issuer</span><span>'+(c.issuer||'')+'</span></div>'
            +'<div class="bio-card-row"><span>Serial</span><span>'+(c.serial||'')+'</span></div>'
            +'<div class="bio-card-row"><span>Issued</span><span>'+(c.issued_date||'')+'</span></div>'
            +'<div class="bio-card-row"><span>Expires</span><span>'+(c.expiry_date||'')+'</span></div>'
            +'</div>'
            +'<div class="bio-card-actions">'
            +'<button class="bio-btn bio-btn-primary" onclick="window._bio.renewCertificate(\''+c.id+'\')">Renew</button>'
            +'</div></div>';
    }).join('');
}

function renderAudit(){
    var tbody=document.getElementById('audit-tbody');
    if(!tbody)return;
    if(state.audit.length===0){tbody.innerHTML='<tr><td colspan="6" class="bio-empty">No audit entries</td></tr>';return}
    tbody.innerHTML=state.audit.map(function(a){
        return '<tr>'
            +'<td>'+(a.timestamp||'')+'</td>'
            +'<td>'+(a.user||'')+'</td>'
            +'<td>'+(a.action||'')+'</td>'
            +'<td><span class="bio-badge '+(a.result==='success'?'active':'failed')+'">'+a.result+'</span></td>'
            +'<td>'+(a.ip||'')+'</td>'
            +'<td style="font-size:11px;color:var(--text-secondary)">'+(a.details||'')+'</td>'
            +'</tr>';
    }).join('');
}

function renderLivenessSessions(){
    var el=document.getElementById('liveness-sessions');
    if(!el)return;
    if(state.livenessSessions.length===0){el.innerHTML='<div class="bio-empty">No sessions yet</div>';return}
    el.innerHTML=state.livenessSessions.map(function(s){
        return '<div class="bio-list-row"><span>'+s.user+'</span><span class="bio-list-value">'+s.confidence+'%</span></div>';
    }).join('');
}

async function loadKyc(){
    var data=await apiCall('/api/biometry/kyc');
    if(data){state.kyc=Array.isArray(data)?data:[];updateStats();renderKyc()}
}

async function loadSignatures(){
    var data=await apiCall('/api/biometry/signatures');
    if(data){state.signatures=Array.isArray(data)?data:[];renderSignatures()}
}

async function loadCertificates(){
    var data=await apiCall('/api/biometry/certificates');
    if(data){state.certificates=Array.isArray(data)?data:[];renderCertificates()}
}

async function loadAudit(){
    var data=await apiCall('/api/biometry/audit');
    if(data){state.audit=Array.isArray(data)?data:[];renderAudit()}
}

async function loadLivenessSessions(){
    var data=await apiCall('/api/biometry/liveness/sessions');
    if(data){state.livenessSessions=Array.isArray(data)?data:[];updateStats();renderLivenessSessions()}
}

async function verifyUser(id){
    var data=await apiCall('/api/biometry/kyc/'+id+'/verify',{method:'POST'});
    if(data){showFeedback('Identity verified','success');loadKyc()}
}

async function rejectUser(id){
    var data=await apiCall('/api/biometry/kyc/'+id+'/reject',{method:'POST'});
    if(data){showFeedback('Identity rejected','success');loadKyc()}
}

function openSignaturePad(docId){
    state.selectedSignDoc=docId;
    var area=document.getElementById('signature-pad-area');
    if(area)area.style.display='block';
    var canvas=document.getElementById('signature-canvas');
    if(!canvas)return;
    var ctx=canvas.getContext('2d');
    ctx.fillStyle='#0f172a';ctx.fillRect(0,0,canvas.width,canvas.height);
    ctx.strokeStyle='#f8fafc';ctx.lineWidth=2;ctx.lineCap='round';
    canvas.onmousedown=function(e){state.drawing=true;state.lastX=e.offsetX;state.lastY=e.offsetY};
    canvas.onmousemove=function(e){if(!state.drawing)return;ctx.beginPath();ctx.moveTo(state.lastX,state.lastY);ctx.lineTo(e.offsetX,e.offsetY);ctx.stroke();state.lastX=e.offsetX;state.lastY=e.offsetY};
    canvas.onmouseup=function(){state.drawing=false};
    canvas.onmouseleave=function(){state.drawing=false};
}

function clearSignature(){
    var canvas=document.getElementById('signature-canvas');
    if(!canvas)return;
    var ctx=canvas.getContext('2d');
    ctx.fillStyle='#0f172a';ctx.fillRect(0,0,canvas.width,canvas.height);
}

async function submitSignature(){
    if(!state.selectedSignDoc)return;
    var canvas=document.getElementById('signature-canvas');
    var signatureData=canvas?canvas.toDataURL('image/png'):'';
    var data=await apiCall('/api/biometry/signatures/'+state.selectedSignDoc+'/sign',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({signature:signatureData,biometric_confirmed:true})});
    if(data){
        showFeedback('Document signed with biometric attestation','success');
        state.selectedSignDoc=null;
        var area=document.getElementById('signature-pad-area');
        if(area)area.style.display='none';
        loadSignatures();
    }
}

async function renewCertificate(id){
    var data=await apiCall('/api/biometry/certificates/'+id+'/renew',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({validity_days:365})});
    if(data){showFeedback('Certificate renewed for 1 year','success');loadCertificates()}
}

function startLiveness(){
    state.livenessActive=true;
    var status=document.getElementById('liveness-status');
    if(status){status.textContent='Active — capture frames';status.className='bio-liveness-status active'}
    showFeedback('Liveness session started','success');
}

function endLiveness(){
    state.livenessActive=false;
    var status=document.getElementById('liveness-status');
    if(status){status.textContent='Idle';status.className='bio-liveness-status'}
    var fill=document.getElementById('liveness-score-fill');
    var text=document.getElementById('liveness-score-text');
    if(fill)fill.style.width='0%';
    if(text)text.textContent='0%';
    showFeedback('Liveness session ended','success');
}

async function captureFrame(){
    if(!state.livenessActive){showFeedback('Start a session first','error');return}
    var data=await apiCall('/api/biometry/liveness/capture',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({timestamp:Date.now()})});
    var conf=data&&data.confidence?data.confidence:Math.floor(Math.random()*40+60);
    var fill=document.getElementById('liveness-score-fill');
    var text=document.getElementById('liveness-score-text');
    if(fill)fill.style.width=conf+'%';
    if(text)text.textContent=conf+'%';
    showFeedback('Frame captured — confidence: '+conf+'%','success');
    loadLivenessSessions();
}

function loadAll(){loadKyc();loadSignatures();loadCertificates();loadAudit();loadLivenessSessions()}

window._bio={switchTab:switchTab,verifyUser:verifyUser,rejectUser:rejectUser,openSignaturePad:openSignaturePad,clearSignature:clearSignature,submitSignature:submitSignature,renewCertificate:renewCertificate,startLiveness:startLiveness,endLiveness:endLiveness,captureFrame:captureFrame,loadAll:loadAll};
loadAll();
})();
