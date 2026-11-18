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
