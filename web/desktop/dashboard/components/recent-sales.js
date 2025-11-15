// RecentSales component
class RecentSales {
  constructor() {
    this.salesData = [
      { name: "Olivia Martin", email: "olivia.martin@email.com", amount: "+$1,999.00" },
      { name: "Jackson Lee", email: "jackson.lee@email.com", amount: "+$39.00" },
      { name: "Isabella Nguyen", email: "isabella.nguyen@email.com", amount: "+$299.00" },
      { name: "William Kim", email: "will@email.com", amount: "+$99.00" },
      { name: "Sofia Davis", email: "sofia.davis@email.com", amount: "+$39.00" }
    ];
    this.element = document.createElement('div');
    this.element.className = 'recent-sales-list';
    this.render();
  }

  render() {
    this.element.innerHTML = `
      <div class="sales-list">
        ${this.salesData.map(sale => `
          <div class="sale-item">
            <div class="sale-info">
              <div class="avatar">${sale.name[0]}</div>
              <div>
                <div class="name">${sale.name}</div>
                <div class="email">${sale.email}</div>
              </div>
            </div>
            <div class="amount">${sale.amount}</div>
          </div>
        `).join('')}
      </div>
    `;
  }
}

// Initialize and mount the component
document.addEventListener('DOMContentLoaded', () => {
  const recentSales = new RecentSales();
  document.querySelector('.recent-sales-list').replaceWith(recentSales.element);
});
