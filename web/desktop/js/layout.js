class Layout {
  static currentPage = 'chat';
  
  static init() {
    this.setCurrentPage();
    this.loadNavbar();
    this.setupNavigation();
  }

  static setCurrentPage() {
    const hash = window.location.hash.substring(1) || 'chat';
    this.currentPage = hash;
    this.updateContent();
  }

  static async loadNavbar() {
    try {
      const response = await fetch('shared/navbar.html');
      const html = await response.text();
      
      if (!document.querySelector('.navbar')) {
        document.body.insertAdjacentHTML('afterbegin', html);
      }
      
      document.querySelectorAll('.nav-link').forEach(link => {
        link.classList.toggle('active', link.dataset.target === this.currentPage);
      });
    } catch (error) {
      console.error('Failed to load navbar:', error);
    }
  }

  static updateContent() {
    // Add your content loading logic here
    // For example: fetch(`pages/${this.currentPage}.html`)
    // and update the main content area
  }

  static setupNavigation() {
    document.addEventListener('click', (e) => {
      const navLink = e.target.closest('.nav-link');
      if (navLink) {
        e.preventDefault();
        const target = navLink.dataset.target;
        window.location.hash = target;
        this.currentPage = target;
        this.loadNavbar();
        this.updateContent();
      }
    });
  }
}

// Initialize on load and also on navigation
Layout.init();
window.addEventListener('popstate', () => Layout.init());
