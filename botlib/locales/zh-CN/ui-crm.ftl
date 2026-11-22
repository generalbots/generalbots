# =============================================================================
# General Bots — zh-CN — CRM
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# CRM
# =============================================================================
# -----------------------------------------------------------------------------
# CRM Navigation & General
# -----------------------------------------------------------------------------
crm-title = 客户关系管理
crm-pipeline = 管道
crm-leads = 潜在客户
crm-opportunities = 机会
crm-accounts = 账户
crm-contacts = 联系方式
crm-activities = 活动

# -----------------------------------------------------------------------------
# CRM Entities
# -----------------------------------------------------------------------------
crm-lead = 铅
crm-lead-desc = 不合格的潜在客户
crm-opportunity = 机会
crm-opportunity-desc = 合格的销售机会
crm-account = 账户
crm-account-desc = 公司或组织
crm-contact = 联系方式
crm-contact-desc = 帐户中的人
crm-activity = 活动
crm-activity-desc = 任务、电话或电子邮件

# -----------------------------------------------------------------------------
# CRM Actions
# -----------------------------------------------------------------------------
crm-qualify = 取得资格
crm-convert = 转换
crm-won = 赢了
crm-lost = 迷失
crm-new-lead = 新线索
crm-new-opportunity = 新机遇
crm-new-account = 新账户
crm-new-contact = 新联系人

# -----------------------------------------------------------------------------
# CRM Fields
# -----------------------------------------------------------------------------
crm-stage = 舞台
crm-value = 价值
crm-probability = 概率
crm-close-date = 截止日期
crm-company = 公司简介
crm-phone = 电话
crm-email = 电子邮件
crm-source = 来源
crm-owner = 业主

# -----------------------------------------------------------------------------
# CRM Pipeline Stages
# -----------------------------------------------------------------------------
crm-pipeline-new = 新
crm-pipeline-contacted = 已联系
crm-pipeline-qualified = 合格
crm-pipeline-proposal = 提案
crm-pipeline-negotiation = 洽谈
crm-pipeline-closed-won = 已结束 获胜
crm-pipeline-closed-lost = 关闭 丢失

# -----------------------------------------------------------------------------
# CRM Stats & Metrics
# -----------------------------------------------------------------------------
crm-subtitle = 管理销售线索、机会和客户
crm-stage-lead = 铅
crm-stage-qualified = 合格
crm-stage-proposal = 提案
crm-stage-negotiation = 洽谈
crm-stage-won = 赢了
crm-stage-lost = 迷失
crm-conversion-rate = 转化率
crm-pipeline-value = 管道价值
crm-avg-deal = 平均交易规模
crm-won-month = 本月获奖

# -----------------------------------------------------------------------------
# CRM Empty States
# -----------------------------------------------------------------------------
crm-no-leads = 未找到线索
crm-no-opportunities = 没有找到机会
crm-no-accounts = 没有找到账户
crm-no-contacts = 未找到联系人
crm-drag-hint = 拖动卡牌改变舞台

# CRM / Mail / Campaigns integration keys
crm-compose-email = 撰写电子邮件
crm-send-email = 发送电子邮件
mail-snooze = 贪睡
mail-snooze-later-today = 今天晚些时候（下午 6:00）
mail-snooze-tomorrow = 明天（上午 8:00）
mail-snooze-next-week = 下周（周一上午 8:00）
mail-crm-log = 登录 CRM
mail-crm-create-lead = 创建潜在客户
mail-add-to-list = 添加到列表
campaign-send-email = 发送电子邮件
crm-add-lead = Add Lead
crm-audit-title = Audit trail
crm-campaigns = Campaigns
crm-col-account = Account
crm-col-actions = Actions
crm-col-city = City
crm-col-close = Expected Close
crm-col-close-date = Expected Close
crm-col-contact = Contact
crm-col-contacts = Deals
crm-col-customer = Customer
crm-col-delivery = Delivery
crm-col-email = Email
crm-col-industry = Industry
crm-col-items = Items
crm-col-name = Name
crm-col-number = Number
crm-col-owner = Owner
crm-col-phone = Phone
crm-col-probability = Probability
crm-col-revenue = Annual Revenue
crm-col-source = Source
crm-col-stage = Stage
crm-col-status = Status
crm-col-title = Title
crm-col-total = Total
crm-col-valid-until = Valid until
crm-col-value = Value
crm-create-lead = Create Contact
crm-currency = Currency
crm-deals = Deals
crm-estimated-value = Estimated Value
crm-filter-all = All
crm-filter-lost = Lost
crm-filter-negotiation = Negotiation
crm-filter-new = New
crm-filter-proposal = Proposal
crm-filter-qualified = Qualified
crm-filter-won = Won
crm-first-name = First Name
crm-job-title = Job Title
crm-last-name = Last Name
crm-loading = Loading…
crm-new = New
crm-new-deal = New Deal
crm-notes = Notes
crm-orders = Orders
crm-quotes = Quotes & Orders
crm-search-placeholder-deals = Search deals, accounts...
crm-source-cold-call = Cold Call
crm-source-email = Email Campaign
crm-source-event = Event/Conference
crm-source-linkedin = LinkedIn
crm-source-other = Other
crm-source-partner = Partner
crm-source-referral = Referral
crm-source-website = Website
crm-this-month = Won This Month
crm-total-value = Total Pipeline Value

# semantic i18n key (#2542)
crm-filter-opportunities-placeholder = 筛选商机…
crm-filter-deals-placeholder = 筛选交易…
crm-filter-accounts-placeholder = 筛选客户…
crm-filter-contacts-placeholder = 筛选联系人…
crm-filter-campaigns-placeholder = 筛选营销活动…

# semantic i18n key (#2542)
crm-placeholder-email = zhangsan@company.com
crm-placeholder-deal-lost-reason = 交易丢失的原因/时间
crm-placeholder-first-name = 三
crm-placeholder-last-name = 张
crm-placeholder-phone = +86 138 0013 8000
crm-placeholder-company-name = 公司名称
crm-placeholder-job-title = CEO、经理等
crm-placeholder-amount = 0.00
crm-placeholder-lead-lost-reason = 该线索变冷的原因
crm-placeholder-lead-notes = 关于此线索的其他信息...
crm-placeholder-website = https://

# semantic i18n key (#2542)
crm-placeholder-search-contacts = 搜索联系人...

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
crm-opps-empty-title = 还没有销售机会
crm-opps-empty-desc = 销售机会跟踪一笔可能的销售，从首次接触到成交。
crm-opps-empty-action = 创建销售机会
crm-deals-empty-title = 还没有交易
crm-deals-empty-desc = 各个阶段的交易都会连同金额和负责人显示在这里。
crm-deals-empty-action = 添加交易
crm-accounts-empty-title = 还没有客户公司
crm-accounts-empty-desc = 客户公司是您的销售对象，联系人和交易都归属于它。
crm-accounts-empty-action = 添加客户公司
crm-contacts-empty-title = 还没有联系人
crm-contacts-empty-desc = 您交流的人都在这里：姓名、电子邮件、电话及其所属客户公司。
crm-contacts-empty-action = 添加联系人
crm-campaigns-empty-title = 还没有营销活动
crm-campaigns-empty-desc = 营销活动汇总为销售漏斗带来潜在客户的外联动作。
crm-campaigns-empty-action = 创建营销活动

# -----------------------------------------------------------------------------
# #2328 — audit drawer + campaign metrics state copy
# -----------------------------------------------------------------------------
crm-audit-empty = 暂无审计记录
crm-audit-unavailable = 审计不可用（{ $status }）
crm-campaign-metrics-sent = 已发送 { $sent } · 已送达 { $delivered } · 已打开 { $opened } · 已点击 { $clicked }
crm-revenue-detail = Replies: { $reply } · Unsubscribes: { $unsubscribe } · Bounces: { $bounce } · Complaints: { $complaint } · Skips: { $skip }
crm-revenue-influenced = Influenced deals: { $deals } · won: { $won }
crm-revenue-unattributed = Unattributed touches: { $count }
crm-revenue-unavailable = Revenue attribution is unavailable.
