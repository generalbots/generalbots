notification-title-new-message = 新消息
notification-title-task-due = 任务到期
notification-title-task-assigned = 分配的任务
notification-title-task-completed = 任务完成
notification-title-meeting-reminder = 会议提醒
notification-title-meeting-started = 会议开始
notification-title-file-shared = 文件共享
notification-title-file-uploaded = 文件已上传
notification-title-comment-added = 新评论
notification-title-mention = 你被提到了
notification-title-system = 系统通知
notification-title-security = 安全警报
notification-title-update = 可用更新
notification-title-error = 发生错误
notification-title-success = 成功
notification-title-warning = 警告
notification-title-info = 信息

notification-message-new = 您有一条来自 { $sender } 的新消息
notification-message-unread = 你有 { $count ->
    [one] { $count } unread message
   *[other] { $count } unread messages
}
notification-task-due-soon = 任务“{ $task }”截止日期为{ $time }
notification-task-due-today = 任务“{ $task }”今天截止
notification-task-due-overdue = 任务“{ $task }”逾期{ $time }
notification-task-assigned-to-you = 您已被分配到任务“{ $task }”
notification-task-assigned-by = { $assigner } 已将您分配给“{ $task }”
notification-task-completed-by = { $user } 完成任务“{ $task }”
notification-task-status-changed = 任务“{ $task }”状态更改为{ $status }

notification-meeting-in-minutes = 会议“{ $meeting }”将在 { $minutes } 分钟后开始
notification-meeting-starting-now = 会议“{ $meeting }”现在开始
notification-meeting-cancelled = 会议“{ $meeting }”已取消
notification-meeting-rescheduled = 会议“{ $meeting }”已改期至{ $datetime }
notification-meeting-invite = { $inviter }邀请你参加“{ $meeting }”
notification-meeting-response = { $user } { $response } 您的会议邀请

notification-file-shared-with-you = { $sharer } 与您分享了“{ $filename }”
notification-file-uploaded-by = { $uploader } 上传了“{ $filename }”
notification-file-modified = “{ $filename }”被{ $user }修改
notification-file-deleted = “{ $filename }”已被{ $user }删除
notification-file-download-ready = 您的文件“{ $filename }”已准备好下载
notification-file-upload-complete = “{ $filename }”上传成功
notification-file-upload-failed = “{ $filename }”上传失败

notification-comment-on-task = { $user }评论了任务“{ $task }”
notification-comment-on-file = { $user }评论了“{ $filename }”
notification-comment-reply = { $user }回复了您的评论
notification-mention-in-comment = { $user } 在评论中提到了你
notification-mention-in-chat = { $user }在{ $channel }提到过你

notification-login-new-device = 在 { $location } 中检测到来自 { $device } 的新登录
notification-login-failed = 您的帐户尝试登录失败
notification-password-changed = 您的密码已成功更改
notification-password-expiring = 您的密码将在 { $days } 天后过期
notification-session-expired = 您的会话已过期
notification-account-locked = 您的帐户已被锁定
notification-two-factor-enabled = 已启用双因素身份验证
notification-two-factor-disabled = 双因素身份验证已被禁用

notification-subscription-expiring = 您的订阅将在 { $days } 天后到期
notification-subscription-expired = 您的订阅已过期
notification-subscription-renewed = 您的订阅已续订至 { $date }
notification-payment-successful = { $amount }支付成功
notification-payment-failed = { $amount }支付失败
notification-invoice-ready = 您的{ $period }发票已准备好

notification-bot-response = { $bot }回复了您的询问
notification-bot-error = { $bot }遇到错误
notification-bot-offline = { $bot }目前离线
notification-bot-online = { $bot }现已上线
notification-bot-updated = { $bot }已更新

notification-system-maintenance = 系统维护预计{ $datetime }
notification-system-update = 系统更新可用：{ $version }
notification-system-restored = 系统已恢复
notification-system-degraded = 系统性能下降

notification-action-view = 查看
notification-action-dismiss = 解雇
notification-action-mark-read = 标记为已读
notification-action-mark-all-read = 全部标记为已读
notification-action-settings = 通知设置
notification-action-reply = 回复
notification-action-open = 打开
notification-action-join = 加入
notification-action-accept = 接受
notification-action-decline = 拒绝

notification-time-just-now = 刚才
notification-time-minutes = { $计数 ->
    [one] { $count } minute ago
   *[other] { $count } minutes ago
}
notification-time-hours = { $计数 ->
    [one] { $count } hour ago
   *[other] { $count } hours ago
}
notification-time-days = { $计数 ->
    [one] { $count } day ago
   *[other] { $count } days ago
}
notification-time-weeks = { $计数 ->
    [one] { $count } week ago
   *[other] { $count } weeks ago
}

notification-preference-all = 所有通知
notification-preference-important = 仅重要
notification-preference-none = 无
notification-preference-email = 电子邮件通知
notification-preference-push = 推送通知
notification-preference-in-app = 应用内通知
notification-preference-sound = 声音已启用
notification-preference-vibration = 启用振动

notification-empty = 无通知
notification-empty-description = 你们都被抓住了！
notification-load-more = 加载更多
notification-clear-all = 清除所有通知
notification-filter-all = 全部
notification-filter-unread = 未读
notification-filter-mentions = 提及
notification-filter-tasks = 任务
notification-filter-messages = 留言
notification-filter-system = 系统
