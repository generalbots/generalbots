//! Extension ↔ MIME tables for the upload policy (#2132).
//!
//! One table, two directions: the extension decides the stored type and the
//! declared `Content-Type` must agree with it. Keeping the pair here means a
//! handler cannot invent a type the validator does not know.

/// MIME type implied by an extension, or `None` when the extension is unknown.
#[must_use]
pub fn mime_for_extension(extension: &str) -> Option<&'static str> {
    Some(match extension {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        "xz" => "application/x-xz",
        "bz2" => "application/x-bzip2",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "odt" => "application/vnd.oasis.opendocument.text",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "odp" => "application/vnd.oasis.opendocument.presentation",
        "doc" => "application/msword",
        "xls" => "application/vnd.ms-excel",
        "ppt" => "application/vnd.ms-powerpoint",
        "csv" => "text/csv",
        "txt" | "log" | "bas" | "ast" => "text/plain",
        "md" | "markdown" => "text/markdown",
        "json" => "application/json",
        "xml" => "application/xml",
        "mpp" => "application/vnd.ms-project",
        "rtf" => "application/rtf",
        "html" | "htm" => "text/html",
        "svg" => "image/svg+xml",
        "ics" | "ical" => "text/calendar",
        "yaml" | "yml" | "toml" | "ini" | "sql" | "rs" | "ts" | "css" | "py" => "text/plain",
        "mp4" | "m4v" => "video/mp4",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" | "oga" => "audio/ogg",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",
        "tsv" => "text/tab-separated-values",
        _ => return None,
    })
}

/// Extension a detected MIME type is stored under. Used by handlers that
/// receive raw bytes with no filename (the legacy `/api/files/upload-binary`).
#[must_use]
pub fn extension_for_mime(mime: &str) -> Option<&'static str> {
    Some(match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "image/tiff" => "tiff",
        "image/svg+xml" => "svg",
        "application/pdf" => "pdf",
        "application/zip" => "zip",
        "application/gzip" => "gz",
        "application/vnd.rar" => "rar",
        "application/x-7z-compressed" => "7z",
        "application/x-xz" => "xz",
        "application/x-bzip2" => "bz2",
        "application/x-ole-storage" => "doc",
        "text/plain" => "txt",
        "text/csv" => "csv",
        "text/markdown" => "md",
        "text/html" => "html",
        "text/xml" | "application/xml" => "xml",
        "application/json" => "json",
        "application/rtf" => "rtf",
        "text/calendar" => "ics",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/x-matroska" => "mkv",
        "video/x-msvideo" => "avi",
        "video/quicktime" => "mov",
        "audio/mpeg" => "mp3",
        "audio/wav" => "wav",
        "audio/ogg" => "ogg",
        "audio/mp4" => "m4a",
        "audio/flac" => "flac",
        _ => return None,
    })
}

/// Text-ish types interchange freely between clients (a `.md` may arrive as
/// `text/plain`), so the declared type is compared as a family.
#[must_use]
pub fn is_text_mime(mime: &str) -> bool {
    mime.starts_with("text/")
        || matches!(
            mime,
            "application/json" | "application/xml" | "application/rtf"
        )
}

/// OOXML and ODF documents are zip containers; legacy Office files are OLE2.
#[must_use]
pub fn is_zip_container(mime: &str) -> bool {
    mime == "application/zip"
        || mime.starts_with("application/vnd.openxmlformats-officedocument.")
        || mime.starts_with("application/vnd.oasis.opendocument.")
}
