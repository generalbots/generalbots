# General Bots - Error Messages (English)
# This file contains all error message translations

# =============================================================================
# HTTP Errors
# =============================================================================

error-http-400 = 错误的请求。请检查您的输入。
error-http-401 = 需要身份验证。请登录。
error-http-403 = 您无权访问此资源。
error-http-404 = { $entity } 未找到。
error-http-409 = 冲突：{ $message }
error-http-429 = 请求太多。请等待{ $seconds }秒。
error-http-500 = 内部服务器错误。请稍后重试。
error-http-502 = 坏网关。服务器收到无效响应。
error-http-503 = 服务暂时不可用。请稍后重试。
error-http-504 = 请求在 { $milliseconds }ms 后超时。

# =============================================================================
# Validation Errors
# =============================================================================

error-validation-required = { $field } 为必填项。
error-validation-email = 请输入有效的电子邮件地址。
error-validation-url = 请输入有效的网址。
error-validation-phone = 请输入有效的电话号码。
error-validation-min-length = { $field } 必须至少为 { $min } 个字符。
error-validation-max-length = { $field } 不得超过 { $max } 个字符。
error-validation-min-value = { $field } 必须至少为 { $min }。
error-validation-max-value = { $field } 不得超过 { $max }。
error-validation-pattern = { $field } 格式无效。
error-validation-unique = { $field }已经存在。
error-validation-mismatch = { $field } 与 { $other } 不匹配。
error-validation-date-format = 请以 { $format } 格式输入有效日期。
error-validation-date-past = { $field } 必须是过去式。
error-validation-date-future = { $field }一定是未来的。

# =============================================================================
# Authentication Errors
# =============================================================================

error-auth-invalid-credentials = 电子邮件或密码无效。
error-auth-account-locked = 您的帐户已被锁定。请联系支持人员。
error-auth-account-disabled = 您的帐户已被禁用。
error-auth-session-expired = 您的会话已过期。请重新登录。
error-auth-token-invalid = 令牌无效或过期。
error-auth-token-missing = 需要身份验证令牌。
error-auth-mfa-required = 需要多重身份验证。
error-auth-mfa-invalid = 验证码无效。
error-auth-password-weak = 密码太弱。请使用更强的密码。
error-auth-password-expired = 您的密码已过期。请重置它。

# =============================================================================
# Configuration Errors
# =============================================================================

error-config = 配置错误：{ $message }
error-config-missing = 缺少配置：{ $key }
error-config-invalid = { $key } 的配置值无效：{ $reason }
error-config-file-not-found = 未找到配置文件：{ $path }
error-config-parse = 解析配置失败：{ $message }

# =============================================================================
# Database Errors
# =============================================================================

error-database = 数据库错误：{ $message }
error-database-connection = 无法连接到数据库。
error-database-timeout = 数据库操作超时。
error-database-constraint = 数据库约束违规：{ $constraint }
error-database-duplicate = 具有此 { $field } 的记录已存在。
error-database-migration = 数据库迁移失败：{ $message }

# =============================================================================
# File & Storage Errors
# =============================================================================

error-file-not-found = 未找到文件：{ $filename }
error-file-too-large = 文件太大。最大尺寸为 { $maxSize }。
error-file-type-not-allowed = 不允许的文件类型。允许的类型：{ $allowedTypes }。
error-file-upload-failed = 文件上传失败：{ $message }
error-file-read = 读取文件失败：{ $message }
error-file-write = 写入文件失败：{ $message }
error-storage-full = 超出存储配额。
error-storage-unavailable = 存储服务不可用。

# =============================================================================
# Network & External Service Errors
# =============================================================================

error-network = 网络错误：{ $message }
error-network-timeout = 连接超时。
error-network-unreachable = 服务器无法访问。
error-service-unavailable = 服务不可用：{ $service }
error-external-api = 外部API错误：{ $message }
error-rate-limit = 速率有限。 { $seconds }s 后重试。

# =============================================================================
# Bot & Dialog Errors
# =============================================================================

error-bot-not-found = 未找到机器人：{ $botId }
error-bot-disabled = 该机器人目前已被禁用。
error-bot-script-error = 第{ $line }行的脚本错误：{ $message }
error-bot-timeout = 机器人响应超时。
error-bot-quota-exceeded = 超出机器人使用配额。
error-dialog-not-found = 找不到对话框：{ $dialogId }
error-dialog-invalid = 无效的对话框配置：{ $message }

# =============================================================================
# LLM & AI Errors
# =============================================================================

error-llm-unavailable = AI服务目前不可用。
error-llm-timeout = AI 请求超时。
error-llm-rate-limit = 超出 AI 速率限制。请稍等后再重试。
error-llm-content-filter = 内容已根据安全准则进行过滤。
error-llm-context-length = 输入太长。请缩短您的消息。
error-llm-invalid-response = 收到来自 AI 服务的无效响应。
error-llm-empty-response = 抱歉，我现在无法处理您的消息。请几秒钟后重试。

# =============================================================================
# Email Errors
# =============================================================================

error-email-send-failed = 发送邮件失败：{ $message }
error-email-invalid-recipient = 收件人电子邮件地址无效：{ $email }
error-email-attachment-failed = 附加文件失败：{ $filename }
error-email-template-not-found = 未找到电子邮件模板：{ $template }

# =============================================================================
# Calendar & Scheduling Errors
# =============================================================================

error-calendar-conflict = 时间段与现有活动冲突。
error-calendar-past-date = 无法安排过去的事件。
error-calendar-invalid-recurrence = 无效的重复模式。
error-calendar-event-not-found = 未找到活动：{ $eventId }

# =============================================================================
# Task Errors
# =============================================================================

error-task-not-found = 未找到任务：{ $taskId }
error-task-already-completed = 任务已经完成了。
error-task-circular-dependency = 在任务中检测到循环依赖。
error-task-invalid-status = 任务状态转换无效。

# =============================================================================
# Permission Errors
# =============================================================================

error-permission-denied = 您无权执行此操作。
error-permission-resource = 您无权访问此{ $resource }。
error-permission-action = 你不能{ $action }这个{ $resource }。
error-permission-owner-only = 只有所有者才能执行此操作。

# =============================================================================
# Generic Errors
# =============================================================================

error-internal = 内部错误：{ $message }
error-unexpected = 发生意外错误。请再试一次。
error-not-implemented = 该功能尚未实现。
error-maintenance = 系统正在维护中。请稍后重试。
error-unknown = 发生未知错误。
