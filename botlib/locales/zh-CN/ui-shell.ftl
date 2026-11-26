# =============================================================================
# General Bots — zh-CN — Desktop shell, navigation, controls
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# General Bots - English UI Translations
# =============================================================================
# -----------------------------------------------------------------------------
# Navigation
# -----------------------------------------------------------------------------
nav-home = 首页
nav-chat = 聊天
nav-drive = 驱动器
nav-tasks = 任务
nav-mail = 邮件
nav-calendar = 日历
nav-meet = 见面
nav-paper = 纸
nav-video = 视频
nav-research = 研究
nav-analytics = 分析
nav-settings = 设置
nav-admin = 管理员
nav-monitoring = 监控
nav-sources = 来源
nav-tools = 工具
nav-attendant = 服务员
nav-learn = 学习
nav-crm = 客户关系管理
nav-billing = 计费
nav-products = 产品展示
nav-tickets = 门票
nav-docs = 文档
nav-sheet = 床单
nav-slides = 幻灯片
nav-social = 社交
nav-all-apps = 所有应用程序
nav-people = 人
nav-editor = 编辑
nav-dashboards = 仪表板
nav-security = 安全性
nav-designer = 设计师
nav-project = 项目
nav-canvas = 帆布
nav-goals = 目标
nav-player = 玩家
nav-workspace = 工作空间

# -----------------------------------------------------------------------------
# Dashboard
# -----------------------------------------------------------------------------
dashboard-title = 仪表板
dashboard-welcome = 欢迎回来，{ $name }！
dashboard-quick-actions = 快速行动
dashboard-recent-activity = 最近的活动
dashboard-no-activity = 最近还没有活动。开始探索吧！
dashboard-analytics = 分析

# -----------------------------------------------------------------------------
# Quick Actions
# -----------------------------------------------------------------------------
quick-start-chat = 开始聊天
quick-upload-files = 上传文件
quick-new-task = 新任务
quick-compose-email = 撰写电子邮件
quick-start-meeting = 开始会议
quick-new-event = 新活动

# -----------------------------------------------------------------------------
# Application Cards
# -----------------------------------------------------------------------------
app-chat-name = 聊天
app-chat-desc = 人工智能驱动的对话。提出问题、获取帮助并自动执行任务。
app-drive-name = 驱动器
app-drive-desc = 您所有文件的云存储。上传、整理和分享。
app-tasks-name = 任务
app-tasks-desc = 保持待办事项列表、优先事项和截止日期的井井有条。
app-mail-name = 邮件
app-mail-desc = 具有人工智能辅助写作和智能组织功能的电子邮件客户端。
app-calendar-name = 日历
app-calendar-desc = 安排会议、活动并有效管理您的时间。
app-meet-name = 见面
app-meet-desc = 具有屏幕共享和实时转录功能的视频会议。
app-paper-name = 纸
app-paper-desc = 在人工智能的帮助下编写文档。笔记、报告等。
app-research-name = 研究
app-research-desc = 由人工智能驱动的跨所有来源的搜索和发现。
app-analytics-name = 分析
app-analytics-desc = 用于跟踪使用情况和见解的仪表板和报告。

# -----------------------------------------------------------------------------
# Suite Header
# -----------------------------------------------------------------------------
suite-title = 通用机器人套件
suite-tagline = 您的人工智能驱动的生产力工作空间。聊天、协作和创造。
suite-new-intent = 新意图

# -----------------------------------------------------------------------------
# AI Panel
# -----------------------------------------------------------------------------
ai-developer = 人工智能开发者
ai-developing = 开发中：{ $project }
ai-quick-actions = 快速行动
ai-add-field = 添加字段
ai-change-color = 改变颜色
ai-add-validation = 添加验证
ai-export-data = 导出数据
ai-placeholder = 输入您的修改...
ai-thinking = AI正在思考...
ai-status-online = 在线
ai-status-offline = 离线

# -----------------------------------------------------------------------------
# Search
# -----------------------------------------------------------------------------
search-placeholder = 搜索...
search-no-results = 没有找到结果
search-results = { $计数 ->
    [one] { $count } result
   *[other] { $count } results
}
search-in-progress = 正在寻找...
search-advanced = 高级搜索
search-filters = 过滤器
search-clear-filters = 清除过滤器

# -----------------------------------------------------------------------------
# Pagination
# -----------------------------------------------------------------------------
# -----------------------------------------------------------------------------
# Tables
# -----------------------------------------------------------------------------
table-no-data = 无可用数据
table-loading = 正在加载数据...
table-actions = 行动
table-select-all = 选择全部
table-deselect-all = 取消全选
table-export = 出口
table-import = 进口

# -----------------------------------------------------------------------------
# Forms
# -----------------------------------------------------------------------------
form-required = 必填
form-optional = 可选
form-submit = 提交
form-reset = 重置
form-clear = 清除
form-uploading = 正在上传...
form-processing = 处理中...

# -----------------------------------------------------------------------------
# Modals / Dialogs
# -----------------------------------------------------------------------------
modal-confirm-title = 确认操作
modal-confirm-message = 您确定要继续吗？
modal-delete-title = 删除确认
modal-delete-message = 此操作无法撤消。你确定吗？

# -----------------------------------------------------------------------------
# Tooltips
# -----------------------------------------------------------------------------
tooltip-copy = 复制到剪贴板
tooltip-copied = 复制了！
tooltip-expand = 展开
tooltip-collapse = 崩溃
tooltip-refresh = 刷新
tooltip-download = 下载
tooltip-upload = 上传
tooltip-print = 打印
tooltip-fullscreen = 全屏
tooltip-exit-fullscreen = 退出全屏

# -----------------------------------------------------------------------------
# Common Labels (additional)
# -----------------------------------------------------------------------------
label-username = 用户名
label-email = 电子邮件
label-display-name = 显示名称
label-password = 密码
label-role = 角色
label-group-name = 群组名称
label-hostname = 主机名
label-record-type = 记录类型
label-target = 目标
label-your-name = 你的名字

# -----------------------------------------------------------------------------
# Actions (additional)
# -----------------------------------------------------------------------------
action-register = 注册

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
notifications-title = 通知
notifications-clear = 全部清除
notifications-empty = 无通知

# #1853 — desktop chrome, launchers and notification settings.
window-ready = 就绪
window-loading = 正在加载应用…
window-minimize = 最小化
window-maximize = 最大化
window-fullscreen = 全屏
window-exit-fullscreen = 退出全屏
window-close = 关闭
app-loading = 应用加载中…
start-menu-search-placeholder = 输入以查找应用…
start-menu-recent = 最近使用
start-menu-no-apps = 未找到应用
start-menu-user = 用户
category-ai = AI
category-business = 业务
category-office = 办公
category-dev = 开发
category-system = 系统
category-tools = 工具
sidebar-apps = 应用
sidebar-apps-toggle = 显示/隐藏应用
sidebar-apps-toggle-aria = 切换应用面板
sidebar-apps-filter = 筛选应用…
sidebar-apps-filter-aria = 筛选应用
sidebar-preview = 预览模式
sidebar-preview-hint = 显示仍在预览中的应用
sidebar-preview-toggle = 切换预览模式
sidebar-app-preview-title = {app}（预览）
sidebar-apps-count = {count} 个应用
sidebar-apps-count-preview = {count}（+ {preview} 个预览）
spotlight-placeholder = 搜索应用、文件…或输入 Concierge 目标
spotlight-hint = ↑↓ 导航 · Enter 运行 · Ctrl+Space 或 Esc 关闭
spotlight-empty = 无结果
spotlight-goal = 运行目标：“{goal}”
spotlight-goal-subtitle = Concierge · 跨应用规划与执行
palette-placeholder = 搜索应用和命令。或输入 @ <命令> 发送给助手...
palette-empty = 没有匹配的应用或命令
palette-kind-app = 应用
palette-hint = ↑↓ 导航 · Enter 运行 · Esc 关闭 · Ctrl+K 切换
palette-open-app = 打开 {app}
palette-value-prompt = {name}：{summary} — {key} 的值：
palette-execute = 执行 {method} {path} 并总结结果。
palette-use-command = 使用 {command} 命令来 {summary}。
toast-dismiss = 关闭
settings-language-display = 显示语言
settings-language-display-desc = 选择应用界面语言
settings-language-regional = 区域设置
settings-language-regional-desc = 配置日期、时间和数字格式
settings-language-date-format = 日期格式
settings-language-time-format = 时间格式
settings-language-timezone = 时区
settings-language-first-day = 每周第一天
settings-language-time-12h = 12 小时制（1:30 PM）
settings-language-time-24h = 24 小时制（13:30）
settings-language-sunday = 星期日
settings-language-monday = 星期一
settings-email-notifications-desc = 由工作区通知发件人发送，绝不通过已连接的 Gmail/Outlook 邮箱发送。
settings-push-enable = 启用推送通知
settings-push-enable-desc = 接收浏览器推送通知
settings-push-sound = 声音
settings-push-sound-desc = 为通知播放声音
settings-desktop-notifications = 桌面通知
settings-desktop-notifications-desc = 在桌面显示通知
settings-badge-count = 徽标计数
settings-badge-count-desc = 显示未读计数徽标
settings-save-preferences = 保存首选项
settings-notifications-saved = 通知首选项已保存
settings-notification-recipient = 通知邮箱
settings-notification-recipient-desc = 留空则使用您的主 Mail 账户或个人资料邮箱。
language-search-placeholder = 搜索语言…
settings-profile-title = 配置文件设置
settings-security-title = 安全设置

# semantic i18n key (#2542)
apps-menu-placeholder-search = 搜索应用...
apps-menu-aria-search = 搜索应用
apps-menu-aria-close = 关闭应用菜单

# --- #2921 missing keys added to every locale catalogue ---
palette-kind-tour = 导览
palette-show-tour = 显示导览
palette-show-tour-hint = 显示产品导览
sidebar-app-hide = 从此栏隐藏 { $app }
sidebar-apps-count-off = { $count } 个已隐藏
sidebar-apps-empty = 没有可显示的应用
sidebar-apps-manage = 管理应用
sidebar-apps-manage-all = 显示全部
sidebar-apps-manage-close = 关闭
sidebar-apps-manage-empty = 没有可用的应用
sidebar-apps-manage-summary = 已显示 { $total } 个中的 { $on } 个
sidebar-apps-none = 没有应用
toast-open = 打开

# Assistant shell — knowledge-base source blocks
trecho = 摘录
trechos = 摘录

# Assistant/Drive shell keys integrated from the gbgov edition
arquivo = file
arquivos = files
assistant-agent-backend-off = Agent backend is off
assistant-agent-locked-hint = Agent is locked for this model
assistant-agent-locked-tip = Choose another model to use the agent
assistant-regenerate-older = Regenerating an older answer starts a new turn
caracteres = characters
corrigido = fixed
de = of
drive-actions = Actions
drive-folders = Folders
drive-usage-hint = Storage used
drive-used = Used
item = item
itens = items
linha = line
linhas = lines
resultado = result
resultados = results
shell-close = Close
shell-conversations = Conversations
shell-machine = Machine

# Assistant shell — literal keys
assistant-vm-wants = The agent wants to perform an action
assistant-vm-wants-command = The agent wants to run a command on the machine
drive-label = Drive

# Chat / assistant parity — VM consent and subscription-agent UI keys
chat-vm-consent-assistant = 助手
chat-vm-consent-subject = 代理
chat-vm-consent-wants = 代理想要执行操作
chat-vm-consent-wants-command = 代理想要运行命令
chat-vm-consent-kicker = 需要你的许可
chat-vm-consent-command-label = 命令
chat-vm-consent-where = 位置
chat-vm-consent-where-vm = 虚拟机
chat-vm-consent-where-drive = 你的 Drive
chat-vm-consent-destination = 目标
chat-vm-consent-language = 语言
chat-vm-consent-packages = 从互联网安装
chat-vm-consent-from = 来源
chat-vm-consent-background = 在后台运行
chat-vm-consent-repair = 程序的修正版本
chat-vm-consent-protected = 此请求涉及代理凭据，仅允许一次。
chat-vm-consent-all-note = 全部允许仅覆盖本次对话中该代理在虚拟机内的所有操作。
chat-vm-consent-outside-note = 此操作将离开虚拟机：全部允许不适用。
chat-vm-consent-allow-once = 仅允许一次
chat-vm-consent-allow-conversation = 在此对话中允许
chat-vm-consent-allow-all = 全部允许
chat-vm-consent-deny = 拒绝
chat-vm-consent-outcome-all = 此对话中的完全许可
chat-vm-consent-outcome-conversation = 已在此对话中允许
chat-vm-consent-outcome-once = 已允许一次
chat-vm-consent-outcome-denied = 你已拒绝：未执行任何操作
chat-vm-consent-outcome-failed = 无法记录你的决定
chat-vm-consent-forbidden = 无法在此记录决定。请求保持打开。
chat-vm-consent-announce = 许可请求已打开。按 Alt+P 回答。
chat-vm-consent-still-open = 许可请求仍未关闭。请选择一项；拒绝不会执行任何操作。
chat-vm-consent-waiting = 等待你的许可
chat-vm-all-chip = 完全许可
chat-vm-all-scope = 仅在虚拟机内
chat-vm-all-revoke = 撤销
chat-vm-all-revoked = 完全许可已撤销
chat-vm-send-head = 确认
chat-vm-send-text = 发送将取消未处理的许可请求。仍要发送吗？
chat-vm-send-back = 返回请求
chat-vm-send-go = 发送
chat-vm-engine-local-short = 本地
chat-vm-engine-local-group = 本地模型
chat-vm-engine-local-detail = 在此计算机上
chat-vm-engine-free-short = 免费
chat-vm-engine-free-group = 免费模型
chat-vm-engine-free-detail = 免费模型
chat-vm-engine-own-short = 自有密钥
chat-vm-engine-own-group = 自有密钥
chat-vm-engine-own-detail = 自有服务器或密钥
chat-vm-engine-sub-short = 订阅
chat-vm-engine-sub-group = 订阅
chat-vm-engine-sub-detail = 订阅代理
chat-vm-agent-locked-tip = 所选代理在虚拟机中运行；选择其他模型可将其关闭
chat-vm-agent-locked-note = 在虚拟机中运行，因此选中期间代理开关保持开启。选择其他模型可将其关闭。
