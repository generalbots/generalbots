(function() {
    'use strict';

    const MAX_ERRORS = 50;
    const REPORT_ENDPOINT = '/api/client-errors';
    let errorQueue = [];
    let isReporting = false;

    function formatError(error, context = {}) {
        return {
            type: error.name || 'Error',
            message: error.message || String(error),
            stack: error.stack,
            url: window.location.href,
            userAgent: navigator.userAgent,
            timestamp: new Date().toISOString(),
            context: context
        };
    }

    async function reportErrors() {
        if (isReporting || errorQueue.length === 0) return;

        isReporting = true;
        const errorsToReport = errorQueue.splice(0, MAX_ERRORS);
        errorQueue = [];

        try {
            const response = await fetch(REPORT_ENDPOINT, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ errors: errorsToReport })
            });

            if (!response.ok) {
                console.warn('[ErrorReporter] Failed to send errors:', response.status);
            } else {
                console.log('[ErrorReporter] Sent', errorsToReport.length, 'errors to server');
            }
        } catch (e) {
            console.warn('[ErrorReporter] Failed to send errors:', e.message);
            errorQueue.unshift(...errorsToReport);
        } finally {
            isReporting = false;

            if (errorQueue.length > 0) {
                setTimeout(reportErrors, 1000);
            }
        }
    }

    function queueError(errorData) {
        errorQueue.push(errorData);
        if (errorQueue.length >= 10) {
            reportErrors();
        }
    }

    window.addEventListener('error', (event) => {
        const errorData = formatError(event.error || new Error(event.message), {
            filename: event.filename,
            lineno: event.lineno,
            colno: event.colno
        });
        queueError(errorData);
    });

    window.addEventListener('unhandledrejection', (event) => {
        const errorData = formatError(event.reason || new Error(String(event.reason)), {
            type: 'unhandledRejection'
        });
        queueError(errorData);
    });

    window.ErrorReporter = {
        report: function(error, context) {
            queueError(formatError(error, context));
        },
        reportNetworkError: function(url, status, statusText) {
            queueError({
                type: 'NetworkError',
                message: `Failed to load ${url}: ${status} ${statusText}`,
                url: window.location.href,
                timestamp: new Date().toISOString(),
                context: { url, status, statusText }
            });
        },
        flush: function() {
            reportErrors();
        }
    };

    if (document.readyState === 'complete') {
        setTimeout(reportErrors, 1000);
    } else {
        window.addEventListener('load', () => {
            setTimeout(reportErrors, 1000);
        });
    }

    console.log('[ErrorReporter] Client-side error reporting initialized');

    window.NavigationLogger = {
        log: function(from, to, method) {
            const navEvent = {
                type: 'navigation',
                from: from,
                to: to,
                method: method,
                url: window.location.href,
                timestamp: new Date().toISOString()
            };

            queueError({
                name: 'Navigation',
                message: `${method}: ${from} -> ${to}`,
                stack: undefined
            });
        }
    };

    function initNavigationTracking() {
        if (!document.body) {
            setTimeout(initNavigationTracking, 50);
            return;
        }

        if (document.body) {
            document.body.addEventListener('click', function(e) {
                const target = e.target.closest('[data-section]');
                if (target) {
                    const section = target.getAttribute('data-section');
                    const currentHash = window.location.hash.slice(1) || '';
                    if (section !== currentHash) {
                        setTimeout(() => {
                            window.NavigationLogger.log(currentHash || 'home', section, 'click');
                        }, 100);
                    }
                }
            }, true);
        }

        window.addEventListener('hashchange', function(e) {
            const oldURL = new URL(e.oldURL);
            const newURL = new URL(e.newURL);
            const fromHash = oldURL.hash.slice(1) || '';
            const toHash = newURL.hash.slice(1) || '';
            window.NavigationLogger.log(fromHash || 'home', toHash, 'hashchange');
        });

        console.log('[NavigationLogger] Navigation tracking initialized');
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initNavigationTracking);
    } else {
        initNavigationTracking();
    }

    // Intercept link onload/onerror events to catch CSS/image load failures
    const originalCreateElement = document.createElement;
    document.createElement = function(tagName) {
        const element = originalCreateElement.call(document, tagName);
        if (tagName.toLowerCase() === 'link') {
            element.addEventListener('error', function() {
                if (this.href && window.ErrorReporter && window.ErrorReporter.reportNetworkError) {
                    window.ErrorReporter.reportNetworkError(this.href, 'LOAD_FAILED', 'Resource failed to load');
                }
            });
        }
        return element;
    };

    window.addEventListener('load', () => {
        setTimeout(() => {
            const failedResources = performance.getEntriesByType('resource').filter(entry =>
                entry.transferSize === 0 && entry.decodedBodySize > 0 && !entry.name.includes('anon') && entry.duration > 100
            );

            if (failedResources.length > 0) {
                console.warn('[ErrorReporter] Detected potentially failed resources:', failedResources);
                failedResources.forEach(resource => {
                    window.ErrorReporter.reportNetworkError(resource.name, 'FAILED', 'Resource load timeout/failure');
                });
            }
        }, 5000);
    });
})();
