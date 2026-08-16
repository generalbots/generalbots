# =============================================================================
# General Bots - Authentication Translations (English)
# =============================================================================
# Authentication, Passkey/WebAuthn, and security interface translations
# =============================================================================

# -----------------------------------------------------------------------------
# Authentication General
# -----------------------------------------------------------------------------
auth-title = 인증
auth-login = 로그인
auth-logout = 로그아웃
auth-signup = 가입
auth-welcome = 환영합니다
auth-welcome-back = 돌아온 것을 환영합니다, { $name }!
auth-session-expired = 세션이 만료되었습니다
auth-session-timeout = { $minutes }분 후 세션 시간 초과

# -----------------------------------------------------------------------------
# Login Form
# -----------------------------------------------------------------------------
auth-login-title = 귀하의 계정에 로그인하세요
auth-login-subtitle = 계속하려면 자격 증명을 입력하세요.
auth-login-email = 이메일 주소
auth-login-username = 사용자 이름
auth-login-password = 비밀번호
auth-login-remember = 나를 기억해
auth-login-forgot = 비밀번호를 잊으셨나요?
auth-login-submit = 로그인
auth-login-loading = 로그인 중...
auth-login-or = 아니면 계속해서
auth-login-no-account = 계정이 없나요?
auth-login-create-account = 계정 만들기

# -----------------------------------------------------------------------------
# Passkey/WebAuthn
# -----------------------------------------------------------------------------
passkey-title = 패스키
passkey-subtitle = 안전한 비밀번호 없는 인증
passkey-description = 패스키는 안전한 피싱 방지 로그인을 위해 장치의 생체 인식 또는 PIN을 사용합니다.
passkey-what-is = 암호키란 무엇입니까?
passkey-benefits = 암호 키의 이점
passkey-benefit-secure = 비밀번호보다 더 안전함
passkey-benefit-easy = 사용하기 쉬움 - 기억할 비밀번호가 없음
passkey-benefit-fast = 생체 인식을 통한 빠른 로그인
passkey-benefit-phishing = 피싱 공격에 강함

# -----------------------------------------------------------------------------
# Passkey Registration
# -----------------------------------------------------------------------------
passkey-register-title = 패스키 설정
passkey-register-subtitle = 더욱 빠르고 안전한 로그인을 위해 비밀번호 키를 만드세요.
passkey-register-description = 기기에서 지문, 얼굴 또는 화면 잠금을 사용하여 신원을 확인하도록 요청합니다.
passkey-register-button = 패스키 생성
passkey-register-name = 패스키 이름
passkey-register-name-placeholder = 예: 맥북 프로, 아이폰
passkey-register-name-hint = 나중에 식별할 수 있도록 패스키에 이름을 지정하세요.
passkey-register-loading = 비밀번호 설정 중...
passkey-register-verifying = 기기를 확인하는 중...
passkey-register-success = 비밀번호가 성공적으로 생성되었습니다
passkey-register-error = 비밀번호를 생성하지 못했습니다.
passkey-register-cancelled = 비밀번호 설정이 취소되었습니다.
passkey-register-not-supported = 귀하의 브라우저는 암호 키를 지원하지 않습니다

# -----------------------------------------------------------------------------
# Passkey Authentication
# -----------------------------------------------------------------------------
passkey-login-title = 암호키로 로그인
passkey-login-subtitle = 비밀번호 없는 안전한 로그인을 위해 비밀번호 키를 사용하세요.
passkey-login-button = 암호키로 로그인
passkey-login-loading = 인증 중...
passkey-login-verifying = 비밀번호 확인 중...
passkey-login-success = 성공적으로 로그인되었습니다
passkey-login-error = 인증 실패
passkey-login-cancelled = 인증이 취소되었습니다.
passkey-login-no-passkeys = 이 계정에 대한 암호 키를 찾을 수 없습니다.
passkey-login-try-another = 다른 방법을 시도해 보세요

# -----------------------------------------------------------------------------
# Passkey Management
# -----------------------------------------------------------------------------
passkey-manage-title = 패스키 관리
passkey-manage-subtitle = 등록된 암호키 보기 및 관리
passkey-manage-count = { $개수 ->
    [one] { $count } passkey registered
   *[other] { $count } passkeys registered
}
passkey-manage-add = 새 패스키 추가
passkey-manage-rename = 이름 바꾸기
passkey-manage-delete = 삭제
passkey-manage-created = 생성됨 { $date }
passkey-manage-last-used = 마지막 사용 날짜: { $date }
passkey-manage-never-used = 한번도 사용하지 않음
passkey-manage-this-device = 이 장치
passkey-manage-cross-platform = 크로스 플랫폼
passkey-manage-platform = 플랫폼 인증자
passkey-manage-security-key = 보안 키
passkey-manage-empty = 등록된 비밀번호가 없습니다.
passkey-manage-empty-description = 더 빠르고 안전한 로그인을 위해 암호 키를 추가하세요.

# -----------------------------------------------------------------------------
# Passkey Deletion
# -----------------------------------------------------------------------------
passkey-delete-title = 패스키 삭제
passkey-delete-confirm = 이 패스키를 삭제하시겠습니까?
passkey-delete-warning = 더 이상 이 비밀번호를 사용하여 로그인할 수 없습니다.
passkey-delete-last-warning = 이것이 유일한 패스키입니다. 삭제 후 비밀번호 인증을 이용하셔야 합니다.
passkey-delete-success = 비밀번호가 삭제되었습니다.
passkey-delete-error = 비밀번호를 삭제하지 못했습니다.

# -----------------------------------------------------------------------------
# Password Fallback
# -----------------------------------------------------------------------------
passkey-fallback-title = 대신 비밀번호를 사용하세요
passkey-fallback-description = 비밀번호 키를 사용할 수 없는 경우 비밀번호로 로그인할 수 있습니다.
passkey-fallback-button = 비밀번호 사용
passkey-fallback-or-passkey = 또는 비밀번호 키로 로그인
passkey-fallback-setup-prompt = 다음 번에 더 빠르게 로그인하려면 암호 키를 설정하세요.
passkey-fallback-setup-later = 어쩌면 나중에
passkey-fallback-setup-now = 지금 설정하세요
passkey-fallback-locked = 계정이 일시적으로 잠겼습니다.
passkey-fallback-locked-description = 실패한 시도가 너무 많습니다. { $minutes }분 후에 다시 시도하세요.
passkey-fallback-attempts = { $remaining } 시도 남음

# -----------------------------------------------------------------------------
# Multi-Factor Authentication
# -----------------------------------------------------------------------------
mfa-title = 2단계 인증
mfa-subtitle = 계정에 추가 보안 계층을 추가하세요
mfa-enabled = 이중 인증이 활성화되었습니다.
mfa-disabled = 이중 인증이 비활성화되었습니다.
mfa-enable = 2FA 활성화
mfa-disable = 2FA 비활성화
mfa-setup = 2FA 설정
mfa-verify = 코드 확인
mfa-code = 인증코드
mfa-code-placeholder = 6자리 코드를 입력하세요
mfa-code-sent = { $destination }로 코드가 전송되었습니다.
mfa-code-expired = 코드가 만료되었습니다
mfa-code-invalid = 잘못된 코드
mfa-resend = 코드 재전송
mfa-resend-in = { $seconds }초 후에 다시 보내기
mfa-methods = 인증 방법
mfa-method-app = 인증자 앱
mfa-method-sms = SMS
mfa-method-email = 이메일
mfa-method-passkey = 패스키
mfa-backup-codes = 백업 코드
mfa-backup-codes-description = 이 코드를 안전한 장소에 저장하세요. 각 코드는 한 번만 사용할 수 있습니다.
mfa-backup-codes-remaining = { $count } 남은 백업 코드
mfa-backup-codes-generate = 새 코드 생성
mfa-backup-codes-download = 코드 다운로드
mfa-backup-codes-copy = 코드 복사

# -----------------------------------------------------------------------------
# Password Management
# -----------------------------------------------------------------------------
password-title = 비밀번호
password-change = 비밀번호 변경
password-current = 현재 비밀번호
password-new = 새 비밀번호
password-confirm = 새 비밀번호 확인
password-requirements = 비밀번호 요구 사항
password-requirement-length = { $length }자 이상
password-requirement-uppercase = 하나 이상의 대문자
password-requirement-lowercase = 하나 이상의 소문자
password-requirement-number = 숫자 1개 이상
password-requirement-special = 하나 이상의 특수 문자
password-strength = 비밀번호 강도
password-strength-weak = 약함
password-strength-fair = 박람회
password-strength-good = 좋음
password-strength-strong = 강한
password-match = 비밀번호가 일치합니다.
password-mismatch = 비밀번호가 일치하지 않습니다.
password-changed = 비밀번호가 성공적으로 변경되었습니다.
password-change-error = 비밀번호를 변경하지 못했습니다.

# -----------------------------------------------------------------------------
# Password Reset
# -----------------------------------------------------------------------------
password-reset-title = 비밀번호 재설정
password-reset-subtitle = 재설정 링크를 받으려면 이메일을 입력하세요.
password-reset-email-sent = 비밀번호 재설정 이메일이 전송되었습니다
password-reset-email-sent-description = 이메일에서 비밀번호 재설정 지침을 확인하세요.
password-reset-invalid-token = 유효하지 않거나 만료된 재설정 링크
password-reset-success = 비밀번호가 재설정되었습니다.
password-reset-error = 비밀번호를 재설정하지 못했습니다.

# -----------------------------------------------------------------------------
# Session Management
# -----------------------------------------------------------------------------
session-title = 활성 세션
session-subtitle = 여러 기기에서 활성 세션을 관리하세요
session-current = 현재 세션
session-device = 장치
session-location = 위치
session-last-active = 마지막 활성
session-ip-address = IP 주소
session-browser = 브라우저
session-os = 운영 체제
session-sign-out = 로그아웃
session-sign-out-all = 다른 모든 세션에서 로그아웃
session-sign-out-confirm = 이 세션에서 로그아웃하시겠습니까?
session-sign-out-all-confirm = 다른 모든 세션에서 로그아웃하시겠습니까?

# -----------------------------------------------------------------------------
# Security Settings
# -----------------------------------------------------------------------------
security-title = 보안
security-subtitle = 계정 보안 설정 관리
security-overview = 보안 개요
security-last-login = 마지막 로그인
security-password-last-changed = 마지막으로 변경된 비밀번호
security-security-checkup = 보안 점검
security-checkup-description = 보안 설정 검토
security-recommendation = 추천
security-add-passkey = 더욱 안전한 로그인을 위해 암호 키를 추가하세요
security-enable-mfa = 이중 인증 활성화
security-update-password = 정기적으로 비밀번호를 업데이트하세요.

# -----------------------------------------------------------------------------
# Error Messages
# -----------------------------------------------------------------------------
auth-error-invalid-credentials = 잘못된 이메일 또는 비밀번호
auth-error-account-locked = 계정이 잠겨 있습니다. 지원팀에 문의하세요.
auth-error-account-disabled = 계정이 비활성화되었습니다
auth-error-email-not-verified = 이메일 주소를 확인해 주세요
auth-error-too-many-attempts = 실패한 시도가 너무 많습니다. 나중에 다시 시도해 주세요.
auth-error-network = 네트워크 오류입니다. 연결을 확인해주세요.
auth-error-server = 서버 오류입니다. 나중에 다시 시도해 주세요.
auth-error-unknown = 알 수 없는 오류가 발생했습니다.
auth-error-session-invalid = 세션이 잘못되었습니다. 다시 로그인해 주세요.
auth-error-token-expired = 세션이 만료되었습니다. 다시 로그인해 주세요.
auth-error-unauthorized = 이 작업을 수행할 권한이 없습니다.

# -----------------------------------------------------------------------------
# Success Messages
# -----------------------------------------------------------------------------
auth-success-login = 성공적으로 로그인되었습니다
auth-success-logout = 성공적으로 로그아웃되었습니다
auth-success-signup = 계정이 성공적으로 생성되었습니다
auth-success-password-changed = 비밀번호가 성공적으로 변경되었습니다.
auth-success-email-verified = 이메일이 확인되었습니다.
auth-success-mfa-enabled = 이중 인증 활성화됨
auth-success-mfa-disabled = 이중 인증이 비활성화되었습니다.
auth-success-session-terminated = 세션이 성공적으로 종료되었습니다.

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
auth-notify-new-login = { $device }에서 { $location }로 새로 로그인
auth-notify-password-changed = 비밀번호가 변경되었습니다
auth-notify-mfa-enabled = 이중 인증이 활성화되었습니다.
auth-notify-passkey-added = 새 비밀번호가 계정에 추가되었습니다
auth-notify-suspicious-activity = 귀하의 계정에서 의심스러운 활동이 감지되었습니다
