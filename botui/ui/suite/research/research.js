/**
 * Research Module JavaScript
 * AI-Powered Search & Discovery
 */
(function() {
    'use strict';

    // DOM Elements
    const elements = {
        searchInput: document.getElementById('search-input'),
        searchForm: document.getElementById('research-form'),
        suggestionsPanel: document.getElementById('suggestions-panel'),
        resultsContainer: document.getElementById('main-results'),
        sourcesPanel: document.getElementById('sources-panel'),
        sidebar: document.getElementById('research-sidebar'),
        focusModeInput: document.getElementById('focus-mode'),
        proModeInput: document.getElementById('pro-mode')
    };

    /**
     * Initialize the Research module
     */
    function init() {
        setupSearchInput();
        setupFocusModes();
        setupProSearch();
        setupSuggestionCards();
        setupPromptChips();
        setupSidebar();
        setupSourcesPanel();
        setupResultHandlers();
    }

    /**
     * Setup search input auto-resize and submit behavior
     */
    function setupSearchInput() {
        if (!elements.searchInput) return;

        // Auto-resize textarea
        elements.searchInput.addEventListener('input', function() {
            this.style.height = 'auto';
            this.style.height = Math.min(this.scrollHeight, 120) + 'px';
        });

        // Handle Enter to submit (Shift+Enter for new line)
        elements.searchInput.addEventListener('keydown', function(e) {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                elements.searchForm?.requestSubmit();
            }
        });
    }

    /**
     * Setup focus mode buttons
     */
    function setupFocusModes() {
        document.querySelectorAll('.focus-btn').forEach(btn => {
            btn.addEventListener('click', function() {
                document.querySelectorAll('.focus-btn').forEach(b => b.classList.remove('active'));
                this.classList.add('active');
                if (elements.focusModeInput) {
                    elements.focusModeInput.value = this.dataset.focus;
                }
            });
        });
    }

    /**
     * Setup pro search toggle
     */
    function setupProSearch() {
        const toggle = document.getElementById('pro-search-toggle');
        if (toggle && elements.proModeInput) {
            toggle.addEventListener('change', function() {
                elements.proModeInput.value = this.checked;
            });
        }
    }

    /**
     * Setup suggestion cards click behavior
     */
    function setupSuggestionCards() {
        document.querySelectorAll('.suggestion-card').forEach(card => {
            card.addEventListener('click', function() {
                if (elements.searchInput) {
                    elements.searchInput.value = this.dataset.query;
                    elements.searchInput.style.height = 'auto';
                    elements.searchInput.style.height = elements.searchInput.scrollHeight + 'px';
                    elements.searchForm?.requestSubmit();
                }
            });
        });
    }

    /**
     * Setup prompt chip buttons
     */
    function setupPromptChips() {
        const prefixes = {
            'explain': 'Explain ',
            'compare': 'Compare ',
            'summarize': 'Summarize ',
            'analyze': 'Analyze ',
            'pros-cons': 'What are the pros and cons of ',
            'how-to': 'How to '
        };

        document.querySelectorAll('.prompt-chip').forEach(chip => {
            chip.addEventListener('click', function() {
                const prompt = this.dataset.prompt;
                const currentValue = elements.searchInput?.value.trim() || '';

                if (elements.searchInput && prefixes[prompt]) {
                    elements.searchInput.value = prefixes[prompt] + currentValue;
                    elements.searchInput.focus();
                }
            });
        });
    }

    /**
     * Setup sidebar toggle
     */
    function setupSidebar() {
        const toggleBtn = document.getElementById('toggle-research-sidebar');
        if (toggleBtn && elements.sidebar) {
            toggleBtn.addEventListener('click', function() {
                elements.sidebar.classList.toggle('collapsed');
            });
        }

        // Collection item click
        document.addEventListener('click', function(e) {
            const collectionItem = e.target.closest('.collection-item');
            if (collectionItem) {
                const collectionId = collectionItem.dataset.id;
                if (typeof htmx !== 'undefined') {
                    htmx.ajax('GET', `/api/research/collections/${collectionId}`, {
                        target: '#main-results'
                    });
                }
            }
        });

        // Recent item click
        document.addEventListener('click', function(e) {
            const recentItem = e.target.closest('.recent-item');
            if (recentItem && elements.searchInput) {
                elements.searchInput.value = recentItem.dataset.query;
                elements.searchForm?.requestSubmit();
            }
        });

        // Source category click
        document.querySelectorAll('.source-category').forEach(cat => {
            cat.addEventListener('click', function() {
                const category = this.dataset.category;
                if (typeof htmx !== 'undefined') {
                    htmx.ajax('GET', `/api/research/sources?category=${category}`, {
                        target: '#sources-list'
                    });
                }
                elements.sourcesPanel?.classList.remove('hidden');
            });
        });
    }

    /**
     * Setup sources panel interactions
     */
    function setupSourcesPanel() {
        // View all sources
        document.addEventListener('click', function(e) {
            if (e.target.id === 'view-all-sources' || e.target.closest('#view-all-sources')) {
                elements.sourcesPanel?.classList.remove('hidden');
            }
        });

        // Close sources panel
        const closeBtn = document.getElementById('close-sources');
        if (closeBtn) {
            closeBtn.addEventListener('click', function() {
                elements.sourcesPanel?.classList.add('hidden');
            });
        }

        // Citation click handler
        document.addEventListener('click', function(e) {
            if (e.target.classList.contains('citation')) {
                const sourceNum = e.target.textContent;
                elements.sourcesPanel?.classList.remove('hidden');

                // Scroll to source in panel
                const sourceCard = elements.sourcesPanel?.querySelector(`[data-source="${sourceNum}"]`);
                if (sourceCard) {
                    sourceCard.scrollIntoView({ behavior: 'smooth', block: 'center' });
                    sourceCard.classList.add('highlight');
                    setTimeout(() => sourceCard.classList.remove('highlight'), 2000);
                }
            }
        });
    }

    /**
     * Setup result interaction handlers
     */
    function setupResultHandlers() {
        // Related question click
        document.addEventListener('click', function(e) {
            const relatedItem = e.target.closest('.related-item');
            if (relatedItem && elements.searchInput) {
                elements.searchInput.value = relatedItem.textContent.trim();
                elements.searchForm?.requestSubmit();
                window.scrollTo({ top: 0, behavior: 'smooth' });
            }
        });

        // Trending tag click
        document.addEventListener('click', function(e) {
            const trendingTag = e.target.closest('.trending-tag');
            if (trendingTag && elements.searchInput) {
                elements.searchInput.value = trendingTag.dataset.query || trendingTag.textContent.trim();
                elements.searchForm?.requestSubmit();
            }
        });

        // Copy answer
        document.addEventListener('click', function(e) {
            const copyBtn = e.target.closest('.action-btn[title="Copy"]');
            if (copyBtn) {
                const content = document.getElementById('answer-content');
                if (content) {
                    navigator.clipboard.writeText(content.innerText);

                    // Show feedback
                    const originalTitle = copyBtn.title;
                    copyBtn.title = 'Copied!';
                    setTimeout(() => copyBtn.title = originalTitle, 2000);
                }
            }
        });

        // Save to collection
        document.addEventListener('click', function(e) {
            const saveBtn = e.target.closest('.action-btn[title="Save to Collection"]');
            if (saveBtn) {
                showSaveToCollectionModal();
            }
        });

        // Export to Paper
        document.addEventListener('click', function(e) {
            const exportBtn = e.target.closest('.action-btn[title="Export to Paper"]');
            if (exportBtn) {
                const content = document.getElementById('answer-content');
                if (content && typeof htmx !== 'undefined') {
                    htmx.ajax('POST', '/api/ui/paper/import', {
                        values: {
                            content: content.innerHTML,
                            title: elements.searchInput?.value || 'Research Export'
                        }
                    }).then(() => {
                        // Navigate to Paper
                        window.location.hash = '#paper';
                    });
                }
            }
        });

        // Handle search results display
        if (typeof htmx !== 'undefined') {
            htmx.on('#main-results', 'htmx:afterSwap', function() {
                // Hide suggestions when results are shown
                elements.suggestionsPanel?.classList.add('hidden');

                // Update source counts
                updateSourceCounts();
            });
        }
    }

    /**
     * Update source counts in sidebar
     */
    function updateSourceCounts() {
        if (typeof htmx !== 'undefined') {
            htmx.ajax('GET', '/api/ui/research/source-counts', {
                swap: 'none'
            }).then(response => {
                // Update counts in sidebar if response contains them
            });
        }
    }

    /**
     * Show save to collection modal
     */
    function showSaveToCollectionModal() {
        // This would typically show a modal with collection options
        // For now, we'll use a simple prompt
        const collectionName = prompt('Enter collection name:');
        if (collectionName && typeof htmx !== 'undefined') {
            const content = document.getElementById('answer-content');
            htmx.ajax('POST', '/api/ui/research/collections/save', {
                values: {
                    collection: collectionName,
                    content: content?.innerHTML || '',
                    query: elements.searchInput?.value || ''
                }
            });
        }
    }

    /**
     * Toggle sidebar on mobile
     */
    window.toggleResearchSidebar = function() {
        elements.sidebar?.classList.toggle('open');
    };

    // Initialize on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    // Expose for external use
    window.Research = {
        search: function(query) {
            if (elements.searchInput) {
                elements.searchInput.value = query;
                elements.searchForm?.requestSubmit();
            }
        },
        setFocusMode: function(mode) {
            const btn = document.querySelector(`.focus-btn[data-focus="${mode}"]`);
            if (btn) btn.click();
        },
        toggleSidebar: function() {
            elements.sidebar?.classList.toggle('collapsed');
        }
    };
})();
