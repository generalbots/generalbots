bot-greeting-default = こんにちは！今日はどのようにお手伝いできますか?
bot-greeting-named = { $name }さん、こんにちは！今日はどのようにお手伝いできますか?
bot-goodbye = さようなら！すてきな一日を！
bot-help-prompt = { $topics } についてお手伝いいたします。何を知りたいですか?
bot-thank-you = メッセージありがとうございます。今日はどのようにお手伝いできますか?
bot-echo-intro = エコーボット: あなたの言うことをすべて繰り返します。 「quit」と入力して終了します。
bot-you-said = あなたはこう言いました: { $message }
bot-thinking = それについて考えさせてください...
bot-processing = リクエストを処理しています...
bot-error-occurred = 申し訳ありませんが、問題が発生しました。もう一度試してください。
bot-not-understood = それはわかりませんでした。言い換えていただけますか？
bot-confirm-action = 続行してもよろしいですか?
bot-action-cancelled = アクションがキャンセルされました。
bot-action-completed = 終わり！

bot-lead-welcome = いらっしゃいませ！始めるお手伝いをさせてください。
bot-lead-ask-name = あなたの名前は何ですか？
bot-lead-ask-email = そしてあなたのメールアドレスは？
bot-lead-ask-company = どこの会社の出身ですか？
bot-lead-ask-phone = 電話番号は何ですか?
bot-lead-hot = 素晴らしい！弊社の営業チームがすぐにご連絡いたします。
bot-lead-nurture = ご興味をお持ちいただきありがとうございます。いくつかのリソースをお送りします。
bot-lead-score = あなたのリードスコアは 100 点中 { $score } です。
bot-lead-saved = あなたの情報は正常に保存されました。

bot-schedule-created = スケジュールされたタスクを実行中: { $name }
bot-schedule-next = 次回の実行は{ $datetime }に予定されています
bot-schedule-cancelled = スケジュールがキャンセルされました。
bot-schedule-paused = スケジュールが一時停止されました。
bot-schedule-resumed = スケジュールが再開されました。

bot-monitor-alert = 警告: { $subject } が変更されました
bot-monitor-threshold = { $metric } がしきい値を超えました: { $value }
bot-monitor-recovered = { $subject }は正常に戻りました。
bot-monitor-status = 現在のステータス: { $status }

bot-order-welcome = 当店へようこそ！どうすれば助けられますか?
bot-order-track = 注文を追跡する
bot-order-browse = 製品を閲覧する
bot-order-support = サポートに連絡する
bot-order-enter-id = 注文番号を入力してください:
bot-order-status = 注文状況: { $status }
bot-order-shipped = ご注文は発送されました！追跡番号: { $tracking }
bot-order-delivered = ご注文は配達されました。
bot-order-processing = ご注文は処理中です。
bot-order-cancelled = ご注文はキャンセルされました。
bot-order-ticket = サポート チケットが作成されました: #{ $ticket }
bot-order-products-available = 利用可能な製品は次のとおりです。
bot-order-product-item = { $name } - { $price }
bot-order-cart-added = { $product } をカートに追加しました。
bot-order-cart-total = カートの合計は { $total } です。
bot-order-checkout = チェックアウトに進んでいます...

bot-hr-welcome = 人事アシスタントはこちら。どうすれば助けられますか?
bot-hr-request-leave = 休暇を申請する
bot-hr-check-balance = 残高を確認する
bot-hr-view-policies = ポリシーを表示する
bot-hr-leave-type = どのような種類の休暇ですか? (休暇/病気/個人)
bot-hr-start-date = 開始日? (YYYY-MM-DD)
bot-hr-end-date = 終了日? (YYYY-MM-DD)
bot-hr-leave-submitted = 休暇リクエストを送信しました!あなたのマネージャーがそれを検討します。
bot-hr-leave-approved = あなたの休暇申請は承認されました。
bot-hr-leave-rejected = あなたの休暇申請は拒否されました。
bot-hr-leave-pending = 休暇申請は承認待ちです。
bot-hr-balance-title = 休暇残高:
bot-hr-vacation-days = 休暇: { $days } 日
bot-hr-sick-days = 病気: { $days } 日
bot-hr-personal-days = 個人: { $days } 日
bot-hr-policy-found = リクエストしたポリシー情報は次のとおりです。
bot-hr-policy-not-found = ポリシーが見つかりません。ポリシー名を確認してください。

bot-health-welcome = 私たちのヘルスケアセンターへようこそ。どうすれば助けられますか?
bot-health-book = 予約する
bot-health-cancel = 予約をキャンセルする
bot-health-view = 予定を表示する
bot-health-reschedule = 予定を変更する
bot-health-type = どのような種類の予定ですか? (一般/専門/研究室)
bot-health-doctor = どちらの医師をご希望ですか?
bot-health-date = あなたにとって最適な日付は何ですか?
bot-health-time = 何時頃をご希望ですか？
bot-health-confirmed = { $datetime }に{ $doctor }にご予約が確定しました。
bot-health-cancelled = あなたの予定はキャンセルされました。
bot-health-rescheduled = ご予約は{ $datetime }に変更されました。
bot-health-reminder = リマインダー: { $datetime } に予定があります。
bot-health-no-appointments = 今後の予定はありません。
bot-health-appointments-list = 今後の予定:

bot-support-welcome = 今日はどのようにお手伝いできますか?
bot-support-describe = 問題について説明してください:
bot-support-category = あなたの問題を最もよく表すカテゴリは何ですか?
bot-support-priority = この問題はどれくらい緊急ですか?
bot-support-ticket-created = サポートチケット #{ $ticket } が作成されました。
bot-support-ticket-status = チケット番号{ $ticket } ステータス: { $status }
bot-support-ticket-updated = チケットが更新されました。
bot-support-ticket-resolved = あなたのチケットは解決されました。さらにサポートが必要な場合はお知らせください。
bot-support-transfer = 人間のエージェントに転送しています...
bot-support-wait-time = 推定待ち時間: { $minutes } 分。
bot-support-agent-joined = エージェント { $name } が会話に参加しました。

bot-survey-intro = ぜひご意見をお聞かせください。
bot-survey-question = { $question }
bot-survey-scale = 1 から 10 のスケールで、{ $subject } をどのように評価しますか?
bot-survey-open = 追加のコメントがあればお知らせください:
bot-survey-thanks = フィードバックをいただきありがとうございます。
bot-survey-completed = 調査は正常に完了しました。
bot-survey-skip = 必要に応じて、この質問をスキップできます。

bot-notification-new-message = { $sender } から新しいメッセージが届きました。
bot-notification-task-due = タスク「{ $task }」の期限は{ $when }です。
bot-notification-reminder = リマインダー: { $message }
bot-notification-update = 更新: { $message }
bot-notification-alert = 警告: { $message }

bot-command-help = 利用可能なコマンド:
bot-command-unknown = 不明なコマンドです。使用可能なコマンドについては「help」と入力してください。
bot-command-invalid = 無効なコマンド構文です。使用法: { $usage }

bot-transfer-to-human = 人間のエージェントに転送します。お待ちください...
bot-transfer-complete = { $agent } と接続されました。
bot-transfer-unavailable = 現在対応可能なエージェントはいません。後でもう一度試してください。
bot-transfer-queue-position = あなたは列の番号{ $position }です。

bot-auth-login-prompt = 続行するには資格情報を入力してください。
bot-auth-login-success = 正常にログインされました。
bot-auth-login-failed = ログインに失敗しました。資格情報を確認してください。
bot-auth-logout-success = ログアウトされました。
bot-auth-session-expired = セッションの有効期限が切れました。再度ログインしてください。

bot-file-upload-prompt = ファイルをアップロードしてください。
bot-file-upload-success = ファイル「{ $filename }」は正常にアップロードされました。
bot-file-upload-failed = ファイルのアップロードに失敗しました。もう一度試してください。
bot-file-download-ready = ファイルをダウンロードする準備ができました。
bot-file-processing = ファイルを処理しています...

bot-payment-amount = 合計金額は{ $amount }です。
bot-payment-method = お支払い方法を選択してください。
bot-payment-processing = お支払いを処理しています...
bot-payment-success = 支払いが成功しました！トランザクションID: { $transactionId }
bot-payment-failed = 支払いに失敗しました。もう一度お試しいただくか、別のお支払い方法をご利用ください。
bot-payment-refund = { $amount } の返金が処理されました。

bot-subscription-active = あなたのサブスクリプションは { $endDate } まで有効です。
bot-subscription-expired = サブスクリプションの有効期限が切れました。
bot-subscription-renew = サブスクリプションを更新しますか?
bot-subscription-upgraded = あなたのサブスクリプションは { $plan } にアップグレードされました。
bot-subscription-cancelled = サブスクリプションはキャンセルされました。

bot-feedback-positive = 肯定的なフィードバックをありがとうございます!
bot-feedback-negative = 申し訳ございません。どうすれば改善できるでしょうか?
bot-feedback-rating = あなたはこのインタラクションを 5 点満点中 { $rating } と評価しました。
