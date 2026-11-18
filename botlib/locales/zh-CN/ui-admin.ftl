# =============================================================================
# General Bots — zh-CN — Admin, monitoring and system apps
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Panel (additional UI keys)
# -----------------------------------------------------------------------------
admin-panel-title = 管理面板

# =============================================================================
# MONITORING SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Monitoring Dashboard
# -----------------------------------------------------------------------------
monitoring-title = 监控仪表板
monitoring-toggle-view = 切换视图
monitoring-last-updated = 最后更新
monitoring-live-view = 实时取景
monitoring-grid-view = 网格视图

# -----------------------------------------------------------------------------
# Monitoring Panels
# -----------------------------------------------------------------------------
monitoring-sessions = 会议
monitoring-messages = 留言
monitoring-resources = 资源
monitoring-services = 服务
monitoring-active-bots = 活跃机器人
monitoring-loading = 加载中...

# -----------------------------------------------------------------------------
# Service Status
# -----------------------------------------------------------------------------
monitoring-status-running = 跑步
monitoring-status-warning = 警告
monitoring-status-stopped = 已停止
monitoring-status-healthy = 健康
monitoring-status-degraded = 降级
monitoring-status-down = 向下

# -----------------------------------------------------------------------------
# Resource Metrics
# -----------------------------------------------------------------------------
monitoring-cpu = 中央处理器
monitoring-memory = 内存
monitoring-disk = 磁盘
monitoring-network = 网络
monitoring-requests-per-sec = 请求数/秒
monitoring-active-connections = 活跃连接数
monitoring-uptime = 正常运行时间

# -----------------------------------------------------------------------------
# Logs
# -----------------------------------------------------------------------------
monitoring-logs-title = 系统日志
monitoring-logs-filter = 过滤日志
monitoring-logs-level = 日志级别
monitoring-logs-all = 所有级别
monitoring-logs-debug = 调试
monitoring-logs-info = 信息
monitoring-logs-warning = 警告
monitoring-logs-error = 错误
monitoring-logs-critical = 关键
monitoring-logs-search = 搜索日志...
monitoring-logs-no-results = 没有找到日志

# -----------------------------------------------------------------------------
# Health
# -----------------------------------------------------------------------------
monitoring-health-title = 系统健康状况
monitoring-health-status = 健康状况
monitoring-health-services = 服务健康
monitoring-health-database = 数据库
monitoring-health-cache = 缓存
monitoring-health-queue = 消息队列
monitoring-health-storage = 存储
monitoring-health-external = 外部服务

# -----------------------------------------------------------------------------
# Metrics
# -----------------------------------------------------------------------------
monitoring-metrics-title = 绩效指标
monitoring-metrics-response-time = 响应时间
monitoring-metrics-throughput = 吞吐量
monitoring-metrics-error-rate = 错误率
monitoring-metrics-latency = 延迟

# -----------------------------------------------------------------------------
# Alerts
# -----------------------------------------------------------------------------
monitoring-alerts-title = 系统警报
monitoring-alerts-active = 主动警报
monitoring-alerts-resolved = 已解决
monitoring-alerts-all = 所有警报
monitoring-alert-severity = 严重性
monitoring-alert-critical = 关键
monitoring-alert-high = 高
monitoring-alert-medium = 中等
monitoring-alert-low = 低
monitoring-alert-info = 信息
monitoring-alert-acknowledge = 确认
monitoring-alert-resolve = 解决
monitoring-no-alerts = 没有活动警报

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
security-tab-compliance = API 合规报告
security-tab-protection = 保护
security-export-report = 出口报告
security-run-scan = 运行合规性扫描
security-critical = 关键
security-critical-desc = 需要立即采取行动
security-high = 高
security-high-desc = 安全风险
security-medium = 中等
security-medium-desc = 应该解决
security-low = 低
security-low-desc = 最佳实践
security-info = 信息
security-info-desc = 信息性
security-filter-severity = 严重程度：
security-filter-all-severities = 所有严重程度
security-filter-type = 类型：
security-filter-all-types = 所有类型
security-type-password = 配置中的密码
security-type-hardcoded = 硬编码的秘密
security-type-deprecated = 已弃用的关键字
security-type-fragile = 脆弱的代码
security-type-config = 配置问题
security-results = 合规问题
security-col-severity = 严重性
security-col-issue = 问题类型
security-col-location = 地点
security-col-details = 描述
security-col-action = 行动

# -----------------------------------------------------------------------------
# Learn Module
# -----------------------------------------------------------------------------
learn-title = 学习
learn-my-progress = 我的进步
learn-completed = 已完成
learn-in-progress = 进行中
learn-certificates = 证书
learn-time-spent = 花费的时间
learn-categories = 类别
learn-all-courses = 所有课程
learn-mandatory = 强制
learn-compliance = 合规性
learn-security = 安全性
learn-skills = 技能
learn-onboarding = 入职
learn-difficulty = 难度
learn-my-certificates = 我的证书
learn-view-all = 查看全部

# -----------------------------------------------------------------------------
# Workspace Module
# -----------------------------------------------------------------------------
workspace-title = 工作空间
workspace-search-pages = 搜索页面...
workspace-recent = 最近
workspace-favorites = 收藏夹
workspace-pages = 页数
workspace-templates = 模板
workspace-trash = 垃圾
workspace-settings = 设置

# -----------------------------------------------------------------------------
# Player Module
# -----------------------------------------------------------------------------
player-title = 媒体播放器
player-no-file = 未选择文件
player-search = 搜索文件...
player-recent = 最近
player-files = 文件

# -----------------------------------------------------------------------------
# Goals Module
# -----------------------------------------------------------------------------
goals-title = 目标和 OKR
goals-dashboard = 仪表板
goals-objectives = 目标
goals-alignment = 对准
goals-ai-suggestions = 人工智能建议

# semantic i18n key (#2542)
admin-ops-dashboard-title = 运维仪表板

# semantic i18n key (#2542)
admin-ops-dashboard-subtitle = 实时系统指标、分布式追踪和性能监控

# semantic i18n key (#2542)
admin-ops-auto-refresh-on = 自动刷新：开启

# semantic i18n key (#2542)
admin-ops-avg-response = 平均响应

# semantic i18n key (#2542)
admin-ops-requests-min = 请求/分钟

# semantic i18n key (#2542)
admin-ops-disk-io = 磁盘 I/O

# semantic i18n key (#2542)
admin-ops-iops-read-write = IOPS（读/写）

# semantic i18n key (#2542)
admin-ops-in-95-out-50 = 入站：95 / 出站：50

# semantic i18n key (#2542)
admin-ops-stable = 稳定

# semantic i18n key (#2542)
admin-ops-request-metrics = 请求指标

# semantic i18n key (#2542)
admin-ops-errors-label = 错误

# semantic i18n key (#2542)
admin-ops-distributed-tracing = 分布式追踪

# semantic i18n key (#2542)
admin-ops-all-traces = 所有追踪

# semantic i18n key (#2542)
admin-ops-errors-only = 仅错误

# semantic i18n key (#2542)
admin-ops-api-requests = API 请求

# semantic i18n key (#2542)
admin-ops-post-api-chat-message = POST /api/chat/message

# semantic i18n key (#2542)
admin-ops-trace-a1b2c3d4e5f6 = trace_a1b2c3d4e5f6

# semantic i18n key (#2542)
admin-ops-get-api-kb-search = GET /api/kb/search

# semantic i18n key (#2542)
admin-ops-trace-b2c3d4e5f6g7 = trace_b2c3d4e5f6g7

# semantic i18n key (#2542)
admin-ops-post-api-billing-checkout = POST /api/billing/checkout

# semantic i18n key (#2542)
admin-ops-trace-c3d4e5f6g7h8 = trace_c3d4e5f6g7h8

# semantic i18n key (#2542)
admin-ops-get-api-user-profile = GET /api/user/profile

# semantic i18n key (#2542)
admin-ops-trace-d4e5f6g7h8i9 = trace_d4e5f6g7h8i9

# semantic i18n key (#2542)
admin-ops-api-gateway = API 网关

# semantic i18n key (#2542)
admin-ops-primary-2-replicas = 主节点 + 2 个副本

# semantic i18n key (#2542)
admin-ops-qdrant-vector-db = Qdrant（向量数据库）

# semantic i18n key (#2542)
admin-ops-redis-cache = Redis 缓存

# semantic i18n key (#2542)
admin-ops-cluster-mode-3-shards = 集群模式 | 3 个分片

# semantic i18n key (#2542)
admin-ops-llm-service = LLM 服务

# semantic i18n key (#2542)
admin-ops-openai-gpt-4-high-latency = OpenAI GPT-4 | 高延迟

# semantic i18n key (#2542)
admin-ops-object-storage = 对象存储

# semantic i18n key (#2542)
admin-ops-s3-compatible = 兼容 S3

# semantic i18n key (#2542)
admin-ops-error-tracking = 错误跟踪

# semantic i18n key (#2542)
admin-ops-view-all-errors = 查看所有错误

# semantic i18n key (#2542)
admin-ops-warnings = 警告

# semantic i18n key (#2542)
admin-ops-payment-processing-failed-stripe-timeout = 付款处理失败：Stripe API 超时

# semantic i18n key (#2542)
admin-ops-billing-checkout-rs-145 = billing/checkout.rs:145

# semantic i18n key (#2542)
admin-ops-rate-limit-approaching-openai = OpenAI API 即将达到速率限制

# semantic i18n key (#2542)
admin-ops-llm-openai-rs-89 = llm/openai.rs:89

# semantic i18n key (#2542)
admin-ops-core-analytics-rs-234 = core/analytics.rs:234

# semantic i18n key (#2542)
admin-ops-cache-miss-rate-above-threshold = 缓存未命中率超过阈值

# semantic i18n key (#2542)
admin-ops-cache-redis-rs-67 = cache/redis.rs:67

# semantic i18n key (#2542)
admin-ops-endpoint-performance = 端点性能

# semantic i18n key (#2542)
admin-ops-by-requests = 按请求数

# semantic i18n key (#2542)
admin-ops-by-latency = 按延迟

# semantic i18n key (#2542)
admin-ops-by-error-rate = 按错误率

# semantic i18n key (#2542)
admin-ops-post = POST

# semantic i18n key (#2542)
admin-ops-api-chat-message = /api/chat/message

# semantic i18n key (#2542)
admin-ops-req-min = 请求/分钟

# semantic i18n key (#2542)
admin-ops-errors-metric = 错误

# semantic i18n key (#2542)
admin-ops-api-kb-search = /api/kb/search

# semantic i18n key (#2542)
admin-roles-title = 角色管理

# semantic i18n key (#2542)
admin-roles-subtitle = 配置角色并分配权限，以控制整个套件的访问

# semantic i18n key (#2542)
admin-roles-sidebar-title = 角色

# semantic i18n key (#2542)
admin-roles-system-filter = 系统角色

# semantic i18n key (#2542)
admin-roles-custom-filter = 自定义角色

# semantic i18n key (#2542)
admin-roles-select-title = 选择角色

# semantic i18n key (#2542)
admin-roles-select-hint = 从列表中选择角色以查看和管理其权限

# semantic i18n key (#2542)
admin-roles-name-label = 角色名称

# semantic i18n key (#2542)
admin-roles-name-badge-placeholder = role_name

# semantic i18n key (#2542)
admin-roles-description-placeholder = 角色描述显示在此处

# semantic i18n key (#2542)
admin-roles-permissions-label = 权限

# semantic i18n key (#2542)
admin-roles-assigned-users-tab = 已分配用户

# semantic i18n key (#2542)
admin-roles-assigned-groups-tab = 已分配组

# semantic i18n key (#2542)
admin-roles-permission-assignment = 权限分配

# semantic i18n key (#2542)
admin-roles-permission-assignment-help = 使用双列表为此角色分配权限

# semantic i18n key (#2542)
admin-roles-available-permissions = 可用权限

# semantic i18n key (#2542)
admin-roles-category-documents = 文档

# semantic i18n key (#2542)
admin-roles-category-spreadsheets = 电子表格

# semantic i18n key (#2542)
admin-roles-category-presentations = 演示文稿

# semantic i18n key (#2542)
admin-roles-category-meetings = 会议

# semantic i18n key (#2542)
admin-roles-category-ai-bots = AI 与机器人

# semantic i18n key (#2542)
admin-roles-category-automation = 自动化

# semantic i18n key (#2542)
admin-roles-assigned-permissions = 已分配权限

# semantic i18n key (#2542)
admin-roles-no-permissions-assigned = 未分配权限

# semantic i18n key (#2542)
admin-roles-reset-to-default = 重置为默认值

# semantic i18n key (#2542)
admin-roles-users-with-role = 拥有此角色的用户

# semantic i18n key (#2542)
admin-roles-groups-with-role = 拥有此角色的组

# semantic i18n key (#2542)
admin-roles-change-history = 角色变更历史

# semantic i18n key (#2542)
admin-roles-create-new-role = 创建新角色

# semantic i18n key (#2542)
admin-roles-name-help = 小写，无空格（使用下划线）

# semantic i18n key (#2542)
admin-roles-copy-permissions-from = 从以下位置复制权限

# semantic i18n key (#2542)
admin-roles-start-with-no-permissions = 从无权限开始

# semantic i18n key (#2542)
admin-roles-standard-user = 标准用户

# semantic i18n key (#2542)
admin-roles-power-user = 高级用户

# semantic i18n key (#2542)
admin-roles-viewer = 查看者

# semantic i18n key (#2542)
admin-roles-guest-user = 访客用户

# semantic i18n key (#2542)
admin-roles-create-role-btn = 创建角色

# semantic i18n key (#2542)
admin-roles-delete-role-title = 删除角色

# semantic i18n key (#2542)
admin-roles-delete-warning = 这将从所有用户和组中移除该角色。此操作无法撤消。

# semantic i18n key (#2542)
admin-roles-assign-to-users = 将角色分配给用户

# semantic i18n key (#2542)
admin-roles-search-users-label = 搜索用户

# semantic i18n key (#2542)
admin-roles-type-to-search-users = 输入以搜索用户

# semantic i18n key (#2542)
admin-roles-expiration-optional = 到期时间（可选）

# semantic i18n key (#2542)
admin-roles-expiration-help = 留空表示永久分配

# semantic i18n key (#2542)
admin-roles-assign-to-groups = 将角色分配给组

# semantic i18n key (#2542)
admin-roles-search-groups-label = 搜索组

# semantic i18n key (#2542)
admin-roles-type-to-search-groups = 输入以搜索组

# semantic i18n key (#2542)
admin-roles-assign-role-btn = 分配角色

# semantic i18n key (#2542)
admin-onboarding-step-welcome = 欢迎

# semantic i18n key (#2542)
admin-onboarding-step-welcome-desc = 介绍一下您自己

# semantic i18n key (#2542)
admin-onboarding-step-org-desc = 创建您的工作区

# semantic i18n key (#2542)
admin-onboarding-step-plan = 选择套餐

# semantic i18n key (#2542)
admin-onboarding-step-plan-desc = 选择您的订阅

# semantic i18n key (#2542)
admin-onboarding-step-payment-desc = 添加付款方式

# semantic i18n key (#2542)
admin-onboarding-step-start = 开始使用

# semantic i18n key (#2542)
admin-onboarding-step-start-desc = 创建您的第一个机器人

# semantic i18n key (#2542)
admin-onboarding-view-guide = 查看指南

# semantic i18n key (#2542)
admin-onboarding-welcome-title = 欢迎使用 General Bots！

# semantic i18n key (#2542)
admin-onboarding-welcome-subtitle = 我们将在几分钟内完成设置。首先，请简单介绍一下您自己。

# semantic i18n key (#2542)
admin-onboarding-full-name = 全名

# semantic i18n key (#2542)
admin-onboarding-your-role = 您的角色

# semantic i18n key (#2542)
admin-onboarding-role-placeholder = 选择您的角色...

# semantic i18n key (#2542)
admin-onboarding-role-developer = 开发人员 / 工程师

# semantic i18n key (#2542)
admin-onboarding-role-product-manager = 产品经理

# semantic i18n key (#2542)
admin-onboarding-role-customer-support = 客户支持

# semantic i18n key (#2542)
admin-onboarding-role-executive = 高管 / 创始人

# semantic i18n key (#2542)
admin-onboarding-use-case-label = 您想构建什么？

# semantic i18n key (#2542)
admin-onboarding-use-case-placeholder = 选择主要用例...

# semantic i18n key (#2542)
admin-onboarding-usecase-support-bot = 客户支持机器人

# semantic i18n key (#2542)
admin-onboarding-usecase-sales-assistant = 销售助理

# semantic i18n key (#2542)
admin-onboarding-usecase-internal-kb = 内部知识库

# semantic i18n key (#2542)
admin-onboarding-usecase-lead-generation = 潜在客户开发

# semantic i18n key (#2542)
admin-onboarding-usecase-employee-onboarding = 员工入职

# semantic i18n key (#2542)
admin-onboarding-usecase-ecommerce = 电子商务助理

# semantic i18n key (#2542)
admin-onboarding-usecase-other = 其他

# semantic i18n key (#2542)
admin-onboarding-hear-about-label = 您是如何了解我们的？

# semantic i18n key (#2542)
admin-onboarding-source-search = 搜索引擎

# semantic i18n key (#2542)
admin-onboarding-source-social = 社交媒体

# semantic i18n key (#2542)
admin-onboarding-source-friend = 朋友 / 同事

# semantic i18n key (#2542)
admin-onboarding-create-org-title = 创建您的组织

# semantic i18n key (#2542)
admin-onboarding-create-org-subtitle = 您将在组织中管理机器人、团队成员和知识库。

# semantic i18n key (#2542)
admin-onboarding-org-name = 组织名称

# semantic i18n key (#2542)
admin-onboarding-org-name-hint = 这将对您的团队成员可见

# semantic i18n key (#2542)
admin-onboarding-org-url = 组织 URL

# semantic i18n key (#2542)
admin-onboarding-org-url-prefix = app.generalbots.com/

# semantic i18n key (#2542)
admin-onboarding-org-url-hint = 仅限小写字母、数字和连字符

# semantic i18n key (#2542)
admin-onboarding-team-size = 团队规模

# semantic i18n key (#2542)
admin-onboarding-team-size-placeholder = 将有多少人使用？

# semantic i18n key (#2542)
admin-onboarding-team-just-me = 仅我自己

# semantic i18n key (#2542)
admin-onboarding-industry-placeholder = 选择您的行业（可选）

# semantic i18n key (#2542)
admin-onboarding-industry-technology = 科技

# semantic i18n key (#2542)
admin-onboarding-industry-healthcare = 医疗保健

# semantic i18n key (#2542)
admin-onboarding-industry-finance = 金融 / 银行

# semantic i18n key (#2542)
admin-onboarding-industry-retail = 零售 / 电子商务

# semantic i18n key (#2542)
admin-onboarding-industry-education = 教育

# semantic i18n key (#2542)
admin-onboarding-industry-manufacturing = 制造业

# semantic i18n key (#2542)
admin-onboarding-industry-consulting = 咨询 / 服务

# semantic i18n key (#2542)
admin-onboarding-industry-government = 政府

# semantic i18n key (#2542)
admin-onboarding-industry-nonprofit = 非营利

# semantic i18n key (#2542)
admin-onboarding-choose-plan-title = 选择您的套餐

# semantic i18n key (#2542)
admin-onboarding-choose-plan-subtitle = 选择最适合您需求的套餐。您可以随时升级或降级。

# semantic i18n key (#2542)
admin-onboarding-plan-free = 免费

# semantic i18n key (#2542)
admin-onboarding-plan-free-desc = 适用于个人和小型项目

# semantic i18n key (#2542)
admin-onboarding-per-month = /月

# semantic i18n key (#2542)
admin-onboarding-select-free = 选择免费版

# semantic i18n key (#2542)
admin-onboarding-most-popular = 最受欢迎

# semantic i18n key (#2542)
admin-onboarding-plan-pro-desc = 适用于成长型团队

# semantic i18n key (#2542)
admin-onboarding-select-pro = 选择专业版

# semantic i18n key (#2542)
admin-dashboard-organization-title = 组织仪表板

# semantic i18n key (#2542)
admin-dashboard-organization-subtitle = 组织活动、成员和系统运行状况概览

# semantic i18n key (#2542)
admin-dashboard-team-members = 团队成员

# semantic i18n key (#2542)
admin-dashboard-system-uptime = 系统正常运行时间

# semantic i18n key (#2542)
admin-dashboard-api-server = API 服务器

# semantic i18n key (#2542)
admin-dashboard-operational = 运行正常

# semantic i18n key (#2542)
admin-dashboard-latency-45ms = 延迟：45ms

# semantic i18n key (#2542)
admin-dashboard-requests-1-2k-min = 请求：1.2K/分钟

# semantic i18n key (#2542)
admin-dashboard-connections-45-100 = 连接数：45/100

# semantic i18n key (#2542)
admin-dashboard-query-time-12ms = 查询时间：12ms

# semantic i18n key (#2542)
admin-dashboard-vector-database = 向量数据库

# semantic i18n key (#2542)
admin-dashboard-vectors-2-4m = 向量：2.4M

# semantic i18n key (#2542)
admin-dashboard-search-8ms = 搜索：8ms

# semantic i18n key (#2542)
admin-dashboard-tokens-125k-min = 令牌：125K/分钟

# semantic i18n key (#2542)
admin-dashboard-latency-850ms = 延迟：850ms

# semantic i18n key (#2542)
admin-dashboard-high-usage = 使用率高

# semantic i18n key (#2542)
admin-dashboard-used-34-5gb = 已用：34.5GB

# semantic i18n key (#2542)
admin-dashboard-iops-2-1k = IOPS：2.1K

# semantic i18n key (#2542)
admin-dashboard-members-filter = 成员

# semantic i18n key (#2542)
admin-dashboard-customer-support-bot = 客户支持机器人

# semantic i18n key (#2542)
admin-dashboard-security-alert = 安全警报：

# semantic i18n key (#2542)
admin-dashboard-customer-feedback-survey = 客户反馈调查

# semantic i18n key (#2542)
admin-dashboard-sales-assistant = 销售助理

# semantic i18n key (#2542)
admin-dashboard-load-more-activity = 加载更多活动

# semantic i18n key (#2542)
admin-dashboard-john-smith = John Smith

# semantic i18n key (#2542)
admin-dashboard-role-distribution = 角色分布

# semantic i18n key (#2542)
admin-dashboard-manage-roles = 管理角色

# semantic i18n key (#2542)
admin-dashboard-bot-status = 机器人状态

# semantic i18n key (#2542)
admin-dashboard-manage-bots = 管理机器人

# semantic i18n key (#2542)
admin-dashboard-handles-customer-inquiries = 处理客户咨询

# semantic i18n key (#2542)
admin-dashboard-product-recommendations = 产品推荐

# semantic i18n key (#2542)
admin-dashboard-hr-assistant = 人力资源助理

# semantic i18n key (#2542)
admin-dashboard-employee-questions = 员工问题

# semantic i18n key (#2542)
admin-dashboard-it-helpdesk = IT 服务台

# semantic i18n key (#2542)
admin-dashboard-technical-support = 技术支持

# semantic i18n key (#2542)
admin-dashboard-pending-invitations = 待处理邀请

# semantic i18n key (#2542)
admin-dashboard-loading-invitations = 正在加载邀请…

# semantic i18n key (#2542)
admin-dashboard-invite-member = 邀请成员

# semantic i18n key (#2542)
admin-dashboard-bulk-invite = 批量邀请

# semantic i18n key (#2542)
admin-dashboard-emails-label = 电子邮件（每行一个，以逗号或分号分隔）

# semantic i18n key (#2542)
admin-dashboard-manage-users = 管理用户

# semantic i18n key (#2542)
admin-dashboard-roles-permissions = 角色和权限

# semantic i18n key (#2542)
workspace-placeholder-untitled = 无标题

# semantic i18n key (#2542)
admin-dashboard-placeholder-invite-email = teammate@example.com
admin-dashboard-placeholder-bulk-emails = a@example.com
b@example.com
