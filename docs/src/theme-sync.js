// Theme synchronization for SVG documents in object elements
(function() {
    'use strict';

    // Function to update SVG theme
    function updateSVGTheme(objectElement) {
        try {
            const svgDoc = objectElement.contentDocument;
            if (!svgDoc) return;

            const html = document.documentElement;
            const body = document.body;
            const svgRoot = svgDoc.documentElement;

            // Detect current theme
            const isRust = html.classList.contains('rust') || body.classList.contains('rust');
            const isLight = html.classList.contains('light') || body.classList.contains('light');
            const isDark = html.classList.contains('dark') ||
                          html.classList.contains('ayu') ||
                          html.classList.contains('navy') ||
                          html.classList.contains('coal') ||
                          body.classList.contains('theme--dark');

            // Set CSS variables in SVG
            if (isRust || isLight) {
                svgRoot.style.setProperty('--main-text-color', '#1a1a1a');
                svgRoot.style.setProperty('--secondary-text-color', '#666');
                svgRoot.style.setProperty('--arrow-color', '#666');
            } else if (isDark) {
                svgRoot.style.setProperty('--main-text-color', '#ffffff');
                svgRoot.style.setProperty('--secondary-text-color', '#b0b0b0');
                svgRoot.style.setProperty('--arrow-color', '#b0b0b0');
            }

            // Also try to call the SVG's internal updateTheme function if it exists
            if (svgDoc.defaultView && typeof svgDoc.defaultView.updateTheme === 'function') {
                svgDoc.defaultView.updateTheme();
            }
        } catch (e) {
            // Silent fail for cross-origin or other errors
            console.debug('Could not update SVG theme:', e);
        }
    }

    // Function to initialize all SVG objects
    function initializeSVGThemes() {
        const objects = document.querySelectorAll('object[type="image/svg+xml"]');
        objects.forEach(obj => {
            // Wait for object to load
            if (obj.contentDocument) {
                updateSVGTheme(obj);
            } else {
                obj.addEventListener('load', () => updateSVGTheme(obj));
            }
        });
    }

    // Watch for theme changes
    function watchThemeChanges() {
        const observer = new MutationObserver((mutations) => {
            // Check if class attribute changed
            const classChanged = mutations.some(m =>
                m.type === 'attributes' && m.attributeName === 'class'
            );

            if (classChanged) {
                initializeSVGThemes();
            }
        });

        // Observe both html and body elements
        observer.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ['class']
        });

        observer.observe(document.body, {
            attributes: true,
            attributeFilter: ['class']
        });
    }

    // Initialize on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            initializeSVGThemes();
            watchThemeChanges();
        });
    } else {
        initializeSVGThemes();
        watchThemeChanges();
    }

    // Re-initialize on mdBook theme change (mdBook specific)
    if (window.playground_text) {
        // mdBook is present
        const themeToggle = document.getElementById('theme-toggle');
        if (themeToggle) {
            themeToggle.addEventListener('click', () => {
                setTimeout(initializeSVGThemes, 100);
            });
        }

        // Also watch for theme picker changes
        const themeChoices = document.querySelectorAll('#theme-list button');
        themeChoices.forEach(btn => {
            btn.addEventListener('click', () => {
                setTimeout(initializeSVGThemes, 100);
            });
        });
    }

    // Expose function globally for manual updates
    window.updateAllSVGThemes = initializeSVGThemes;
})();
