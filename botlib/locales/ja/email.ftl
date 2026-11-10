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
