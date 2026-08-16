# General Bots - Error Messages (English)
# This file contains all error message translations

# =============================================================================
# HTTP Errors
# =============================================================================

error-http-400 = 要求の形式が正しくありません。入力内容を確認してください。
error-http-401 = 認証が必要です。ログインしてください。
error-http-403 = このリソースにアクセスする権限がありません。
error-http-404 = { $entity } が見つかりません。
error-http-409 = 紛争: { $message }
error-http-429 = リクエストが多すぎます。 { $seconds } 秒お待ちください。
error-http-500 = 内部サーバーエラー。後でもう一度試してください。
error-http-502 = ゲートウェイが不良です。サーバーは無効な応答を受け取りました。
error-http-503 = サービスが一時的に利用できなくなりました。後でもう一度試してください。
error-http-504 = リクエストは { $milliseconds }ms 後にタイムアウトしました。

# =============================================================================
# Validation Errors
# =============================================================================

error-validation-required = { $field }は必須です。
error-validation-email = 有効なメールアドレスを入力してください。
error-validation-url = 有効な URL を入力してください。
error-validation-phone = 有効な電話番号を入力してください。
error-validation-min-length = { $field } は少なくとも { $min } 文字でなければなりません。
error-validation-max-length = { $field } は { $max } 文字以下にする必要があります。
error-validation-min-value = { $field } は少なくとも { $min } でなければなりません。
error-validation-max-value = { $field } は { $max } 以下でなければなりません。
error-validation-pattern = { $field } 形式が無効です。
error-validation-unique = { $field } はすでに存在します。
error-validation-mismatch = { $field } は { $other } と一致しません。
error-validation-date-format = 有効な日付を { $format } の形式で入力してください。
error-validation-date-past = { $field }は過去のはずです。
error-validation-date-future = { $field } は未来のはずです。

# =============================================================================
# Authentication Errors
# =============================================================================

error-auth-invalid-credentials = メールアドレスまたはパスワードが無効です。
error-auth-account-locked = あなたのアカウントはロックされています。サポートにお問い合わせください。
error-auth-account-disabled = あなたのアカウントは無効になっています。
error-auth-session-expired = セッションの有効期限が切れました。再度ログインしてください。
error-auth-token-invalid = トークンが無効または期限切れです。
error-auth-token-missing = 認証トークンが必要です。
error-auth-mfa-required = 多要素認証が必要です。
error-auth-mfa-invalid = 無効な確認コードです。
error-auth-password-weak = パスワードが弱すぎます。より強力なパスワードを使用してください。
error-auth-password-expired = パスワードの有効期限が切れています。リセットしてください。

# =============================================================================
# Configuration Errors
# =============================================================================

error-config = 構成エラー: { $message }
error-config-missing = 設定がありません: { $key }
error-config-invalid = { $key } の構成値が無効です: { $reason }
error-config-file-not-found = 構成ファイルが見つかりません: { $path }
error-config-parse = 構成の解析に失敗しました: { $message }

# =============================================================================
# Database Errors
# =============================================================================

error-database = データベース エラー: { $message }
error-database-connection = データベースへの接続に失敗しました。
error-database-timeout = データベース操作がタイムアウトしました。
error-database-constraint = データベース制約違反: { $constraint }
error-database-duplicate = この { $field } のレコードはすでに存在します。
error-database-migration = データベースの移行に失敗しました: { $message }

# =============================================================================
# File & Storage Errors
# =============================================================================

error-file-not-found = ファイルが見つかりません: { $filename }
error-file-too-large = ファイルが大きすぎます。最大サイズは{ $maxSize }です。
error-file-type-not-allowed = ファイルの種類は許可されていません。許可されるタイプ: { $allowedTypes }。
error-file-upload-failed = ファイルのアップロードに失敗しました: { $message }
error-file-read = ファイルの読み取りに失敗しました: { $message }
error-file-write = ファイルの書き込みに失敗しました: { $message }
error-storage-full = ストレージ割り当てを超過しました。
error-storage-unavailable = ストレージサービスは利用できません。

# =============================================================================
# Network & External Service Errors
# =============================================================================

error-network = ネットワークエラー: { $message }
error-network-timeout = 接続がタイムアウトしました。
error-network-unreachable = サーバーに到達できません。
error-service-unavailable = サービスが利用できません: { $service }
error-external-api = 外部 API エラー: { $message }
error-rate-limit = レート制限あり。 { $seconds } 秒後に再試行してください。

# =============================================================================
# Bot & Dialog Errors
# =============================================================================

error-bot-not-found = ボットが見つかりません: { $botId }
error-bot-disabled = このボットは現在無効になっています。
error-bot-script-error = { $line } 行目のスクリプト エラー: { $message }
error-bot-timeout = ボットの応答がタイムアウトしました。
error-bot-quota-exceeded = ボットの使用量の割り当てを超過しました。
error-dialog-not-found = ダイアログが見つかりません: { $dialogId }
error-dialog-invalid = 無効なダイアログ構成: { $message }

# =============================================================================
# LLM & AI Errors
# =============================================================================

error-llm-unavailable = 現在AIサービスはご利用いただけません。
error-llm-timeout = AI リクエストがタイムアウトしました。
error-llm-rate-limit = AI レート制限を超えました。再試行する前にお待ちください。
error-llm-content-filter = コンテンツは安全ガイドラインによってフィルタリングされました。
error-llm-context-length = 入力が長すぎます。メッセージを短くしてください。
error-llm-invalid-response = AI サービスから無効な応答を受け取りました。
error-llm-empty-response = 申し訳ありませんが、現在あなたのメッセージを処理できませんでした。数秒後にもう一度試してください。

# =============================================================================
# Email Errors
# =============================================================================

error-email-send-failed = 電子メールの送信に失敗しました: { $message }
error-email-invalid-recipient = 無効な受信者の電子メール アドレス: { $email }
error-email-attachment-failed = ファイルの添付に失敗しました: { $filename }
error-email-template-not-found = 電子メール テンプレートが見つかりません: { $template }

# =============================================================================
# Calendar & Scheduling Errors
# =============================================================================

error-calendar-conflict = タイムスロットが既存のイベントと競合します。
error-calendar-past-date = 過去のイベントをスケジュールすることはできません。
error-calendar-invalid-recurrence = 無効な再発パターンです。
error-calendar-event-not-found = イベントが見つかりません: { $eventId }

# =============================================================================
# Task Errors
# =============================================================================

error-task-not-found = タスクが見つかりません: { $taskId }
error-task-already-completed = タスクはすでに完了しています。
error-task-circular-dependency = タスク内で循環依存関係が検出されました。
error-task-invalid-status = タスクの状態遷移が不正です。

# =============================================================================
# Permission Errors
# =============================================================================

error-permission-denied = このアクションを実行する権限がありません。
error-permission-resource = この { $resource } にはアクセスできません。
error-permission-action = これ { $action } を { $resource } することはできません。
error-permission-owner-only = このアクションを実行できるのは所有者のみです。

# =============================================================================
# Generic Errors
# =============================================================================

error-internal = 内部エラー: { $message }
error-unexpected = 予期しないエラーが発生しました。もう一度試してください。
error-not-implemented = この機能はまだ実装されていません。
error-maintenance = システムメンテナンス中です。後でもう一度試してください。
error-unknown = 不明なエラーが発生しました。
