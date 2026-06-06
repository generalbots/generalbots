(function(){
var records=[];
var overtime=[];
var reports=[];
var currentTab='clock';
var clockInterval=null;
var gpsAvailable=false;

function showFeedback(msg,type){
    var el=document.getElementById('tc-feedback');
    if(!el){el=document.createElement('div');el.id='tc-feedback';el.style.cssText='position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;transition:opacity .3s';document.body.appendChild(el)}
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
    document.querySelectorAll('.tc-tab').forEach(function(t){t.classList.remove('active')});
    document.querySelectorAll('.tc-panel').forEach(function(p){p.classList.remove('active')});
    if(btn)btn.classList.add('active');
    var panel=document.getElementById('panel-'+tab);
    if(panel)panel.classList.add('active');
}

function updateClock(){
    var now=new Date();
    var h=String(now.getHours()).padStart(2,'0');
    var m=String(now.getMinutes()).padStart(2,'0');
    var s=String(now.getSeconds()).padStart(2,'0');
    var clockEl=document.getElementById('live-clock');
    if(clockEl)clockEl.textContent=h+':'+m+':'+s;
    var opts={weekday:'long',year:'numeric',month:'long',day:'numeric'};
    var dateEl=document.getElementById('live-date');
    if(dateEl)dateEl.textContent=now.toLocaleDateString('en-US',opts);
}

function checkGPS(){
    if('geolocation' in navigator){
        gpsAvailable=true;
        var dot=document.getElementById('gps-dot');
        var status=document.getElementById('gps-status');
        if(dot)dot.classList.remove('offline');
        if(status)status.textContent='GPS active';
        navigator.geolocation.getCurrentPosition(function(){},function(){
            gpsAvailable=false;
            if(dot)dot.classList.add('offline');
            if(status)status.textContent='GPS unavailable';
        });
    }else{
        gpsAvailable=false;
        var dot=document.getElementById('gps-dot');
        var status=document.getElementById('gps-status');
        if(dot)dot.classList.add('offline');
        if(status)status.textContent='GPS not supported';
    }
}

function updateStats(){
    var todayHours=records.length>0?records[records.length-1].hours_worked||0:0;
    var weekHours=records.reduce(function(sum,r){return sum+(r.hours_worked||0)},0);
    var otHours=overtime.filter(function(o){return o.status==='approved'}).reduce(function(sum,o){return sum+(o.hours||0)},0);
    var el1=document.getElementById('stat-today-hours');
    var el2=document.getElementById('stat-week-hours');
    var el3=document.getElementById('stat-overtime');
    var el4=document.getElementById('stat-status');
    if(el1)el1.textContent=todayHours.toFixed(1)+'h';
    if(el2)el2.textContent=weekHours.toFixed(1)+'h';
    if(el3)el3.textContent=otHours.toFixed(1)+'h';
    var lastRecord=records.length>0?records[records.length-1]:null;
    if(el4){
        if(lastRecord&&!lastRecord.clock_out){el4.textContent='Clocked In';el4.style.color='#22c55e'}
        else{el4.textContent='Clocked Out';el4.style.color='#94a3b8'}
    }
    var btnIn=document.getElementById('btn-clock-in');
    var btnOut=document.getElementById('btn-clock-out');
    if(btnIn&&btnOut){
        if(lastRecord&&!lastRecord.clock_out){btnIn.disabled=true;btnOut.disabled=false}
        else{btnIn.disabled=false;btnOut.disabled=true}
    }
}

function renderRecords(){
    var tbody=document.getElementById('records-tbody');
    if(!tbody)return;
    if(records.length===0){tbody.innerHTML='<tr><td colspan="5" class="tc-empty">No records found</td></tr>';return}
    tbody.innerHTML=records.map(function(r){
        var status=r.hours_worked>8?'overtime':r.clock_in?'normal':'absent';
        return '<tr>'
            +'<td>'+(r.date||'')+'</td>'
            +'<td>'+(r.clock_in||'--')+'</td>'
            +'<td>'+(r.clock_out||'--')+'</td>'
            +'<td>'+(r.hours_worked!=null?r.hours_worked.toFixed(1)+'h':'--')+'</td>'
            +'<td><span class="tc-badge '+status+'">'+status+'</span></td>'
            +'</tr>';
    }).join('');
}

function renderOvertime(){
    var grid=document.getElementById('overtime-grid');
    if(!grid)return;
    if(overtime.length===0){grid.innerHTML='<div class="tc-empty"><div class="tc-empty-icon">⏰</div><h3>No Overtime Requests</h3><p>No pending overtime approvals</p></div>';return}
    grid.innerHTML=overtime.map(function(o){
        var statusClass=o.status==='pending'?'pending':o.status==='approved'?'approved':'rejected';
        return '<div class="tc-ot-card">'
            +'<div class="tc-ot-card-header"><span class="tc-ot-card-title">'+(o.employee_name||'')+'</span><span class="tc-badge '+statusClass+'">'+o.status+'</span></div>'
            +'<div class="tc-ot-card-body">'
            +'<div class="tc-ot-card-row"><span class="label">Date</span><span>'+(o.date||'')+'</span></div>'
            +'<div class="tc-ot-card-row"><span class="label">Hours</span><span>'+(o.hours||0)+'h</span></div>'
            +'<div class="tc-ot-card-row"><span class="label">Reason</span><span>'+(o.reason||'N/A')+'</span></div>'
            +'</div>'
            +(o.status==='pending'?'<div class="tc-ot-card-actions"><button class="tc-btn tc-btn-success" onclick="window._tc.approveOT(\''+o.id+'\')">Approve</button><button class="tc-btn tc-btn-danger" onclick="window._tc.rejectOT(\''+o.id+'\')">Reject</button></div>':'')
            +'</div>';
    }).join('');
}

function renderReports(){
    var grid=document.getElementById('reports-grid');
    if(!grid)return;
    if(reports.length===0){grid.innerHTML='<div class="tc-empty"><div class="tc-empty-icon">📊</div><h3>No Reports</h3><p>No department data available</p></div>';return}
    grid.innerHTML=reports.map(function(r){
        return '<div class="tc-report-card">'
            +'<h3>'+(r.department||'Department')+'</h3>'
            +'<div class="tc-report-row"><span class="label">Employees</span><span class="value">'+(r.employee_count||0)+'</span></div>'
            +'<div class="tc-report-row"><span class="label">Total Hours</span><span class="value">'+(r.total_hours||0).toFixed(1)+'h</span></div>'
            +'<div class="tc-report-row"><span class="label">Overtime Hours</span><span class="value">'+(r.overtime_hours||0).toFixed(1)+'h</span></div>'
            +'<div class="tc-report-row"><span class="label">Avg Hours/Day</span><span class="value">'+(r.avg_hours||0).toFixed(1)+'h</span></div>'
            +'</div>';
    }).join('');
}

async function loadRecords(){
    var month=document.getElementById('records-month');
    var url='/api/timeclock/records';
    if(month&&month.value)url+='?month='+month.value;
    var data=await apiCall(url);
    if(data){records=Array.isArray(data)?data:[];updateStats();renderRecords()}
}

async function loadOvertime(){
    var data=await apiCall('/api/timeclock/overtime');
    if(data){overtime=Array.isArray(data)?data:[];renderOvertime()}
}

async function loadReports(){
    var data=await apiCall('/api/timeclock/reports');
    if(data){reports=Array.isArray(data)?data:[];renderReports()}
}

async function loadEmployee(){
    var data=await apiCall('/api/timeclock/clock');
    if(data){
        var nameEl=document.getElementById('employee-name');
        if(nameEl)nameEl.textContent=data.employee_name||'Employee';
        if(data.records)records=data.records;
        updateStats();renderRecords();
    }
}

async function clockIn(){
    var pos=gpsAvailable?await new Promise(function(resolve){navigator.geolocation.getCurrentPosition(resolve,function(){resolve(null)})}):null;
    var payload={};
    if(pos&&pos.coords){payload.latitude=pos.coords.latitude;payload.longitude=pos.coords.longitude}
    var data=await apiCall('/api/timeclock/clock/in',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)});
    if(data){showFeedback('Clocked in successfully','success');loadEmployee();loadRecords()}
}

async function clockOut(){
    var data=await apiCall('/api/timeclock/clock/out',{method:'POST'});
    if(data){showFeedback('Clocked out successfully','success');loadEmployee();loadRecords()}
}

async function approveOT(id){
    var data=await apiCall('/api/timeclock/overtime/'+id+'/approve',{method:'POST'});
    if(data){showFeedback('Overtime approved','success');loadOvertime()}
}

async function rejectOT(id){
    var data=await apiCall('/api/timeclock/overtime/'+id+'/reject',{method:'POST'});
    if(data){showFeedback('Overtime rejected','success');loadOvertime()}
}

var capturedPhoto=null;
var mediaStream=null;

async function capturePhoto(){
    var modal=document.getElementById('tc-photo-modal');
    if(!modal)return;
    modal.style.display='flex';
    try{
        mediaStream=await navigator.mediaDevices.getUserMedia({video:{facingMode:'user',width:480,height:360},audio:false});
        var video=document.getElementById('tc-camera-feed');
        if(video)video.srcObject=mediaStream;
    }catch(e){
        showFeedback('Camera unavailable: '+e.message,'error');
        closePhotoModal();
    }
}

function snapPhoto(){
    var video=document.getElementById('tc-camera-feed');
    var canvas=document.getElementById('tc-photo-canvas');
    if(!video||!canvas)return;
    canvas.width=video.videoWidth||480;
    canvas.height=video.videoHeight||360;
    var ctx=canvas.getContext('2d');
    ctx.drawImage(video,0,0,canvas.width,canvas.height);
    capturedPhoto=canvas.toDataURL('image/jpeg',0.85);
    var result=document.getElementById('tc-photo-result');
    if(result){result.style.display='block';result.innerHTML='<img src="'+capturedPhoto+'" style="width:100%;max-width:240px;border-radius:8px" alt="Captured photo">'}
    var snap=document.getElementById('tc-photo-snap');
    var confirm=document.getElementById('tc-photo-confirm');
    if(snap)snap.style.display='none';
    if(confirm)confirm.style.display='inline-flex';
}

function confirmPhoto(){
    var preview=document.getElementById('photo-preview');
    if(preview&&capturedPhoto)preview.innerHTML='<img src="'+capturedPhoto+'" alt="Clock photo">';
    closePhotoModal();
    showFeedback('Photo captured. Will be attached to next clock event.','success');
}

function closePhotoModal(){
    var modal=document.getElementById('tc-photo-modal');
    if(modal)modal.style.display='none';
    if(mediaStream){
        mediaStream.getTracks().forEach(function(t){t.stop()});
        mediaStream=null;
    }
    var snap=document.getElementById('tc-photo-snap');
    var confirm=document.getElementById('tc-photo-confirm');
    var result=document.getElementById('tc-photo-result');
    if(snap)snap.style.display='inline-flex';
    if(confirm)confirm.style.display='none';
    if(result)result.style.display='none';
}

async function breakStart(){
    var data=await apiCall('/api/timeclock/break/start',{method:'POST'});
    if(data){showFeedback('Break started','success');loadEmployee();loadRecords()}
}

async function breakEnd(){
    var data=await apiCall('/api/timeclock/break/end',{method:'POST'});
    if(data){showFeedback('Break ended','success');loadEmployee();loadRecords()}
}

async function submitJustification(){
    var date=document.getElementById('just-date');
    var type=document.getElementById('just-type');
    var reason=document.getElementById('just-reason');
    if(!date||!type||!reason||!reason.value.trim()){
        showFeedback('Please fill in all required fields','error');
        return;
    }
    var payload={date:date.value,type:type.value,reason:reason.value};
    var data=await apiCall('/api/timeclock/justifications',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)});
    if(data){
        showFeedback('Justification submitted for approval','success');
        reason.value='';
        loadJustifications();
    }
}

function renderJustifications(list){
    var tbody=document.getElementById('justifications-tbody');
    if(!tbody)return;
    if(!list||list.length===0){tbody.innerHTML='<tr><td colspan="6" class="tc-empty">No justifications submitted</td></tr>';return}
    tbody.innerHTML=list.map(function(j){
        var statusClass=j.status==='approved'?'approved':j.status==='rejected'?'rejected':'pending';
        return '<tr>'
            +'<td>'+(j.date||'')+'</td>'
            +'<td>'+(j.type||'')+'</td>'
            +'<td style="max-width:300px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap" title="'+(j.reason||'').replace(/"/g,'&quot;')+'">'+(j.reason||'')+'</td>'
            +'<td>'+(j.submitted_at||'')+'</td>'
            +'<td><span class="tc-badge '+statusClass+'">'+(j.status||'pending')+'</span></td>'
            +'<td>'+(j.approver||'--')+'</td>'
            +'</tr>';
    }).join('');
}

async function loadJustifications(){
    var data=await apiCall('/api/timeclock/justifications');
    renderJustifications(Array.isArray(data)?data:[]);
}

function renderHolidays(list){
    var tbody=document.getElementById('holidays-tbody');
    if(!tbody)return;
    if(!list||list.length===0){tbody.innerHTML='<tr><td colspan="5" class="tc-empty">No holidays found for selected year</td></tr>';return}
    tbody.innerHTML=list.map(function(h){
        return '<tr>'
            +'<td>'+(h.date||'')+'</td>'
            +'<td>'+(h.name||'')+'</td>'
            +'<td>'+(h.type||'national')+'</td>'
            +'<td>'+(h.region||'all')+'</td>'
            +'<td>'+(h.optional?'Yes':'No')+'</td>'
            +'</tr>';
    }).join('');
}

async function loadHolidays(){
    var yearEl=document.getElementById('holidays-year');
    var year=yearEl?yearEl.value:new Date().getFullYear();
    var data=await apiCall('/api/timeclock/holidays?year='+year);
    renderHolidays(Array.isArray(data)?data:[]);
}

function renderAuditLog(list){
    var tbody=document.getElementById('audit-tbody');
    if(!tbody)return;
    if(!list||list.length===0){tbody.innerHTML='<tr><td colspan="6" class="tc-empty">No audit entries</td></tr>';return}
    tbody.innerHTML=list.map(function(a){
        return '<tr>'
            +'<td>'+(a.timestamp||'')+'</td>'
            +'<td>'+(a.employee||'')+'</td>'
            +'<td>'+(a.action||'')+'</td>'
            +'<td>'+(a.method||'password')+'</td>'
            +'<td>'+(a.gps?'✓':'—')+'</td>'
            +'<td><code style="font-size:11px">'+(a.ip||'')+'</code></td>'
            +'</tr>';
    }).join('');
}

async function loadAuditLog(){
    var data=await apiCall('/api/timeclock/audit');
    renderAuditLog(Array.isArray(data)?data:[]);
}

function renderTeam(list){
    var grid=document.getElementById('team-status-grid');
    if(!grid)return;
    if(!list||list.length===0){grid.innerHTML='<div class="tc-empty"><div class="tc-empty-icon">👥</div><h3>No team data</h3><p>Team data unavailable</p></div>';return}
    grid.innerHTML=list.map(function(t){
        var statusClass=t.status==='in'?'normal':t.status==='break'?'pending':'absent';
        return '<div class="tc-ot-card">'
            +'<div class="tc-ot-card-header"><span class="tc-ot-card-title">'+(t.name||'')+'</span><span class="tc-badge '+statusClass+'">'+(t.status||'out')+'</span></div>'
            +'<div class="tc-ot-card-body">'
            +'<div class="tc-ot-card-row"><span class="label">Department</span><span>'+(t.department||'--')+'</span></div>'
            +'<div class="tc-ot-card-row"><span class="label">Last clock-in</span><span>'+(t.last_clock_in||'--')+'</span></div>'
            +'<div class="tc-ot-card-row"><span class="label">Hours today</span><span>'+(t.hours_today||0).toFixed(1)+'h</span></div>'
            +'</div>'
            +'</div>';
    }).join('');
}

async function loadTeam(){
    var data=await apiCall('/api/timeclock/team');
    renderTeam(Array.isArray(data)?data:[]);
}

function exportCsv(){
    if(!records||records.length===0){showFeedback('No records to export','error');return}
    var header='Date,Clock In,Clock Out,Hours,Status\n';
    var rows=records.map(function(r){
        return [r.date||'',r.clock_in||'',r.clock_out||'',r.hours_worked!=null?r.hours_worked.toFixed(2):'',r.status||''].join(',');
    }).join('\n');
    var blob=new Blob([header+rows],{type:'text/csv;charset=utf-8'});
    var url=URL.createObjectURL(blob);
    var a=document.createElement('a');
    a.href=url;
    a.download='timeclock-'+new Date().toISOString().slice(0,10)+'.csv';
    a.click();
    setTimeout(function(){URL.revokeObjectURL(url)},1000);
    showFeedback('CSV exported','success');
}

function printReceipt(){
    var last=records&&records.length>0?records[records.length-1]:null;
    var html='<html><head><title>Clock receipt</title><style>body{font-family:monospace;padding:20px}h1{font-size:14px}p{font-size:12px;margin:4px 0}</style></head><body>'
        +'<h1>Electronic Time Clock Receipt</h1>'
        +'<p>Issued: '+new Date().toLocaleString()+'</p>'
        +'<p>Employee: '+(document.getElementById('employee-name')?document.getElementById('employee-name').textContent:'')+'</p>'
        +(last?'<p>Date: '+(last.date||'')+'</p><p>Clock in: '+(last.clock_in||'')+'</p><p>Clock out: '+(last.clock_out||'pending')+'</p><p>Hours: '+(last.hours_worked||0).toFixed(2)+'h</p>':'<p>No records available.</p>')
        +'</body></html>';
    var w=window.open('','_blank');
    if(w){w.document.write(html);w.document.close();w.print()}
}

function openOvertimeRequest(){
    var modal=document.getElementById('tc-overtime-modal');
    if(modal)modal.style.display='flex';
    var dateEl=document.getElementById('ot-date');
    if(dateEl&&!dateEl.value)dateEl.value=new Date().toISOString().slice(0,10);
}

function closeOvertimeModal(){
    var modal=document.getElementById('tc-overtime-modal');
    if(modal)modal.style.display='none';
}

async function submitOvertimeRequest(){
    var date=document.getElementById('ot-date');
    var hours=document.getElementById('ot-hours');
    var reason=document.getElementById('ot-reason');
    if(!date||!date.value||!hours||!hours.value||!reason||!reason.value.trim()){
        showFeedback('Please fill date, hours and reason','error');
        return;
    }
    var payload={date:date.value,hours:parseFloat(hours.value),reason:reason.value};
    var data=await apiCall('/api/timeclock/overtime',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)});
    if(data){
        showFeedback('Overtime request submitted','success');
        closeOvertimeModal();
        loadOvertime();
    }
}

function prevWeek(){var lbl=document.getElementById('schedule-week-label');if(lbl)lbl.textContent='Previous week'}
function nextWeek(){var lbl=document.getElementById('schedule-week-label');if(lbl)lbl.textContent='Next week'}

function loadAll(){loadEmployee();loadRecords();loadOvertime();loadReports();loadJustifications();loadHolidays();loadAuditLog();loadTeam()}

window._tc={switchTab:switchTab,clockIn:clockIn,clockOut:clockOut,approveOT:approveOT,rejectOT:rejectOT,loadRecords:loadRecords,loadOvertime:loadOvertime,loadReports:loadReports,loadAll:loadAll,capturePhoto:capturePhoto,snapPhoto:snapPhoto,confirmPhoto:confirmPhoto,closePhotoModal:closePhotoModal,breakStart:breakStart,breakEnd:breakEnd,submitJustification:submitJustification,loadHolidays:loadHolidays,loadAuditLog:loadAuditLog,loadTeam:loadTeam,exportCsv:exportCsv,printReceipt:printReceipt,openOvertimeRequest:openOvertimeRequest,closeOvertimeModal:closeOvertimeModal,submitOvertimeRequest:submitOvertimeRequest,prevWeek:prevWeek,nextWeek:nextWeek};
clockInterval=setInterval(updateClock,1000);
updateClock();
checkGPS();
loadAll();
})();
