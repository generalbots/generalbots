if (window.GBAppLifecycle) GBAppLifecycle.begin("kyc");
(function(){
var verifications=[];
var signatures=[];
var certificates=[];
var currentTab='verifications';
var selectedSignDoc=null;
var signaturePad=null;
var drawing=false;
var lastX=0;
var lastY=0;

function showFeedback(msg,type){
    var el=document.getElementById('kyc-feedback');
    if(!el){el=document.createElement('div');el.id='kyc-feedback';el.style.cssText='position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;transition:opacity .3s';document.body.appendChild(el)}
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
    document.querySelectorAll('.kyc-tab').forEach(function(t){t.classList.remove('active')});
    document.querySelectorAll('.kyc-panel').forEach(function(p){p.classList.remove('active')});
    if(btn)btn.classList.add('active');
    var panel=document.getElementById('panel-'+tab);
    if(panel)panel.classList.add('active');
}

function updateStats(){
    var pending=verifications.filter(function(v){return v.status==='pending'}).length;
    var verified=verifications.filter(function(v){return v.status==='verified'}).length;
    var failed=verifications.filter(function(v){return v.status==='failed'}).length;
    var elP=document.getElementById('stat-pending');
    var elV=document.getElementById('stat-verified');
    var elF=document.getElementById('stat-failed');
    var elT=document.getElementById('stat-total');
    if(elP)elP.textContent=pending;
    if(elV)elV.textContent=verified;
    if(elF)elF.textContent=failed;
    if(elT)elT.textContent=verifications.length;
}

function renderVerifications(){
    var grid=document.getElementById('verifications-grid');
    if(!grid)return;
    if(verifications.length===0){grid.innerHTML='<div class="kyc-empty"><div class="kyc-empty-icon">🔍</div><h3>No Verifications</h3><p>No pending verifications at this time</p></div>';return}
    grid.innerHTML=verifications.map(function(v){
        var scoreClass=v.liveness_score>=80?'high':v.liveness_score>=50?'medium':'low';
        var statusClass=v.status==='pending'?'pending':v.status==='verified'?'verified':'failed';
        return '<div class="kyc-card">'
            +'<div class="kyc-card-header"><span class="kyc-card-title">'+(v.user_name||'Unknown')+'</span><span class="kyc-badge '+statusClass+'">'+v.status+'</span></div>'
            +'<div class="kyc-card-body">'
            +'<div class="kyc-card-row"><span class="label">Document</span><span>'+(v.document_type||'N/A')+'</span></div>'
            +'<div class="kyc-card-row"><span class="label">Document #</span><span>'+(v.document_number||'N/A')+'</span></div>'
            +'<div class="kyc-card-row"><span class="label">Selfie</span><span>'+(v.selfie_status||'N/A')+'</span></div>'
            +'<div class="kyc-card-row"><span class="label">Liveness Score</span><span>'+(v.liveness_score!=null?v.liveness_score+'%':'N/A')+'</span></div>'
            +(v.liveness_score!=null?'<div class="kyc-score-bar"><div class="kyc-score-fill '+scoreClass+'" style="width:'+v.liveness_score+'%"></div></div>':'')
            +'</div>'
            +'<div class="kyc-card-actions">'
            +(v.status==='pending'?'<button class="kyc-btn kyc-btn-success" onclick="window._kyc.verifyUser(\''+v.id+'\')">Approve</button><button class="kyc-btn kyc-btn-danger" onclick="window._kyc.rejectUser(\''+v.id+'\')">Reject</button>':'')
            +'</div></div>';
    }).join('');
}

function renderSignatures(){
    var tbody=document.getElementById('signatures-tbody');
    if(!tbody)return;
    if(signatures.length===0){tbody.innerHTML='<tr><td colspan="5" class="kyc-empty">No documents awaiting signature</td></tr>';return}
    tbody.innerHTML=signatures.map(function(s){
        return '<tr>'
            +'<td>'+(s.document_name||'')+'</td>'
            +'<td><span class="kyc-badge active">'+(s.document_type||'')+'</span></td>'
            +'<td>'+(s.requested_at||'')+'</td>'
            +'<td>'+(s.signer||'')+'</td>'
            +'<td><button class="kyc-btn kyc-btn-primary" onclick="window._kyc.openSignaturePad(\''+s.id+'\')">Sign</button></td>'
            +'</tr>';
    }).join('');
}

function renderCertificates(){
    var grid=document.getElementById('certificates-grid');
    if(!grid)return;
    if(certificates.length===0){grid.innerHTML='<div class="kyc-empty"><div class="kyc-empty-icon">📜</div><h3>No Certificates</h3><p>No digital certificates found</p></div>';return}
    grid.innerHTML=certificates.map(function(c){
        var isExpired=new Date(c.expiry_date)<new Date();
        var isExpiring=new Date(c.expiry_date)<new Date(Date.now()+30*24*60*60*1000);
        var badgeClass=isExpired?'failed':isExpiring?'expiring':'active';
        return '<div class="kyc-card">'
            +'<div class="kyc-card-header"><span class="kyc-card-title">'+(c.name||'')+'</span><span class="kyc-badge '+badgeClass+'">'+(isExpired?'Expired':isExpiring?'Expiring':'Active')+'</span></div>'
            +'<div class="kyc-card-body">'
            +'<div class="kyc-card-row"><span class="label">Issuer</span><span>'+(c.issuer||'')+'</span></div>'
            +'<div class="kyc-card-row"><span class="label">Serial</span><span>'+(c.serial||'')+'</span></div>'
            +'<div class="kyc-card-row"><span class="label">Issued</span><span>'+(c.issued_date||'')+'</span></div>'
            +'<div class="kyc-card-row"><span class="label">Expires</span><span>'+(c.expiry_date||'')+'</span></div>'
            +'</div>'
            +'<div class="kyc-card-actions">'
            +'<button class="kyc-btn kyc-btn-primary" onclick="window._kyc.showRenewModal(\''+c.id+'\')">Renew</button>'
            +'</div></div>';
    }).join('');
}

async function loadVerifications(){
    var data=await apiCall('/api/kyc/verifications');
    if(data){verifications=Array.isArray(data)?data:[];updateStats();renderVerifications()}
}

async function loadSignatures(){
    var data=await apiCall('/api/kyc/signatures');
    if(data){signatures=Array.isArray(data)?data:[];renderSignatures()}
}

async function loadCertificates(){
    var data=await apiCall('/api/kyc/certificates');
    if(data){certificates=Array.isArray(data)?data:[];renderCertificates()}
}

async function verifyUser(id){
    var data=await apiCall('/api/kyc/verifications/'+id+'/verify',{method:'POST'});
    if(data){showFeedback('User verified','success');loadVerifications()}
}

async function rejectUser(id){
    var data=await apiCall('/api/kyc/verifications/'+id+'/reject',{method:'POST'});
    if(data){showFeedback('User rejected','success');loadVerifications()}
}

function openSignaturePad(docId){
    selectedSignDoc=docId;
    var area=document.getElementById('signature-pad-area');
    if(area)area.style.display='block';
    var canvas=document.getElementById('signature-canvas');
    if(!canvas)return;
    var ctx=canvas.getContext('2d');
    ctx.fillStyle='#0f172a';
    ctx.fillRect(0,0,canvas.width,canvas.height);
    ctx.strokeStyle='#f8fafc';
    ctx.lineWidth=2;
    ctx.lineCap='round';
    canvas.onmousedown=function(e){drawing=true;lastX=e.offsetX;lastY=e.offsetY};
    canvas.onmousemove=function(e){if(!drawing)return;ctx.beginPath();ctx.moveTo(lastX,lastY);ctx.lineTo(e.offsetX,e.offsetY);ctx.stroke();lastX=e.offsetX;lastY=e.offsetY};
    canvas.onmouseup=function(){drawing=false};
    canvas.onmouseleave=function(){drawing=false};
}

function clearSignature(){
    var canvas=document.getElementById('signature-canvas');
    if(!canvas)return;
    var ctx=canvas.getContext('2d');
    ctx.fillStyle='#0f172a';
    ctx.fillRect(0,0,canvas.width,canvas.height);
}

async function submitSignature(){
    if(!selectedSignDoc)return;
    var canvas=document.getElementById('signature-canvas');
    var signatureData=canvas?canvas.toDataURL('image/png'):'';
    var data=await apiCall('/api/kyc/signatures/'+selectedSignDoc+'/sign',{
        method:'POST',
        headers:{'Content-Type':'application/json'},
        body:JSON.stringify({signature:signatureData})
    });
    if(data){
        showFeedback('Document signed','success');
        selectedSignDoc=null;
        var area=document.getElementById('signature-pad-area');
        if(area)area.style.display='none';
        loadSignatures();
    }
}

function showRenewModal(certId){
    var input=document.getElementById('renew-cert-id');
    if(input)input.value=certId;
    showModal('renew-modal');
}

async function renewCertificate(){
    var certId=document.getElementById('renew-cert-id').value;
    var validity=document.getElementById('renew-validity').value;
    var data=await apiCall('/api/kyc/certificates/'+certId+'/renew',{
        method:'POST',
        headers:{'Content-Type':'application/json'},
        body:JSON.stringify({validity_days:parseInt(validity)})
    });
    if(data){hideModal('renew-modal');showFeedback('Certificate renewed','success');loadCertificates()}
}

function showModal(id){var el=document.getElementById(id);if(el)el.classList.add('visible')}
function hideModal(id){var el=document.getElementById(id);if(el)el.classList.remove('visible')}

function loadAll(){loadVerifications();loadSignatures();loadCertificates()}

window._kyc={switchTab:switchTab,verifyUser:verifyUser,rejectUser:rejectUser,openSignaturePad:openSignaturePad,clearSignature:clearSignature,submitSignature:submitSignature,showRenewModal:showRenewModal,renewCertificate:renewCertificate,showModal:showModal,hideModal:hideModal,loadVerifications:loadVerifications,loadSignatures:loadSignatures,loadCertificates:loadCertificates,loadAll:loadAll};
loadAll();
})();
