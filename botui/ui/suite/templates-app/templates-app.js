(function(){
var templates=[];
var currentTab='all';
var selectedTemplate=null;

function showFeedback(msg,type){
    var el=document.getElementById('tpl-feedback');
    if(!el){el=document.createElement('div');el.id='tpl-feedback';el.style.cssText='position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;transition:opacity .3s';document.body.appendChild(el)}
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
    document.querySelectorAll('.tpl-tab').forEach(function(t){t.classList.remove('active')});
    document.querySelectorAll('.tpl-panel').forEach(function(p){p.classList.remove('active')});
    if(btn)btn.classList.add('active');
    var panel=document.getElementById('tpl-panel-'+tab);
    if(panel)panel.classList.add('active');
}

function updateStats(){
    var total=templates.length;
    var business=templates.filter(function(t){return t.category==='business'}).length;
    var service=templates.filter(function(t){return t.category==='service'}).length;
    var lifestyle=templates.filter(function(t){return t.category==='lifestyle'}).length;
    var el1=document.getElementById('tpl-stat-total');
    var el2=document.getElementById('tpl-stat-business');
    var el3=document.getElementById('tpl-stat-service');
    var el4=document.getElementById('tpl-stat-lifestyle');
    if(el1)el1.textContent=total;
    if(el2)el2.textContent=business;
    if(el3)el3.textContent=service;
    if(el4)el4.textContent=lifestyle;
}

function renderTemplateCard(t){
    return '<div class="tpl-card" onclick="window._tpl.previewTemplate(\''+t.id+'\')">'
        +'<div class="tpl-card-header"><span class="tpl-card-icon">'+(t.icon||'🤖')+'</span><span class="tpl-card-title">'+(t.name||'')+'</span><span class="tpl-badge '+(t.category||'custom')+'">'+(t.category||'custom')+'</span></div>'
        +'<div class="tpl-card-desc">'+(t.description||'')+'</div>'
        +'<div class="tpl-card-meta"><span>'+(t.features||[]).length+' features</span><span>'+(t.conversations||0)+' conversations</span><span>'+(t.uses||0)+' uses</span></div>'
        +'<div class="tpl-card-actions"><button class="tpl-btn tpl-btn-primary" onclick="event.stopPropagation();window._tpl.useTemplate(\''+t.id+'\')">Use Template</button></div>'
        +'</div>';
}

function renderTemplates(){
    var allGrid=document.getElementById('tpl-all-grid');
    var businessGrid=document.getElementById('tpl-business-grid');
    var serviceGrid=document.getElementById('tpl-service-grid');
    var lifestyleGrid=document.getElementById('tpl-lifestyle-grid');
    var customGrid=document.getElementById('tpl-custom-grid');
    if(allGrid){
        if(templates.length===0){allGrid.innerHTML='<div class="tpl-empty"><div class="tpl-empty-icon">🤖</div><h3>No Templates</h3><p>Bot templates will appear here</p></div>'}
        else{allGrid.innerHTML=templates.map(renderTemplateCard).join('')}
    }
    if(businessGrid)businessGrid.innerHTML=templates.filter(function(t){return t.category==='business'}).map(renderTemplateCard).join('')||'<div class="tpl-empty"><p>No business templates</p></div>';
    if(serviceGrid)serviceGrid.innerHTML=templates.filter(function(t){return t.category==='service'}).map(renderTemplateCard).join('')||'<div class="tpl-empty"><p>No service templates</p></div>';
    if(lifestyleGrid)lifestyleGrid.innerHTML=templates.filter(function(t){return t.category==='lifestyle'}).map(renderTemplateCard).join('')||'<div class="tpl-empty"><p>No lifestyle templates</p></div>';
    if(customGrid)customGrid.innerHTML=templates.filter(function(t){return t.category==='custom'}).map(renderTemplateCard).join('')||'<div class="tpl-empty"><p>No custom templates</p></div>';
}

async function loadTemplates(){
    var data=await apiCall('/api/templates/list');
    if(data){templates=Array.isArray(data)?data:[];updateStats();renderTemplates()}
}

function previewTemplate(templateId){
    selectedTemplate=templates.find(function(t){return t.id===templateId});
    if(!selectedTemplate)return;
    var title=document.getElementById('tpl-preview-title');
    var body=document.getElementById('tpl-preview-body');
    if(title)title.textContent=selectedTemplate.name||'Template Preview';
    if(body){
        var features=(selectedTemplate.features||[]);
        var sampleConv=(selectedTemplate.sample_conversations||[]);
        body.innerHTML='<div style="margin-bottom:16px">'
            +'<span class="tpl-badge '+(selectedTemplate.category||'custom')+'">'+(selectedTemplate.category||'custom')+'</span>'
            +'</div>'
            +'<p style="font-size:14px;line-height:1.6;color:var(--text-secondary,#94a3b8);margin-bottom:16px">'+(selectedTemplate.description||'')+'</p>'
            +'<div class="tpl-preview-section"><h4>Features</h4><div class="tpl-preview-list">'+features.map(function(f){return '<div class="tpl-preview-item">'+f+'</div>'}).join('')+'</div></div>'
            +(sampleConv.length>0?'<div class="tpl-preview-section"><h4>Sample Conversations</h4><div class="tpl-conversation">'+sampleConv.map(function(c){return '<div class="tpl-msg '+(c.role||'bot')+'">'+(c.text||'')+'</div>'}).join('')+'</div></div>':'')
            +'<div class="tpl-preview-section"><h4>Categories</h4><div style="display:flex;flex-wrap:wrap;gap:6px">'+(selectedTemplate.categories||[]).map(function(c){return '<span style="padding:4px 8px;background:var(--bg-tertiary,#334155);border-radius:4px;font-size:12px;color:var(--text-primary,#f8fafc)">'+c+'</span>'}).join('')+'</div></div>';
    }
    showModal('tpl-preview-modal');
}

function useTemplate(templateId){
    selectedTemplate=templates.find(function(t){return t.id===templateId})||selectedTemplate;
    showDeployModal();
}

function showDeployModal(){
    hideModal('tpl-preview-modal');
    showModal('tpl-deploy-modal');
}

async function deployBot(){
    var name=document.getElementById('tpl-bot-name').value;
    var desc=document.getElementById('tpl-bot-desc').value;
    if(!name){showFeedback('Bot name is required','error');return}
    if(!selectedTemplate){showFeedback('No template selected','error');return}
    var data=await apiCall('/api/templates/deploy',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({template_id:selectedTemplate.id,bot_name:name,description:desc})});
    if(data){hideModal('tpl-deploy-modal');showFeedback('Bot deployed successfully!','success');selectedTemplate=null}
}

function showModal(id){var el=document.getElementById(id);if(el)el.classList.add('visible')}
function hideModal(id){var el=document.getElementById(id);if(el)el.classList.remove('visible')}

function loadAll(){loadTemplates()}

window._tpl={switchTab:switchTab,previewTemplate:previewTemplate,useTemplate:useTemplate,showDeployModal:showDeployModal,deployBot:deployBot,showModal:showModal,hideModal:hideModal,loadTemplates:loadTemplates,loadAll:loadAll};
loadAll();
})();
