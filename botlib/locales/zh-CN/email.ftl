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
