# General Bots — desktop shell and setup wizard (#2089)

desktop-document-title = General Bots 桌面
sidebar-toggle = 切换侧边栏
sidebar-collapsed-chat = 聊天
sidebar-collapsed-terminal = 终端
sidebar-collapsed-drive = 云端硬盘
sidebar-collapsed-settings = 设置
sidebar-collapsed-account = 账户
setup-welcome-title = 欢迎使用 General Bots
setup-welcome-subtitle = 通过几个简单步骤完成设置
setup-welcome-quick-subtitle = 两个简单步骤即可开始
setup-step-llm = LLM 提供商
setup-step-profile = 用户资料
setup-step-bot = 机器人配置
setup-step-training = 训练
setup-step-ai-model = AI 模型
setup-step-your-name = 您的姓名
setup-choose-provider-title = 选择您的 LLM 提供商
setup-choose-provider-subtitle = 为您的机器人选择 AI 模型提供商
setup-provider-openai-desc = GPT-4、GPT-3.5 模型
setup-provider-nvidia-desc = NVIDIA NIM 推理
setup-provider-anthropic-desc = Claude 3、Claude 3.5 模型
setup-provider-local-desc = llama.cpp、Ollama 等
setup-profile-title = 哪种描述最符合您?
setup-profile-subtitle = 选择您的资料类型以获得个性化默认设置
setup-profile-developer = 开发者
setup-profile-developer-desc = 构建和部署机器人,编写 BASIC 脚本
setup-profile-business = 业务
setup-profile-business-desc = 自动化工作流,管理 CRM
setup-profile-support = 支持
setup-profile-support-desc = 客户服务,工单管理
setup-bot-title = 配置您的机器人
setup-bot-subtitle = 为您的机器人指定名称、用途以及可选的模板
setup-bot-name = 机器人名称
setup-bot-name-placeholder = 我的助手
setup-bot-purpose = 机器人用途
setup-bot-purpose-placeholder = 描述您的机器人应该做什么...
setup-bot-template = 机器人模板
setup-optional = (可选)
setup-template-scratch = — 从头开始 —
setup-name-title = 我们该如何称呼您?
setup-name-subtitle = 此名称将显示在您的会话中
setup-name-label = 您的姓名
setup-name-placeholder = 例如 Maria
setup-training-title = 训练您的机器人
setup-training-subtitle = 拖入 .pdf、.xls、.doc 文件进行初始训练。您可以访问云端硬盘并稍后更改。
setup-drop-text = 将文件拖到此处或点击选择
setup-drop-hint = .pdf、.xls、.doc — 每个最大 10MB
setup-select-files = 选择文件
setup-file-status-pending = 等待中
setup-file-status-uploading = 正在上传
setup-file-status-done = 完成
setup-file-status-error = 错误
setup-saving = 正在保存...
setup-complete-message = 设置完成!欢迎使用 General Bots。
setup-failed = 设置失败:{error}
setup-unknown-error = 未知错误
setup-back = 返回
setup-next = 下一步
setup-finish = 完成
setup-skip = 跳过设置
agent-all-apps = 所有应用

# --- Desktop shell panels: versions, accessibility, sidebar, collaboration,
# --- updates, modals and usage (suite/js/*.js) — #2095

versions-component = 组件
versions-version = 版本
versions-status = 状态
versions-action = 操作
versions-up-to-date = 已是最新
versions-unknown = 未知
versions-check-update = 检查更新
versions-checking = 正在检查...
versions-checked = 已检查
versions-error = 错误
versions-update-available = 更新:{ $version }
versions-could-not-load = 无法加载版本信息。
versions-ensure-backend = 请确保后端服务器正在运行。
versions-retry = 重试

a11y-chat-conversation = 聊天会话
a11y-chat-messages = 聊天消息
a11y-send-message = 发送消息
a11y-type-message = 输入您的消息
a11y-keyboard-shortcuts = 键盘快捷键
a11y-shortcut-focus-chat = 聚焦聊天输入框
a11y-shortcut-show-help = 显示此帮助
a11y-shortcut-escape = 关闭模态框 / 清除选择
a11y-shortcut-command-palette = 命令面板
a11y-shortcut-privacy = 隐私设置
a11y-shortcut-scroll-history = 滚动聊天记录
a11y-shortcut-navigate = 在元素之间导航
a11y-shortcut-activate = 激活聚焦的元素
a11y-shortcut-delete-files = 删除选中的文件(云端硬盘)
a11y-shortcut-select-all = 全选文件(云端硬盘)
a11y-shortcut-clipboard = 复制 / 剪切 / 粘贴文件
a11y-shortcut-navigate-up = 在云端硬盘中向上导航
a11y-shortcut-rename = 重命名选中的文件

sidebar-locked-conversations = 已锁定的会话
sidebar-no-conversations = 暂无会话
sidebar-sign-in-conversations = 登录后查看您的会话
sidebar-history-unavailable = 历史记录不可用
sidebar-back-to-conversations = ← 返回会话
sidebar-no-locked-conversations = 没有已锁定的会话
sidebar-locked-conversation = 已锁定的会话
sidebar-sign-in = 登录
sidebar-add-pin = 添加 PIN

collab-collaborators = 协作者({ $count })
collab-no-one-else = 没有其他人在此

update-available-title = 有可用更新 — v{ $version }
update-now = 立即更新
update-later = 稍后
update-skip-version = 跳过此版本
update-platform = 平台:{ $name }
update-asset = 资源:{ $name }
update-check-failed = 检查更新失败:{ $error }
update-view-release-notes = 查看发行说明
update-release-notes-title = 发行说明 — v{ $version }
update-no-release-notes = 没有可用的发行说明。

modal-alert-title = 警告
modal-input-title = 输入

external-save-to = 保存到 { $target }

usage-title = 使用量与成本
usage-last-days = 最近 { $days } 天
usage-loading = 正在加载使用量…
usage-per-model = 按模型
usage-per-day = 按天
usage-model = 模型
usage-tokens = 令牌
usage-messages = 消息
usage-cost = 成本
usage-day = 日期
usage-days-7 = 7 天
usage-days-30 = 30 天
usage-days-90 = 90 天

pin-close = 关闭
pin-length-hint = 4–8 位数字
pin-cancel = 取消
pin-ok = 确定

# --- Collaboration surfaces shared by every suite app: share dialog, activity
# --- trail, conflict banner and version history (suite/js/collab-*.js) — #2095

role-owner = 所有者
role-editor = 编辑者
role-commenter = 评论者
role-viewer = 查看者

share-request-failed = 请求失败({ $status })
share-warning-title = 共享链接尚无法访问
share-warning-body = 此安装仅在 localhost 上可访问,因此这里的链接在别处无法打开。请先设置以下之一:在设置 → 网络中启用 P2P,或将共享链接指向您自己的域名。
share-open-shared-links = 打开共享链接
share-open-network = 打开网络
share-invite-people = 邀请他人
share-anyone-with-link = 任何拥有链接的人
share-create-link = 创建链接
share-transfer-ownership = 转移所有权
share-transfer = 转移
share-new-owner-placeholder = 新所有者电子邮件
share-owner-suffix = · 所有者
share-no-people = 尚未添加任何人。
share-no-links = 暂无公开链接。
share-copied = 已复制
share-revoke = 撤销
share-enter-email = 输入电子邮件
share-enter-new-owner = 输入新所有者的电子邮件
share-needs-base = 创建共享链接前请先设置 P2P 或您自己的域名。

activity-title = 活动
activity-request-failed = 请求失败({ $status })
activity-load-more = 加载更多
activity-loading = 正在加载…
activity-empty = 暂无活动
activity-action-create = 创建了
activity-action-edit = 编辑了
activity-action-comment = 评论了
activity-action-delete = 删除了
activity-action-resolve = 解决了以下评论:
activity-action-reopen = 重新打开了以下评论:
activity-action-reaction = 回应了
activity-action-share = 共享了
activity-action-restore = 恢复了
activity-action-transfer = 转移了

conflict-title = 编辑冲突
conflict-dismiss = 忽略
conflict-edited-concurrently = 同时编辑({ $op })
conflict-keep-server = 保留服务器版本
conflict-keep-mine = 保留我的更改
conflict-not-started = 冲突监视未启动
conflict-request-failed = 请求失败({ $status })

versions-title = 版本历史
versions-loading = 正在加载…
versions-empty = 暂无版本 — 首次保存会创建一个。
versions-version-n = 版本 { $n }
versions-current-tag = (当前)
versions-name = 名称…
versions-restore-failed = 恢复失败:{ $error }
versions-rename-failed = 重命名失败:{ $error }
versions-restore-dialog-title = 恢复版本
versions-restore-confirm = 恢复此版本?将创建一个新的当前版本 — 不会丢失任何内容。
versions-name-dialog-title = 命名版本
versions-name-prompt = 为此版本命名(例如 "v2 — 已批准"):
versions-request-failed = 请求失败({ $status })

# --- Desktop shell: snap picker, workspace tabs, virtual desktops, context
# --- menu, git panel, browser panel, widgets and data export (suite/js/*.js)

snap-title = 贴靠“{ $app }”
snap-layout-top-left = 左上
snap-layout-top = 上半部分
snap-layout-top-right = 右上
snap-layout-left = 左半部分
snap-layout-full = 最大化
snap-layout-right = 右半部分
snap-layout-bottom-left = 左下
snap-layout-bottom = 下半部分
snap-layout-bottom-right = 右下
snap-layout-third-left = 左侧三分之一
snap-layout-two-thirds = 三分之二
snap-layout-third-right = 右侧三分之一

wstabs-close-tab = 关闭标签页
wstabs-new-tab = 新建工作区标签页
wstabs-default-name = 工作区 { $n }
wstabs-default-main = 主
wstabs-rename-title = 重命名工作区
wstabs-name-prompt = 工作区名称:

vdesk-default-name = 桌面 { $n }
vdesk-title = 虚拟桌面
vdesk-new = 新建桌面
vdesk-rename-title = 重命名桌面
vdesk-name-prompt = 桌面名称:
vdesk-restore-title = 恢复会话
vdesk-restore-body = 要在此桌面上重新打开上次会话的 { $count } 个窗口吗?
vdesk-restore-body-short = 恢复上次会话({ $count } 个窗口)?

ctxmenu-search = 使用 DuckDuckGo 搜索
ctxmenu-search-prompt = 使用 DuckDuckGo 搜索:
ctxmenu-wikipedia = 维基百科定义
ctxmenu-wikipedia-prompt = 维基百科定义:

git-loading-status = 正在加载状态...
git-no-changes = 无更改
git-new-branch-prompt = 新分支名称:
git-loading-diff = 正在加载差异...
git-diff-failed = 加载差异失败
git-commit-message-required = 请输入提交消息
git-pushing = 正在推送...
git-committed = 已成功提交并推送
git-failed = Git 操作失败

browser-session-active = 浏览器会话 { $id } 处于活动状态。
browser-init-failed = 初始化浏览器失败
browser-init-first = 请先初始化浏览器
browser-capture = 捕获
browser-no-session = 没有活动会话
browser-export-failed = 导出失败
browser-no-actions = 没有记录的操作

widget-load-failed = 小组件加载失败
widget-no-source = 未配置小组件源
widget-open-windowed = 在窗口中打开
widget-pin-launcher = 固定到启动器

dataexport-title = 数据导出与备份
dataexport-desc = 将您的所有数据(包括聊天记录、记忆、云端硬盘元数据和设置)导出为 JSON 备份文件。
dataexport-export-all = 导出所有数据
dataexport-import = 导入备份
dataexport-exported = 备份已成功导出
dataexport-too-large = 备份被拒绝:文件大于 4 MB
dataexport-invalid = 无效的备份文件:{ $error }
dataexport-nothing = 此备份不包含可恢复的设置
dataexport-unknown-date = 未知日期
dataexport-restore-confirm = 这将从 { $date } 的备份中恢复 { $count } 项设置。凭据和会话令牌永远不会被导入。继续吗?
dataexport-restored = 备份已恢复({ $count } 项)。
dataexport-rejected = { $count } 个条目被允许列表拒绝。
dataexport-reloading = 正在重新加载...

comments-title = 评论
comments-request-failed = 请求失败({ $status })
comments-loading = 正在加载评论…
comments-load-failed = 加载评论失败:{ $error }
comments-empty-open = 没有未解决的评论。使用 @name 提及某人。
comments-empty-resolved = 没有已解决的评论。使用 @name 提及某人。
comments-resolved-chip = 已解决
comments-resolve = 解决
comments-reopen = 重新打开
comments-resolved-announce = 评论已解决
comments-reopened-announce = 评论已重新打开
comments-resolve-failed = 解决失败:{ $error }
comments-added = 评论已添加
comments-post = 发布
comments-post-failed = 评论失败:{ $error }
comments-reaction-failed = 回应失败:{ $error }
comments-delete-failed = 删除失败:{ $error }
comments-reply-failed = 回复失败:{ $error }
comments-delete-title = 删除评论
comments-delete-confirm = 删除此评论?
comments-reply-title = 回复
comments-reply-prompt = 回复评论:
comments-filter-open = 未解决
comments-filter-all = 全部
comments-filter-resolved = 已解决
comments-input-placeholder = 添加评论… 使用 @name 提及
comments-someone = 某人
comments-mention-announce = { $author } 在评论中提到了您
