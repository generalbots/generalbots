# =============================================================================
# General Bots - Authentication Translations (English)
# =============================================================================
# Authentication, Passkey/WebAuthn, and security interface translations
# =============================================================================

# -----------------------------------------------------------------------------
# Authentication General
# -----------------------------------------------------------------------------
auth-title = 認証
auth-login = ログイン
auth-logout = ログアウト
auth-signup = サインアップ
auth-welcome = ようこそ
auth-welcome-back = { $name }さん、おかえりなさい！
auth-session-expired = セッションの有効期限が切れました
auth-session-timeout = セッションタイムアウト: { $minutes } 分

# -----------------------------------------------------------------------------
# Login Form
# -----------------------------------------------------------------------------
auth-login-title = アカウントにサインインする
auth-login-subtitle = 続行するには資格情報を入力してください
auth-login-email = メールアドレス
auth-login-username = ユーザー名
auth-login-password = パスワード
auth-login-remember = 私を覚えていてください
auth-login-forgot = パスワードをお忘れですか?
auth-login-submit = サインイン
auth-login-loading = サインイン中...
auth-login-or = または続行してください
auth-login-no-account = アカウントをお持ちでない場合は、
auth-login-create-account = アカウントを作成する

# -----------------------------------------------------------------------------
# Passkey/WebAuthn
# -----------------------------------------------------------------------------
passkey-title = パスキー
passkey-subtitle = 安全なパスワード不要の認証
passkey-description = パスキーはデバイスの生体認証または PIN を使用して、安全でフィッシング耐性のあるサインインを実現します。
passkey-what-is = パスキーとは何ですか?
passkey-benefits = パスキーの利点
passkey-benefit-secure = パスワードより安全
passkey-benefit-easy = 使いやすい - パスワードを覚える必要はありません
passkey-benefit-fast = 生体認証による高速サインイン
passkey-benefit-phishing = フィッシング攻撃に対する耐性

# -----------------------------------------------------------------------------
# Passkey Registration
# -----------------------------------------------------------------------------
passkey-register-title = パスキーの設定
passkey-register-subtitle = より速く、より安全なサインインのためのパスキーを作成する
passkey-register-description = デバイスは、指紋、顔、または画面ロックを使用して本人確認を求めるメッセージを表示します。
passkey-register-button = パスキーの作成
passkey-register-name = パスキー名
passkey-register-name-placeholder = 例: MacBook Pro、iPhone
passkey-register-name-hint = 後で識別できるようにパスキーに名前を付けます
passkey-register-loading = パスキーを設定しています...
passkey-register-verifying = デバイスで確認しています...
passkey-register-success = パスキーが正常に作成されました
passkey-register-error = パスキーの作成に失敗しました
passkey-register-cancelled = パスキーの設定がキャンセルされました
passkey-register-not-supported = お使いのブラウザはパスキーをサポートしていません

# -----------------------------------------------------------------------------
# Passkey Authentication
# -----------------------------------------------------------------------------
passkey-login-title = パスキーを使用してサインインする
passkey-login-subtitle = パスキーを使用して安全なパスワードなしのサインインを行う
passkey-login-button = パスキーを使用してサインインする
passkey-login-loading = 認証中...
passkey-login-verifying = パスキーを確認しています...
passkey-login-success = 正常にサインインしました
passkey-login-error = 認証に失敗しました
passkey-login-cancelled = 認証がキャンセルされました
passkey-login-no-passkeys = このアカウントのパスキーは見つかりませんでした
passkey-login-try-another = 別の方法を試してください

# -----------------------------------------------------------------------------
# Passkey Management
# -----------------------------------------------------------------------------
passkey-manage-title = パスキーの管理
passkey-manage-subtitle = 登録されたパスキーを表示および管理する
passkey-manage-count = { $count ->
    [one] { $count } passkey registered
   *[other] { $count } passkeys registered
}
passkey-manage-add = 新しいパスキーを追加
passkey-manage-rename = 名前の変更
passkey-manage-delete = 削除
passkey-manage-created = { $date } を作成しました
passkey-manage-last-used = 最後に使用したのは { $date }
passkey-manage-never-used = 一度も使用されていない
passkey-manage-this-device = このデバイス
passkey-manage-cross-platform = クロスプラットフォーム
passkey-manage-platform = プラットフォーム認証システム
passkey-manage-security-key = セキュリティキー
passkey-manage-empty = パスキーが登録されていません
passkey-manage-empty-description = パスキーを追加して、より速く、より安全なサインインを実現します

# -----------------------------------------------------------------------------
# Passkey Deletion
# -----------------------------------------------------------------------------
passkey-delete-title = パスキーの削除
passkey-delete-confirm = このパスキーを削除してもよろしいですか?
passkey-delete-warning = このパスキーを使用してサインインすることはできなくなります
passkey-delete-last-warning = これが唯一のパスキーです。削除後はパスワード認証が必要になります。
passkey-delete-success = パスキーが正常に削除されました
passkey-delete-error = パスキーの削除に失敗しました

# -----------------------------------------------------------------------------
# Password Fallback
# -----------------------------------------------------------------------------
passkey-fallback-title = 代わりにパスワードを使用してください
passkey-fallback-description = パスキーを使用できない場合は、パスワードを使用してサインインできます
passkey-fallback-button = パスワードを使用する
passkey-fallback-or-passkey = またはパスキーを使用してサインインします
passkey-fallback-setup-prompt = 次回より速くサインインできるようにパスキーを設定する
passkey-fallback-setup-later = たぶん後で
passkey-fallback-setup-now = 今すぐセットアップ
passkey-fallback-locked = アカウントが一時的にロックされました
passkey-fallback-locked-description = 失敗した試行が多すぎます。 { $minutes } 分後にもう一度お試しください。
passkey-fallback-attempts = 残り{ $remaining } 回の試行回数

# -----------------------------------------------------------------------------
# Multi-Factor Authentication
# -----------------------------------------------------------------------------
mfa-title = 二要素認証
mfa-subtitle = アカウントに追加のセキュリティ層を追加します
mfa-enabled = 二要素認証が有効になっています
mfa-disabled = 二要素認証が無効になっています
mfa-enable = 2FA を有効にする
mfa-disable = 2FA を無効にする
mfa-setup = 2FA のセットアップ
mfa-verify = コードの検証
mfa-code = 検証コード
mfa-code-placeholder = 6桁のコードを入力してください
mfa-code-sent = コードは { $destination } に送信されました
mfa-code-expired = コードの有効期限が切れています
mfa-code-invalid = 無効なコード
mfa-resend = コードを再送信する
mfa-resend-in = { $seconds } 秒以内に再送信します
mfa-methods = 認証方法
mfa-method-app = 認証アプリ
mfa-method-sms = SMS
mfa-method-email = 電子メール
mfa-method-passkey = パスキー
mfa-backup-codes = バックアップコード
mfa-backup-codes-description = これらのコードを安全な場所に保存してください。各コードは 1 回のみ使用できます。
mfa-backup-codes-remaining = { $count } バックアップ コードが残っています
mfa-backup-codes-generate = 新しいコードの生成
mfa-backup-codes-download = ダウンロードコード
mfa-backup-codes-copy = コードをコピーする

# -----------------------------------------------------------------------------
# Password Management
# -----------------------------------------------------------------------------
password-title = パスワード
password-change = パスワードの変更
password-current = 現在のパスワード
password-new = 新しいパスワード
password-confirm = 新しいパスワードを確認する
password-requirements = パスワード要件
password-requirement-length = 少なくとも { $length } 文字
password-requirement-uppercase = 少なくとも 1 つの大文字
password-requirement-lowercase = 少なくとも 1 つの小文字
password-requirement-number = 少なくとも 1 つの数字
password-requirement-special = 少なくとも 1 つの特殊文字
password-strength = パスワードの強度
password-strength-weak = 弱い
password-strength-fair = フェア
password-strength-good = 良い
password-strength-strong = 強い
password-match = パスワードが一致する
password-mismatch = パスワードが一致しません
password-changed = パスワードが正常に変更されました
password-change-error = パスワードの変更に失敗しました

# -----------------------------------------------------------------------------
# Password Reset
# -----------------------------------------------------------------------------
password-reset-title = パスワードのリセット
password-reset-subtitle = メールアドレスを入力してリセットリンクを受信してください
password-reset-email-sent = パスワードリセットメールを送信しました
password-reset-email-sent-description = パスワードをリセットする手順については、電子メールを確認してください
password-reset-invalid-token = リセットリンクが無効または期限切れです
password-reset-success = パスワードが正常にリセットされました
password-reset-error = パスワードのリセットに失敗しました

# -----------------------------------------------------------------------------
# Session Management
# -----------------------------------------------------------------------------
session-title = アクティブなセッション
session-subtitle = デバイス間でアクティブなセッションを管理する
session-current = 現在のセッション
session-device = デバイス
session-location = 場所
session-last-active = 最終アクティブ
session-ip-address = IPアドレス
session-browser = ブラウザ
session-os = オペレーティングシステム
session-sign-out = サインアウト
session-sign-out-all = 他のすべてのセッションからサインアウトする
session-sign-out-confirm = このセッションからサインアウトしてもよろしいですか?
session-sign-out-all-confirm = 他のすべてのセッションからサインアウトしてもよろしいですか?

# -----------------------------------------------------------------------------
# Security Settings
# -----------------------------------------------------------------------------
security-title = セキュリティ
security-subtitle = アカウントのセキュリティ設定を管理する
security-overview = セキュリティの概要
security-last-login = 最終サインイン
security-password-last-changed = パスワードの最終変更日
security-security-checkup = セキュリティ診断
security-checkup-description = セキュリティ設定を見直してください
security-recommendation = おすすめ
security-add-passkey = より安全なサインインのためにパスキーを追加する
security-enable-mfa = 二要素認証を有効にする
security-update-password = パスワードを定期的に更新してください

# -----------------------------------------------------------------------------
# Error Messages
# -----------------------------------------------------------------------------
auth-error-invalid-credentials = 無効な電子メールまたはパスワード
auth-error-account-locked = アカウントがロックされています。サポートにお問い合わせください。
auth-error-account-disabled = アカウントが無効になりました
auth-error-email-not-verified = メールアドレスを確認してください
auth-error-too-many-attempts = 失敗した試行が多すぎます。後でもう一度試してください。
auth-error-network = ネットワークエラー。接続を確認してください。
auth-error-server = サーバーエラー。後でもう一度試してください。
auth-error-unknown = 不明なエラーが発生しました
auth-error-session-invalid = 無効なセッションです。再度サインインしてください。
auth-error-token-expired = セッションの有効期限が切れました。再度サインインしてください。
auth-error-unauthorized = このアクションを実行する権限がありません

# -----------------------------------------------------------------------------
# Success Messages
# -----------------------------------------------------------------------------
auth-success-login = 正常にサインインしました
auth-success-logout = 正常にサインアウトしました
auth-success-signup = アカウントが正常に作成されました
auth-success-password-changed = パスワードが正常に変更されました
auth-success-email-verified = メールが正常に認証されました
auth-success-mfa-enabled = 二要素認証が有効になっています
auth-success-mfa-disabled = 二要素認証が無効になっています
auth-success-session-terminated = セッションは正常に終了しました

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
auth-notify-new-login = { $location } の { $device } からの新規サインイン
auth-notify-password-changed = パスワードが変更されました
auth-notify-mfa-enabled = 二要素認証が有効になりました
auth-notify-passkey-added = 新しいパスキーがアカウントに追加されました
auth-notify-suspicious-activity = アカウントで不審なアクティビティが検出されました
