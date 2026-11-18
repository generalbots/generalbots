# =============================================================================
# General Bots — zh-CN — Productivity apps (tasks, calendar, meet, mail, research, paper)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Tasks
# -----------------------------------------------------------------------------
tasks-title = 任务
tasks-new = 新任务
tasks-due-today = 今天到期
tasks-overdue = 逾期
tasks-completed = 已完成
tasks-all = 所有任务
tasks-priority-high = 高优先级
tasks-priority-medium = 中优先级
tasks-priority-low = 低优先级
tasks-no-due-date = 无截止日期
tasks-add-subtask = 添加子任务
tasks-mark-complete = 标记为完成
tasks-mark-incomplete = 标记为不完整
tasks-delete-confirm = 您确定要删除此任务吗？
tasks-count = { $计数 ->
    [zero] No tasks
    [one] { $count } task
   *[other] { $count } tasks
}

# -----------------------------------------------------------------------------
# Calendar
# -----------------------------------------------------------------------------
calendar-title = 日历
calendar-today = 今天
calendar-new-event = 新活动
calendar-all-day = 全天
calendar-repeat = 重复
calendar-reminder = 提醒
calendar-view-day = 日
calendar-view-week = 周
calendar-view-month = 月份
calendar-view-year = 年份
calendar-no-events = 没有安排活动
calendar-event-title = 活动标题
calendar-event-location = 地点
calendar-event-description = 描述
calendar-event-attendees = 出席者

# -----------------------------------------------------------------------------
# Meet / Video Conferencing
# -----------------------------------------------------------------------------
meet-title = 见面
meet-join = 加入会议
meet-start = 开始会议
meet-mute = 静音
meet-unmute = 取消静音
meet-video-on = 相机开启
meet-video-off = 相机关闭
meet-share-screen = 共享屏幕
meet-stop-sharing = 停止分享
meet-end-call = 结束通话
meet-leave = 离开会议
meet-participants = { $计数 ->
    [one] { $count } participant
   *[other] { $count } participants
}
meet-waiting-room = 候诊室
meet-admit = 承认
meet-remove = 删除
meet-chat = 会议聊天
meet-raise-hand = 举手
meet-lower-hand = 下手
meet-recording = 录制中
meet-start-recording = 开始录音
meet-stop-recording = 停止录音

# -----------------------------------------------------------------------------
# Mail / Email
# -----------------------------------------------------------------------------
mail-title = 邮件
mail-compose = 撰写
mail-inbox = 收件箱
mail-sent = 已发送
mail-drafts = 草稿
mail-trash = 垃圾
mail-spam = 垃圾邮件
mail-starred = 已加星标
mail-archive = 存档
mail-to = 至
mail-cc = 抄送
mail-bcc = 密件抄送
mail-subject = 主题
mail-body = 留言
mail-reply = 回复
mail-reply-all = 全部回复
mail-forward = 前进
mail-send = 发送
mail-discard = 丢弃
mail-save-draft = 保存草稿
mail-attach = 附加文件
mail-unread = { $计数 ->
    [one] { $count } unread
   *[other] { $count } unread
}
mail-empty-inbox = 您的收件箱是空的
mail-no-subject = （无主题）

# -----------------------------------------------------------------------------
# Paper (Document Editor)
# -----------------------------------------------------------------------------
paper-title = 纸
paper-new-note = 新笔记
paper-search-notes = 搜索笔记...
paper-quick-start = 快速入门
paper-template-blank = 空白
paper-template-meeting = 会议
paper-template-todo = 待办事项
paper-template-research = 研究
paper-untitled = 无题
paper-placeholder = 开始编写，或输入 / 作为命令...
paper-commands = 命令
paper-heading1 = 标题 1
paper-heading1-desc = 大节标题
paper-heading2 = 标题 2
paper-heading2-desc = 中节标题
paper-heading3 = 标题 3
paper-heading3-desc = 小节标题
paper-paragraph = 段落
paper-paragraph-desc = 纯文本
paper-bullet-list = 项目符号列表
paper-bullet-list-desc = 无序列表
paper-numbered-list = 编号列表
paper-numbered-list-desc = 有序列表
paper-todo-list = 待办事项清单
paper-todo-list-desc = 可检查的任务列表
paper-quote = 报价
paper-quote-desc = 引用的块引用
paper-divider = 分频器
paper-divider-desc = 水平线
paper-code-block = 代码块
paper-code-block-desc = 格式化代码
paper-table = 表
paper-table-desc = 插入表格
paper-image = 图片
paper-image-desc = 从 URL 插入图像
paper-callout = 标注
paper-callout-desc = 突出显示的信息框
paper-ai-write = 人工智能书写
paper-ai-write-desc = 使用 AI 生成文本
paper-ai-summarize = 人工智能总结
paper-ai-summarize-desc = 总结所选文本
paper-ai-expand = 人工智能 展开
paper-ai-expand-desc = 展开所选文本
paper-ai-improve = 人工智能改进
paper-ai-improve-desc = 提高写作质量
paper-ai-translate = 人工智能翻译
paper-ai-translate-desc = 翻译成另一种语言
paper-ai-assistant = 人工智能助手
paper-ai-quick-actions = 快速行动
paper-ai-rewrite = 重写
paper-ai-make-shorter = 缩短
paper-ai-make-longer = 更长
paper-ai-fix-grammar = 修复语法
paper-ai-tone = 语气
paper-ai-tone-professional = 专业
paper-ai-tone-casual = 休闲
paper-ai-tone-friendly = 友善
paper-ai-tone-formal = 正式
paper-ai-translate-to = 翻译为
paper-ai-custom-prompt = 自定义提示
paper-ai-custom-placeholder = 描述一下你想要什么...
paper-ai-generate = 生成
paper-ai-response = 人工智能响应
paper-ai-apply = 申请
paper-ai-regenerate = 再生
paper-ai-copy = 复制
paper-word-count = { $count }字
paper-char-count = { $count } 个字符
paper-saved = 已保存
paper-saving = 正在保存...
paper-last-edited = 最后编辑：{ $time }
paper-last-edited-now = 最后编辑：刚刚
paper-export = 出口文件
paper-export-pdf = PDF
paper-export-docx = 字 (.docx)
paper-export-markdown = 降价
paper-export-html = 超文本标记语言
paper-export-txt = 纯文本

# Tasks translations
tasks-active = 主动意图
tasks-awaiting = 等待决定
tasks-paused = 已暂停
tasks-blocked = 被阻止/问题
tasks-time-saved = 节省的活动时间：
tasks-input-placeholder = 你想做什么？例如，“创建 CRM 应用程序”或“提醒我明天给 John 打电话”

# Calendar additional translations
calendar-my-calendars = 我的日历

# Email additional translations
email-scheduled = 预定
email-tracking = 追踪

# Email folder translations
email-inbox = 收件箱
email-starred = 已加星标
email-sent = 已发送
email-drafts = 草稿
email-spam = 垃圾邮件
email-trash = 垃圾
email-compose = 撰写

# -----------------------------------------------------------------------------
# Research
# -----------------------------------------------------------------------------
research-title = 研究
research-search-placeholder = 有什么问...
research-collections = 收藏
research-new-collection = 新系列
research-recent = 最近
research-academic = 学术
research-code = 代码
research-internal = 内部
research-search-all = 搜索一切
research-academic-papers = 学术论文
research-code-docs = 代码和文档
research-internal-kb = 内部知识库
research-sources = 来源
research-trending = 流行趋势
research-pro-search = 专业搜索
research-include-images = 包括图像
research-try-asking = 尝试询问
research-related = 相关问题
research-view-all-sources = 查看所有来源
research-export-citations = 出口引文
research-save-to-collection = 保存至收藏

# -----------------------------------------------------------------------------
# Meet (additional keys)
# -----------------------------------------------------------------------------
meet-new-meeting = 新会议
meet-join-meeting = 加入会议
meet-active-rooms = 活动室
meet-room-title = 会议室
meet-record = 录制
meet-camera = 相机
meet-share = 分享
meet-info = 信息
meet-more = 更多
meet-share-meeting = 分享会
meet-meeting-title = 会议标题
meet-meeting-code = 会议代码
meet-meeting-link = 会议链接
meet-send-invite = 发送邀请

# ── #2307 Tasks goal/monitor detail panel ──────────────────────────
tasks-detail-untitled-goal = 未命名目标
tasks-detail-goal-progress = 目标进度
tasks-detail-current-value = 当前值
tasks-detail-target-value = 目标值
tasks-detail-percent = 进度
tasks-detail-no-actions = 尚无操作
tasks-detail-pending-info = 缺失信息
tasks-detail-scheduler-info = 计划程序信息
tasks-detail-monitor-info = 监控信息
tasks-detail-unavailable = 不可用
tasks-detail-load-failed = 无法加载目标详情


# ── #2307 Database insert/confirm dialogs ──────────────────────────
db-insert-row = 插入行
db-insert-into = 插入到
db-provide-value = 请至少提供一个值
db-insert-failed = 插入失败
db-confirm-title = 确认
db-confirm-accept = 确认
db-confirm-delete = 删除此行？
db-confirm-drop = 删除表
db-delete-failed = 删除失败
db-drop-failed = 删除表失败

# Tasks detail view (#2307)
tasks-detail-type-goal = Goal
tasks-detail-status-pending = Pending
tasks-detail-pending-title = Pending items
tasks-detail-required-reason = A reason is required.
tasks-detail-type-scheduler = Scheduler
tasks-detail-type-monitor = Monitor
tasks-detail-last-action = Last action
tasks-detail-next-run = Next run
tasks-detail-file = File
tasks-detail-interval = Interval
tasks-detail-last-check = Last check
tasks-detail-last-value = Last value
tasks-detail-progress = Progress
tasks-detail-app = App
tasks-detail-field = Field
tasks-detail-reason = Reason
tasks-detail-cron = Cron

# Database dialog copy (#2307)
db-error = Error
db-database = Database
db-empty-query = Execute a query to see results
db-insert = Insert
db-select-table = Select a table
tasks-load-error = 无法加载任务。
tasks-sign-in-to-load = 请登录以加载任务。
calendar-collapse-sidebar = 折叠侧边栏
calendar-prev-month = 上个月
calendar-next-month = 下个月
calendar-prev-period = 上一周期
calendar-next-period = 下一周期
calendar-add-calendar = 添加日历
paper-toggle-sidebar = 切换侧边栏
paper-insert-ai-response = 插入 AI 回复
paper-replace-selection = 替换所选内容
paper-highlight = 高亮
paper-align-left = 左对齐
paper-align-center = 居中对齐
paper-align-right = 右对齐
paper-insert-link = 插入链接
paper-insert-image = 插入图片
paper-insert-table = 插入表格
paper-bold = 加粗
paper-italic = 斜体
paper-underline = 下划线
paper-strikethrough = 删除线
products-view-grid = 网格视图
products-view-list = 列表视图
sheet-freeze-panes = 冻结窗格

# semantic i18n key (#2542)
db-schema-title = 🗄️ 数据库架构

# semantic i18n key (#2542)
db-loading-tables = 正在加载表...

# semantic i18n key (#2542)
db-query-builder = 查询生成器

# semantic i18n key (#2542)
db-run-query = 运行查询

# semantic i18n key (#2542)
db-select-table-hint = 选择一个表以查看数据

# semantic i18n key (#2542)
db-prev-page = ◀ 上一页

# semantic i18n key (#2542)
db-page-info-default = 第 1 页，共 1 页

# semantic i18n key (#2542)
db-next-page = 下一页 ▶

# semantic i18n key (#2542)
research-searching-sources = 正在搜索来源...

# semantic i18n key (#2542)
research-suggestion-latest-ai = 最新 AI 发展

# semantic i18n key (#2542)
research-suggestion-quantum = 量子计算解析

# semantic i18n key (#2542)
research-suggestion-software-architecture = 软件架构

# semantic i18n key (#2542)
research-suggestion-ai-productivity = AI 生产力工具

# semantic i18n key (#2542)
research-related-heading = 相关

# semantic i18n key (#2542)
research-loading = 正在加载研究…

# semantic i18n key (#2542)
research-service-unavailable = 研究服务不可用

# semantic i18n key (#2542)
calendar-mini-month-placeholder = 2025年1月

# semantic i18n key (#2542)
calendar-upcoming = 即将到来

# semantic i18n key (#2542)
calendar-import-export = 导入 / 导出

# semantic i18n key (#2542)
calendar-popup-event-title = 活动标题

# semantic i18n key (#2542)
calendar-popup-location-placeholder = 会议室 A

# semantic i18n key (#2542)
calendar-popup-event-description = 活动描述

# semantic i18n key (#2542)
calendar-personal = 个人

# semantic i18n key (#2542)
calendar-import-ical-title = 导入 iCal 文件

# semantic i18n key (#2542)
calendar-select-ics-file = 选择 .ics 文件

# semantic i18n key (#2542)
calendar-import-to-calendar-label = 导入到日历

# semantic i18n key (#2542)
calendar-default-calendar = 默认日历

# semantic i18n key (#2542)
calendar-loading = 正在加载日历…

# semantic i18n key (#2542)
calendar-unavailable = 日历不可用

# semantic i18n key (#2542)
tasks-progress-status = 状态

# semantic i18n key (#2542)
tasks-progress-sample-title = 实现用户身份验证系统

# semantic i18n key (#2542)
tasks-progress-runtime = 运行时间：

# semantic i18n key (#2542)
tasks-progress-sample-action = 选择令牌过期策略

# semantic i18n key (#2542)
tasks-progress-estimated = 预计：

# semantic i18n key (#2542)
tasks-progress-sample-decision = 安全配置需要审批

# semantic i18n key (#2542)
tasks-progress-log = 进度日志

# semantic i18n key (#2542)
tasks-progress-terminal-live = 终端（实时代理活动）

# semantic i18n key (#2542)
tasks-progress-processed = 已处理：

# semantic i18n key (#2542)
tasks-progress-data-points = 数据点

# semantic i18n key (#2542)
tasks-progress-sample-speed = 约每分钟 8 个来源

# semantic i18n key (#2542)
tasks-progress-sample-insight = 正在综合竞争洞察...

# semantic i18n key (#2542)
tasks-progress-sample-pricing = 正在交叉引用定价数据...

# semantic i18n key (#2542)
tasks-progress-view-details = 查看详情 ▸

# semantic i18n key (#2542)
tasks-create-run = 创建并运行

# semantic i18n key (#2542)
tasks-loading-tasks = 正在加载任务...

# semantic i18n key (#2542)
tasks-select-task = 选择一个任务

# semantic i18n key (#2542)
tasks-select-task-desc = 点击列表中的任务以查看详情

# semantic i18n key (#2542)
tasks-bot-database-label = 机器人数据库：

# semantic i18n key (#2542)
tasks-shared-resources-label = 共享资源：

# semantic i18n key (#2542)
tasks-starting = 正在启动...

# semantic i18n key (#2542)
tasks-create-new-intent = 创建新意图

# semantic i18n key (#2542)
tasks-intent-accomplish-label = 您想完成什么？

# semantic i18n key (#2542)
tasks-execution-mode = 执行模式

# semantic i18n key (#2542)
tasks-mode-automatic = 自动

# semantic i18n key (#2542)
tasks-mode-supervised = 监督

# semantic i18n key (#2542)
tasks-mode-manual-approval = 手动批准

# semantic i18n key (#2542)
tasks-make-decision = 做出决定

# semantic i18n key (#2542)
tasks-decision-required = 需要决定

# semantic i18n key (#2542)
tasks-loading-decision-details = 正在加载决策详情...

# semantic i18n key (#2542)
tasks-skip-for-now = 暂时跳过

# semantic i18n key (#2542)
tasks-confirm-decision = 确认决定

# semantic i18n key (#2542)
tasks-due-soon = 即将到期

# semantic i18n key (#2542)
tasks-autotask-items = // 自动任务项

# semantic i18n key (#2542)
tasks-edit-task = 编辑任务

# semantic i18n key (#2542)
tasks-assignee-user-id = 负责人（用户 ID）

# semantic i18n key (#2542)
tasks-parent-task-subtask-of = 父任务（…的子任务）

# semantic i18n key (#2542)
tasks-reopen-action = ↩ 重新打开

# semantic i18n key (#2542)
tasks-complete-action = ✓ 完成

# semantic i18n key (#2542)
tasks-delete-action = 🗑 删除

# semantic i18n key (#2542)
db-schema-browser = 架构浏览器

# semantic i18n key (#2542)
db-loading-schema = 正在加载架构...

# semantic i18n key (#2542)
db-new-table = + 新建表

# semantic i18n key (#2542)
db-previous = ← 上一页

# semantic i18n key (#2542)
db-next = 下一页 →

# semantic i18n key (#2542)
db-sql-console = SQL 控制台

# semantic i18n key (#2542)
db-execute-query = 执行 (Ctrl+Enter)

# semantic i18n key (#2542)
db-explain = EXPLAIN

# semantic i18n key (#2542)
db-create-new-table = 创建新表

# semantic i18n key (#2542)
db-table-name = 表名

# semantic i18n key (#2542)
db-columns = 列

# semantic i18n key (#2542)
db-not-null = NOT NULL

# semantic i18n key (#2542)
db-type-varchar-255 = VARCHAR(255)

# semantic i18n key (#2542)
db-type-text = TEXT

# semantic i18n key (#2542)
db-type-integer = INTEGER

# semantic i18n key (#2542)
db-type-bigint = BIGINT

# semantic i18n key (#2542)
db-type-decimal = DECIMAL

# semantic i18n key (#2542)
db-type-boolean = BOOLEAN

# semantic i18n key (#2542)
db-type-date = DATE

# semantic i18n key (#2542)
db-type-timestamptz = TIMESTAMPTZ

# semantic i18n key (#2542)
db-type-jsonb = JSONB

# semantic i18n key (#2542)
db-add-column = + 添加列

# semantic i18n key (#2542)
db-create-table = 创建表

# semantic i18n key (#2542)
db-edit-row = 编辑行

# semantic i18n key (#2542)
db-import-csv = 导入 CSV

# semantic i18n key (#2542)
db-select-csv-file = 选择 CSV 文件

# semantic i18n key (#2542)
analytics-ask-placeholder = 询问您的分析数据...
tasks-intent-description-placeholder = 描述您想创建或自动化的内容...
tasks-assignee-placeholder = 负责人 UUID（可选）
tasks-parent-placeholder = 父任务 UUID（可选）

# semantic i18n key (#2542)
db-placeholder-search-tables = 搜索表...
db-aria-close-sql-console = 关闭 SQL 控制台
db-placeholder-sql-query = SELECT * FROM 表名 WHERE 条件...
db-aria-sql-query = SQL 查询
db-placeholder-table-name-example = my_table
db-placeholder-column-id = id
db-placeholder-column-name-example = column_name
