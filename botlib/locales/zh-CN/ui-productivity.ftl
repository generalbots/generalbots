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
