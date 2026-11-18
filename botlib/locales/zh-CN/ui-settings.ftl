# =============================================================================
# General Bots — zh-CN — Settings
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Settings
# -----------------------------------------------------------------------------
settings-title = 设置
settings-general = 一般
settings-account = 账户
settings-notifications = 通知
settings-privacy = 隐私
settings-security = 安全设置
settings-language = 语言
settings-theme = 主题
settings-theme-light = 光
settings-theme-dark = 黑暗
settings-theme-system = 系统
settings-save = 保存更改
settings-saved = 设置保存成功
settings-create-api-key = 创建 API 密钥
settings-api-key-name = 按键名称
settings-api-key-scopes = 范围（以逗号分隔）
settings-api-key-scopes-hint = 对于默认的“用户”范围，将其留空。
settings-api-key-expiry = 到期时间（天）
settings-api-key-expiry-hint = 留空则不会过期。
settings-add-webhook = 添加网络钩子
settings-webhook-url = 网络钩子 URL
settings-webhook-events = 事件（以逗号分隔）
settings-webhook-events-hint = 留空以接收所有事件。
settings-timezone = 时区
settings-date-format = 日期格式
settings-time-format = 时间格式

# -----------------------------------------------------------------------------
# Settings - Language & Localization
# -----------------------------------------------------------------------------
settings-language-desc = 选择您的首选语言
settings-display-language = 显示语言
settings-language-affects = 影响应用程序中的所有文本
settings-date-format-desc = 日期如何显示
settings-time-format-desc = 12 小时制或 24 小时制
settings-language-changed = 语言更改成功
settings-reload-required = 需要重新加载页面才能应用更改

# Settings - Profile
settings-profile-desc = 管理您的个人信息和偏好
settings-profile-photo = 个人资料照片
settings-profile-photo-desc = 其他用户可以看到您的个人资料照片
settings-upload-photo = 上传照片
settings-remove-photo = 删除
settings-basic-info = 基本信息
settings-display-name = 显示名称
settings-username = 用户名
settings-email-address = 电子邮件地址
settings-bio = 简介
settings-bio-placeholder = 告诉我们你自己...
settings-contact-info = 联系方式
settings-phone-number = 电话号码
settings-location = 地点
settings-website = 网站

# Settings - Security
settings-security-desc = 通过增强的安全性保护您的帐户
settings-change-password = 更改密码
settings-change-password-desc = 定期更新您的密码以提高安全性
settings-current-password = 当前密码
settings-new-password = 新密码
settings-confirm-password = 确认新密码
settings-update-password = 更新密码
settings-2fa = 双因素身份验证
settings-2fa-desc = 为您的帐户添加额外的安全层
settings-authenticator-app = 验证器应用程序
settings-authenticator-desc = 使用验证器应用程序获取 2FA 代码
settings-enable-2fa = 启用 2FA
settings-disable-2fa = 禁用 2FA
settings-active-sessions = 活跃会话
settings-active-sessions-desc = 管理您的活动登录会话
settings-this-device = 这个装置
settings-terminate-session = 终止
settings-terminate-all = 终止所有其他会话

# Settings - Appearance
settings-appearance = 外观
settings-appearance-desc = 自定义应用程序的外观
settings-theme-selection = 主题
settings-theme-selection-desc = 选择您喜欢的颜色主题
settings-theme-blue = 蓝色
settings-theme-purple = 紫色
settings-theme-green = 绿色
settings-theme-orange = 橙色
settings-layout-preferences = 布局首选项
settings-compact-mode = 紧凑模式
settings-compact-mode-desc = 减少间距以显示更多内容
settings-show-sidebar = 显示侧边栏
settings-show-sidebar-desc = 始终显示导航侧边栏
settings-animations = 动画
settings-animations-desc = 启用 UI 动画和过渡

# Settings - Notifications
settings-notifications-title = 通知
settings-notifications-desc = 控制您接收通知的方式
settings-email-notifications = 电子邮件通知
settings-direct-messages = 直接消息
settings-direct-messages-desc = 接收新私信的电子邮件
settings-mentions = 提及
settings-mentions-desc = 当有人提及您时收到电子邮件
settings-weekly-digest = 每周文摘
settings-weekly-digest-desc = 获取每周活动摘要
settings-marketing = 营销
settings-marketing-desc = 接收新闻和产品更新
settings-push-notifications = 推送通知
settings-enable-push = 启用推送通知
settings-enable-push-desc = 接收浏览器推送通知
settings-notification-sound = 声音
settings-notification-sound-desc = 播放通知声音
settings-in-app-notifications = 应用内通知

# Settings - Storage
settings-storage = 存储
settings-storage-desc = 管理您的存储使用情况
settings-storage-usage = 存储使用情况
settings-storage-used = 已使用 { $total } 中的{ $used }
settings-storage-upgrade = 升级存储

# Settings - Privacy
settings-privacy-title = 隐私
settings-privacy-desc = 控制您的隐私设置
settings-data-collection = 数据收集
settings-analytics = 分析
settings-analytics-desc = 通过发送匿名使用数据帮助我们改进
settings-crash-reports = 崩溃报告
settings-crash-reports-desc = 自动发送崩溃报告
settings-download-data = 下载您的数据
settings-download-data-desc = 获取所有数据的副本
settings-delete-account = 删除帐户
settings-delete-account-desc = 永久删除您的帐户和所有数据
settings-delete-account-warning = 此操作无法撤消

# Settings - Billing
settings-billing = 计费
settings-billing-desc = 管理您的订阅和付款方式
settings-current-plan = 当前计划
settings-free-plan = 免费计划
settings-pro-plan = 专业计划
settings-enterprise-plan = 企业计划
settings-upgrade-plan = 升级计划
settings-payment-methods = 付款方式
settings-add-payment = 添加付款方式
settings-billing-history = 账单历史记录

# -----------------------------------------------------------------------------
# OAuth Account Linking (Settings)
# -----------------------------------------------------------------------------
oauth-connected-accounts = 关联账户
oauth-connect = 连接
oauth-unlink = 取消链接
oauth-not-connected = 未连接
oauth-linked = 链接
oauth-no-accounts = 尚未关联任何帐户。
oauth-loading = 正在加载关联帐户...

## Payment cards (Stripe SetupIntent)
cards-title = 支付与卡
cards-saved = 已保存的卡片
cards-hint = 卡由我们的支付提供商安全存储。卡号永远不会到达我们的服务器。
cards-add = 添加卡
cards-add-first = 添加您的第一张卡
cards-none = 尚未保存卡片
cards-empty-hint = 添加卡以启用自动计费和更快的结账速度。您将被重定向到我们的安全支付提供商以输入您的卡详细信息。
cards-default = 默认
cards-set-default = 设置默认值
cards-default-btn = 默认卡
cards-remove = 删除
cards-remove-confirm = 删除这张卡？
cards-expires = 过期
cards-load-error = 无法加载已保存的卡片。
cards-add-error = 无法添加卡
cards-default-error = 无法更新默认值
cards-remove-error = 无法移除卡
cards-default-updated = 默认卡已更新
cards-removed = 卡已移除

# Settings → Model (#2568)
mdl-title = 模型
mdl-subtitle = 助手思考的位置。选择在本机运行的模型、您的服务商端点，或免费托管模型。
mdl-current = 当前模型
mdl-where = 助手思考的位置
mdl-local = 本机
mdl-own = 组织服务器或服务商密钥
mdl-free = 免费托管模型
mdl-active = 使用中
mdl-endpoint = 端点 URL
mdl-model = 模型
mdl-key = API 密钥
mdl-key-stored = 密钥已保存
mdl-test = 测试
mdl-save = 保存
mdl-test-ok = 可访问，延迟 %ms ms
mdl-test-fail = 失败：%e
mdl-testing = 测试中…
mdl-saving = 保存中…
mdl-saved = 已保存。从下一条消息开始生效。
mdl-swapped = 就绪。新模型已生效。
mdl-restart = 已保存。重启应用后使用新模型。
mdl-admin-only = 只有管理员可以更改模型。
mdl-loading = 加载中…
mdl-local-active = 活动的本地模型：%m
mdl-local-hint = 请在 On-device AI 页面选择一个模型，即可在此使用。
mdl-local-open = 打开 On-device AI
mdl-free-note = 您编写和附加的所有内容都会发送到 %h。
mdl-inconsistent = 模型配置不一致。
mdl-restore-free = 恢复免费模型
mdl-confirm-external = 您编写和附加的内容将发送到本机之外的 %h。继续吗？
mdl-scope-machine = 本机
mdl-scope-network = 组织服务器（%h）
mdl-scope-external = 外部主机（%h）
mdl-unavailable = 此服务器未提供模型 API。

# Settings → Instructions, Shortcuts, Text size (#2568)
instr-title = 自定义指令
instr-subtitle = 告诉助手关于您的信息以及您希望的回复方式。
instr-enabled = 启用自定义指令
instr-enabled-desc = 关闭后，助手将在没有这些指令的情况下回复。
instr-about = 助手应该了解您的哪些信息？
instr-about-placeholder = 角色、语言、背景、偏好…
instr-response = 助手应该如何回复？
instr-response-placeholder = 语气、详细程度、格式…
instr-save = 保存
instr-saving = 保存中…
instr-saved = 已保存。
instr-save-failed = 无法保存：%e
instr-loading = 加载中…
instr-load-failed = 无法加载：%e
instr-unavailable = 此服务器未提供自定义指令。
sc-subtitle = 在套件任意位置按 Shift + ? 可打开此列表。
sc-group-general = 通用
sc-group-drive = 云端硬盘
sc-note = 在输入框中输入时快捷键会被忽略，因此不会打断消息或表单。
ts-title = 文字大小
ts-desc = 选择整个套件的文字大小
ts-small = 小
ts-default = 默认
ts-large = 大
ts-xlarge = 特大
# Settings → Appearance: chat background pattern
chatbg-title = 聊天背景
chatbg-desc = 选择对话背后的图案
chatbg-none = 无
chatbg-image = 纸张
chatbg-dots = 圆点
chatbg-microdots = 微点
chatbg-grid = 网格
chatbg-blueprint = 蓝图
chatbg-weave = 织纹
chatbg-mosaic = 马赛克

# semantic i18n key (#2542)
settings-network-participation-desc = 控制此安装是否参与对等网络

# semantic i18n key (#2542)
settings-p2p-network = P2P 网络

# semantic i18n key (#2542)
settings-network-mode-label = 网络模式

# semantic i18n key (#2542)
settings-network-mode-p2p = P2P（推荐）
settings-network-p2p-description = 离线模式会将所有连接保持在 <code>127.0.0.1</code>，共享链接仅在本地网络中可用。<br />P2P 将启用未来功能，例如免费共享链接中继和对等节点发现。在您明确共享内容之前，不会有任何数据离开此设备。
settings-network-p2p-restart-hint = 此首选项会保存在本地，并在应用重启后生效。中继端点是固定的 (<code id="network-relay-host">peer.generalbots.org</code>)；对等节点和主站点解析始终使用它，因此无需手动配置。

# semantic i18n key (#2542)
settings-role-viewer = 查看者

# semantic i18n key (#2542)
settings-register-dns-hostname = 注册 DNS 主机名

# semantic i18n key (#2542)
settings-dns-aaaa-ipv6 = AAAA (IPv6)

# semantic i18n key (#2542)
settings-dns-cname = CNAME

# semantic i18n key (#2542)
settings-create-token = 创建令牌

# semantic i18n key (#2542)
settings-system-preview-desc = 此部署的预览模式和版本信息

# semantic i18n key (#2542)
settings-system-preview-surface-desc = 控制显示多少界面，而不是安装了什么。

# semantic i18n key (#2542)
settings-system-preview-toggle-desc = 显示预览中的应用和完整设置树。关闭时仅保留此版本的基本设置。

# semantic i18n key (#2542)
settings-component-versions = 组件版本

# semantic i18n key (#2542)
settings-component-versions-desc = 此安装的 BotServer、BotUI 和服务版本

# semantic i18n key (#2542)
settings-open-versions = 打开版本

# semantic i18n key (#2542)
settings-on-device-ai = 设备端 AI

# semantic i18n key (#2542)
settings-operating-system = 操作系统

# semantic i18n key (#2542)
settings-ledger = 账本

# semantic i18n key (#2542)
settings-updates = 更新

# semantic i18n key (#2542)
settings-this-workspace = 此工作区

# semantic i18n key (#2542)
settings-loading-identity = 正在加载身份…

# semantic i18n key (#2542)
settings-plan-placeholder = 套餐：—

# semantic i18n key (#2542)
settings-preview-placeholder = 预览：—

# semantic i18n key (#2542)
settings-oda-subtitle = 在此设备上运行的模型。下载适合硬件的模型并激活，本地引擎将在下次启动时使用它。此处不使用 API 密钥。

# semantic i18n key (#2542)
settings-oda-detecting-hardware = 正在检测硬件…

# semantic i18n key (#2542)
settings-oda-recommended-for-machine = 推荐用于此设备

# semantic i18n key (#2542)
settings-oda-activate = 激活

# semantic i18n key (#2542)
settings-oda-models = 模型

# semantic i18n key (#2542)
settings-oda-model = 模型

# semantic i18n key (#2542)
settings-oda-params = 参数

# semantic i18n key (#2542)
settings-oda-quant = 量化

# semantic i18n key (#2542)
settings-oda-min-vram = 最小显存

# semantic i18n key (#2542)
settings-oda-loading-models = 正在加载模型…

# semantic i18n key (#2542)
settings-date-format-mm-dd-yyyy-us = MM/DD/YYYY（美国）

# semantic i18n key (#2542)
settings-date-format-dd-mm-yyyy-eu-br = DD/MM/YYYY（欧盟/巴西）

# semantic i18n key (#2542)
settings-date-format-yyyy-mm-dd-iso = YYYY-MM-DD（ISO）

# semantic i18n key (#2542)
settings-timezone-america-new-york = 纽约（GMT-5）

# semantic i18n key (#2542)
settings-timezone-america-los-angeles = 洛杉矶（GMT-8）

# semantic i18n key (#2542)
settings-timezone-europe-london = 伦敦（GMT+0）

# semantic i18n key (#2542)
settings-timezone-europe-paris = 巴黎（GMT+1）

# semantic i18n key (#2542)
settings-timezone-asia-tokyo = 东京（GMT+9）

# semantic i18n key (#2542)
settings-desktop-wallpaper = 桌面壁纸

# semantic i18n key (#2542)
settings-desktop-wallpaper-desc = 选择在桌面上显示的背景

# semantic i18n key (#2542)
settings-organizations-bots = 组织和机器人

# semantic i18n key (#2542)
settings-organizations-bots-desc = 管理您的组织和机器人层级

# semantic i18n key (#2542)
settings-current-organization = 当前组织

# semantic i18n key (#2542)
settings-current-organization-desc = 选择要管理的组织

# semantic i18n key (#2542)
settings-loading-organizations = 正在加载组织...

# semantic i18n key (#2542)
settings-bot-tree = 机器人树

# semantic i18n key (#2542)
settings-bot-tree-desc = 属于所选组织的机器人

# semantic i18n key (#2542)
settings-loading-bots = 正在加载机器人...

# semantic i18n key (#2542)
settings-storage-sync-desc = 管理您的存储和同步设置

# semantic i18n key (#2542)
settings-knowledge = 知识

# semantic i18n key (#2542)
settings-drive-files-as-context = 将所有云端硬盘文件用作知识上下文

# semantic i18n key (#2542)
settings-drive-files-as-context-desc = 启用后（默认），您最近的云端硬盘文件名会随每次聊天发送，以便助手将其用作上下文。关闭后可将云端硬盘排除在提示之外。

# semantic i18n key (#2542)
settings-loading-storage-information = 正在加载存储信息...

# semantic i18n key (#2542)
settings-connected-cloud-storage = 已连接的云存储

# semantic i18n key (#2542)
settings-loading-storage-connections = 正在加载存储连接...

# semantic i18n key (#2542)
settings-os-subtitle = 此 shell 安装的底层主机设置和工具

# semantic i18n key (#2542)
settings-os-host-identity-desc = 主机标识、内核和时区

# semantic i18n key (#2542)
settings-os-kernel = 内核

# semantic i18n key (#2542)
settings-os-architecture = 架构

# semantic i18n key (#2542)
settings-os-host-readonly-hint = 此阶段主机名和时区为只读。请使用终端应用或自助终端镜像中的主机工具进行更改。

# semantic i18n key (#2542)
settings-os-services-desc = 启动、停止和重启主机服务

# semantic i18n key (#2542)
settings-os-load-services-hint = 打开面板以加载服务。

# semantic i18n key (#2542)
settings-os-disk-usage-desc = 已挂载文件系统的磁盘使用情况

# semantic i18n key (#2542)
settings-os-load-storage-hint = 打开面板以加载存储。

# semantic i18n key (#2542)
settings-os-log-files = 日志文件

# semantic i18n key (#2542)
settings-os-log-files-desc = 此安装写入的文件及其位置和大小

# semantic i18n key (#2542)
settings-os-load-log-files-hint = 打开面板以加载日志文件。

# semantic i18n key (#2542)
settings-os-tail-log-desc = 查看服务日志的最后 200 行

# semantic i18n key (#2542)
settings-os-reload = 重新加载

# semantic i18n key (#2542)
settings-os-restart-service-title = 重启服务

# semantic i18n key (#2542)
settings-os-the-service = 该服务

# semantic i18n key (#2542)
settings-os-service-name = 服务名称

# semantic i18n key (#2542)
settings-os-restart-action = 重启

# semantic i18n key (#2542)
settings-os-restart-confirm-message = 重启<strong id="os-restart-service-name" data-i18n="settings-os-the-service">该服务</strong>？请输入服务名称以确认；重启操作将写入审计日志。

# semantic i18n key (#2542)
settings-profile-directory-managed = 由 Zitadel 目录管理

# semantic i18n key (#2542)
settings-profile-open-directory-profile = 打开目录配置文件

# semantic i18n key (#2542)
settings-profile-public-title = 公开个人资料

# semantic i18n key (#2542)
settings-profile-public-desc = 您在 General Bots 主页上的显示方式

# semantic i18n key (#2542)
settings-profile-public-display-name = 公开显示名称

# semantic i18n key (#2542)
settings-profile-public-url-label = 您的个人资料 URL 将是

# semantic i18n key (#2542)
settings-profile-coming-soon = 即将推出

# semantic i18n key (#2542)
settings-profile-public-bio = 公开简介

# semantic i18n key (#2542)
settings-profile-public-url-placeholder = your-name.generalbots.org

# semantic i18n key (#2542)
settings-profile-language-region = 语言和区域

# semantic i18n key (#2542)
settings-profile-public-url-hint-html = General Bots 主站中继发布后（#1867），页面将在 <strong><span id="public-profile-url-hint" data-i18n="settings-profile-public-url-placeholder">your-name.generalbots.org</span></strong> 上线。

# semantic i18n key (#2542)
settings-security-directory-note = 身份、会话和双重验证位于 Zitadel 目录中

# semantic i18n key (#2542)
settings-security-open-directory-security = 打开目录安全设置

# semantic i18n key (#2542)
settings-security-connected-devices = 已连接设备

# semantic i18n key (#2542)
settings-security-connected-devices-desc = 曾访问您帐户的设备

# semantic i18n key (#2542)
settings-security-local-data = 本地数据

# semantic i18n key (#2542)
settings-security-local-data-desc = 此安装会将所有内容保留在此设备上。

# semantic i18n key (#2542)
settings-security-export-local-data = 导出本地数据

# semantic i18n key (#2542)
settings-security-export-local-data-desc = 下载存储在此设备上的数据

# semantic i18n key (#2542)
settings-security-clear-local-cache = 清除本地缓存

# semantic i18n key (#2542)
settings-security-clear-local-cache-desc = 移除此浏览器配置文件中存储的缓存设置和草稿

# semantic i18n key (#2542)
settings-security-clear-cache-action = 清除缓存

# semantic i18n key (#2542)
settings-shared-links-subtitle = 选择公共文件链接如何送达给收件人

# semantic i18n key (#2542)
settings-shared-links-loopback-warning-title = 目前只能访问此计算机。

# semantic i18n key (#2542)
settings-shared-links-loopback-warning-body-html = 复制的回环链接（<code>127.0.0.1</code>）会在收件人自己的计算机上打开，而不是在您的计算机上。在此计算机外共享链接之前，请使用局域网地址、设置域名或等待免费的 General Bots 地址。

# semantic i18n key (#2542)
settings-shared-links-lan-title = 此网络

# semantic i18n key (#2542)
settings-shared-links-lan-desc = 您的 Wi-Fi 或有线网络上的任何人

# semantic i18n key (#2542)
settings-shared-links-on-this-computer = 在此计算机上

# semantic i18n key (#2542)
settings-shared-links-on-this-network = 在此网络上

# semantic i18n key (#2542)
settings-shared-links-lan-detail = 适用于同一网络上的任何人。数据不会离开此计算机。

# semantic i18n key (#2542)
settings-shared-links-lan-config-title = 配置

# semantic i18n key (#2542)
settings-shared-links-lan-config-body-html = 此层级默认启用：本地服务器在所有接口上侦听。请从同一网络上的其他设备打开网络地址。如果设备间流量被阻止，请在此计算机上允许入站 TCP 端口 <code id="shared-links-port">8080</code>。

# semantic i18n key (#2542)
settings-shared-links-lan-config-local-html = 如果 <code>.local</code> 名称无法解析，请改用此计算机的 IPv4 地址（例如 <code>http://192.168.1.20:8080</code>）。

# semantic i18n key (#2542)
settings-shared-links-domain-title = 我自己的域名

# semantic i18n key (#2542)
settings-shared-links-domain-desc = 您控制的、可从互联网访问的名称

# semantic i18n key (#2542)
settings-shared-links-public-link = 公开链接

# semantic i18n key (#2542)
settings-shared-links-domain-not-configured = 未配置 — 新链接将回退到上面的本地地址。

# semantic i18n key (#2542)
settings-shared-links-domain-config-title = 如何配置

# semantic i18n key (#2542)
settings-shared-links-domain-config-dns-html = 将 DNS 记录（例如 <code>files.example.com</code>）指向此计算机的公共地址。

# semantic i18n key (#2542)
settings-shared-links-domain-config-url-html = 为此安装设置公共基础 URL：<pre><code>PUBLIC_BASE_URL=https://files.example.com</code></pre> 或在产品文件中设置：<pre><code>public_base_url=https://files.example.com</code></pre>

# semantic i18n key (#2542)
settings-shared-links-domain-config-restart = 重启桌面应用，以便新共享链接使用该域名生成；现有链接将保留其令牌。

# semantic i18n key (#2542)
settings-shared-links-domain-config-hint = 注册商令牌保留在本地保管库中，绝不会写入日志。自动 DNS 刷新和 TLS 证书是此问题中跟踪的域名提供商工作的一部分；在该功能发布之前，请在注册商处保持记录更新。

# semantic i18n key (#2542)
settings-shared-links-free-title = 免费的 General Bots 地址

# semantic i18n key (#2542)
settings-shared-links-free-desc = 由 General Bots 提供的名称，无需域名

# semantic i18n key (#2542)
settings-shared-links-free-unavailable-html = 尚不可用。桌面可离线工作，因此此层级需要免费的 DNS 区域和用于无法直接访问的计算机的中继。它随 #1864 中跟踪的 <code>botpublic</code> 工作一起以禁用状态提供。

# semantic i18n key (#2542)
settings-shared-links-free-what-title = 它将执行的操作

# semantic i18n key (#2542)
settings-shared-links-free-what-body-html = 免费帐户将保留一个名称，而不是您的文件。当无法直接访问某台计算机时，托管中继（<code id="shared-links-relay-host">peer.generalbots.org</code>）仅保留共享文件的加密副本，且该副本会过期。无需手动配置中继。

# semantic i18n key (#2542)
settings-shared-links-peer-title = 主对等开关

# semantic i18n key (#2542)
settings-shared-links-peer-desc-html = 停止此安装的所有 P2P 流量。离线模式会将每个连接保持在 <code>127.0.0.1</code> 上。

# semantic i18n key (#2542)
settings-shared-links-offline-default = 离线（默认）

# semantic i18n key (#2542)
settings-shared-links-p2p-opt-in = P2P（选择加入）

# semantic i18n key (#2542)
settings-shared-links-relay-hint-html = 中继端点是固定的（<code>peer.generalbots.org</code>）。更改此设置可能需要重启应用才能完全生效。

# semantic i18n key (#2542)
settings-shared-links-check-again = 再次检查

# semantic i18n key (#2542)
settings-admin-bots-loading = 正在加载机器人...

# semantic i18n key (#2542)
settings-admin-domain-configuration = 域名配置

# semantic i18n key (#2542)
settings-admin-audit-loading = 正在加载审核日志...

# semantic i18n key (#2542)
settings-updates-subtitle = 检查最新的 GitHub 版本并跳转到官方下载

# semantic i18n key (#2542)
settings-updates-desktop-title = 桌面更新

# semantic i18n key (#2542)
settings-updates-desktop-checks-html = 应用会检查 <code>github.com/generalbots/generalbots/releases/latest</code>。安装始终为可选操作。

# semantic i18n key (#2542)
settings-updates-check-startup = 启动时检查更新

# semantic i18n key (#2542)
settings-updates-check-startup-desc = 每天最多运行一次，并且仅显示通知。关闭此选项时，您仍然可以使用“立即检查”。

# semantic i18n key (#2542)
settings-updates-current-version = 当前版本

# semantic i18n key (#2542)
settings-updates-never = 从不

# semantic i18n key (#2542)
settings-updates-check-now = 立即检查

# semantic i18n key (#2542)
settings-updates-view-release-notes = 查看发行说明

# semantic i18n key (#2542)
settings-updates-not-checked-yet = 尚未检查。

# semantic i18n key (#2542)
settings-versions-subtitle = 组件版本和更新可用性

# semantic i18n key (#2542)
settings-versions-loading = 正在加载版本信息...

# semantic i18n key (#2542)
settings-ledger-description = 每个终端和浏览器操作，以及同意决定、结果和屏幕截图路径。

# semantic i18n key (#2542)
settings-ledger-actions-title = 操作记录

# semantic i18n key (#2542)
settings-ledger-tab-terminal = 终端

# semantic i18n key (#2542)
settings-ledger-tab-browser = 浏览器

# semantic i18n key (#2542)
settings-ledger-tab-budgets = 预算

# semantic i18n key (#2542)
settings-ledger-tab-ttft = TTFT

# semantic i18n key (#2542)
settings-integrations-subtitle = 连接外部服务并管理 API 访问

# semantic i18n key (#2542)
settings-open-integrations = 打开集成

# semantic i18n key (#2542)
settings-personal-access-tokens = 个人访问令牌

# semantic i18n key (#2542)
settings-personal-tokens-desc = 您为以自己身份调用 API 而创建的令牌（<code>gb_…</code>）。模型提供商密钥位于集成应用中。

# semantic i18n key (#2542)
settings-no-api-keys = 尚未创建 API 密钥

# semantic i18n key (#2542)
settings-webhooks-title = Webhook

# semantic i18n key (#2542)
settings-no-webhooks = 未配置 Webhook

# semantic i18n key (#2542)
settings-full-catalog = 完整目录

# semantic i18n key (#2542)
settings-loading-mcp-servers = 正在加载 MCP 服务器

# semantic i18n key (#2542)
settings-agent-skills = 代理技能

# semantic i18n key (#2542)
settings-import-skill = 导入技能

# semantic i18n key (#2542)
settings-loading-skills = 正在加载技能

# semantic i18n key (#2542)
settings-ai-accounts-subtitle = Claude Code 和 ChatGPT 套餐订阅按用户计费。凭据保留在您的代理 VM 或本地保管库中；此处仅存储状态和显示元数据。

# semantic i18n key (#2542)
settings-subscription-accounts = 订阅帐户

# semantic i18n key (#2542)
settings-ai-accounts-desc = 连接帐户，然后将其分配给 Vibe 代理类型

# semantic i18n key (#2542)
settings-loading-ai-accounts = 正在加载 AI 帐户…

# semantic i18n key (#2542)
settings-vibe-subtitle = 为每种 Vibe 代理类型分配 LLM 提供商。提供商在“集成 → 模型提供商 (BYOK)”或云商店中定义。

# semantic i18n key (#2542)
settings-agent-llm-providers = 代理 LLM 提供商

# semantic i18n key (#2542)
settings-agent-llm-providers-desc = 每种代理类型都会将其运行路由到此处分支所选的提供商

# semantic i18n key (#2542)
settings-loading-agent-providers = 正在加载代理提供商…

# semantic i18n key (#2542)
settings-privacy-subtitle = 管理您的数据和隐私设置

# semantic i18n key (#2542)
settings-privacy-preferences = 隐私偏好

# semantic i18n key (#2542)
settings-profile-visibility = 个人资料可见性

# semantic i18n key (#2542)
settings-profile-visibility-desc = 谁可以查看您的个人资料

# semantic i18n key (#2542)
settings-contacts-only = 仅限联系人

# semantic i18n key (#2542)
settings-online-status = 在线状态

# semantic i18n key (#2542)
settings-online-status-desc = 显示您的在线状态

# semantic i18n key (#2542)
settings-read-receipts = 已读回执

# semantic i18n key (#2542)
settings-read-receipts-desc = 让其他人知道您何时已读消息

# semantic i18n key (#2542)
settings-your-data = 您的数据

# semantic i18n key (#2542)
settings-export-data = 导出数据

# semantic i18n key (#2542)
settings-export-data-desc = 以可移植格式下载您的所有数据

# semantic i18n key (#2542)
settings-delete-account-title = 永久删除您的帐户及所有相关数据

# semantic i18n key (#2542)
settings-delete-my-account = 删除我的帐户

# semantic i18n key (#2542)
settings-memory-subtitle = 助手在多次对话中记住的关于您的信息

# semantic i18n key (#2542)
settings-remembered-facts = 已记住的事实

# semantic i18n key (#2542)
settings-memory-facts-desc = 这些事实会随每条消息发送，以使回答在对话之间保持一致。请移除您不希望助手再使用的任何内容。

# semantic i18n key (#2542)
settings-memory-all-scopes = 所有范围

# semantic i18n key (#2542)
settings-memory-global-all-bots = 全局（所有机器人）

# semantic i18n key (#2542)
settings-memory-bot-specific = 特定机器人

# semantic i18n key (#2542)
settings-default-scope = 默认

# semantic i18n key (#2542)
settings-manage-memories = 管理记忆

# semantic i18n key (#2542)
settings-manage-memories-desc = 记忆由助手创建。您可以在此处编辑或删除它们。

# semantic i18n key (#2542)
settings-no-plan-info = 此部署不管理套餐信息。

# semantic i18n key (#2542)
settings-manage-plan-on-cloud = 在云端管理套餐

# semantic i18n key (#2542)
settings-add-method = 添加方式

# semantic i18n key (#2542)
settings-no-payment-method = 没有已保存的付款方式。

# semantic i18n key (#2542)
settings-add-method-on-cloud = 在云端添加方式

# semantic i18n key (#2542)
settings-no-invoices = 此帐户没有发票。

# semantic i18n key (#2542)
settings-view-invoices-on-cloud = 在云端查看发票

# semantic i18n key (#2542)
settings-skills-placeholder =
    粘贴 SKILL.md 包：
    ---
    name: deploy-check
    description: 在发布前验证部署
    ---
    1. …
settings-memory-search-placeholder = 搜索记忆…
settings-notification-recipient-placeholder = you@example.com

# semantic i18n key (#2542)
settings-profile-placeholder-display-name = 张三
settings-profile-placeholder-username = zhangsan
settings-profile-placeholder-email = zhangsan@example.com
settings-profile-placeholder-public-display-name = 您的姓名
settings-profile-placeholder-public-bio = 在您的公开个人资料中显示的简短描述...
settings-profile-placeholder-phone = +86 138 0000 0000
settings-profile-placeholder-location = 城市，国家
settings-profile-placeholder-website = https://example.com

# semantic i18n key (#2542)
settings-placeholder-dns-hostname = mybot.example.com
settings-placeholder-dns-target = 192.168.1.1 或 target.domain.com
settings-placeholder-api-key-name = 生产密钥
settings-placeholder-api-key-scopes = user, admin
settings-placeholder-api-key-expiry = 90
settings-placeholder-webhook-url = https://example.com/hooks/gb
settings-placeholder-webhook-events = user.created, order.completed
