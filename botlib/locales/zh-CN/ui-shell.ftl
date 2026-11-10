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
