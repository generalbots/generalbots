/**
 * CSS Lazy Loader - Efficient on-demand stylesheet loading
 * Prevents duplicate loads and handles caching automatically
 */
const CSSLoader = (function () {
    const loadedStyles = new Set();
    const loadingPromises = new Map();

    function getAbsoluteUrl(href) {
        if (href.startsWith("http://") || href.startsWith("https://")) {
            return href;
        }
        const base = window.location.pathname.includes("/suite/")
            ? "/suite/"
            : "/";
        if (href.startsWith("/")) {
            return href;
        }
        return base + href;
    }

    function load(href, options = {}) {
        const absoluteUrl = getAbsoluteUrl(href);

        if (loadedStyles.has(absoluteUrl)) {
            return Promise.resolve();
        }

        if (loadingPromises.has(absoluteUrl)) {
            return loadingPromises.get(absoluteUrl);
        }

        const promise = new Promise((resolve, reject) => {
            const existingLink = document.querySelector(
                `link[href="${href}"], link[href="${absoluteUrl}"]`
            );
            if (existingLink) {
                loadedStyles.add(absoluteUrl);
                resolve();
                return;
            }

            const link = document.createElement("link");
            link.rel = "stylesheet";
            link.href = href;

            if (options.media) {
                link.media = options.media;
            }

            if (options.crossOrigin) {
                link.crossOrigin = options.crossOrigin;
            }

            link.onload = function () {
                loadedStyles.add(absoluteUrl);
                loadingPromises.delete(absoluteUrl);
                resolve();
            };

            link.onerror = function () {
                loadingPromises.delete(absoluteUrl);
                reject(new Error(`Failed to load CSS: ${href}`));
            };

            const insertPoint =
                options.insertAfter ||
                document.querySelector('link[rel="stylesheet"]:last-of-type') ||
                document.head.lastChild;

            if (insertPoint && insertPoint.parentNode) {
                insertPoint.parentNode.insertBefore(
                    link,
                    insertPoint.nextSibling
                );
            } else {
                document.head.appendChild(link);
            }
        });

        loadingPromises.set(absoluteUrl, promise);
        return promise;
    }

    function loadMultiple(hrefs, options = {}) {
        return Promise.all(hrefs.map((href) => load(href, options)));
    }

    function preload(href) {
        const absoluteUrl = getAbsoluteUrl(href);

        if (loadedStyles.has(absoluteUrl)) {
            return;
        }

        const existingPreload = document.querySelector(
            `link[rel="preload"][href="${href}"]`
        );
        if (existingPreload) {
            return;
        }

        const link = document.createElement("link");
        link.rel = "preload";
        link.as = "style";
        link.href = href;
        document.head.appendChild(link);
    }

    function isLoaded(href) {
        const absoluteUrl = getAbsoluteUrl(href);
        return loadedStyles.has(absoluteUrl);
    }

    function unload(href) {
        const absoluteUrl = getAbsoluteUrl(href);
        const link = document.querySelector(
            `link[href="${href}"], link[href="${absoluteUrl}"]`
        );
        if (link) {
            link.remove();
            loadedStyles.delete(absoluteUrl);
        }
    }

    function loadForApp(appName) {
        const appCssMap = {
            admin: ["admin/admin.css"],
            analytics: ["analytics/analytics.css"],
            attendant: ["attendant/attendant.css"],
            auth: ["auth/auth.css"],
            billing: ["billing/billing.css"],
            calendar: ["calendar/calendar.css"],
            chat: ["chat/chat.css"],
            crm: ["crm/crm.css"],
            dashboards: ["dashboards/dashboards.css"],
            docs: ["docs/docs.css"],
            drive: ["drive/drive.css"],
            learn: ["learn/learn.css"],
            mail: ["mail/mail.css"],
            meet: ["meet/meet.css"],
            monitoring: ["monitoring/monitoring.css"],
            paper: ["paper/paper.css"],
            people: ["people/people.css"],
            products: ["products/products.css"],
            research: ["research/research.css"],
            settings: ["settings/settings.css"],
            sheet: ["sheet/sheet.css"],
            slides: ["slides/slides.css"],
            social: ["social/social.css"],
            sources: ["sources/sources.css"],
            tasks: ["tasks/tasks.css"],
            tickets: ["tickets/tickets.css"],
            tools: ["tools/tools.css"],
        };

        const cssFiles = appCssMap[appName];
        if (cssFiles && cssFiles.length > 0) {
            return loadMultiple(cssFiles);
        }
        return Promise.resolve();
    }

    function init() {
        document.querySelectorAll('link[rel="stylesheet"]').forEach((link) => {
            if (link.href) {
                loadedStyles.add(link.href);
            }
        });

        document.body.addEventListener("htmx:beforeSwap", function (event) {
            const content = event.detail.serverResponse;
            if (content && typeof content === "string") {
                const cssMatches = content.match(
                    /<link[^>]+rel=["']stylesheet["'][^>]*>/gi
                );
                if (cssMatches) {
                    cssMatches.forEach((match) => {
                        const hrefMatch = match.match(/href=["']([^"']+)["']/i);
                        if (hrefMatch && hrefMatch[1]) {
                            load(hrefMatch[1]).catch((err) => {
                                console.warn(
                                    "CSS preload failed:",
                                    err.message
                                );
                            });
                        }
                    });
                }
            }
        });
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", init);
    } else {
        init();
    }

    return {
        load: load,
        loadMultiple: loadMultiple,
        preload: preload,
        isLoaded: isLoaded,
        unload: unload,
        loadForApp: loadForApp,
    };
})();

if (typeof window !== "undefined") {
    window.CSSLoader = CSSLoader;
}
