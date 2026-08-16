notification-title-new-message = 新しいメッセージ
notification-title-task-due = タスクの期限
notification-title-task-assigned = 割り当てられたタスク
notification-title-task-completed = タスクが完了しました
notification-title-meeting-reminder = 会議リマインダー
notification-title-meeting-started = 会議が開始されました
notification-title-file-shared = ファイル共有
notification-title-file-uploaded = ファイルがアップロードされました
notification-title-comment-added = 新しいコメント
notification-title-mention = あなたが言及されました
notification-title-system = システム通知
notification-title-security = セキュリティ警告
notification-title-update = 利用可能なアップデート
notification-title-error = エラーが発生しました
notification-title-success = 成功
notification-title-warning = 警告
notification-title-info = 情報

notification-message-new = { $sender } から新しいメッセージが届きました
notification-message-unread = あなたは { $count -> を持っています
    [one] { $count } unread message
   *[other] { $count } unread messages
}
notification-task-due-soon = タスク「{ $task }」の期限は{ $time }です
notification-task-due-today = タスク「{ $task }」の期限は今日です
notification-task-due-overdue = タスク「{ $task }」は{ $time }までに期限が切れています
notification-task-assigned-to-you = あなたにはタスク「{ $task }」が割り当てられました
notification-task-assigned-by = { $assigner } はあなたを「{ $task }」に割り当てました
notification-task-completed-by = { $user } タスク「{ $task }」を完了しました
notification-task-status-changed = タスク「{ $task }」のステータスが{ $status }に変更されました

notification-meeting-in-minutes = 会議「{ $meeting }」あと { $minutes } 分で始まります
notification-meeting-starting-now = ミーティング「{ $meeting }」が始まります
notification-meeting-cancelled = ミーティング「{ $meeting }」はキャンセルされました
notification-meeting-rescheduled = ミーティング「{ $meeting }」は{ $datetime }に変更されました
notification-meeting-invite = { $inviter } さんが「{ $meeting }」に招待しました
notification-meeting-response = { $user } { $response } 会議への招待状

notification-file-shared-with-you = { $sharer } さんが「{ $filename }」をあなたと共有しました
notification-file-uploaded-by = { $uploader }「{ $filename }」をアップロードしました
notification-file-modified = 「{ $filename }」は{ $user } によって変更されました
notification-file-deleted = 「{ $filename }」は{ $user } によって削除されました
notification-file-download-ready = ファイル「{ $filename }」をダウンロードする準備ができました
notification-file-upload-complete = 「{ $filename }」のアップロードが正常に完了しました
notification-file-upload-failed = 「{ $filename }」のアップロードに失敗しました

notification-comment-on-task = { $user } がタスク「{ $task }」にコメントしました
notification-comment-on-file = { $user } さんが「{ $filename }」にコメントしました
notification-comment-reply = { $user } さんがコメントに返信しました
notification-mention-in-comment = { $user } さんがコメントであなたについて言及しました
notification-mention-in-chat = { $user } さんが { $channel } であなたについて言及しました

notification-login-new-device = { $location } の { $device } からの新しいログインが検出されました
notification-login-failed = アカウントへのログイン試行に失敗しました
notification-password-changed = パスワードは正常に変更されました
notification-password-expiring = パスワードの有効期限は { $days } 日後に切れます
notification-session-expired = セッションの有効期限が切れました
notification-account-locked = あなたのアカウントはロックされています
notification-two-factor-enabled = 二要素認証が有効になっています
notification-two-factor-disabled = 二要素認証が無効になっています

notification-subscription-expiring = サブスクリプションは { $days } 日後に期限切れになります
notification-subscription-expired = サブスクリプションの有効期限が切れました
notification-subscription-renewed = あなたのサブスクリプションは { $date } まで更新されました
notification-payment-successful = { $amount } の支払いが完了しました
notification-payment-failed = { $amount } の支払いに失敗しました
notification-invoice-ready = { $period } の請求書の準備ができました

notification-bot-response = { $bot } があなたのクエリに回答しました
notification-bot-error = { $bot } でエラーが発生しました
notification-bot-offline = { $bot } は現在オフラインです
notification-bot-online = { $bot } はオンラインになりました
notification-bot-updated = { $bot }を更新しました

notification-system-maintenance = システムメンテナンスは{ $datetime }に予定されています
notification-system-update = 利用可能なシステムアップデート: { $version }
notification-system-restored = システムが復旧しました
notification-system-degraded = システムのパフォーマンスが低下しています

notification-action-view = 見る
notification-action-dismiss = 解雇する
notification-action-mark-read = 既読としてマークする
notification-action-mark-all-read = すべて既読としてマークする
notification-action-settings = 通知設定
notification-action-reply = 返信
notification-action-open = 開く
notification-action-join = 参加する
notification-action-accept = 受け入れる
notification-action-decline = 辞退

notification-time-just-now = たった今
notification-time-minutes = { $count ->
    [one] { $count } minute ago
   *[other] { $count } minutes ago
}
notification-time-hours = { $count ->
    [one] { $count } hour ago
   *[other] { $count } hours ago
}
notification-time-days = { $count ->
    [one] { $count } day ago
   *[other] { $count } days ago
}
notification-time-weeks = { $count ->
    [one] { $count } week ago
   *[other] { $count } weeks ago
}

notification-preference-all = すべての通知
notification-preference-important = 重要のみ
notification-preference-none = なし
notification-preference-email = 電子メール通知
notification-preference-push = プッシュ通知
notification-preference-in-app = アプリ内通知
notification-preference-sound = サウンドを有効にする
notification-preference-vibration = 振動有効

notification-empty = 通知はありません
notification-empty-description = 皆さんも追い込まれていますね！
notification-load-more = さらにロードする
notification-clear-all = すべての通知をクリアする
notification-filter-all = すべて
notification-filter-unread = 未読
notification-filter-mentions = 言及
notification-filter-tasks = タスク
notification-filter-messages = メッセージ
notification-filter-system = システム
