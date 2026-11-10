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
