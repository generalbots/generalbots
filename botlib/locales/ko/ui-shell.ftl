# =============================================================================
# General Bots — ko — Desktop shell, navigation, controls
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# General Bots - English UI Translations
# =============================================================================
# -----------------------------------------------------------------------------
# Navigation
# -----------------------------------------------------------------------------
nav-home = 홈
nav-chat = 채팅
nav-drive = 드라이브
nav-tasks = 작업
nav-mail = 메일
nav-calendar = 달력
nav-meet = 만나다
nav-paper = 종이
nav-video = 비디오
nav-research = 연구
nav-analytics = 분석
nav-settings = 설정
nav-admin = 관리자
nav-monitoring = 모니터링
nav-sources = 출처
nav-tools = 도구
nav-attendant = 수행자
nav-learn = 배우다
nav-crm = CRM
nav-billing = 청구
nav-products = 제품
nav-tickets = 티켓
nav-docs = 문서
nav-sheet = 시트
nav-slides = 슬라이드
nav-social = 소셜
nav-all-apps = 모든 애플리케이션
nav-people = 사람
nav-editor = 편집자
nav-dashboards = 대시보드
nav-security = 보안
nav-designer = 디자이너
nav-project = 프로젝트
nav-canvas = 캔버스
nav-goals = 목표
nav-player = 플레이어
nav-workspace = 작업공간

# -----------------------------------------------------------------------------
# Dashboard
# -----------------------------------------------------------------------------
dashboard-title = 대시보드
dashboard-welcome = 돌아온 것을 환영합니다, { $name }!
dashboard-quick-actions = 빠른 작업
dashboard-recent-activity = 최근 활동
dashboard-no-activity = 아직 최근 활동이 없습니다. 탐험을 시작하세요!
dashboard-analytics = 분석

# -----------------------------------------------------------------------------
# Quick Actions
# -----------------------------------------------------------------------------
quick-start-chat = 채팅 시작
quick-upload-files = 파일 업로드
quick-new-task = 새 작업
quick-compose-email = 이메일 작성
quick-start-meeting = 회의 시작
quick-new-event = 새로운 이벤트

# -----------------------------------------------------------------------------
# Application Cards
# -----------------------------------------------------------------------------
app-chat-name = 채팅
app-chat-desc = AI 기반 대화. 질문하고, 도움을 받고, 작업을 자동화하세요.
app-drive-name = 드라이브
app-drive-desc = 모든 파일을 위한 클라우드 스토리지입니다. 업로드하고 정리하고 공유하세요.
app-tasks-name = 작업
app-tasks-desc = 할 일 목록, 우선순위, 마감일을 체계적으로 정리하세요.
app-mail-name = 메일
app-mail-desc = AI 지원 글쓰기 및 스마트 구성을 갖춘 이메일 클라이언트입니다.
app-calendar-name = 달력
app-calendar-desc = 회의, 이벤트를 예약하고 시간을 효과적으로 관리하세요.
app-meet-name = 만나다
app-meet-desc = 화면 공유 및 실시간 전사를 통한 화상 회의.
app-paper-name = 종이
app-paper-desc = AI 지원으로 문서를 작성하세요. 메모, 보고서 등
app-research-name = 연구
app-research-desc = 모든 소스에서 AI 기반 검색 및 발견이 가능합니다.
app-analytics-name = 분석
app-analytics-desc = 사용량과 통찰력을 추적하는 대시보드 및 보고서.

# -----------------------------------------------------------------------------
# Suite Header
# -----------------------------------------------------------------------------
suite-title = 일반 봇 스위트
suite-tagline = AI 기반 생산성 작업 공간. 채팅하고, 협업하고, 창작하세요.
suite-new-intent = 새로운 의도

# -----------------------------------------------------------------------------
# AI Panel
# -----------------------------------------------------------------------------
ai-developer = AI 개발자
ai-developing = 개발 중: { $project }
ai-quick-actions = 빠른 작업
ai-add-field = 필드 추가
ai-change-color = 색상 변경
ai-add-validation = 유효성 검사 추가
ai-export-data = 데이터 내보내기
ai-placeholder = 수정사항을 입력하세요...
ai-thinking = AI는 생각하고 있다…
ai-status-online = 온라인
ai-status-offline = 오프라인

# -----------------------------------------------------------------------------
# Search
# -----------------------------------------------------------------------------
search-placeholder = 검색...
search-no-results = 검색결과가 없습니다
search-results = { $개수 ->
    [one] { $count } result
   *[other] { $count } results
}
search-in-progress = 검색 중...
search-advanced = 고급 검색
search-filters = 필터
search-clear-filters = 필터 지우기

# -----------------------------------------------------------------------------
# Pagination
# -----------------------------------------------------------------------------
# -----------------------------------------------------------------------------
# Tables
# -----------------------------------------------------------------------------
table-no-data = 데이터가 없습니다
table-loading = 데이터 로드 중...
table-actions = 작업
table-select-all = 모두 선택
table-deselect-all = 모두 선택 해제
table-export = 수출
table-import = 가져오기

# -----------------------------------------------------------------------------
# Forms
# -----------------------------------------------------------------------------
form-required = 필수
form-optional = 선택사항
form-submit = 제출
form-reset = 재설정
form-clear = 지우기
form-uploading = 업로드 중...
form-processing = 처리 중...

# -----------------------------------------------------------------------------
# Modals / Dialogs
# -----------------------------------------------------------------------------
modal-confirm-title = 조치 확인
modal-confirm-message = 계속하시겠습니까?
modal-delete-title = 삭제 확인
modal-delete-message = 이 작업은 취소할 수 없습니다. 확실합니까?

# -----------------------------------------------------------------------------
# Tooltips
# -----------------------------------------------------------------------------
tooltip-copy = 클립보드에 복사
tooltip-copied = 복사되었습니다!
tooltip-expand = 펼치기
tooltip-collapse = 접기
tooltip-refresh = 새로고침
tooltip-download = 다운로드
tooltip-upload = 업로드
tooltip-print = 인쇄
tooltip-fullscreen = 전체 화면
tooltip-exit-fullscreen = 전체 화면 종료

# -----------------------------------------------------------------------------
# Common Labels (additional)
# -----------------------------------------------------------------------------
label-username = 사용자 이름
label-email = 이메일
label-display-name = 표시 이름
label-password = 비밀번호
label-role = 역할
label-group-name = 그룹 이름
label-hostname = 호스트 이름
label-record-type = 레코드 유형
label-target = 대상
label-your-name = 당신의 이름

# -----------------------------------------------------------------------------
# Actions (additional)
# -----------------------------------------------------------------------------
action-register = 등록

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
notifications-title = 알림
notifications-clear = 모두 지우기
notifications-empty = 알림 없음

# #1853 — desktop chrome, launchers and notification settings.
window-ready = 준비됨
window-loading = 앱 로드 중…
window-minimize = 최소화
window-maximize = 최대화
window-fullscreen = 전체 화면
window-exit-fullscreen = 전체 화면 종료
window-close = 닫기
app-loading = 앱 로드 중…
start-menu-search-placeholder = 앱을 찾으려면 입력하세요…
start-menu-recent = 최근 항목
start-menu-no-apps = 앱을 찾을 수 없습니다
start-menu-user = 사용자
category-ai = AI
category-business = 비즈니스
category-office = 오피스
category-dev = 개발
category-system = 시스템
category-tools = 도구
sidebar-apps = 앱
sidebar-apps-toggle = 앱 표시/숨기기
sidebar-apps-toggle-aria = 앱 패널 전환
sidebar-apps-filter = 앱 필터…
sidebar-apps-filter-aria = 앱 필터
sidebar-preview = 미리보기 모드
sidebar-preview-hint = 아직 미리보기 중인 앱 표시
sidebar-preview-toggle = 미리보기 모드 전환
sidebar-app-preview-title = {app} (미리보기)
sidebar-apps-count = 앱 {count}개
sidebar-apps-count-preview = {count} (+ 미리보기 {preview}개)
spotlight-placeholder = 앱, 파일 검색… 또는 Concierge 목표 입력
spotlight-hint = ↑↓ 이동 · Enter 실행 · Ctrl+Space 또는 Esc 닫기
spotlight-empty = 결과 없음
spotlight-goal = 목표 실행: “{goal}”
spotlight-goal-subtitle = Concierge · 앱 전반에 걸쳐 계획 및 실행
palette-placeholder = 앱과 명령 검색. 또는 @ <명령>을 입력해 어시스턴트에 전송...
palette-empty = 일치하는 앱이나 명령이 없습니다
palette-kind-app = 앱
palette-hint = ↑↓ 이동 · Enter 실행 · Esc 닫기 · Ctrl+K 전환
palette-open-app = {app} 열기
palette-value-prompt = {name}: {summary} — {key} 값:
palette-execute = {method} {path}를 실행하고 결과를 요약합니다.
palette-use-command = {command} 명령을 사용하여 {summary}.
toast-dismiss = 닫기
settings-language-display = 표시 언어
settings-language-display-desc = 애플리케이션 인터페이스 언어를 선택하세요
settings-language-regional = 지역 설정
settings-language-regional-desc = 날짜, 시간, 숫자 형식을 구성합니다
settings-language-date-format = 날짜 형식
settings-language-time-format = 시간 형식
settings-language-timezone = 시간대
settings-language-first-day = 주의 첫째 날
settings-language-time-12h = 12시간제 (1:30 PM)
settings-language-time-24h = 24시간제 (13:30)
settings-language-sunday = 일요일
settings-language-monday = 월요일
settings-email-notifications-desc = 워크스페이스 알림 발신자가 보내며, 연결된 Gmail/Outlook 사서함으로는 전송되지 않습니다.
settings-push-enable = 푸시 알림 사용
settings-push-enable-desc = 브라우저 푸시 알림을 받습니다
settings-push-sound = 소리
settings-push-sound-desc = 알림 소리를 재생합니다
settings-desktop-notifications = 데스크톱 알림
settings-desktop-notifications-desc = 데스크톱에 알림을 표시합니다
settings-badge-count = 배지 수
settings-badge-count-desc = 읽지 않은 수 배지를 표시합니다
settings-save-preferences = 환경설정 저장
settings-notifications-saved = 알림 환경설정이 저장되었습니다
settings-notification-recipient = 수신 사서함
settings-notification-recipient-desc = 비워 두면 기본 Mail 계정 또는 프로필 이메일을 사용합니다.
language-search-placeholder = 언어 검색…
settings-profile-title = 프로필 설정
settings-security-title = 보안 설정
