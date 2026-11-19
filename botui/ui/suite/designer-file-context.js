/* Dialog Designer — per-window Drive file context (#2282).
 * Drive stamps the window body with data-gb-file-* and registers the context in
 * __GB_WINDOW_FILE_CTX[winId]. Page globals are read ONLY as a fallback for
 * direct app opens; opening a second designer/editor never overwrites them. */
(function () {
    "use strict";

    function designerRoot(el) {
        var node = el && el.nodeType ? el : (el && el.target ? el.target : null);
        if (node && node.closest) {
            var fromEl = node.closest('[id^="window-body-"]');
            if (fromEl) return fromEl;
        }
        var script = document.currentScript;
        if (script && script.closest) {
            var fromScript = script.closest('[id^="window-body-"]');
            if (fromScript) return fromScript;
        }
        return null;
    }

    function designerFileCtx(el) {
        var root = designerRoot(el);
        var ds = (root && root.dataset) || {};
        var winId = ds.windowId || (root && root.id ? root.id.replace('window-body-', '') : '');
        var map = (winId && window.__GB_WINDOW_FILE_CTX && window.__GB_WINDOW_FILE_CTX[winId]) || {};
        var query = new URLSearchParams(window.location.search);
        var hash = window.location.hash || '';
        var at = hash.indexOf('?');
        var fragment = new URLSearchParams(at === -1 ? '' : hash.substring(at + 1));
        return {
            bucket: ds.gbFileBucket || map.bucket || window.__EDITOR_FILE_BUCKET ||
                query.get('bucket') || fragment.get('bucket') || '',
            path: ds.gbFilePath || map.path || window.__EDITOR_FILE_PATH ||
                query.get('path') || fragment.get('path') || '',
            scope: ds.gbFileScope || map.scope || window.__EDITOR_FILE_SCOPE ||
                query.get('scope') || fragment.get('scope') || 'bot'
        };
    }

    // "Edit Text" hands the SAME per-window context to a new editor window.
    function openDesignerInTextEditor(el) {
        var ctx = designerFileCtx(el);
        if (!ctx.path || !ctx.bucket) return;
        var winId = 'editor-' + Date.now();
        var editorCtx = { bucket: ctx.bucket, path: ctx.path, scope: ctx.scope };
        window.__EDITOR_FILE_CTX = window.__EDITOR_FILE_CTX || {};
        window.__EDITOR_FILE_CTX[winId] = editorCtx;
        window.__GB_WINDOW_FILE_CTX = window.__GB_WINDOW_FILE_CTX || {};
        window.__GB_WINDOW_FILE_CTX[winId] = editorCtx;
        window.__EDITOR_BOOTS = window.__EDITOR_BOOTS || {};
        window.__EDITOR_BOOTS[winId] = Promise.resolve();
        window.WindowManager.open(winId, ctx.path.split('/').pop() || 'Untitled', '');
        var body = document.getElementById('window-body-' + winId);
        if (body) {
            body.dataset.gbFileBucket = editorCtx.bucket;
            body.dataset.gbFilePath = editorCtx.path;
            body.dataset.gbFileScope = editorCtx.scope;
        }
        fetch('/suite/editor.html').then(function (r) { return r.text(); }).then(function (html) {
            window.WindowManager._injectBodyContent(winId, html);
        })
          .catch(function (fetchError) {
            window.gbLogError ? gbLogError(fetchError, "openDesignerInTextEditor")
                                : console.error("if", fetchError);
          });
    }

    window.gbDesignerFileCtx = designerFileCtx;
    window.openDesignerInTextEditor = openDesignerInTextEditor;
})();
