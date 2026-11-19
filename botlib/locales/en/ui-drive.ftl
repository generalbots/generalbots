# =============================================================================
# General Bots — en — Drive and files
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Drive / Files
# -----------------------------------------------------------------------------
drive-title = Drive
drive-upload = Upload
drive-new-folder = New Folder
drive-empty = No files yet. Upload something!
drive-search = Search files...
drive-sort-name = Name
drive-sort-date = Date
drive-sort-size = Size
drive-sort-type = Type
drive-view-grid = Grid View
drive-view-list = List View
drive-selected = { $count ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
drive-file-size = { $size ->
    [bytes] { $value } B
    [kb] { $value } KB
    [mb] { $value } MB
    [gb] { $value } GB
   *[other] { $value } bytes
}
drive-drop-files = Drop files here to upload

# Drive translations
drive-my-drive = My Drive
drive-shared = Shared with me
drive-recent = Recent
drive-starred = Starred
drive-trash = Trash
drive-loading-storage = Loading storage...
drive-storage-used = { $used } of { $total } used
drive-empty-folder = This folder is empty

# Drive → Use in chat (#2568)
drive-use-in-chat = Use in chat
drive-use-in-chat-preparing = Preparing %n…
drive-use-in-chat-attached = Attached to the conversation: %n
drive-use-in-chat-failed = Could not attach the file to the conversation.
drive-use-in-chat-timeout = Open the chat to attach the file.
# Share intake (PWA share target)
drive-share-intake-title = Save shared files to Drive
drive-share-intake-intro = Review the shared items below, then save them to your Drive. You may need to sign in before saving.
drive-share-intake-heading = Shared items
drive-share-intake-loading = Loading shared items…
drive-share-intake-auth-hint = If saving fails, sign in to General Bots and try again. Shared files are stored temporarily in this browser until saved or cleared.
drive-share-intake-save = Save to Drive
drive-share-intake-open-drive = Open Drive
drive-share-intake-no-shared = No shared files were found. You can still open Drive and upload manually.
drive-share-intake-no-files = No files were shared.
drive-share-intake-text-shown = Shared text/link shown above.
drive-share-intake-unavailable = unavailable
drive-share-intake-read-error = Could not read shared items.
drive-share-intake-read-failed = Shared files could not be read. Open Drive to upload manually.
drive-share-intake-sign-in-required = Sign in required to save files to Drive.
drive-share-intake-uploading = Uploading
drive-share-intake-saved = Saved files to Drive.
drive-share-intake-saved-partial = Some files could not be saved. You can retry.

# semantic i18n key (#2542)
drive-my-files = My Files

# semantic i18n key (#2542)
drive-loading-files = Loading files...

# semantic i18n key (#2542)
drive-drop-hint = Drop files or folders here to upload

# semantic i18n key (#2542)
drive-search-all-placeholder = Search all files…
drive-search-all-aria-label = Search all files by name and content

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
drive-starred-empty-title = No starred files
drive-starred-empty-desc = Star any file, or use its ⋮ menu, to keep it one click away.
drive-shared-empty-title = No shared files
drive-shared-empty-desc = Files shared with you appear here. Share one of yours to start a review.
drive-list-error-title = This list could not be loaded
drive-list-error-desc = Drive did not answer in time. Check your connection and try again.
