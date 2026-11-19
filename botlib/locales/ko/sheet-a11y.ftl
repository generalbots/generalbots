# =============================================================================
# General Bots — ko — Sheets grid accessibility (#2493)
# English is the source of truth: every key must be mirrored in all locales.
# Placeholders ({value}, {range}, ...) are preserved by the FTL loader.
# =============================================================================

# Grid semantics
sheet-a11y-grid-label = 스프레드시트 그리드
sheet-a11y-rowgroup-label = 그리드 행

# Formula bar and name box
sheet-a11y-formula-bar-label = 수식 입력줄
sheet-a11y-name-box-label = 이름 상자, 셀 참조를 입력하세요
sheet-a11y-name-box-placeholder = A1
sheet-a11y-invalid-reference = 잘못된 셀 참조입니다: {value}

# Cell labels and live announcements
sheet-a11y-cell-label = 셀 {address}: {value}
sheet-a11y-cell-formula-label = 수식 {formula}
sheet-a11y-live-selection = {range}, {rows}행 {cols}열, 합계 {sum}
sheet-a11y-live-edited = 셀 {address}이(가) {value}(으)로 업데이트되었습니다
sheet-a11y-clear-too-large = 선택 영역이 너무 커서 키보드로 지울 수 없습니다

# Accessibility checker
sheet-a11y-checker-title = 접근성 검사
sheet-a11y-checker-close = 접근성 검사 닫기
sheet-a11y-checker-run = 접근성 검사 실행
sheet-a11y-checker-summary = 접근성 문제 {count}개가 발견되었습니다
sheet-a11y-checker-empty = 접근성 문제가 발견되지 않았습니다.
sheet-a11y-checker-sheet = 시트
sheet-a11y-checker-sheet-fallback = 시트 {index}
sheet-a11y-severity-warning = 경고
sheet-a11y-finding-merged = {range}의 병합된 셀은 화면 읽기 프로그램을 혼동시킬 수 있습니다. 접근 가능한 통합 문서에서는 병합을 피하세요.
sheet-a11y-finding-color-only = {range}의 조건부 서식이 색상만 사용합니다. 아이콘, 레이블 또는 패턴을 추가하세요.
sheet-a11y-finding-no-alt = 이미지 {name}에 대체 텍스트가 없습니다.
sheet-a11y-unnamed-image = 이름 없음
sheet-a11y-finding-no-header = 워크시트 {sheet}에 데이터는 있지만 머리글 행이 없습니다.
