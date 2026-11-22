# =============================================================================
# General Bots — ja — Settings
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

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
settings-create-api-key = APIキーの作成
settings-api-key-name = キー名
settings-api-key-scopes = スコープ (カンマ区切り)
settings-api-key-scopes-hint = デフォルトの「user」スコープの場合は空のままにします。
settings-api-key-expiry = 有効期限は (日) で切れます
settings-api-key-expiry-hint = 有効期限を設けない場合は空のままにします。
settings-add-webhook = Webhook の追加
settings-webhook-url = Webhook URL
settings-webhook-events = イベント (カンマ区切り)
settings-webhook-events-hint = すべてのイベントを受信するには、空のままにします。
settings-timezone = タイムゾーン
settings-date-format = 日付形式
settings-time-format = 時間の形式

# -----------------------------------------------------------------------------
# Settings - Language & Localization
# -----------------------------------------------------------------------------
settings-language-desc = 好みの言語を選択してください
settings-display-language = 表示言語
settings-language-affects = アプリケーション内のすべてのテキストに影響します
settings-date-format-desc = 日付の表示方法
settings-time-format-desc = 12 時間制または 24 時間制
settings-language-changed = 言語が正常に変更されました
settings-reload-required = 変更を適用するにはページのリロードが必要です

# Settings - Profile
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

# Settings → Model (#2568)
mdl-title = モデル
mdl-subtitle = アシスタントが考える場所です。このマシンで動くモデル、プロバイダーのエンドポイント、無料のホストモデルから選びます。
mdl-current = 現在のモデル
mdl-where = アシスタントが考える場所
mdl-local = このコンピューター
mdl-own = 組織のサーバーまたはプロバイダーキー
mdl-free = 無料のホストモデル
mdl-active = 使用中
mdl-endpoint = エンドポイントURL
mdl-model = モデル
mdl-key = APIキー
mdl-key-stored = キー保存済み
mdl-test = テスト
mdl-save = 保存
mdl-test-ok = 接続可能、レイテンシ %ms ms
mdl-test-fail = 失敗: %e
mdl-testing = テスト中…
mdl-saving = 保存中…
mdl-saved = 保存しました。次のメッセージから有効になります。
mdl-swapped = 準備完了。新しいモデルが有効です。
mdl-restart = 保存しました。新しいモデルを使うにはアプリを再起動してください。
mdl-admin-only = モデルを変更できるのは管理者のみです。
mdl-loading = 読み込み中…
mdl-local-active = 有効なローカルモデル: %m
mdl-local-hint = On-device AI ページでモデルを選ぶと、ここで使用できます。
mdl-local-open = On-device AI を開く
mdl-free-note = 入力・添付した内容は %h に送信されます。
mdl-inconsistent = モデル設定が矛盾しています。
mdl-restore-free = 無料モデルに戻す
mdl-confirm-external = 入力・添付した内容はこのコンピューターの外にある %h に送信されます。続行しますか？
mdl-scope-machine = このコンピューター
mdl-scope-network = 組織のサーバー (%h)
mdl-scope-external = 外部ホスト (%h)
mdl-unavailable = このサーバーはモデルAPIを公開していません。
# Settings → Model — provider picker and per-key models (#2568)
mdl-prov-intro = 选择提供商。地址会自动填入；您只需选择模型并粘贴密钥。
mdl-prov-search = 搜索提供商
mdl-prov-connect = 连接
mdl-prov-none = 没有该名称的提供商。
mdl-prov-org-title = 您的组织服务器
mdl-prov-org-desc = 任何与 OpenAI API 兼容的服务器（vLLM、Ollama、LM Studio、Azure…）
mdl-prov-add = 添加
mdl-prov-back = 所有提供商
mdl-prov-address = 地址
mdl-prov-auto = 自动
mdl-prov-url-ph = https://server.example/v1/chat/completions
mdl-prov-model-list = 模型列表
mdl-prov-other = 其他模型…
mdl-prov-other-ph = 准确的模型名称
mdl-prov-get-key = 获取密钥
mdl-prov-key-ph = 在此粘贴密钥
mdl-prov-key-opt = 如果服务器不需要密钥，请留空。
mdl-prov-key-again = 已存储密钥。请重新粘贴密钥以保存。
mdl-prov-sent = 您编写和附加的内容将发送到 %h（此计算机之外）。
mdl-prov-said = 提供商返回：
mdl-prov-test = 测试连接
mdl-prov-use = 使用此提供商
mdl-prov-url-needed = 请输入服务器地址。
mdl-prov-model-needed = 请选择模型。
mdl-prov-key-needed = 请粘贴 API 密钥。
mdl-err-key-refused = 密钥被拒绝
mdl-err-provider-refused = 提供商拒绝了请求：请检查密钥和模型
mdl-err-not-found = 找不到模型或地址
mdl-err-limit = 已达到使用限额
mdl-err-no-answer = 服务器未响应
mdl-err-timeout = 服务器响应超时
mdl-prov-desc-openai = 模型 GPT
mdl-prov-desc-xai = 模型 Grok
mdl-prov-desc-kimi = 模型 Kimi
mdl-prov-desc-zhipu = 模型 GLM
mdl-prov-desc-hunyuan = 模型 Hunyuan
mdl-prov-desc-minimax = 模型 MiniMax
mdl-prov-desc-anthropic = Claude Opus、Sonnet 和 Haiku
mdl-prov-desc-gemini = 通过 Google AI Studio 使用 Gemini
mdl-prov-desc-meta = 通过 Meta API 使用 Llama
mdl-prov-desc-mistral = Mistral Large 和 Small（欧洲）
mdl-prov-desc-deepseek = DeepSeek Chat 和 Reasoner
mdl-prov-desc-qwen = 通过 Model Studio 使用 Qwen
mdl-prov-desc-openrouter = 一个密钥即可使用数百个模型
mdl-prov-desc-groq = 响应极快的开放模型
mdl-km-title = 此密钥的模型
mdl-km-fetch = 获取模型
mdl-km-hint = 获取此密钥可用的模型，并开启要在聊天菜单中显示的模型。
mdl-km-unavailable = 此服务器不支持模型发现功能。
mdl-km-no-match = 没有该名称的模型。
mdl-km-filter = 筛选模型
mdl-km-count = %t 个中已开启 %n 个
mdl-km-save = 保存模型
mdl-km-fetch-fail = 无法获取模型（%e）。
mdl-km-empty = 提供商未返回任何模型。
mdl-km-fetching = 正在获取模型…
mdl-km-saving = 正在保存…
mdl-km-saved = 模型已保存。已开启的模型将显示在聊天菜单中。
mdl-km-save-fail = 无法保存（%e）。
mdl-km-stored = 密钥字段为空时，搜索将使用已存储的密钥。

# Settings → Instructions, Shortcuts, Text size (#2568)
instr-title = カスタム指示
instr-subtitle = アシスタントにあなた自身と回答の希望を伝えます。
instr-enabled = カスタム指示を有効化
instr-enabled-desc = オフのとき、アシスタントはこの指示なしで回答します。
instr-about = アシスタントに知っておいてほしいこと
instr-about-placeholder = 役割、言語、状況、好み…
instr-response = アシスタントの回答方法
instr-response-placeholder = 口調、詳細度、形式…
instr-save = 保存
instr-saving = 保存中…
instr-saved = 保存しました。
instr-save-failed = 保存できませんでした: %e
instr-loading = 読み込み中…
instr-load-failed = 読み込めませんでした: %e
instr-unavailable = このサーバーはカスタム指示を提供していません。
sc-subtitle = スイート内のどこでも Shift + ? を押すとこの一覧が開きます。
sc-group-general = 一般
sc-group-drive = ドライブ
sc-note = 入力中はショートカットが無効になり、メッセージやフォームを中断しません。
ts-title = 文字サイズ
ts-desc = スイート全体の文字サイズを選択
ts-small = 小
ts-default = 標準
ts-large = 大
ts-xlarge = 特大
# Settings → Appearance: chat background pattern
chatbg-title = チャットの背景
chatbg-desc = 会話の背景パターンを選択
chatbg-none = なし
chatbg-image = 紙
chatbg-dots = ドット
chatbg-microdots = マイクロドット
chatbg-grid = グリッド
chatbg-blueprint = 設計図
chatbg-weave = 織り目
chatbg-mosaic = モザイク

# semantic i18n key (#2542)
settings-network-participation-desc = このインストールがピアネットワークに参加するかどうかを制御します

# semantic i18n key (#2542)
settings-p2p-network = P2Pネットワーク

# semantic i18n key (#2542)
settings-network-mode-label = ネットワークモード

# semantic i18n key (#2542)
settings-network-mode-p2p = P2P（推奨）
settings-network-p2p-description = オフラインモードではすべての接続が <code>127.0.0.1</code> に保たれ、共有リンクはローカルネットワーク内でのみ機能します。<br />P2P は、共有リンクの無料リレーやピア検出などの将来機能を有効にします。明示的に共有するまで、このマシンからデータが外部に送信されることはありません。
settings-network-p2p-restart-hint = この設定はローカルに保存され、アプリの再起動後に有効になります。リレーのエンドポイントは固定です (<code id="network-relay-host">peer.generalbots.org</code>)。ピアおよびホームサイトの解決には常にこれが使用されるため、手動設定は不要です。

# semantic i18n key (#2542)
settings-role-viewer = 閲覧者

# semantic i18n key (#2542)
settings-register-dns-hostname = DNSホスト名を登録

# semantic i18n key (#2542)
settings-dns-aaaa-ipv6 = AAAA (IPv6)

# semantic i18n key (#2542)
settings-dns-cname = CNAME

# semantic i18n key (#2542)
settings-create-token = トークンを作成

# semantic i18n key (#2542)
settings-system-preview-desc = このデプロイメントのプレビューモードとバージョン情報

# semantic i18n key (#2542)
settings-system-preview-surface-desc = インストール内容ではなく、表示される範囲を制御します。

# semantic i18n key (#2542)
settings-system-preview-toggle-desc = プレビュー中のアプリと設定ツリー全体を表示します。オフにするとこのエディションの基本設定のみ表示されます。

# semantic i18n key (#2542)
settings-component-versions = コンポーネントのバージョン

# semantic i18n key (#2542)
settings-component-versions-desc = このインストールの BotServer、BotUI、サービスのバージョン

# semantic i18n key (#2542)
settings-open-versions = バージョンを開く

# semantic i18n key (#2542)
settings-on-device-ai = デバイス上のAI

# semantic i18n key (#2542)
settings-operating-system = オペレーティングシステム

# semantic i18n key (#2542)
settings-ledger = 台帳

# semantic i18n key (#2542)
settings-updates = 更新

# semantic i18n key (#2542)
settings-this-workspace = このワークスペース

# semantic i18n key (#2542)
settings-loading-identity = IDを読み込み中…

# semantic i18n key (#2542)
settings-plan-placeholder = プラン: —

# semantic i18n key (#2542)
settings-preview-placeholder = プレビュー: —

# semantic i18n key (#2542)
settings-oda-subtitle = このマシン上で動作するモデルです。ハードウェアに合ったモデルをダウンロードして有効化すると、ローカルエンジンは次回起動時にそれを使用します。ここではAPIキーは使用されません。

# semantic i18n key (#2542)
settings-oda-detecting-hardware = ハードウェアを検出中…

# semantic i18n key (#2542)
settings-oda-recommended-for-machine = このマシンに推奨

# semantic i18n key (#2542)
settings-oda-activate = 有効化

# semantic i18n key (#2542)
settings-oda-models = モデル

# semantic i18n key (#2542)
settings-oda-model = モデル

# semantic i18n key (#2542)
settings-oda-params = パラメータ

# semantic i18n key (#2542)
settings-oda-quant = 量子化

# semantic i18n key (#2542)
settings-oda-min-vram = 最小VRAM

# semantic i18n key (#2542)
settings-oda-loading-models = モデルを読み込み中…

# semantic i18n key (#2542)
settings-date-format-mm-dd-yyyy-us = MM/DD/YYYY（米国）

# semantic i18n key (#2542)
settings-date-format-dd-mm-yyyy-eu-br = DD/MM/YYYY（EU/ブラジル）

# semantic i18n key (#2542)
settings-date-format-yyyy-mm-dd-iso = YYYY-MM-DD（ISO）

# semantic i18n key (#2542)
settings-timezone-america-new-york = ニューヨーク (GMT-5)

# semantic i18n key (#2542)
settings-timezone-america-los-angeles = ロサンゼルス (GMT-8)

# semantic i18n key (#2542)
settings-timezone-europe-london = ロンドン (GMT+0)

# semantic i18n key (#2542)
settings-timezone-europe-paris = パリ (GMT+1)

# semantic i18n key (#2542)
settings-timezone-asia-tokyo = 東京 (GMT+9)

# semantic i18n key (#2542)
settings-desktop-wallpaper = デスクトップの壁紙

# semantic i18n key (#2542)
settings-desktop-wallpaper-desc = デスクトップに表示する背景を選択します

# semantic i18n key (#2542)
settings-organizations-bots = 組織とボット

# semantic i18n key (#2542)
settings-organizations-bots-desc = 組織とボットの階層を管理します

# semantic i18n key (#2542)
settings-current-organization = 現在の組織

# semantic i18n key (#2542)
settings-current-organization-desc = 管理する組織を選択してください

# semantic i18n key (#2542)
settings-loading-organizations = 組織を読み込み中...

# semantic i18n key (#2542)
settings-bot-tree = ボットツリー

# semantic i18n key (#2542)
settings-bot-tree-desc = 選択した組織に属するボット

# semantic i18n key (#2542)
settings-loading-bots = ボットを読み込み中...

# semantic i18n key (#2542)
settings-storage-sync-desc = ストレージと同期の設定を管理します

# semantic i18n key (#2542)
settings-knowledge = ナレッジ

# semantic i18n key (#2542)
settings-drive-files-as-context = すべてのDriveファイルをナレッジコンテキストとして使用する

# semantic i18n key (#2542)
settings-drive-files-as-context-desc = 有効（デフォルト）の場合、最近のDriveファイル名が各チャットのやり取りに含まれ、アシスタントがコンテキストとして使用できます。Driveをプロンプトから除外するにはオフにしてください。

# semantic i18n key (#2542)
settings-loading-storage-information = ストレージ情報を読み込み中...

# semantic i18n key (#2542)
settings-connected-cloud-storage = 接続済みクラウドストレージ

# semantic i18n key (#2542)
settings-loading-storage-connections = ストレージ接続を読み込み中...

# semantic i18n key (#2542)
settings-os-subtitle = このシェルインストール用の低レベルなホスト設定とツール

# semantic i18n key (#2542)
settings-os-host-identity-desc = ホストの識別情報、カーネル、タイムゾーン

# semantic i18n key (#2542)
settings-os-kernel = カーネル

# semantic i18n key (#2542)
settings-os-architecture = アーキテクチャ

# semantic i18n key (#2542)
settings-os-host-readonly-hint = この段階ではホスト名とタイムゾーンは読み取り専用です。変更するには、ターミナルアプリまたはキオスクイメージのホストツールを使用してください。

# semantic i18n key (#2542)
settings-os-services-desc = ホストサービスの開始、停止、再起動

# semantic i18n key (#2542)
settings-os-load-services-hint = パネルを開いてサービスを読み込みます。

# semantic i18n key (#2542)
settings-os-disk-usage-desc = マウントされたファイルシステムのディスク使用量

# semantic i18n key (#2542)
settings-os-load-storage-hint = パネルを開いてストレージを読み込みます。

# semantic i18n key (#2542)
settings-os-log-files = ログファイル

# semantic i18n key (#2542)
settings-os-log-files-desc = このインストールが書き込むファイル（場所とサイズ付き）

# semantic i18n key (#2542)
settings-os-load-log-files-hint = パネルを開いてログファイルを読み込みます。

# semantic i18n key (#2542)
settings-os-tail-log-desc = サービスログの最後の200行を表示

# semantic i18n key (#2542)
settings-os-reload = 再読み込み

# semantic i18n key (#2542)
settings-os-restart-service-title = サービスを再起動

# semantic i18n key (#2542)
settings-os-the-service = サービス

# semantic i18n key (#2542)
settings-os-service-name = サービス名

# semantic i18n key (#2542)
settings-os-restart-action = 再起動

# semantic i18n key (#2542)
settings-os-restart-confirm-message = <strong id="os-restart-service-name" data-i18n="settings-os-the-service">サービス</strong>を再起動しますか？確認のためサービス名を入力してください。再起動は監査ログに記録されます。

# semantic i18n key (#2542)
settings-profile-directory-managed = Zitadel ディレクトリで管理

# semantic i18n key (#2542)
settings-profile-open-directory-profile = ディレクトリプロファイルを開く

# semantic i18n key (#2542)
settings-profile-public-title = 公開プロフィール

# semantic i18n key (#2542)
settings-profile-public-desc = General Bots ホームページでの表示方法

# semantic i18n key (#2542)
settings-profile-public-display-name = 公開表示名

# semantic i18n key (#2542)
settings-profile-public-url-label = プロフィール URL は次のようになります

# semantic i18n key (#2542)
settings-profile-coming-soon = 近日公開

# semantic i18n key (#2542)
settings-profile-public-bio = 公開プロフィール

# semantic i18n key (#2542)
settings-profile-public-url-placeholder = your-name.generalbots.org

# semantic i18n key (#2542)
settings-profile-language-region = 言語と地域

# semantic i18n key (#2542)
settings-profile-public-url-hint-html = General Bots ホームサイトリレーが提供されると（#1867）、ページは <strong><span id="public-profile-url-hint" data-i18n="settings-profile-public-url-placeholder">your-name.generalbots.org</span></strong> で公開されます。

# semantic i18n key (#2542)
settings-security-directory-note = ID、セッション、2FA は Zitadel ディレクトリで管理されます

# semantic i18n key (#2542)
settings-security-open-directory-security = ディレクトリのセキュリティを開く

# semantic i18n key (#2542)
settings-security-connected-devices = 接続済みデバイス

# semantic i18n key (#2542)
settings-security-connected-devices-desc = アカウントにアクセスしたデバイス

# semantic i18n key (#2542)
settings-security-local-data = ローカルデータ

# semantic i18n key (#2542)
settings-security-local-data-desc = このインストールでは、すべてがこのデバイスに保存されます。

# semantic i18n key (#2542)
settings-security-export-local-data = ローカルデータをエクスポート

# semantic i18n key (#2542)
settings-security-export-local-data-desc = このデバイスに保存されているデータをダウンロード

# semantic i18n key (#2542)
settings-security-clear-local-cache = ローカルキャッシュをクリア

# semantic i18n key (#2542)
settings-security-clear-local-cache-desc = このブラウザプロファイルに保存されたキャッシュ設定と下書きを削除します

# semantic i18n key (#2542)
settings-security-clear-cache-action = キャッシュをクリア

# semantic i18n key (#2542)
settings-shared-links-subtitle = 公開ファイルリンクを送信先の相手にどのように届けるかを選択します

# semantic i18n key (#2542)
settings-shared-links-loopback-warning-title = 現在、このマシンにしかアクセスできません。

# semantic i18n key (#2542)
settings-shared-links-loopback-warning-body-html = コピーしたループバックリンク（<code>127.0.0.1</code>）は、あなたのコンピューターではなく受信者自身のコンピューターで開きます。このマシン外でリンクを共有する前に、LANアドレスを使用するか、ドメインを設定するか、無料の General Bots アドレスを待ってください。

# semantic i18n key (#2542)
settings-shared-links-lan-title = このネットワーク

# semantic i18n key (#2542)
settings-shared-links-lan-desc = Wi-Fi または有線ネットワーク上の全員

# semantic i18n key (#2542)
settings-shared-links-on-this-computer = このコンピューター上

# semantic i18n key (#2542)
settings-shared-links-on-this-network = このネットワーク上

# semantic i18n key (#2542)
settings-shared-links-lan-detail = 同じネットワーク上の誰でも利用できます。このマシンからデータが外部に出ることはありません。

# semantic i18n key (#2542)
settings-shared-links-lan-config-title = 設定

# semantic i18n key (#2542)
settings-shared-links-lan-config-body-html = この階層は既定で有効です。ローカルサーバーはすべてのインターフェイスで待ち受けます。同じネットワーク上の別のデバイスからネットワークアドレスを開いてください。デバイス間の通信がブロックされている場合は、このコンピューターで受信 TCP ポート <code id="shared-links-port">8080</code> を許可してください。

# semantic i18n key (#2542)
settings-shared-links-lan-config-local-html = <code>.local</code> 名が解決されない場合は、このコンピューターの IPv4 アドレスを使用してください（例: <code>http://192.168.1.20:8080</code>）。

# semantic i18n key (#2542)
settings-shared-links-domain-title = 自分のドメイン

# semantic i18n key (#2542)
settings-shared-links-domain-desc = 自分で管理でき、インターネットからアクセス可能な名前

# semantic i18n key (#2542)
settings-shared-links-public-link = 公開リンク

# semantic i18n key (#2542)
settings-shared-links-domain-not-configured = 未設定 — 新しいリンクは上記のローカルアドレスにフォールバックします。

# semantic i18n key (#2542)
settings-shared-links-domain-config-title = 設定方法

# semantic i18n key (#2542)
settings-shared-links-domain-config-dns-html = DNS レコード（例: <code>files.example.com</code>）をこのコンピューターの公開アドレスに向けてください。

# semantic i18n key (#2542)
settings-shared-links-domain-config-url-html = このインストールの公開ベース URL を設定します: <pre><code>PUBLIC_BASE_URL=https://files.example.com</code></pre> または製品ファイルで: <pre><code>public_base_url=https://files.example.com</code></pre>

# semantic i18n key (#2542)
settings-shared-links-domain-config-restart = 新しい共有リンクがドメインで作成されるようにデスクトップアプリを再起動してください。既存のリンクはトークンを保持します。

# semantic i18n key (#2542)
settings-shared-links-domain-config-hint = レジストラトークンはローカルの保管庫に保存され、ログには書き込まれません。自動 DNS 更新と TLS 証明書はこの issue で追跡されているドメインプロバイダー作業の一部です。それが提供されるまでは、レジストラでレコードを最新の状態に保ってください。

# semantic i18n key (#2542)
settings-shared-links-free-title = 無料の General Bots アドレス

# semantic i18n key (#2542)
settings-shared-links-free-desc = General Bots が提供する名前。ドメインは不要です

# semantic i18n key (#2542)
settings-shared-links-free-unavailable-html = まだ利用できません。デスクトップはオフラインで動作するため、この階層には無料の DNS ゾーンと、直接到達できないマシン用のリレーが必要です。#1864 で追跡されている <code>botpublic</code> 作業とともに無効の状態で提供されます。

# semantic i18n key (#2542)
settings-shared-links-free-what-title = 実行される内容

# semantic i18n key (#2542)
settings-shared-links-free-what-body-html = 無料アカウントは名前を保持します。ファイルは保持しません。マシンに直接到達できない場合、管理されたリレー（<code id="shared-links-relay-host">peer.generalbots.org</code>）は共有ファイルの暗号化されたコピーのみを保持し、そのコピーは期限切れになります。手動でのリレー設定は不要です。

# semantic i18n key (#2542)
settings-shared-links-peer-title = マスターピアスイッチ

# semantic i18n key (#2542)
settings-shared-links-peer-desc-html = このインストールのすべての P2P トラフィックを停止します。オフラインモードでは、すべての接続が <code>127.0.0.1</code> に保たれます。

# semantic i18n key (#2542)
settings-shared-links-offline-default = オフライン（既定）

# semantic i18n key (#2542)
settings-shared-links-p2p-opt-in = P2P（オプトイン）

# semantic i18n key (#2542)
settings-shared-links-relay-hint-html = リレーエンドポイントは固定です（<code>peer.generalbots.org</code>）。この設定を変更すると、完全に反映するためにアプリの再起動が必要になる場合があります。

# semantic i18n key (#2542)
settings-shared-links-check-again = 再確認

# semantic i18n key (#2542)
settings-admin-bots-loading = ボットを読み込み中...

# semantic i18n key (#2542)
settings-admin-domain-configuration = ドメイン設定

# semantic i18n key (#2542)
settings-admin-audit-loading = 監査ログを読み込み中...

# semantic i18n key (#2542)
settings-updates-subtitle = 最新の GitHub リリースを確認し、公式ダウンロードへ移動します

# semantic i18n key (#2542)
settings-updates-desktop-title = デスクトップの更新

# semantic i18n key (#2542)
settings-updates-desktop-checks-html = アプリは <code>github.com/generalbots/generalbots/releases/latest</code> を確認します。インストールは常に任意です。

# semantic i18n key (#2542)
settings-updates-check-startup = 起動時に更新を確認する

# semantic i18n key (#2542)
settings-updates-check-startup-desc = 1日に最大1回実行され、通知のみを表示します。これがオフの場合でも「今すぐ確認」を使用できます。

# semantic i18n key (#2542)
settings-updates-current-version = 現在のバージョン

# semantic i18n key (#2542)
settings-updates-never = なし

# semantic i18n key (#2542)
settings-updates-check-now = 今すぐ確認

# semantic i18n key (#2542)
settings-updates-view-release-notes = リリースノートを表示

# semantic i18n key (#2542)
settings-updates-not-checked-yet = まだ確認されていません。

# semantic i18n key (#2542)
settings-versions-subtitle = コンポーネントのバージョンと更新の可用性

# semantic i18n key (#2542)
settings-versions-loading = バージョン情報を読み込み中...

# semantic i18n key (#2542)
settings-ledger-description = すべてのターミナルおよびブラウザー操作、同意の判断、結果、スクリーンショットのパス。

# semantic i18n key (#2542)
settings-ledger-actions-title = アクション台帳

# semantic i18n key (#2542)
settings-ledger-tab-terminal = ターミナル

# semantic i18n key (#2542)
settings-ledger-tab-browser = ブラウザー

# semantic i18n key (#2542)
settings-ledger-tab-budgets = 予算

# semantic i18n key (#2542)
settings-ledger-tab-ttft = TTFT

# semantic i18n key (#2542)
settings-integrations-subtitle = 外部サービスを接続し、API アクセスを管理します

# semantic i18n key (#2542)
settings-open-integrations = 統合を開く

# semantic i18n key (#2542)
settings-personal-access-tokens = 個人アクセストークン

# semantic i18n key (#2542)
settings-personal-tokens-desc = 自分として API を呼び出すために発行するトークン（<code>gb_…</code>）。モデルプロバイダーのキーは統合アプリにあります。

# semantic i18n key (#2542)
settings-no-api-keys = API キーはまだ作成されていません

# semantic i18n key (#2542)
settings-webhooks-title = Webhook

# semantic i18n key (#2542)
settings-no-webhooks = Webhook が設定されていません

# semantic i18n key (#2542)
settings-full-catalog = 全カタログ

# semantic i18n key (#2542)
settings-loading-mcp-servers = MCP サーバーを読み込み中

# semantic i18n key (#2542)
settings-agent-skills = エージェントスキル

# semantic i18n key (#2542)
settings-import-skill = スキルをインポート

# semantic i18n key (#2542)
settings-loading-skills = スキルを読み込み中

# semantic i18n key (#2542)
settings-ai-accounts-subtitle = Claude Code と ChatGPT のプラン契約はユーザーごとです。認証情報はエージェント VM またはローカルボールトに保持され、ここにはステータスと表示メタデータのみが保存されます。

# semantic i18n key (#2542)
settings-subscription-accounts = サブスクリプションアカウント

# semantic i18n key (#2542)
settings-ai-accounts-desc = アカウントを接続し、Vibe エージェントタイプに割り当てます

# semantic i18n key (#2542)
settings-loading-ai-accounts = AI アカウントを読み込み中…

# semantic i18n key (#2542)
settings-vibe-subtitle = 各 Vibe エージェントタイプに LLM プロバイダーを割り当てます。プロバイダーは統合 → モデルプロバイダー（BYOK）またはクラウドストアで定義されます。

# semantic i18n key (#2542)
settings-agent-llm-providers = エージェント LLM プロバイダー

# semantic i18n key (#2542)
settings-agent-llm-providers-desc = 各エージェントタイプは、このブランチでここに選択されたプロバイダーに実行をルーティングします

# semantic i18n key (#2542)
settings-loading-agent-providers = エージェントプロバイダーを読み込み中…

# semantic i18n key (#2542)
settings-privacy-subtitle = データとプライバシー設定を管理します

# semantic i18n key (#2542)
settings-privacy-preferences = プライバシー設定

# semantic i18n key (#2542)
settings-profile-visibility = プロフィールの公開範囲

# semantic i18n key (#2542)
settings-profile-visibility-desc = プロフィールを閲覧できるユーザー

# semantic i18n key (#2542)
settings-contacts-only = 連絡先のみ

# semantic i18n key (#2542)
settings-online-status = オンライン状態

# semantic i18n key (#2542)
settings-online-status-desc = オンライン時に表示する

# semantic i18n key (#2542)
settings-read-receipts = 既読通知

# semantic i18n key (#2542)
settings-read-receipts-desc = メッセージを読んだことを他のユーザーに知らせる

# semantic i18n key (#2542)
settings-your-data = あなたのデータ

# semantic i18n key (#2542)
settings-export-data = データをエクスポート

# semantic i18n key (#2542)
settings-export-data-desc = すべてのデータをポータブル形式でダウンロード

# semantic i18n key (#2542)
settings-delete-account-title = アカウントと関連するすべてのデータを完全に削除します

# semantic i18n key (#2542)
settings-delete-my-account = アカウントを削除

# semantic i18n key (#2542)
settings-memory-subtitle = 会話をまたいでアシスタントがあなたについて覚えていること

# semantic i18n key (#2542)
settings-remembered-facts = 記憶された事実

# semantic i18n key (#2542)
settings-memory-facts-desc = これらの事実は各メッセージとともに送信され、会話間で回答の一貫性が保たれます。アシスタントに使用してほしくないものは削除してください。

# semantic i18n key (#2542)
settings-memory-all-scopes = すべてのスコープ

# semantic i18n key (#2542)
settings-memory-global-all-bots = グローバル（すべてのボット）

# semantic i18n key (#2542)
settings-memory-bot-specific = ボット固有

# semantic i18n key (#2542)
settings-default-scope = デフォルト

# semantic i18n key (#2542)
settings-manage-memories = 記憶を管理

# semantic i18n key (#2542)
settings-manage-memories-desc = 記憶はアシスタントによって作成されます。ここで編集または削除できます。

# semantic i18n key (#2542)
settings-no-plan-info = このデプロイメントではプラン情報は管理されていません。

# semantic i18n key (#2542)
settings-manage-plan-on-cloud = クラウドでプランを管理

# semantic i18n key (#2542)
settings-add-method = 方法を追加

# semantic i18n key (#2542)
settings-no-payment-method = 登録されている支払い方法はありません。

# semantic i18n key (#2542)
settings-add-method-on-cloud = クラウドで方法を追加

# semantic i18n key (#2542)
settings-no-invoices = このアカウントの請求書はありません。

# semantic i18n key (#2542)
settings-view-invoices-on-cloud = クラウドで請求書を表示

# semantic i18n key (#2542)
settings-skills-placeholder =
    SKILL.md パッケージを貼り付け:
    ---
    name: deploy-check
    description: 公開前にデプロイを確認する
    ---
    1. …
settings-memory-search-placeholder = メモリを検索…
settings-notification-recipient-placeholder = you@example.com

# semantic i18n key (#2542)
settings-profile-placeholder-display-name = 山田 太郎
settings-profile-placeholder-username = yamada
settings-profile-placeholder-email = taro@example.com
settings-profile-placeholder-public-display-name = 表示名
settings-profile-placeholder-public-bio = 公開プロフィールに表示される短い説明...
settings-profile-placeholder-phone = +81 90-0000-0000
settings-profile-placeholder-location = 市区町村、国
settings-profile-placeholder-website = https://example.com

# semantic i18n key (#2542)
settings-placeholder-dns-hostname = mybot.example.com
settings-placeholder-dns-target = 192.168.1.1 または target.domain.com
settings-placeholder-api-key-name = 本番キー
settings-placeholder-api-key-scopes = user, admin
settings-placeholder-api-key-expiry = 90
settings-placeholder-webhook-url = https://example.com/hooks/gb
settings-placeholder-webhook-events = user.created, order.completed
os-update-channel = Channel
os-update-size = Download size
os-update-usb = Install from USB stick
os-update-slot = Active slot: {slot} · channel: {channel} · rollback: {rollback}
