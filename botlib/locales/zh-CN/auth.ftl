# =============================================================================
# General Bots - Authentication Translations (English)
# =============================================================================
# Authentication, Passkey/WebAuthn, and security interface translations
# =============================================================================

# -----------------------------------------------------------------------------
# Authentication General
# -----------------------------------------------------------------------------
auth-title = 认证
auth-login = 登录
auth-logout = 退出
auth-signup = 注册
auth-welcome = 欢迎
auth-welcome-back = 欢迎回来，{ $name }！
auth-session-expired = 您的会话已过期
auth-session-timeout = { $minutes } 分钟后会话超时

# -----------------------------------------------------------------------------
# Login Form
# -----------------------------------------------------------------------------
auth-login-title = 登录您的帐户
auth-login-subtitle = 输入您的凭据以继续
auth-login-email = 电子邮件地址
auth-login-username = 用户名
auth-login-password = 密码
auth-login-remember = 记住我
auth-login-forgot = 忘记密码？
auth-login-submit = 登录
auth-login-loading = 正在登录...
auth-login-or = 或继续
auth-login-no-account = 没有帐户？
auth-login-create-account = 创建帐户

# -----------------------------------------------------------------------------
# Passkey/WebAuthn
# -----------------------------------------------------------------------------
passkey-title = 万能钥匙
passkey-subtitle = 安全、无密码的身份验证
passkey-description = 密钥使用您设备的生物识别或 PIN 码进行安全、防网络钓鱼的登录
passkey-what-is = 什么是万能钥匙？
passkey-benefits = 密钥的好处
passkey-benefit-secure = 比密码更安全
passkey-benefit-easy = 易于使用 - 无需记住密码
passkey-benefit-fast = 使用生物识别技术快速登录
passkey-benefit-phishing = 抵御网络钓鱼攻击

# -----------------------------------------------------------------------------
# Passkey Registration
# -----------------------------------------------------------------------------
passkey-register-title = 设置密钥
passkey-register-subtitle = 创建密钥以实现更快、更安全的登录
passkey-register-description = 您的设备将要求您使用指纹、面部或屏幕锁定来验证您的身份
passkey-register-button = 创建密钥
passkey-register-name = 密钥名称
passkey-register-name-placeholder = 例如，MacBook Pro、iPhone
passkey-register-name-hint = 为您的密钥命名以便稍后识别
passkey-register-loading = 正在设置密钥...
passkey-register-verifying = 正在使用您的设备验证...
passkey-register-success = 密钥创建成功
passkey-register-error = 创建密钥失败
passkey-register-cancelled = 密钥设置已取消
passkey-register-not-supported = 您的浏览器不支持密钥

# -----------------------------------------------------------------------------
# Passkey Authentication
# -----------------------------------------------------------------------------
passkey-login-title = 使用密码登录
passkey-login-subtitle = 使用您的密码进行安全、无密码登录
passkey-login-button = 使用密码登录
passkey-login-loading = 正在验证...
passkey-login-verifying = 正在验证密钥...
passkey-login-success = 登录成功
passkey-login-error = 认证失败
passkey-login-cancelled = 认证已取消
passkey-login-no-passkeys = 找不到该帐户的密钥
passkey-login-try-another = 尝试另一种方法

# -----------------------------------------------------------------------------
# Passkey Management
# -----------------------------------------------------------------------------
passkey-manage-title = 管理密钥
passkey-manage-subtitle = 查看和管理您注册的密钥
passkey-manage-count = { $计数 ->
    [one] { $count } passkey registered
   *[other] { $count } passkeys registered
}
passkey-manage-add = 添加新密钥
passkey-manage-rename = 重命名
passkey-manage-delete = 删除
passkey-manage-created = 创建{ $date }
passkey-manage-last-used = 最后使用{ $date }
passkey-manage-never-used = 从未使用过
passkey-manage-this-device = 这个装置
passkey-manage-cross-platform = 跨平台
passkey-manage-platform = 平台认证器
passkey-manage-security-key = 安全密钥
passkey-manage-empty = 没有注册密钥
passkey-manage-empty-description = 添加密钥以实现更快、更安全的登录

# -----------------------------------------------------------------------------
# Passkey Deletion
# -----------------------------------------------------------------------------
passkey-delete-title = 删除密钥
passkey-delete-confirm = 您确定要删除此密钥吗？
passkey-delete-warning = 您将无法再使用此密钥登录
passkey-delete-last-warning = 这是您唯一的密码。删除后需要使用密码验证。
passkey-delete-success = 密钥删除成功
passkey-delete-error = 删除密钥失败

# -----------------------------------------------------------------------------
# Password Fallback
# -----------------------------------------------------------------------------
passkey-fallback-title = 使用密码代替
passkey-fallback-description = 如果您无法使用密码，可以使用密码登录
passkey-fallback-button = 使用密码
passkey-fallback-or-passkey = 或使用密码登录
passkey-fallback-setup-prompt = 设置密钥以便下次更快登录
passkey-fallback-setup-later = 也许稍后
passkey-fallback-setup-now = 立即设置
passkey-fallback-locked = 帐户暂时被锁定
passkey-fallback-locked-description = 失败的尝试太多了。请在 { $minutes } 分钟后重试。
passkey-fallback-attempts = 剩余{ $remaining } 尝试次数

# -----------------------------------------------------------------------------
# Multi-Factor Authentication
# -----------------------------------------------------------------------------
mfa-title = 双因素身份验证
mfa-subtitle = 为您的帐户添加额外的安全层
mfa-enabled = 已启用双因素身份验证
mfa-disabled = 双因素身份验证已禁用
mfa-enable = 启用 2FA
mfa-disable = 禁用 2FA
mfa-setup = 设置 2FA
mfa-verify = 验证码
mfa-code = 验证码
mfa-code-placeholder = 输入 6 位数字代码
mfa-code-sent = 代码已发送至{ $destination }
mfa-code-expired = 代码已过期
mfa-code-invalid = 代码无效
mfa-resend = 重新发送代码
mfa-resend-in = { $seconds }秒后重新发送
mfa-methods = 认证方式
mfa-method-app = 验证器应用程序
mfa-method-sms = 短信
mfa-method-email = 电子邮件
mfa-method-passkey = 万能钥匙
mfa-backup-codes = 备份代码
mfa-backup-codes-description = 将这些代码保存在安全的地方。每个代码只能使用一次。
mfa-backup-codes-remaining = 剩余{ $count }备份码
mfa-backup-codes-generate = 生成新代码
mfa-backup-codes-download = 下载代码
mfa-backup-codes-copy = 复制代码

# -----------------------------------------------------------------------------
# Password Management
# -----------------------------------------------------------------------------
password-title = 密码
password-change = 更改密码
password-current = 当前密码
password-new = 新密码
password-confirm = 确认新密码
password-requirements = 密码要求
password-requirement-length = 至少{ $length }个字符
password-requirement-uppercase = 至少一个大写字母
password-requirement-lowercase = 至少一个小写字母
password-requirement-number = 至少一个数字
password-requirement-special = 至少一个特殊字符
password-strength = 密码强度
password-strength-weak = 弱
password-strength-fair = 公平
password-strength-good = 好
password-strength-strong = 强
password-match = 密码匹配
password-mismatch = 密码不匹配
password-changed = 密码修改成功
password-change-error = 更改密码失败

# -----------------------------------------------------------------------------
# Password Reset
# -----------------------------------------------------------------------------
password-reset-title = 重置密码
password-reset-subtitle = 输入您的电子邮件以接收重置链接
password-reset-email-sent = 密码重置电子邮件已发送
password-reset-email-sent-description = 检查您的电子邮件以获取重置密码的说明
password-reset-invalid-token = 重置链接无效或过期
password-reset-success = 密码重置成功
password-reset-error = 重置密码失败

# -----------------------------------------------------------------------------
# Session Management
# -----------------------------------------------------------------------------
session-title = 活跃会话
session-subtitle = 管理跨设备的活动会话
session-current = 当前会话
session-device = 设备
session-location = 地点
session-last-active = 最后活跃
session-ip-address = IP地址
session-browser = 浏览器
session-os = 操作系统
session-sign-out = 退出
session-sign-out-all = 注销所有其他会话
session-sign-out-confirm = 您确定要退出此会话吗？
session-sign-out-all-confirm = 您确定要退出所有其他会话吗？

# -----------------------------------------------------------------------------
# Security Settings
# -----------------------------------------------------------------------------
security-title = 安全性
security-subtitle = 管理您的帐户安全设置
security-overview = 安全概述
security-last-login = 最后登录
security-password-last-changed = 密码上次更改
security-security-checkup = 安全检查
security-checkup-description = 检查您的安全设置
security-recommendation = 推荐
security-add-passkey = 添加密钥以更安全地登录
security-enable-mfa = 启用双因素身份验证
security-update-password = 定期更新您的密码

# -----------------------------------------------------------------------------
# Error Messages
# -----------------------------------------------------------------------------
auth-error-invalid-credentials = 电子邮件或密码无效
auth-error-account-locked = 帐户已被锁定。请联系支持人员。
auth-error-account-disabled = 帐户已被禁用
auth-error-email-not-verified = 请验证您的电子邮件地址
auth-error-too-many-attempts = 失败的尝试太多了。请稍后重试。
auth-error-network = 网络错误。请检查您的连接。
auth-error-server = 服务器错误。请稍后重试。
auth-error-unknown = 发生未知错误
auth-error-session-invalid = 无效会话。请重新登录。
auth-error-token-expired = 您的会话已过期。请重新登录。
auth-error-unauthorized = 您无权执行此操作

# -----------------------------------------------------------------------------
# Success Messages
# -----------------------------------------------------------------------------
auth-success-login = 登录成功
auth-success-logout = 退出成功
auth-success-signup = 账户创建成功
auth-success-password-changed = 密码修改成功
auth-success-email-verified = 邮箱验证成功
auth-success-mfa-enabled = 启用双因素身份验证
auth-success-mfa-disabled = 禁用双因素身份验证
auth-success-session-terminated = 会话成功终止

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
auth-notify-new-login = { $device } { $location } 新登录
auth-notify-password-changed = 您的密码已更改
auth-notify-mfa-enabled = 已启用双因素身份验证
auth-notify-passkey-added = 新密码已添加到您的帐户
auth-notify-suspicious-activity = 在您的帐户中检测到可疑活动
