# Transactional email text (#2094). Subject and body only; MIME and envelope stay untouched.

email-greeting = { $name } 様
email-signoff = どうぞよろしくお願いいたします。
email-team = General Bots チーム
email-welcome-subject = General Bots Cloud へようこそ — { $plan } プラン
email-welcome-body = General Bots Cloud へようこそ!{ $plan } プランが有効になりました。{ $url } でボットの作成とワークスペースの管理を開始できます。
email-invoice-subject = 請求書 #{ $invoice } — General Bots Cloud
email-invoice-body = 新しい請求書が作成されました。請求書: { $invoice }。金額: { $amount } { $currency }。詳細は { $url } でご確認ください。
email-payment-success-subject = お支払いが確認されました — General Bots Cloud
email-payment-success-body = { $amount } { $currency } のお支払いが確認されました。プラン: { $plan }。ご登録ありがとうございます。
email-subscription-activated-subject = サブスクリプションが有効になりました — General Bots Cloud
email-subscription-activated-body = { $plan } のサブスクリプションが有効になりました。{ $url } で管理できます。
email-recurring-charge-subject = 定期請求 — 請求書 #{ $invoice }
email-recurring-charge-body = 定期サブスクリプションの請求が行われました。請求書: { $invoice }。金額: { $amount } { $currency }。詳細は { $url } でご確認ください。
email-payment-failed-subject = お支払いに失敗しました — 対応が必要です
email-payment-failed-body = { $amount } { $currency } のお支払いに失敗しました。請求書: { $invoice }。{ $url } でお支払い方法を更新してください。
email-subscription-cancelled-subject = サブスクリプションが解約されました — General Bots Cloud
email-subscription-cancelled-body = { $plan } のサブスクリプションが解約されました。再開する場合は { $url } にアクセスしてください。
email-grace-warning-subject = お支払いの遅延 — サービス停止の予告
email-grace-warning-body = お支払いが遅延しています。{ $days } 日後にサービスが停止されます。至急 { $url } でお支払い方法を更新してください。
email-invitation-subject = { $org } への招待
email-invitation-body = { $org } への参加に招待されました。{ $url } で招待を承認してください。この招待は { $expires } に失効します。
email-quota-limit-reached = { $metric } のクォータ上限に達しました。
email-quota-threshold = { $metric } のクォータの { $current }% を使用しました(しきい値: { $threshold }%)。
email-metric-messages = メッセージ
email-metric-storage = ストレージ
email-metric-api-calls = API 呼び出し
email-metric-bots = ボット
email-metric-users = ユーザー
email-metric-kb-documents = ナレッジベースのドキュメント
email-metric-apps = アプリ
email-metric-emails-per-day = 1 日あたりのメール
email-metric-builds-per-day = 1 日あたりのビルド
email-metric-uploads-per-hour = 1 時間あたりのアップロード

# Sharing and mention notifications (#2094)
email-share-subject = { $sender } が「{ $file }」をあなたと共有しました
email-share-download = { $file } をダウンロード
email-share-link-expires = このリンクは { $date } に失効します。いつでも無効にできます。
email-share-link-no-expiry = このリンクは失効しません。いつでも無効にできます。
email-mention-subject = { $author } がコメントであなたをメンションしました
email-mention-body = { $author } が { $resource_type } ({ $resource_id }) であなたをメンションしました。「{ $preview }」アプリを開いて確認し、返信してください。

# semantic i18n key (#2542)
mail-new-message = 新規メッセージ

# semantic i18n key (#2542)
mail-cc-bcc = Cc/Bcc

# semantic i18n key (#2542)
mail-subject-label = 件名:

# semantic i18n key (#2542)
mail-track-opens = 開封を追跡

# semantic i18n key (#2542)
mail-request-read-receipt = 開封確認を要求

# semantic i18n key (#2542)
mail-priority-normal = 通常の優先度

# semantic i18n key (#2542)
mail-priority-high = 高優先度

# semantic i18n key (#2542)
mail-priority-low = 低優先度

# semantic i18n key (#2542)
mail-dialog-templates-title = メールテンプレート

# semantic i18n key (#2542)
mail-dialog-new-template = + 新しいテンプレート

# semantic i18n key (#2542)
mail-dialog-signatures-title = メール署名

# semantic i18n key (#2542)
mail-dialog-new-signature = + 新しい署名

# semantic i18n key (#2542)
mail-dialog-rules-title = メールルール

# semantic i18n key (#2542)
mail-dialog-new-rule = + 新しいルール

# semantic i18n key (#2542)
mail-dialog-automatic-replies-title = 自動返信

# semantic i18n key (#2542)
mail-dialog-send-automatic-replies = 自動返信を送信する

# semantic i18n key (#2542)
mail-dialog-start-date = 開始日

# semantic i18n key (#2542)
mail-dialog-end-date = 終了日

# semantic i18n key (#2542)
mail-dialog-reply-only-org = 組織内のユーザーにのみ送信する

# semantic i18n key (#2542)
mail-dialog-add-account-title = メールアカウントを追加

# semantic i18n key (#2542)
mail-dialog-account-name = アカウント名

# semantic i18n key (#2542)
mail-dialog-imap-server = IMAP サーバー

# semantic i18n key (#2542)
mail-dialog-imap-port = IMAP ポート

# semantic i18n key (#2542)
mail-dialog-smtp-server = SMTP サーバー

# semantic i18n key (#2542)
mail-dialog-smtp-port = SMTP ポート

# semantic i18n key (#2542)
mail-dialog-use-ssl-tls = SSL/TLS を使用する

# semantic i18n key (#2542)
mail-dialog-add-account-action = アカウントを追加

# semantic i18n key (#2542)
mail-dialog-schedule-send-title = 送信を予約

# semantic i18n key (#2542)
mail-dialog-schedule-action = 予約

# semantic i18n key (#2542)
mail-manage-labels-title = ラベルを管理

# semantic i18n key (#2542)
mail-label-name = ラベル名

# semantic i18n key (#2542)
mail-label-color = 色

# semantic i18n key (#2542)
mail-new-signature-title = 新しいメール署名

# semantic i18n key (#2542)
mail-signature-name = 署名名

# semantic i18n key (#2542)
mail-html-content = HTML コンテンツ

# semantic i18n key (#2542)
mail-new-rule-title = 新しいメールルール

# semantic i18n key (#2542)
mail-rule-name = ルール名

# semantic i18n key (#2542)
mail-condition-sender-contains = 条件（送信者に含む）

# semantic i18n key (#2542)
mail-action-star = スターを付ける

# semantic i18n key (#2542)
mail-new-template-title = 新しいメールテンプレート

# semantic i18n key (#2542)
mail-template-name = テンプレート名

# semantic i18n key (#2542)
mail-body-html = 本文 HTML

# semantic i18n key (#2542)
mail-placeholder-label-name = 例: 仕事
mail-placeholder-signature-name = 例: ビジネス署名
mail-placeholder-signature-content = よろしくお願いいたします。<br><b>お名前</b>

# semantic i18n key (#2542)
mail-placeholder-rule-name = 例: ニュースレターをアーカイブ
mail-placeholder-rule-condition = 例: newsletter
mail-placeholder-template-name = 例: ウェルカムメール
mail-placeholder-template-subject = 標準のウェルカム件名
mail-placeholder-template-body = こんにちは {{name}} さん、プラットフォームへようこそ！
mail-placeholder-autoreply-subject = 返信件名
mail-placeholder-autoreply-message = 自動返信メッセージ...

# semantic i18n key (#2542)
mail-placeholder-search-emails = メールを検索...
mail-placeholder-recipients = 宛先
mail-placeholder-carbon-copy = CC
mail-placeholder-blind-carbon-copy = BCC
mail-placeholder-subject = 件名
mail-placeholder-compose-message = メッセージを入力...
mail-placeholder-account-name = 仕事、個人など
