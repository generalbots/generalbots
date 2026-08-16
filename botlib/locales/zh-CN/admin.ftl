# =============================================================================
# General Bots - Admin Translations (English)
# =============================================================================
# Administrative interface translations for the GB Admin Panel
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Navigation & Dashboard
# -----------------------------------------------------------------------------
admin-title = 行政管理
admin-dashboard = 管理仪表板
admin-overview = 概述
admin-welcome = 欢迎来到管理面板

admin-nav-dashboard = 仪表板
admin-nav-users = 用户
admin-nav-bots = 机器人
admin-nav-tenants = 租户
admin-nav-settings = 设置
admin-nav-logs = 日志
admin-nav-analytics = 分析
admin-nav-security = 安全性
admin-nav-integrations = 集成
admin-nav-billing = 计费
admin-nav-support = 支持
admin-nav-groups = 团体
admin-nav-dns = 域名系统
admin-nav-system = 系统

# -----------------------------------------------------------------------------
# Admin Quick Actions
# -----------------------------------------------------------------------------
admin-quick-actions = 快速行动
admin-create-user = 创建用户
admin-create-group = 创建群组
admin-register-dns = 注册DNS
admin-recent-activity = 最近的活动
admin-system-health = 系统健康状况

# -----------------------------------------------------------------------------
# User Management
# -----------------------------------------------------------------------------
admin-users-title = 用户管理
admin-users-list = 用户列表
admin-users-add = 添加用户
admin-users-edit = 编辑用户
admin-users-delete = 删除用户
admin-users-search = 搜索用户...
admin-users-filter = 过滤用户
admin-users-export = 导出用户
admin-users-import = 导入用户
admin-users-total = 用户总数
admin-users-active = 活跃用户
admin-users-inactive = 不活跃用户
admin-users-suspended = 暂停的用户
admin-users-pending = 待验证
admin-users-last-login = 上次登录
admin-users-created = 已创建
admin-users-role = 角色
admin-users-status = 状态
admin-users-actions = 行动
admin-users-no-users = 没有找到用户
admin-users-confirm-delete = 您确定要删除该用户吗？
admin-users-deleted = 用户删除成功
admin-users-saved = 用户保存成功
admin-users-invite = 邀请用户
admin-users-invite-sent = 邀请发送成功
admin-users-bulk-actions = 批量操作
admin-users-select-all = 选择全部
admin-users-deselect-all = 取消全选

# User Details
admin-user-details = 用户详细信息
admin-user-profile = 公司简介
admin-user-email = 电子邮件
admin-user-name = 名称
admin-user-phone = 电话
admin-user-avatar = 阿凡达
admin-user-timezone = 时区
admin-user-language = 语言
admin-user-role-admin = 管理员
admin-user-role-manager = 经理
admin-user-role-user = 用户
admin-user-role-viewer = 观众
admin-user-status-active = 活跃
admin-user-status-inactive = 不活跃
admin-user-status-suspended = 暂停
admin-user-status-pending = 待定
admin-user-permissions = 权限
admin-user-activity = 活动日志
admin-user-sessions = 活跃会话
admin-user-terminate-session = 终止会话
admin-user-terminate-all = 终止所有会话
admin-user-reset-password = 重置密码
admin-user-force-logout = 强制注销
admin-user-enable-2fa = 启用 2FA
admin-user-disable-2fa = 禁用 2FA

# -----------------------------------------------------------------------------
# Group Management
# -----------------------------------------------------------------------------
admin-groups-title = 集团管理
admin-groups-subtitle = 管理群组、成员和权限
admin-groups-list = 团体名单
admin-groups-add = 添加组
admin-groups-create = 创建群组
admin-groups-edit = 编辑组
admin-groups-delete = 删除组
admin-groups-search = 搜索组...
admin-groups-filter = 过滤组
admin-groups-total = 总组数
admin-groups-active = 活跃团体
admin-groups-no-groups = 未找到群组
admin-groups-confirm-delete = 您确定要删除该群组吗？
admin-groups-deleted = 群组删除成功
admin-groups-saved = 群组保存成功
admin-groups-created = 群组创建成功
admin-groups-loading = 正在加载组...

# Group Details
admin-group-details = 团体详情
admin-group-name = 群组名称
admin-group-description = 描述
admin-group-visibility = 能见度
admin-group-visibility-public = 公共
admin-group-visibility-private = 私人
admin-group-visibility-hidden = 隐藏
admin-group-join-policy = 加盟政策
admin-group-join-invite = 仅限受邀者
admin-group-join-request = 请求加入
admin-group-join-open = 打开
admin-group-members = 会员
admin-group-member-count = { $计数 ->
    [one] { $count } member
   *[other] { $count } members
}
admin-group-add-member = 添加会员
admin-group-remove-member = 删除会员
admin-group-permissions = 权限
admin-group-settings = 设置
admin-group-analytics = 分析
admin-group-overview = 概述

# Group View Modes
admin-groups-view-grid = 网格视图
admin-groups-view-list = 列表视图
admin-groups-all-visibility = 所有可见性

# -----------------------------------------------------------------------------
# DNS Management
# -----------------------------------------------------------------------------
admin-dns-title = DNS管理
admin-dns-subtitle = 为您的机器人注册和管理 DNS 主机名
admin-dns-register = 注册主机名
admin-dns-registered = 注册主机名
admin-dns-search = 搜索主机名...
admin-dns-refresh = 刷新
admin-dns-loading = 正在加载 DNS 记录...
admin-dns-no-records = 未找到 DNS 记录
admin-dns-confirm-delete = 您确定要删除此主机名吗？
admin-dns-deleted = 主机名删除成功
admin-dns-saved = DNS记录保存成功
admin-dns-created = 主机名注册成功

# DNS Form Fields
admin-dns-hostname = 主机名
admin-dns-hostname-placeholder = mybot.example.com
admin-dns-hostname-help = 输入您要注册的完整域名
admin-dns-record-type = 记录类型
admin-dns-record-type-a = 一个（IPv4）
admin-dns-record-type-aaaa = AAAA (IPv6)
admin-dns-record-type-cname = 别名记录
admin-dns-ttl = TTL（秒）
admin-dns-ttl-5min = 5 分钟 (300)
admin-dns-ttl-1hour = 1小时（3600）
admin-dns-ttl-1day = 1 天 (86400)
admin-dns-target = 目标/IP地址
admin-dns-target-placeholder-ipv4 = 192.168.1.1
admin-dns-target-placeholder-ipv6 = 2001:db8::1
admin-dns-target-placeholder-cname = target.example.com
admin-dns-target-help-a = 输入要指向的 IPv4 地址
admin-dns-target-help-aaaa = 输入要指向的 IPv6 地址
admin-dns-target-help-cname = 输入目标域名
admin-dns-auto-ssl = 自动提供 SSL 证书

# DNS Table Headers
admin-dns-col-hostname = 主机名
admin-dns-col-type = 类型
admin-dns-col-target = 目标
admin-dns-col-ttl = TTL
admin-dns-col-ssl = SSL协议
admin-dns-col-status = 状态
admin-dns-col-actions = 行动

# DNS Status
admin-dns-status-active = 活跃
admin-dns-status-pending = 待定
admin-dns-status-error = 错误
admin-dns-ssl-enabled = 启用 SSL
admin-dns-ssl-disabled = 无 SSL
admin-dns-ssl-pending = SSL 待定

# DNS Info Cards
admin-dns-help-title = DNS 配置帮助
admin-dns-help-a-record = 记录
admin-dns-help-a-record-desc = 将域名映射到 IPv4 地址。使用它可以将您的主机名直接指向服务器 IP。
admin-dns-help-aaaa-record = AAAA记录
admin-dns-help-aaaa-record-desc = 将域名映射到 IPv6 地址。与 A 记录类似，但用于 IPv6 连接。
admin-dns-help-cname-record = CNAME记录
admin-dns-help-cname-record-desc = 创建从一个域到另一个域的别名。对于将子域指向您的主域很有用。
admin-dns-help-ssl = SSL/TLS
admin-dns-help-ssl-desc = 自动配置 Let's Encrypt 证书以实现安全 HTTPS 连接。

# DNS Edit/Remove Modals
admin-dns-edit-title = 编辑 DNS 记录
admin-dns-remove-title = 删除主机名
admin-dns-remove-warning = 这将删除 DNS 记录和任何关联的 SSL 证书。主机名将不再解析。

# -----------------------------------------------------------------------------
# Bot Management
# -----------------------------------------------------------------------------
admin-bots-title = 机器人管理
admin-bots-list = 机器人列表
admin-bots-add = 添加机器人
admin-bots-edit = 编辑机器人
admin-bots-delete = 删除机器人
admin-bots-search = 搜索机器人...
admin-bots-filter = 过滤机器人
admin-bots-total = 机器人总数
admin-bots-active = 活跃机器人
admin-bots-inactive = 不活跃的机器人
admin-bots-draft = 选秀机器人
admin-bots-published = 已发布的机器人
admin-bots-no-bots = 未找到机器人
admin-bots-confirm-delete = 您确定要删除此机器人吗？
admin-bots-deleted = 机器人删除成功
admin-bots-saved = 机器人保存成功
admin-bots-duplicate = 重复机器人
admin-bots-export = 导出机器人
admin-bots-import = 导入机器人
admin-bots-publish = 发布
admin-bots-unpublish = 取消发布
admin-bots-test = 测试机器人
admin-bots-logs = 机器人日志
admin-bots-analytics = 机器人分析
admin-bots-conversations = 对话
admin-bots-templates = 模板
admin-bots-dialogs = 对话框
admin-bots-knowledge-base = 知识库

# Bot Details
admin-bot-details = 机器人详情
admin-bot-name = 机器人名称
admin-bot-description = 描述
admin-bot-avatar = 机器人头像
admin-bot-language = 语言
admin-bot-timezone = 时区
admin-bot-greeting = 问候语
admin-bot-fallback = 回退消息
admin-bot-channels = 渠道
admin-bot-channel-web = 网络聊天
admin-bot-channel-whatsapp = WhatsApp
admin-bot-channel-telegram = 电报
admin-bot-channel-slack = 松弛
admin-bot-channel-teams = 微软团队
admin-bot-channel-email = 电子邮件
admin-bot-model = 人工智能模型
admin-bot-temperature = 温度
admin-bot-max-tokens = 最大代币数
admin-bot-system-prompt = 系统提示

# -----------------------------------------------------------------------------
# Tenant Management
# -----------------------------------------------------------------------------
admin-tenants-title = 租户管理
admin-tenants-list = 租户名单
admin-tenants-add = 添加租户
admin-tenants-edit = 编辑租户
admin-tenants-delete = 删除租户
admin-tenants-search = 搜寻租户...
admin-tenants-total = 租户总数
admin-tenants-active = 活跃租户
admin-tenants-suspended = 暂停租客
admin-tenants-trial = 试用租户
admin-tenants-no-tenants = 没有找到租户
admin-tenants-confirm-delete = 您确定要删除该租户吗？
admin-tenants-deleted = 租户删除成功
admin-tenants-saved = 租户保存成功

# Tenant Details
admin-tenant-details = 租户详情
admin-tenant-name = 租户名称
admin-tenant-domain = 域名
admin-tenant-plan = 计划
admin-tenant-plan-free = 免费
admin-tenant-plan-starter = 入门者
admin-tenant-plan-professional = 专业
admin-tenant-plan-enterprise = 企业
admin-tenant-users = 用户
admin-tenant-bots = 机器人
admin-tenant-storage = 已用存储空间
admin-tenant-api-calls = API调用
admin-tenant-limits = 使用限制
admin-tenant-billing = 账单信息

# -----------------------------------------------------------------------------
# System Settings
# -----------------------------------------------------------------------------
admin-settings-title = 系统设置
admin-settings-general = 常规设置
admin-settings-security = 安全设置
admin-settings-email = 电子邮件设置
admin-settings-storage = 存储设置
admin-settings-integrations = 集成
admin-settings-api = API设置
admin-settings-appearance = 外观
admin-settings-localization = 本地化
admin-settings-notifications = 通知
admin-settings-backup = 备份与恢复
admin-settings-maintenance = 维护方式
admin-settings-saved = 设置保存成功
admin-settings-reset = 重置为默认值
admin-settings-confirm-reset = 您确定要将所有设置重置为默认值吗？

# General Settings
admin-settings-site-name = 站点名称
admin-settings-site-url = 站点网址
admin-settings-admin-email = 管理员邮箱
admin-settings-support-email = 支持电子邮件
admin-settings-default-language = 默认语言
admin-settings-default-timezone = 默认时区
admin-settings-date-format = 日期格式
admin-settings-time-format = 时间格式
admin-settings-currency = 货币

# Email Settings
admin-settings-smtp-host = SMTP 主机
admin-settings-smtp-port = SMTP 端口
admin-settings-smtp-user = SMTP 用户名
admin-settings-smtp-password = 邮件发送密码
admin-settings-smtp-encryption = 加密
admin-settings-smtp-from-name = 来自姓名
admin-settings-smtp-from-email = 来自电子邮件
admin-settings-smtp-test = 发送测试电子邮件
admin-settings-smtp-test-success = 测试邮件发送成功
admin-settings-smtp-test-failed = 发送测试邮件失败

# Storage Settings
admin-settings-storage-provider = 存储提供商
admin-settings-storage-local = 本地存储
admin-settings-storage-s3 = 亚马逊S3
admin-settings-storage-minio = 最小IO
admin-settings-storage-gcs = 谷歌云存储
admin-settings-storage-azure = Azure Blob 存储
admin-settings-storage-bucket = 桶名称
admin-settings-storage-region = 地区
admin-settings-storage-access-key = 访问密钥
admin-settings-storage-secret-key = 秘密钥匙
admin-settings-storage-endpoint = 端点 URL

# -----------------------------------------------------------------------------
# System Logs
# -----------------------------------------------------------------------------
admin-logs-title = 系统日志
admin-logs-search = 搜索日志...
admin-logs-filter-level = 按级别过滤
admin-logs-filter-source = 按来源过滤
admin-logs-filter-date = 按日期过滤
admin-logs-level-all = 所有级别
admin-logs-level-debug = 调试
admin-logs-level-info = 信息
admin-logs-level-warning = 警告
admin-logs-level-error = 错误
admin-logs-level-critical = 关键
admin-logs-export = 导出日志
admin-logs-clear = 清除日志
admin-logs-confirm-clear = 您确定要清除所有日志吗？
admin-logs-cleared = 日志清除成功
admin-logs-no-logs = 没有找到日志
admin-logs-refresh = 刷新
admin-logs-auto-refresh = 自动刷新
admin-logs-timestamp = 时间戳
admin-logs-level = 级别
admin-logs-source = 来源
admin-logs-message = 留言
admin-logs-details = 详情

# -----------------------------------------------------------------------------
# Analytics
# -----------------------------------------------------------------------------
admin-analytics-title = 分析
admin-analytics-overview = 概述
admin-analytics-users = 用户分析
admin-analytics-bots = 机器人分析
admin-analytics-conversations = 对话分析
admin-analytics-performance = 性能
admin-analytics-period = 时间段
admin-analytics-period-today = 今天
admin-analytics-period-week = 本周
admin-analytics-period-month = 本月
admin-analytics-period-quarter = 本季度
admin-analytics-period-year = 今年
admin-analytics-period-custom = 定制范围
admin-analytics-export = 出口报告
admin-analytics-total-users = 用户总数
admin-analytics-new-users = 新用户
admin-analytics-active-users = 活跃用户
admin-analytics-total-bots = 机器人总数
admin-analytics-active-bots = 活跃机器人
admin-analytics-total-conversations = 总对话数
admin-analytics-avg-response-time = 平均响应时间
admin-analytics-satisfaction-rate = 满意率
admin-analytics-resolution-rate = 解决率

# -----------------------------------------------------------------------------
# Security
# -----------------------------------------------------------------------------
admin-security-title = 安全性
admin-security-overview = 安全概述
admin-security-audit-log = 审核日志
admin-security-login-attempts = 登录尝试
admin-security-blocked-ips = 被封锁的IP
admin-security-api-keys = API 密钥
admin-security-webhooks = 网络钩子
admin-security-cors = CORS 设置
admin-security-rate-limiting = 速率限制
admin-security-encryption = 加密
admin-security-2fa = 双因素身份验证
admin-security-sso = 单点登录
admin-security-password-policy = 密码政策

# API Keys
admin-api-keys-title = API 密钥
admin-api-keys-add = 创建 API 密钥
admin-api-keys-name = 按键名称
admin-api-keys-key = API密钥
admin-api-keys-secret = 秘密钥匙
admin-api-keys-created = 已创建
admin-api-keys-last-used = 最后使用
admin-api-keys-expires = 过期
admin-api-keys-never = 从来没有
admin-api-keys-revoke = 撤销
admin-api-keys-confirm-revoke = 您确定要撤销此 API 密钥吗？
admin-api-keys-revoked = API密钥已成功撤销
admin-api-keys-created-success = API 密钥创建成功
admin-api-keys-copy = 复制到剪贴板
admin-api-keys-copied = 复制了！
admin-api-keys-warning = 请确保立即复制您的 API 密钥。你将再也看不到它了！

# -----------------------------------------------------------------------------
# Billing
# -----------------------------------------------------------------------------
admin-billing-title = 计费
admin-billing-overview = 计费概览
admin-billing-current-plan = 当前计划
admin-billing-usage = 用途
admin-billing-invoices = 发票
admin-billing-payment-methods = 付款方式
admin-billing-upgrade = 升级计划
admin-billing-downgrade = 降级计划
admin-billing-cancel = 取消订阅
admin-billing-invoice-date = 发票日期
admin-billing-invoice-amount = 金额
admin-billing-invoice-status = 状态
admin-billing-invoice-paid = 付费
admin-billing-invoice-pending = 待定
admin-billing-invoice-overdue = 逾期
admin-billing-invoice-download = 下载发票

# -----------------------------------------------------------------------------
# Backup & Restore
# -----------------------------------------------------------------------------
admin-backup-title = 备份与恢复
admin-backup-create = 创建备份
admin-backup-restore = 恢复备份
admin-backup-schedule = 安排备份
admin-backup-list = 备份历史记录
admin-backup-name = 备份名称
admin-backup-size = 尺寸
admin-backup-created = 已创建
admin-backup-download = 下载
admin-backup-delete = 删除
admin-backup-confirm-restore = 您确定要恢复此备份吗？这将覆盖当前数据。
admin-backup-confirm-delete = 您确定要删除此备份吗？
admin-backup-in-progress = 备份正在进行中...
admin-backup-completed = 备份成功完成
admin-backup-failed = 备份失败
admin-backup-restore-in-progress = 恢复正在进行中...
admin-backup-restore-completed = 恢复成功完成
admin-backup-restore-failed = 恢复失败

# -----------------------------------------------------------------------------
# Maintenance Mode
# -----------------------------------------------------------------------------
admin-maintenance-title = 维护方式
admin-maintenance-enable = 启用维护模式
admin-maintenance-disable = 禁用维护模式
admin-maintenance-status = 目前状态
admin-maintenance-active = 维护模式已激活
admin-maintenance-inactive = 维护模式未激活
admin-maintenance-message = 维修留言
admin-maintenance-default-message = 我们目前正在进行定期维护。请尽快回来查看。
admin-maintenance-allowed-ips = 允许的 IP 地址
admin-maintenance-confirm-enable = 您确定要启用维护模式吗？用户将无法访问系统。

# -----------------------------------------------------------------------------
# Common Admin UI Elements
# -----------------------------------------------------------------------------
admin-required = 必填
admin-optional = 可选
admin-loading = 加载中...
admin-saving = 正在保存...
admin-deleting = 正在删除...
admin-confirm = 确认
admin-cancel = 取消
admin-save = 保存
admin-create = 创建
admin-update = 更新
admin-delete = 删除
admin-edit = 编辑
admin-view = 查看
admin-close = 关闭
admin-back = 返回
admin-next = 下一步
admin-previous = 上一页
admin-refresh = 刷新
admin-export = 出口
admin-import = 进口
admin-search = 搜索
admin-filter = 过滤器
admin-clear = 清除
admin-select = 选择
admin-select-all = 选择全部
admin-deselect-all = 取消全选
admin-actions = 行动
admin-more-actions = 更多行动
admin-no-data = 无可用数据
admin-error = 发生错误
admin-success = 成功
admin-warning = 警告
admin-info = 信息

# Table Pagination
admin-showing = 显示第 { $from } 至 { $to }（共 { $total }）结果
admin-page = 第 { $current } 页（共 { $total }）
admin-items-per-page = 每页项目数
admin-go-to-page = 前往页面

# Bulk Actions
admin-bulk-delete = 删除所选内容
admin-bulk-export = 导出选定的内容
admin-bulk-activate = 激活选定的
admin-bulk-deactivate = 停用选定的
admin-selected-count = { $计数 ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
