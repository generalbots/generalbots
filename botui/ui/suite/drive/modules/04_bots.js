/* Drive Module v2.0 — 04 Bots: show .gbot config folders */
"use strict";

async function loadBotConfigs() {
    if (!currentGborgBranch || !currentGborgBucket) {
        const el = document.getElementById("drive-content") || document.getElementById("file-grid");
        if (el) el.innerHTML = '<div class="empty-state"><h3>No branch context</h3><p>Select a branch first to view bot configs.</p></div>';
        return;
    }
    var prefix = currentGborgBranch + ".gbai/";
    var bucket = currentGborgBucket;

    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state"><div class="spinner"></div><p>Loading bots...</p></div>';

    try {
        const params = new URLSearchParams();
        params.set("bucket", bucket);
        params.set("path", prefix);
        params.set("scope", "bot");
        var files = await apiRequest("/list?" + params.toString());

        var botFolders = (files || []).filter(function(f) {
            return f.is_dir && f.name.endsWith(".gbot");
        }).sort(function(a, b) { return a.name.localeCompare(b.name); });

        var otherFolders = (files || []).filter(function(f) {
            return f.is_dir && !f.name.endsWith(".gbot");
        }).sort(function(a, b) { return a.name.localeCompare(b.name); });

        if (botFolders.length === 0 && otherFolders.length === 0) {
            content.innerHTML = '<div class="empty-state"><svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><rect x="3" y="11" width="18" height="10" rx="2"></rect><circle cx="12" cy="5" r="2"></circle><path d="M12 7v4"></path></svg><h3>No bots configured</h3><p>No .gbot folders found in ' + escapeHtml(prefix) + '.</p></div>';
            return;
        }

        var html = '<div class="bot-config-grid">';
        botFolders.forEach(function(f) {
            html += renderBotConfigCard(f);
        });
        html += '</div>';

        if (otherFolders.length > 0) {
            html += '<h4 class="other-folders-heading">Other Folders</h4><div class="file-grid">';
            otherFolders.forEach(function(f) {
                var name = f.name.replace(currentGborgBranch + ".", "");
                html += '<div class="file-card folder" data-path="' + escapeHtml(f.path) + '" data-name="' + escapeHtml(f.name) + '" data-type="folder" onclick="DriveModule.loadFiles(\'' + escapeJs(f.path) + '\', \'' + escapeJs(currentBucket) + '\')">'
                    + '<div class="file-card-preview folder">' + getFolderIcon() + '</div>'
                    + '<div class="file-card-info"><div class="file-card-name">' + escapeHtml(name) + '</div></div></div>';
            });
            html += '</div>';
        }

        content.innerHTML = html;
    } catch (err) {
        content.innerHTML = '<div class="empty-state"><h3>Failed to load bots</h3><p>' + escapeHtml(err.message) + '</p></div>';
    }
}

function renderBotConfigCard(folder) {
    var name = folder.name.replace(/\.gbot$/, "");
    var iconSvg = '<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#6366f1" stroke-width="2"><rect x="3" y="11" width="18" height="10" rx="2"></rect><circle cx="12" cy="5" r="2"></circle><path d="M12 7v4"></path></svg>';
    var desc = "Bot Configuration";
    return '<div class="bot-config-card" data-path="' + escapeHtml(folder.path) + '" onclick="DriveModule.loadFiles(\'' + escapeJs(folder.path) + '\', \'' + escapeJs(currentBucket) + '\')">'
        + '<div class="bot-config-icon">' + iconSvg + '</div>'
        + '<div class="bot-config-name">' + escapeHtml(name) + '</div>'
        + '<div class="bot-config-desc">' + desc + '</div>'
        + '</div>';
}
