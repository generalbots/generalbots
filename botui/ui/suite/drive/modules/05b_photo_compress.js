/* Drive Module — 05b Photo compression (#2510)
 *
 * A phone photo is 4-12 MB. Uploading that raw over cellular is the slowest
 * part of mobile intake, so an image above GBC_IMAGE_MIN_BYTES is re-encoded to
 * a 2048 px longest edge before the upload starts. Everything else is sent
 * untouched: small images gain nothing, animated GIFs break, and SVG is text.
 *
 * The original is never discarded locally — the file input already left it on
 * the device — and a compression failure returns the original file instead of
 * blocking the upload.
 */
"use strict";

var GBC_IMAGE_MIN_BYTES = 2 * 1024 * 1024;
var GBC_IMAGE_MAX_EDGE = 2048;
var GBC_IMAGE_QUALITY = 0.82;

function gbcIsCompressibleImage(file) {
    if (!file || !file.type || file.type.indexOf("image/") !== 0) return false;
    // Re-encoding an animated GIF keeps one frame; SVG is vector text.
    if (file.type === "image/gif" || file.type === "image/svg+xml") return false;
    return file.size > GBC_IMAGE_MIN_BYTES;
}

// The re-encoded file must not claim the old extension: a 5 MB PNG that became
// a JPEG and still says .png is a lie the next downloader has to pay for.
function gbcCompressedName(name, mime) {
    var ext = mime === "image/webp" ? ".webp" : (mime === "image/png" ? ".png" : ".jpg");
    var stem = String(name || "photo").replace(/\.[^./\\]+$/, "");
    return stem + ext;
}

function gbcEncodeCanvas(canvas, file) {
    // PNG keeps transparency as WebP; everything else goes to JPEG. A browser
    // that cannot encode the requested type silently returns PNG, which the
    // size check below then rejects when it is not smaller.
    var mime = file.type === "image/png" ? "image/webp" : "image/jpeg";
    return new Promise(function (resolve) {
        canvas.toBlob(function (blob) {
            if (!blob || blob.size >= file.size) {
                resolve(file);
                return;
            }
            resolve(new File([blob], gbcCompressedName(file.name, blob.type), {
                type: blob.type,
                lastModified: file.lastModified,
            }));
        }, mime, GBC_IMAGE_QUALITY);
    });
}

async function gbcCompressImage(file) {
    var bitmap;
    try {
        bitmap = await createImageBitmap(file);
    } catch (err) {
        // createImageBitmap is absent on older Safari: send the original.
        if (window.gbLogError) gbLogError(err, "gbcCompressImage");
        else console.error("gbcCompressImage", err);
        return file;
    }
    try {
        var scale = Math.min(1, GBC_IMAGE_MAX_EDGE / Math.max(bitmap.width, bitmap.height));
        var canvas = document.createElement("canvas");
        canvas.width = Math.max(1, Math.round(bitmap.width * scale));
        canvas.height = Math.max(1, Math.round(bitmap.height * scale));
        var ctx = canvas.getContext("2d");
        if (!ctx) return file;
        ctx.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
        return await gbcEncodeCanvas(canvas, file);
    } catch (err) {
        if (window.gbLogError) gbLogError(err, "gbcCompressImage");
        else console.error("gbcCompressImage", err);
        return file;
    } finally {
        if (bitmap && typeof bitmap.close === "function") bitmap.close();
    }
}

// Entry point for the upload path: always resolves with a usable File.
function gbcPrepareUploadFile(file) {
    if (!gbcIsCompressibleImage(file)) return Promise.resolve(file);
    return gbcCompressImage(file);
}
