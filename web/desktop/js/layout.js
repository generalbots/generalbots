const sections = {
  drive: 'drive/drive.html',
  tasks: 'tasks/tasks.html',
  mail: 'mail/mail.html',
  dashboard: 'dashboard/dashboard.html',
  editor: 'editor/editor.html',
  player: 'player/player.html',
  paper: 'paper/paper.html',
  settings: 'settings/settings.html',
  tables: 'tables/tables.html',
  news: 'news/news.html'
};

async function loadSectionHTML(path) {
  const response = await fetch(path);
  if (!response.ok) throw new Error('Failed to load section');
  return await response.text();
}

async function switchSection(section) {
  const mainContent = document.getElementById('main-content');
  
  try {
    const htmlPath = sections[section];
    console.log('Loading section:', section, 'from', htmlPath);
    const cssPath = 
       htmlPath.replace('.html', '.css');
    
    // Remove any existing section CSS
    document.querySelectorAll('link[data-section-css]').forEach(link => link.remove());
    
    // Load CSS first
    const cssLink = document.createElement('link');
    cssLink.rel = 'stylesheet';
    cssLink.href = cssPath;
    cssLink.setAttribute('data-section-css', 'true');
    document.head.appendChild(cssLink);

    // First load HTML
    const html = await loadSectionHTML(htmlPath);
    mainContent.innerHTML = html;

    // Then load JS after HTML is inserted
    const jsPath = htmlPath.replace('.html', '.js');
    const existingScript = document.querySelector(`script[src="${jsPath}"]`);
    if (!existingScript) {
      const script = document.createElement('script');
      script.src = jsPath;
      script.defer = true;
      document.body.appendChild(script);
    }
    window.history.pushState({}, '', `#${section}`);
    Alpine.initTree(mainContent);
  } catch (err) {
    console.error('Error loading section:', err);
    mainContent.innerHTML = `<div class="error">Failed to load ${section} section</div>`;
  }
}

// Handle initial load based on URL hash
window.addEventListener('DOMContentLoaded', () => {
  const initialSection = window.location.hash.substring(1) || 'drive';
  switchSection(initialSection);
});

// Handle browser back/forward navigation
window.addEventListener('popstate', () => {
  const section = window.location.hash.substring(1) || 'drive';
  switchSection(section);
});
