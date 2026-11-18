# =============================================================================
# General Bots — ko — Admin, monitoring and system apps
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Panel (additional UI keys)
# -----------------------------------------------------------------------------
admin-panel-title = 관리자 패널

# =============================================================================
# MONITORING SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Monitoring Dashboard
# -----------------------------------------------------------------------------
monitoring-title = 모니터링 대시보드
monitoring-toggle-view = 보기 전환
monitoring-last-updated = 마지막 업데이트
monitoring-live-view = 라이브 뷰
monitoring-grid-view = 그리드 보기

# -----------------------------------------------------------------------------
# Monitoring Panels
# -----------------------------------------------------------------------------
monitoring-sessions = 세션
monitoring-messages = 메시지
monitoring-resources = 자원
monitoring-services = 서비스
monitoring-active-bots = 활성 봇
monitoring-loading = 로드 중...

# -----------------------------------------------------------------------------
# Service Status
# -----------------------------------------------------------------------------
monitoring-status-running = 달리기
monitoring-status-warning = 경고
monitoring-status-stopped = 중지됨
monitoring-status-healthy = 건강하다
monitoring-status-degraded = 저하됨
monitoring-status-down = 아래로

# -----------------------------------------------------------------------------
# Resource Metrics
# -----------------------------------------------------------------------------
monitoring-cpu = CPU
monitoring-memory = 메모리
monitoring-disk = 디스크
monitoring-network = 네트워크
monitoring-requests-per-sec = 요청/초
monitoring-active-connections = 활성 연결
monitoring-uptime = 가동 시간

# -----------------------------------------------------------------------------
# Logs
# -----------------------------------------------------------------------------
monitoring-logs-title = 시스템 로그
monitoring-logs-filter = 로그 필터링
monitoring-logs-level = 로그 수준
monitoring-logs-all = 모든 레벨
monitoring-logs-debug = 디버그
monitoring-logs-info = 정보
monitoring-logs-warning = 경고
monitoring-logs-error = 오류
monitoring-logs-critical = 심각
monitoring-logs-search = 로그 검색...
monitoring-logs-no-results = 로그를 찾을 수 없습니다.

# -----------------------------------------------------------------------------
# Health
# -----------------------------------------------------------------------------
monitoring-health-title = 시스템 상태
monitoring-health-status = 건강상태
monitoring-health-services = 서비스 상태
monitoring-health-database = 데이터베이스
monitoring-health-cache = 캐시
monitoring-health-queue = 메시지 대기열
monitoring-health-storage = 저장
monitoring-health-external = 외부 서비스

# -----------------------------------------------------------------------------
# Metrics
# -----------------------------------------------------------------------------
monitoring-metrics-title = 성능 지표
monitoring-metrics-response-time = 응답 시간
monitoring-metrics-throughput = 처리량
monitoring-metrics-error-rate = 오류율
monitoring-metrics-latency = 대기 시간

# -----------------------------------------------------------------------------
# Alerts
# -----------------------------------------------------------------------------
monitoring-alerts-title = 시스템 경고
monitoring-alerts-active = 활성 경고
monitoring-alerts-resolved = 해결됨
monitoring-alerts-all = 모든 경고
monitoring-alert-severity = 심각도
monitoring-alert-critical = 심각
monitoring-alert-high = 높음
monitoring-alert-medium = 중간
monitoring-alert-low = 낮음
monitoring-alert-info = 정보
monitoring-alert-acknowledge = 승인
monitoring-alert-resolve = 해결
monitoring-no-alerts = 활성 알림 없음

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
security-tab-compliance = API 규정 준수 보고서
security-tab-protection = 보호
security-export-report = 보고서 내보내기
security-run-scan = 규정 준수 검사 실행
security-critical = 심각
security-critical-desc = 즉각적인 조치가 필요함
security-high = 높음
security-high-desc = 보안 위험
security-medium = 중간
security-medium-desc = 해결해야 할 사항
security-low = 낮음
security-low-desc = 모범 사례
security-info = 정보
security-info-desc = 정보 제공
security-filter-severity = 심각도:
security-filter-all-severities = 모든 심각도
security-filter-type = 유형:
security-filter-all-types = 모든 유형
security-type-password = 구성의 비밀번호
security-type-hardcoded = 하드코딩된 비밀
security-type-deprecated = 더 이상 사용되지 않는 키워드
security-type-fragile = 취약한 코드
security-type-config = 구성 문제
security-results = 규정 준수 문제
security-col-severity = 심각도
security-col-issue = 이슈 유형
security-col-location = 위치
security-col-details = 설명
security-col-action = 액션

# -----------------------------------------------------------------------------
# Learn Module
# -----------------------------------------------------------------------------
learn-title = 배우다
learn-my-progress = 나의 진행 상황
learn-completed = 완료됨
learn-in-progress = 진행 중
learn-certificates = 인증서
learn-time-spent = 소요 시간
learn-categories = 카테고리
learn-all-courses = 모든 강좌
learn-mandatory = 필수
learn-compliance = 규정 준수
learn-security = 보안
learn-skills = 기술
learn-onboarding = 온보딩
learn-difficulty = 난이도
learn-my-certificates = 내 인증서
learn-view-all = 모두 보기

# -----------------------------------------------------------------------------
# Workspace Module
# -----------------------------------------------------------------------------
workspace-title = 작업공간
workspace-search-pages = 페이지 검색...
workspace-recent = 최근
workspace-favorites = 즐겨찾기
workspace-pages = 페이지
workspace-templates = 템플릿
workspace-trash = 휴지통
workspace-settings = 설정

# -----------------------------------------------------------------------------
# Player Module
# -----------------------------------------------------------------------------
player-title = 미디어 플레이어
player-no-file = 선택한 파일이 없습니다.
player-search = 파일 검색...
player-recent = 최근
player-files = 파일

# -----------------------------------------------------------------------------
# Goals Module
# -----------------------------------------------------------------------------
goals-title = 목표 및 OKR
goals-dashboard = 대시보드
goals-objectives = 목표
goals-alignment = 정렬
goals-ai-suggestions = AI 제안

# semantic i18n key (#2542)
admin-ops-dashboard-title = 운영 대시보드

# semantic i18n key (#2542)
admin-ops-dashboard-subtitle = 실시간 시스템 메트릭, 분산 추적 및 성능 모니터링

# semantic i18n key (#2542)
admin-ops-auto-refresh-on = 자동 새로고침: 켜짐

# semantic i18n key (#2542)
admin-ops-avg-response = 평균 응답

# semantic i18n key (#2542)
admin-ops-requests-min = 요청/분

# semantic i18n key (#2542)
admin-ops-disk-io = 디스크 I/O

# semantic i18n key (#2542)
admin-ops-iops-read-write = IOPS(읽기/쓰기)

# semantic i18n key (#2542)
admin-ops-in-95-out-50 = 수신: 95 / 송신: 50

# semantic i18n key (#2542)
admin-ops-stable = 안정

# semantic i18n key (#2542)
admin-ops-request-metrics = 요청 메트릭

# semantic i18n key (#2542)
admin-ops-errors-label = 오류

# semantic i18n key (#2542)
admin-ops-distributed-tracing = 분산 추적

# semantic i18n key (#2542)
admin-ops-all-traces = 모든 추적

# semantic i18n key (#2542)
admin-ops-errors-only = 오류만

# semantic i18n key (#2542)
admin-ops-api-requests = API 요청

# semantic i18n key (#2542)
admin-ops-post-api-chat-message = POST /api/chat/message

# semantic i18n key (#2542)
admin-ops-trace-a1b2c3d4e5f6 = trace_a1b2c3d4e5f6

# semantic i18n key (#2542)
admin-ops-get-api-kb-search = GET /api/kb/search

# semantic i18n key (#2542)
admin-ops-trace-b2c3d4e5f6g7 = trace_b2c3d4e5f6g7

# semantic i18n key (#2542)
admin-ops-post-api-billing-checkout = POST /api/billing/checkout

# semantic i18n key (#2542)
admin-ops-trace-c3d4e5f6g7h8 = trace_c3d4e5f6g7h8

# semantic i18n key (#2542)
admin-ops-get-api-user-profile = GET /api/user/profile

# semantic i18n key (#2542)
admin-ops-trace-d4e5f6g7h8i9 = trace_d4e5f6g7h8i9

# semantic i18n key (#2542)
admin-ops-api-gateway = API 게이트웨이

# semantic i18n key (#2542)
admin-ops-primary-2-replicas = 기본 + 복제본 2개

# semantic i18n key (#2542)
admin-ops-qdrant-vector-db = Qdrant(벡터 DB)

# semantic i18n key (#2542)
admin-ops-redis-cache = Redis 캐시

# semantic i18n key (#2542)
admin-ops-cluster-mode-3-shards = 클러스터 모드 | 샤드 3개

# semantic i18n key (#2542)
admin-ops-llm-service = LLM 서비스

# semantic i18n key (#2542)
admin-ops-openai-gpt-4-high-latency = OpenAI GPT-4 | 높은 지연 시간

# semantic i18n key (#2542)
admin-ops-object-storage = 오브젝트 스토리지

# semantic i18n key (#2542)
admin-ops-s3-compatible = S3 호환

# semantic i18n key (#2542)
admin-ops-error-tracking = 오류 추적

# semantic i18n key (#2542)
admin-ops-view-all-errors = 모든 오류 보기

# semantic i18n key (#2542)
admin-ops-warnings = 경고

# semantic i18n key (#2542)
admin-ops-payment-processing-failed-stripe-timeout = 결제 처리 실패: Stripe API 시간 초과

# semantic i18n key (#2542)
admin-ops-billing-checkout-rs-145 = billing/checkout.rs:145

# semantic i18n key (#2542)
admin-ops-rate-limit-approaching-openai = OpenAI API 속도 제한에 근접 중

# semantic i18n key (#2542)
admin-ops-llm-openai-rs-89 = llm/openai.rs:89

# semantic i18n key (#2542)
admin-ops-core-analytics-rs-234 = core/analytics.rs:234

# semantic i18n key (#2542)
admin-ops-cache-miss-rate-above-threshold = 캐시 미스율이 임계값을 초과했습니다

# semantic i18n key (#2542)
admin-ops-cache-redis-rs-67 = cache/redis.rs:67

# semantic i18n key (#2542)
admin-ops-endpoint-performance = 엔드포인트 성능

# semantic i18n key (#2542)
admin-ops-by-requests = 요청별

# semantic i18n key (#2542)
admin-ops-by-latency = 지연 시간별

# semantic i18n key (#2542)
admin-ops-by-error-rate = 오류율별

# semantic i18n key (#2542)
admin-ops-post = POST

# semantic i18n key (#2542)
admin-ops-api-chat-message = /api/chat/message

# semantic i18n key (#2542)
admin-ops-req-min = 요청/분

# semantic i18n key (#2542)
admin-ops-errors-metric = 오류

# semantic i18n key (#2542)
admin-ops-api-kb-search = /api/kb/search

# semantic i18n key (#2542)
admin-roles-title = 역할 관리

# semantic i18n key (#2542)
admin-roles-subtitle = 역할을 구성하고 권한을 할당하여 제품군 전체의 액세스를 제어합니다

# semantic i18n key (#2542)
admin-roles-sidebar-title = 역할

# semantic i18n key (#2542)
admin-roles-system-filter = 시스템 역할

# semantic i18n key (#2542)
admin-roles-custom-filter = 사용자 지정 역할

# semantic i18n key (#2542)
admin-roles-select-title = 역할 선택

# semantic i18n key (#2542)
admin-roles-select-hint = 목록에서 역할을 선택하여 권한을 보고 관리하세요

# semantic i18n key (#2542)
admin-roles-name-label = 역할 이름

# semantic i18n key (#2542)
admin-roles-name-badge-placeholder = role_name

# semantic i18n key (#2542)
admin-roles-description-placeholder = 역할 설명이 여기에 표시됩니다

# semantic i18n key (#2542)
admin-roles-permissions-label = 권한

# semantic i18n key (#2542)
admin-roles-assigned-users-tab = 할당된 사용자

# semantic i18n key (#2542)
admin-roles-assigned-groups-tab = 할당된 그룹

# semantic i18n key (#2542)
admin-roles-permission-assignment = 권한 할당

# semantic i18n key (#2542)
admin-roles-permission-assignment-help = 이중 목록을 사용하여 이 역할에 권한을 할당하세요

# semantic i18n key (#2542)
admin-roles-available-permissions = 사용 가능한 권한

# semantic i18n key (#2542)
admin-roles-category-documents = 문서

# semantic i18n key (#2542)
admin-roles-category-spreadsheets = 스프레드시트

# semantic i18n key (#2542)
admin-roles-category-presentations = 프레젠테이션

# semantic i18n key (#2542)
admin-roles-category-meetings = 회의

# semantic i18n key (#2542)
admin-roles-category-ai-bots = AI 및 봇

# semantic i18n key (#2542)
admin-roles-category-automation = 자동화

# semantic i18n key (#2542)
admin-roles-assigned-permissions = 할당된 권한

# semantic i18n key (#2542)
admin-roles-no-permissions-assigned = 할당된 권한 없음

# semantic i18n key (#2542)
admin-roles-reset-to-default = 기본값으로 재설정

# semantic i18n key (#2542)
admin-roles-users-with-role = 이 역할의 사용자

# semantic i18n key (#2542)
admin-roles-groups-with-role = 이 역할의 그룹

# semantic i18n key (#2542)
admin-roles-change-history = 역할 변경 기록

# semantic i18n key (#2542)
admin-roles-create-new-role = 새 역할 만들기

# semantic i18n key (#2542)
admin-roles-name-help = 소문자, 공백 없음(밑줄 사용)

# semantic i18n key (#2542)
admin-roles-copy-permissions-from = 다음에서 권한 복사

# semantic i18n key (#2542)
admin-roles-start-with-no-permissions = 권한 없이 시작

# semantic i18n key (#2542)
admin-roles-standard-user = 표준 사용자

# semantic i18n key (#2542)
admin-roles-power-user = 고급 사용자

# semantic i18n key (#2542)
admin-roles-viewer = 뷰어

# semantic i18n key (#2542)
admin-roles-guest-user = 게스트 사용자

# semantic i18n key (#2542)
admin-roles-create-role-btn = 역할 만들기

# semantic i18n key (#2542)
admin-roles-delete-role-title = 역할 삭제

# semantic i18n key (#2542)
admin-roles-delete-warning = 이렇게 하면 모든 사용자 및 그룹에서 역할이 제거됩니다. 이 작업은 취소할 수 없습니다.

# semantic i18n key (#2542)
admin-roles-assign-to-users = 사용자에게 역할 할당

# semantic i18n key (#2542)
admin-roles-search-users-label = 사용자 검색

# semantic i18n key (#2542)
admin-roles-type-to-search-users = 입력하여 사용자 검색

# semantic i18n key (#2542)
admin-roles-expiration-optional = 만료(선택 사항)

# semantic i18n key (#2542)
admin-roles-expiration-help = 영구 할당의 경우 비워 두세요

# semantic i18n key (#2542)
admin-roles-assign-to-groups = 그룹에 역할 할당

# semantic i18n key (#2542)
admin-roles-search-groups-label = 그룹 검색

# semantic i18n key (#2542)
admin-roles-type-to-search-groups = 입력하여 그룹 검색

# semantic i18n key (#2542)
admin-roles-assign-role-btn = 역할 할당
