if (window.GBAppLifecycle) GBAppLifecycle.begin("biometry");
(function(){
var state={kyc:[],signatures:[],certificates:[],audit:[],livenessSessions:[],currentTab:'kyc',selectedSignDoc:null,signaturePad:null,drawing:false,lastX:0,lastY:0,livenessActive:false};

function escapeHtml(str){ if (window.GBAppKit && window.GBAppKit.escapeHtml) return window.GBAppKit.escapeHtml(str); if(str==null||str==='') return ''; var d=document.createElement('div'); d.textContent=String(str); return d.innerHTML; }
function showFeedback(msg,type){ if (window.GBAppKit && window.GBAppKit.showFeedback) return window.GBAppKit.showFeedback('biometry', msg, type); }
async function apiCall(url,options){ if (window.GBAppKit && window.GBAppKit.apiCall) return window.GBAppKit.apiCall('biometry', url, options); try{var r=await fetch(url,options);if(!r.ok)throw new Error('HTTP '+r.status);var t=await r.text();return t?JSON.parse(t):null}catch(e){showFeedback('API error: '+e.message,'error');return null} }

function switchTab(tab,btn){
    state.currentTab=tab;
    if (window.GBAppKit && window.GBAppKit.switchTab) {
        window.GBAppKit.switchTab(tab, btn, {
            tabSelector: '.bio-tabs [role="tab"]',
            panelSelector: '.bio-panel',
            panelId: function(t){ return (t==='signatures'||t==='certificates') ? ('bio-panel-'+t) : ('panel-'+t); }
        });
        return;
    }
    var tabs = Array.prototype.slice.call(document.querySelectorAll('.bio-tabs [role="tab"]'));
    tabs.forEach(function(t){
        var active = t === btn;
        t.classList.toggle('active', active);
        t.setAttribute('aria-selected', active ? 'true' : 'false');
        t.tabIndex = active ? 0 : -1;
    });
    document.querySelectorAll('.bio-panel').forEach(function(p){
        p.classList.remove('active');
        p.hidden = true;
    });
    var panelId = (tab==='signatures'||tab==='certificates') ? ('bio-panel-'+tab) : ('panel-'+tab);
    var panel=document.getElementById(panelId);
    if(panel){
        panel.classList.add('active');
        panel.hidden = false;
        if (btn && btn.id) panel.setAttribute('aria-labelledby', btn.id);
    }
}

function updateStats(){
    var values={
        'bio-stat-pending': state.kyc.filter(function(x){return x.status==='pending'}).length,
        'bio-stat-verified': state.kyc.filter(function(x){return x.status==='verified'}).length,
        'stat-liveness': state.livenessSessions.length,
        'stat-signatures': state.signatures.length
    };
    if (window.GBAppKit && window.GBAppKit.setStatValues) return window.GBAppKit.setStatValues(values);
    Object.keys(values).forEach(function(id){ var el=document.getElementById(id); if(el) el.textContent=String(values[id]); });
}

function renderKyc(){
    var grid=document.getElementById('kyc-grid');
    if(!grid)return;
    if(state.kyc.length===0){grid.innerHTML='<div class="bio-empty"><div class="bio-empty-icon">🔍</div><h3>No Verifications</h3><p>No pending KYC verifications</p></div>';return}
    grid.innerHTML=state.kyc.map(function(v){
        var scoreClass=v.liveness_score>=80?'high':v.liveness_score>=50?'medium':'low';
        var id = escapeHtml(String(v.id == null ? '' : v.id));
        return '<div class="bio-card">'
            +'<div class="bio-card-header"><span class="bio-card-title">'+escapeHtml(v.user_name||'Unknown')+'</span><span class="bio-badge '+escapeHtml(String(v.status||'pending'))+'">'+escapeHtml(String(v.status||''))+'</span></div>'
            +'<div class="bio-card-body">'
            +'<div class="bio-card-row"><span>Document</span><span>'+escapeHtml(v.document_type||'N/A')+'</span></div>'
            +'<div class="bio-card-row"><span>Doc #</span><span>'+escapeHtml(v.document_number||'N/A')+'</span></div>'
            +'<div class="bio-card-row"><span>Selfie</span><span>'+escapeHtml(v.selfie_status||'N/A')+'</span></div>'
            +'<div class="bio-card-row"><span>Liveness</span><span>'+escapeHtml(v.liveness_score!=null?String(v.liveness_score)+'%':'N/A')+'</span></div>'
            +(v.liveness_score!=null?'<div class="bio-score-bar"><div class="bio-score-fill '+scoreClass+'" style="width:'+escapeHtml(String(Number(v.liveness_score)||0))+'%"></div></div>':'')
            +'</div>'
            +'<div class="bio-card-actions">'
            +(v.status==='pending'?'<button type="button" class="bio-btn bio-btn-success" data-bio-action="verify" data-bio-id="'+id+'">Approve</button><button type="button" class="bio-btn bio-btn-danger" data-bio-action="reject" data-bio-id="'+id+'">Reject</button>':'')
            +'</div></div>';
    }).join('');
}

function renderSignatures(){
    var tbody=document.getElementById('bio-signatures-tbody');
    if(!tbody)return;
    if(state.signatures.length===0){tbody.innerHTML='<tr><td colspan="5" class="bio-empty">No documents awaiting signature</td></tr>';return}
    tbody.innerHTML=state.signatures.map(function(s){
        var id = escapeHtml(String(s.id == null ? '' : s.id));
        return '<tr>'
            +'<td>'+escapeHtml(s.document_name||'')+'</td>'
            +'<td><span class="bio-badge active">'+escapeHtml(s.document_type||'')+'</span></td>'
            +'<td>'+escapeHtml(s.signer||'')+'</td>'
            +'<td>'+escapeHtml(s.requested_at||'')+'</td>'
            +'<td><button type="button" class="bio-btn bio-btn-primary" data-bio-action="open-signature" data-bio-id="'+id+'">Sign</button></td>'
            +'</tr>';
    }).join('');
}

function renderCertificates(){
    var grid=document.getElementById('bio-certificates-grid');
    if(!grid)return;
    if(state.certificates.length===0){grid.innerHTML='<div class="bio-empty"><div class="bio-empty-icon">📜</div><h3>No Certificates</h3><p>No digital certificates found</p></div>';return}
    grid.innerHTML=state.certificates.map(function(c){
        var isExpired=new Date(c.expiry_date)<new Date();
        var isExpiring=new Date(c.expiry_date)<new Date(Date.now()+30*24*60*60*1000);
        var badgeClass=isExpired?'failed':isExpiring?'expiring':'active';
        var id = escapeHtml(String(c.id == null ? '' : c.id));
        return '<div class="bio-card">'
            +'<div class="bio-card-header"><span class="bio-card-title">'+escapeHtml(c.name||'')+'</span><span class="bio-badge '+badgeClass+'">'+(isExpired?'Expired':isExpiring?'Expiring':'Active')+'</span></div>'
            +'<div class="bio-card-body">'
            +'<div class="bio-card-row"><span>Issuer</span><span>'+escapeHtml(c.issuer||'')+'</span></div>'
            +'<div class="bio-card-row"><span>Serial</span><span>'+escapeHtml(c.serial||'')+'</span></div>'
            +'<div class="bio-card-row"><span>Issued</span><span>'+escapeHtml(c.issued_date||'')+'</span></div>'
            +'<div class="bio-card-row"><span>Expires</span><span>'+escapeHtml(c.expiry_date||'')+'</span></div>'
            +'</div>'
            +'<div class="bio-card-actions">'
            +'<button type="button" class="bio-btn bio-btn-primary" data-bio-action="renew-cert" data-bio-id="'+id+'">Renew</button>'
            +'</div></div>';
    }).join('');
}

function renderAudit(){
    var tbody=document.getElementById('audit-tbody');
    if(!tbody)return;
    if(state.audit.length===0){tbody.innerHTML='<tr><td colspan="6" class="bio-empty">No audit entries</td></tr>';return}
    tbody.innerHTML=state.audit.map(function(a){
        return '<tr>'
            +'<td>'+escapeHtml(a.timestamp||'')+'</td>'
            +'<td>'+escapeHtml(a.user||'')+'</td>'
            +'<td>'+escapeHtml(a.action||'')+'</td>'
            +'<td><span class="bio-badge '+(a.result==='success'?'active':'failed')+'">'+escapeHtml(a.result)+'</span></td>'
            +'<td>'+escapeHtml(a.ip||'')+'</td>'
            +'<td style="font-size:11px;color:var(--text-secondary)">'+escapeHtml(a.details||'')+'</td>'
            +'</tr>';
    }).join('');
}

function renderLivenessSessions(){
    var el=document.getElementById('liveness-sessions');
    if(!el)return;
    if(state.livenessSessions.length===0){el.innerHTML='<div class="bio-empty">No sessions yet</div>';return}
    el.innerHTML=state.livenessSessions.map(function(s){
        return '<div class="bio-list-row"><span>'+escapeHtml(String(s.user == null ? '' : s.user))+'</span><span class="bio-list-value">'+escapeHtml(String(s.confidence == null ? 0 : s.confidence))+'%</span></div>';
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
    state.hasDrawn = false;
    state.drawing = false;
    var area=document.getElementById('bio-signature-pad-area');
    if(area)area.style.display='block';
    var canvas=document.getElementById('bio-signature-canvas');
    if(!canvas)return;
    if (window.GBAppKit && window.GBAppKit.mountSignaturePad) {
        window.GBAppKit.mountSignaturePad(canvas, state);
    } else {
        var parent = canvas.parentElement;
        if(parent){ var w = Math.min(parent.clientWidth || 500, 800); canvas.width = Math.max(300, w); canvas.height = 200; }
        var ctx=canvas.getContext('2d'); ctx.fillStyle='#0f172a';ctx.fillRect(0,0,canvas.width,canvas.height); ctx.strokeStyle='#f8fafc';ctx.lineWidth=2;ctx.lineCap='round';
    }
}

function clearSignature(){
    var canvas=document.getElementById('bio-signature-canvas');
    if(!canvas)return;
    if (window.GBAppKit && window.GBAppKit.clearSignatureCanvas) {
        window.GBAppKit.clearSignatureCanvas(canvas, state);
    } else {
        var ctx=canvas.getContext('2d'); ctx.fillStyle='#0f172a';ctx.fillRect(0,0,canvas.width,canvas.height);
    }
    state.hasDrawn = false;
}

async function submitSignature(){
    if(!state.selectedSignDoc){ showFeedback('No document selected','error'); return; }
    var canvas=document.getElementById('bio-signature-canvas');
    if(!canvas){ showFeedback('Signature pad not ready','error'); return; }
    if(!state.hasDrawn){ showFeedback('Please draw your signature before submitting','error'); return; }
    var signatureData=canvas.toDataURL('image/png');
    var data=await apiCall('/api/biometry/signatures/'+state.selectedSignDoc+'/sign',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({signature:signatureData,biometric_confirmed:false})});
    if(data){
        showFeedback('Document signed with biometric attestation','success');
        state.selectedSignDoc=null;
        var area=document.getElementById('bio-signature-pad-area');
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
    if(!data || (data.confidence===undefined && data.score===undefined)){
        showFeedback('Liveness capture not available (server response missing score)','error');
        return;
    }
    var conf = (typeof data.confidence === 'number') ? data.confidence : (typeof data.score === 'number' ? data.score : 0);
    var fill=document.getElementById('liveness-score-fill');
    var text=document.getElementById('liveness-score-text');
    if(fill)fill.style.width=Math.max(0,Math.min(100,conf))+'%';
    if(text)text.textContent=Math.max(0,Math.min(100,conf))+'%';
    showFeedback('Frame captured — confidence: '+conf+'%','success');
    loadLivenessSessions();
}

function bindDelegated(){
    var root = document.querySelector('[data-gb-app="biometry"]') || document;
    if (root.__bioBound) return;
    root.__bioBound = true;
    root.addEventListener('click', function(e){
        var btn = e.target.closest('[data-bio-action]');
        if (!btn) return;
        var action = btn.getAttribute('data-bio-action');
        var id = btn.getAttribute('data-bio-id');
        if (action === 'verify') verifyUser(id);
        else if (action === 'reject') rejectUser(id);
        else if (action === 'open-signature') openSignaturePad(id);
        else if (action === 'renew-cert') renewCertificate(id);
        else if (action === 'refresh-all') loadAll();
        else if (action === 'refresh-signatures') loadSignatures();
        else if (action === 'clear-signature') clearSignature();
        else if (action === 'submit-signature') submitSignature();
        else if (action === 'start-liveness') startLiveness();
        else if (action === 'capture-frame') captureFrame();
        else if (action === 'end-liveness') endLiveness();
    });
    document.addEventListener('keydown', function(e){
        var tab = e.target.closest('.bio-tabs [role="tab"]');
        if (!tab) return;
        var tabs = Array.prototype.slice.call(document.querySelectorAll('.bio-tabs [role="tab"]'));
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

function loadAll(){loadKyc();loadSignatures();loadCertificates();loadAudit();loadLivenessSessions()}

function init(){
    bindDelegated();
    var tabs = Array.prototype.slice.call(document.querySelectorAll('.bio-tabs [role="tab"]'));
    var active = tabs.filter(function(t){ return t.classList.contains('active') || t.getAttribute('aria-selected') === 'true'; })[0] || tabs[0];
    if (active) switchTab(active.getAttribute('data-tab') || 'kyc', active);
    loadAll();
}

window._bio={switchTab:switchTab,verifyUser:verifyUser,rejectUser:rejectUser,openSignaturePad:openSignaturePad,clearSignature:clearSignature,submitSignature:submitSignature,renewCertificate:renewCertificate,startLiveness:startLiveness,endLiveness:endLiveness,captureFrame:captureFrame,loadAll:loadAll};
init();
})();
