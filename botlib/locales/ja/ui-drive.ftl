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
