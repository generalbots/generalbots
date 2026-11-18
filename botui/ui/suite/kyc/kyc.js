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

function escapeHtml(str){ if (window.GBAppKit && window.GBAppKit.escapeHtml) return window.GBAppKit.escapeHtml(str); if(str==null||str==='') return ''; var d=document.createElement('div'); d.textContent=String(str); return d.innerHTML; }
function showFeedback(msg,type){ if (window.GBAppKit && window.GBAppKit.showFeedback) return window.GBAppKit.showFeedback('kyc', msg, type); }
async function apiCall(url,options){ if (window.GBAppKit && window.GBAppKit.apiCall) return window.GBAppKit.apiCall('kyc', url, options); try{var opts=options?Object.assign({},options):{};var headers=Object.assign({},(options&&options.headers)||{});var hasAuth=false;Object.keys(headers).forEach(function(k){if(String(k).toLowerCase()==='authorization')hasAuth=true;});if(!hasAuth){var token=(window.GBAppKit&&window.GBAppKit.getAuthToken)?window.GBAppKit.getAuthToken():(localStorage.getItem('gb_token')||localStorage.getItem('gb-access-token')||sessionStorage.getItem('gb-access-token')||'');if(token)headers.Authorization='Bearer '+token;}opts.headers=headers;var r=await fetch(url,opts);if(!r.ok)throw new Error('HTTP '+r.status);var t=await r.text();return t?JSON.parse(t):null}catch(e){showFeedback('API error: '+e.message,'error');return null} }

function switchTab(tab,btn){
    currentTab=tab;
    if (window.GBAppKit && window.GBAppKit.switchTab) {
        window.GBAppKit.switchTab(tab, btn, {
            tabSelector: '.kyc-tabs [role="tab"]',
            panelSelector: '.kyc-panel',
            panelId: function(t){ return t === 'verifications' ? 'panel-verifications' : 'kyc-panel-' + t; }
        });
        return;
    }
    var tabs = Array.prototype.slice.call(document.querySelectorAll('.kyc-tabs [role="tab"]'));
    tabs.forEach(function(t){
        var active = t === btn;
        t.classList.toggle('active', active);
        t.setAttribute('aria-selected', active ? 'true' : 'false');
        t.tabIndex = active ? 0 : -1;
    });
    document.querySelectorAll('.kyc-panel').forEach(function(p){
        p.classList.remove('active');
        p.hidden = true;
    });
    var panel=document.getElementById(tab==='verifications' ? 'panel-verifications' : 'kyc-panel-'+tab);
    if(panel){
        panel.classList.add('active');
        panel.hidden = false;
        if (btn && btn.id) panel.setAttribute('aria-labelledby', btn.id);
    }
    if (btn && btn.focus) { /* focus managed by caller when keyboard */ }
}

function updateStats(){
    var pending=verifications.filter(function(v){return v.status==='pending'}).length;
    var verified=verifications.filter(function(v){return v.status==='verified'}).length;
    var failed=verifications.filter(function(v){return v.status==='failed'}).length;
    var values={
        'kyc-stat-pending': pending,
        'kyc-stat-verified': verified,
        'kyc-stat-failed': failed,
        'kyc-stat-total': verifications.length
    };
    if (window.GBAppKit && window.GBAppKit.setStatValues) return window.GBAppKit.setStatValues(values);
    Object.keys(values).forEach(function(id){ var el=document.getElementById(id); if(el) el.textContent=String(values[id]); });
}

function renderVerifications(){
    var grid=document.getElementById('verifications-grid');
    if(!grid)return;
    if(verifications.length===0){grid.innerHTML='<div class="kyc-empty"><div class="kyc-empty-icon">🔍</div><h3>No Verifications</h3><p>No pending verifications at this time</p></div>';return}
    grid.innerHTML=verifications.map(function(v){
        var scoreClass=v.liveness_score>=80?'high':v.liveness_score>=50?'medium':'low';
        var statusClass=v.status==='pending'?'pending':v.status==='verified'?'verified':'failed';
        var id = escapeHtml(String(v.id == null ? '' : v.id));
        return '<div class="kyc-card">'
            +'<div class="kyc-card-header"><span class="kyc-card-title">'+escapeHtml(v.user_name||'Unknown')+'</span><span class="kyc-badge '+statusClass+'">'+escapeHtml(String(v.status||''))+'</span></div>'
            +'<div class="kyc-card-body">'
            +'<div class="kyc-card-row"><span class="label">Document</span><span>'+escapeHtml(v.document_type||'N/A')+'</span></div>'
            +'<div class="kyc-card-row"><span class="label">Document #</span><span>'+escapeHtml(v.document_number||'N/A')+'</span></div>'
            +'<div class="kyc-card-row"><span class="label">Selfie</span><span>'+escapeHtml(v.selfie_status||'N/A')+'</span></div>'
            +'<div class="kyc-card-row"><span class="label">Liveness Score</span><span>'+escapeHtml(v.liveness_score!=null?String(v.liveness_score)+'%':'N/A')+'</span></div>'
            +(v.liveness_score!=null?'<div class="kyc-score-bar"><div class="kyc-score-fill '+scoreClass+'" style="width:'+escapeHtml(String(Number(v.liveness_score)||0))+'%"></div></div>':'')
            +'</div>'
            +'<div class="kyc-card-actions">'
            +(v.status==='pending'?'<button type="button" class="kyc-btn kyc-btn-success" data-kyc-action="verify" data-kyc-id="'+id+'">Approve</button><button type="button" class="kyc-btn kyc-btn-danger" data-kyc-action="reject" data-kyc-id="'+id+'">Reject</button>':'')
            +'</div></div>';
    }).join('');
}

function renderSignatures(){
    var tbody=document.getElementById('kyc-signatures-tbody');
    if(!tbody)return;
    if(signatures.length===0){tbody.innerHTML='<tr><td colspan="5" class="kyc-empty">No documents awaiting signature</td></tr>';return}
    tbody.innerHTML=signatures.map(function(s){
        var id = escapeHtml(String(s.id == null ? '' : s.id));
        return '<tr>'
            +'<td>'+escapeHtml(s.document_name||'')+'</td>'
            +'<td><span class="kyc-badge active">'+escapeHtml(s.document_type||'')+'</span></td>'
            +'<td>'+escapeHtml(s.requested_at||'')+'</td>'
            +'<td>'+escapeHtml(s.signer||'')+'</td>'
            +'<td><button type="button" class="kyc-btn kyc-btn-primary" data-kyc-action="open-signature" data-kyc-id="'+id+'">Sign</button></td>'
            +'</tr>';
    }).join('');
}

function renderCertificates(){
    var grid=document.getElementById('kyc-certificates-grid');
    if(!grid)return;
    if(certificates.length===0){grid.innerHTML='<div class="kyc-empty"><div class="kyc-empty-icon">📜</div><h3>No Certificates</h3><p>No digital certificates found</p></div>';return}
    grid.innerHTML=certificates.map(function(c){
        var isExpired=new Date(c.expiry_date)<new Date();
        var isExpiring=new Date(c.expiry_date)<new Date(Date.now()+30*24*60*60*1000);
        var badgeClass=isExpired?'failed':isExpiring?'expiring':'active';
        var id = escapeHtml(String(c.id == null ? '' : c.id));
        return '<div class="kyc-card">'
            +'<div class="kyc-card-header"><span class="kyc-card-title">'+escapeHtml(c.name||'')+'</span><span class="kyc-badge '+badgeClass+'">'+(isExpired?'Expired':isExpiring?'Expiring':'Active')+'</span></div>'
            +'<div class="kyc-card-body">'
            +'<div class="kyc-card-row"><span class="label">Issuer</span><span>'+escapeHtml(c.issuer||'')+'</span></div>'
            +'<div class="kyc-card-row"><span class="label">Serial</span><span>'+escapeHtml(c.serial||'')+'</span></div>'
            +'<div class="kyc-card-row"><span class="label">Issued</span><span>'+escapeHtml(c.issued_date||'')+'</span></div>'
            +'<div class="kyc-card-row"><span class="label">Expires</span><span>'+escapeHtml(c.expiry_date||'')+'</span></div>'
            +'</div>'
            +'<div class="kyc-card-actions">'
            +'<button type="button" class="kyc-btn kyc-btn-primary" data-kyc-action="renew-modal" data-kyc-id="'+id+'">Renew</button>'
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
    var area=document.getElementById('kyc-signature-pad-area');
    if(area)area.style.display='block';
    var canvas=document.getElementById('kyc-signature-canvas');
    if(!canvas)return;
    if (window.GBAppKit && window.GBAppKit.mountSignaturePad) {
        window.GBAppKit.mountSignaturePad(canvas);
    } else {
        // Fallback: local mount (behavior-preserving)
        var parent = canvas.parentElement;
        if(parent){
            var w = Math.min(parent.clientWidth || 500, 800);
            canvas.width = Math.max(300, w);
            canvas.height = 200;
        }
        var ctx=canvas.getContext('2d');
        ctx.fillStyle='#0f172a';
        ctx.fillRect(0,0,canvas.width,canvas.height);
        ctx.strokeStyle='#f8fafc';
        ctx.lineWidth=2;
        ctx.lineCap='round';
        canvas.hasDrawn = false;
        function getPos(e){
            var rect = canvas.getBoundingClientRect();
            var clientX, clientY;
            if(e.touches && e.touches.length){ clientX = e.touches[0].clientX; clientY = e.touches[0].clientY; }
            else if(e.changedTouches && e.changedTouches.length){ clientX = e.changedTouches[0].clientX; clientY = e.changedTouches[0].clientY; }
            else { clientX = e.clientX; clientY = e.clientY; }
            return { x: clientX - rect.left, y: clientY - rect.top };
        }
        canvas.onmousedown=function(e){drawing=true; var p=getPos(e); lastX=p.x; lastY=p.y; canvas.hasDrawn=true; e.preventDefault();};
        canvas.onmousemove=function(e){if(!drawing)return; var p=getPos(e); ctx.beginPath(); ctx.moveTo(lastX,lastY); ctx.lineTo(p.x,p.y); ctx.stroke(); lastX=p.x; lastY=p.y; e.preventDefault();};
        canvas.onmouseup=function(){drawing=false};
        canvas.onmouseleave=function(){drawing=false};
        canvas.ontouchstart=function(e){drawing=true; var p=getPos(e); lastX=p.x; lastY=p.y; canvas.hasDrawn=true; e.preventDefault();};
        canvas.ontouchmove=function(e){if(!drawing)return; var p=getPos(e); ctx.beginPath(); ctx.moveTo(lastX,lastY); ctx.lineTo(p.x,p.y); ctx.stroke(); lastX=p.x; lastY=p.y; e.preventDefault();};
        canvas.ontouchend=function(e){drawing=false; e.preventDefault();};
        canvas.ontouchcancel=function(){drawing=false;};
    }
}

function clearSignature(){
    var canvas=document.getElementById('kyc-signature-canvas');
    if(!canvas)return;
    if (window.GBAppKit && window.GBAppKit.clearSignatureCanvas) {
        window.GBAppKit.clearSignatureCanvas(canvas);
    } else {
        var ctx=canvas.getContext('2d');
        ctx.fillStyle='#0f172a';
        ctx.fillRect(0,0,canvas.width,canvas.height);
        canvas.hasDrawn = false;
    }
}

async function submitSignature(){
    if(!selectedSignDoc){ showFeedback('No document selected','error'); return; }
    var canvas=document.getElementById('kyc-signature-canvas');
    if(!canvas){ showFeedback('Signature pad not ready','error'); return; }
    if(!canvas.hasDrawn){ showFeedback('Please draw your signature before submitting','error'); return; }
    var signatureData=canvas.toDataURL('image/png');
    var data=await apiCall('/api/kyc/signatures/'+selectedSignDoc+'/sign',{
        method:'POST',
        headers:{'Content-Type':'application/json'},
        body:JSON.stringify({signature:signatureData})
    });
    if(data){
        showFeedback('Document signed','success');
        selectedSignDoc=null;
        var area=document.getElementById('kyc-signature-pad-area');
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

function showModal(id){ if (window.GBAppKit && window.GBAppKit.showModal) return window.GBAppKit.showModal(id, {className:'visible'}); var el=document.getElementById(id);if(el)el.classList.add('visible'); }
function hideModal(id){ if (window.GBAppKit && window.GBAppKit.hideModal) return window.GBAppKit.hideModal(id, {className:'visible'}); var el=document.getElementById(id);if(el)el.classList.remove('visible'); }

function bindDelegated(){
    var root = document.querySelector('[data-gb-app="kyc"]') || document;
    if (root.__kycBound) return;
    root.__kycBound = true;
    root.addEventListener('click', function(e){
        var tab = e.target.closest('.kyc-tabs [data-tab]');
        if (tab) { switchTab(tab.getAttribute('data-tab') || 'verifications', tab); return; }
        var btn = e.target.closest('[data-kyc-action]');
        if (!btn) return;
        var action = btn.getAttribute('data-kyc-action');
        var id = btn.getAttribute('data-kyc-id');
        if (action === 'verify') verifyUser(id);
        else if (action === 'reject') rejectUser(id);
        else if (action === 'open-signature') openSignaturePad(id);
        else if (action === 'renew-modal') showRenewModal(id);
        else if (action === 'refresh-all') loadAll();
        else if (action === 'refresh-signatures') loadSignatures();
        else if (action === 'clear-signature') clearSignature();
        else if (action === 'submit-signature') submitSignature();
        else if (action === 'hide-modal') hideModal(btn.getAttribute('data-kyc-modal') || 'renew-modal');
        else if (action === 'renew-cert') renewCertificate();
    });
    document.addEventListener('keydown', function(e){
        var tab = e.target.closest('.kyc-tabs [role="tab"]');
        if (!tab) return;
        var tabs = Array.prototype.slice.call(document.querySelectorAll('.kyc-tabs [role="tab"]'));
        var idx = tabs.indexOf(tab);
        if (idx === -1) return;
        var next = null;
        if (e.key === 'ArrowRight') next = tabs[(idx + 1) % tabs.length];
        else if (e.key === 'ArrowLeft') next = tabs[(idx - 1 + tabs.length) % tabs.length];
        else if (e.key === 'Home') next = tabs[0];
        else if (e.key === 'End') next = tabs[tabs.length - 1];
        if (next) {
            e.preventDefault();
            var nextTab = next.getAttribute('data-tab');
            switchTab(nextTab, next);
            next.focus();
        }
    });
}

function loadAll(){loadVerifications();loadSignatures();loadCertificates()}

function init(){
    bindDelegated();
    var tabs = Array.prototype.slice.call(document.querySelectorAll('.kyc-tabs [role="tab"]'));
    var active = tabs.filter(function(t){ return t.classList.contains('active') || t.getAttribute('aria-selected') === 'true'; })[0] || tabs[0];
    if (active) switchTab(active.getAttribute('data-tab') || 'verifications', active);
    loadAll();
}

window._kyc={switchTab:switchTab,verifyUser:verifyUser,rejectUser:rejectUser,openSignaturePad:openSignaturePad,clearSignature:clearSignature,submitSignature:submitSignature,showRenewModal:showRenewModal,renewCertificate:renewCertificate,showModal:showModal,hideModal:hideModal,loadVerifications:loadVerifications,loadSignatures:loadSignatures,loadCertificates:loadCertificates,loadAll:loadAll};
init();
})();
