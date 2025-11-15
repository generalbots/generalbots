const sections = {
  drive: 'drive/index.html',
  tasks: 'tasks/index.html', 
  mail: 'mail/index.html'
};

async function loadSectionHTML(path) {
  const response = await fetch(path);
  if (!response.ok) throw new Error('Failed to load section');
  return await response.text();
}

async function switchSection(section) {
  const mainContent = document.getElementById('main-content');
  
  try {
    const html = await loadSectionHTML(sections[section]);
    mainContent.innerHTML = html;
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
