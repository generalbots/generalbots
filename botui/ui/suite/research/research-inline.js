"use strict";

if (window.GBAppLifecycle) GBAppLifecycle.begin("research");
(function () {
        const searchInput = document.getElementById("search-input");
        const suggestionsPanel = document.getElementById("suggestions-panel");
        const resultsContainer = document.getElementById("main-results");
        const sourcesPanel = document.getElementById("sources-panel");
        const sidebar = document.getElementById("research-sidebar");

        // Auto-resize textarea
        searchInput.addEventListener("input", function () {
            this.style.height = "auto";
            this.style.height = Math.min(this.scrollHeight, 120) + "px";
        });

        // Handle Enter to submit (Shift+Enter for new line)
        searchInput.addEventListener("keydown", function (e) {
            if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                document.getElementById("research-form").requestSubmit();
            }
        });

        // Focus modes
        document.querySelectorAll(".focus-btn").forEach((btn) => {
            btn.addEventListener("click", function () {
                document
                    .querySelectorAll(".focus-btn")
                    .forEach((b) => b.classList.remove("active"));
                this.classList.add("active");
                document.getElementById("focus-mode").value =
                    this.dataset.focus;
            });
        });

        // Pro search toggle
        document
            .getElementById("pro-search-toggle")
            .addEventListener("change", function () {
                document.getElementById("pro-mode").value = this.checked;
            });

        // Suggestion cards
        document.querySelectorAll(".suggestion-card").forEach((card) => {
            card.addEventListener("click", function () {
                searchInput.value = this.dataset.query;
                searchInput.style.height = "auto";
                searchInput.style.height = searchInput.scrollHeight + "px";
                document.getElementById("research-form").requestSubmit();
            });
        });

        // Prompt chips
        document.querySelectorAll(".prompt-chip").forEach((chip) => {
            chip.addEventListener("click", function () {
                const prompt = this.dataset.prompt;
                const currentValue = searchInput.value.trim();

                const prefixes = {
                    explain: "Explain ",
                    compare: "Compare ",
                    summarize: "Summarize ",
                    analyze: "Analyze ",
                    "pros-cons": "What are the pros and cons of ",
                    "how-to": "How to ",
                };

                searchInput.value = prefixes[prompt] + currentValue;
                searchInput.focus();
            });
        });

        // Toggle sidebar
        document
            .getElementById("toggle-research-sidebar")
            .addEventListener("click", function () {
                sidebar.classList.toggle("collapsed");
            });

        // View all sources
        document.addEventListener("click", function (e) {
            if (
                e.target.id === "view-all-sources" ||
                e.target.closest("#view-all-sources")
            ) {
                sourcesPanel.classList.remove("hidden");
            }
        });

        // Close sources panel
        document
            .getElementById("close-sources")
            .addEventListener("click", function () {
                sourcesPanel.classList.add("hidden");
            });

        // Citation click handler
        document.addEventListener("click", function (e) {
            if (e.target.classList.contains("citation")) {
                const sourceNum = e.target.textContent;
                sourcesPanel.classList.remove("hidden");

                // Scroll to source in panel
                const sourceCard = sourcesPanel.querySelector(
                    `[data-source="${sourceNum}"]`,
                );
                if (sourceCard) {
                    sourceCard.scrollIntoView({
                        behavior: "smooth",
                        block: "center",
                    });
                    sourceCard.classList.add("highlight");
                    setTimeout(
                        () => sourceCard.classList.remove("highlight"),
                        2000,
                    );
                }
            }
        });

        // Related question click
        document.addEventListener("click", function (e) {
            const relatedItem = e.target.closest(".related-item");
            if (relatedItem) {
                searchInput.value = relatedItem.textContent.trim();
                document.getElementById("research-form").requestSubmit();
                window.scrollTo({ top: 0, behavior: "smooth" });
            }
        });

        // Trending tag click
        document.addEventListener("click", function (e) {
            const trendingTag = e.target.closest(".trending-tag");
            if (trendingTag) {
                searchInput.value =
                    trendingTag.dataset.query || trendingTag.textContent.trim();
                document.getElementById("research-form").requestSubmit();
            }
        });

        // Copy answer
        document.addEventListener("click", function (e) {
            const copyBtn = e.target.closest('.action-btn[title="Copy"]');
            if (copyBtn) {
                const content = document.getElementById("answer-content");
                if (content) {
                    navigator.clipboard.writeText(content.innerText);

                    // Show feedback
                    const originalTitle = copyBtn.title;
                    copyBtn.title = "Copied!";
                    setTimeout(() => (copyBtn.title = originalTitle), 2000);
                }
            }
        });

        // Export to Paper
        document.addEventListener("click", function (e) {
            const exportBtn = e.target.closest(
                '.action-btn[title="Export to Paper"]',
            );
            if (exportBtn) {
                const content = document.getElementById("answer-content");
                if (content) {
                    // Send to Paper via HTMX
                    htmx.ajax("POST", "/api/ui/paper/import", {
                        values: {
                            content: content.innerHTML,
                            title: searchInput.value,
                        },
                    }).then(() => {
                        // Navigate to Paper
                        window.location.hash = "#paper";
                    });
                }
            }
        });

        // Source category click
        document.querySelectorAll(".source-category").forEach((cat) => {
            cat.addEventListener("click", function () {
                const category = this.dataset.category;
                htmx.ajax("GET", `/api/ui/research/sources?category=${category}`, {
                    target: "#sources-list",
                });
                sourcesPanel.classList.remove("hidden");
            });
        });

        // Handle search results display
        htmx.on("#main-results", "htmx:afterSwap", function () {
            // Hide suggestions when results are shown
            suggestionsPanel.classList.add("hidden");

            // Update source counts
            updateSourceCounts();
        });

        function updateSourceCounts() {
            htmx.ajax("GET", "/api/ui/research/source-counts", {
                swap: "none",
            }).then((response) => {
                // Update counts in sidebar
            });
        }

        // Collection item click
        document.addEventListener("click", function (e) {
            const collectionItem = e.target.closest(".collection-item");
            if (collectionItem) {
                const collectionId = collectionItem.dataset.id;
                htmx.ajax("GET", `/api/ui/research/collections/${collectionId}`, {
                    target: "#main-results",
                });
            }
        });

        // Recent item click
        document.addEventListener("click", function (e) {
            const recentItem = e.target.closest(".recent-item");
            if (recentItem) {
                searchInput.value = recentItem.dataset.query;
                document.getElementById("research-form").requestSubmit();
            }
        });

        // Mobile sidebar toggle
        window.toggleResearchSidebar = function () {
            sidebar.classList.toggle("open");
        };
    })();
