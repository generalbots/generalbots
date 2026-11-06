            // Bind the taskbar universal search (mount on first paint)
            (function () {
                var bindSearch = function () {
                    if (window.productConfig && window.productConfig.search_enabled === false) { return; }
                    var input = document.getElementById("taskbarSearch");
                    if (input && window.GBUniversalSearch) {
                        window.GBUniversalSearch.mount(input);
                    }
                };
                if (document.readyState === "loading") {
                    document.addEventListener("DOMContentLoaded", bindSearch);
                } else {
                    bindSearch();
                }
            })();
        
