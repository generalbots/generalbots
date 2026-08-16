# ============================================================================
# General Bots - Common Translations (English)
# ============================================================================
# This file contains shared strings used across all GB components.
# Keep message IDs lowercase with hyphens: category-subcategory-descriptor
# ============================================================================

# -----------------------------------------------------------------------------
# Brand
# -----------------------------------------------------------------------------
app-name = 通用机器人
app-tagline = 您的人工智能驱动的生产力工作空间

# -----------------------------------------------------------------------------
# Common Actions
# -----------------------------------------------------------------------------
action-save = 保存
action-cancel = 取消
action-delete = 删除
action-edit = 编辑
action-close = 关闭
action-confirm = 确认
action-retry = 重试
action-back = 返回
action-next = 下一步
action-submit = 提交
action-search = 搜索
action-refresh = 刷新
action-copy = 复制
action-paste = 粘贴
action-undo = 撤消
action-redo = 重做
action-select = 选择
action-select-all = 选择全部
action-clear = 清除
action-reset = 重置
action-apply = 申请
action-create = 创建
action-update = 更新
action-remove = 删除
action-add = 添加
action-upload = 上传
action-download = 下载
action-export = 出口
action-import = 进口
action-share = 分享
action-send = 发送
action-reply = 回复
action-forward = 前进
action-archive = 存档
action-restore = 恢复
action-duplicate = 重复
action-rename = 重命名
action-move = 移动
action-filter = 过滤器
action-sort = 排序
action-view = 查看
action-hide = 隐藏
action-show = 显示
action-expand = 展开
action-collapse = 崩溃
action-enable = 启用
action-disable = 禁用
action-connect = 连接
action-disconnect = 断开连接
action-sync = 同步
action-start = 开始
action-stop = 停止
action-pause = 暂停
action-resume = 简历
action-continue = 继续
action-finish = 完成
action-complete = 完成
action-approve = 批准
action-reject = 拒绝
action-accept = 接受
action-decline = 拒绝
action-login = 登录
action-logout = 退出
action-signup = 注册
action-forgot-password = 忘记密码

# -----------------------------------------------------------------------------
# Common Labels
# -----------------------------------------------------------------------------
label-loading = 加载中...
label-saving = 正在保存...
label-processing = 处理中...
label-searching = 正在寻找...
label-uploading = 正在上传...
label-downloading = 正在下载...
label-no-results = 没有找到结果
label-no-data = 无可用数据
label-empty = 空
label-none = 无
label-all = 全部
label-selected = 已选择
label-required = 必填
label-optional = 可选
label-default = 默认
label-custom = 定制
label-new = 新
label-draft = 吃水
label-pending = 待定
label-active = 活跃
label-inactive = 不活跃
label-enabled = 启用
label-disabled = 残疾人
label-public = 公共
label-private = 私人
label-shared = 共享
label-yes = 是的
label-no = 否
label-on = 开
label-off = 关闭
label-true = 真实
label-false = 错误
label-unknown = 未知
label-other = 其他
label-more = 更多
label-less = 少
label-details = 详情
label-summary = 总结
label-description = 描述
label-name = 名称
label-title = 标题
label-type = 类型
label-status = 状态
label-priority = 优先级
label-date = 日期
label-time = 时间
label-size = 尺寸
label-count = 计数
label-total = 总计
label-average = 平均
label-minimum = 最低
label-maximum = 最大
label-version = 版本
label-id = 身份证号
label-created = 已创建
label-updated = 已更新
label-modified = 修改
label-deleted = 已删除
label-by = 由
label-from = 来自
label-to = 至
label-at = 在
label-in = 在
label-of = 的

# -----------------------------------------------------------------------------
# Status Messages
# -----------------------------------------------------------------------------
status-success = 成功
status-error = 错误
status-warning = 警告
status-info = 信息
status-loading = 加载中
status-complete = 完成
status-incomplete = 不完整
status-failed = 失败
status-cancelled = 取消
status-pending = 待定
status-in-progress = 进行中
status-done = 完成
status-ready = 准备好
status-not-ready = 未准备好
status-connected = 已连接
status-disconnected = 已断开连接
status-online = 在线
status-offline = 离线
status-available = 可用
status-unavailable = 不可用
status-busy = 忙
status-away = 离开

# -----------------------------------------------------------------------------
# Confirmation Dialogs
# -----------------------------------------------------------------------------
confirm-delete = 您确定要删除此内容吗？
confirm-delete-item = 您确定要删除“{ $name }”吗？
confirm-delete-items = 您确定要删除 { $count ->
    [one] this item
   *[other] these { $count } items
}?
confirm-discard-changes = 您有未保存的更改。您确定要丢弃它们吗？
confirm-logout = 您确定要退出吗？
confirm-cancel = 您确定要取消吗？

# -----------------------------------------------------------------------------
# Time and Dates
# -----------------------------------------------------------------------------
time-now = 现在
time-seconds-ago = { $计数 ->
    [one] { $count } second ago
   *[other] { $count } seconds ago
}
time-minutes-ago = { $计数 ->
    [one] { $count } minute ago
   *[other] { $count } minutes ago
}
time-hours-ago = { $计数 ->
    [one] { $count } hour ago
   *[other] { $count } hours ago
}
time-days-ago = { $计数 ->
    [one] { $count } day ago
   *[other] { $count } days ago
}
time-weeks-ago = { $计数 ->
    [one] { $count } week ago
   *[other] { $count } weeks ago
}
time-months-ago = { $计数 ->
    [one] { $count } month ago
   *[other] { $count } months ago
}
time-years-ago = { $计数 ->
    [one] { $count } year ago
   *[other] { $count } years ago
}
time-in-seconds = { $计数 ->
    [one] in { $count } second
   *[other] in { $count } seconds
}
time-in-minutes = { $计数 ->
    [one] in { $count } minute
   *[other] in { $count } minutes
}
time-in-hours = { $计数 ->
    [one] in { $count } hour
   *[other] in { $count } hours
}
time-in-days = { $计数 ->
    [one] in { $count } day
   *[other] in { $count } days
}
time-today = 今天
time-yesterday = 昨天
time-tomorrow = 明天
time-this-week = 本周
time-last-week = 上周
time-next-week = 下周
time-this-month = 这个月
time-last-month = 上个月
time-next-month = 下个月
time-this-year = 今年
time-last-year = 去年
time-next-year = 明年

# Days of the week
day-sunday = 周日
day-monday = 星期一
day-tuesday = 星期二
day-wednesday = 星期三
day-thursday = 星期四
day-friday = 周五
day-saturday = 星期六
day-sun = 太阳
day-mon = 周一
day-tue = 星期二
day-wed = 周三
day-thu = 星期四
day-fri = 周五
day-sat = 星期六

# Months
month-january = 一月
month-february = 二月
month-march = 三月
month-april = 四月
month-may = 五月
month-june = 六月
month-july = 七月
month-august = 八月
month-september = 九月
month-october = 十月
month-november = 十一月
month-december = 十二月
month-jan = 扬
month-feb = 二月
month-mar = 三月
month-apr = 四月
month-may-short = 五月
month-jun = 君
month-jul = 七月
month-aug = 八月
month-sep = 九月
month-oct = 十月
month-nov = 十一月
month-dec = 十二月

# -----------------------------------------------------------------------------
# File Sizes
# -----------------------------------------------------------------------------
size-bytes = { $value }B
size-kilobytes = { $value }KB
size-megabytes = { $value }MB
size-gigabytes = { $value } GB
size-terabytes = { $value }TB

# -----------------------------------------------------------------------------
# Pagination
# -----------------------------------------------------------------------------
pagination-page = 第 { $current } 页（共 { $total }）
pagination-showing = 显示 { $total } 中的{ $start } 至 { $end }
pagination-items-per-page = 每页项目数
pagination-first = 第一
pagination-previous = 上一页
pagination-next = 下一步
pagination-last = 最后
pagination-go-to-page = 前往页面

# -----------------------------------------------------------------------------
# Form Validation
# -----------------------------------------------------------------------------
validation-required = 该字段为必填项
validation-required-field = { $field } 为必填项
validation-email-invalid = 请输入有效的电子邮件地址
validation-url-invalid = 请输入有效的网址
validation-number-invalid = 请输入有效号码
validation-date-invalid = 请输入有效日期
validation-min-length = 必须至少 { $min } 个字符
validation-max-length = 不得超过 { $max } 个字符
validation-min-value = 必须至少 { $min }
validation-max-value = 不得超过 { $max }
validation-pattern-mismatch = 格式无效
validation-passwords-mismatch = 密码不匹配
validation-file-too-large = 文件太大。最大尺寸为{ $max }
validation-file-type-invalid = 文件类型无效。允许的类型：{ $types }

# -----------------------------------------------------------------------------
# Accessibility
# -----------------------------------------------------------------------------
a11y-skip-to-content = 跳至主要内容
a11y-loading = 正在加载中，请稍候
a11y-menu-open = 打开菜单
a11y-menu-close = 关闭菜单
a11y-expand = 展开
a11y-collapse = 崩溃
a11y-selected = 已选择
a11y-not-selected = 未选择
a11y-required = 必填字段
a11y-error = 错误
a11y-success = 成功
a11y-warning = 警告
a11y-info = 信息
