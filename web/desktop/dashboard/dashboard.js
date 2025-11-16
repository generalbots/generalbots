// Dashboard module JavaScript
document.addEventListener('DOMContentLoaded', () => {
  // Dashboard state
  const state = {
    dateRange: { 
      startDate: new Date(),
      endDate: new Date() 
    },
    salesData: [
      { name: "Olivia Martin", email: "olivia.martin@email.com", amount: "+$1,999.00" },
      { name: "Jackson Lee", email: "jackson.lee@email.com", amount: "+$39.00" },
      { name: "Isabella Nguyen", email: "isabella.nguyen@email.com", amount: "+$299.00" },
      { name: "William Kim", email: "will@email.com", amount: "+$99.00" },
      { name: "Sofia Davis", email: "sofia.davis@email.com", amount: "+$39.00" },
    ],
    cards: [
      { title: "Total Revenue", value: "$45,231.89", subtext: "+20.1% from last month" },
      { title: "Subscriptions", value: "+2350", subtext: "+180.1% from last month" },
      { title: "Sales", value: "+12,234", subtext: "+19% from last month" },
      { title: "Active Now", value: "+573", subtext: "+201 since last hour" },
    ]
  };

  // Initialize dashboard safely
  function init() {
    const ensure = setInterval(() => {
      const main = document.querySelector('#main-content');
      const section = main && main.querySelector('.cards-grid');
      const btn = main && main.querySelector('.download-btn');
      if (section && btn) {
        clearInterval(ensure);
        renderCards();
        btn.addEventListener('click', handleDownload);
      }
    }, 100);
  }

  // Render dashboard cards
  function renderCards() {
    const container = document.querySelector('.cards-grid');
    container.innerHTML = state.cards.map(card => `
      <div class="card">
        <h3>${card.title}</h3>
        <p class="value">${card.value}</p>
        <p class="subtext">${card.subtext}</p>
      </div>
    `).join('');
  }

  // Handle download button click
  function handleDownload() {
    console.log('Downloading dashboard data...');
  }

  // Format date helper
  function formatDate(date) {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: '2-digit',
      year: 'numeric'
    });
  }

  // Initialize dashboard
  document.addEventListener('DOMContentLoaded',()=>{init();});
});
