# =============================================================================
# General Bots — ko — Compliance
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# TOOLS / COMPLIANCE SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Compliance Main
# -----------------------------------------------------------------------------
compliance-title = API 규정 준수 보고서
compliance-subtitle = 모든 봇에 대한 보안 검사 - 비밀번호, 취약한 코드 및 잘못된 구성을 확인합니다.
compliance-export-report = 보고서 내보내기
compliance-run-scan = 규정 준수 검사 실행
compliance-scanning = 스캔 중...

# -----------------------------------------------------------------------------
# Bot Selector
# -----------------------------------------------------------------------------
compliance-all-bots = 모든 봇
compliance-select-bots = 봇 선택

# -----------------------------------------------------------------------------
# Stats Cards
# -----------------------------------------------------------------------------
compliance-critical = 심각
compliance-critical-desc = 즉각적인 조치가 필요함
compliance-high = 높음
compliance-high-desc = 보안 위험
compliance-medium = 중간
compliance-medium-desc = 해결해야 할 사항
compliance-low = 낮음
compliance-low-desc = 모범 사례
compliance-info = 정보
compliance-info-desc = 정보 제공

# -----------------------------------------------------------------------------
# Filters
# -----------------------------------------------------------------------------
compliance-filter-severity = 심각도
compliance-filter-type = 유형
compliance-filter-all-severities = 모든 심각도
compliance-filter-all-types = 모든 유형
compliance-search-issues = 검색 문제...

# -----------------------------------------------------------------------------
# Issue Types
# -----------------------------------------------------------------------------
compliance-type-password = 구성의 비밀번호
compliance-type-hardcoded = 하드코딩된 비밀
compliance-type-deprecated = 더 이상 사용되지 않는 키워드
compliance-type-fragile = 취약한 코드
compliance-type-config = 구성 문제

# -----------------------------------------------------------------------------
# Results Table
# -----------------------------------------------------------------------------
compliance-results = 결과
compliance-results-count = { $개수 ->
    [one] { $count } issue found
   *[other] { $count } issues found
}
compliance-col-severity = 심각도
compliance-col-issue = 이슈
compliance-col-location = 위치
compliance-col-details = 세부정보
compliance-col-action = 액션
compliance-view-details = 세부정보 보기
compliance-fix-issue = 문제 해결
compliance-ignore = 무시
compliance-no-issues = 발견된 문제 없음
compliance-no-issues-desc = 좋아요! 귀하의 봇은 규정을 준수합니다.

# -----------------------------------------------------------------------------
# Scan Progress
# -----------------------------------------------------------------------------
compliance-scan-in-progress = 스캔 진행 중...
compliance-scan-checking = 확인 중 { $item }...
compliance-scan-complete = 스캔 완료
compliance-scan-failed = 스캔 실패

## Compliance frameworks (enterprise-grade release)
compliance-frameworks = 프레임워크
compliance-new-framework = 새로운
compliance-framework-name = 이름
compliance-framework-version = 버전
compliance-framework-description = 설명
compliance-create-framework = 프레임워크 생성
compliance-controls = 컨트롤
compliance-add-control = 컨트롤 추가
compliance-control-id = 컨트롤 ID
compliance-control-title = 제목
compliance-control-category = 카테고리
compliance-control-description = 설명
compliance-mandatory = 필수
compliance-optional = 선택사항
compliance-evidence = 증거
compliance-attach-evidence = 증거 첨부
compliance-evidence-path = 파일 경로(드라이브 아티팩트)
compliance-evidence-type = 유형
compliance-approve = 승인하다
compliance-covered = 덮음
compliance-no-evidence = 증거 없음
compliance-export-csv = CSV 내보내기
compliance-archive = 아카이브
compliance-total-controls = 전체 통제
compliance-coverage = 적용 범위
compliance-no-frameworks = 아직 구성된 프레임워크가 없습니다.

# semantic i18n key (#2542)
admin-compliance-dashboard-title = 규정 준수 대시보드

# semantic i18n key (#2542)
admin-compliance-subtitle = SOC 2 Type II 규정 준수 상태, 통제 모니터링 및 감사 증거

# semantic i18n key (#2542)
admin-compliance-compliant = 준수

# semantic i18n key (#2542)
admin-compliance-overall-score = 전체 규정 준수 점수

# semantic i18n key (#2542)
admin-compliance-controls-passing = 통과한 통제

# semantic i18n key (#2542)
admin-compliance-evidence-items-due = 기한이 된 증거 항목

# semantic i18n key (#2542)
admin-compliance-until-next-audit = 다음 감사까지

# semantic i18n key (#2542)
admin-compliance-next-audit-date = 예정: 2025년 3월 7일

# semantic i18n key (#2542)
admin-compliance-trust-service-criteria = 신뢰 서비스 기준

# semantic i18n key (#2542)
admin-compliance-availability = 가용성

# semantic i18n key (#2542)
admin-compliance-processing-integrity = 처리 무결성

# semantic i18n key (#2542)
admin-compliance-confidentiality = 기밀성

# semantic i18n key (#2542)
admin-compliance-controls-requiring-attention = 주의가 필요한 통제

# semantic i18n key (#2542)
admin-compliance-control-mfa-privileged = 권한 있는 계정에 대한 다단계 인증

# semantic i18n key (#2542)
admin-compliance-due-jan-25-2025 = 기한: 2025년 1월 25일

# semantic i18n key (#2542)
admin-compliance-remediate = 수정

# semantic i18n key (#2542)
admin-compliance-control-incident-response-testing = 보안 사고 대응 계획 테스트

# semantic i18n key (#2542)
admin-compliance-issue-annual-test-overdue = 연간 테스트가 15일 지연됨

# semantic i18n key (#2542)
admin-compliance-control-dpa-documentation = 데이터 처리 계약 문서

# semantic i18n key (#2542)
admin-compliance-due-feb-1-2025 = 기한: 2025년 2월 1일

# semantic i18n key (#2542)
admin-compliance-control-data-classification-review = 데이터 분류 정책 검토

# semantic i18n key (#2542)
admin-compliance-issue-policy-review-pending = 정책 검토 승인 대기 중

# semantic i18n key (#2542)
admin-compliance-due-feb-15-2025 = 기한: 2025년 2월 15일

# semantic i18n key (#2542)
admin-compliance-control-privacy-notice-updates = 개인정보 고지 업데이트

# semantic i18n key (#2542)
admin-compliance-issue-privacy-policy-update = 새로운 데이터 수집에 맞게 개인정보처리방침 업데이트가 필요합니다

# semantic i18n key (#2542)
admin-compliance-due-feb-28-2025 = 기한: 2025년 2월 28일

# semantic i18n key (#2542)
admin-compliance-evidence-collection = 증거 수집

# semantic i18n key (#2542)
admin-compliance-current = 최신

# semantic i18n key (#2542)
admin-compliance-due-soon = 곧 기한

# semantic i18n key (#2542)
admin-compliance-evidence-pentest-q4 = 침투 테스트 보고서 2024년 4분기

# semantic i18n key (#2542)
admin-compliance-evidence-cc4-vuln = CC4.1.1 - 취약점 평가

# semantic i18n key (#2542)
admin-compliance-evidence-access-review-jan = 액세스 검토 보고서 - 2025년 1월

# semantic i18n key (#2542)
admin-compliance-evidence-cc6-access = CC6.2.1 - 액세스 제어

# semantic i18n key (#2542)
admin-compliance-due-in-3-days = 3일 후 기한

# semantic i18n key (#2542)
admin-compliance-evidence-backup-restore = 백업 복원 테스트 결과

# semantic i18n key (#2542)
admin-compliance-evidence-a1-backup = A1.2.3 - 백업 및 복구

# semantic i18n key (#2542)
admin-compliance-due-in-5-days = 5일 후 기한

# semantic i18n key (#2542)
admin-compliance-evidence-security-training = 보안 인식 교육 기록

# semantic i18n key (#2542)
admin-compliance-evidence-cc1-training = CC1.4.1 - 보안 교육

# semantic i18n key (#2542)
admin-compliance-view-all-evidence = 모든 증거 보기

# semantic i18n key (#2542)
admin-compliance-access-events = 액세스 이벤트

# semantic i18n key (#2542)
admin-compliance-configuration-changes = 구성 변경

# semantic i18n key (#2542)
admin-compliance-security-events = 보안 이벤트

# semantic i18n key (#2542)
admin-compliance-compliance-events = 규정 준수 이벤트

# semantic i18n key (#2542)
admin-compliance-failed-login-attempt = 로그인 시도 실패

# semantic i18n key (#2542)
admin-compliance-failed-login-details = 사용자: admin@company.com | IP: 192.168.1.105 | 사유: 잘못된 비밀번호

# semantic i18n key (#2542)
admin-compliance-user-logged-in = 사용자 로그인

# semantic i18n key (#2542)
admin-compliance-user-logged-in-details = 사용자: sarah.johnson@company.com | IP: 10.0.0.45
