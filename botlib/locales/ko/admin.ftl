# =============================================================================
# General Bots - Admin Translations (English)
# =============================================================================
# Administrative interface translations for the GB Admin Panel
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Navigation & Dashboard
# -----------------------------------------------------------------------------
admin-title = 행정
admin-dashboard = 관리 대시보드
admin-overview = 개요
admin-welcome = 관리자 패널에 오신 것을 환영합니다

admin-nav-dashboard = 대시보드
admin-nav-users = 사용자
admin-nav-bots = 봇
admin-nav-tenants = 임차인
admin-nav-settings = 설정
admin-nav-logs = 로그
admin-nav-analytics = 분석
admin-nav-security = 보안
admin-nav-integrations = 통합
admin-nav-billing = 청구
admin-nav-support = 지원
admin-nav-groups = 그룹
admin-nav-dns = DNS
admin-nav-system = 시스템

# -----------------------------------------------------------------------------
# Admin Quick Actions
# -----------------------------------------------------------------------------
admin-quick-actions = 빠른 작업
admin-create-user = 사용자 생성
admin-create-group = 그룹 만들기
admin-register-dns = DNS 등록
admin-recent-activity = 최근 활동
admin-system-health = 시스템 상태

# -----------------------------------------------------------------------------
# User Management
# -----------------------------------------------------------------------------
admin-users-title = 사용자 관리
admin-users-list = 사용자 목록
admin-users-add = 사용자 추가
admin-users-edit = 사용자 편집
admin-users-delete = 사용자 삭제
admin-users-search = 사용자 검색...
admin-users-filter = 사용자 필터링
admin-users-export = 사용자 내보내기
admin-users-import = 사용자 가져오기
admin-users-total = 총 사용자
admin-users-active = 활성 사용자
admin-users-inactive = 비활성 사용자
admin-users-suspended = 정지된 사용자
admin-users-pending = 확인 대기 중
admin-users-last-login = 마지막 로그인
admin-users-created = 생성됨
admin-users-role = 역할
admin-users-status = 상태
admin-users-actions = 작업
admin-users-no-users = 사용자를 찾을 수 없습니다.
admin-users-confirm-delete = 이 사용자를 삭제하시겠습니까?
admin-users-deleted = 사용자가 삭제되었습니다.
admin-users-saved = 사용자가 성공적으로 저장되었습니다.
admin-users-invite = 사용자 초대
admin-users-invite-sent = 초대장이 성공적으로 전송되었습니다.
admin-users-bulk-actions = 대량 작업
admin-users-select-all = 모두 선택
admin-users-deselect-all = 모두 선택 해제

# User Details
admin-user-details = 사용자 세부정보
admin-user-profile = 프로필
admin-user-email = 이메일
admin-user-name = 이름
admin-user-phone = 전화
admin-user-avatar = 아바타
admin-user-timezone = 시간대
admin-user-language = 언어
admin-user-role-admin = 관리자
admin-user-role-manager = 관리자
admin-user-role-user = 사용자
admin-user-role-viewer = 뷰어
admin-user-status-active = 활성
admin-user-status-inactive = 비활성
admin-user-status-suspended = 정지됨
admin-user-status-pending = 보류 중
admin-user-permissions = 권한
admin-user-activity = 활동 로그
admin-user-sessions = 활성 세션
admin-user-terminate-session = 세션 종료
admin-user-terminate-all = 모든 세션 종료
admin-user-reset-password = 비밀번호 재설정
admin-user-force-logout = 강제 로그아웃
admin-user-enable-2fa = 2FA 활성화
admin-user-disable-2fa = 2FA 비활성화

# -----------------------------------------------------------------------------
# Group Management
# -----------------------------------------------------------------------------
admin-groups-title = 그룹 관리
admin-groups-subtitle = 그룹, 구성원, 권한 관리
admin-groups-list = 그룹 목록
admin-groups-add = 그룹 추가
admin-groups-create = 그룹 만들기
admin-groups-edit = 그룹 편집
admin-groups-delete = 그룹 삭제
admin-groups-search = 그룹 검색...
admin-groups-filter = 필터 그룹
admin-groups-total = 총 그룹
admin-groups-active = 활성 그룹
admin-groups-no-groups = 그룹을 찾을 수 없습니다.
admin-groups-confirm-delete = 이 그룹을 삭제하시겠습니까?
admin-groups-deleted = 그룹이 삭제되었습니다.
admin-groups-saved = 그룹이 저장되었습니다.
admin-groups-created = 그룹이 성공적으로 생성되었습니다.
admin-groups-loading = 그룹 로드 중...

# Group Details
admin-group-details = 그룹 세부정보
admin-group-name = 그룹 이름
admin-group-description = 설명
admin-group-visibility = 가시성
admin-group-visibility-public = 공개
admin-group-visibility-private = 비공개
admin-group-visibility-hidden = 숨겨진
admin-group-join-policy = 가입 정책
admin-group-join-invite = 초대만 가능
admin-group-join-request = 가입 요청
admin-group-join-open = 열기
admin-group-members = 회원
admin-group-member-count = { $개수 ->
    [one] { $count } member
   *[other] { $count } members
}
admin-group-add-member = 회원 추가
admin-group-remove-member = 회원 삭제
admin-group-permissions = 권한
admin-group-settings = 설정
admin-group-analytics = 분석
admin-group-overview = 개요

# Group View Modes
admin-groups-view-grid = 그리드 보기
admin-groups-view-list = 목록 보기
admin-groups-all-visibility = 모든 가시성

# -----------------------------------------------------------------------------
# DNS Management
# -----------------------------------------------------------------------------
admin-dns-title = DNS 관리
admin-dns-subtitle = 봇의 DNS 호스트 이름 등록 및 관리
admin-dns-register = 호스트 이름 등록
admin-dns-registered = 등록된 호스트 이름
admin-dns-search = 호스트 이름 검색...
admin-dns-refresh = 새로고침
admin-dns-loading = DNS 레코드 로드 중...
admin-dns-no-records = DNS 레코드를 찾을 수 없습니다.
admin-dns-confirm-delete = 이 호스트 이름을 삭제하시겠습니까?
admin-dns-deleted = 호스트 이름이 성공적으로 제거되었습니다.
admin-dns-saved = DNS 레코드가 성공적으로 저장되었습니다.
admin-dns-created = 호스트 이름이 성공적으로 등록되었습니다.

# DNS Form Fields
admin-dns-hostname = 호스트 이름
admin-dns-hostname-placeholder = mybot.example.com
admin-dns-hostname-help = 등록하려는 전체 도메인 이름을 입력하세요.
admin-dns-record-type = 레코드 유형
admin-dns-record-type-a = A(IPv4)
admin-dns-record-type-aaaa = AAAA(IPv6)
admin-dns-record-type-cname = CNAME
admin-dns-ttl = TTL(초)
admin-dns-ttl-5min = 5분(300)
admin-dns-ttl-1hour = 1시간 (3600)
admin-dns-ttl-1day = 1일(86400)
admin-dns-target = 대상/IP 주소
admin-dns-target-placeholder-ipv4 = 192.168.1.1
admin-dns-target-placeholder-ipv6 = 2001:db8::1
admin-dns-target-placeholder-cname = target.example.com
admin-dns-target-help-a = 가리킬 IPv4 주소를 입력하세요.
admin-dns-target-help-aaaa = 가리킬 IPv6 주소를 입력하세요.
admin-dns-target-help-cname = 대상 도메인 이름을 입력하세요.
admin-dns-auto-ssl = SSL 인증서 자동 프로비저닝

# DNS Table Headers
admin-dns-col-hostname = 호스트 이름
admin-dns-col-type = 유형
admin-dns-col-target = 대상
admin-dns-col-ttl = TTL
admin-dns-col-ssl = SSL
admin-dns-col-status = 상태
admin-dns-col-actions = 작업

# DNS Status
admin-dns-status-active = 활성
admin-dns-status-pending = 보류 중
admin-dns-status-error = 오류
admin-dns-ssl-enabled = SSL 활성화
admin-dns-ssl-disabled = SSL 없음
admin-dns-ssl-pending = SSL 보류 중

# DNS Info Cards
admin-dns-help-title = DNS 구성 도움말
admin-dns-help-a-record = 기록
admin-dns-help-a-record-desc = 도메인 이름을 IPv4 주소에 매핑합니다. 이를 사용하여 호스트 이름을 서버 IP로 직접 가리킵니다.
admin-dns-help-aaaa-record = AAAA 기록
admin-dns-help-aaaa-record-desc = 도메인 이름을 IPv6 주소에 매핑합니다. A 레코드와 유사하지만 IPv6 연결용입니다.
admin-dns-help-cname-record = CNAME 레코드
admin-dns-help-cname-record-desc = 한 도메인에서 다른 도메인으로 별칭을 만듭니다. 하위 도메인을 기본 도메인으로 가리키는 데 유용합니다.
admin-dns-help-ssl = SSL/TLS
admin-dns-help-ssl-desc = 보안 HTTPS 연결을 위해 Let's Encrypt 인증서를 자동으로 프로비저닝합니다.

# DNS Edit/Remove Modals
admin-dns-edit-title = DNS 레코드 편집
admin-dns-remove-title = 호스트 이름 제거
admin-dns-remove-warning = 이렇게 하면 DNS 레코드 및 관련 SSL 인증서가 삭제됩니다. 호스트 이름이 더 이상 확인되지 않습니다.

# -----------------------------------------------------------------------------
# Bot Management
# -----------------------------------------------------------------------------
admin-bots-title = 봇 관리
admin-bots-list = 봇 목록
admin-bots-add = 봇 추가
admin-bots-edit = 봇 편집
admin-bots-delete = 봇 삭제
admin-bots-search = 봇 검색...
admin-bots-filter = 필터 봇
admin-bots-total = 총 봇
admin-bots-active = 활성 봇
admin-bots-inactive = 비활성 봇
admin-bots-draft = 초안 봇
admin-bots-published = 게시된 봇
admin-bots-no-bots = 봇을 찾을 수 없습니다.
admin-bots-confirm-delete = 이 봇을 삭제하시겠습니까?
admin-bots-deleted = 봇이 삭제되었습니다.
admin-bots-saved = 봇이 성공적으로 저장되었습니다.
admin-bots-duplicate = 중복 봇
admin-bots-export = 봇 내보내기
admin-bots-import = 봇 가져오기
admin-bots-publish = 게시
admin-bots-unpublish = 게시 취소
admin-bots-test = 테스트봇
admin-bots-logs = 봇 로그
admin-bots-analytics = 봇 분석
admin-bots-conversations = 대화
admin-bots-templates = 템플릿
admin-bots-dialogs = 대화상자
admin-bots-knowledge-base = 기술 자료

# Bot Details
admin-bot-details = 봇 세부정보
admin-bot-name = 봇 이름
admin-bot-description = 설명
admin-bot-avatar = 봇 아바타
admin-bot-language = 언어
admin-bot-timezone = 시간대
admin-bot-greeting = 인사말 메시지
admin-bot-fallback = 대체 메시지
admin-bot-channels = 채널
admin-bot-channel-web = 웹 채팅
admin-bot-channel-whatsapp = 왓츠앱
admin-bot-channel-telegram = 텔레그램
admin-bot-channel-slack = 슬랙
admin-bot-channel-teams = 마이크로소프트 팀즈
admin-bot-channel-email = 이메일
admin-bot-model = AI 모델
admin-bot-temperature = 온도
admin-bot-max-tokens = 최대 토큰
admin-bot-system-prompt = 시스템 프롬프트

# -----------------------------------------------------------------------------
# Tenant Management
# -----------------------------------------------------------------------------
admin-tenants-title = 임차인 관리
admin-tenants-list = 임차인 목록
admin-tenants-add = 테넌트 추가
admin-tenants-edit = 테넌트 편집
admin-tenants-delete = 테넌트 삭제
admin-tenants-search = 임차인 검색...
admin-tenants-total = 총 임차인
admin-tenants-active = 활성 테넌트
admin-tenants-suspended = 정지된 임차인
admin-tenants-trial = 평가판 테넌트
admin-tenants-no-tenants = 테넌트를 찾을 수 없습니다.
admin-tenants-confirm-delete = 이 테넌트를 삭제하시겠습니까?
admin-tenants-deleted = 테넌트가 삭제되었습니다.
admin-tenants-saved = 테넌트가 성공적으로 저장되었습니다.

# Tenant Details
admin-tenant-details = 임차인 세부정보
admin-tenant-name = 테넌트 이름
admin-tenant-domain = 도메인
admin-tenant-plan = 계획
admin-tenant-plan-free = 무료
admin-tenant-plan-starter = 스타터
admin-tenant-plan-professional = 전문가
admin-tenant-plan-enterprise = 기업
admin-tenant-users = 사용자
admin-tenant-bots = 봇
admin-tenant-storage = 사용된 저장소
admin-tenant-api-calls = API 호출
admin-tenant-limits = 사용량 한도
admin-tenant-billing = 청구 정보

# -----------------------------------------------------------------------------
# System Settings
# -----------------------------------------------------------------------------
admin-settings-title = 시스템 설정
admin-settings-general = 일반 설정
admin-settings-security = 보안 설정
admin-settings-email = 이메일 설정
admin-settings-storage = 저장소 설정
admin-settings-integrations = 통합
admin-settings-api = API 설정
admin-settings-appearance = 외관
admin-settings-localization = 현지화
admin-settings-notifications = 알림
admin-settings-backup = 백업 및 복원
admin-settings-maintenance = 유지 관리 모드
admin-settings-saved = 설정이 성공적으로 저장되었습니다.
admin-settings-reset = 기본값으로 재설정
admin-settings-confirm-reset = 모든 설정을 기본값으로 재설정하시겠습니까?

# General Settings
admin-settings-site-name = 사이트 이름
admin-settings-site-url = 사이트 URL
admin-settings-admin-email = 관리자 이메일
admin-settings-support-email = 지원 이메일
admin-settings-default-language = 기본 언어
admin-settings-default-timezone = 기본 시간대
admin-settings-date-format = 날짜 형식
admin-settings-time-format = 시간 형식
admin-settings-currency = 통화

# Email Settings
admin-settings-smtp-host = SMTP 호스트
admin-settings-smtp-port = SMTP 포트
admin-settings-smtp-user = SMTP 사용자 이름
admin-settings-smtp-password = SMTP 비밀번호
admin-settings-smtp-encryption = 암호화
admin-settings-smtp-from-name = 이름에서
admin-settings-smtp-from-email = 이메일에서
admin-settings-smtp-test = 테스트 이메일 보내기
admin-settings-smtp-test-success = 테스트 이메일이 성공적으로 전송되었습니다.
admin-settings-smtp-test-failed = 테스트 이메일을 보내지 못했습니다.

# Storage Settings
admin-settings-storage-provider = 스토리지 제공자
admin-settings-storage-local = 로컬 저장소
admin-settings-storage-s3 = 아마존 S3
admin-settings-storage-minio = 미니IO
admin-settings-storage-gcs = 구글 클라우드 스토리지
admin-settings-storage-azure = Azure Blob 저장소
admin-settings-storage-bucket = 버킷 이름
admin-settings-storage-region = 지역
admin-settings-storage-access-key = 액세스 키
admin-settings-storage-secret-key = 비밀키
admin-settings-storage-endpoint = 엔드포인트 URL

# -----------------------------------------------------------------------------
# System Logs
# -----------------------------------------------------------------------------
admin-logs-title = 시스템 로그
admin-logs-search = 로그 검색...
admin-logs-filter-level = 레벨별로 필터링
admin-logs-filter-source = 소스로 필터링
admin-logs-filter-date = 날짜별로 필터링
admin-logs-level-all = 모든 레벨
admin-logs-level-debug = 디버그
admin-logs-level-info = 정보
admin-logs-level-warning = 경고
admin-logs-level-error = 오류
admin-logs-level-critical = 심각
admin-logs-export = 로그 내보내기
admin-logs-clear = 로그 지우기
admin-logs-confirm-clear = 모든 로그를 지우시겠습니까?
admin-logs-cleared = 로그가 성공적으로 지워졌습니다.
admin-logs-no-logs = 로그를 찾을 수 없습니다.
admin-logs-refresh = 새로고침
admin-logs-auto-refresh = 자동 새로고침
admin-logs-timestamp = 타임스탬프
admin-logs-level = 레벨
admin-logs-source = 소스
admin-logs-message = 메시지
admin-logs-details = 세부정보

# -----------------------------------------------------------------------------
# Analytics
# -----------------------------------------------------------------------------
admin-analytics-title = 분석
admin-analytics-overview = 개요
admin-analytics-users = 사용자 분석
admin-analytics-bots = 봇 분석
admin-analytics-conversations = 대화 분석
admin-analytics-performance = 성능
admin-analytics-period = 기간
admin-analytics-period-today = 오늘
admin-analytics-period-week = 이번 주
admin-analytics-period-month = 이번 달
admin-analytics-period-quarter = 이번 분기
admin-analytics-period-year = 올해
admin-analytics-period-custom = 맞춤 범위
admin-analytics-export = 보고서 내보내기
admin-analytics-total-users = 총 사용자
admin-analytics-new-users = 신규 사용자
admin-analytics-active-users = 활성 사용자
admin-analytics-total-bots = 총 봇
admin-analytics-active-bots = 활성 봇
admin-analytics-total-conversations = 총 대화
admin-analytics-avg-response-time = 평균 응답 시간
admin-analytics-satisfaction-rate = 만족도
admin-analytics-resolution-rate = 해결률

# -----------------------------------------------------------------------------
# Security
# -----------------------------------------------------------------------------
admin-security-title = 보안
admin-security-overview = 보안 개요
admin-security-audit-log = 감사 로그
admin-security-login-attempts = 로그인 시도
admin-security-blocked-ips = 차단된 IP
admin-security-api-keys = API 키
admin-security-webhooks = 웹훅
admin-security-cors = CORS 설정
admin-security-rate-limiting = 속도 제한
admin-security-encryption = 암호화
admin-security-2fa = 2단계 인증
admin-security-sso = 싱글 사인온(SSO)
admin-security-password-policy = 비밀번호 정책

# API Keys
admin-api-keys-title = API 키
admin-api-keys-add = API 키 생성
admin-api-keys-name = 키 이름
admin-api-keys-key = API 키
admin-api-keys-secret = 비밀키
admin-api-keys-created = 생성됨
admin-api-keys-last-used = 마지막으로 사용됨
admin-api-keys-expires = 만료
admin-api-keys-never = 절대로
admin-api-keys-revoke = 취소
admin-api-keys-confirm-revoke = 이 API 키를 취소하시겠습니까?
admin-api-keys-revoked = API 키가 취소되었습니다.
admin-api-keys-created-success = API 키가 생성되었습니다.
admin-api-keys-copy = 클립보드에 복사
admin-api-keys-copied = 복사되었습니다!
admin-api-keys-warning = 지금 API 키를 복사하세요. 다시는 볼 수 없습니다!

# -----------------------------------------------------------------------------
# Billing
# -----------------------------------------------------------------------------
admin-billing-title = 청구
admin-billing-overview = 결제 개요
admin-billing-current-plan = 현재 계획
admin-billing-usage = 사용법
admin-billing-invoices = 송장
admin-billing-payment-methods = 결제 방법
admin-billing-upgrade = 업그레이드 계획
admin-billing-downgrade = 다운그레이드 계획
admin-billing-cancel = 구독 취소
admin-billing-invoice-date = 송장 날짜
admin-billing-invoice-amount = 금액
admin-billing-invoice-status = 상태
admin-billing-invoice-paid = 유료
admin-billing-invoice-pending = 보류 중
admin-billing-invoice-overdue = 연체
admin-billing-invoice-download = 송장 다운로드

# -----------------------------------------------------------------------------
# Backup & Restore
# -----------------------------------------------------------------------------
admin-backup-title = 백업 및 복원
admin-backup-create = 백업 생성
admin-backup-restore = 백업 복원
admin-backup-schedule = 백업 예약
admin-backup-list = 백업 기록
admin-backup-name = 백업 이름
admin-backup-size = 크기
admin-backup-created = 생성됨
admin-backup-download = 다운로드
admin-backup-delete = 삭제
admin-backup-confirm-restore = 이 백업을 복원하시겠습니까? 현재 데이터를 덮어쓰게 됩니다.
admin-backup-confirm-delete = 이 백업을 삭제하시겠습니까?
admin-backup-in-progress = 백업 진행 중...
admin-backup-completed = 백업이 성공적으로 완료되었습니다
admin-backup-failed = 백업 실패
admin-backup-restore-in-progress = 복원 진행 중...
admin-backup-restore-completed = 복원이 성공적으로 완료되었습니다.
admin-backup-restore-failed = 복원 실패

# -----------------------------------------------------------------------------
# Maintenance Mode
# -----------------------------------------------------------------------------
admin-maintenance-title = 유지 관리 모드
admin-maintenance-enable = 유지 관리 모드 활성화
admin-maintenance-disable = 유지 관리 모드 비활성화
admin-maintenance-status = 현황
admin-maintenance-active = 유지관리 모드가 활성화되었습니다.
admin-maintenance-inactive = 유지관리 모드가 비활성 상태입니다.
admin-maintenance-message = 유지보수 메시지
admin-maintenance-default-message = 현재 정기 점검을 진행하고 있습니다. 곧 다시 확인해 주세요.
admin-maintenance-allowed-ips = 허용된 IP 주소
admin-maintenance-confirm-enable = 유지 관리 모드를 활성화하시겠습니까? 사용자는 시스템에 액세스할 수 없습니다.

# -----------------------------------------------------------------------------
# Common Admin UI Elements
# -----------------------------------------------------------------------------
admin-required = 필수
admin-optional = 선택사항
admin-loading = 로드 중...
admin-saving = 저장 중...
admin-deleting = 삭제 중...
admin-confirm = 확인
admin-cancel = 취소
admin-save = 저장
admin-create = 만들기
admin-update = 업데이트
admin-delete = 삭제
admin-edit = 편집
admin-view = 보기
admin-close = 닫기
admin-back = 뒤로
admin-next = 다음
admin-previous = 이전
admin-refresh = 새로고침
admin-export = 수출
admin-import = 가져오기
admin-search = 검색
admin-filter = 필터
admin-clear = 지우기
admin-select = 선택
admin-select-all = 모두 선택
admin-deselect-all = 모두 선택 해제
admin-actions = 작업
admin-more-actions = 추가 작업
admin-no-data = 데이터가 없습니다
admin-error = 오류가 발생했습니다
admin-success = 성공
admin-warning = 경고
admin-info = 정보

# Table Pagination
admin-showing = { $total } 결과 중 { $from }~{ $to } 표시 중
admin-page = 페이지 { $current }/{ $total }
admin-items-per-page = 페이지당 항목
admin-go-to-page = 페이지로 이동

# Bulk Actions
admin-bulk-delete = 선택 항목 삭제
admin-bulk-export = 선택 내보내기
admin-bulk-activate = 선택 항목 활성화
admin-bulk-deactivate = 선택 항목 비활성화
admin-selected-count = { $개수 ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
