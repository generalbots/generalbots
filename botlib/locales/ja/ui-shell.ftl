# =============================================================================
# General Bots — ja — Desktop shell, navigation, controls
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# General Bots - English UI Translations
# =============================================================================
# -----------------------------------------------------------------------------
# Navigation
# -----------------------------------------------------------------------------
nav-home = ホーム
nav-chat = チャット
nav-drive = ドライブ
nav-tasks = タスク
nav-mail = メール
nav-calendar = カレンダー
nav-meet = 会う
nav-paper = 紙
nav-video = ビデオ
nav-research = 研究
nav-analytics = 分析
nav-settings = 設定
nav-admin = 管理者
nav-monitoring = モニタリング
nav-sources = 情報源
nav-tools = ツール
nav-attendant = アテンダント
nav-learn = 学ぶ
nav-crm = CRM
nav-billing = 請求
nav-products = 製品
nav-tickets = チケット
nav-docs = ドキュメント
nav-sheet = シート
nav-slides = スライド
nav-social = ソーシャル
nav-all-apps = すべてのアプリケーション
nav-people = 人々
nav-editor = 編集者
nav-dashboards = ダッシュボード
nav-security = セキュリティ
nav-designer = デザイナー
nav-project = プロジェクト
nav-canvas = キャンバス
nav-goals = 目標
nav-player = プレーヤー
nav-workspace = ワークスペース

# -----------------------------------------------------------------------------
# Dashboard
# -----------------------------------------------------------------------------
dashboard-title = ダッシュボード
dashboard-welcome = { $name }さん、おかえりなさい！
dashboard-quick-actions = クイックアクション
dashboard-recent-activity = 最近の活動
dashboard-no-activity = 最近の活動はまだありません。探検を始めましょう！
dashboard-analytics = 分析

# -----------------------------------------------------------------------------
# Quick Actions
# -----------------------------------------------------------------------------
quick-start-chat = チャットを開始する
quick-upload-files = ファイルをアップロードする
quick-new-task = 新しいタスク
quick-compose-email = 電子メールを作成する
quick-start-meeting = ミーティングを開始する
quick-new-event = 新しいイベント

# -----------------------------------------------------------------------------
# Application Cards
# -----------------------------------------------------------------------------
app-chat-name = チャット
app-chat-desc = AI を活用した会話。質問し、助けを求め、タスクを自動化します。
app-drive-name = ドライブ
app-drive-desc = すべてのファイルをクラウド ストレージに保存します。アップロード、整理、共有します。
app-tasks-name = タスク
app-tasks-desc = To Do リスト、優先順位、期限を設定して整理しましょう。
app-mail-name = メール
app-mail-desc = AI 支援によるライティングとスマートな構成を備えた電子メール クライアント。
app-calendar-name = カレンダー
app-calendar-desc = 会議やイベントをスケジュールし、時間を効果的に管理します。
app-meet-name = 会う
app-meet-desc = 画面共有とライブ文字起こしによるビデオ会議。
app-paper-name = 紙
app-paper-desc = AI 支援を利用して文書を作成します。メモ、レポートなど。
app-research-name = 研究
app-research-desc = すべてのソースにわたる AI を活用した検索と検出。
app-analytics-name = 分析
app-analytics-desc = 使用状況と洞察を追跡するためのダッシュボードとレポート。

# -----------------------------------------------------------------------------
# Suite Header
# -----------------------------------------------------------------------------
suite-title = 一般的なボット スイート
suite-tagline = AI を活用した生産性向上のワークスペース。チャット、共同作業、作成。
suite-new-intent = 新しい意図

# -----------------------------------------------------------------------------
# AI Panel
# -----------------------------------------------------------------------------
ai-developer = AI開発者
ai-developing = 開発中: { $project }
ai-quick-actions = クイックアクション
ai-add-field = フィールドの追加
ai-change-color = 色を変更する
ai-add-validation = 検証の追加
ai-export-data = データのエクスポート
ai-placeholder = 変更内容を入力してください...
ai-thinking = AIは考えています...
ai-status-online = オンライン
ai-status-offline = オフライン

# -----------------------------------------------------------------------------
# Search
# -----------------------------------------------------------------------------
search-placeholder = 検索...
search-no-results = 結果が見つかりませんでした
search-results = { $count ->
    [one] { $count } result
   *[other] { $count } results
}
search-in-progress = 検索中...
search-advanced = 高度な検索
search-filters = フィルター
search-clear-filters = フィルターをクリアする

# -----------------------------------------------------------------------------
# Pagination
# -----------------------------------------------------------------------------
# -----------------------------------------------------------------------------
# Tables
# -----------------------------------------------------------------------------
table-no-data = 利用可能なデータがありません
table-loading = データをロード中...
table-actions = アクション
table-select-all = すべて選択
table-deselect-all = すべての選択を解除
table-export = エクスポート
table-import = インポート

# -----------------------------------------------------------------------------
# Forms
# -----------------------------------------------------------------------------
form-required = 必須
form-optional = オプション
form-submit = 送信する
form-reset = リセット
form-clear = クリア
form-uploading = アップロード中...
form-processing = 処理中...

# -----------------------------------------------------------------------------
# Modals / Dialogs
# -----------------------------------------------------------------------------
modal-confirm-title = アクションの確認
modal-confirm-message = 続行してもよろしいですか?
modal-delete-title = 削除の確認
modal-delete-message = この操作は元に戻すことができません。本気ですか？

# -----------------------------------------------------------------------------
# Tooltips
# -----------------------------------------------------------------------------
tooltip-copy = クリップボードにコピー
tooltip-copied = コピーしました！
tooltip-expand = 拡大する
tooltip-collapse = 崩壊する
tooltip-refresh = リフレッシュ
tooltip-download = ダウンロード
tooltip-upload = アップロード
tooltip-print = 印刷する
tooltip-fullscreen = フルスクリーン
tooltip-exit-fullscreen = 全画面表示を終了する

# -----------------------------------------------------------------------------
# Common Labels (additional)
# -----------------------------------------------------------------------------
label-username = ユーザー名
label-email = 電子メール
label-display-name = 表示名
label-password = パスワード
label-role = 役割
label-group-name = グループ名
label-hostname = ホスト名
label-record-type = レコードタイプ
label-target = ターゲット
label-your-name = あなたの名前

# -----------------------------------------------------------------------------
# Actions (additional)
# -----------------------------------------------------------------------------
action-register = 登録する

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
notifications-title = 通知
notifications-clear = すべてクリア
notifications-empty = 通知はありません

# #1853 — desktop chrome, launchers and notification settings.
window-ready = 準備完了
window-loading = アプリを読み込み中…
window-minimize = 最小化
window-maximize = 最大化
window-fullscreen = 全画面表示
window-exit-fullscreen = 全画面表示を終了
window-close = 閉じる
app-loading = アプリを読み込み中…
start-menu-search-placeholder = アプリを検索…
start-menu-recent = 最近使用したアプリ
start-menu-no-apps = アプリが見つかりません
start-menu-user = ユーザー
category-ai = AI
category-business = ビジネス
category-office = オフィス
category-dev = 開発
category-system = システム
category-tools = ツール
sidebar-apps = アプリ
sidebar-apps-toggle = アプリの表示/非表示
sidebar-apps-toggle-aria = アプリパネルを切り替え
sidebar-apps-filter = アプリを絞り込む…
sidebar-apps-filter-aria = アプリを絞り込む
sidebar-preview = プレビューモード
sidebar-preview-hint = プレビュー中のアプリを表示する
sidebar-preview-toggle = プレビューモードを切り替え
sidebar-app-preview-title = {app}（プレビュー）
sidebar-apps-count = {count} 個のアプリ
sidebar-apps-count-preview = {count}（+ {preview} プレビュー）
spotlight-placeholder = アプリやファイルを検索… または Concierge の目標を入力
spotlight-hint = ↑↓ 移動 · Enter 実行 · Ctrl+Space または Esc で閉じる
spotlight-empty = 結果がありません
spotlight-goal = 目標を実行: “{goal}”
spotlight-goal-subtitle = Concierge · アプリを横断して計画・実行
palette-placeholder = アプリとコマンドを検索。または @ <コマンド> と入力してアシスタントに送信...
palette-empty = 一致するアプリやコマンドがありません
palette-kind-app = アプリ
palette-hint = ↑↓ 移動 · Enter 実行 · Esc 閉じる · Ctrl+K 切り替え
palette-open-app = {app} を開く
palette-value-prompt = {name}: {summary} — {key} の値:
palette-execute = {method} {path} を実行し、結果を要約します。
palette-use-command = {command} コマンドを使用して {summary}。
toast-dismiss = 閉じる
settings-language-display = 表示言語
settings-language-display-desc = アプリのインターフェース言語を選択してください
settings-language-regional = 地域設定
settings-language-regional-desc = 日付、時刻、数値の形式を設定します
settings-language-date-format = 日付形式
settings-language-time-format = 時刻形式
settings-language-timezone = タイムゾーン
settings-language-first-day = 週の開始曜日
settings-language-time-12h = 12時間制 (1:30 PM)
settings-language-time-24h = 24時間制 (13:30)
settings-language-sunday = 日曜日
settings-language-monday = 月曜日
settings-email-notifications-desc = ワークスペースの通知送信者から送信され、接続された Gmail/Outlook メールボックスからは送信されません。
settings-push-enable = プッシュ通知を有効にする
settings-push-enable-desc = ブラウザのプッシュ通知を受け取ります
settings-push-sound = サウンド
settings-push-sound-desc = 通知時にサウンドを再生します
settings-desktop-notifications = デスクトップ通知
settings-desktop-notifications-desc = デスクトップに通知を表示します
settings-badge-count = バッジ件数
settings-badge-count-desc = 未読件数バッジを表示します
settings-save-preferences = 設定を保存
settings-notifications-saved = 通知設定を保存しました
settings-notification-recipient = 配信メールボックス
settings-notification-recipient-desc = 空欄の場合は、主要なメールアカウントまたはプロフィールのメールアドレスを使用します。
language-search-placeholder = 言語を検索…
settings-profile-title = プロファイル設定
settings-security-title = セキュリティ設定
