/* Drive Module v2.0 — 06 Render: grid/list, cards/rows */
"use strict";

function renderFiles(files) {
    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    if (!files || files.length === 0) {
        content.innerHTML = '<div class="empty-state"><svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg><h3>This folder is empty</h3><p>Upload files or create a new folder to get started</p></div>';
        return;
    }
    const folders = files.filter(function(f) { return f.is_dir; }).sort(function(a, b) { return a.name.localeCompare(b.name); });
    const regularFiles = files.filter(function(f) { return !f.is_dir; }).sort(function(a, b) { return a.name.localeCompare(b.name); });
    const sorted = [].concat(folders).concat(regularFiles);
    if (viewMode === "grid") {
        content.innerHTML = '<div class="file-grid">' + sorted.map(function(f) { return renderFileCard(f); }).join("") + '</div>';
    } else {
        content.innerHTML = '<div class="file-list"><div class="file-list-header"><div class="file-col file-name-col">Name</div><div class="file-col file-modified-col">Modified</div><div class="file-col file-size-col">Size</div><div class="file-col file-actions-col"></div></div>' + sorted.map(function(f) { return renderFileRow(f); }).join("") + '</div>';
    }
    bindFileEvents();
    updateSelectionBar();
}

function renderFileCard(file) {
    var iconClass = file.is_dir ? "folder" : getFileTypeClass(file.name);
    var iconSvg = file.is_dir ? getFolderIcon() : getFileIcon(file.name);
    var sizeText = file.is_dir ? "" : formatFileSize(file.size);
    var checked = selectedFiles.has(file.path) ? "checked" : "";
    var selected = selectedFiles.has(file.path) ? "selected" : "";
    var kbTag = file.is_kb ? '<span class="kb-tag ' + (file.is_public ? "public" : "private") + '" title="' + (file.is_public ? "Public KB" : "Restricted KB") + '">KB</span>' : "";
    return '<div class="file-card ' + selected + '" data-path="' + escapeHtml(file.path) + '" data-name="' + escapeHtml(file.name) + '" data-type="' + (file.is_dir ? "folder" : "file") + '" data-size="' + (file.size || 0) + '"><input type="checkbox" class="file-checkbox" ' + checked + ' onchange="DriveModule.toggleSelection(\'' + escapeJs(file.path) + '\')"><div class="file-card-preview ' + iconClass + '">' + iconSvg + kbTag + '</div><div class="file-card-info"><div class="file-card-name" title="' + escapeHtml(file.name) + '">' + escapeHtml(file.name) + '</div><div class="file-card-meta">' + sizeText + '</div></div></div>';
}

function renderFileRow(file) {
    var iconSvg = file.is_dir ? getFolderIcon() : getFileIcon(file.name);
    var sizeText = file.is_dir ? "\u2014" : formatFileSize(file.size);
    var modifiedText = file.modified ? formatDate(file.modified) : "\u2014";
    var checked = selectedFiles.has(file.path) ? "checked" : "";
    var selected = selectedFiles.has(file.path) ? "selected" : "";
    var downloadIcon = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>';
    var moreIcon = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="1"></circle><circle cx="19" cy="12" r="1"></circle><circle cx="5" cy="12" r="1"></circle></svg>';
    var downloadBtn = !file.is_dir
        ? '<button class="btn-icon-sm" title="Download" onclick="event.stopPropagation(); DriveModule.downloadFile(\'' + escapeJs(file.path) + '\')">' + downloadIcon + '</button>'
        : '';
    var kbTag = file.is_kb ? '<span class="kb-tag ' + (file.is_public ? "public" : "private") + '" title="' + (file.is_public ? "Public KB" : "Restricted KB") + '">' + (file.is_public ? "\uD83D\uDD13" : "\uD83D\uDD12") + ' KB</span>' : '';
    return '<div class="drive-file-item ' + (file.is_dir ? "folder" : "") + ' ' + selected + '" data-path="' + escapeHtml(file.path) + '" data-name="' + escapeHtml(file.name) + '" data-type="' + (file.is_dir ? "folder" : "file") + '" data-size="' + (file.size || 0) + '"><div class="file-col file-name-col"><input type="checkbox" class="file-checkbox" ' + checked + ' onclick="event.stopPropagation()" onchange="DriveModule.toggleSelection(\'' + escapeJs(file.path) + '\')">' + iconSvg + '<span>' + escapeHtml(file.name) + '</span>' + kbTag + '</div><div class="file-col file-modified-col">' + modifiedText + '</div><div class="file-col file-size-col">' + sizeText + '</div><div class="file-col file-actions-col">' + downloadBtn + '<button class="btn-icon-sm" title="More" onclick="event.stopPropagation(); DriveModule.showContextMenuFor(event, \'' + escapeJs(file.path) + '\')">' + moreIcon + '</button></div></div>';
}

function bindFileEvents() {
    document.querySelectorAll(".file-card, .drive-file-item").forEach(function(el) {
        el.addEventListener("click", function(e) {
            if (e.target.closest(".file-checkbox") || e.target.closest(".btn-icon-sm")) return;
            var path = this.dataset.path;
            toggleSelection(path);
        });
        el.addEventListener("dblclick", function(e) {
            if (e.target.closest(".file-checkbox")) return;
            var path = this.dataset.path;
            var type = this.dataset.type;
            if (type === "folder") loadFiles(path, getEffectiveBucket());
            else openFile(path);
        });
    });
}

function toggleSelection(path) {
    if (selectedFiles.has(path)) selectedFiles.delete(path);
    else selectedFiles.add(path);
    var el = document.querySelector('[data-path="' + CSS.escape(path) + '"]');
    if (el) {
        el.classList.toggle("selected", selectedFiles.has(path));
        var checkbox = el.querySelector(".file-checkbox");
        if (checkbox) checkbox.checked = selectedFiles.has(path);
    }
    updateSelectionBar();
}

function selectAll() {
    document.querySelectorAll(".file-card, .drive-file-item").forEach(function(el) {
        selectedFiles.add(el.dataset.path);
        el.classList.add("selected");
        var checkbox = el.querySelector(".file-checkbox");
        if (checkbox) checkbox.checked = true;
    });
    updateSelectionBar();
}

function clearSelection() {
    selectedFiles.clear();
    document.querySelectorAll(".file-card.selected, .drive-file-item.selected").forEach(function(el) {
        el.classList.remove("selected");
        var checkbox = el.querySelector(".file-checkbox");
        if (checkbox) checkbox.checked = false;
    });
    updateSelectionBar();
}

function updateSelectionBar() {
    const bar = document.getElementById("selection-bar");
    const countEl = document.getElementById("selected-count");
    if (bar) bar.style.display = selectedFiles.size > 0 ? "flex" : "none";
    if (countEl) countEl.textContent = selectedFiles.size;
}

function updateBreadcrumb() {
    const breadcrumb = document.querySelector(".breadcrumb, .drive-breadcrumb");
    if (!breadcrumb) return;
    const parts = currentPath ? currentPath.split("/").filter(Boolean) : [];

    var rootLabel = "My Drive";
    if (currentGborgBucket && currentGborgBranch) rootLabel = currentGborgBranch;

    var html = '<button class="breadcrumb-item" onclick="DriveModule.loadFiles(\'\', \'' + currentBucket + '\')">' + escapeHtml(rootLabel) + '</button>';
    var cumulativePath = "";
    parts.forEach(function(part, idx) {
        cumulativePath += (cumulativePath ? "/" : "") + part;
        var isLast = idx === parts.length - 1;
        html += '<span class="breadcrumb-sep">/</span>';
        html += isLast
            ? '<span class="breadcrumb-current">' + escapeHtml(part) + '</span>'
            : '<button class="breadcrumb-item" onclick="DriveModule.loadFiles(\'' + escapeJs(cumulativePath) + '\', \'' + currentBucket + '\')">' + escapeHtml(part) + '</button>';
    });
    breadcrumb.innerHTML = html;
}

function basename(path) {
    var parts = path.split("/").filter(Boolean);
    return parts.length > 0 ? parts[parts.length - 1] : path;
}

function renderSharedList(rows) {
    const el = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!el) return;
    if (!rows || rows.length === 0) {
        el.innerHTML = '<div class="empty-state"><svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle></svg><h3>No shared folders yet</h3><p>Files shared with you by your team will appear here.</p></div>';
        return;
    }
    var html = '<table class="shares-table"><thead><tr><th>Name</th><th>Owner</th><th>Path</th><th>Permissions</th><th>Actions</th></tr></thead><tbody>';
    for (var i = 0; i < rows.length; i++) {
        var r = rows[i];
        var name = basename(r.path);
        html += '<tr data-share-id="' + escapeHtml(r.id) + '">'
            + '<td class="file-name-col"><span class="share-icon">&#128193;</span>' + escapeHtml(name) + '</td>'
            + '<td>' + escapeHtml(r.owner_id) + '</td>'
            + '<td><code>' + escapeHtml(r.bucket + "/" + r.path) + '</code></td>'
            + '<td><span class="perm-badge perm-' + escapeHtml(r.permissions) + '">' + escapeHtml(r.permissions) + '</span></td>'
            + '<td><button class="btn-primary-sm" onclick="DriveModule.openSharedFile(\'' + escapeJs(r.bucket) + '\',\'' + escapeJs(r.path) + '\')">Open</button></td>'
            + '</tr>';
    }
    html += '</tbody></table>';
    el.innerHTML = html;
}

function openSharedFile(bucket, path) {
    currentBucket = bucket;
    currentPath = path;
    openFile(path);
}

