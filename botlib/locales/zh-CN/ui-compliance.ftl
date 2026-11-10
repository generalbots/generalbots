# =============================================================================
# General Bots — zh-CN — Compliance
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# TOOLS / COMPLIANCE SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Compliance Main
# -----------------------------------------------------------------------------
compliance-title = API 合规报告
compliance-subtitle = 对所有机器人进行安全扫描 - 检查密码、脆弱代码和错误配置
compliance-export-report = 出口报告
compliance-run-scan = 运行合规性扫描
compliance-scanning = 正在扫描...

# -----------------------------------------------------------------------------
# Bot Selector
# -----------------------------------------------------------------------------
compliance-all-bots = 所有机器人
compliance-select-bots = 选择机器人

# -----------------------------------------------------------------------------
# Stats Cards
# -----------------------------------------------------------------------------
compliance-critical = 关键
compliance-critical-desc = 需要立即采取行动
compliance-high = 高
compliance-high-desc = 安全风险
compliance-medium = 中等
compliance-medium-desc = 应该解决
compliance-low = 低
compliance-low-desc = 最佳实践
compliance-info = 信息
compliance-info-desc = 信息性

# -----------------------------------------------------------------------------
# Filters
# -----------------------------------------------------------------------------
compliance-filter-severity = 严重性
compliance-filter-type = 类型
compliance-filter-all-severities = 所有严重程度
compliance-filter-all-types = 所有类型
compliance-search-issues = 搜索问题...

# -----------------------------------------------------------------------------
# Issue Types
# -----------------------------------------------------------------------------
compliance-type-password = 配置中的密码
compliance-type-hardcoded = 硬编码的秘密
compliance-type-deprecated = 已弃用的关键字
compliance-type-fragile = 脆弱的代码
compliance-type-config = 配置问题

# -----------------------------------------------------------------------------
# Results Table
# -----------------------------------------------------------------------------
compliance-results = 结果
compliance-results-count = { $计数 ->
    [one] { $count } issue found
   *[other] { $count } issues found
}
compliance-col-severity = 严重性
compliance-col-issue = 问题
compliance-col-location = 地点
compliance-col-details = 详情
compliance-col-action = 行动
compliance-view-details = 查看详情
compliance-fix-issue = 修复问题
compliance-ignore = 忽略
compliance-no-issues = 没有发现问题
compliance-no-issues-desc = 太棒了！您的机器人是合规的。

# -----------------------------------------------------------------------------
# Scan Progress
# -----------------------------------------------------------------------------
compliance-scan-in-progress = 扫描正在进行中...
compliance-scan-checking = 检查{ $item }...
compliance-scan-complete = 扫描完成
compliance-scan-failed = 扫描失败

## Compliance frameworks (enterprise-grade release)
compliance-frameworks = 框架
compliance-new-framework = 新
compliance-framework-name = 名称
compliance-framework-version = 版本
compliance-framework-description = 描述
compliance-create-framework = 创建框架
compliance-controls = 控制
compliance-add-control = 添加控件
compliance-control-id = 控制ID
compliance-control-title = 标题
compliance-control-category = 类别
compliance-control-description = 描述
compliance-mandatory = 强制
compliance-optional = 可选
compliance-evidence = 证据
compliance-attach-evidence = 附上证据
compliance-evidence-path = 文件路径（驱动器工件）
compliance-evidence-type = 类型
compliance-approve = 批准
compliance-covered = 覆盖
compliance-no-evidence = 没有证据
compliance-export-csv = 导出 CSV
compliance-archive = 存档
compliance-total-controls = 全面控制
compliance-coverage = 覆盖范围
compliance-no-frameworks = 尚未配置框架。
