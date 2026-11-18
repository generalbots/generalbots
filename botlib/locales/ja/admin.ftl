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

admin-audit-subtitle = システム活動の記録
admin-users-subtitle = このワークスペースにアクセスできるメンバー
admin-bots-subtitle = アシスタントボットの管理
admin-users = ユーザー
admin-groups = グループ
admin-bots = ボット
admin-dns = DNS
admin-audit = 監査ログ
nav-about = バージョン情報
nav-organizations = 組織


# ── #2307 Users & Groups dialogs / status regions ──────────────────
admin-add-user = ユーザーを追加
admin-add-group = グループを追加
admin-search-users = ユーザーを検索...
admin-search-groups = グループを検索...
admin-confirm-title = 操作の確認
admin-confirm-accept = 確認
admin-col-user = ユーザー
admin-col-email = メール
admin-col-organization = 組織
admin-col-roles = ロール
admin-col-status = ステータス
admin-col-actions = 操作
admin-status-active = 有効
admin-status-inactive = 無効
admin-action-edit = 編集
admin-action-delete = 削除
admin-view-members = メンバーを表示
admin-members-suffix = メンバー
admin-creating = 作成中…
admin-user-created = ユーザーを作成し組織に追加しました
admin-group-created = グループを作成しました
admin-user-deleted = ユーザーを削除しました
admin-group-deleted = グループを削除しました
admin-member-removed = メンバーを削除しました
admin-confirm-delete-user = このユーザーを削除しますか？
admin-confirm-delete-group = このグループを削除しますか？
admin-confirm-remove-member = メンバーを削除しますか？
admin-failed-load-users = ユーザーを読み込めませんでした。
admin-failed-load-groups = グループを読み込めませんでした。
admin-failed-create-user = ユーザーを作成できませんでした。
admin-failed-create-group = グループを作成できませんでした。
admin-no-users = ユーザーが見つかりません
admin-no-groups = グループが見つかりません

# semantic i18n key (#2542)
admin-org-switch-title = 組織を切り替え

# semantic i18n key (#2542)
admin-org-create-new-title = 新しい組織を作成

# semantic i18n key (#2542)
admin-org-name-label = 組織名

# semantic i18n key (#2542)
admin-org-url-slug-label = URLスラッグ

# semantic i18n key (#2542)
admin-org-slug-preview = organization-slug

# semantic i18n key (#2542)
admin-org-plan-label = 計画

# semantic i18n key (#2542)
admin-org-plan-free = 無料

# semantic i18n key (#2542)
admin-org-plan-starter = スターター

# semantic i18n key (#2542)
admin-org-create-btn = 組織を作成

# semantic i18n key (#2542)
admin-filter-all-roles = すべてのロール

# semantic i18n key (#2542)
admin-user-role-moderator = モデレーター

# semantic i18n key (#2542)
admin-filter-all-status = すべてのステータス

# semantic i18n key (#2542)
admin-users-last-active = 最終アクティブ

# semantic i18n key (#2542)
admin-users-loading = ユーザーを読み込み中...

# semantic i18n key (#2542)
admin-users-create-new = 新しいユーザーを作成

# semantic i18n key (#2542)
admin-users-send-welcome-email = ウェルカムメールを送信

# semantic i18n key (#2542)
admin-users-require-password-change = 初回ログイン時にパスワード変更を要求

# semantic i18n key (#2542)
admin-users-delete-warning = この操作は元に戻せません。すべてのユーザーデータが完全に削除されます。

# semantic i18n key (#2542)
accounts-subtitle = ソーシャルメディアとメッセージングチャネルの連携を管理します

# semantic i18n key (#2542)
accounts-channel-facebook = Facebook

# semantic i18n key (#2542)
accounts-channel-twitter-x = Twitter / X

# semantic i18n key (#2542)
accounts-channel-bluesky = Bluesky

# semantic i18n key (#2542)
accounts-channel-threads = Threads

# semantic i18n key (#2542)
accounts-channel-tiktok = TikTok

# semantic i18n key (#2542)
accounts-channel-youtube = YouTube

# semantic i18n key (#2542)
accounts-channel-pinterest = Pinterest

# semantic i18n key (#2542)
accounts-channel-reddit = Reddit

# semantic i18n key (#2542)
accounts-channel-snapchat = Snapchat

# semantic i18n key (#2542)
accounts-messaging = メッセージング

# semantic i18n key (#2542)
accounts-channel-whatsapp-business = WhatsApp Business

# semantic i18n key (#2542)
accounts-channel-twilio-sms = Twilio SMS

# semantic i18n key (#2542)
accounts-channel-wechat = WeChat

# semantic i18n key (#2542)
accounts-channel-gmail-google-workspace = Gmail / Google Workspace

# semantic i18n key (#2542)
accounts-channel-outlook-office-365 = Outlook / Office 365

# semantic i18n key (#2542)
accounts-channel-custom-smtp = カスタム SMTP

# semantic i18n key (#2542)
accounts-redirecting-authorization = 認証にリダイレクトしています...

# semantic i18n key (#2542)
accounts-configure-smtp-server = SMTP サーバーを設定

# semantic i18n key (#2542)
accounts-save-configuration = 設定を保存

# semantic i18n key (#2542)
accounts-account-details = アカウントの詳細

# semantic i18n key (#2542)
accounts-connection-status = 接続状態

# semantic i18n key (#2542)
accounts-sample-username = @username

# semantic i18n key (#2542)
accounts-sample-connected-since = 2025年1月15日

# semantic i18n key (#2542)
accounts-permissions = 権限

# semantic i18n key (#2542)
admin-groups-hidden = 非表示

# semantic i18n key (#2542)
admin-groups-detail-title = グループの詳細

# semantic i18n key (#2542)
admin-groups-tab-overview = 概要

# semantic i18n key (#2542)
admin-groups-tab-members = メンバー

# semantic i18n key (#2542)
admin-groups-tab-permissions = 権限

# semantic i18n key (#2542)
admin-groups-tab-kbs = ナレッジベース

# semantic i18n key (#2542)
admin-groups-create-title = 新しいグループを作成

# semantic i18n key (#2542)
admin-groups-visibility = 公開範囲

# semantic i18n key (#2542)
admin-groups-join-policy = 参加ポリシー

# semantic i18n key (#2542)
admin-groups-invite-only = 招待制

# semantic i18n key (#2542)
admin-groups-request-join = 参加をリクエスト

# semantic i18n key (#2542)
admin-groups-allow-member-invite = メンバーが他のユーザーを招待できるようにする

# semantic i18n key (#2542)
admin-groups-enable-chat = グループチャットを有効にする

# semantic i18n key (#2542)
admin-groups-add-members = メンバーを追加

# semantic i18n key (#2542)
admin-groups-search-users = ユーザーを検索

# semantic i18n key (#2542)
admin-groups-selected-users = 選択されたユーザー

# semantic i18n key (#2542)
admin-groups-no-users-selected = ユーザーが選択されていません

# semantic i18n key (#2542)
admin-groups-role-member = メンバー

# semantic i18n key (#2542)
admin-groups-role-moderator = モデレーター

# semantic i18n key (#2542)
admin-groups-send-invites = 招待を送信

# semantic i18n key (#2542)
admin-groups-email-addresses = メールアドレス

# semantic i18n key (#2542)
admin-groups-emails-help = 1行に1つのメールアドレスを入力してください

# semantic i18n key (#2542)
admin-groups-personal-message-optional = 個人的なメッセージ（任意）

# semantic i18n key (#2542)
admin-groups-delete-warning = これにより、すべてのメンバーが削除され、グループのすべてのデータが削除されます。この操作は元に戻せません。

# semantic i18n key (#2542)
admin-billing-dashboard-title = 請求ダッシュボード

# semantic i18n key (#2542)
admin-billing-dashboard-subtitle = 組織の請求と使用状況に関する包括的な分析と洞察

# semantic i18n key (#2542)
admin-billing-dashboard-current-period = 現在の期間

# semantic i18n key (#2542)
admin-billing-dashboard-year-to-date = 年初来

# semantic i18n key (#2542)
admin-billing-dashboard-current-spending = 現在の支出

# semantic i18n key (#2542)
admin-billing-dashboard-quota-utilization = クォータ使用率

# semantic i18n key (#2542)
admin-billing-dashboard-average-quotas = すべてのクォータの平均

# semantic i18n key (#2542)
admin-billing-dashboard-api-calls = API 呼び出し

# semantic i18n key (#2542)
admin-billing-dashboard-storage-used = 使用済みストレージ

# semantic i18n key (#2542)
admin-billing-dashboard-spending-trends = 支出の傾向

# semantic i18n key (#2542)
admin-billing-dashboard-daily = 日次

# semantic i18n key (#2542)
admin-billing-dashboard-weekly = 週次

# semantic i18n key (#2542)
admin-billing-dashboard-cost-breakdown = サービス別コスト内訳

# semantic i18n key (#2542)
admin-billing-dashboard-quota-management = クォータ管理

# semantic i18n key (#2542)
admin-billing-dashboard-team-members = チームメンバー

# semantic i18n key (#2542)
admin-billing-dashboard-no-alerts = アラートは設定されていません

# semantic i18n key (#2542)
admin-billing-dashboard-usage-by-bot = ボット別使用状況

# semantic i18n key (#2542)
admin-billing-dashboard-bot-customer-support = カスタマーサポートボット

# semantic i18n key (#2542)
admin-billing-dashboard-bot-cs-01 = bot_cs_01

# semantic i18n key (#2542)
admin-billing-dashboard-bot-sales-assistant = 営業アシスタント

# semantic i18n key (#2542)
admin-billing-dashboard-bot-sa-01 = bot_sa_01

# semantic i18n key (#2542)
admin-billing-dashboard-bot-hr-assistant = 人事アシスタント

# semantic i18n key (#2542)
admin-billing-dashboard-bot-hr-01 = bot_hr_01

# semantic i18n key (#2542)
admin-billing-dashboard-bot-it-helpdesk = IT ヘルプデスク

# semantic i18n key (#2542)
admin-billing-dashboard-bot-it-01 = bot_it_01

# semantic i18n key (#2542)
admin-billing-dashboard-recent-invoices = 最近の請求書

# semantic i18n key (#2542)
admin-billing-dashboard-alerts = 請求アラート

# semantic i18n key (#2542)
admin-billing-dashboard-invoice-generated-january = 1月分の請求書が生成されました

# semantic i18n key (#2542)
admin-billing-dashboard-quota-settings = クォータ設定

# semantic i18n key (#2542)
admin-billing-dashboard-invoices-limit = 請求書の上限

# semantic i18n key (#2542)
admin-billing-dashboard-team-members-limit = チームメンバーの上限

# semantic i18n key (#2542)
admin-billing-dashboard-bots-limit = ボットの上限

# semantic i18n key (#2542)
admin-billing-dashboard-alert-thresholds = 請求アラートのしきい値

# semantic i18n key (#2542)
admin-billing-subscription-title = 請求とサブスクリプション

# semantic i18n key (#2542)
admin-billing-manage-subscription-desc = 組織のサブスクリプション、支払い方法、請求履歴を管理します

# semantic i18n key (#2542)
admin-billing-per-month = /月

# semantic i18n key (#2542)
admin-billing-next-billing-date = 次回請求日:

# semantic i18n key (#2542)
admin-billing-next-billing-date-value = 2025年2月21日

# semantic i18n key (#2542)
admin-billing-usage-this-period = この期間の使用量

# semantic i18n key (#2542)
admin-billing-usage-period-value = 2025年1月21日 - 2月21日

# semantic i18n key (#2542)
admin-billing-payment-method = 支払い方法

# semantic i18n key (#2542)
admin-billing-card-expiry-value = 有効期限 12/2026

# semantic i18n key (#2542)
admin-billing-billing-address = 請求先住所

# semantic i18n key (#2542)
admin-billing-address-suite = スイート 456

# semantic i18n key (#2542)
admin-billing-address-city = サンフランシスコ, CA 94102

# semantic i18n key (#2542)
admin-billing-address-country = アメリカ合衆国

# semantic i18n key (#2542)
admin-billing-upgrade-your-plan = プランをアップグレード

# semantic i18n key (#2542)
admin-billing-recommended = おすすめ

# semantic i18n key (#2542)
admin-billing-priority-support = 優先サポート

# semantic i18n key (#2542)
admin-billing-upgrade-to-business = Business にアップグレード

# semantic i18n key (#2542)
admin-billing-enterprise = エンタープライズ

# semantic i18n key (#2542)
admin-billing-unlimited-everything = すべて無制限

# semantic i18n key (#2542)
admin-billing-dedicated-support = 専任サポート

# semantic i18n key (#2542)
admin-billing-custom-integrations = カスタム連携

# semantic i18n key (#2542)
admin-billing-sla-guarantee = SLA 保証

# semantic i18n key (#2542)
admin-billing-contact-sales = 営業に連絡

# semantic i18n key (#2542)
admin-billing-prorated-notice = 次回の請求書は、残りの請求期間に基づいて日割り計算されます。

# semantic i18n key (#2542)
admin-billing-sorry-to-see-you-go = ご利用いただきありがとうございました。

# semantic i18n key (#2542)
admin-billing-cancel-consider-options = キャンセルする前に、次のオプションをご検討ください。

# semantic i18n key (#2542)
admin-billing-stay-reduced-rate = 割引料金でご利用を継続

# semantic i18n key (#2542)
admin-billing-accept-offer = オファーを受け入れる

# semantic i18n key (#2542)
admin-billing-pause-your-subscription = サブスクリプションを一時停止

# semantic i18n key (#2542)
admin-billing-pause-up-to-3-months = 最大3か月間休止できます

# semantic i18n key (#2542)
admin-billing-pause-instead = 代わりに一時停止

# semantic i18n key (#2542)
admin-billing-downgrade-smaller-plan = より小さいプランにダウングレード

# semantic i18n key (#2542)
admin-billing-keep-essential-features = 重要な機能を低コストで維持

# semantic i18n key (#2542)
admin-billing-view-plans = プランを見る

# semantic i18n key (#2542)
admin-billing-why-cancelling = キャンセルする理由を教えてください

# semantic i18n key (#2542)
admin-billing-select-reason = 理由を選択...

# semantic i18n key (#2542)
admin-billing-reason-too-expensive = 高すぎる

# semantic i18n key (#2542)
admin-billing-reason-missing-features = 必要な機能がない

# semantic i18n key (#2542)
admin-billing-reason-not-using = 十分に使っていない

# semantic i18n key (#2542)
admin-billing-reason-switching = 別の製品に切り替える

# semantic i18n key (#2542)
admin-billing-reason-temporary = 一時的 - また利用します

# semantic i18n key (#2542)
admin-billing-additional-feedback = 追加のフィードバック（任意）

# semantic i18n key (#2542)
admin-billing-keep-subscription = サブスクリプションを維持
