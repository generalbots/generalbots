// DateRangePicker component
class DateRangePicker {
  constructor() {
    this.state = {
      startDate: new Date(),
      endDate: new Date()
    };
    this.element = document.createElement('div');
    this.element.className = 'date-range-picker';
    this.render();
    this.bindEvents();
  }

  render() {
    this.element.innerHTML = `
      <div class="flex items-center gap-2">
        <button class="start-date-btn">
          Start: ${this.formatDate(this.state.startDate)}
        </button>
        <span>to</span>
        <button class="end-date-btn">
          End: ${this.formatDate(this.state.endDate)}
        </button>
      </div>
    `;
  }

  bindEvents() {
    this.element.querySelector('.start-date-btn').addEventListener('click', () => {
      this.setStartDate();
    });

    this.element.querySelector('.end-date-btn').addEventListener('click', () => {
      this.setEndDate();
    });
  }

  setStartDate() {
    const input = prompt("Enter start date (YYYY-MM-DD)");
    if (input) {
      this.state.startDate = new Date(input);
      this.render();
      this.onDateChange();
    }
  }

  setEndDate() {
    const input = prompt("Enter end date (YYYY-MM-DD)");
    if (input) {
      this.state.endDate = new Date(input);
      this.render();
      this.onDateChange();
    }
  }

  formatDate(date) {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: '2-digit',
      year: 'numeric'
    });
  }

  onDateChange() {
    // To be implemented by parent
  }
}

// Initialize and mount the component
document.addEventListener('DOMContentLoaded', () => {
  const picker = new DateRangePicker();
  document.querySelector('.date-range-picker').replaceWith(picker.element);
});
