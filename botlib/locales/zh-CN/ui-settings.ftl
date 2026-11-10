# =============================================================================
# General Bots — zh-CN — Settings
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Settings
# -----------------------------------------------------------------------------
settings-title = 设置
settings-general = 一般
settings-account = 账户
settings-notifications = 通知
settings-privacy = 隐私
settings-security = 安全设置
settings-language = 语言
settings-theme = 主题
settings-theme-light = 光
settings-theme-dark = 黑暗
settings-theme-system = 系统
settings-save = 保存更改
settings-saved = 设置保存成功
settings-create-api-key = 创建 API 密钥
settings-api-key-name = 按键名称
settings-api-key-scopes = 范围（以逗号分隔）
settings-api-key-scopes-hint = 对于默认的“用户”范围，将其留空。
settings-api-key-expiry = 到期时间（天）
settings-api-key-expiry-hint = 留空则不会过期。
settings-add-webhook = 添加网络钩子
settings-webhook-url = 网络钩子 URL
settings-webhook-events = 事件（以逗号分隔）
settings-webhook-events-hint = 留空以接收所有事件。
settings-timezone = 时区
settings-date-format = 日期格式
settings-time-format = 时间格式

# -----------------------------------------------------------------------------
# Settings - Language & Localization
# -----------------------------------------------------------------------------
settings-language-desc = 选择您的首选语言
settings-display-language = 显示语言
settings-language-affects = 影响应用程序中的所有文本
settings-date-format-desc = 日期如何显示
settings-time-format-desc = 12 小时制或 24 小时制
settings-language-changed = 语言更改成功
settings-reload-required = 需要重新加载页面才能应用更改

# Settings - Profile
settings-profile-desc = 管理您的个人信息和偏好
settings-profile-photo = 个人资料照片
settings-profile-photo-desc = 其他用户可以看到您的个人资料照片
settings-upload-photo = 上传照片
settings-remove-photo = 删除
settings-basic-info = 基本信息
settings-display-name = 显示名称
settings-username = 用户名
settings-email-address = 电子邮件地址
settings-bio = 简介
settings-bio-placeholder = 告诉我们你自己...
settings-contact-info = 联系方式
settings-phone-number = 电话号码
settings-location = 地点
settings-website = 网站

# Settings - Security
settings-security-desc = 通过增强的安全性保护您的帐户
settings-change-password = 更改密码
settings-change-password-desc = 定期更新您的密码以提高安全性
settings-current-password = 当前密码
settings-new-password = 新密码
settings-confirm-password = 确认新密码
settings-update-password = 更新密码
settings-2fa = 双因素身份验证
settings-2fa-desc = 为您的帐户添加额外的安全层
settings-authenticator-app = 验证器应用程序
settings-authenticator-desc = 使用验证器应用程序获取 2FA 代码
settings-enable-2fa = 启用 2FA
settings-disable-2fa = 禁用 2FA
settings-active-sessions = 活跃会话
settings-active-sessions-desc = 管理您的活动登录会话
settings-this-device = 这个装置
settings-terminate-session = 终止
settings-terminate-all = 终止所有其他会话

# Settings - Appearance
settings-appearance = 外观
settings-appearance-desc = 自定义应用程序的外观
settings-theme-selection = 主题
settings-theme-selection-desc = 选择您喜欢的颜色主题
settings-theme-blue = 蓝色
settings-theme-purple = 紫色
settings-theme-green = 绿色
settings-theme-orange = 橙色
settings-layout-preferences = 布局首选项
settings-compact-mode = 紧凑模式
settings-compact-mode-desc = 减少间距以显示更多内容
settings-show-sidebar = 显示侧边栏
settings-show-sidebar-desc = 始终显示导航侧边栏
settings-animations = 动画
settings-animations-desc = 启用 UI 动画和过渡

# Settings - Notifications
settings-notifications-title = 通知
settings-notifications-desc = 控制您接收通知的方式
settings-email-notifications = 电子邮件通知
settings-direct-messages = 直接消息
settings-direct-messages-desc = 接收新私信的电子邮件
settings-mentions = 提及
settings-mentions-desc = 当有人提及您时收到电子邮件
settings-weekly-digest = 每周文摘
settings-weekly-digest-desc = 获取每周活动摘要
settings-marketing = 营销
settings-marketing-desc = 接收新闻和产品更新
settings-push-notifications = 推送通知
settings-enable-push = 启用推送通知
settings-enable-push-desc = 接收浏览器推送通知
settings-notification-sound = 声音
settings-notification-sound-desc = 播放通知声音
settings-in-app-notifications = 应用内通知

# Settings - Storage
settings-storage = 存储
settings-storage-desc = 管理您的存储使用情况
settings-storage-usage = 存储使用情况
settings-storage-used = 已使用 { $total } 中的{ $used }
settings-storage-upgrade = 升级存储

# Settings - Privacy
settings-privacy-title = 隐私
settings-privacy-desc = 控制您的隐私设置
settings-data-collection = 数据收集
settings-analytics = 分析
settings-analytics-desc = 通过发送匿名使用数据帮助我们改进
settings-crash-reports = 崩溃报告
settings-crash-reports-desc = 自动发送崩溃报告
settings-download-data = 下载您的数据
settings-download-data-desc = 获取所有数据的副本
settings-delete-account = 删除帐户
settings-delete-account-desc = 永久删除您的帐户和所有数据
settings-delete-account-warning = 此操作无法撤消

# Settings - Billing
settings-billing = 计费
settings-billing-desc = 管理您的订阅和付款方式
settings-current-plan = 当前计划
settings-free-plan = 免费计划
settings-pro-plan = 专业计划
settings-enterprise-plan = 企业计划
settings-upgrade-plan = 升级计划
settings-payment-methods = 付款方式
settings-add-payment = 添加付款方式
settings-billing-history = 账单历史记录

# -----------------------------------------------------------------------------
# OAuth Account Linking (Settings)
# -----------------------------------------------------------------------------
oauth-connected-accounts = 关联账户
oauth-connect = 连接
oauth-unlink = 取消链接
oauth-not-connected = 未连接
oauth-linked = 链接
oauth-no-accounts = 尚未关联任何帐户。
oauth-loading = 正在加载关联帐户...

## Payment cards (Stripe SetupIntent)
cards-title = 支付与卡
cards-saved = 已保存的卡片
cards-hint = 卡由我们的支付提供商安全存储。卡号永远不会到达我们的服务器。
cards-add = 添加卡
cards-add-first = 添加您的第一张卡
cards-none = 尚未保存卡片
cards-empty-hint = 添加卡以启用自动计费和更快的结账速度。您将被重定向到我们的安全支付提供商以输入您的卡详细信息。
cards-default = 默认
cards-set-default = 设置默认值
cards-default-btn = 默认卡
cards-remove = 删除
cards-remove-confirm = 删除这张卡？
cards-expires = 过期
cards-load-error = 无法加载已保存的卡片。
cards-add-error = 无法添加卡
cards-default-error = 无法更新默认值
cards-remove-error = 无法移除卡
cards-default-updated = 默认卡已更新
cards-removed = 卡已移除
