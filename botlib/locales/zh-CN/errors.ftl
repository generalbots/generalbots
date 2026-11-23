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

error-locale-invalid = 请求的语言无效。
error-locale-unsupported = 此语言不可用。
error-locale-persist-failed = 无法保存您的语言偏好。
error-bot-id-required = 机器人 ID 是必需的。
error-bot-id-invalid = 机器人 ID 无效。
error-bot-name-required = 机器人名称是必需的。
error-token-invalid-format = 令牌格式无效。
error-token-signing-failed = 无法签署令牌。
error-token-signature-invalid = 令牌签名无效。
error-token-payload-invalid = 令牌负载无效。
error-token-payload-json-invalid = 令牌负载不是有效的 JSON。
error-cloud-token-required = 请提供 cloud_token。
error-llm-subscription-token-missing = 缺少订阅令牌。
error-llm-token-store-unavailable = 令牌存储不可用。
error-llm-subscription-token-expired = 订阅令牌已过期;请重新发送消息。
error-llm-prompt-missing = 请求中没有提示词。
error-llm-account-unlinked = 请先在设置 → AI 账户中连接您的 Claude 或 ChatGPT 账户。
attendance-no-attendant-matching-skills = No attendant was found with the required skills.
attendance-response-sent = Response sent.
error-agent-drive-bad-bot = The bot record for this conversation is not valid.
error-agent-drive-bad-owner = The conversation owner record is not valid.
error-agent-drive-bad-path = The Drive path returned by the store is not valid.
error-agent-drive-conversation-not-found = Conversation not found.
error-agent-drive-database-unavailable = The file store is temporarily unavailable.
error-agent-drive-file-missing = The file is not in the workspace output folder.
error-agent-drive-forbidden = This conversation belongs to another user.
error-agent-drive-link-failed = Could not create the share link.
error-agent-drive-lookup-failed = Could not read the conversation. Please try again.
error-agent-drive-named-files-missing = The requested files are not in the workspace output folder.
error-agent-drive-no-files = There are no files in the workspace output folder.
error-agent-drive-save-failed = Could not save any file to Drive.
error-agent-drive-save-one-failed = Could not save the file to Drive.
error-agent-drive-workspace-stopped = The agent workspace is not running.
error-agent-vm-disabled = Agent/VM is disabled for this conversation. Turn it on in the chat controls to use the workspace.
error-ai-account-agent-invalid = The agent must be reasoning, agentic or fast.
error-ai-account-not-connected-chat = Connect the account before selecting it for chat.
error-ai-account-not-connected-vibe = Connect the account before assigning it to Vibe.
error-ai-account-unknown-provider = The AI account provider is unknown.
error-ai-account-workspace-unavailable = The agent workspace is not available in this build.
error-attendance-attendant-id-required = An attendant id is required.
error-attendance-session-id-invalid = The session id is invalid.
error-attendance-session-not-found = Session not found.
error-attendance-webhook-not-found = Webhook not found.
error-autotask-fields-required = Title, schedule and command are required.
error-autotask-intent-required = Describe what the task should do.
error-autotask-kind-invalid = The kind must be 'cron' or 'systemd'.
error-autotask-schedule-invalid = The schedule must be a valid five-field cron expression.
error-autotask-not-found = Autotask not found.
error-collab-comment-body-required = A comment body is required.
error-collab-comment-not-found = The comment was not found.
error-collab-comment-resolve-forbidden = The comment was not found, or it is not yours to resolve.
error-collab-comment-too-long = The comment is limited to 8000 characters.
error-collab-content-required = Content is required.
error-collab-content-too-large = The content is too large.
error-collab-grant-not-found = The grant was not found.
error-collab-invalid-action = The action is invalid.
error-collab-invalid-base-version = The base version is invalid.
error-collab-invalid-cursor = The 'before' cursor is invalid.
error-collab-invalid-emoji = The emoji is invalid.
error-collab-invalid-grantee-id = The grantee id is invalid.
error-collab-invalid-grantee-type = The grantee type must be 'user' or 'domain'.
error-collab-invalid-name = The name is invalid.
error-collab-invalid-new-owner = The new owner id is invalid.
error-collab-invalid-op-type = The operation type is invalid.
error-collab-invalid-parent = The parent id is invalid.
error-collab-invalid-resolution = The resolution must be 'accept-server' or 'accept-client'.
error-collab-invalid-resource = The resource type or resource id is invalid.
error-collab-invalid-role = The role is invalid.
error-collab-link-not-found = The share link was not found.
error-collab-not-a-conflict = The operation is not an unresolved conflict.
error-collab-operation-not-found = The operation was not found.
error-collab-owner-only = Only the owner can do this.
error-collab-version-not-found = The version was not found.
error-computer-use-action-id-mismatch = The action id does not match the path.
error-computer-use-action-not-found = Action not found.
error-computer-use-browser-only = This action must run in the browser panel.
error-computer-use-kind-invalid = The kind must be 'browser' or 'terminal'.
error-computer-use-no-consent = There is no consent record for this action; approve it first.
error-computer-use-session-required = A session id is required.
error-computer-use-url-missing = The action has no URL.
error-conversation-not-found = The conversation was not found.
error-conversation-session-invalid = The conversation session is invalid.
error-db-unavailable = The database is unavailable right now. Please try again.
error-drive-sync-conflict-path-escape = The conflict path escapes its sync folder.
error-drive-sync-conflict-path-required = A conflict path is required.
error-drive-sync-field-absolute = { $field } must be an absolute path.
error-drive-sync-field-drive-folder = The Drive folder
error-drive-sync-field-local-folder = The local folder
error-drive-sync-field-not-folder = { $field } is not a folder on this machine: { $path }
error-drive-sync-field-parent = { $field } must not contain '..'.
error-drive-sync-field-required = { $field } is required.
error-drive-sync-job-exists = A sync job named { $name } already exists.
error-drive-sync-job-name-required = A sync job name is required.
error-drive-sync-job-not-found = Sync job { $id } was not found.
error-drive-sync-task-failed = The { $task } task failed: { $error }
error-file-archive-unsafe = The archive was refused: { $reason }.
error-file-content-mismatch = The file content does not match its type ({ $expected } expected, { $detected } found).
error-file-empty = The uploaded file is empty.
error-file-extension-blocked = Files with the .{ $extension } extension are not allowed.
error-file-extension-unknown = The .{ $extension } extension is not recognised.
error-file-name-invalid = The file name is not valid.
error-goal-run-advance-refused = The run cannot advance in its current state.
error-goal-run-not-paused = This goal run is not paused.
error-goal-run-not-running = This goal run is not running.
error-group-budget-amount-invalid = The amount must be zero or positive.
error-group-budget-caps-invalid = The caps must be zero or positive (zero means no limit).
error-group-budget-invalid-id = The budget id is invalid.
error-group-budget-not-found = The group budget was not found.
error-http-no-route = No route matches this path.
error-invalid-value = The supplied value could not be parsed.
error-org-auth-service-unavailable = The directory service is not available.
error-org-default-delete = The default organization cannot be deleted.
error-org-name-required = The organization name is required.
error-org-settings-save-failed = Could not save the organization settings.
error-org-tenant-missing = No tenant exists yet — run the migrations or the bootstrap first.
error-peer-network-disabled = The peer network is disabled on this node.
error-peer-network-forbidden = An administrator session is required for this peer control.
error-peer-network-invalid = The peer request is not valid: { $message }
error-peer-network-payload-store-full = The peer payload store is full: { $message }
error-peer-network-recipient-required = The recipient query parameter is required.
error-peer-network-registry-full = The peer registry is full.
error-peer-network-unauthorized = A valid peer bearer token is required.
error-peer-network-unconfigured = This node has no peer token configured, so peer endpoints are closed.
error-peer-network-unknown-node = Unknown peer node: { $message }
error-people-invalid-end-date = The end date is invalid. Use the YYYY-MM-DD format.
error-people-invalid-start-date = The start date is invalid. Use the YYYY-MM-DD format.
error-person-not-found = Person not found.
error-product-not-found = Product not found.
error-service-invalid-value = The service value is invalid.
error-service-not-found = Service not found.
error-service-price-required = This service has no price; provide a value.
error-subagent-invalid-tasks = The sub-agent tasks are invalid.
error-subagent-no-llm = No LLM provider is configured for sub-agents.
error-team-not-found = Team not found.
error-terminal-action-id-required = Terminal execution requires an approved action id.
error-terminal-approval-required = Terminal execution requires explicit user approval.
error-terminal-blocked-pattern = The command matches a blocked destructive pattern.
error-terminal-command-required = A command is required.
error-terminal-command-too-long = The command is limited to 16000 characters.
error-terminal-timeout = The command timed out.
error-terminal-workspace-stopped = The agent workspace is not running. Finish provisioning (or restart the desktop app), then retry.
error-time-off-not-found = Time-off request not found.
error-value-required = A value is required for this service.
