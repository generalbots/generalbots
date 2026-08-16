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
# Chat
# -----------------------------------------------------------------------------
chat-title = チャット
chat-placeholder = メッセージを入力してください...
chat-send = 送信
chat-new-conversation = 新しい会話
chat-history = チャット履歴
chat-clear = チャットをクリアする
chat-export = チャットのエクスポート
chat-typing = { $name } が入力中です...
chat-online = オンライン
chat-offline = オフライン
chat-last-seen = 最後に見たもの { $time }
chat-mention-title = 参照エンティティ
chat-mention-placeholder = メッセージ... (メンションするには @ を入力してください)
chat-mention-search = エンティティを検索...
chat-mention-no-results = 結果が見つかりませんでした
chat-mention-type-hint = 入力: 検索するには

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

# -----------------------------------------------------------------------------
# Tasks
# -----------------------------------------------------------------------------
tasks-title = タスク
tasks-new = 新しいタスク
tasks-due-today = 今日が期限
tasks-overdue = 期限を過ぎました
tasks-completed = 完了しました
tasks-all = すべてのタスク
tasks-priority-high = 高優先度
tasks-priority-medium = 中優先度
tasks-priority-low = 低優先度
tasks-no-due-date = 期限なし
tasks-add-subtask = サブタスクの追加
tasks-mark-complete = 完了としてマークする
tasks-mark-incomplete = 未完了としてマークする
tasks-delete-confirm = このタスクを削除してもよろしいですか?
tasks-count = { $count ->
    [zero] No tasks
    [one] { $count } task
   *[other] { $count } tasks
}

# -----------------------------------------------------------------------------
# Calendar
# -----------------------------------------------------------------------------
calendar-title = カレンダー
calendar-today = 今日
calendar-new-event = 新しいイベント
calendar-all-day = 一日中
calendar-repeat = リピート
calendar-reminder = リマインダー
calendar-view-day = 日
calendar-view-week = 週
calendar-view-month = 月
calendar-view-year = 年
calendar-no-events = 予定されているイベントはありません
calendar-event-title = イベントタイトル
calendar-event-location = 場所
calendar-event-description = 説明
calendar-event-attendees = 出席者

# -----------------------------------------------------------------------------
# Meet / Video Conferencing
# -----------------------------------------------------------------------------
meet-title = 会う
meet-join = ミーティングに参加する
meet-start = ミーティングを開始する
meet-mute = ミュート
meet-unmute = ミュートを解除する
meet-video-on = カメラオン
meet-video-off = カメラオフ
meet-share-screen = 画面共有
meet-stop-sharing = 共有を停止する
meet-end-call = 通話を終了する
meet-leave = 会議から退席する
meet-participants = { $count ->
    [one] { $count } participant
   *[other] { $count } participants
}
meet-waiting-room = 待合室
meet-admit = 認める
meet-remove = 削除する
meet-chat = ミーティングチャット
meet-raise-hand = 挙手
meet-lower-hand = 低い手
meet-recording = 録音
meet-start-recording = 録音を開始する
meet-stop-recording = 録音を停止する

# -----------------------------------------------------------------------------
# Mail / Email
# -----------------------------------------------------------------------------
mail-title = メール
mail-compose = 作曲する
mail-inbox = 受信箱
mail-sent = 送信済み
mail-drafts = 下書き
mail-trash = ゴミ箱
mail-spam = スパムメール
mail-starred = スター付き
mail-archive = アーカイブ
mail-to = に
mail-cc = CC
mail-bcc = BCC
mail-subject = 件名
mail-body = メッセージ
mail-reply = 返信
mail-reply-all = 全員に返信
mail-forward = 進む
mail-send = 送信
mail-discard = 廃棄する
mail-save-draft = ドラフトの保存
mail-attach = ファイルを添付する
mail-unread = { $count ->
    [one] { $count } unread
   *[other] { $count } unread
}
mail-empty-inbox = 受信箱は空です
mail-no-subject = (件名なし)

# -----------------------------------------------------------------------------
# Settings
# -----------------------------------------------------------------------------
settings-title = 設定
settings-general = 一般
settings-account = アカウント
settings-notifications = 通知
settings-privacy = プライバシー
settings-security = セキュリティ設定
settings-language = 言語
settings-theme = テーマ
settings-theme-light = ライト
settings-theme-dark = 暗い
settings-theme-system = システム
settings-save = 変更を保存
settings-saved = 設定が正常に保存されました
settings-timezone = タイムゾーン
settings-date-format = 日付形式
settings-time-format = 時間の形式

# -----------------------------------------------------------------------------
# Auth / Login
# -----------------------------------------------------------------------------
auth-login = ログイン
auth-logout = ログアウト
auth-signup = サインアップ
auth-forgot-password = パスワードをお忘れですか?
auth-reset-password = パスワードのリセット
auth-email = 電子メール
auth-password = パスワード
auth-confirm-password = パスワードの確認
auth-remember-me = 私を覚えていてください
auth-login-success = 正常にログインしました
auth-logout-success = 正常にログアウトされました
auth-invalid-credentials = 無効な電子メールまたはパスワード
auth-session-expired = セッションの有効期限が切れました。再度ログインしてください。

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
pagination-previous = 前へ
pagination-next = 次へ
pagination-first = まず
pagination-last = 最後
pagination-page = ページ { $current }/{ $total }
pagination-showing = { $total }中 { $from }から{ $to }を表示中

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
# Settings - Language & Localization
# -----------------------------------------------------------------------------
settings-language = 言語
settings-language-desc = 好みの言語を選択してください
settings-display-language = 表示言語
settings-language-affects = アプリケーション内のすべてのテキストに影響します
settings-date-format = 日付形式
settings-date-format-desc = 日付の表示方法
settings-time-format = 時間の形式
settings-time-format-desc = 12 時間制または 24 時間制
settings-saved = 設定が正常に保存されました
settings-language-changed = 言語が正常に変更されました
settings-reload-required = 変更を適用するにはページのリロードが必要です

# Settings - Profile
settings-profile = プロファイル設定
settings-profile-desc = 個人情報と設定を管理する
settings-profile-photo = プロフィール写真
settings-profile-photo-desc = あなたのプロフィール写真は他のユーザーに表示されます
settings-upload-photo = 写真をアップロードする
settings-remove-photo = 削除する
settings-basic-info = 基本情報
settings-display-name = 表示名
settings-username = ユーザー名
settings-email-address = メールアドレス
settings-bio = 略歴
settings-bio-placeholder = あなた自身について教えてください...
settings-contact-info = 連絡先情報
settings-phone-number = 電話番号
settings-location = 場所
settings-website = ウェブサイト

# Settings - Security
settings-security = セキュリティ設定
settings-security-desc = 強化されたセキュリティでアカウントを保護する
settings-change-password = パスワードの変更
settings-change-password-desc = セキュリティを強化するためにパスワードを定期的に更新してください
settings-current-password = 現在のパスワード
settings-new-password = 新しいパスワード
settings-confirm-password = 新しいパスワードを確認する
settings-update-password = パスワードを更新する
settings-2fa = 二要素認証
settings-2fa-desc = アカウントに追加のセキュリティ層を追加します
settings-authenticator-app = 認証アプリ
settings-authenticator-desc = 2FA コードの認証アプリを使用する
settings-enable-2fa = 2FA を有効にする
settings-disable-2fa = 2FA を無効にする
settings-active-sessions = アクティブなセッション
settings-active-sessions-desc = アクティブなログインセッションを管理する
settings-this-device = このデバイス
settings-terminate-session = 終了
settings-terminate-all = 他のすべてのセッションを終了する

# Settings - Appearance
settings-appearance = 外観
settings-appearance-desc = アプリケーションの外観をカスタマイズする
settings-theme-selection = テーマ
settings-theme-selection-desc = 好みのカラーテーマを選択してください
settings-theme-dark = 暗い
settings-theme-light = ライト
settings-theme-blue = ブルー
settings-theme-purple = 紫
settings-theme-green = 緑
settings-theme-orange = オレンジ
settings-layout-preferences = レイアウト設定
settings-compact-mode = コンパクトモード
settings-compact-mode-desc = より多くのコンテンツを表示するにはスペースを減らしてください
settings-show-sidebar = サイドバーを表示
settings-show-sidebar-desc = ナビゲーションサイドバーを常に表示する
settings-animations = アニメーション
settings-animations-desc = UI アニメーションとトランジションを有効にする

# Settings - Notifications
settings-notifications-title = 通知
settings-notifications-desc = 通知の受け取り方法を制御する
settings-email-notifications = 電子メール通知
settings-direct-messages = ダイレクトメッセージ
settings-direct-messages-desc = 新しいダイレクトメッセージのメールを受信する
settings-mentions = 言及
settings-mentions-desc = 誰かがあなたにメンションしたときにメールを受け取る
settings-weekly-digest = ウィークリーダイジェスト
settings-weekly-digest-desc = 毎週のアクティビティの概要を取得する
settings-marketing = マーケティング
settings-marketing-desc = ニュースや製品の最新情報を受け取る
settings-push-notifications = プッシュ通知
settings-enable-push = プッシュ通知を有効にする
settings-enable-push-desc = ブラウザのプッシュ通知を受信する
settings-notification-sound = サウンド
settings-notification-sound-desc = 通知音を鳴らす
settings-in-app-notifications = アプリ内通知

# Settings - Storage
settings-storage = ストレージ
settings-storage-desc = ストレージの使用状況を管理する
settings-storage-usage = ストレージの使用量
settings-storage-used = { $total }中{ $used }が使用されました
settings-storage-upgrade = ストレージのアップグレード

# Settings - Privacy
settings-privacy-title = プライバシー
settings-privacy-desc = プライバシー設定を管理する
settings-data-collection = データ収集
settings-analytics = 分析
settings-analytics-desc = 匿名の使用状況データを送信して改善にご協力ください
settings-crash-reports = クラッシュレポート
settings-crash-reports-desc = クラッシュレポートを自動的に送信する
settings-download-data = データをダウンロードする
settings-download-data-desc = すべてのデータのコピーを取得する
settings-delete-account = アカウントの削除
settings-delete-account-desc = アカウントとすべてのデータを完全に削除します
settings-delete-account-warning = この操作は元に戻せません

# Settings - Billing
settings-billing = 請求
settings-billing-desc = サブスクリプションと支払い方法を管理する
settings-current-plan = 現在の計画
settings-free-plan = 無料プラン
settings-pro-plan = プロプラン
settings-enterprise-plan = エンタープライズプラン
settings-upgrade-plan = アップグレードプラン
settings-payment-methods = 支払い方法
settings-add-payment = 支払い方法の追加
settings-billing-history = 請求履歴

# -----------------------------------------------------------------------------
# Paper (Document Editor)
# -----------------------------------------------------------------------------
paper-title = 紙
paper-new-note = 新しいメモ
paper-search-notes = メモを検索...
paper-quick-start = クイックスタート
paper-template-blank = 空白
paper-template-meeting = 会議
paper-template-todo = やるべきこと
paper-template-research = 研究
paper-untitled = 無題
paper-placeholder = 書き込みを開始するか、コマンドとして / を入力します...
paper-commands = コマンド
paper-heading1 = 見出し1
paper-heading1-desc = 大きなセクション見出し
paper-heading2 = 見出し2
paper-heading2-desc = 中セクションの見出し
paper-heading3 = 見出し 3
paper-heading3-desc = 小さなセクションの見出し
paper-paragraph = 段落
paper-paragraph-desc = プレーンテキスト
paper-bullet-list = 箇条書きリスト
paper-bullet-list-desc = 順序なしリスト
paper-numbered-list = 番号付きリスト
paper-numbered-list-desc = 順序付きリスト
paper-todo-list = やることリスト
paper-todo-list-desc = 確認可能なタスクリスト
paper-quote = 引用
paper-quote-desc = 引用のための引用文
paper-divider = ディバイダー
paper-divider-desc = 水平線
paper-code-block = コードブロック
paper-code-block-desc = フォーマットされたコード
paper-table = テーブル
paper-table-desc = テーブルの挿入
paper-image = 画像
paper-image-desc = URLから画像を挿入
paper-callout = 吹き出し
paper-callout-desc = ハイライト表示された情報ボックス
paper-ai-write = AI書き込み
paper-ai-write-desc = AIでテキストを生成
paper-ai-summarize = AIサマリー
paper-ai-summarize-desc = 選択したテキストを要約する
paper-ai-expand = AI拡張
paper-ai-expand-desc = 選択したテキストを展開する
paper-ai-improve = AIの改善
paper-ai-improve-desc = 文章の質を向上させる
paper-ai-translate = AI翻訳
paper-ai-translate-desc = 別の言語に翻訳する
paper-ai-assistant = AIアシスタント
paper-ai-quick-actions = クイックアクション
paper-ai-rewrite = リライト
paper-ai-make-shorter = 短くする
paper-ai-make-longer = 長くする
paper-ai-fix-grammar = 文法を修正する
paper-ai-tone = トーン
paper-ai-tone-professional = プロフェッショナル
paper-ai-tone-casual = カジュアル
paper-ai-tone-friendly = フレンドリー
paper-ai-tone-formal = フォーマル
paper-ai-translate-to = に翻訳する
paper-ai-custom-prompt = カスタムプロンプト
paper-ai-custom-placeholder = 欲しいものを説明してください...
paper-ai-generate = 生成する
paper-ai-response = AIの対応
paper-ai-apply = 申し込む
paper-ai-regenerate = 再生する
paper-ai-copy = コピー
paper-word-count = { $count } 単語
paper-char-count = { $count } 文字
paper-saved = 保存されました
paper-saving = 保存中...
paper-last-edited = 最終編集日: { $time }
paper-last-edited-now = 最終編集日: たった今
paper-export = ドキュメントのエクスポート
paper-export-pdf = PDF
paper-export-docx = ワード (.docx)
paper-export-markdown = マークダウン
paper-export-html = HTML
paper-export-txt = プレーンテキスト

# Additional Chat translations
chat-voice = 音声入力
chat-message-placeholder = メッセージ...

# Drive translations
drive-my-drive = 私のドライブ
drive-shared = 私と共有しました
drive-recent = 最近の
drive-starred = スター付き
drive-trash = ゴミ箱
drive-loading-storage = ストレージを読み込み中...
drive-storage-used = { $total }中{ $used }が使用されました
drive-empty-folder = このフォルダは空です

# Tasks translations
tasks-active = アクティブなインテント
tasks-awaiting = 決定待ち
tasks-paused = 一時停止中
tasks-blocked = ブロック/問題
tasks-time-saved = 保存されたアクティブ時間:
tasks-input-placeholder = 何をしたいですか?例: 「CRM アプリを作成する」または「明日ジョンに電話するようリマインドする」

# Calendar additional translations
calendar-my-calendars = 私のカレンダー

# Email additional translations
email-scheduled = 予定されている
email-tracking = 追跡

# Email folder translations
email-inbox = 受信箱
email-starred = スター付き
email-sent = 送信済み
email-drafts = 下書き
email-spam = スパムメール
email-trash = ゴミ箱
email-compose = 作曲する

# -----------------------------------------------------------------------------
# Research
# -----------------------------------------------------------------------------
research-title = 研究
research-search-placeholder = 何でも聞いてください...
research-collections = コレクション
research-new-collection = 新しいコレクション
research-recent = 最近の
research-academic = アカデミック
research-code = コード
research-internal = 内部
research-search-all = すべてを検索
research-academic-papers = 学術論文
research-code-docs = コードとドキュメント
research-internal-kb = 社内ナレッジベース
research-sources = 情報源
research-trending = トレンド
research-pro-search = プロ検索
research-include-images = 画像を含める
research-try-asking = について聞いてみてください
research-related = 関連する質問
research-view-all-sources = すべてのソースを表示
research-export-citations = 引用のエクスポート
research-save-to-collection = コレクションに保存

# -----------------------------------------------------------------------------
# Admin Panel (additional UI keys)
# -----------------------------------------------------------------------------
admin-panel-title = 管理者パネル
admin-quick-actions = クイックアクション
admin-create-user = ユーザーの作成
admin-create-group = グループの作成
admin-register-dns = DNSを登録する
admin-recent-activity = 最近の活動
admin-system-health = システムの健全性

# -----------------------------------------------------------------------------
# Meet (additional keys)
# -----------------------------------------------------------------------------
meet-new-meeting = 新しい会議
meet-join-meeting = ミーティングに参加する
meet-active-rooms = アクティブルーム
meet-room-title = 会議室
meet-record = 記録
meet-camera = カメラ
meet-share = シェアする
meet-info = 情報
meet-more = もっと見る
meet-share-meeting = シェアミーティング
meet-meeting-title = 会議のタイトル
meet-meeting-code = 会議コード
meet-meeting-link = ミーティングリンク
meet-send-invite = 招待を送信する

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
# Analytics (additional UI keys)
# -----------------------------------------------------------------------------
analytics-dashboard-title = 分析ダッシュボード
analytics-last-hour = 最後の1時間
analytics-last-6h = 過去6時間
analytics-last-24h = 過去 24 時間
analytics-last-7d = 過去 7 日間
analytics-last-30d = 過去 30 日間

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
notifications-title = 通知
notifications-clear = すべてクリア
notifications-empty = 通知はありません

# -----------------------------------------------------------------------------
# All Applications
# -----------------------------------------------------------------------------
nav-all-apps = すべてのアプリケーション

# =============================================================================
# AUTH SCREENS - Complete translations for login, register, forgot/reset password
# =============================================================================

# -----------------------------------------------------------------------------
# Login Screen
# -----------------------------------------------------------------------------
auth-welcome-back = おかえりなさい
auth-sign-in-to-account = General Bots アカウントにサインインします
auth-email-address = メールアドレス
auth-email-placeholder = you@example.com
auth-password-placeholder = ••••••••
auth-sign-in = サインイン
auth-or-continue-with = または続行してください
auth-dont-have-account = アカウントをお持ちでない場合は、
auth-create-account = アカウントを作成する
auth-google = Google
auth-microsoft = マイクロソフト
auth-github = GitHub
auth-apple = アップル

# -----------------------------------------------------------------------------
# Two-Factor Authentication
# -----------------------------------------------------------------------------
auth-2fa-title = 二要素認証
auth-2fa-subtitle = 認証アプリから 6 桁のコードを入力してください
auth-2fa-verify = コードの検証
auth-2fa-didnt-receive = コードを受け取っていませんか?
auth-2fa-resend = コードを再送信する
auth-2fa-back-to-login = ログインに戻る
auth-2fa-trust-device = このデバイスを信頼します
auth-2fa-trust-desc = このデバイスでは 30 日間 2FA を要求しないでください

# -----------------------------------------------------------------------------
# Register Screen
# -----------------------------------------------------------------------------
auth-create-your-account = アカウントを作成する
auth-join-general-bots = 一般的なボットに参加して構築を始めましょう
auth-first-name = 名
auth-last-name = 姓
auth-create-password = パスワードの作成
auth-confirm-your-password = パスワードの確認
auth-password-strength = パスワードの強度
auth-password-weak = 弱い
auth-password-fair = フェア
auth-password-good = 良い
auth-password-strong = 強い
auth-password-req-length = 少なくとも 8 文字
auth-password-req-uppercase = 大文字 1 文字
auth-password-req-lowercase = 小文字 1 文字
auth-password-req-number = 1 つの数字
auth-password-req-special = 特殊文字 1 文字
auth-passwords-match = パスワードが一致する
auth-passwords-dont-match = パスワードが一致しません
auth-agree-terms = に同意します
auth-terms-of-service = 利用規約
auth-and = そして
auth-privacy-policy = プライバシーポリシー
auth-sign-up = サインアップ
auth-already-have-account = すでにアカウントをお持ちですか?
auth-sign-in-link = サインイン
auth-registration-success = アカウントが正常に作成されました。
auth-check-email = メールをチェックしてアカウントを確認してください
auth-email-sent-to = 確認リンクを送信しました
auth-resend-verification = 確認メールを再送信する
auth-go-to-login = ログインに移動

# -----------------------------------------------------------------------------
# Forgot Password Screen
# -----------------------------------------------------------------------------
auth-forgot-password-title = パスワードをお忘れですか?
auth-forgot-password-subtitle = 心配ない！メールアドレスを入力してください。リセット手順をお送りします。
auth-send-reset-link = リセットリンクを送信する
auth-back-to-login = ログインに戻る
auth-reset-email-sent = リセットメールを送信しました!
auth-reset-instructions = パスワードのリセット手順を送信しました
auth-check-inbox = 受信箱を確認してください
auth-check-spam = 迷惑メールフォルダが見つからない場合は確認してください
auth-link-expires = リンクの有効期限は 1 時間です
auth-resend-email = メールを再送信する
auth-didnt-receive-email = メールを受信しませんでしたか?

# -----------------------------------------------------------------------------
# Reset Password Screen
# -----------------------------------------------------------------------------
auth-reset-password-title = パスワードのリセット
auth-reset-password-subtitle = アカウント用に新しい安全なパスワードを作成します
auth-new-password = 新しいパスワード
auth-confirm-new-password = 新しいパスワードを確認する
auth-reset-password-btn = パスワードのリセット
auth-password-reset-success = パスワードが正常にリセットされました!
auth-password-updated = パスワードが更新されました。新しいパスワードを使用してサインインできるようになりました。
auth-invalid-token = リンクが無効または期限切れです
auth-invalid-token-desc = このパスワード リセット リンクは無効であるか、有効期限が切れています。新しいものをリクエストしてください。
auth-request-new-link = 新しいリンクをリクエスト

# =============================================================================
# MONITORING SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# Monitoring Dashboard
# -----------------------------------------------------------------------------
monitoring-title = モニタリングダッシュボード
monitoring-toggle-view = ビューの切り替え
monitoring-last-updated = 最終更新日
monitoring-live-view = ライブビュー
monitoring-grid-view = グリッドビュー

# -----------------------------------------------------------------------------
# Monitoring Panels
# -----------------------------------------------------------------------------
monitoring-sessions = セッション
monitoring-messages = メッセージ
monitoring-resources = リソース
monitoring-services = サービス
monitoring-active-bots = アクティブなボット
monitoring-loading = 読み込み中...

# -----------------------------------------------------------------------------
# Service Status
# -----------------------------------------------------------------------------
monitoring-status-running = ランニング
monitoring-status-warning = 警告
monitoring-status-stopped = 停止しました
monitoring-status-healthy = 健康
monitoring-status-degraded = 劣化した
monitoring-status-down = ダウン

# -----------------------------------------------------------------------------
# Resource Metrics
# -----------------------------------------------------------------------------
monitoring-cpu = CPU
monitoring-memory = 記憶
monitoring-disk = ディスク
monitoring-network = ネットワーク
monitoring-requests-per-sec = リクエスト/秒
monitoring-active-connections = アクティブな接続
monitoring-uptime = 稼働時間

# -----------------------------------------------------------------------------
# Logs
# -----------------------------------------------------------------------------
monitoring-logs-title = システムログ
monitoring-logs-filter = ログのフィルタリング
monitoring-logs-level = ログレベル
monitoring-logs-all = すべてのレベル
monitoring-logs-debug = デバッグ
monitoring-logs-info = 情報
monitoring-logs-warning = 警告
monitoring-logs-error = エラー
monitoring-logs-critical = クリティカル
monitoring-logs-search = ログを検索...
monitoring-logs-no-results = ログが見つかりません

# -----------------------------------------------------------------------------
# Health
# -----------------------------------------------------------------------------
monitoring-health-title = システムの健全性
monitoring-health-status = 健康状態
monitoring-health-services = サービスの健全性
monitoring-health-database = データベース
monitoring-health-cache = キャッシュ
monitoring-health-queue = メッセージキュー
monitoring-health-storage = ストレージ
monitoring-health-external = 外部サービス

# -----------------------------------------------------------------------------
# Metrics
# -----------------------------------------------------------------------------
monitoring-metrics-title = パフォーマンス指標
monitoring-metrics-response-time = 応答時間
monitoring-metrics-throughput = スループット
monitoring-metrics-error-rate = エラー率
monitoring-metrics-latency = レイテンシ

# -----------------------------------------------------------------------------
# Alerts
# -----------------------------------------------------------------------------
monitoring-alerts-title = システムアラート
monitoring-alerts-active = アクティブなアラート
monitoring-alerts-resolved = 解決済み
monitoring-alerts-all = すべてのアラート
monitoring-alert-severity = 重大度
monitoring-alert-critical = クリティカル
monitoring-alert-high = 高
monitoring-alert-medium = 中
monitoring-alert-low = 低い
monitoring-alert-info = 情報
monitoring-alert-acknowledge = 了承する
monitoring-alert-resolve = 解決する
monitoring-no-alerts = アクティブなアラートはありません

# =============================================================================
# SOURCES SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# Sources Main
# -----------------------------------------------------------------------------
sources-title = 情報源
sources-subtitle = リポジトリ、アプリ、プロンプト、テンプレート、MCP サーバー
sources-search = ソースを検索...

# -----------------------------------------------------------------------------
# Sources Tabs
# -----------------------------------------------------------------------------
sources-repositories = リポジトリ
sources-apps = アプリ
sources-prompts = プロンプト
sources-templates = テンプレート
sources-servers = MCPサーバー
sources-models = AIモデル
sources-news = ニュース

# -----------------------------------------------------------------------------
# Repository Cards
# -----------------------------------------------------------------------------
sources-repo-connect = 接続する
sources-repo-disconnect = 切断する
sources-repo-browse = 閲覧する
sources-repo-connected = 接続済み
sources-repo-disconnected = 切断されました
sources-repo-stars = スター
sources-repo-forks = フォーク
sources-repo-last-updated = 最終更新日

# -----------------------------------------------------------------------------
# Prompt Cards
# -----------------------------------------------------------------------------
sources-prompt-use = 使用する
sources-prompt-copy = コピー
sources-prompt-edit = 編集
sources-prompt-rating = 評価
sources-prompt-uses = 用途

# -----------------------------------------------------------------------------
# Server Cards
# -----------------------------------------------------------------------------
sources-server-active = アクティブ
sources-server-inactive = 非アクティブ
sources-server-connect = 接続する
sources-server-configure = 設定する

# -----------------------------------------------------------------------------
# Model Cards
# -----------------------------------------------------------------------------
sources-model-active = アクティブ
sources-model-coming-soon = 近日公開予定
sources-model-provider = プロバイダー
sources-model-context = コンテキスト
sources-model-tokens = トークン

# -----------------------------------------------------------------------------
# App Cards
# -----------------------------------------------------------------------------
sources-app-open = 開く
sources-app-edit = 編集
sources-app-installed = インストール済み
sources-app-install = インストール

# -----------------------------------------------------------------------------
# Template Cards
# -----------------------------------------------------------------------------
sources-template-preview = プレビュー
sources-template-use = テンプレートを使用する
sources-template-components = コンポーネント

# -----------------------------------------------------------------------------
# Categories
# -----------------------------------------------------------------------------
sources-category-all = すべて
sources-category-development = 開発
sources-category-productivity = 生産性
sources-category-communication = コミュニケーション
sources-category-analytics = 分析
sources-category-security = セキュリティ
sources-category-other = その他

# -----------------------------------------------------------------------------
# Empty States
# -----------------------------------------------------------------------------
sources-empty-repos = リポジトリが接続されていません
sources-empty-apps = 利用可能なアプリはありません
sources-empty-prompts = プロンプトが見つかりません
sources-empty-templates = 利用可能なテンプレートはありません
sources-empty-servers = MCP サーバーが構成されていません
sources-empty-models = 利用可能なモデルはありません
sources-empty-results = 結果が見つかりませんでした
sources-empty-results-desc = 検索またはフィルターを調整してみてください

# =============================================================================
# TOOLS / COMPLIANCE SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# Compliance Main
# -----------------------------------------------------------------------------
compliance-title = APIコンプライアンスレポート
compliance-subtitle = すべてのボットのセキュリティ スキャン - パスワード、脆弱なコード、構成ミスをチェックします。
compliance-export-report = レポートのエクスポート
compliance-run-scan = コンプライアンススキャンの実行
compliance-scanning = スキャン中...

# -----------------------------------------------------------------------------
# Bot Selector
# -----------------------------------------------------------------------------
compliance-all-bots = すべてのボット
compliance-select-bots = ボットの選択

# -----------------------------------------------------------------------------
# Stats Cards
# -----------------------------------------------------------------------------
compliance-critical = クリティカル
compliance-critical-desc = 即時の対応が必要です
compliance-high = 高
compliance-high-desc = セキュリティリスク
compliance-medium = 中
compliance-medium-desc = 対処すべき
compliance-low = 低い
compliance-low-desc = ベストプラクティス
compliance-info = 情報
compliance-info-desc = 情報提供

# -----------------------------------------------------------------------------
# Filters
# -----------------------------------------------------------------------------
compliance-filter-severity = 重大度
compliance-filter-type = 種類
compliance-filter-all-severities = すべての重大度
compliance-filter-all-types = 全種類
compliance-search-issues = 検索の問題...

# -----------------------------------------------------------------------------
# Issue Types
# -----------------------------------------------------------------------------
compliance-type-password = 設定内のパスワード
compliance-type-hardcoded = ハードコードされたシークレット
compliance-type-deprecated = 非推奨のキーワード
compliance-type-fragile = 脆弱なコード
compliance-type-config = 構成の問題

# -----------------------------------------------------------------------------
# Results Table
# -----------------------------------------------------------------------------
compliance-results = 結果
compliance-results-count = { $count ->
    [one] { $count } issue found
   *[other] { $count } issues found
}
compliance-col-severity = 重大度
compliance-col-issue = 問題
compliance-col-location = 場所
compliance-col-details = 詳細
compliance-col-action = アクション
compliance-view-details = 詳細を見る
compliance-fix-issue = 問題を修正する
compliance-ignore = 無視する
compliance-no-issues = 問題は見つかりませんでした
compliance-no-issues-desc = 素晴らしい！ボットは準拠しています。

# -----------------------------------------------------------------------------
# Scan Progress
# -----------------------------------------------------------------------------
compliance-scan-in-progress = スキャン中...
compliance-scan-checking = { $item } を確認中...
compliance-scan-complete = スキャンが完了しました
compliance-scan-failed = スキャンに失敗しました

# =============================================================================
# ATTENDANT / CRM SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# CRM Disabled State
# -----------------------------------------------------------------------------
attendant-crm-disabled = CRM機能が有効になっていません
attendant-crm-disabled-desc = アテンダント コンソールでは、このボットに対して CRM 機能を有効にする必要があります。これにより、人間のエージェントがボットから転送された会話を受信して​​応答できるようになります。
attendant-crm-enable-instruction = CRM 機能を有効にするには、ボットの
attendant-crm-config-file = config.csv
attendant-crm-create-attendant = 次に、
attendant-crm-attendant-file = アテンダント.csv
attendant-crm-configure-team = チームを構成するファイル

# -----------------------------------------------------------------------------
# Queue Sidebar
# -----------------------------------------------------------------------------
attendant-title = アテンダントコンソール
attendant-status-online = オンライン
attendant-status-busy = 忙しい
attendant-status-away = 離れて
attendant-status-offline = オフライン
attendant-status-ready = オンライン - 会話の準備完了
attendant-status-busy-msg = 忙しい - 会話の処理
attendant-status-away-msg = 離れています - すぐに戻ってきます
attendant-status-offline-msg = オフライン - 利用できません

# -----------------------------------------------------------------------------
# Queue Stats
# -----------------------------------------------------------------------------
attendant-waiting = 待っています
attendant-active = アクティブ
attendant-resolved = 解決済み
attendant-mine = 私のもの

# -----------------------------------------------------------------------------
# Queue Filters
# -----------------------------------------------------------------------------
attendant-filter-all = すべて
attendant-filter-waiting = 待っています
attendant-filter-mine = 私のもの
attendant-filter-priority = 優先順位

# -----------------------------------------------------------------------------
# Conversation List
# -----------------------------------------------------------------------------
attendant-no-conversations = キューに会話はありません
attendant-new-conversations-appear = 新しい会話がここに表示されます
attendant-unread = 未読
attendant-typing = タイピング中...
attendant-select-conversation = 会話を選択してください
attendant-select-conversation-desc = キューから会話を選択して応答を開始します

# -----------------------------------------------------------------------------
# Channel Tags
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = ワッツアップ
attendant-channel-teams = チーム
attendant-channel-instagram = インスタグラム
attendant-channel-web = ウェブ
attendant-channel-telegram = 電報
attendant-channel-email = 電子メール

# -----------------------------------------------------------------------------
# Priority Tags
# -----------------------------------------------------------------------------
attendant-priority-urgent = 緊急
attendant-priority-high = 高
attendant-priority-normal = ノーマル

# -----------------------------------------------------------------------------
# Chat Area
# -----------------------------------------------------------------------------
attendant-message-placeholder = メッセージを入力してください...
attendant-send = 送信
attendant-attach-file = ファイルを添付する
attendant-insert-emoji = 絵文字の挿入
attendant-quick-responses = 素早い対応
attendant-transfer = 転送
attendant-resolve = 解決する
attendant-more-actions = さらなるアクション

# -----------------------------------------------------------------------------
# Quick Responses
# -----------------------------------------------------------------------------
attendant-quick-greeting = こんにちは！今日はどのようにお手伝いできますか?
attendant-quick-thanks = ご理解いただきありがとうございます。
attendant-quick-checking = それを確認させてください。
attendant-quick-moment = ちょっとお待ちください。

# -----------------------------------------------------------------------------
# Transfer Modal
# -----------------------------------------------------------------------------
attendant-transfer-title = 転送の会話
attendant-transfer-to = 転送先
attendant-transfer-reason = 理由（任意）
attendant-transfer-reason-placeholder = なぜこの会話を転送するのですか?
attendant-transfer-cancel = キャンセル
attendant-transfer-confirm = 転送

# -----------------------------------------------------------------------------
# AI Insights Sidebar
# -----------------------------------------------------------------------------
attendant-ai-insights = AI の洞察
attendant-ai-summary = 会話の要約
attendant-ai-sentiment = 顧客感情
attendant-sentiment-positive = ポジティブ
attendant-sentiment-neutral = ニュートラル
attendant-sentiment-negative = ネガティブ
attendant-smart-replies = スマート リプライ
attendant-confidence = 自信
attendant-source = ソース

# -----------------------------------------------------------------------------
# Customer Details
# -----------------------------------------------------------------------------
attendant-customer-details = 顧客の詳細
attendant-customer-name = 名前
attendant-customer-email = 電子メール
attendant-customer-phone = 電話
attendant-customer-location = 場所
attendant-customer-tags = タグ

# -----------------------------------------------------------------------------
# Conversation History
# -----------------------------------------------------------------------------
attendant-history = 歴史
attendant-history-resolved = 解決済み
attendant-history-transferred = 転送されました
attendant-history-abandoned = 放棄された
attendant-view-history = 全履歴を表示

# -----------------------------------------------------------------------------
# Toast Messages
# -----------------------------------------------------------------------------
attendant-toast-transferred = 会話は正常に転送されました
attendant-toast-resolved = 会話が解決済みとしてマークされました
attendant-toast-assigned = あなたに割り当てられた会話
attendant-toast-error = エラーが発生しました
attendant-toast-connection-lost = 接続が失われました。再接続中...
attendant-toast-connection-restored = 接続が回復しました

# =============================================================================
# CRM
# =============================================================================

# -----------------------------------------------------------------------------
# CRM Navigation & General
# -----------------------------------------------------------------------------
crm-title = CRM
crm-pipeline = パイプライン
crm-leads = リード
crm-opportunities = 機会
crm-accounts = アカウント
crm-contacts = 連絡先
crm-activities = 活動内容

# -----------------------------------------------------------------------------
# CRM Entities
# -----------------------------------------------------------------------------
crm-lead = リード
crm-lead-desc = 資格のない見込み客
crm-opportunity = 機会
crm-opportunity-desc = 適格な販売機会
crm-account = アカウント
crm-account-desc = 会社または団体
crm-contact = お問い合わせ
crm-contact-desc = アカウントの人
crm-activity = アクティビティ
crm-activity-desc = 仕事、電話、メール

# -----------------------------------------------------------------------------
# CRM Actions
# -----------------------------------------------------------------------------
crm-qualify = 資格を得る
crm-convert = 変換する
crm-won = 勝った
crm-lost = 紛失
crm-new-lead = 新しいリード
crm-new-opportunity = 新しい機会
crm-new-account = 新しいアカウント
crm-new-contact = 新しい連絡先

# -----------------------------------------------------------------------------
# CRM Fields
# -----------------------------------------------------------------------------
crm-stage = ステージ
crm-value = 値
crm-probability = 確率
crm-close-date = 終了日
crm-company = 会社名
crm-phone = 電話
crm-email = 電子メール
crm-source = ソース
crm-owner = オーナー

# -----------------------------------------------------------------------------
# CRM Pipeline Stages
# -----------------------------------------------------------------------------
crm-pipeline-new = 新しい
crm-pipeline-contacted = 連絡済み
crm-pipeline-qualified = 資格のある
crm-pipeline-proposal = 提案
crm-pipeline-negotiation = 交渉
crm-pipeline-closed-won = クローズドウォン
crm-pipeline-closed-lost = 閉店しました 紛失しました

# -----------------------------------------------------------------------------
# CRM Stats & Metrics
# -----------------------------------------------------------------------------
crm-subtitle = リード、商談、顧客を管理する
crm-stage-lead = リード
crm-stage-qualified = 資格のある
crm-stage-proposal = 提案
crm-stage-negotiation = 交渉
crm-stage-won = 勝った
crm-stage-lost = 紛失
crm-conversion-rate = コンバージョン率
crm-pipeline-value = パイプラインの価値
crm-avg-deal = 平均取引サイズ
crm-won-month = 今月の勝ち

# -----------------------------------------------------------------------------
# CRM Empty States
# -----------------------------------------------------------------------------
crm-no-leads = リードが見つかりませんでした
crm-no-opportunities = 機会が見つかりませんでした
crm-no-accounts = アカウントが見つかりませんでした
crm-no-contacts = 連絡先が見つかりませんでした
crm-drag-hint = カードをドラッグしてステージを変更します

# =============================================================================
# Billing
# =============================================================================

# -----------------------------------------------------------------------------
# Billing Navigation & General
# -----------------------------------------------------------------------------
billing-title = 請求
billing-invoices = 請求書
billing-payments = 支払い
billing-quotes = 引用
billing-dashboard = ダッシュボード

# -----------------------------------------------------------------------------
# Billing Entities
# -----------------------------------------------------------------------------
billing-invoice = 請求書
billing-invoice-desc = 顧客への請求書
billing-payment = お支払い
billing-payment-desc = 支払いを受け取りました
billing-quote = 引用
billing-quote-desc = 価格見積

# -----------------------------------------------------------------------------
# Billing Status
# -----------------------------------------------------------------------------
billing-due-date = 期限
billing-overdue = 期限を過ぎました
billing-paid = 有料
billing-pending = 保留中
billing-draft = 草案
billing-sent = 送信済み
billing-partial = 部分的
billing-cancelled = キャンセルされました

# -----------------------------------------------------------------------------
# Billing Actions
# -----------------------------------------------------------------------------
billing-new-invoice = 新しい請求書
billing-new-quote = 新しい見積書
billing-new-payment = 新規支払い
billing-send-invoice = 請求書の送信
billing-record-payment = 支払いの記録
billing-mark-paid = 有料としてマークする
billing-void = ボイド

# -----------------------------------------------------------------------------
# Billing Fields
# -----------------------------------------------------------------------------
billing-amount = 金額
billing-tax = 税金
billing-subtotal = 小計
billing-total = 合計
billing-discount = 割引
billing-line-items = 品目
billing-add-item = アイテムの追加
billing-remove-item = アイテムの削除
billing-customer = お客様
billing-issue-date = 発行日
billing-payment-terms = 支払い条件
billing-notes = 注意事項
billing-invoice-number = 請求書番号
billing-quote-number = 見積書番号

# -----------------------------------------------------------------------------
# Billing Reports
# -----------------------------------------------------------------------------
billing-revenue = 収益
billing-outstanding = 優れた
billing-this-month = 今月
billing-last-month = 先月
billing-total-paid = 支払総額
billing-total-overdue = 延滞合計
billing-subtitle = 請求書、支払い、見積書
billing-revenue-month = 今月の収益
billing-total-revenue = 総収益
billing-paid-month = 今月支払い済み

# -----------------------------------------------------------------------------
# Billing Empty States
# -----------------------------------------------------------------------------
billing-no-invoices = 請求書が見つかりません
billing-no-payments = 支払いが見つかりませんでした
billing-no-quotes = 引用符が見つかりませんでした

# =============================================================================
# Products
# =============================================================================

# -----------------------------------------------------------------------------
# Products Navigation & General
# -----------------------------------------------------------------------------
products-title = 製品
products-catalog = カタログ
products-services = サービス
products-price-lists = 価格表
products-inventory = 在庫

# -----------------------------------------------------------------------------
# Products Entities
# -----------------------------------------------------------------------------
products-product = 製品
products-product-desc = 物理的またはデジタル製品
products-service = サービス
products-service-desc = サービス内容
products-price-list = 価格表
products-price-list-desc = 価格帯

# -----------------------------------------------------------------------------
# Products Actions
# -----------------------------------------------------------------------------
products-new-product = 新製品
products-new-service = 新サービス
products-new-price-list = 新しい価格表
products-new-pricelist = 新しい価格表
products-edit-product = 製品の編集
products-duplicate = 重複

# -----------------------------------------------------------------------------
# Products Fields
# -----------------------------------------------------------------------------
products-sku = SKU
products-category = カテゴリ
products-price = 価格
products-unit = ユニット
products-stock = 在庫
products-cost = コスト
products-margin = マージン
products-barcode = バーコード

# -----------------------------------------------------------------------------
# Products Status
# -----------------------------------------------------------------------------
products-in-stock = 在庫あり
products-out-of-stock = 在庫切れ
products-low-stock = 在庫僅少
products-active = アクティブ
products-inactive = 非アクティブ
products-featured = 注目の
products-archived = アーカイブ済み

# -----------------------------------------------------------------------------
# Products Stats & Metrics
# -----------------------------------------------------------------------------
products-subtitle = 製品、サービス、価格を管理する
products-items = 製品
products-pricelists = 価格表
products-total-products = 総製品数
products-total-services = トータルサービス

# -----------------------------------------------------------------------------
# Products Empty States
# -----------------------------------------------------------------------------
products-no-products = 製品が見つかりませんでした
products-no-services = サービスが見つかりませんでした
products-no-price-lists = 価格表が見つかりません

# =============================================================================
# Tickets (Support Cases)
# =============================================================================

# -----------------------------------------------------------------------------
# Tickets Navigation & General
# -----------------------------------------------------------------------------
tickets-title = チケット
tickets-cases = 事例
tickets-open = 開く
tickets-closed = 閉店
tickets-all = すべてのチケット
tickets-my-tickets = 私のチケット

# -----------------------------------------------------------------------------
# Tickets Entities
# -----------------------------------------------------------------------------
tickets-case = ケース
tickets-case-desc = サポートチケット
tickets-resolution = 解像度
tickets-resolution-desc = AIが提案するソリューション

# -----------------------------------------------------------------------------
# Tickets Priority
# -----------------------------------------------------------------------------
tickets-priority = 優先順位
tickets-priority-low = 低い
tickets-priority-medium = 中
tickets-priority-high = 高
tickets-priority-urgent = 緊急

# -----------------------------------------------------------------------------
# Tickets Status
# -----------------------------------------------------------------------------
tickets-status = ステータス
tickets-status-new = 新しい
tickets-status-open = 開く
tickets-status-pending = 保留中
tickets-status-resolved = 解決済み
tickets-status-closed = 閉店
tickets-status-on-hold = 保留中

# -----------------------------------------------------------------------------
# Tickets Actions
# -----------------------------------------------------------------------------
tickets-new-ticket = 新しいチケット
tickets-assign = 割り当てる
tickets-reassign = 再割り当て
tickets-escalate = エスカレーション
tickets-resolve = 解決する
tickets-reopen = 再開
tickets-close = 閉じる
tickets-merge = マージ

# -----------------------------------------------------------------------------
# Tickets Fields
# -----------------------------------------------------------------------------
tickets-subject = 件名
tickets-description = 説明
tickets-category = カテゴリ
tickets-assigned = 担当者
tickets-unassigned = 未割り当て
tickets-created = 作成されました
tickets-updated = 更新されました
tickets-response-time = 応答時間
tickets-resolution-time = 解決時間
tickets-customer = お客様
tickets-internal-notes = 内部メモ
tickets-attachments = 添付ファイル

# -----------------------------------------------------------------------------
# Tickets AI Features
# -----------------------------------------------------------------------------
tickets-ai-suggestion = AIによる提案
tickets-apply-suggestion = 提案を適用する
tickets-ai-summary = AIの概要
tickets-similar-tickets = 類似のチケット
tickets-suggested-articles = おすすめの記事

# -----------------------------------------------------------------------------
# Tickets Empty States
# -----------------------------------------------------------------------------
tickets-no-tickets = チケットが見つかりませんでした
tickets-no-open = オープンチケットはありません
tickets-no-closed = 終了したチケットはありません

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
security-title = セキュリティ
security-subtitle = セキュリティ ツール、コンプライアンス スキャン、およびサーバー保護
security-tab-compliance = APIコンプライアンスレポート
security-tab-protection = 保護
security-export-report = レポートのエクスポート
security-run-scan = コンプライアンススキャンの実行
security-critical = クリティカル
security-critical-desc = 直ちに対応が必要です
security-high = 高
security-high-desc = セキュリティリスク
security-medium = 中
security-medium-desc = 対処すべき
security-low = 低い
security-low-desc = ベストプラクティス
security-info = 情報
security-info-desc = 情報提供
security-filter-severity = 重大度:
security-filter-all-severities = すべての重大度
security-filter-type = タイプ:
security-filter-all-types = 全種類
security-type-password = 設定内のパスワード
security-type-hardcoded = ハードコードされたシークレット
security-type-deprecated = 非推奨のキーワード
security-type-fragile = 脆弱なコード
security-type-config = 構成の問題
security-results = コンプライアンスの問題
security-col-severity = 重大度
security-col-issue = 問題の種類
security-col-location = 場所
security-col-details = 説明
security-col-action = アクション

# -----------------------------------------------------------------------------
# Learn Module
# -----------------------------------------------------------------------------
learn-title = 学ぶ
learn-my-progress = 私の進歩
learn-completed = 完了しました
learn-in-progress = 進行中
learn-certificates = 証明書
learn-time-spent = 費やした時間
learn-categories = カテゴリー
learn-all-courses = 全コース
learn-mandatory = 必須
learn-compliance = コンプライアンス
learn-security = セキュリティ
learn-skills = スキル
learn-onboarding = オンボーディング
learn-difficulty = 難易度
learn-my-certificates = 私の証明書
learn-view-all = すべて見る

# -----------------------------------------------------------------------------
# Workspace Module
# -----------------------------------------------------------------------------
workspace-title = ワークスペース
workspace-search-pages = ページを検索...
workspace-recent = 最近の
workspace-favorites = お気に入り
workspace-pages = ページ
workspace-templates = テンプレート
workspace-trash = ゴミ箱
workspace-settings = 設定

# -----------------------------------------------------------------------------
# Player Module
# -----------------------------------------------------------------------------
player-title = メディアプレーヤー
player-no-file = ファイルが選択されていません
player-search = ファイルを検索...
player-recent = 最近の
player-files = ファイル

# -----------------------------------------------------------------------------
# Goals Module
# -----------------------------------------------------------------------------
goals-title = 目標とOKR
goals-dashboard = ダッシュボード
goals-objectives = 目的
goals-alignment = 位置合わせ
goals-ai-suggestions = AIによる提案

# CRM / Mail / Campaigns integration keys
crm-email = 電子メール
crm-compose-email = 電子メールを作成する
crm-send-email = 電子メールを送信する
mail-snooze = スヌーズ
mail-snooze-later-today = 本日この後（午後6時）
mail-snooze-tomorrow = 明日（午前8時）
mail-snooze-next-week = 来週（月曜午前8時）
mail-crm-log = CRMにログを記録する
mail-crm-create-lead = リードの作成
mail-add-to-list = リストに追加
campaign-send-email = 電子メールを送信する

# -----------------------------------------------------------------------------
# OAuth Account Linking (Settings)
# -----------------------------------------------------------------------------
oauth-connected-accounts = 接続されたアカウント
oauth-connect = 接続する
oauth-unlink = リンクを解除する
oauth-not-connected = 接続されていません
oauth-linked = リンク済み
oauth-no-accounts = まだアカウントがリンクされていません。
oauth-loading = リンクされたアカウントを読み込んでいます…

## Payment cards (Stripe SetupIntent)
cards-title = 支払いとカード
cards-saved = 保存されたカード
cards-hint = カードは当社の決済プロバイダーによって安全に保管されます。カード番号が当社のサーバーに届くことはありません。
cards-add = カードを追加
cards-add-first = 最初のカードを追加する
cards-none = まだカードが保存されていません
cards-empty-hint = カードを追加すると、自動請求と迅速なチェックアウトが可能になります。安全な支払いプロバイダーにリダイレクトされ、カードの詳細を入力します。
cards-default = デフォルト
cards-set-default = デフォルトを設定する
cards-default-btn = デフォルトカード
cards-remove = 削除する
cards-remove-confirm = このカードを削除しますか?
cards-expires = 有効期限が切れます
cards-load-error = 保存されたカードを読み込めませんでした。
cards-add-error = カードを追加できませんでした
cards-default-error = デフォルトを更新できませんでした
cards-remove-error = カードを取り出せませんでした
cards-default-updated = デフォルトカードが更新されました
cards-removed = カードが削除されました

## Compliance frameworks (enterprise-grade release)
compliance-frameworks = フレームワーク
compliance-new-framework = 新しい
compliance-framework-name = 名前
compliance-framework-version = バージョン
compliance-framework-description = 説明
compliance-create-framework = フレームワークの作成
compliance-controls = コントロール
compliance-add-control = コントロールの追加
compliance-control-id = コントロールID
compliance-control-title = タイトル
compliance-control-category = カテゴリ
compliance-control-description = 説明
compliance-mandatory = 必須
compliance-optional = オプション
compliance-evidence = 証拠
compliance-attach-evidence = 証拠を添付する
compliance-evidence-path = ファイルパス (ドライブアーティファクト)
compliance-evidence-type = 種類
compliance-approve = 承認する
compliance-covered = カバーされた
compliance-no-evidence = 証拠がない
compliance-export-csv = CSVのエクスポート
compliance-archive = アーカイブ
compliance-total-controls = トータルコントロール
compliance-coverage = 適用範囲
compliance-no-frameworks = まだフレームワークが構成されていません。

## Sources connectors (enterprise-grade release)
sources-connectors = コネクタ
sources-add-connector = コネクタの追加
sources-connector-name = 名前
sources-connector-description = 説明
sources-connector-schedule = 同期スケジュール (cron)
sources-connector-type = 種類
sources-connector-host = ホスト
sources-connector-port = 港
sources-connector-database = データベース
sources-connector-username = ユーザー名
sources-connector-password = パスワード
sources-connector-base-url = ベースURL
sources-connector-api-key = APIキー
sources-connector-credentials-hint = 認証情報は Vault に保存され、保存後に再度表示されることはありません。
sources-create-connector = コネクタの作成
sources-test-connector = テスト
sources-sync-now = 今すぐ同期する
sources-remove-connector = 削除する
sources-connector-health = 健康
sources-connector-last-sync = 最終同期
sources-no-connectors = コネクタが構成されていません
