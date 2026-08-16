# ============================================================================
# General Bots - Common Translations (English)
# ============================================================================
# This file contains shared strings used across all GB components.
# Keep message IDs lowercase with hyphens: category-subcategory-descriptor
# ============================================================================

# -----------------------------------------------------------------------------
# Brand
# -----------------------------------------------------------------------------
app-name = 一般的なボット
app-tagline = AI を活用した生産性向上のワークスペース

# -----------------------------------------------------------------------------
# Common Actions
# -----------------------------------------------------------------------------
action-save = 保存
action-cancel = キャンセル
action-delete = 削除
action-edit = 編集
action-close = 閉じる
action-confirm = 確認する
action-retry = 再試行
action-back = 戻る
action-next = 次へ
action-submit = 送信する
action-search = 検索
action-refresh = リフレッシュ
action-copy = コピー
action-paste = ペースト
action-undo = 元に戻す
action-redo = やり直し
action-select = 選択
action-select-all = すべて選択
action-clear = クリア
action-reset = リセット
action-apply = 申し込む
action-create = 作成
action-update = アップデート
action-remove = 削除する
action-add = 追加
action-upload = アップロード
action-download = ダウンロード
action-export = エクスポート
action-import = インポート
action-share = シェアする
action-send = 送信
action-reply = 返信
action-forward = 進む
action-archive = アーカイブ
action-restore = 復元
action-duplicate = 重複
action-rename = 名前の変更
action-move = 移動
action-filter = フィルター
action-sort = 並べ替え
action-view = 見る
action-hide = 隠す
action-show = 表示する
action-expand = 拡大する
action-collapse = 崩壊する
action-enable = 有効にする
action-disable = 無効にする
action-connect = 接続する
action-disconnect = 切断する
action-sync = 同期
action-start = スタート
action-stop = 停止
action-pause = 一時停止
action-resume = 再開
action-continue = 続ける
action-finish = 終了
action-complete = 完了
action-approve = 承認する
action-reject = 拒否する
action-accept = 受け入れる
action-decline = 辞退
action-login = ログイン
action-logout = ログアウト
action-signup = サインアップ
action-forgot-password = パスワードを忘れた場合

# -----------------------------------------------------------------------------
# Common Labels
# -----------------------------------------------------------------------------
label-loading = 読み込み中...
label-saving = 保存中...
label-processing = 処理中...
label-searching = 検索中...
label-uploading = アップロード中...
label-downloading = ダウンロード中...
label-no-results = 結果が見つかりませんでした
label-no-data = 利用可能なデータがありません
label-empty = 空の
label-none = なし
label-all = すべて
label-selected = 選択済み
label-required = 必須
label-optional = オプション
label-default = デフォルト
label-custom = カスタム
label-new = 新しい
label-draft = 草案
label-pending = 保留中
label-active = アクティブ
label-inactive = 非アクティブ
label-enabled = 有効
label-disabled = 障害者
label-public = 公共
label-private = プライベート
label-shared = 共有
label-yes = はい
label-no = いいえ
label-on = オン
label-off = オフ
label-true = 本当
label-false = 偽
label-unknown = 不明
label-other = その他
label-more = もっと見る
label-less = 少ない
label-details = 詳細
label-summary = 概要
label-description = 説明
label-name = 名前
label-title = タイトル
label-type = 種類
label-status = ステータス
label-priority = 優先順位
label-date = 日付
label-time = 時間
label-size = サイズ
label-count = カウント
label-total = 合計
label-average = 平均
label-minimum = 最小値
label-maximum = 最大値
label-version = バージョン
label-id = ID
label-created = 作成されました
label-updated = 更新されました
label-modified = 修正済み
label-deleted = 削除されました
label-by = によって
label-from = から
label-to = に
label-at = で
label-in = で
label-of = の

# -----------------------------------------------------------------------------
# Status Messages
# -----------------------------------------------------------------------------
status-success = 成功
status-error = エラー
status-warning = 警告
status-info = 情報
status-loading = 読み込み中
status-complete = 完了
status-incomplete = 不完全
status-failed = 失敗しました
status-cancelled = キャンセルされました
status-pending = 保留中
status-in-progress = 進行中
status-done = 完了
status-ready = 準備完了
status-not-ready = 準備ができていません
status-connected = 接続済み
status-disconnected = 切断されました
status-online = オンライン
status-offline = オフライン
status-available = 利用可能
status-unavailable = 利用不可
status-busy = 忙しい
status-away = 離れて

# -----------------------------------------------------------------------------
# Confirmation Dialogs
# -----------------------------------------------------------------------------
confirm-delete = これを削除してもよろしいですか?
confirm-delete-item = 「{ $name }」を削除してもよろしいですか?
confirm-delete-items = { $count -> を削除してもよろしいですか?
    [one] this item
   *[other] these { $count } items
}?
confirm-discard-changes = 未保存の変更があります。本当に破棄してもよろしいですか?
confirm-logout = ログアウトしてもよろしいですか?
confirm-cancel = 本当にキャンセルしてもよろしいですか?

# -----------------------------------------------------------------------------
# Time and Dates
# -----------------------------------------------------------------------------
time-now = ちょうど今
time-seconds-ago = { $count ->
    [one] { $count } second ago
   *[other] { $count } seconds ago
}
time-minutes-ago = { $count ->
    [one] { $count } minute ago
   *[other] { $count } minutes ago
}
time-hours-ago = { $count ->
    [one] { $count } hour ago
   *[other] { $count } hours ago
}
time-days-ago = { $count ->
    [one] { $count } day ago
   *[other] { $count } days ago
}
time-weeks-ago = { $count ->
    [one] { $count } week ago
   *[other] { $count } weeks ago
}
time-months-ago = { $count ->
    [one] { $count } month ago
   *[other] { $count } months ago
}
time-years-ago = { $count ->
    [one] { $count } year ago
   *[other] { $count } years ago
}
time-in-seconds = { $count ->
    [one] in { $count } second
   *[other] in { $count } seconds
}
time-in-minutes = { $count ->
    [one] in { $count } minute
   *[other] in { $count } minutes
}
time-in-hours = { $count ->
    [one] in { $count } hour
   *[other] in { $count } hours
}
time-in-days = { $count ->
    [one] in { $count } day
   *[other] in { $count } days
}
time-today = 今日
time-yesterday = 昨日
time-tomorrow = 明日
time-this-week = 今週
time-last-week = 先週
time-next-week = 来週
time-this-month = 今月
time-last-month = 先月
time-next-month = 来月
time-this-year = 今年は
time-last-year = 昨年
time-next-year = 来年

# Days of the week
day-sunday = 日曜日
day-monday = 月曜日
day-tuesday = 火曜日
day-wednesday = 水曜日
day-thursday = 木曜日
day-friday = 金曜日
day-saturday = 土曜日
day-sun = 太陽
day-mon = 月
day-tue = 火
day-wed = 水
day-thu = 木
day-fri = 金
day-sat = 土

# Months
month-january = 1月
month-february = 2月
month-march = 3月
month-april = 4月
month-may = 5月
month-june = 6月
month-july = 7月
month-august = 8月
month-september = 9月
month-october = 10月
month-november = 11月
month-december = 12月
month-jan = 1月
month-feb = 2月
month-mar = 3月
month-apr = 4月
month-may-short = 5月
month-jun = ジュン
month-jul = 7月
month-aug = 8月
month-sep = 9月
month-oct = 10月
month-nov = 11月
month-dec = 12月

# -----------------------------------------------------------------------------
# File Sizes
# -----------------------------------------------------------------------------
size-bytes = { $value } B
size-kilobytes = { $value }KB
size-megabytes = { $value } MB
size-gigabytes = { $value } GB
size-terabytes = { $value }TB

# -----------------------------------------------------------------------------
# Pagination
# -----------------------------------------------------------------------------
pagination-page = ページ { $current }/{ $total }
pagination-showing = { $total }中 { $start }から{ $end }を表示中
pagination-items-per-page = ページごとの項目
pagination-first = まず
pagination-previous = 前へ
pagination-next = 次へ
pagination-last = 最後
pagination-go-to-page = ページに移動

# -----------------------------------------------------------------------------
# Form Validation
# -----------------------------------------------------------------------------
validation-required = このフィールドは必須です
validation-required-field = { $field }は必須です
validation-email-invalid = 有効なメールアドレスを入力してください
validation-url-invalid = 有効な URL を入力してください
validation-number-invalid = 有効な番号を入力してください
validation-date-invalid = 有効な日付を入力してください
validation-min-length = { $min } 文字以上である必要があります
validation-max-length = { $max } 文字以内にしてください
validation-min-value = { $min } 以上である必要があります
validation-max-value = { $max } 以下でなければなりません
validation-pattern-mismatch = 無効な形式
validation-passwords-mismatch = パスワードが一致しません
validation-file-too-large = ファイルが大きすぎます。最大サイズは{ $max }です
validation-file-type-invalid = 無効なファイルタイプです。許可されるタイプ: { $types }

# -----------------------------------------------------------------------------
# Accessibility
# -----------------------------------------------------------------------------
a11y-skip-to-content = メインコンテンツにスキップ
a11y-loading = 読み込み中です、お待ちください
a11y-menu-open = メニューを開く
a11y-menu-close = メニューを閉じる
a11y-expand = 拡大する
a11y-collapse = 崩壊する
a11y-selected = 選択済み
a11y-not-selected = 未選択
a11y-required = 必須フィールド
a11y-error = エラー
a11y-success = 成功
a11y-warning = 警告
a11y-info = 情報
