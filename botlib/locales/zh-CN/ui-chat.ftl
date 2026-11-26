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
chat-consent-affected-count = 受影响的记录
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

# Model attribution on each bot answer
chat-model-written-by = 答案由以下模型撰写：

# Assistant parity in chat
assistant-parity-block-allow = 允许
assistant-parity-block-allowed = 已允许
assistant-parity-block-api-command = API 命令
assistant-parity-block-attachment-read = 已读取附件
assistant-parity-block-chars = 字符
assistant-parity-block-command = 命令
assistant-parity-block-decision-failed = 无法记录决定
assistant-parity-block-denied = 已拒绝
assistant-parity-block-deny = 拒绝
assistant-parity-block-detail = 详情
assistant-parity-block-drive-list = Drive 列表
assistant-parity-block-drive-read = 已读取 Drive 文件
assistant-parity-block-drive-search = Drive 搜索
assistant-parity-block-elapsed = 已用时
assistant-parity-block-error = 错误
assistant-parity-block-file = 文件
assistant-parity-block-kb-search = 知识搜索
assistant-parity-block-notice = 通知
assistant-parity-block-path = 路径
assistant-parity-block-permission = 请求权限
assistant-parity-block-query = 查询
assistant-parity-block-read = 读取
assistant-parity-block-reason = 原因
assistant-parity-block-result = 结果
assistant-parity-block-retry = 重试
assistant-parity-block-seconds = 秒
assistant-parity-block-size = 大小
assistant-parity-block-status-done = 已完成
assistant-parity-block-status-failed = 失败
assistant-parity-block-status-needs-you = 需要您输入
assistant-parity-block-status-running = 运行中
assistant-parity-block-step = 步骤
assistant-parity-block-thinking = 思考中
assistant-parity-block-tool = 工具
assistant-parity-block-tool-run = 工具运行
assistant-parity-block-total = 总计
assistant-parity-block-truncated = 已截断
assistant-parity-block-type = 类型
assistant-parity-block-yes = 是
chat-parity-attach-title = 附件预览
chat-parity-attach-unavailable = 内容不可用 - 请使用“下载”打开此文件。
chat-parity-attach-download-failed = 无法下载附件。
chat-parity-model-tiers = 自动与档位
chat-parity-model-accounts = 提供商账户
chat-parity-model-local = 本地与免费
chat-drive-parity-attaching = 正在附加...
chat-drive-parity-cancel = 取消
chat-drive-parity-delete = 删除
chat-drive-parity-deleted = 已删除
chat-drive-parity-delete-failed = 无法删除文件。
chat-drive-parity-delete-question = 删除此文件？
chat-drive-parity-delete-title = 删除文件
chat-drive-parity-deleting = 正在删除...
chat-drive-parity-download = 下载
chat-drive-parity-download-failed = 无法下载文件。
chat-drive-parity-downloading = 正在下载...
chat-drive-parity-more = 更多操作
chat-drive-parity-new-name = 新名称
chat-drive-parity-preview = 预览
chat-drive-parity-preview-failed = 无法预览文件。
chat-drive-parity-previewing = 正在打开预览...
chat-drive-parity-rename = 重命名
chat-drive-parity-renamed = 已重命名
chat-drive-parity-rename-failed = 无法重命名文件。
chat-drive-parity-rename-title = 重命名文件
chat-drive-parity-renaming = 正在重命名...
chat-drive-parity-unavailable = 此部署中不可用此操作。
chat-drive-parity-use-in-chat = 在对话中使用
chat-parity-agent-does = 由代理执行
chat-parity-back = 返回
chat-parity-checked-now = 刚刚检查
chat-parity-close = 关闭
chat-parity-coming-soon = 即将推出
chat-parity-cost-contract = 按合同而定
chat-parity-cost-free = 免费
chat-parity-cost-free-limit = 免费，有使用限制
chat-parity-cost-sub = 您的订阅
chat-parity-cuda-no = 未检测到合适 GPU
chat-parity-cuda-yes = 已检测到显卡
chat-parity-depends-agent = 取决于代理
chat-parity-driver = 驱动程序
chat-parity-engine = 正在使用的引擎
chat-parity-feature-docs = 文档
chat-parity-feature-ocr = 图片中的文字
chat-parity-feature-photos = 照片和图表
chat-parity-feature-web = 网络搜索
chat-parity-features = 已启用功能
chat-parity-gains = 您的收益
chat-parity-go-agent = 转到代理提供商
chat-parity-go-ext = 转到外部提供商
chat-parity-go-net = 留在组织网络内
chat-parity-go-server = 转到您设置的服务器
chat-parity-guide = 不知道选哪个？
chat-parity-guide-sub = 回答两个问题。
chat-parity-in-use = 使用中
chat-parity-load-failed = 无法加载此信息。
chat-parity-loading = 加载中...
chat-parity-machine = 此计算机
chat-parity-machine-ok = 此计算机：%d，%v GB
chat-parity-machine-slow = 此计算机没有合适的显卡：运行会很慢
chat-parity-need-acp = 安装代理并登录您的账户
chat-parity-need-free = 无
chat-parity-need-local = 6 GB 或以上显卡及 4–6 GB 下载
chat-parity-need-own = 服务器地址和密钥
chat-parity-no-read = 不读取
chat-parity-no-work = 不工作
chat-parity-none-found = 未找到
chat-parity-not-yet = 尚未
chat-parity-only-net = 仅在您的网络中有服务器时
chat-parity-optional = 可选，由您开启
chat-parity-pick = 请选择一个选项
chat-parity-priv-ext = 您的对话将发送到外部主机（%h）。
chat-parity-priv-local = 您的对话留在此计算机上。
chat-parity-priv-net = 您的对话将发送到组织服务器（%h）。
chat-parity-privacy = 隐私
chat-parity-q1 = 您写的内容可以离开此计算机吗？
chat-parity-q1-help = 请考虑机密或个人的信息。
chat-parity-q1-no = 不，必须留在这里
chat-parity-q1-no-desc = 仅接受本地处理。
chat-parity-q1-yes = 是，可以离开
chat-parity-q1-yes-desc = 外部服务可能会处理它。
chat-parity-q2-no = 不，或不知道
chat-parity-q2-no-desc = 没有公司服务器或密钥。
chat-parity-q2-open = 您的组织是否提供 AI 服务器或密钥？
chat-parity-q2-open-help = 不确定请咨询 IT。
chat-parity-q2-secret = 您的组织是否允许外部 AI？
chat-parity-q2-secret-help = 某些策略禁止外部服务。
chat-parity-q2-secret-no-desc = 不允许外部 AI。
chat-parity-q2-secret-yes-desc = 允许外部 AI。
chat-parity-q2-yes = 是
chat-parity-q2-yes-desc = 有公司服务器或密钥。
chat-parity-reads = 读取
chat-parity-ready = 就绪
chat-parity-rec-free = 从免费模型开始：无需配置。涉及机密内容请优先使用本地。
chat-parity-rec-local = 使用本地：信息不外泄且此机器有显卡。
chat-parity-rec-local-slow = 此机器没有适合本地模型的显卡。请使用组织网络内的服务器。
chat-parity-rec-local-tight = 本地模型可用，但此显卡可能较慢。
chat-parity-rec-own = 使用组织的服务器或密钥：费用和限制由合同决定。
chat-parity-rec-own-net = 为保护数据，请使用组织网络内的服务器。
chat-parity-restart = 重新开始
chat-parity-retry = 重试
chat-parity-row-cost = 费用
chat-parity-row-docs = 文档
chat-parity-row-needs = 所需条件
chat-parity-row-ocr = 图片中的文字
chat-parity-row-offline = 无需网络
chat-parity-row-photos = 照片和图表
chat-parity-row-privacy = 您的文本
chat-parity-row-web = 网络搜索
chat-parity-rule = 规则
chat-parity-show-me = 查看如何启用
chat-parity-stay-here = 留在此计算机上
chat-parity-step = 问题 %n / 2
chat-parity-sub-acp = 您的代理订阅
chat-parity-sub-free = 随时可用
chat-parity-sub-local = 私密且离线
chat-parity-sub-own = 您组织的
chat-parity-suggested = 推荐
chat-parity-system-memory = 系统内存
chat-parity-verdict-bad-gpu = 无合适显卡
chat-parity-verdict-bad-gpu-long = 未找到支持 CUDA 的 NVIDIA 显卡。本地模型将在 CPU 上运行，速度慢得多：一个回答可能需要几分钟。要让文本留在网络内，请使用组织服务器。
chat-parity-verdict-capacity = 此显卡可运行需要最多 %v 显存的模型。
chat-parity-verdict-good = 运行流畅
chat-parity-verdict-low-vram = 显存不足的显卡（%v）
chat-parity-verdict-low-vram-long = 此显卡有 %v 显存；最小的本地模型需要 %m。
chat-parity-verdict-mid = 勉强可用
chat-parity-verdict-none = 无法检查此计算机
chat-parity-video-memory = 显存
chat-parity-where = 选择引擎
chat-parity-where-acp = 代理 (ACP)
chat-parity-where-free = 免费模型
chat-parity-where-local = 在此计算机上
chat-parity-where-own = 自有服务器或密钥
chat-parity-where-sub = 比较使用助手的四种方式。点击可查看详情并启用。
chat-parity-works = 可用
chat-parity-workshop = 助手工作台
chat-parity-workshop-lead = 调整助手在哪里以及如何思考。
chat-parity-workshop-title = 助手工作台
chat-parity-graphics-card = 显卡

# --- #2921 missing keys added to every locale catalogue ---
chat-confirm-title = 确认操作
chat-confirm-unavailable = 无法确认
chat-copy = 复制
chat-delete = 删除
chat-delete-message-confirm = 从对话中删除此消息？
chat-delete-title = 删除消息
chat-edit = 编辑
chat-edit-unavailable = 输入框不可用
chat-message-actions = 消息操作
chat-regenerate = 重新生成
chat-regenerate-no-source = 没有可重新生成的上一条用户消息
chat-regenerate-unavailable = 无法重新生成此轮
chat-voice-capture-silent = 未捕获到音频
chat-voice-duplex-audio = 音频
chat-voice-duplex-closed = 语音会话已关闭
chat-voice-duplex-connect = 连接语音
chat-voice-duplex-connected = 语音已连接
chat-voice-duplex-disabled = 语音已禁用
chat-voice-duplex-error = 语音错误
chat-voice-duplex-fallback = 回退到文本
chat-voice-duplex-live = 实时语音
chat-voice-duplex-lost = 连接已断开
chat-voice-duplex-timeout = 语音会话超时
chat-voice-duplex-unavailable = 语音不可用
chat-voice-mic-failed = 麦克风不可用
chat-web-title = 网页搜索
voice-mode-connecting = 连接中...
voice-mode-fallback-idle = 空闲
voice-mode-fallback-listening = 正在聆听
voice-mode-fallback-speaking = 正在说话
voice-mode-fallback-thinking = 正在思考
voice-mode-live-listening = 正在聆听
voice-mode-live-ready = 就绪
voice-mode-live-speaking = 正在说话

# gbgov assistant parity: busy-send and actionable engine refusals (#2563)
chat-busy-send-head = 回答仍在生成中
chat-busy-send-text = 请等待回答完成，或停止当前回答并立即发送你的消息。
chat-busy-send-wait = 继续等待
chat-busy-send-stop = 停止并发送
chat-engine-error-not-linked = 你的 %label 账户尚未连接。请选择其他模型，或在设置中连接账户。
chat-engine-error-switch-off = 此对话中需要打开 %label 的“代理”开关。
chat-engine-error-unavailable = 此计算机上无法使用订阅代理。请选择其他模型。
chat-engine-error-model-unavailable = 模型 %model 在 %label 上不可用。请选择其他模型。
chat-engine-error-workspace-failed = 无法启动代理的虚拟机。请检查其状态后重试。
chat-engine-error-account-model-off = 模型 %model 未对 %label 启用。请选择其他模型。
chat-engine-error-free-unavailable = 免费模型当前不可用。请稍后重试或选择其他模型。
chat-engine-error-limit-reached = 你已达到 %label 套餐的使用上限。%when 请选择其他模型。
chat-engine-error-unsafe-version = %label（%version）版本因在操作前未请求许可而被阻止。请选择其他模型。
chat-engine-error-choose-model = 选择其他模型
chat-engine-error-back-at = 将于 %when 恢复。
