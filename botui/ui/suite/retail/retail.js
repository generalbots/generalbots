(function(){
if (window.GBAppLifecycle) GBAppLifecycle.begin("retail");
var state={branches:[],stock:[],promos:[],suppliers:[],topProducts:[],currentTab:'dashboard',currentBranch:'all',editingBranchId:null,editingPromoId:null};

function escapeHtml(value){
    if (window.GBShared && window.GBShared.html && typeof window.GBShared.html.escape === 'function') {
        return window.GBShared.html.escape(value);
    }
    return String(value == null ? '' : value)
        .replace(/&/g,'&amp;')
        .replace(/</g,'&lt;')
        .replace(/>/g,'&gt;')
        .replace(/"/g,'&quot;')
        .replace(/'/g,'&#39;');
}

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
    var filteredStock = state.stock;
    if(state.currentBranch && state.currentBranch !== 'all'){
        filteredStock = state.stock.filter(function(s){return s.branch_id == state.currentBranch || s.branch_id === state.currentBranch;});
    }
    var rev = 0;
    if(state.topProducts && state.topProducts.length) rev = state.topProducts.reduce(function(s,x){return s + (Number(x.revenue)||0);}, 0);
    if(revEl)revEl.textContent='$'+rev.toFixed(2);
    if(ordEl)ordEl.textContent = String(filteredStock.length);
    if(lsEl)lsEl.textContent = String(filteredStock.filter(function(s){return (Number(s.available)||0) <= (Number(s.min)||0)}).length);
    if(prEl)prEl.textContent = String((state.promos||[]).filter(function(p){return p.status==='active'}).length);
}

function renderDashboard(){
    var tpEl=document.getElementById('top-products-list');
    if(tpEl){
        if(state.topProducts.length===0){tpEl.innerHTML='<div class="retail-empty">No data</div>'}
        else{tpEl.innerHTML=state.topProducts.slice(0,5).map(function(p){return '<div class="retail-list-row"><span>'+escapeHtml(p.name||'')+'</span><span class="retail-list-value">$'+escapeHtml((Number(p.revenue)||0).toFixed(2))+'</span></div>'}).join('')}
    }
    var bpEl=document.getElementById('branch-performance');
    if(bpEl){
        if(state.branches.length===0){bpEl.innerHTML='<div class="retail-empty">No branches</div>'}
        else{bpEl.innerHTML=state.branches.map(function(b){return '<div class="retail-list-row"><span>'+escapeHtml(b.name||'')+'</span><span class="retail-list-value">$'+escapeHtml((Number(b.stock_value)||0).toFixed(2))+'</span></div>'}).join('')}
    }
    var saEl=document.getElementById('stock-alerts');
    if(saEl){
        var lowStock=state.stock.filter(function(s){return Number(s.available||0)<=Number(s.min||0)});
        if(lowStock.length===0){saEl.innerHTML='<div class="retail-empty">All stock OK</div>'}
        else{saEl.innerHTML=lowStock.slice(0,5).map(function(s){return '<div class="retail-list-row"><span>'+escapeHtml(s.product||'')+' ('+escapeHtml(s.sku||'')+')</span><span class="retail-list-value retail-alert">'+escapeHtml(String(s.available||0))+' left</span></div>'}).join('')}
    }
}

function renderBranches(){
    var tbody=document.getElementById('branches-tbody');
    if(!tbody)return;
    if(state.branches.length===0){tbody.innerHTML='<tr><td colspan="7" class="retail-empty">No branches configured</td></tr>';return}
    tbody.innerHTML=state.branches.map(function(b){
        var status = (b.status==='active')?'active':'inactive';
        var badge = (status==='active')?'active':'inactive';
        return '<tr>'
            +'<td>'+escapeHtml(b.code||'')+'</td>'
            +'<td>'+escapeHtml(b.name||'')+'</td>'
            +'<td>'+escapeHtml(b.address||'')+'</td>'
            +'<td>'+escapeHtml(b.manager||'')+'</td>'
            +'<td>$'+escapeHtml(String(b.stock_value||''))+'</td>'
            +'<td><span class="retail-badge '+badge+'">'+escapeHtml(status)+'</span></td>'
            +'<td><button class="retail-btn" type="button" data-retail-edit-branch="'+escapeHtml(String(b.id==null?'':b.id))+'">Edit</button></td>'
            +'</tr>';
    }).join('');
    tbody.querySelectorAll('[data-retail-edit-branch]').forEach(function(btn){
        btn.addEventListener('click', function(){ editBranch(this.getAttribute('data-retail-edit-branch')); });
    });
}

function renderStock(){
    var tbody=document.getElementById('stock-tbody');
    if(!tbody)return;
    var filtered=state.stock;
    if(state.currentBranch!=='all')filtered=filtered.filter(function(s){return String(s.branch_id)==String(state.currentBranch)});
    if(filtered.length===0){tbody.innerHTML='<tr><td colspan="8" class="retail-empty">No stock entries</td></tr>';return}
    tbody.innerHTML=filtered.map(function(s){
        var avail = Number(s.available||0), min = Number(s.min||0);
        var status=avail<=0?'out':avail<=min?'low':'in';
        return '<tr>'
            +'<td>'+escapeHtml(s.sku||'')+'</td>'
            +'<td>'+escapeHtml(s.product||'')+'</td>'
            +'<td>'+escapeHtml(s.branch_name||'')+'</td>'
            +'<td>'+escapeHtml(String(s.on_hand==null?'':s.on_hand))+'</td>'
            +'<td>'+escapeHtml(String(s.reserved==null?'':s.reserved))+'</td>'
            +'<td>'+escapeHtml(String(s.available==null?'':s.available))+'</td>'
            +'<td>'+escapeHtml(String(s.min==null?'':s.min))+'</td>'
            +'<td><span class="retail-badge '+escapeHtml(status)+'">'+escapeHtml(status)+'</span></td>'
            +'</tr>';
    }).join('');
}

function renderPromos(){
    var tbody=document.getElementById('promos-tbody');
    if(!tbody)return;
    if(state.promos.length===0){tbody.innerHTML='<tr><td colspan="6" class="retail-empty">No active promotions</td></tr>';return}
    tbody.innerHTML=state.promos.map(function(p){
        var status = (p.status==='active')?'active':'inactive';
        return '<tr>'
            +'<td>'+escapeHtml(p.name||'')+'</td>'
            +'<td>'+escapeHtml(p.type||'')+'</td>'
            +'<td>'+escapeHtml(String(p.discount==null?'':p.discount))+'</td>'
            +'<td>'+escapeHtml(String(p.valid_from==null?'':p.valid_from))+'</td>'
            +'<td>'+escapeHtml(String(p.valid_to==null?'':p.valid_to))+'</td>'
            +'<td><span class="retail-badge '+status+'">'+escapeHtml(status)+'</span></td>'
            +'</tr>';
    }).join('');
}

function renderSuppliers(){
    var tbody=document.getElementById('suppliers-tbody');
    if(!tbody)return;
    if(state.suppliers.length===0){tbody.innerHTML='<tr><td colspan="6" class="retail-empty">No suppliers</td></tr>';return}
    tbody.innerHTML=state.suppliers.map(function(s){
        var rating = Math.max(0, Math.min(5, Math.round(Number(s.rating||0))));
        var stars='★'.repeat(rating)+'☆'.repeat(5-rating);
        return '<tr>'
            +'<td>'+escapeHtml(s.cnpj||'')+'</td>'
            +'<td>'+escapeHtml(s.name||'')+'</td>'
            +'<td>'+escapeHtml(s.contact||'')+'</td>'
            +'<td>'+escapeHtml(s.email||'')+'</td>'
            +'<td>'+escapeHtml(String(s.lead_time_days==null?'':s.lead_time_days))+' days</td>'
            +'<td><span style="color:#fbbf24">'+escapeHtml(stars)+'</span></td>'
            +'</tr>';
    }).join('');
}

function renderPricing(){
    var grid=document.getElementById('pricing-grid');
    if(!grid)return;
    var rules=state.branches.flatMap(function(b){return b.pricing_rules||[]});
    if(rules.length===0){grid.innerHTML='<div class="retail-empty"><div class="retail-empty-icon">💲</div><h3>No Pricing Rules</h3><p>Configure price tiers and branch-specific pricing</p></div>';return}
    grid.innerHTML=rules.map(function(r){
        return '<div class="retail-card"><h4>'+escapeHtml(r.name||'')+'</h4><div class="retail-list-row"><span>Margin</span><span>'+escapeHtml(String(r.margin==null?'':r.margin))+'%</span></div><div class="retail-list-row"><span>Min Margin</span><span>'+escapeHtml(String(r.min_margin==null?'':r.min_margin))+'%</span></div><div class="retail-list-row"><span>Markup</span><span>'+escapeHtml(String(r.markup==null?'':r.markup))+'×</span></div></div>';
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

async function apiPost(url, body){
    try{
        var r=await fetch(url,{method:'POST',headers:{'Content-Type':'application/json','Authorization':'Bearer '+(localStorage.getItem('gb_token')||'')},body:JSON.stringify(body)});
        if(!r.ok)throw new Error('HTTP '+r.status);
        var t=await r.text();return t?JSON.parse(t):null;
    }catch(e){showFeedback('API error: '+e.message,'error');return null}
}

function openModal(id){var el=document.getElementById(id);if(el)el.style.display='flex';}
function closeModal(id){var el=document.getElementById(id);if(el)el.style.display='none';}

function showCreateBranch(){state.editingBranchId=null;['branchCode','branchName','branchAddress','branchManager','branchStock','branchStatus'].forEach(function(i){var e=document.getElementById(i);if(e)e.value=(i==='branchStatus')?'active':'';});openModal('retailBranchModal');}
function showCreatePromo(){state.editingPromoId=null;['promoName','promoType','promoDiscount','promoFrom','promoTo','promoStatus'].forEach(function(i){var e=document.getElementById(i);if(e)e.value=(i==='promoStatus')?'active':'';});openModal('retailPromoModal');}
function showCreateSupplier(){['supplierCnpj','supplierName','supplierContact','supplierEmail','supplierLead','supplierRating'].forEach(function(i){var e=document.getElementById(i);if(e)e.value='';});openModal('retailSupplierModal');}

async function editBranch(id){
    var data=await apiCall('/api/retail/branches');
    var list=Array.isArray(data)?data:(data&&data.items)||[];
    var b=list.find(function(x){return x.id===id;});
    if(!b){showFeedback('Branch not found','error');return;}
    state.editingBranchId=id;
    var set=function(i,v){var e=document.getElementById(i);if(e)e.value=v||'';};
    set('branchCode',b.code);set('branchName',b.name);set('branchAddress',b.address);set('branchManager',b.manager);
    set('branchStock',b.stock_value);set('branchStatus',b.status);
    openModal('retailBranchModal');
}

async function submitBranch(e){
    if(e)e.preventDefault();
    var name=document.getElementById('branchName').value.trim();
    if(!name){showFeedback('Name is required','error');return;}
    var payload={code:document.getElementById('branchCode').value.trim(),name:name,address:document.getElementById('branchAddress').value.trim(),manager:document.getElementById('branchManager').value.trim(),status:document.getElementById('branchStatus').value,stock_value:document.getElementById('branchStock').value};
    var url=state.editingBranchId?('/api/retail/branches/'+state.editingBranchId):'/api/retail/branches';
    var method=state.editingBranchId?'PUT':'POST';
    try{
        var r=await fetch(url,{method:method,headers:{'Content-Type':'application/json','Authorization':'Bearer '+(localStorage.getItem('gb_token')||'')},body:JSON.stringify(payload)});
        if(!r.ok)throw new Error('HTTP '+r.status);
        closeModal('retailBranchModal');showFeedback('Branch saved','success');loadBranches();
    }catch(err){showFeedback('Save failed: '+err.message,'error');}
}

async function submitPromo(e){
    if(e)e.preventDefault();
    var name=document.getElementById('promoName').value.trim();
    if(!name){showFeedback('Name is required','error');return;}
    var payload={name:name,type:document.getElementById('promoType').value,discount:document.getElementById('promoDiscount').value,valid_from:document.getElementById('promoFrom').value||null,valid_to:document.getElementById('promoTo').value||null,status:document.getElementById('promoStatus').value};
    var res=await apiPost('/api/retail/promotions',payload);
    if(res){closeModal('retailPromoModal');showFeedback('Promotion created','success');loadPromos();}
}

async function submitSupplier(e){
    if(e)e.preventDefault();
    var name=document.getElementById('supplierName').value.trim();
    if(!name){showFeedback('Name is required','error');return;}
    var payload={cnpj:document.getElementById('supplierCnpj').value.trim(),name:name,contact:document.getElementById('supplierContact').value.trim(),email:document.getElementById('supplierEmail').value.trim(),lead_time_days:parseInt(document.getElementById('supplierLead').value||'0',10),rating:parseFloat(document.getElementById('supplierRating').value||'0')};
    var res=await apiPost('/api/retail/suppliers',payload);
    if(res){closeModal('retailSupplierModal');showFeedback('Supplier created','success');loadSuppliers();}
}

function loadAll(){loadBranches();loadStock();loadPromos();loadSuppliers();loadTopProducts();renderPricing()}

window._retail={switchTab:switchTab,filterBranch:filterBranch,searchStock:searchStock,showCreateBranch:showCreateBranch,showCreatePromo:showCreatePromo,showCreateSupplier:showCreateSupplier,editBranch:editBranch,loadAll:loadAll,submitBranch:submitBranch,submitPromo:submitPromo,submitSupplier:submitSupplier,openModal:openModal,closeModal:closeModal};
loadAll();
})();
