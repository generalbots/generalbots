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

# semantic i18n key (#2542)
admin-compliance-dashboard-title = 合规仪表板

# semantic i18n key (#2542)
admin-compliance-subtitle = SOC 2 Type II 合规状态、控制监控和审计证据

# semantic i18n key (#2542)
admin-compliance-compliant = 合规

# semantic i18n key (#2542)
admin-compliance-overall-score = 总体合规评分

# semantic i18n key (#2542)
admin-compliance-controls-passing = 通过的控制项

# semantic i18n key (#2542)
admin-compliance-evidence-items-due = 待提交的证据项

# semantic i18n key (#2542)
admin-compliance-until-next-audit = 距离下次审计

# semantic i18n key (#2542)
admin-compliance-next-audit-date = 计划时间：2025年3月7日

# semantic i18n key (#2542)
admin-compliance-trust-service-criteria = 信任服务标准

# semantic i18n key (#2542)
admin-compliance-availability = 可用性

# semantic i18n key (#2542)
admin-compliance-processing-integrity = 处理完整性

# semantic i18n key (#2542)
admin-compliance-confidentiality = 机密性

# semantic i18n key (#2542)
admin-compliance-controls-requiring-attention = 需要关注的控制项

# semantic i18n key (#2542)
admin-compliance-control-mfa-privileged = 特权帐户的多因素身份验证

# semantic i18n key (#2542)
admin-compliance-due-jan-25-2025 = 截止日期：2025年1月25日

# semantic i18n key (#2542)
admin-compliance-remediate = 修复

# semantic i18n key (#2542)
admin-compliance-control-incident-response-testing = 安全事件响应计划测试

# semantic i18n key (#2542)
admin-compliance-issue-annual-test-overdue = 年度测试逾期 15 天

# semantic i18n key (#2542)
admin-compliance-control-dpa-documentation = 数据处理协议文档

# semantic i18n key (#2542)
admin-compliance-due-feb-1-2025 = 截止日期：2025年2月1日

# semantic i18n key (#2542)
admin-compliance-control-data-classification-review = 数据分类策略审查

# semantic i18n key (#2542)
admin-compliance-issue-policy-review-pending = 策略审查待批准

# semantic i18n key (#2542)
admin-compliance-due-feb-15-2025 = 截止日期：2025年2月15日

# semantic i18n key (#2542)
admin-compliance-control-privacy-notice-updates = 隐私声明更新

# semantic i18n key (#2542)
admin-compliance-issue-privacy-policy-update = 隐私政策需要针对新的数据收集进行更新

# semantic i18n key (#2542)
admin-compliance-due-feb-28-2025 = 截止日期：2025年2月28日

# semantic i18n key (#2542)
admin-compliance-evidence-collection = 证据收集

# semantic i18n key (#2542)
admin-compliance-current = 当前

# semantic i18n key (#2542)
admin-compliance-due-soon = 即将到期

# semantic i18n key (#2542)
admin-compliance-evidence-pentest-q4 = 2024年第4季度渗透测试报告

# semantic i18n key (#2542)
admin-compliance-evidence-cc4-vuln = CC4.1.1 - 漏洞评估

# semantic i18n key (#2542)
admin-compliance-evidence-access-review-jan = 访问审查报告 - 2025年1月

# semantic i18n key (#2542)
admin-compliance-evidence-cc6-access = CC6.2.1 - 访问控制

# semantic i18n key (#2542)
admin-compliance-due-in-3-days = 3天后到期

# semantic i18n key (#2542)
admin-compliance-evidence-backup-restore = 备份恢复测试结果

# semantic i18n key (#2542)
admin-compliance-evidence-a1-backup = A1.2.3 - 备份与恢复

# semantic i18n key (#2542)
admin-compliance-due-in-5-days = 5天后到期

# semantic i18n key (#2542)
admin-compliance-evidence-security-training = 安全意识培训记录

# semantic i18n key (#2542)
admin-compliance-evidence-cc1-training = CC1.4.1 - 安全培训

# semantic i18n key (#2542)
admin-compliance-view-all-evidence = 查看所有证据

# semantic i18n key (#2542)
admin-compliance-access-events = 访问事件

# semantic i18n key (#2542)
admin-compliance-configuration-changes = 配置变更

# semantic i18n key (#2542)
admin-compliance-security-events = 安全事件

# semantic i18n key (#2542)
admin-compliance-compliance-events = 合规事件

# semantic i18n key (#2542)
admin-compliance-failed-login-attempt = 登录尝试失败

# semantic i18n key (#2542)
admin-compliance-failed-login-details = 用户：admin@company.com | IP：192.168.1.105 | 原因：密码无效

# semantic i18n key (#2542)
admin-compliance-user-logged-in = 用户已登录

# semantic i18n key (#2542)
admin-compliance-user-logged-in-details = 用户：sarah.johnson@company.com | IP：10.0.0.45
