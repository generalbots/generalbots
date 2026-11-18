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
