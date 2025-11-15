// Overview component
class Overview {
  constructor() {
    this.element = document.createElement('div');
    this.element.className = 'overview-chart';
    this.render();
  }

  render() {
    this.element.innerHTML = `
      <div class="chart-container">
        <div class="flex justify-between items-end h-40">
          ${[100, 80, 60, 40, 20].map((h, i) => `
            <div class="chart-bar" 
                 style="height:${h}px;background-color:hsl(var(--chart-${(i%5)+1}))">
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }
}

// Initialize and mount the component
document.addEventListener('DOMContentLoaded', () => {
  const overview = new Overview();
  document.querySelector('.overview-chart').replaceWith(overview.element);
});
