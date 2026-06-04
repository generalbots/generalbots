const PosApp = {
    state: { products: [], cart: [], paymentMethod: 'cash', categories: [], activeCategory: 'all' },

    init() {
        document.getElementById('posSearch').addEventListener('input', e => this.filterProducts(e.target.value));
        this.loadProducts();
    },

    async api(path, opts) {
        const token = localStorage.getItem('gb_token');
        const res = await fetch(path, { headers: { 'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json' }, ...opts });
        if (!res.ok) throw new Error('API error: ' + res.status);
        return res.json();
    },

    async loadProducts() {
        try {
            const data = await this.api('/api/pos/products');
            this.state.products = Array.isArray(data) ? data : (data.items || []);
            this.buildCategories();
            this.renderProducts();
        } catch (e) {
            document.getElementById('pos-grid').innerHTML = '<div class="loading-row">Failed to load products</div>';
        }
    },

    buildCategories() {
        const cats = new Set(this.state.products.map(p => p.category).filter(Boolean));
        this.state.categories = ['all', ...cats];
        document.getElementById('pos-categories').innerHTML = this.state.categories.map(c =>
            `<button class="pos-cat-btn ${c === 'all' ? 'active' : ''}" onclick="PosApp.filterCategory('${c}',this)">${c === 'all' ? 'All' : this.esc(c)}</button>`
        ).join('');
    },

    filterCategory(cat, el) {
        this.state.activeCategory = cat;
        document.querySelectorAll('.pos-cat-btn').forEach(b => b.classList.remove('active'));
        el.classList.add('active');
        this.renderProducts();
    },

    renderProducts() {
        const query = (document.getElementById('posSearch').value || '').toLowerCase();
        let products = this.state.products;
        if (this.state.activeCategory !== 'all') {
            products = products.filter(p => p.category === this.state.activeCategory);
        }
        if (query) {
            products = products.filter(p =>
                (p.name || '').toLowerCase().includes(query) ||
                (p.sku || '').toLowerCase().includes(query)
            );
        }
        const grid = document.getElementById('pos-grid');
        if (!products.length) { grid.innerHTML = '<div class="loading-row">No products found</div>'; return; }
        grid.innerHTML = products.map(p => {
            const stock = p.stock || 0;
            const stockClass = stock <= 0 ? 'out' : stock <= 5 ? 'low' : 'in';
            const stockText = stock <= 0 ? 'Out of stock' : stock <= 5 ? `${stock} left` : `${stock} in stock`;
            return `
                <div class="pos-product-card ${stock <= 0 ? 'out-of-stock' : ''}" onclick="PosApp.addToCart('${p.id}')">
                    <div class="pos-product-img">
                        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
                    </div>
                    <span class="pos-product-name">${this.esc(p.name)}</span>
                    <span class="pos-product-price">${this.fmt(p.price)}</span>
                    <span class="pos-product-stock ${stockClass}">${stockText}</span>
                </div>
            `;
        }).join('');
    },

    filterProducts(q) {
        this.renderProducts();
    },

    addToCart(id) {
        const product = this.state.products.find(p => p.id === id);
        if (!product || (product.stock || 0) <= 0) return;
        const existing = this.state.cart.find(i => i.id === id);
        if (existing) {
            if (existing.qty < (product.stock || 0)) existing.qty++;
        } else {
            this.state.cart.push({ id: product.id, name: product.name, price: product.price || 0, qty: 1 });
        }
        this.renderCart();
    },

    removeFromCart(id) {
        this.state.cart = this.state.cart.filter(i => i.id !== id);
        this.renderCart();
    },

    updateQty(id, delta) {
        const item = this.state.cart.find(i => i.id === id);
        const product = this.state.products.find(p => p.id === id);
        if (!item) return;
        item.qty += delta;
        if (item.qty <= 0) { this.removeFromCart(id); return; }
        if (product && item.qty > (product.stock || 0)) item.qty = product.stock || 0;
        this.renderCart();
    },

    renderCart() {
        const items = this.state.cart;
        const count = items.reduce((s, i) => s + i.qty, 0);
        document.getElementById('cart-count').textContent = count + ' item' + (count !== 1 ? 's' : '');
        const container = document.getElementById('cart-items');
        if (!items.length) {
            container.innerHTML = `<div class="pos-cart-empty"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="9" cy="21" r="1"/><circle cx="20" cy="21" r="1"/><path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"/></svg><p>Cart is empty</p></div>`;
        } else {
            container.innerHTML = items.map(i => `
                <div class="pos-cart-item">
                    <div class="pos-cart-item-info">
                        <div class="pos-cart-item-name">${this.esc(i.name)}</div>
                        <div class="pos-cart-item-price">${this.fmt(i.price)} each</div>
                    </div>
                    <div class="pos-cart-qty">
                        <button onclick="PosApp.updateQty('${i.id}',-1)">-</button>
                        <span>${i.qty}</span>
                        <button onclick="PosApp.updateQty('${i.id}',1)">+</button>
                    </div>
                    <span class="pos-cart-item-total">${this.fmt(i.price * i.qty)}</span>
                    <button class="pos-cart-item-remove" onclick="PosApp.removeFromCart('${i.id}')">&times;</button>
                </div>
            `).join('');
        }
        const subtotal = items.reduce((s, i) => s + i.price * i.qty, 0);
        const tax = subtotal * 0.1;
        const total = subtotal + tax;
        document.getElementById('cart-subtotal').textContent = this.fmt(subtotal);
        document.getElementById('cart-tax').textContent = this.fmt(tax);
        document.getElementById('cart-total').textContent = this.fmt(total);
        document.getElementById('pos-confirm').disabled = !items.length;
    },

    setPayment(method, el) {
        this.state.paymentMethod = method;
        document.querySelectorAll('.pos-pay-btn').forEach(b => b.classList.remove('active'));
        el.classList.add('active');
    },

    async confirmOrder() {
        if (!this.state.cart.length) return;
        const subtotal = this.state.cart.reduce((s, i) => s + i.price * i.qty, 0);
        const tax = subtotal * 0.1;
        const total = subtotal + tax;
        const order = {
            items: this.state.cart.map(i => ({ product_id: i.id, name: i.name, price: i.price, qty: i.qty })),
            subtotal, tax, total,
            payment_method: this.state.paymentMethod
        };
        try {
            await this.api('/api/pos/orders', { method: 'POST', body: JSON.stringify(order) });
            this.state.cart = [];
            this.renderCart();
            this.loadProducts();
            alert('Order confirmed!');
        } catch (e) {
            alert('Order failed: ' + e.message);
        }
    },

    clearCart() {
        this.state.cart = [];
        this.renderCart();
    },

    fmt(v) { return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(v || 0); },
    esc(s) { const d = document.createElement('div'); d.textContent = s; return d.innerHTML; }
};

document.addEventListener('DOMContentLoaded', () => PosApp.init());
