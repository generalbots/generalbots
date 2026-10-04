/* Drive Module v2.0 — 04 Discover: resolve which Drive layout the bot uses */
"use strict";

/// Classify a Drive layout probe: "content" when the folder holds items,
/// "empty" when the caller may address it but it holds nothing, "error" when
/// the bucket is missing or not visible to the caller.
async function probeLayout(bucket, path) {
    try {
        const items = await apiRequest(
            "/list?bucket=" + encodeURIComponent(bucket) +
            "&path=" + encodeURIComponent(path) + "&scope=bot"
        );
        return (Array.isArray(items) && items.length > 0) ? "content" : "empty";
    } catch (err) {
        console.warn("Drive layout probe failed for " + bucket + "/" + path + ":", err);
        return "error";
    }
}

async function discoverBuckets() {
    try {
        // An org claim from the session decides the layout. It used to be
        // treated as a hint that could be overridden by probing, because
        // channel stagers wrote org bots to a standalone `{bot}.gbai` bucket
        // and the org path therefore listed empty. Those stagers now resolve
        // the bucket through the bot's org (server side), so the org layout is
        // authoritative: an empty org Drive means empty, not "try elsewhere".
        if (userInfo && userInfo.bucket) {
            var bucketName = userInfo.bucket;
            currentBucket = bucketName;
            if (bucketName.indexOf(".gborg") > 0) {
                currentGborgBucket = bucketName;
                currentGborgBranch = bucketName.replace(".gborg", "");
            } else {
                currentGborgBucket = null;
                currentGborgBranch = null;
            }
            retryCount = 0;
            return;
        }

        // Fallback: derive the layout from the folders that actually exist.
        // A standalone bot owns the `{bot}.gbai` bucket; an org workspace
        // nests the same tree at `{org}.gborg/{bot}.gbai/`. Probe the org
        // layout first and only fall back to the standalone bucket when the
        // org bucket is not addressable at all (not merely empty).
        var botName = window.__INITIAL_BOT_NAME__ || window.location.pathname.split('/').filter(Boolean)[0] || '';
        if (botName) {
            currentGborgBranch = botName;
            if (bucketLayoutResolved) return;
            bucketLayoutResolved = true;
            var orgBucket = botName + ".gborg";
            var ownBucket = botName + ".gbai";
            if (await probeLayout(orgBucket, botName + ".gbai/" + botName + ".gbdrive") === "error") {
                currentBucket = ownBucket;
                currentGborgBucket = null;
            } else {
                currentBucket = orgBucket;
                currentGborgBucket = orgBucket;
            }
            return;
        }

        // Admin-only: attempt API to list all buckets
        try {
            var url = botName ? '/buckets?bot=' + encodeURIComponent(botName) : '/buckets';
            const buckets = await apiRequest(url);
            availableBuckets = buckets || [];
            retryCount = 0;

            var gborg = availableBuckets.find(function(b) { return b.is_gborg; });
            var gbai = availableBuckets.find(function(b) { return b.is_gbai; });

            if (gborg) {
                currentGborgBucket = gborg.name;
                currentGborgBranch = gborg.name.replace(".gborg", "");
                currentBucket = gborg.name;
            } else if (gbai) {
                currentGborgBucket = null;
                currentGborgBranch = null;
                currentBucket = gbai.name;
            } else if (availableBuckets.length > 0) {
                currentGborgBucket = null;
                currentGborgBranch = null;
                currentBucket = availableBuckets[0].name;
            }
        } catch (apiErr) {
            console.warn("Failed to list buckets via API (admin only):", apiErr);
            // Non-admin: derive bucket from session bot name
            if (!currentBucket) {
                var sessionBot = window.__INITIAL_BOT_NAME__ || 'default';
                currentBucket = sessionBot + '.gbai';
                currentGborgBucket = null;
                currentGborgBranch = sessionBot;
            }
        }

    } catch (err) {
        console.error("Failed to discover buckets:", err);
        const content = document.getElementById("drive-content") || document.getElementById("file-grid");
        if (content) {
            var canRetry = retryCount < MAX_RETRIES;
            var retryMsg = canRetry
                ? '<button class="btn-primary" onclick="DriveModule.retryWithBackoff()">Retry</button>'
                : '<p class="text-muted">Max retries reached. Please refresh the page.</p>';
            content.innerHTML = '<div class="empty-state"><svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg><h3>Drive connection error</h3><p>' + escapeHtml(err.message) + '</p>' + retryMsg + '</div>';
        }
    }
}
