(function(){
var courses=[];
var certifications=[];
var progress={};
var currentTab='courses';
var selectedCourse=null;

function showFeedback(msg,type){
    var el=document.getElementById('la-feedback');
    if(!el){el=document.createElement('div');el.id='la-feedback';el.style.cssText='position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;transition:opacity .3s';document.body.appendChild(el)}
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
    document.querySelectorAll('.learn-app-tab').forEach(function(t){t.classList.remove('active')});
    document.querySelectorAll('.learn-app-panel').forEach(function(p){p.classList.remove('active')});
    if(btn)btn.classList.add('active');
    var panel=document.getElementById('la-panel-'+tab);
    if(panel)panel.classList.add('active');
}

function updateStats(){
    var el1=document.getElementById('la-stat-hours');
    var el2=document.getElementById('la-stat-completed');
    var el3=document.getElementById('la-stat-streak');
    var el4=document.getElementById('la-stat-badges');
    if(el1)el1.textContent=(progress.hours_learned||0).toFixed(1);
    if(el2)el2.textContent=progress.courses_completed||0;
    if(el3)el3.textContent=(progress.streak||0)+' days';
    if(el4)el4.textContent=(progress.badges||[]).length;
}

function renderCourses(){
    var grid=document.getElementById('la-courses-grid');
    if(!grid)return;
    if(courses.length===0){grid.innerHTML='<div class="learn-app-empty"><div class="learn-app-empty-icon">📚</div><h3>No Courses Available</h3><p>Check back later for new content</p></div>';return}
    grid.innerHTML=courses.map(function(c){
        var diffClass=c.difficulty==='beginner'?'beginner':c.difficulty==='advanced'?'advanced':'intermediate';
        var pct=c.progress!=null?c.progress:0;
        return '<div class="learn-app-card" onclick="window._learnApp.previewCourse(\''+c.id+'\')">'
            +'<div class="learn-app-card-header"><span class="learn-app-card-icon">'+(c.icon||'📚')+'</span><span class="learn-app-card-title">'+(c.title||'')+'</span><span class="learn-app-badge '+diffClass+'">'+(c.difficulty||'')+'</span></div>'
            +'<div class="learn-app-card-desc">'+(c.description||'')+'</div>'
            +'<div class="learn-app-card-meta"><span>'+(c.duration||'')+'</span><span>'+(c.lessons||0)+' lessons</span><span>'+(c.enrolled||0)+' enrolled</span></div>'
            +(pct>0?'<div class="learn-app-progress"><div class="learn-app-progress-fill" style="width:'+pct+'%"></div></div>':'')
            +'<div class="learn-app-card-actions"><button class="learn-app-btn learn-app-btn-primary" onclick="event.stopPropagation();window._learnApp.previewCourse(\''+c.id+'\')">'+(pct>0?'Continue':'Start Course')+'</button></div>'
            +'</div>';
    }).join('');
}

function renderCertifications(){
    var grid=document.getElementById('la-certs-grid');
    if(!grid)return;
    if(certifications.length===0){grid.innerHTML='<div class="learn-app-empty"><div class="learn-app-empty-icon">🏆</div><h3>No Certifications</h3><p>Complete courses to earn certifications</p></div>';return}
    grid.innerHTML=certifications.map(function(c){
        var isEarned=c.earned===true;
        return '<div class="learn-app-card">'
            +'<div class="learn-app-card-header"><span class="learn-app-card-icon">'+(isEarned?'🏆':'📋')+'</span><span class="learn-app-card-title">'+(c.name||'')+'</span><span class="learn-app-badge '+(isEarned?'earned':'available')+'">'+(isEarned?'Earned':'Available')+'</span></div>'
            +'<div class="learn-app-card-desc">'+(c.description||'')+'</div>'
            +'<div class="learn-app-checklist">'+(c.requirements||[]).map(function(r){return '<div class="learn-app-check '+(r.completed?'done':'pending')+'"><span>'+(r.completed?'✅':'⬜')+'</span><span>'+(r.text||'')+'</span></div>'}).join('')+'</div>'
            +(isEarned?'<div style="margin-top:8px;font-size:12px;color:var(--text-secondary,#94a3b8)">Earned: '+(c.earned_date||'')+'</div>':'')
            +'</div>';
    }).join('');
}

function renderProgress(){
    var statsList=document.getElementById('la-stats-list');
    if(statsList){
        var stats=[
            {label:'Hours Learned',value:(progress.hours_learned||0).toFixed(1)+'h'},
            {label:'Courses Completed',value:progress.courses_completed||0},
            {label:'Courses In Progress',value:progress.courses_in_progress||0},
            {label:'Current Streak',value:(progress.streak||0)+' days'},
            {label:'Longest Streak',value:(progress.longest_streak||0)+' days'},
            {label:'Avg Session',value:(progress.avg_session||0)+' min'}
        ];
        statsList.innerHTML=stats.map(function(s){
            return '<div style="display:flex;justify-content:space-between;padding:8px 0;border-bottom:1px solid var(--border-color,#334155);font-size:13px"><span style="color:var(--text-secondary,#94a3b8)">'+s.label+'</span><span style="font-weight:600">'+s.value+'</span></div>';
        }).join('');
    }
    var badgesGrid=document.getElementById('la-badges-grid');
    if(badgesGrid){
        var badges=progress.badges||[];
        if(badges.length===0){badgesGrid.innerHTML='<div class="learn-app-empty" style="grid-column:1/-1"><p>No badges earned yet</p></div>';return}
        badgesGrid.innerHTML=badges.map(function(b){
            return '<div class="learn-app-badge-item"><div class="learn-app-badge-icon">'+(b.icon||'🏅')+'</div><div class="learn-app-badge-name">'+(b.name||'')+'</div><div class="learn-app-badge-date">'+(b.date||'')+'</div></div>';
        }).join('');
    }
}

async function loadCourses(){
    var data=await apiCall('/api/learn/courses');
    if(data){courses=Array.isArray(data)?data:[];renderCourses()}
}

async function loadCertifications(){
    var data=await apiCall('/api/learn/certifications');
    if(data){certifications=Array.isArray(data)?data:[];renderCertifications()}
}

async function loadProgress(){
    var data=await apiCall('/api/learn/progress');
    if(data){progress=data||{};updateStats();renderProgress()}
}

function previewCourse(courseId){
    selectedCourse=courses.find(function(c){return c.id===courseId});
    if(!selectedCourse)return;
    var title=document.getElementById('la-preview-title');
    var body=document.getElementById('la-preview-body');
    if(title)title.textContent=selectedCourse.title||'Course Preview';
    if(body){
        body.innerHTML='<div style="margin-bottom:16px"><span class="learn-app-badge '+(selectedCourse.difficulty||'beginner')+'">'+(selectedCourse.difficulty||'beginner')+'</span> <span style="margin-left:8px;font-size:13px;color:var(--text-secondary,#94a3b8)">'+(selectedCourse.duration||'')+' | '+(selectedCourse.lessons||0)+' lessons</span></div>'
            +'<p style="font-size:14px;line-height:1.6;color:var(--text-secondary,#94a3b8)">'+(selectedCourse.description||'')+'</p>'
            +(selectedCourse.features?'<div style="margin-top:16px"><h4 style="font-size:13px;font-weight:600;margin:0 0 8px 0">Features</h4><div style="display:flex;flex-direction:column;gap:4px">'+(selectedCourse.features||[]).map(function(f){return '<div style="font-size:13px;color:var(--text-secondary,#94a3b8)">✅ '+f+'</div>'}).join('')+'</div></div>':'');
    }
    showModal('la-preview-modal');
}

async function enrollCourse(){
    if(!selectedCourse)return;
    var data=await apiCall('/api/learn/courses/'+selectedCourse.id+'/enroll',{method:'POST'});
    if(data){hideModal('la-preview-modal');showFeedback('Enrolled successfully','success');loadCourses();loadProgress()}
}

function createContent(type){
    var title=document.getElementById('la-content-title');
    var desc=document.getElementById('la-content-desc');
    var typeEl=document.getElementById('la-content-type');
    if(title)title.value='';
    if(desc)desc.value='';
    if(typeEl)typeEl.value=type||'lesson';
    showFeedback('Content editor ready for: '+type,'success');
}

async function aiAssist(){
    showFeedback('AI assist is analyzing your content...','success');
    var data=await apiCall('/api/learn/ai-assist',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({title:document.getElementById('la-content-title').value,description:document.getElementById('la-content-desc').value})});
    if(data&&data.suggestion){
        var desc=document.getElementById('la-content-desc');
        if(desc)desc.value=data.suggestion;
        showFeedback('AI suggestion applied','success');
    }
}

async function saveDraft(){
    var title=document.getElementById('la-content-title').value;
    var desc=document.getElementById('la-content-desc').value;
    var type=document.getElementById('la-content-type').value;
    if(!title){showFeedback('Title is required','error');return}
    var data=await apiCall('/api/learn/content',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({title:title,description:desc,type:type,status:'draft'})});
    if(data){showFeedback('Draft saved','success')}
}

async function publishContent(){
    var title=document.getElementById('la-content-title').value;
    var desc=document.getElementById('la-content-desc').value;
    var type=document.getElementById('la-content-type').value;
    if(!title){showFeedback('Title is required','error');return}
    var data=await apiCall('/api/learn/content',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({title:title,description:desc,type:type,status:'published'})});
    if(data){showFeedback('Content published','success')}
}

function showModal(id){var el=document.getElementById(id);if(el)el.style.display='flex'}
function hideModal(id){var el=document.getElementById(id);if(el)el.style.display='none'}

function loadAll(){loadCourses();loadCertifications();loadProgress()}

window._learnApp={switchTab:switchTab,previewCourse:previewCourse,enrollCourse:enrollCourse,createContent:createContent,aiAssist:aiAssist,saveDraft:saveDraft,publishContent:publishContent,showModal:showModal,hideModal:hideModal,loadCourses:loadCourses,loadCertifications:loadCertifications,loadProgress:loadProgress,loadAll:loadAll};
loadAll();
})();
