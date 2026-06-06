(function(){
var state={branches:[],stock:[],promos:[],suppliers:[],topProducts:[],currentTab:'dashboard',currentBranch:'all'};

function showFeedback(msg,type){
    var el=document.getElementById('retail-feedback');
    if(!el){el=document.createElement('div');el.id='retail-feedback';el.style.cssText='position:fixed;top:16px;right:16px;padding:10px 16px;border-radius:6px;font-size:13px;z-index:2000;transition:opacity .3s';document.body.appendChild(el)}
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
    document.querySelectorAll('.retail-tab').forEach(function(t){t.classList.remove('active')});
    document.querySelectorAll('.retail-panel').forEach(function(p){p.classList.remove('active')});
    if(btn)btn.classList.add('active');
    var panel=document.getElementById('panel-'+tab);
    if(panel)panel.classList.add('active');
}

function filterBranch(branchId){
    state.currentBranch=branchId;
    loadStock();
}

function updateStats(){
    var revEl=document.getElementById('stat-revenue');
    var ordEl=document.getElementById('stat-orders');
    var lsEl=document.getElementById('stat-low-stock');
    var prEl=document.getElementById('stat-promos');
    if(revEl)revEl.textContent='$'+state.stock.reduce(function(s,x){return s+(x.revenue||0)},0).toFixed(2);
    if(ordEl)ordEl.textContent=state.stock.length;
    if(lsEl)lsEl.textContent=state.stock.filter(function(s){return s.available<=s.min}).length;
    if(prEl)prEl.textContent=state.promos.filter(function(p){return p.status==='active'}).length;
}

function renderDashboard(){
    var tpEl=document.getElementById('top-products-list');
    if(tpEl){
        if(state.topProducts.length===0){tpEl.innerHTML='<div class="retail-empty">No data</div>'}
        else{tpEl.innerHTML=state.topProducts.slice(0,5).map(function(p){return '<div class="retail-list-row"><span>'+p.name+'</span><span class="retail-list-value">$'+p.revenue.toFixed(2)+'</span></div>'}).join('')}
    }
    var bpEl=document.getElementById('branch-performance');
    if(bpEl){
        if(state.branches.length===0){bpEl.innerHTML='<div class="retail-empty">No branches</div>'}
        else{bpEl.innerHTML=state.branches.map(function(b){return '<div class="retail-list-row"><span>'+b.name+'</span><span class="retail-list-value">$'+b.stock_value+'</span></div>'}).join('')}
    }
    var saEl=document.getElementById('stock-alerts');
    if(saEl){
        var lowStock=state.stock.filter(function(s){return s.available<=s.min});
        if(lowStock.length===0){saEl.innerHTML='<div class="retail-empty">All stock OK</div>'}
        else{saEl.innerHTML=lowStock.slice(0,5).map(function(s){return '<div class="retail-list-row"><span>'+s.product+' ('+s.sku+')</span><span class="retail-list-value retail-alert">'+s.available+' left</span></div>'}).join('')}
    }
}

function renderBranches(){
    var tbody=document.getElementById('branches-tbody');
    if(!tbody)return;
    if(state.branches.length===0){tbody.innerHTML='<tr><td colspan="7" class="retail-empty">No branches configured</td></tr>';return}
    tbody.innerHTML=state.branches.map(function(b){
        return '<tr>'
            +'<td>'+b.code+'</td>'
            +'<td>'+b.name+'</td>'
            +'<td>'+b.address+'</td>'
            +'<td>'+b.manager+'</td>'
            +'<td>$'+b.stock_value+'</td>'
            +'<td><span class="retail-badge '+(b.status==='active'?'active':'inactive')+'">'+b.status+'</span></td>'
            +'<td><button class="retail-btn" onclick="window._retail.editBranch(\''+b.id+'\')">Edit</button></td>'
            +'</tr>';
    }).join('');
}

function renderStock(){
    var tbody=document.getElementById('stock-tbody');
    if(!tbody)return;
    var filtered=state.stock;
    if(state.currentBranch!=='all')filtered=filtered.filter(function(s){return s.branch_id==state.currentBranch});
    if(filtered.length===0){tbody.innerHTML='<tr><td colspan="8" class="retail-empty">No stock entries</td></tr>';return}
    tbody.innerHTML=filtered.map(function(s){
        var status=s.available<=0?'out':s.available<=s.min?'low':'in';
        return '<tr>'
            +'<td>'+s.sku+'</td>'
            +'<td>'+s.product+'</td>'
            +'<td>'+s.branch_name+'</td>'
            +'<td>'+s.on_hand+'</td>'
            +'<td>'+s.reserved+'</td>'
            +'<td>'+s.available+'</td>'
            +'<td>'+s.min+'</td>'
            +'<td><span class="retail-badge '+status+'">'+status+'</span></td>'
            +'</tr>';
    }).join('');
}

function renderPromos(){
    var tbody=document.getElementById('promos-tbody');
    if(!tbody)return;
    if(state.promos.length===0){tbody.innerHTML='<tr><td colspan="6" class="retail-empty">No active promotions</td></tr>';return}
    tbody.innerHTML=state.promos.map(function(p){
        return '<tr>'
            +'<td>'+p.name+'</td>'
            +'<td>'+p.type+'</td>'
            +'<td>'+p.discount+'</td>'
            +'<td>'+p.valid_from+'</td>'
            +'<td>'+p.valid_to+'</td>'
            +'<td><span class="retail-badge '+(p.status==='active'?'active':'inactive')+'">'+p.status+'</span></td>'
            +'</tr>';
    }).join('');
}

function renderSuppliers(){
    var tbody=document.getElementById('suppliers-tbody');
    if(!tbody)return;
    if(state.suppliers.length===0){tbody.innerHTML='<tr><td colspan="6" class="retail-empty">No suppliers</td></tr>';return}
    tbody.innerHTML=state.suppliers.map(function(s){
        var stars='★'.repeat(Math.round(s.rating||0))+'☆'.repeat(5-Math.round(s.rating||0));
        return '<tr>'
            +'<td>'+s.cnpj+'</td>'
            +'<td>'+s.name+'</td>'
            +'<td>'+s.contact+'</td>'
            +'<td>'+s.email+'</td>'
            +'<td>'+s.lead_time_days+' days</td>'
            +'<td><span style="color:#fbbf24">'+stars+'</span></td>'
            +'</tr>';
    }).join('');
}

function renderPricing(){
    var grid=document.getElementById('pricing-grid');
    if(!grid)return;
    var rules=state.branches.flatMap(function(b){return b.pricing_rules||[]});
    if(rules.length===0){grid.innerHTML='<div class="retail-empty"><div class="retail-empty-icon">💲</div><h3>No Pricing Rules</h3><p>Configure price tiers and branch-specific pricing</p></div>';return}
    grid.innerHTML=rules.map(function(r){
        return '<div class="retail-card"><h4>'+r.name+'</h4><div class="retail-list-row"><span>Margin</span><span>'+r.margin+'%</span></div><div class="retail-list-row"><span>Min Margin</span><span>'+r.min_margin+'%</span></div><div class="retail-list-row"><span>Markup</span><span>'+r.markup+'×</span></div></div>';
    }).join('');
}

async function loadBranches(){
    var data=await apiCall('/api/retail/branches');
    if(data){state.branches=Array.isArray(data)?data:[];renderBranches();renderDashboard()}
}

async function loadStock(){
    var data=await apiCall('/api/retail/stock');
    if(data){state.stock=Array.isArray(data)?data:[];renderStock();updateStats();renderDashboard()}
}

async function loadPromos(){
    var data=await apiCall('/api/retail/promotions');
    if(data){state.promos=Array.isArray(data)?data:[];renderPromos();updateStats()}
}

async function loadSuppliers(){
    var data=await apiCall('/api/retail/suppliers');
    if(data){state.suppliers=Array.isArray(data)?data:[];renderSuppliers()}
}

async function loadTopProducts(){
    var data=await apiCall('/api/retail/top-products');
    if(data){state.topProducts=Array.isArray(data)?data:[];renderDashboard()}
}

function searchStock(q){
    var rows=document.querySelectorAll('#stock-tbody tr');
    rows.forEach(function(r){r.style.display=r.textContent.toLowerCase().includes(q.toLowerCase())?'':'none'});
}

function showCreateBranch(){showFeedback('Opening new branch form...','success')}
function showCreatePromo(){showFeedback('Opening promotion form...','success')}
function showCreateSupplier(){showFeedback('Opening supplier form...','success')}
function editBranch(id){showFeedback('Editing branch '+id,'success')}

function loadAll(){loadBranches();loadStock();loadPromos();loadSuppliers();loadTopProducts();renderPricing()}

window._retail={switchTab:switchTab,filterBranch:filterBranch,searchStock:searchStock,showCreateBranch:showCreateBranch,showCreatePromo:showCreatePromo,showCreateSupplier:showCreateSupplier,editBranch:editBranch,loadAll:loadAll};
loadAll();
})();
