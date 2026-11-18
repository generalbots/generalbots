# =============================================================================
# General Bots — ja — Drive and files
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Drive / Files
# -----------------------------------------------------------------------------
drive-title = ドライブ
drive-upload = アップロード
drive-new-folder = 新しいフォルダー
drive-empty = まだファイルがありません。何かをアップロードしてください！
drive-search = ファイルを検索...
drive-sort-name = 名前
drive-sort-date = 日付
drive-sort-size = サイズ
drive-sort-type = 種類
drive-view-grid = グリッドビュー
drive-view-list = リストビュー
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
drive-drop-files = ここにファイルをドロップしてアップロードします

# Drive translations
drive-my-drive = 私のドライブ
drive-shared = 私と共有しました
drive-recent = 最近の
drive-starred = スター付き
drive-trash = ゴミ箱
drive-loading-storage = ストレージを読み込み中...
drive-storage-used = { $total }中{ $used }が使用されました
drive-empty-folder = このフォルダは空です

# Drive → Use in chat (#2568)
drive-use-in-chat = 会話で使用
drive-use-in-chat-preparing = %n を準備中…
drive-use-in-chat-attached = 会話に添付しました: %n
drive-use-in-chat-failed = ファイルを会話に添付できませんでした。
drive-use-in-chat-timeout = ファイルを添付するには会話を開いてください。
# 共有受信（PWA共有ターゲット）
drive-share-intake-title = 共有ファイルをDriveに保存
drive-share-intake-intro = 以下の共有項目を確認し、Driveに保存してください。保存する前にサインインが必要な場合があります。
drive-share-intake-heading = 共有項目
drive-share-intake-loading = 共有項目を読み込み中…
drive-share-intake-auth-hint = 保存に失敗した場合は、General Botsにサインインして再試行してください。共有ファイルは保存または削除されるまで、このブラウザに一時的に保存されます。
drive-share-intake-save = Driveに保存
drive-share-intake-open-drive = Driveを開く
drive-share-intake-no-shared = 共有ファイルが見つかりませんでした。Driveを開いて手動でアップロードできます。
drive-share-intake-no-files = ファイルは共有されませんでした。
drive-share-intake-text-shown = 共有されたテキスト/リンクは上に表示されています。
drive-share-intake-unavailable = 利用不可
drive-share-intake-read-error = 共有項目を読み取れませんでした。
drive-share-intake-read-failed = 共有ファイルを読み取れませんでした。Driveを開いて手動でアップロードしてください。
drive-share-intake-sign-in-required = ファイルをDriveに保存するにはサインインが必要です。
drive-share-intake-uploading = アップロード中
drive-share-intake-saved = ファイルをDriveに保存しました。
drive-share-intake-saved-partial = 一部のファイルを保存できませんでした。再試行できます。

# semantic i18n key (#2542)
drive-my-files = マイファイル

# semantic i18n key (#2542)
drive-loading-files = ファイルを読み込み中...

# semantic i18n key (#2542)
drive-drop-hint = ここにファイルまたはフォルダをドロップしてアップロード
