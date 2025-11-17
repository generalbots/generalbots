const sections = {
  drive: 'drive/drive.html',
  tasks: 'tasks/tasks.html',
  mail: 'mail/mail.html',
  chat: 'chat/chat.html',
};
const sectionCache = {};

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
    const cssPath = htmlPath.replace('.html', '.css');

    // Remove any existing section CSS
    document.querySelectorAll('link[data-section-css]').forEach(link => link.remove());

    // Load CSS first (skip if already loaded)
    let cssLink = document.querySelector(`link[href="${cssPath}"]`);
    if (!cssLink) {
      cssLink = document.createElement('link');
      cssLink.rel = 'stylesheet';
      cssLink.href = cssPath;
      cssLink.setAttribute('data-section-css', 'true');
      document.head.appendChild(cssLink);
    }

    // Hide previously loaded sections and show the requested one
    // Ensure a container exists for sections
    let container = document.getElementById('section-container');
    if (!container) {
      container = document.createElement('div');
      container.id = 'section-container';
      mainContent.appendChild(container);
    }

    const targetDiv = document.getElementById(`section-${section}`);

    if (targetDiv) {
      // Section already loaded: hide others, show this one
      container.querySelectorAll('.section').forEach(div => {
        div.style.display = 'none';
      });
      targetDiv.style.display = 'block';
    } else {
      // Show loading placeholder inside the container
      const loadingDiv = document.createElement('div');
      loadingDiv.className = 'loading';
      loadingDiv.textContent = 'Loading…';
      container.appendChild(loadingDiv);

      // Load HTML
      const html = await loadSectionHTML(htmlPath);
      // Create wrapper for the new section
      const wrapper = document.createElement('div');
      wrapper.id = `section-${section}`;
      wrapper.className = 'section';
      wrapper.innerHTML = html;

      // Hide any existing sections
      container.querySelectorAll('.section').forEach(div => {
        div.style.display = 'none';
      });

      // Remove loading placeholder
      container.removeChild(loadingDiv);

      // Add the new section to the container and cache it
      container.appendChild(wrapper);
      sectionCache[section] = wrapper;

      // Ensure the new section is visible
      wrapper.style.display = 'block';
    }

    // Then load JS after HTML is inserted (skip if already loaded)
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
