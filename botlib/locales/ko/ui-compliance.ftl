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
