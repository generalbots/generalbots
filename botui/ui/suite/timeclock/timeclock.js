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

function loadAll(){loadEmployee();loadRecords();loadOvertime();loadReports()}

window._tc={switchTab:switchTab,clockIn:clockIn,clockOut:clockOut,approveOT:approveOT,rejectOT:rejectOT,loadRecords:loadRecords,loadOvertime:loadOvertime,loadReports:loadReports,loadAll:loadAll};
clockInterval=setInterval(updateClock,1000);
updateClock();
checkGPS();
loadAll();
})();
