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
