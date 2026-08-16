# =============================================================================
# General Bots - Admin Translations (English)
# =============================================================================
# Administrative interface translations for the GB Admin Panel
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Navigation & Dashboard
# -----------------------------------------------------------------------------
admin-title = 管理
admin-dashboard = 管理者ダッシュボード
admin-overview = 概要
admin-welcome = 管理者パネルへようこそ

admin-nav-dashboard = ダッシュボード
admin-nav-users = ユーザー
admin-nav-bots = ボット
admin-nav-tenants = テナント
admin-nav-settings = 設定
admin-nav-logs = ログ
admin-nav-analytics = 分析
admin-nav-security = セキュリティ
admin-nav-integrations = 統合
admin-nav-billing = 請求
admin-nav-support = サポート
admin-nav-groups = グループ
admin-nav-dns = DNS
admin-nav-system = システム

# -----------------------------------------------------------------------------
# Admin Quick Actions
# -----------------------------------------------------------------------------
admin-quick-actions = クイックアクション
admin-create-user = ユーザーの作成
admin-create-group = グループの作成
admin-register-dns = DNSを登録する
admin-recent-activity = 最近の活動
admin-system-health = システムの健全性

# -----------------------------------------------------------------------------
# User Management
# -----------------------------------------------------------------------------
admin-users-title = ユーザー管理
admin-users-list = ユーザーリスト
admin-users-add = ユーザーの追加
admin-users-edit = ユーザーの編集
admin-users-delete = ユーザーの削除
admin-users-search = ユーザーを検索...
admin-users-filter = ユーザーをフィルタリングする
admin-users-export = ユーザーのエクスポート
admin-users-import = ユーザーのインポート
admin-users-total = 総ユーザー数
admin-users-active = アクティブユーザー
admin-users-inactive = 非アクティブなユーザー
admin-users-suspended = 停止されたユーザー
admin-users-pending = 検証保留中
admin-users-last-login = 最終ログイン
admin-users-created = 作成されました
admin-users-role = 役割
admin-users-status = ステータス
admin-users-actions = アクション
admin-users-no-users = ユーザーが見つかりませんでした
admin-users-confirm-delete = このユーザーを削除してもよろしいですか?
admin-users-deleted = ユーザーは正常に削除されました
admin-users-saved = ユーザーは正常に保存されました
admin-users-invite = ユーザーを招待する
admin-users-invite-sent = 招待状が正常に送信されました
admin-users-bulk-actions = 一括アクション
admin-users-select-all = すべて選択
admin-users-deselect-all = すべての選択を解除

# User Details
admin-user-details = ユーザーの詳細
admin-user-profile = プロフィール
admin-user-email = 電子メール
admin-user-name = 名前
admin-user-phone = 電話
admin-user-avatar = アバター
admin-user-timezone = タイムゾーン
admin-user-language = 言語
admin-user-role-admin = 管理者
admin-user-role-manager = マネージャー
admin-user-role-user = ユーザー
admin-user-role-viewer = ビューア
admin-user-status-active = アクティブ
admin-user-status-inactive = 非アクティブ
admin-user-status-suspended = 一時停止中
admin-user-status-pending = 保留中
admin-user-permissions = 権限
admin-user-activity = アクティビティログ
admin-user-sessions = アクティブなセッション
admin-user-terminate-session = セッションを終了する
admin-user-terminate-all = すべてのセッションを終了する
admin-user-reset-password = パスワードのリセット
admin-user-force-logout = 強制ログアウト
admin-user-enable-2fa = 2FA を有効にする
admin-user-disable-2fa = 2FA を無効にする

# -----------------------------------------------------------------------------
# Group Management
# -----------------------------------------------------------------------------
admin-groups-title = グループ経営
admin-groups-subtitle = グループ、メンバー、権限を管理する
admin-groups-list = グループリスト
admin-groups-add = グループの追加
admin-groups-create = グループの作成
admin-groups-edit = グループの編集
admin-groups-delete = グループの削除
admin-groups-search = グループを検索...
admin-groups-filter = フィルターグループ
admin-groups-total = グループの合計数
admin-groups-active = アクティブなグループ
admin-groups-no-groups = グループが見つかりませんでした
admin-groups-confirm-delete = このグループを削除してもよろしいですか?
admin-groups-deleted = グループは正常に削除されました
admin-groups-saved = グループは正常に保存されました
admin-groups-created = グループが正常に作成されました
admin-groups-loading = グループを読み込んでいます...

# Group Details
admin-group-details = グループ詳細
admin-group-name = グループ名
admin-group-description = 説明
admin-group-visibility = 可視性
admin-group-visibility-public = 公共
admin-group-visibility-private = プライベート
admin-group-visibility-hidden = 隠された
admin-group-join-policy = 参加ポリシー
admin-group-join-invite = 招待のみ
admin-group-join-request = 参加リクエスト
admin-group-join-open = 開く
admin-group-members = メンバー
admin-group-member-count = { $count ->
    [one] { $count } member
   *[other] { $count } members
}
admin-group-add-member = メンバーを追加
admin-group-remove-member = メンバーの削除
admin-group-permissions = 権限
admin-group-settings = 設定
admin-group-analytics = 分析
admin-group-overview = 概要

# Group View Modes
admin-groups-view-grid = グリッドビュー
admin-groups-view-list = リストビュー
admin-groups-all-visibility = すべての可視性

# -----------------------------------------------------------------------------
# DNS Management
# -----------------------------------------------------------------------------
admin-dns-title = DNS管理
admin-dns-subtitle = ボットの DNS ホスト名を登録および管理する
admin-dns-register = ホスト名の登録
admin-dns-registered = 登録されたホスト名
admin-dns-search = ホスト名を検索...
admin-dns-refresh = リフレッシュ
admin-dns-loading = DNS レコードをロードしています...
admin-dns-no-records = DNS レコードが見つかりませんでした
admin-dns-confirm-delete = このホスト名を削除してもよろしいですか?
admin-dns-deleted = ホスト名が正常に削除されました
admin-dns-saved = DNS レコードが正常に保存されました
admin-dns-created = ホスト名が正常に登録されました

# DNS Form Fields
admin-dns-hostname = ホスト名
admin-dns-hostname-placeholder = mybot.example.com
admin-dns-hostname-help = 登録したい完全なドメイン名を入力してください
admin-dns-record-type = レコードタイプ
admin-dns-record-type-a = A（IPv4）
admin-dns-record-type-aaaa = AAAA (IPv6)
admin-dns-record-type-cname = CNAME
admin-dns-ttl = TTL (秒)
admin-dns-ttl-5min = 5分(300)
admin-dns-ttl-1hour = 1時間（3600）
admin-dns-ttl-1day = 1日(86400)
admin-dns-target = ターゲット/IPアドレス
admin-dns-target-placeholder-ipv4 = 192.168.1.1
admin-dns-target-placeholder-ipv6 = 2001:db8::1
admin-dns-target-placeholder-cname = target.example.com
admin-dns-target-help-a = 指すIPv4アドレスを入力します
admin-dns-target-help-aaaa = 指すIPv6アドレスを入力します。
admin-dns-target-help-cname = 対象のドメイン名を入力してください
admin-dns-auto-ssl = SSL証明書を自動的にプロビジョニングする

# DNS Table Headers
admin-dns-col-hostname = ホスト名
admin-dns-col-type = 種類
admin-dns-col-target = ターゲット
admin-dns-col-ttl = TTL
admin-dns-col-ssl = SSL
admin-dns-col-status = ステータス
admin-dns-col-actions = アクション

# DNS Status
admin-dns-status-active = アクティブ
admin-dns-status-pending = 保留中
admin-dns-status-error = エラー
admin-dns-ssl-enabled = SSLの有効化
admin-dns-ssl-disabled = SSLなし
admin-dns-ssl-pending = SSL保留中

# DNS Info Cards
admin-dns-help-title = DNS設定ヘルプ
admin-dns-help-a-record = 記録
admin-dns-help-a-record-desc = ドメイン名を IPv4 アドレスにマップします。これを使用して、ホスト名がサーバー IP を直接指すようにします。
admin-dns-help-aaaa-record = AAAAレコード
admin-dns-help-aaaa-record-desc = ドメイン名を IPv6 アドレスにマッピングします。 A レコードに似ていますが、IPv6 接続用です。
admin-dns-help-cname-record = CNAMEレコード
admin-dns-help-cname-record-desc = あるドメインから別のドメインへのエイリアスを作成します。サブドメインをメイン ドメインに向ける場合に便利です。
admin-dns-help-ssl = SSL/TLS
admin-dns-help-ssl-desc = 安全な HTTPS 接続用に Let's Encrypt 証明書を自動的にプロビジョニングします。

# DNS Edit/Remove Modals
admin-dns-edit-title = DNSレコードの編集
admin-dns-remove-title = ホスト名の削除
admin-dns-remove-warning = これにより、DNS レコードと関連する SSL 証明書が削除されます。ホスト名は解決されなくなります。

# -----------------------------------------------------------------------------
# Bot Management
# -----------------------------------------------------------------------------
admin-bots-title = ボット管理
admin-bots-list = ボットリスト
admin-bots-add = ボットの追加
admin-bots-edit = ボットの編集
admin-bots-delete = ボットの削除
admin-bots-search = ボットを検索...
admin-bots-filter = フィルターボット
admin-bots-total = ボットの総数
admin-bots-active = アクティブなボット
admin-bots-inactive = 非アクティブなボット
admin-bots-draft = ドラフトボット
admin-bots-published = 公開されたボット
admin-bots-no-bots = ボットは見つかりませんでした
admin-bots-confirm-delete = このボットを削除してもよろしいですか?
admin-bots-deleted = ボットは正常に削除されました
admin-bots-saved = ボットは正常に保存されました
admin-bots-duplicate = ボットの複製
admin-bots-export = エクスポートボット
admin-bots-import = インポートボット
admin-bots-publish = 発行する
admin-bots-unpublish = 非公開にする
admin-bots-test = テストボット
admin-bots-logs = ボットログ
admin-bots-analytics = ボット分析
admin-bots-conversations = 会話
admin-bots-templates = テンプレート
admin-bots-dialogs = ダイアログ
admin-bots-knowledge-base = ナレッジベース

# Bot Details
admin-bot-details = ボットの詳細
admin-bot-name = ボット名
admin-bot-description = 説明
admin-bot-avatar = ボットアバター
admin-bot-language = 言語
admin-bot-timezone = タイムゾーン
admin-bot-greeting = ご挨拶メッセージ
admin-bot-fallback = フォールバックメッセージ
admin-bot-channels = チャンネル
admin-bot-channel-web = ウェブチャット
admin-bot-channel-whatsapp = ワッツアップ
admin-bot-channel-telegram = 電報
admin-bot-channel-slack = たるみ
admin-bot-channel-teams = マイクロソフトチーム
admin-bot-channel-email = 電子メール
admin-bot-model = AIモデル
admin-bot-temperature = 温度
admin-bot-max-tokens = 最大トークン数
admin-bot-system-prompt = システムプロンプト

# -----------------------------------------------------------------------------
# Tenant Management
# -----------------------------------------------------------------------------
admin-tenants-title = テナント管理
admin-tenants-list = テナント一覧
admin-tenants-add = テナントの追加
admin-tenants-edit = テナントの編集
admin-tenants-delete = テナントの削除
admin-tenants-search = テナントを検索...
admin-tenants-total = テナントの合計
admin-tenants-active = アクティブなテナント
admin-tenants-suspended = 停止中のテナント
admin-tenants-trial = トライアルテナント
admin-tenants-no-tenants = テナントが見つかりませんでした
admin-tenants-confirm-delete = このテナントを削除してもよろしいですか?
admin-tenants-deleted = テナントが正常に削除されました
admin-tenants-saved = テナントが正常に保存されました

# Tenant Details
admin-tenant-details = テナント詳細
admin-tenant-name = テナント名
admin-tenant-domain = ドメイン
admin-tenant-plan = 計画
admin-tenant-plan-free = 無料
admin-tenant-plan-starter = スターター
admin-tenant-plan-professional = プロフェッショナル
admin-tenant-plan-enterprise = エンタープライズ
admin-tenant-users = ユーザー
admin-tenant-bots = ボット
admin-tenant-storage = 使用済みストレージ
admin-tenant-api-calls = API呼び出し
admin-tenant-limits = 使用制限
admin-tenant-billing = 請求情報

# -----------------------------------------------------------------------------
# System Settings
# -----------------------------------------------------------------------------
admin-settings-title = システム設定
admin-settings-general = 一般設定
admin-settings-security = セキュリティ設定
admin-settings-email = メール設定
admin-settings-storage = ストレージ設定
admin-settings-integrations = 統合
admin-settings-api = API設定
admin-settings-appearance = 外観
admin-settings-localization = ローカリゼーション
admin-settings-notifications = 通知
admin-settings-backup = バックアップと復元
admin-settings-maintenance = メンテナンスモード
admin-settings-saved = 設定が正常に保存されました
admin-settings-reset = デフォルトにリセット
admin-settings-confirm-reset = すべての設定をデフォルトにリセットしてもよろしいですか?

# General Settings
admin-settings-site-name = サイト名
admin-settings-site-url = サイトURL
admin-settings-admin-email = 管理者のメールアドレス
admin-settings-support-email = サポートメール
admin-settings-default-language = デフォルトの言語
admin-settings-default-timezone = デフォルトのタイムゾーン
admin-settings-date-format = 日付形式
admin-settings-time-format = 時間の形式
admin-settings-currency = 通貨

# Email Settings
admin-settings-smtp-host = SMTPホスト
admin-settings-smtp-port = SMTPポート
admin-settings-smtp-user = SMTP ユーザー名
admin-settings-smtp-password = SMTPパスワード
admin-settings-smtp-encryption = 暗号化
admin-settings-smtp-from-name = 差出人名
admin-settings-smtp-from-email = メールから
admin-settings-smtp-test = テストメールを送信する
admin-settings-smtp-test-success = テストメールが正常に送信されました
admin-settings-smtp-test-failed = テストメールの送信に失敗しました

# Storage Settings
admin-settings-storage-provider = ストレージプロバイダー
admin-settings-storage-local = ローカルストレージ
admin-settings-storage-s3 = アマゾンS3
admin-settings-storage-minio = MinIO
admin-settings-storage-gcs = Googleクラウドストレージ
admin-settings-storage-azure = Azure Blob ストレージ
admin-settings-storage-bucket = バケット名
admin-settings-storage-region = 地域
admin-settings-storage-access-key = アクセスキー
admin-settings-storage-secret-key = 秘密鍵
admin-settings-storage-endpoint = エンドポイント URL

# -----------------------------------------------------------------------------
# System Logs
# -----------------------------------------------------------------------------
admin-logs-title = システムログ
admin-logs-search = ログを検索...
admin-logs-filter-level = レベルによるフィルター
admin-logs-filter-source = ソースによるフィルター
admin-logs-filter-date = 日付でフィルターする
admin-logs-level-all = すべてのレベル
admin-logs-level-debug = デバッグ
admin-logs-level-info = 情報
admin-logs-level-warning = 警告
admin-logs-level-error = エラー
admin-logs-level-critical = クリティカル
admin-logs-export = ログのエクスポート
admin-logs-clear = ログをクリアする
admin-logs-confirm-clear = すべてのログをクリアしてもよろしいですか?
admin-logs-cleared = ログは正常に消去されました
admin-logs-no-logs = ログが見つかりません
admin-logs-refresh = リフレッシュ
admin-logs-auto-refresh = 自動更新
admin-logs-timestamp = タイムスタンプ
admin-logs-level = レベル
admin-logs-source = ソース
admin-logs-message = メッセージ
admin-logs-details = 詳細

# -----------------------------------------------------------------------------
# Analytics
# -----------------------------------------------------------------------------
admin-analytics-title = 分析
admin-analytics-overview = 概要
admin-analytics-users = ユーザー分析
admin-analytics-bots = ボット分析
admin-analytics-conversations = 会話分析
admin-analytics-performance = パフォーマンス
admin-analytics-period = 期間
admin-analytics-period-today = 今日
admin-analytics-period-week = 今週
admin-analytics-period-month = 今月
admin-analytics-period-quarter = 今四半期
admin-analytics-period-year = 今年
admin-analytics-period-custom = カスタム範囲
admin-analytics-export = レポートのエクスポート
admin-analytics-total-users = 総ユーザー数
admin-analytics-new-users = 新規ユーザー
admin-analytics-active-users = アクティブユーザー
admin-analytics-total-bots = ボットの総数
admin-analytics-active-bots = アクティブなボット
admin-analytics-total-conversations = 総会話数
admin-analytics-avg-response-time = 平均応答時間
admin-analytics-satisfaction-rate = 満足度
admin-analytics-resolution-rate = 解像度レート

# -----------------------------------------------------------------------------
# Security
# -----------------------------------------------------------------------------
admin-security-title = セキュリティ
admin-security-overview = セキュリティの概要
admin-security-audit-log = 監査ログ
admin-security-login-attempts = ログイン試行
admin-security-blocked-ips = ブロックされたIP
admin-security-api-keys = APIキー
admin-security-webhooks = Webhook
admin-security-cors = CORS 設定
admin-security-rate-limiting = レート制限
admin-security-encryption = 暗号化
admin-security-2fa = 二要素認証
admin-security-sso = シングルサインオン
admin-security-password-policy = パスワードポリシー

# API Keys
admin-api-keys-title = APIキー
admin-api-keys-add = APIキーの作成
admin-api-keys-name = キー名
admin-api-keys-key = APIキー
admin-api-keys-secret = 秘密鍵
admin-api-keys-created = 作成されました
admin-api-keys-last-used = 最後に使用したもの
admin-api-keys-expires = 有効期限が切れます
admin-api-keys-never = 決してしない
admin-api-keys-revoke = 取り消し
admin-api-keys-confirm-revoke = この API キーを取り消してもよろしいですか?
admin-api-keys-revoked = API キーが正常に取り消されました
admin-api-keys-created-success = APIキーが正常に作成されました
admin-api-keys-copy = クリップボードにコピー
admin-api-keys-copied = コピーしました！
admin-api-keys-warning = ここで必ず API キーをコピーしてください。もう二度と見ることはできなくなります！

# -----------------------------------------------------------------------------
# Billing
# -----------------------------------------------------------------------------
admin-billing-title = 請求
admin-billing-overview = 請求の概要
admin-billing-current-plan = 現在の計画
admin-billing-usage = 使用法
admin-billing-invoices = 請求書
admin-billing-payment-methods = 支払い方法
admin-billing-upgrade = アップグレードプラン
admin-billing-downgrade = ダウングレードプラン
admin-billing-cancel = サブスクリプションをキャンセルする
admin-billing-invoice-date = 請求書の日付
admin-billing-invoice-amount = 金額
admin-billing-invoice-status = ステータス
admin-billing-invoice-paid = 有料
admin-billing-invoice-pending = 保留中
admin-billing-invoice-overdue = 期限を過ぎました
admin-billing-invoice-download = 請求書のダウンロード

# -----------------------------------------------------------------------------
# Backup & Restore
# -----------------------------------------------------------------------------
admin-backup-title = バックアップと復元
admin-backup-create = バックアップの作成
admin-backup-restore = バックアップを復元する
admin-backup-schedule = バックアップのスケジュールを設定する
admin-backup-list = バックアップ履歴
admin-backup-name = バックアップ名
admin-backup-size = サイズ
admin-backup-created = 作成されました
admin-backup-download = ダウンロード
admin-backup-delete = 削除
admin-backup-confirm-restore = このバックアップを復元してもよろしいですか?これにより、現在のデータが上書きされます。
admin-backup-confirm-delete = このバックアップを削除してもよろしいですか?
admin-backup-in-progress = バックアップ中...
admin-backup-completed = バックアップが正常に完了しました
admin-backup-failed = バックアップに失敗しました
admin-backup-restore-in-progress = 復元中です...
admin-backup-restore-completed = 復元が正常に完了しました
admin-backup-restore-failed = 復元に失敗しました

# -----------------------------------------------------------------------------
# Maintenance Mode
# -----------------------------------------------------------------------------
admin-maintenance-title = メンテナンスモード
admin-maintenance-enable = メンテナンスモードを有効にする
admin-maintenance-disable = メンテナンスモードを無効にする
admin-maintenance-status = 現在の状況
admin-maintenance-active = メンテナンスモードがアクティブです
admin-maintenance-inactive = メンテナンスモードが非アクティブです
admin-maintenance-message = メンテナンスメッセージ
admin-maintenance-default-message = 現在、定期メンテナンスを行っております。すぐにもう一度ご確認ください。
admin-maintenance-allowed-ips = 許可されたIPアドレス
admin-maintenance-confirm-enable = メンテナンス モードを有効にしてもよろしいですか?ユーザーはシステムにアクセスできなくなります。

# -----------------------------------------------------------------------------
# Common Admin UI Elements
# -----------------------------------------------------------------------------
admin-required = 必須
admin-optional = オプション
admin-loading = 読み込み中...
admin-saving = 保存中...
admin-deleting = 削除中...
admin-confirm = 確認する
admin-cancel = キャンセル
admin-save = 保存
admin-create = 作成
admin-update = アップデート
admin-delete = 削除
admin-edit = 編集
admin-view = 見る
admin-close = 閉じる
admin-back = 戻る
admin-next = 次へ
admin-previous = 前へ
admin-refresh = リフレッシュ
admin-export = エクスポート
admin-import = インポート
admin-search = 検索
admin-filter = フィルター
admin-clear = クリア
admin-select = 選択
admin-select-all = すべて選択
admin-deselect-all = すべての選択を解除
admin-actions = アクション
admin-more-actions = さらなるアクション
admin-no-data = 利用可能なデータがありません
admin-error = エラーが発生しました
admin-success = 成功
admin-warning = 警告
admin-info = 情報

# Table Pagination
admin-showing = { $total } 件中 { $from } ～ { $to } を表示中
admin-page = ページ { $current }/{ $total }
admin-items-per-page = ページごとの項目
admin-go-to-page = ページに移動

# Bulk Actions
admin-bulk-delete = 選択したものを削除
admin-bulk-export = 選択したものをエクスポート
admin-bulk-activate = 選択したものをアクティブ化する
admin-bulk-deactivate = 選択したものを非アクティブ化する
admin-selected-count = { $count ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
