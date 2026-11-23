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

error-locale-invalid = 要求された言語は無効です。
error-locale-unsupported = この言語は利用できません。
error-locale-persist-failed = 言語設定を保存できませんでした。
error-bot-id-required = ボット ID は必須です。
error-bot-id-invalid = ボット ID が無効です。
error-bot-name-required = ボット名は必須です。
error-token-invalid-format = トークンの形式が無効です。
error-token-signing-failed = トークンに署名できませんでした。
error-token-signature-invalid = トークンの署名が無効です。
error-token-payload-invalid = トークンのペイロードが無効です。
error-token-payload-json-invalid = トークンのペイロードが有効な JSON ではありません。
error-cloud-token-required = cloud_token を指定してください。
error-llm-subscription-token-missing = サブスクリプショントークンがありません。
error-llm-token-store-unavailable = トークンストアを利用できません。
error-llm-subscription-token-expired = サブスクリプショントークンの有効期限が切れました。メッセージを再送してください。
error-llm-prompt-missing = リクエストにプロンプトがありません。
error-llm-account-unlinked = 先に設定 → AI アカウントで Claude または ChatGPT アカウントを接続してください。
attendance-no-attendant-matching-skills = No attendant was found with the required skills.
attendance-response-sent = Response sent.
error-agent-drive-bad-bot = The bot record for this conversation is not valid.
error-agent-drive-bad-owner = The conversation owner record is not valid.
error-agent-drive-bad-path = The Drive path returned by the store is not valid.
error-agent-drive-conversation-not-found = Conversation not found.
error-agent-drive-database-unavailable = The file store is temporarily unavailable.
error-agent-drive-file-missing = The file is not in the workspace output folder.
error-agent-drive-forbidden = This conversation belongs to another user.
error-agent-drive-link-failed = Could not create the share link.
error-agent-drive-lookup-failed = Could not read the conversation. Please try again.
error-agent-drive-named-files-missing = The requested files are not in the workspace output folder.
error-agent-drive-no-files = There are no files in the workspace output folder.
error-agent-drive-save-failed = Could not save any file to Drive.
error-agent-drive-save-one-failed = Could not save the file to Drive.
error-agent-drive-workspace-stopped = The agent workspace is not running.
error-agent-vm-disabled = Agent/VM is disabled for this conversation. Turn it on in the chat controls to use the workspace.
error-ai-account-agent-invalid = The agent must be reasoning, agentic or fast.
error-ai-account-not-connected-chat = Connect the account before selecting it for chat.
error-ai-account-not-connected-vibe = Connect the account before assigning it to Vibe.
error-ai-account-unknown-provider = The AI account provider is unknown.
error-ai-account-workspace-unavailable = The agent workspace is not available in this build.
error-attendance-attendant-id-required = An attendant id is required.
error-attendance-session-id-invalid = The session id is invalid.
error-attendance-session-not-found = Session not found.
error-attendance-webhook-not-found = Webhook not found.
error-autotask-fields-required = Title, schedule and command are required.
error-autotask-intent-required = Describe what the task should do.
error-autotask-kind-invalid = The kind must be 'cron' or 'systemd'.
error-autotask-schedule-invalid = The schedule must be a valid five-field cron expression.
error-autotask-not-found = Autotask not found.
error-collab-comment-body-required = A comment body is required.
error-collab-comment-not-found = The comment was not found.
error-collab-comment-resolve-forbidden = The comment was not found, or it is not yours to resolve.
error-collab-comment-too-long = The comment is limited to 8000 characters.
error-collab-content-required = Content is required.
error-collab-content-too-large = The content is too large.
error-collab-grant-not-found = The grant was not found.
error-collab-invalid-action = The action is invalid.
error-collab-invalid-base-version = The base version is invalid.
error-collab-invalid-cursor = The 'before' cursor is invalid.
error-collab-invalid-emoji = The emoji is invalid.
error-collab-invalid-grantee-id = The grantee id is invalid.
error-collab-invalid-grantee-type = The grantee type must be 'user' or 'domain'.
error-collab-invalid-name = The name is invalid.
error-collab-invalid-new-owner = The new owner id is invalid.
error-collab-invalid-op-type = The operation type is invalid.
error-collab-invalid-parent = The parent id is invalid.
error-collab-invalid-resolution = The resolution must be 'accept-server' or 'accept-client'.
error-collab-invalid-resource = The resource type or resource id is invalid.
error-collab-invalid-role = The role is invalid.
error-collab-link-not-found = The share link was not found.
error-collab-not-a-conflict = The operation is not an unresolved conflict.
error-collab-operation-not-found = The operation was not found.
error-collab-owner-only = Only the owner can do this.
error-collab-version-not-found = The version was not found.
error-computer-use-action-id-mismatch = The action id does not match the path.
error-computer-use-action-not-found = Action not found.
error-computer-use-browser-only = This action must run in the browser panel.
error-computer-use-kind-invalid = The kind must be 'browser' or 'terminal'.
error-computer-use-no-consent = There is no consent record for this action; approve it first.
error-computer-use-session-required = A session id is required.
error-computer-use-url-missing = The action has no URL.
error-conversation-not-found = The conversation was not found.
error-conversation-session-invalid = The conversation session is invalid.
error-db-unavailable = The database is unavailable right now. Please try again.
error-drive-sync-conflict-path-escape = The conflict path escapes its sync folder.
error-drive-sync-conflict-path-required = A conflict path is required.
error-drive-sync-field-absolute = { $field } must be an absolute path.
error-drive-sync-field-drive-folder = The Drive folder
error-drive-sync-field-local-folder = The local folder
error-drive-sync-field-not-folder = { $field } is not a folder on this machine: { $path }
error-drive-sync-field-parent = { $field } must not contain '..'.
error-drive-sync-field-required = { $field } is required.
error-drive-sync-job-exists = A sync job named { $name } already exists.
error-drive-sync-job-name-required = A sync job name is required.
error-drive-sync-job-not-found = Sync job { $id } was not found.
error-drive-sync-task-failed = The { $task } task failed: { $error }
error-file-archive-unsafe = The archive was refused: { $reason }.
error-file-content-mismatch = The file content does not match its type ({ $expected } expected, { $detected } found).
error-file-empty = The uploaded file is empty.
error-file-extension-blocked = Files with the .{ $extension } extension are not allowed.
error-file-extension-unknown = The .{ $extension } extension is not recognised.
error-file-name-invalid = The file name is not valid.
error-goal-run-advance-refused = The run cannot advance in its current state.
error-goal-run-not-paused = This goal run is not paused.
error-goal-run-not-running = This goal run is not running.
error-group-budget-amount-invalid = The amount must be zero or positive.
error-group-budget-caps-invalid = The caps must be zero or positive (zero means no limit).
error-group-budget-invalid-id = The budget id is invalid.
error-group-budget-not-found = The group budget was not found.
error-http-no-route = No route matches this path.
error-invalid-value = The supplied value could not be parsed.
error-org-auth-service-unavailable = The directory service is not available.
error-org-default-delete = The default organization cannot be deleted.
error-org-name-required = The organization name is required.
error-org-settings-save-failed = Could not save the organization settings.
error-org-tenant-missing = No tenant exists yet — run the migrations or the bootstrap first.
error-peer-network-disabled = The peer network is disabled on this node.
error-peer-network-forbidden = An administrator session is required for this peer control.
error-peer-network-invalid = The peer request is not valid: { $message }
error-peer-network-payload-store-full = The peer payload store is full: { $message }
error-peer-network-recipient-required = The recipient query parameter is required.
error-peer-network-registry-full = The peer registry is full.
error-peer-network-unauthorized = A valid peer bearer token is required.
error-peer-network-unconfigured = This node has no peer token configured, so peer endpoints are closed.
error-peer-network-unknown-node = Unknown peer node: { $message }
error-people-invalid-end-date = The end date is invalid. Use the YYYY-MM-DD format.
error-people-invalid-start-date = The start date is invalid. Use the YYYY-MM-DD format.
error-person-not-found = Person not found.
error-product-not-found = Product not found.
error-service-invalid-value = The service value is invalid.
error-service-not-found = Service not found.
error-service-price-required = This service has no price; provide a value.
error-subagent-invalid-tasks = The sub-agent tasks are invalid.
error-subagent-no-llm = No LLM provider is configured for sub-agents.
error-team-not-found = Team not found.
error-terminal-action-id-required = Terminal execution requires an approved action id.
error-terminal-approval-required = Terminal execution requires explicit user approval.
error-terminal-blocked-pattern = The command matches a blocked destructive pattern.
error-terminal-command-required = A command is required.
error-terminal-command-too-long = The command is limited to 16000 characters.
error-terminal-timeout = The command timed out.
error-terminal-workspace-stopped = The agent workspace is not running. Finish provisioning (or restart the desktop app), then retry.
error-time-off-not-found = Time-off request not found.
error-value-required = A value is required for this service.
