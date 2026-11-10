# =============================================================================
# General Bots — zh-CN — Attendant (support queue)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# ATTENDANT / CRM SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# CRM Disabled State
# -----------------------------------------------------------------------------
attendant-crm-disabled = CRM 功能未启用
attendant-crm-disabled-desc = 话务员控制台需要为此机器人启用 CRM 功能。这允许人工代理接收并响应从机器人传输的对话。
attendant-crm-enable-instruction = 要启用 CRM 功能，请将此行添加到您的机器人的
attendant-crm-config-file = 配置.csv
attendant-crm-create-attendant = 然后创建一个
attendant-crm-attendant-file = 服务员.csv
attendant-crm-configure-team = 文件来配置您的团队

# -----------------------------------------------------------------------------
# Queue Sidebar
# -----------------------------------------------------------------------------
attendant-title = 话务台
attendant-status-online = 在线
attendant-status-busy = 忙
attendant-status-away = 离开
attendant-status-offline = 离线
attendant-status-ready = 在线 - 准备对话
attendant-status-busy-msg = 忙碌 - 处理对话
attendant-status-away-msg = 离开 - 很快就会回来
attendant-status-offline-msg = 离线 - 不可用

# -----------------------------------------------------------------------------
# Queue Stats
# -----------------------------------------------------------------------------
attendant-waiting = 等待
attendant-active = 活跃
attendant-resolved = 已解决
attendant-mine = 我的

# -----------------------------------------------------------------------------
# Queue Filters
# -----------------------------------------------------------------------------
attendant-filter-all = 全部
attendant-filter-waiting = 等待
attendant-filter-mine = 我的
attendant-filter-priority = 优先级

# -----------------------------------------------------------------------------
# Conversation List
# -----------------------------------------------------------------------------
attendant-no-conversations = 队列中没有对话
attendant-new-conversations-appear = 新的对话将出现在这里
attendant-unread = 未读
attendant-typing = 打字...
attendant-select-conversation = 选择一个对话
attendant-select-conversation-desc = 从队列中选择一个对话以开始响应

# -----------------------------------------------------------------------------
# Channel Tags
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = WhatsApp
attendant-channel-teams = 团队
attendant-channel-instagram = Instagram
attendant-channel-web = 网络
attendant-channel-telegram = 电报
attendant-channel-email = 电子邮件

# -----------------------------------------------------------------------------
# Priority Tags
# -----------------------------------------------------------------------------
attendant-priority-urgent = 紧急
attendant-priority-high = 高
attendant-priority-normal = 正常

# -----------------------------------------------------------------------------
# Chat Area
# -----------------------------------------------------------------------------
attendant-message-placeholder = 输入您的消息...
attendant-send = 发送
attendant-attach-file = 附加文件
attendant-insert-emoji = 插入表情符号
attendant-quick-responses = 快速响应
attendant-transfer = 转乘
attendant-resolve = 解决
attendant-more-actions = 更多行动

# -----------------------------------------------------------------------------
# Quick Responses
# -----------------------------------------------------------------------------
attendant-quick-greeting = 你好！今天我能为您提供什么帮助？
attendant-quick-thanks = 感谢您的耐心等待。
attendant-quick-checking = 让我帮你检查一下。
attendant-quick-moment = 请稍等一下。

# -----------------------------------------------------------------------------
# Transfer Modal
# -----------------------------------------------------------------------------
attendant-transfer-title = 转移对话
attendant-transfer-to = 转移至
attendant-transfer-reason = 原因（可选）
attendant-transfer-reason-placeholder = 你为什么要转移这个对话？
attendant-transfer-cancel = 取消
attendant-transfer-confirm = 转乘

# -----------------------------------------------------------------------------
# AI Insights Sidebar
# -----------------------------------------------------------------------------
attendant-ai-insights = 人工智能洞察
attendant-ai-summary = 谈话总结
attendant-ai-sentiment = 客户情绪
attendant-sentiment-positive = 积极
attendant-sentiment-neutral = 中性
attendant-sentiment-negative = 阴性
attendant-smart-replies = 智能回复
attendant-confidence = 信心
attendant-source = 来源

# -----------------------------------------------------------------------------
# Customer Details
# -----------------------------------------------------------------------------
attendant-customer-details = 客户详情
attendant-customer-name = 名称
attendant-customer-email = 电子邮件
attendant-customer-phone = 电话
attendant-customer-location = 地点
attendant-customer-tags = 标签

# -----------------------------------------------------------------------------
# Conversation History
# -----------------------------------------------------------------------------
attendant-history = 历史
attendant-history-resolved = 已解决
attendant-history-transferred = 已转入
attendant-history-abandoned = 被遗弃
attendant-view-history = 查看完整历史记录

# -----------------------------------------------------------------------------
# Toast Messages
# -----------------------------------------------------------------------------
attendant-toast-transferred = 对话转移成功
attendant-toast-resolved = 对话标记为已解决
attendant-toast-assigned = 分配给您的对话
attendant-toast-error = 发生错误
attendant-toast-connection-lost = 连接丢失。正在重新连接...
attendant-toast-connection-restored = 连接已恢复
