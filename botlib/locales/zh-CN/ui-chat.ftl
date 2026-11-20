# =============================================================================
# General Bots — zh-CN — Chat
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Chat
# -----------------------------------------------------------------------------
chat-title = 聊天
chat-placeholder = 输入您的消息...
chat-send = 发送
chat-new-conversation = 新的对话
chat-history = 聊天记录
chat-clear = 清除聊天
chat-export = 导出聊天记录
chat-typing = { $name } 正在输入...
chat-online = 在线
chat-offline = 离线
chat-last-seen = 最后出现{ $time }
chat-mention-title = 参考实体
chat-mention-placeholder = 消息...（输入 @ 提及）
chat-mention-search = 搜索实体...
chat-mention-no-results = 没有找到结果
chat-mention-type-hint = 输入： 进行搜索

# Additional Chat translations
chat-camera = 相机
chat-camera-title = 拍照
chat-camera-capture = 拍照
chat-camera-cancel = 取消
chat-camera-sent = 照片已发送。
chat-camera-denied = 相机权限被拒绝。请在系统设置中为桌面应用允许相机访问以拍照。
chat-camera-missing = 此设备上未找到相机。
chat-camera-error = 无法启动相机。
chat-camera-unsupported = 此桌面应用不提供相机访问。
voice-mode-title = 语音对话
voice-mode-listening = 正在聆听… 请自然说话
voice-mode-thinking = 思考中…
voice-mode-speaking = 正在说话…
voice-mode-idle = 语音对话
voice-mode-end = 结束语音对话
chat-message-placeholder = 留言...

# Chat message controls and notices
chat-copy-message = 复制消息
chat-copied = 已复制
chat-message-copied = 消息已复制
chat-copy-failed = 复制失败
chat-listen-tts = 朗读 (TTS)
chat-listen-message = 朗读此消息
chat-image-unavailable = 图片不可用
chat-open = 打开
chat-expand = 展开
chat-collapse = 收起
chat-preview = 预览
chat-close-preview = 关闭预览
chat-preview-load-failed = 无法加载预览。
chat-show-more = 显示更多
chat-show-less = 显示更少
chat-tool = 工具
chat-param = 参数
chat-params = 参数
chat-tool-result = 工具结果
chat-retry = 重试
chat-open-as-document = 作为文档打开
chat-show-more-history = 显示更多…
chat-disconnected-reconnecting = 聊天已断开连接。正在重新连接...
chat-connect-retrying = 无法连接到聊天服务器。正在重试...
chat-connection-lost = 连接丢失。正在尝试重新连接...
chat-attachment-sent = 附件已发送：
chat-attachments-sent = 附件已发送：
chat-message-not-delivered = 消息未送达 — 正在重新连接
chat-attachment-read-failed = 无法读取附件。请重新附加。
chat-attachment-queued = 连接即将恢复 — 附件已排队
chat-reconnecting-queued = 正在重新连接 — 连接恢复后将发送您的消息。
chat-bot-private-signin = 此机器人为私有 — 请登录以继续
chat-approved-plan = 已批准的计划：
chat-retrying-attachment = 正在重试附件
chat-expanded-preview = 展开的预览

# #2362 — chat code/artifact control labels
chat-code-show-all = Show all
chat-code-lines = lines
chat-artifact-save-drive = Save to Drive
chat-artifact-download-svg = Download SVG
chat-artifact-download-png = Download PNG
chat-artifact-open-canvas = Open in Canvas
chat-artifact-folder = Folder (optional)
chat-artifact-kind-markdown = Markdown document
chat-artifact-kind-html = HTML page
chat-artifact-kind-code = Code
chat-artifact-words = words

# #2362 — attachment action label
chat-open-in-app = Open in app
# Assistant-look chat: transcript export and answer feedback
chat-export-actions = 对话操作
chat-export-md = 将对话导出为 Markdown
chat-export-print = 打印或保存为 PDF
chat-export-done = 对话已导出
chat-export-empty = 暂无可导出的内容
chat-export-untitled = 对话
chat-export-user = 你
chat-export-assistant = 助手
chat-feedback-label = 回答反馈
chat-feedback-good = 回答有帮助
chat-feedback-bad = 回答无帮助
chat-feedback-comment = 哪里可以改进？
chat-feedback-send = 发送
chat-staged-files = 附件
chat-staged-close = 关闭预览
chat-camera-waiting = Waiting for the camera to start…
chat-camera-waiting-frames = Permission granted; waiting for video frames
chat-camera-ready = Camera ready. Take the photo or cancel.
chat-camera-started = Camera started — tap Take photo.
chat-camera-no-device = no video device
chat-camera-device = video device
chat-camera-no-frames = The camera preview did not start. Choose a photo from the system picker instead.
chat-camera-timeout = The camera did not answer. Choose a photo from the system picker instead.

# Chat engineer status strip (#2342)
chat-status-bot = 机器人
chat-status-session = 会话
chat-status-model = 模型
chat-status-state = 状态
chat-status-no-session = —
chat-status-copy-session = 复制会话 ID
chat-status-copied = 已复制
chat-status-idle = 空闲
chat-status-streaming = 流式输出
chat-status-thinking = 思考中
chat-status-error = 错误
chat-status-stopped = 已停止

chat-response-ready = 回复就绪
chat-response-started = 助手正在回复
chat-tool-result-ready = 工具结果可用

# semantic i18n key (#2542)
chat-no-suggestions-available = 没有可用建议

# semantic i18n key (#2542)
chat-start-conversation = 开始对话

# semantic i18n key (#2542)
chat-empty-hint = 在下方输入消息或使用语音输入

# semantic i18n key (#2542)
chat-no-conversations-yet = 暂无对话

# semantic i18n key (#2542)
chat-context-label = 上下文：

# semantic i18n key (#2542)
chat-select-context = 选择上下文...

# semantic i18n key (#2542)
chat-no-contexts-configured = 未配置上下文

# semantic i18n key (#2542)
projector-title = 内容查看器

# semantic i18n key (#2542)
projector-loading-content = 正在加载内容...

# semantic i18n key (#2542)
projector-slide-info = 幻灯片 1 / 1

# semantic i18n key (#2542)
projector-code-info = code.rs

# Projector viewer control (#2542)
chat-projector-rotate = 旋转

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
chat-empty-title = 开始对话
chat-empty-desc = 在下方输入消息，或选择机器人给出的建议。
chat-empty-action = 写第一条消息

# -----------------------------------------------------------------------------
# #2342 — chat engineer UX: slash commands, help, inspect/copy-markdown
# -----------------------------------------------------------------------------
chat-cmd-menu-title = Commands
chat-cmd-help = Show keyboard shortcuts
chat-cmd-new = Start a new conversation
chat-cmd-clear = Clear this conversation
chat-cmd-stop = Stop generating
chat-stop = 停止
chat-cmd-failed = Command failed
chat-help = Keyboard shortcuts
chat-inspect = Inspect
chat-inspect-title = Message details
chat-inspect-sender = Sender
chat-inspect-id = Message id
chat-inspect-no-id = none
chat-inspect-length = Characters
chat-copy-markdown = Copy as Markdown
chat-copied-markdown = Copied as Markdown
