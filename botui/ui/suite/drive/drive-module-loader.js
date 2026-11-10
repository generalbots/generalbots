// Sequential module loader — ensures scripts execute in order
(function() {
    var modules = [
        '/suite/drive/modules/01_state.js',
        '/suite/drive/modules/02_api.js',
        '/suite/drive/modules/03_utils.js',
        '/suite/drive/modules/04_bots.js',
        '/suite/drive/modules/04_discover.js',
        '/suite/drive/modules/05_files.js',
        '/suite/drive/modules/05_files_lists.js',
        '/suite/drive/modules/05_files_tabs.js',
        '/suite/drive/modules/05_upload.js',
        '/suite/drive/modules/06_render.js',
        '/suite/drive/modules/07_events.js',
        '/suite/drive/modules/08_modals.js',
        '/suite/drive/modules/08_modals_editor.js',
        '/suite/drive/modules/08_preview_parse.js',
        '/suite/drive/modules/08_preview_render.js',
        '/suite/drive/modules/08_preview.js',
        '/suite/drive/modules/09_ai.js',
        '/suite/drive/modules/10_comments.js',
        '/suite/drive/modules/11_external.js',
        '/suite/drive/modules/98_actions.js',
        '/suite/drive/modules/98_actions_clipboard.js',
        '/suite/drive/modules/98_actions_versions.js',
        '/suite/drive/modules/98_actions_extra.js',
        '/suite/drive/modules/13_sync.js',
        '/suite/drive/modules/14_quick_open.js',
        '/suite/drive/modules/99_init.js',
    ];
    window.DriveAI = window.DriveAI || null;
    window.DriveComments = window.DriveComments || null;
    var ts = Date.now();
    function loadNext() {
        if (modules.length === 0) return;
        var src = modules.shift() + '?_=' + ts;
        var s = document.createElement('script');
        s.src = src;
        s.onload = loadNext;
        s.onerror = loadNext;
        document.body.appendChild(s);
    }
    loadNext();
})();
