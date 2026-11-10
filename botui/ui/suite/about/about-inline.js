CSSLoader.loadMultiple(['about.css']);

async function loadAboutData() {
    const t = (key, fallback) => (window.gbT ? window.gbT(key, fallback) : fallback);
    try {
        const response = await fetch('/api/product');
        const config = await response.json();
        
        // Update version info
        const versionEl = document.getElementById('about-version');
        if (versionEl) versionEl.textContent = config.version || '6.1.0';
        
        // Update product name
        const nameEl = document.getElementById('about-product-name');
        if (nameEl) nameEl.textContent = config.name || 'General Bots';
        
        // Populate compiled features
        const compiledContainer = document.getElementById('compiled-features');
        if (compiledContainer && config.compiled_features) {
            compiledContainer.innerHTML = config.compiled_features
                .sort()
                .map(f => `<span class="feature-badge enabled">${f}</span>`)
                .join('');
        }
        
        // Populate enabled apps
        const enabledContainer = document.getElementById('enabled-apps');
        if (enabledContainer && config.apps) {
            enabledContainer.innerHTML = config.apps
                .sort()
                .map(a => `<span class="feature-badge active">${a}</span>`)
                .join('');
        }
        
        // Calculate disabled features
        const disabledContainer = document.getElementById('disabled-features');
        if (disabledContainer && config.compiled_features && config.apps) {
            const compiled = new Set(config.compiled_features);
            const enabled = new Set(config.apps.map(a => a.toLowerCase()));
            const disabled = [...compiled].filter(f => !enabled.has(f.toLowerCase()));
            
            if (disabled.length > 0) {
                disabledContainer.innerHTML = disabled
                    .sort()
                    .map(f => `<span class="feature-badge disabled">${f}</span>`)
                    .join('');
            } else {
                disabledContainer.innerHTML = `<span class="text-muted">${t('about-all-features-enabled', 'All compiled features are enabled')}</span>`;
            }
        }
        
        // Update copyright
        const copyrightEl = document.getElementById('about-copyright');
        if (copyrightEl) {
            copyrightEl.textContent = config.copyright || t('about-copyright', '© 2026 General Bots. All rights reserved.');
        }
        
    } catch (e) {
        console.warn('Failed to load product config:', e);
    }
}

loadAboutData();
window.addEventListener('localeChanged', loadAboutData);

// The dependency tree SVG is optional in a trimmed build; show a translated
// fallback instead of an English literal baked into an inline handler.
(function bindFeatureTreeFallback() {
    const img = document.getElementById('about-feature-tree');
    if (!img) return;
    img.addEventListener('error', function () {
        const fallback = document.createElement('p');
        fallback.className = 'text-muted';
        fallback.setAttribute('data-i18n', 'about-dependency-tree-missing');
        fallback.textContent = window.gbT
            ? window.gbT('about-dependency-tree-missing', 'Dependency tree not available')
            : 'Dependency tree not available';
        img.replaceWith(fallback);
    });
})();
