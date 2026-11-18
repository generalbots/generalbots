# Transactional email text (#2094). Subject and body only; MIME and envelope stay untouched.

email-greeting = { $name },您好:
email-signoff = 此致敬礼,
email-team = General Bots 团队
email-welcome-subject = 欢迎使用 General Bots Cloud — { $plan } 方案
email-welcome-body = 欢迎使用 General Bots Cloud!您的 { $plan } 方案已生效。您可以访问 { $url } 开始创建机器人并管理工作区。
email-invoice-subject = 发票 #{ $invoice } — General Bots Cloud
email-invoice-body = 已生成新的发票。发票:{ $invoice }。金额:{ $amount } { $currency }。详情请访问 { $url } 查看。
email-payment-success-subject = 付款已确认 — General Bots Cloud
email-payment-success-body = 您支付 { $amount } { $currency } 的款项已确认。方案:{ $plan }。感谢您的订阅!
email-subscription-activated-subject = 订阅已激活 — General Bots Cloud
email-subscription-activated-body = 您的 { $plan } 订阅已生效。您可以访问 { $url } 进行管理。
email-recurring-charge-subject = 定期扣款 — 发票 #{ $invoice }
email-recurring-charge-body = 您的定期订阅已完成扣款。发票:{ $invoice }。金额:{ $amount } { $currency }。详情请访问 { $url } 查看。
email-payment-failed-subject = 付款失败 — 需要处理
email-payment-failed-body = 您支付 { $amount } { $currency } 的款项失败。发票:{ $invoice }。请在 { $url } 更新您的付款方式。
email-subscription-cancelled-subject = 订阅已取消 — General Bots Cloud
email-subscription-cancelled-body = 您的 { $plan } 订阅已取消。如需重新激活,请访问 { $url }。
email-grace-warning-subject = 付款逾期 — 服务暂停警告
email-grace-warning-body = 您的付款已逾期。服务将在 { $days } 天后暂停。请立即在 { $url } 更新付款方式。
email-invitation-subject = 邀请加入 { $org }
email-invitation-body = 您已被邀请加入 { $org }。请在 { $url } 接受邀请。此邀请将于 { $expires } 过期。
email-quota-limit-reached = 您已达到 { $metric } 配额上限。
email-quota-threshold = 您已使用 { $metric } 配额的 { $current }%(阈值:{ $threshold }%)。
email-metric-messages = 消息
email-metric-storage = 存储
email-metric-api-calls = API 调用
email-metric-bots = 机器人
email-metric-users = 用户
email-metric-kb-documents = 知识库文档
email-metric-apps = 应用
email-metric-emails-per-day = 每日邮件
email-metric-builds-per-day = 每日构建
email-metric-uploads-per-hour = 每小时上传

# Sharing and mention notifications (#2094)
email-share-subject = { $sender } 与您共享了 "{ $file }"
email-share-download = 下载 { $file }
email-share-link-expires = 此链接将于 { $date } 过期。您可以随时撤销。
email-share-link-no-expiry = 此链接不会过期。您可以随时撤销。
email-mention-subject = { $author } 在评论中提到了您
email-mention-body = { $author } 在 { $resource_type }({ $resource_id })中提到了您。“{ $preview }”请打开应用查看并回复。

# semantic i18n key (#2542)
mail-new-message = 新邮件

# semantic i18n key (#2542)
mail-cc-bcc = 抄送/密送

# semantic i18n key (#2542)
mail-subject-label = 主题：

# semantic i18n key (#2542)
mail-track-opens = 跟踪打开

# semantic i18n key (#2542)
mail-request-read-receipt = 请求已读回执

# semantic i18n key (#2542)
mail-priority-normal = 普通优先级

# semantic i18n key (#2542)
mail-priority-high = 高优先级

# semantic i18n key (#2542)
mail-priority-low = 低优先级

# semantic i18n key (#2542)
mail-dialog-templates-title = 邮件模板

# semantic i18n key (#2542)
mail-dialog-new-template = + 新建模板

# semantic i18n key (#2542)
mail-dialog-signatures-title = 邮件签名

# semantic i18n key (#2542)
mail-dialog-new-signature = + 新建签名

# semantic i18n key (#2542)
mail-dialog-rules-title = 邮件规则

# semantic i18n key (#2542)
mail-dialog-new-rule = + 新建规则

# semantic i18n key (#2542)
mail-dialog-automatic-replies-title = 自动答复

# semantic i18n key (#2542)
mail-dialog-send-automatic-replies = 发送自动答复

# semantic i18n key (#2542)
mail-dialog-start-date = 开始日期

# semantic i18n key (#2542)
mail-dialog-end-date = 结束日期

# semantic i18n key (#2542)
mail-dialog-reply-only-org = 仅发送给组织内的人员

# semantic i18n key (#2542)
mail-dialog-add-account-title = 添加电子邮件帐户

# semantic i18n key (#2542)
mail-dialog-account-name = 帐户名称

# semantic i18n key (#2542)
mail-dialog-imap-server = IMAP 服务器

# semantic i18n key (#2542)
mail-dialog-imap-port = IMAP 端口

# semantic i18n key (#2542)
mail-dialog-smtp-server = SMTP 服务器

# semantic i18n key (#2542)
mail-dialog-smtp-port = SMTP 端口

# semantic i18n key (#2542)
mail-dialog-use-ssl-tls = 使用 SSL/TLS

# semantic i18n key (#2542)
mail-dialog-add-account-action = 添加帐户

# semantic i18n key (#2542)
mail-dialog-schedule-send-title = 定时发送

# semantic i18n key (#2542)
mail-dialog-schedule-action = 计划

# semantic i18n key (#2542)
mail-manage-labels-title = 管理标签

# semantic i18n key (#2542)
mail-label-name = 标签名称

# semantic i18n key (#2542)
mail-label-color = 颜色

# semantic i18n key (#2542)
mail-new-signature-title = 新建电子邮件签名

# semantic i18n key (#2542)
mail-signature-name = 签名名称

# semantic i18n key (#2542)
mail-html-content = HTML 内容

# semantic i18n key (#2542)
mail-new-rule-title = 新建邮件规则

# semantic i18n key (#2542)
mail-rule-name = 规则名称

# semantic i18n key (#2542)
mail-condition-sender-contains = 条件（发件人包含）

# semantic i18n key (#2542)
mail-action-star = 加星标

# semantic i18n key (#2542)
mail-new-template-title = 新建邮件模板

# semantic i18n key (#2542)
mail-template-name = 模板名称

# semantic i18n key (#2542)
mail-body-html = 正文 HTML

# semantic i18n key (#2542)
mail-placeholder-label-name = 例如：工作
mail-placeholder-signature-name = 例如：专业签名
mail-placeholder-signature-content = 此致敬礼，<br><b>您的姓名</b>

# semantic i18n key (#2542)
mail-placeholder-rule-name = 例如：归档新闻通讯
mail-placeholder-rule-condition = 例如：newsletter
mail-placeholder-template-name = 例如：欢迎邮件
mail-placeholder-template-subject = 标准欢迎主题
mail-placeholder-template-body = 您好 {{name}}，欢迎来到我们的平台！
mail-placeholder-autoreply-subject = 回复主题
mail-placeholder-autoreply-message = 您的自动回复消息...

# semantic i18n key (#2542)
mail-placeholder-search-emails = 搜索邮件...
mail-placeholder-recipients = 收件人
mail-placeholder-carbon-copy = 抄送
mail-placeholder-blind-carbon-copy = 密送
mail-placeholder-subject = 主题
mail-placeholder-compose-message = 撰写邮件...
mail-placeholder-account-name = 工作、个人等

# semantic i18n key (#2542)
mail-placeholder-imap-server = imap.example.com
mail-placeholder-smtp-server = smtp.example.com
mail-placeholder-username-or-email = 用户名或邮箱
mail-placeholder-password = ••••••••
