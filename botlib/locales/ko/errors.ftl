# General Bots - Error Messages (English)
# This file contains all error message translations

# =============================================================================
# HTTP Errors
# =============================================================================

error-http-400 = 요청이 잘못되었습니다. 입력 내용을 확인하세요.
error-http-401 = 인증이 필요합니다. 로그인해주세요.
error-http-403 = 이 리소스에 액세스할 수 있는 권한이 없습니다.
error-http-404 = { $entity } 찾을 수 없습니다.
error-http-409 = 갈등: { $message }
error-http-429 = 요청이 너무 많습니다. { $seconds }초만 기다려주세요.
error-http-500 = 내부 서버 오류입니다. 나중에 다시 시도해 주세요.
error-http-502 = 나쁜 게이트웨이. 서버가 잘못된 응답을 받았습니다.
error-http-503 = 일시적으로 서비스를 이용할 수 없습니다. 나중에 다시 시도해 주세요.
error-http-504 = { $milliseconds }ms 후에 요청 시간이 초과되었습니다.

# =============================================================================
# Validation Errors
# =============================================================================

error-validation-required = { $field }가 필요합니다.
error-validation-email = 유효한 이메일 주소를 입력하세요.
error-validation-url = 유효한 URL을 입력하세요.
error-validation-phone = 유효한 전화번호를 입력하세요.
error-validation-min-length = { $field }는 { $min }자 이상이어야 합니다.
error-validation-max-length = { $field }는 { $max }자 이하여야 합니다.
error-validation-min-value = { $field }는 { $min } 이상이어야 합니다.
error-validation-max-value = { $field }는 { $max } 이하여야 합니다.
error-validation-pattern = { $field } 형식이 잘못되었습니다.
error-validation-unique = { $field }가 이미 존재합니다.
error-validation-mismatch = { $field }가 { $other }와 일치하지 않습니다.
error-validation-date-format = { $format } 형식으로 유효한 날짜를 입력하세요.
error-validation-date-past = { $field }은 과거여야 합니다.
error-validation-date-future = { $field }는 미래에 있어야 합니다.

# =============================================================================
# Authentication Errors
# =============================================================================

error-auth-invalid-credentials = 이메일이나 비밀번호가 잘못되었습니다.
error-auth-account-locked = 귀하의 계정이 잠겼습니다. 지원팀에 문의하세요.
error-auth-account-disabled = 귀하의 계정이 비활성화되었습니다.
error-auth-session-expired = 세션이 만료되었습니다. 다시 로그인해주세요.
error-auth-token-invalid = 토큰이 잘못되었거나 만료되었습니다.
error-auth-token-missing = 인증 토큰이 필요합니다.
error-auth-mfa-required = 다단계 인증이 필요합니다.
error-auth-mfa-invalid = 인증 코드가 잘못되었습니다.
error-auth-password-weak = 비밀번호가 너무 취약합니다. 더 강력한 비밀번호를 사용하세요.
error-auth-password-expired = 귀하의 비밀번호가 만료되었습니다. 재설정해 주세요.

# =============================================================================
# Configuration Errors
# =============================================================================

error-config = 구성 오류: { $message }
error-config-missing = 누락된 구성: { $key }
error-config-invalid = { $key }에 대한 잘못된 구성 값: { $reason }
error-config-file-not-found = 구성 파일을 찾을 수 없습니다: { $path }
error-config-parse = 구성 분석 실패: { $message }

# =============================================================================
# Database Errors
# =============================================================================

error-database = 데이터베이스 오류: { $message }
error-database-connection = 데이터베이스에 연결하지 못했습니다.
error-database-timeout = 데이터베이스 작업 시간이 초과되었습니다.
error-database-constraint = 데이터베이스 제약 조건 위반: { $constraint }
error-database-duplicate = { $field }라는 기록이 이미 존재합니다.
error-database-migration = 데이터베이스 마이그레이션 실패: { $message }

# =============================================================================
# File & Storage Errors
# =============================================================================

error-file-not-found = 파일을 찾을 수 없음: { $filename }
error-file-too-large = 파일이 너무 큽니다. 최대 크기는 { $maxSize }입니다.
error-file-type-not-allowed = 파일 형식이 허용되지 않습니다. 허용되는 유형: { $allowedTypes }.
error-file-upload-failed = 파일 업로드 실패: { $message }
error-file-read = 파일을 읽지 못했습니다: { $message }
error-file-write = 파일 쓰기 실패: { $message }
error-storage-full = 저장소 할당량을 초과했습니다.
error-storage-unavailable = 보관 서비스를 이용할 수 없습니다.

# =============================================================================
# Network & External Service Errors
# =============================================================================

error-network = 네트워크 오류: { $message }
error-network-timeout = 연결 시간이 초과되었습니다.
error-network-unreachable = 서버에 연결할 수 없습니다.
error-service-unavailable = 서비스 이용 불가: { $service }
error-external-api = 외부 API 오류: { $message }
error-rate-limit = 요금이 제한되어 있습니다. { $seconds }초 후에 다시 시도하세요.

# =============================================================================
# Bot & Dialog Errors
# =============================================================================

error-bot-not-found = 봇을 찾을 수 없음: { $botId }
error-bot-disabled = 이 봇은 현재 비활성화되어 있습니다.
error-bot-script-error = { $line }행에 스크립트 오류가 있습니다: { $message }
error-bot-timeout = 봇 응답 시간이 초과되었습니다.
error-bot-quota-exceeded = 봇 사용 할당량을 초과했습니다.
error-dialog-not-found = 대화상자를 찾을 수 없습니다: { $dialogId }
error-dialog-invalid = 잘못된 대화 상자 구성: { $message }

# =============================================================================
# LLM & AI Errors
# =============================================================================

error-llm-unavailable = 현재 AI 서비스를 이용할 수 없습니다.
error-llm-timeout = AI 요청 시간이 초과되었습니다.
error-llm-rate-limit = AI 비율 제한을 초과했습니다. 다시 시도하기 전에 잠시 기다려 주세요.
error-llm-content-filter = 콘텐츠는 안전 지침에 따라 필터링되었습니다.
error-llm-context-length = 입력이 너무 깁니다. 메시지를 줄여주세요.
error-llm-invalid-response = AI 서비스로부터 잘못된 응답을 받았습니다.
error-llm-empty-response = 죄송합니다. 지금은 메시지를 처리할 수 없습니다. 몇 초 후에 다시 시도해 주세요.

# =============================================================================
# Email Errors
# =============================================================================

error-email-send-failed = 이메일 전송 실패: { $message }
error-email-invalid-recipient = 잘못된 수신자 이메일 주소: { $email }
error-email-attachment-failed = 파일 첨부 실패: { $filename }
error-email-template-not-found = 이메일 템플릿을 찾을 수 없습니다: { $template }

# =============================================================================
# Calendar & Scheduling Errors
# =============================================================================

error-calendar-conflict = 시간 슬롯이 기존 이벤트와 충돌합니다.
error-calendar-past-date = 과거에는 이벤트를 예약할 수 없습니다.
error-calendar-invalid-recurrence = 반복 패턴이 잘못되었습니다.
error-calendar-event-not-found = 이벤트를 찾을 수 없음: { $eventId }

# =============================================================================
# Task Errors
# =============================================================================

error-task-not-found = 작업을 찾을 수 없음: { $taskId }
error-task-already-completed = 작업이 이미 완료되었습니다.
error-task-circular-dependency = 작업에서 순환 종속성이 감지되었습니다.
error-task-invalid-status = 작업 상태 전환이 잘못되었습니다.

# =============================================================================
# Permission Errors
# =============================================================================

error-permission-denied = 이 작업을 수행할 권한이 없습니다.
error-permission-resource = { $resource }에 액세스할 수 없습니다.
error-permission-action = { $action } 이건 { $resource } 할 수 없습니다.
error-permission-owner-only = 소유자만 이 작업을 수행할 수 있습니다.

# =============================================================================
# Generic Errors
# =============================================================================

error-internal = 내부 오류: { $message }
error-unexpected = 예상치 못한 오류가 발생했습니다. 다시 시도해 주세요.
error-not-implemented = 이 기능은 아직 구현되지 않았습니다.
error-maintenance = 시스템 유지보수 중입니다. 나중에 다시 시도해 주세요.
error-unknown = 알 수 없는 오류가 발생했습니다.

error-locale-invalid = 요청한 언어가 올바르지 않습니다.
error-locale-unsupported = 이 언어는 사용할 수 없습니다.
error-locale-persist-failed = 언어 설정을 저장하지 못했습니다.
error-bot-id-required = 봇 ID가 필요합니다.
error-bot-id-invalid = 봇 ID가 올바르지 않습니다.
error-bot-name-required = 봇 이름이 필요합니다.
error-token-invalid-format = 토큰 형식이 올바르지 않습니다.
error-token-signing-failed = 토큰에 서명하지 못했습니다.
error-token-signature-invalid = 토큰 서명이 올바르지 않습니다.
error-token-payload-invalid = 토큰 페이로드가 올바르지 않습니다.
error-token-payload-json-invalid = 토큰 페이로드가 올바른 JSON이 아닙니다.
error-cloud-token-required = cloud_token을 제공하세요.
error-llm-subscription-token-missing = 구독 토큰이 없습니다.
error-llm-token-store-unavailable = 토큰 저장소를 사용할 수 없습니다.
error-llm-subscription-token-expired = 구독 토큰이 만료되었습니다. 메시지를 다시 보내세요.
error-llm-prompt-missing = 요청에 프롬프트가 없습니다.
error-llm-account-unlinked = 먼저 설정 → AI 계정에서 Claude 또는 ChatGPT 계정을 연결하세요.
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
