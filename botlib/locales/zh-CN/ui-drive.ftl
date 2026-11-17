# =============================================================================
# General Bots — zh-CN — Drive and files
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Drive / Files
# -----------------------------------------------------------------------------
drive-title = 驱动器
drive-upload = 上传
drive-new-folder = 新建文件夹
drive-empty = 还没有文件。上传一些东西！
drive-search = 搜索文件...
drive-sort-name = 名称
drive-sort-date = 日期
drive-sort-size = 尺寸
drive-sort-type = 类型
drive-view-grid = 网格视图
drive-view-list = 列表视图
drive-selected = { $计数 ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
drive-file-size = { $大小->
    [bytes] { $value } B
    [kb] { $value } KB
    [mb] { $value } MB
    [gb] { $value } GB
   *[other] { $value } bytes
}
drive-drop-files = 将文件拖放到此处进行上传

# Drive translations
drive-my-drive = 我的驾驶
drive-shared = 与我分享
drive-recent = 最近
drive-starred = 已加星标
drive-trash = 垃圾
drive-loading-storage = 正在加载存储...
drive-storage-used = 已使用 { $total } 中的{ $used }
drive-empty-folder = 该文件夹是空的

# Drive → Use in chat (#2568)
drive-use-in-chat = 在对话中使用
drive-use-in-chat-preparing = 正在准备 %n…
drive-use-in-chat-attached = 已附加到对话：%n
drive-use-in-chat-failed = 无法将文件附加到对话。
drive-use-in-chat-timeout = 请打开对话以附加文件。
# 共享接收（PWA 分享目标）
drive-share-intake-title = 将共享文件保存到云盘
drive-share-intake-intro = 请查看下方共享项目，然后将其保存到您的云盘。保存前可能需要登录。
drive-share-intake-heading = 共享项目
drive-share-intake-loading = 正在加载共享项目…
drive-share-intake-auth-hint = 如果保存失败，请登录 General Bots 后重试。共享文件会临时存储在此浏览器中，直到保存或清除。
drive-share-intake-save = 保存到云盘
drive-share-intake-open-drive = 打开云盘
drive-share-intake-no-shared = 未找到共享文件。您仍可以打开云盘并手动上传。
drive-share-intake-no-files = 未共享任何文件。
drive-share-intake-text-shown = 上方显示了共享的文本/链接。
drive-share-intake-unavailable = 不可用
drive-share-intake-read-error = 无法读取共享项目。
drive-share-intake-read-failed = 无法读取共享文件。请打开云盘手动上传。
drive-share-intake-sign-in-required = 需要登录才能将文件保存到云盘。
drive-share-intake-uploading = 正在上传
drive-share-intake-saved = 文件已保存到云盘。
drive-share-intake-saved-partial = 部分文件无法保存。您可以重试。
