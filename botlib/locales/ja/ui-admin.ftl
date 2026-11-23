# =============================================================================
# General Bots — ja — Admin, monitoring and system apps
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Panel (additional UI keys)
# -----------------------------------------------------------------------------
admin-panel-title = 管理者パネル

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

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
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
goals-enter-objective-title = 目標のタイトルを入力してください:
goals-edit-objective-title = 目標のタイトルを編集:
goals-delete-objective-confirm = この目標を削除しますか？この操作は取り消せません。
goals-enter-key-result-title = 主要成果のタイトルを入力してください:
goals-enter-target-value = 目標値を入力してください:

# semantic i18n key (#2542)
admin-ops-dashboard-title = 運用ダッシュボード

# semantic i18n key (#2542)
admin-ops-dashboard-subtitle = リアルタイムのシステムメトリクス、分散トレーシング、パフォーマンス監視

# semantic i18n key (#2542)
admin-ops-auto-refresh-on = 自動更新: オン

# semantic i18n key (#2542)
admin-ops-avg-response = 平均応答

# semantic i18n key (#2542)
admin-ops-requests-min = リクエスト/分

# semantic i18n key (#2542)
admin-ops-disk-io = ディスク I/O

# semantic i18n key (#2542)
admin-ops-iops-read-write = IOPS（読み取り/書き込み）

# semantic i18n key (#2542)
admin-ops-in-95-out-50 = 受信: 95 / 送信: 50

# semantic i18n key (#2542)
admin-ops-stable = 安定

# semantic i18n key (#2542)
admin-ops-request-metrics = リクエストメトリクス

# semantic i18n key (#2542)
admin-ops-errors-label = エラー

# semantic i18n key (#2542)
admin-ops-distributed-tracing = 分散トレーシング

# semantic i18n key (#2542)
admin-ops-all-traces = すべてのトレース

# semantic i18n key (#2542)
admin-ops-errors-only = エラーのみ

# semantic i18n key (#2542)
admin-ops-api-requests = API リクエスト

# semantic i18n key (#2542)
admin-ops-post-api-chat-message = POST /api/chat/message

# semantic i18n key (#2542)
admin-ops-trace-a1b2c3d4e5f6 = trace_a1b2c3d4e5f6

# semantic i18n key (#2542)
admin-ops-get-api-kb-search = GET /api/kb/search

# semantic i18n key (#2542)
admin-ops-trace-b2c3d4e5f6g7 = trace_b2c3d4e5f6g7

# semantic i18n key (#2542)
admin-ops-post-api-billing-checkout = POST /api/billing/checkout

# semantic i18n key (#2542)
admin-ops-trace-c3d4e5f6g7h8 = trace_c3d4e5f6g7h8

# semantic i18n key (#2542)
admin-ops-get-api-user-profile = GET /api/user/profile

# semantic i18n key (#2542)
admin-ops-trace-d4e5f6g7h8i9 = trace_d4e5f6g7h8i9

# semantic i18n key (#2542)
admin-ops-api-gateway = API ゲートウェイ

# semantic i18n key (#2542)
admin-ops-primary-2-replicas = プライマリ + 2 レプリカ

# semantic i18n key (#2542)
admin-ops-qdrant-vector-db = Qdrant（ベクトル DB）

# semantic i18n key (#2542)
admin-ops-redis-cache = Redis キャッシュ

# semantic i18n key (#2542)
admin-ops-cluster-mode-3-shards = クラスターモード | 3 シャード

# semantic i18n key (#2542)
admin-ops-llm-service = LLM サービス

# semantic i18n key (#2542)
admin-ops-openai-gpt-4-high-latency = OpenAI GPT-4 | 高レイテンシ

# semantic i18n key (#2542)
admin-ops-object-storage = オブジェクトストレージ

# semantic i18n key (#2542)
admin-ops-s3-compatible = S3 互換

# semantic i18n key (#2542)
admin-ops-error-tracking = エラー追跡

# semantic i18n key (#2542)
admin-ops-view-all-errors = すべてのエラーを表示

# semantic i18n key (#2542)
admin-ops-warnings = 警告

# semantic i18n key (#2542)
admin-ops-payment-processing-failed-stripe-timeout = 支払い処理に失敗しました: Stripe API タイムアウト

# semantic i18n key (#2542)
admin-ops-billing-checkout-rs-145 = billing/checkout.rs:145

# semantic i18n key (#2542)
admin-ops-rate-limit-approaching-openai = OpenAI API のレート制限に近づいています

# semantic i18n key (#2542)
admin-ops-llm-openai-rs-89 = llm/openai.rs:89

# semantic i18n key (#2542)
admin-ops-core-analytics-rs-234 = core/analytics.rs:234

# semantic i18n key (#2542)
admin-ops-cache-miss-rate-above-threshold = キャッシュミス率がしきい値を超えています

# semantic i18n key (#2542)
admin-ops-cache-redis-rs-67 = cache/redis.rs:67

# semantic i18n key (#2542)
admin-ops-endpoint-performance = エンドポイントパフォーマンス

# semantic i18n key (#2542)
admin-ops-by-requests = リクエスト順

# semantic i18n key (#2542)
admin-ops-by-latency = レイテンシ順

# semantic i18n key (#2542)
admin-ops-by-error-rate = エラー率順

# semantic i18n key (#2542)
admin-ops-post = POST

# semantic i18n key (#2542)
admin-ops-api-chat-message = /api/chat/message

# semantic i18n key (#2542)
admin-ops-req-min = リクエスト/分

# semantic i18n key (#2542)
admin-ops-errors-metric = エラー

# semantic i18n key (#2542)
admin-ops-api-kb-search = /api/kb/search

# semantic i18n key (#2542)
admin-roles-title = ロール管理

# semantic i18n key (#2542)
admin-roles-subtitle = ロールを構成し、権限を割り当ててスイート全体のアクセスを制御します

# semantic i18n key (#2542)
admin-roles-sidebar-title = ロール

# semantic i18n key (#2542)
admin-roles-system-filter = システムロール

# semantic i18n key (#2542)
admin-roles-custom-filter = カスタムロール

# semantic i18n key (#2542)
admin-roles-select-title = ロールを選択

# semantic i18n key (#2542)
admin-roles-select-hint = リストからロールを選択して、権限を表示および管理します

# semantic i18n key (#2542)
admin-roles-name-label = ロール名

# semantic i18n key (#2542)
admin-roles-name-badge-placeholder = role_name

# semantic i18n key (#2542)
admin-roles-description-placeholder = ロールの説明がここに表示されます

# semantic i18n key (#2542)
admin-roles-permissions-label = 権限

# semantic i18n key (#2542)
admin-roles-assigned-users-tab = 割り当て済みユーザー

# semantic i18n key (#2542)
admin-roles-assigned-groups-tab = 割り当て済みグループ

# semantic i18n key (#2542)
admin-roles-permission-assignment = 権限の割り当て

# semantic i18n key (#2542)
admin-roles-permission-assignment-help = デュアルリストを使用してこのロールに権限を割り当てます

# semantic i18n key (#2542)
admin-roles-available-permissions = 利用可能な権限

# semantic i18n key (#2542)
admin-roles-category-documents = ドキュメント

# semantic i18n key (#2542)
admin-roles-category-spreadsheets = スプレッドシート

# semantic i18n key (#2542)
admin-roles-category-presentations = プレゼンテーション

# semantic i18n key (#2542)
admin-roles-category-meetings = 会議

# semantic i18n key (#2542)
admin-roles-category-ai-bots = AI とボット

# semantic i18n key (#2542)
admin-roles-category-automation = 自動化

# semantic i18n key (#2542)
admin-roles-assigned-permissions = 割り当て済み権限

# semantic i18n key (#2542)
admin-roles-no-permissions-assigned = 権限が割り当てられていません

# semantic i18n key (#2542)
admin-roles-reset-to-default = 既定にリセット

# semantic i18n key (#2542)
admin-roles-users-with-role = このロールのユーザー

# semantic i18n key (#2542)
admin-roles-groups-with-role = このロールのグループ

# semantic i18n key (#2542)
admin-roles-change-history = ロール変更履歴

# semantic i18n key (#2542)
admin-roles-create-new-role = 新しいロールを作成

# semantic i18n key (#2542)
admin-roles-name-help = 小文字、スペースなし（アンダースコアを使用）

# semantic i18n key (#2542)
admin-roles-copy-permissions-from = 権限のコピー元

# semantic i18n key (#2542)
admin-roles-start-with-no-permissions = 権限なしで開始

# semantic i18n key (#2542)
admin-roles-standard-user = 標準ユーザー

# semantic i18n key (#2542)
admin-roles-power-user = パワーユーザー

# semantic i18n key (#2542)
admin-roles-viewer = 閲覧者

# semantic i18n key (#2542)
admin-roles-guest-user = ゲストユーザー

# semantic i18n key (#2542)
admin-roles-create-role-btn = ロールを作成

# semantic i18n key (#2542)
admin-roles-delete-role-title = ロールを削除

# semantic i18n key (#2542)
admin-roles-delete-warning = これにより、すべてのユーザーとグループからロールが削除されます。この操作は元に戻せません。

# semantic i18n key (#2542)
admin-roles-assign-to-users = ユーザーにロールを割り当て

# semantic i18n key (#2542)
admin-roles-search-users-label = ユーザーを検索

# semantic i18n key (#2542)
admin-roles-type-to-search-users = 入力してユーザーを検索

# semantic i18n key (#2542)
admin-roles-expiration-optional = 有効期限（任意）

# semantic i18n key (#2542)
admin-roles-expiration-help = 永続的な割り当ての場合は空欄のままにします

# semantic i18n key (#2542)
admin-roles-assign-to-groups = グループにロールを割り当て

# semantic i18n key (#2542)
admin-roles-search-groups-label = グループを検索

# semantic i18n key (#2542)
admin-roles-type-to-search-groups = 入力してグループを検索

# semantic i18n key (#2542)
admin-roles-assign-role-btn = ロールを割り当て

# semantic i18n key (#2542)
admin-onboarding-step-welcome = ようこそ

# semantic i18n key (#2542)
admin-onboarding-step-welcome-desc = あなたについて教えてください

# semantic i18n key (#2542)
admin-onboarding-step-org-desc = ワークスペースを作成

# semantic i18n key (#2542)
admin-onboarding-step-plan = プランを選択

# semantic i18n key (#2542)
admin-onboarding-step-plan-desc = サブスクリプションを選択

# semantic i18n key (#2542)
admin-onboarding-step-payment-desc = 支払い方法を追加

# semantic i18n key (#2542)
admin-onboarding-step-start = 始める

# semantic i18n key (#2542)
admin-onboarding-step-start-desc = 最初のボットを作成

# semantic i18n key (#2542)
admin-onboarding-view-guide = ガイドを見る

# semantic i18n key (#2542)
admin-onboarding-welcome-title = General Botsへようこそ！

# semantic i18n key (#2542)
admin-onboarding-welcome-subtitle = 数分でセットアップを完了しましょう。まず、あなたについて少し教えてください。

# semantic i18n key (#2542)
admin-onboarding-full-name = 氏名

# semantic i18n key (#2542)
admin-onboarding-your-role = あなたの役割

# semantic i18n key (#2542)
admin-onboarding-role-placeholder = 役割を選択...

# semantic i18n key (#2542)
admin-onboarding-role-developer = 開発者 / エンジニア

# semantic i18n key (#2542)
admin-onboarding-role-product-manager = プロダクトマネージャー

# semantic i18n key (#2542)
admin-onboarding-role-customer-support = カスタマーサポート

# semantic i18n key (#2542)
admin-onboarding-role-executive = 経営者 / 創業者

# semantic i18n key (#2542)
admin-onboarding-use-case-label = 何を作りたいですか？

# semantic i18n key (#2542)
admin-onboarding-use-case-placeholder = 主なユースケースを選択...

# semantic i18n key (#2542)
admin-onboarding-usecase-support-bot = カスタマーサポートボット

# semantic i18n key (#2542)
admin-onboarding-usecase-sales-assistant = 営業アシスタント

# semantic i18n key (#2542)
admin-onboarding-usecase-internal-kb = 社内ナレッジベース

# semantic i18n key (#2542)
admin-onboarding-usecase-lead-generation = リード獲得

# semantic i18n key (#2542)
admin-onboarding-usecase-employee-onboarding = 従業員オンボーディング

# semantic i18n key (#2542)
admin-onboarding-usecase-ecommerce = EC アシスタント

# semantic i18n key (#2542)
admin-onboarding-usecase-other = その他

# semantic i18n key (#2542)
admin-onboarding-hear-about-label = どこで私たちを知りましたか？

# semantic i18n key (#2542)
admin-onboarding-source-search = 検索エンジン

# semantic i18n key (#2542)
admin-onboarding-source-social = ソーシャルメディア

# semantic i18n key (#2542)
admin-onboarding-source-friend = 友人 / 同僚

# semantic i18n key (#2542)
admin-onboarding-create-org-title = 組織を作成

# semantic i18n key (#2542)
admin-onboarding-create-org-subtitle = 組織では、ボット、チームメンバー、ナレッジベースを管理します。

# semantic i18n key (#2542)
admin-onboarding-org-name = 組織名

# semantic i18n key (#2542)
admin-onboarding-org-name-hint = これはチームメンバーに表示されます

# semantic i18n key (#2542)
admin-onboarding-org-url = 組織 URL

# semantic i18n key (#2542)
admin-onboarding-org-url-prefix = app.generalbots.com/

# semantic i18n key (#2542)
admin-onboarding-org-url-hint = 小文字、数字、ハイフンのみ

# semantic i18n key (#2542)
admin-onboarding-team-size = チーム規模

# semantic i18n key (#2542)
admin-onboarding-team-size-placeholder = 何人が利用しますか？

# semantic i18n key (#2542)
admin-onboarding-team-just-me = 自分だけ

# semantic i18n key (#2542)
admin-onboarding-industry-placeholder = 業界を選択（任意）

# semantic i18n key (#2542)
admin-onboarding-industry-technology = テクノロジー

# semantic i18n key (#2542)
admin-onboarding-industry-healthcare = ヘルスケア

# semantic i18n key (#2542)
admin-onboarding-industry-finance = 金融 / 銀行

# semantic i18n key (#2542)
admin-onboarding-industry-retail = 小売 / EC

# semantic i18n key (#2542)
admin-onboarding-industry-education = 教育

# semantic i18n key (#2542)
admin-onboarding-industry-manufacturing = 製造

# semantic i18n key (#2542)
admin-onboarding-industry-consulting = コンサルティング / サービス

# semantic i18n key (#2542)
admin-onboarding-industry-government = 政府

# semantic i18n key (#2542)
admin-onboarding-industry-nonprofit = 非営利

# semantic i18n key (#2542)
admin-onboarding-choose-plan-title = プランを選択

# semantic i18n key (#2542)
admin-onboarding-choose-plan-subtitle = ニーズに最適なプランを選択してください。いつでもアップグレードやダウングレードが可能です。

# semantic i18n key (#2542)
admin-onboarding-plan-free = 無料

# semantic i18n key (#2542)
admin-onboarding-plan-free-desc = 個人や小規模プロジェクト向け

# semantic i18n key (#2542)
admin-onboarding-per-month = /月

# semantic i18n key (#2542)
admin-onboarding-select-free = 無料を選択

# semantic i18n key (#2542)
admin-onboarding-most-popular = 最も人気

# semantic i18n key (#2542)
admin-onboarding-plan-pro-desc = 成長中のチーム向け

# semantic i18n key (#2542)
admin-onboarding-select-pro = Pro を選択

# semantic i18n key (#2542)
admin-dashboard-organization-title = 組織ダッシュボード

# semantic i18n key (#2542)
admin-dashboard-organization-subtitle = 組織のアクティビティ、メンバー、システム状態の概要

# semantic i18n key (#2542)
admin-dashboard-team-members = チームメンバー

# semantic i18n key (#2542)
admin-dashboard-system-uptime = システム稼働時間

# semantic i18n key (#2542)
admin-dashboard-api-server = API サーバー

# semantic i18n key (#2542)
admin-dashboard-operational = 稼働中

# semantic i18n key (#2542)
admin-dashboard-latency-45ms = レイテンシ: 45ms

# semantic i18n key (#2542)
admin-dashboard-requests-1-2k-min = リクエスト: 1.2K/分

# semantic i18n key (#2542)
admin-dashboard-connections-45-100 = 接続数: 45/100

# semantic i18n key (#2542)
admin-dashboard-query-time-12ms = クエリ時間: 12ms

# semantic i18n key (#2542)
admin-dashboard-vector-database = ベクトルデータベース

# semantic i18n key (#2542)
admin-dashboard-vectors-2-4m = ベクトル: 2.4M

# semantic i18n key (#2542)
admin-dashboard-search-8ms = 検索: 8ms

# semantic i18n key (#2542)
admin-dashboard-tokens-125k-min = トークン: 125K/分

# semantic i18n key (#2542)
admin-dashboard-latency-850ms = レイテンシ: 850ms

# semantic i18n key (#2542)
admin-dashboard-high-usage = 使用率高

# semantic i18n key (#2542)
admin-dashboard-used-34-5gb = 使用量: 34.5GB

# semantic i18n key (#2542)
admin-dashboard-iops-2-1k = IOPS: 2.1K

# semantic i18n key (#2542)
admin-dashboard-members-filter = メンバー

# semantic i18n key (#2542)
admin-dashboard-customer-support-bot = カスタマーサポートボット

# semantic i18n key (#2542)
admin-dashboard-security-alert = セキュリティアラート:

# semantic i18n key (#2542)
admin-dashboard-customer-feedback-survey = 顧客フィードバック調査

# semantic i18n key (#2542)
admin-dashboard-sales-assistant = 営業アシスタント

# semantic i18n key (#2542)
admin-dashboard-load-more-activity = さらにアクティビティを読み込む

# semantic i18n key (#2542)
admin-dashboard-john-smith = John Smith

# semantic i18n key (#2542)
admin-dashboard-role-distribution = ロール分布

# semantic i18n key (#2542)
admin-dashboard-manage-roles = ロールを管理

# semantic i18n key (#2542)
admin-dashboard-bot-status = ボットの状態

# semantic i18n key (#2542)
admin-dashboard-manage-bots = ボットを管理

# semantic i18n key (#2542)
admin-dashboard-handles-customer-inquiries = 顧客からの問い合わせに対応

# semantic i18n key (#2542)
admin-dashboard-product-recommendations = 製品のおすすめ

# semantic i18n key (#2542)
admin-dashboard-hr-assistant = 人事アシスタント

# semantic i18n key (#2542)
admin-dashboard-employee-questions = 従業員からの質問

# semantic i18n key (#2542)
admin-dashboard-it-helpdesk = IT ヘルプデスク

# semantic i18n key (#2542)
admin-dashboard-technical-support = 技術サポート

# semantic i18n key (#2542)
admin-dashboard-pending-invitations = 保留中の招待

# semantic i18n key (#2542)
admin-dashboard-loading-invitations = 招待を読み込み中…

# semantic i18n key (#2542)
admin-dashboard-invite-member = メンバーを招待

# semantic i18n key (#2542)
admin-dashboard-bulk-invite = 一括招待

# semantic i18n key (#2542)
admin-dashboard-emails-label = メールアドレス（1行に1件、カンマまたはセミコロン区切り）

# semantic i18n key (#2542)
admin-dashboard-manage-users = ユーザーを管理

# semantic i18n key (#2542)
admin-dashboard-roles-permissions = ロールと権限

# semantic i18n key (#2542)
workspace-placeholder-untitled = 無題

# semantic i18n key (#2542)
admin-dashboard-placeholder-invite-email = teammate@example.com
admin-dashboard-placeholder-bulk-emails = a@example.com
b@example.com

# semantic i18n key (#2542)
admin-roles-placeholder-search = ロールを検索...
admin-roles-placeholder-filter-available = 利用可能を絞り込み...
admin-roles-placeholder-filter-assigned = 割り当て済みを絞り込み...
admin-roles-placeholder-name-example = 例: content_editor
admin-roles-placeholder-display-name-example = 例: コンテンツ編集者
admin-roles-placeholder-description = このロールの目的を説明してください...
admin-roles-placeholder-search-users = 入力してユーザーを検索...
admin-roles-placeholder-search-groups = 入力してグループを検索...

# semantic i18n key (#2542)
admin-onboarding-placeholder-full-name = 山田 太郎
admin-onboarding-placeholder-org-name = 株式会社サンプル
admin-onboarding-placeholder-org-slug = sample
admin-onboarding-team-2-5 = 2〜5人
admin-onboarding-team-6-20 = 6〜20人
admin-onboarding-team-21-50 = 21〜50人
admin-onboarding-team-51-plus = 50人以上
# semantic i18n key (#2542)
admin-ops-configure-alerts = アラートを設定
admin-ops-trace-search-placeholder = トレースIDを検索...
admin-ops-traces-slow = 遅い (>500ms)

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
workspace-state-empty-recent-title = まだ何も開いていません
workspace-state-empty-recent-desc = 開いたページがここに表示され、続きから再開できます。
workspace-state-empty-favorites-title = まだお気に入りがありません
workspace-state-empty-favorites-desc = ページをお気に入りにすると、サイドバーの上部に固定されます。
workspace-state-empty-pages-title = まだページがありません
workspace-state-empty-pages-desc = ページはこのワークスペースのメモと文書です。最初のページを作成しましょう。

# -----------------------------------------------------------------------------
# #2604 — Protection panel (status comes from /api/security/protection)
# -----------------------------------------------------------------------------
protection-title = Protection
protection-subtitle = Host security tools reported by the server — never a hardcoded "enabled" state.
protection-scanner-status = Scanner status
protection-loading = Loading scanner status…
protection-unknown = unknown
protection-days-old = {days} days old
protection-unknown-tool = unknown tool
protection-active = Active
protection-not-scanning = NOT SCANNING — no scanner is active
protection-version = Version
protection-signature-age = Signature age
protection-last-scan = Last scan
protection-never = never
protection-run-scan = Run scan
protection-opt-out = Uploads are NOT scanned: unscanned imports are allowed.
protection-no-scanner = No scanner is registered.
protection-quarantine = Quarantine
protection-tool = Tool
protection-list = List quarantined files
protection-quarantine-empty = No quarantined files.
protection-restore = Restore
protection-action-failed = The action failed.
protection-load-error = Scanner status is unavailable.
protection-clamav = ClamAV
protection-lmd = LMD
protection-quarantined-count = {count} 件の隔離
