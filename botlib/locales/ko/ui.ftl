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
# Chat
# -----------------------------------------------------------------------------
chat-title = 채팅
chat-placeholder = 메시지를 입력하세요...
chat-send = 보내기
chat-new-conversation = 새로운 대화
chat-history = 채팅 기록
chat-clear = 채팅 지우기
chat-export = 채팅 내보내기
chat-typing = { $name } 입력 중...
chat-online = 온라인
chat-offline = 오프라인
chat-last-seen = 마지막으로 본 날짜: { $time }
chat-mention-title = 참조 엔터티
chat-mention-placeholder = 메시지... (멘션하려면 @ 입력)
chat-mention-search = 항목 검색...
chat-mention-no-results = 검색결과가 없습니다
chat-mention-type-hint = 유형: 검색

# -----------------------------------------------------------------------------
# Drive / Files
# -----------------------------------------------------------------------------
drive-title = 드라이브
drive-upload = 업로드
drive-new-folder = 새 폴더
drive-empty = 아직 파일이 없습니다. 뭔가를 업로드하세요!
drive-search = 파일 검색...
drive-sort-name = 이름
drive-sort-date = 날짜
drive-sort-size = 크기
drive-sort-type = 유형
drive-view-grid = 그리드 보기
drive-view-list = 목록 보기
drive-selected = { $개수 ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
drive-file-size = { $크기 ->
    [bytes] { $value } B
    [kb] { $value } KB
    [mb] { $value } MB
    [gb] { $value } GB
   *[other] { $value } bytes
}
drive-drop-files = 업로드할 파일을 여기에 드롭하세요.

# -----------------------------------------------------------------------------
# Tasks
# -----------------------------------------------------------------------------
tasks-title = 작업
tasks-new = 새 작업
tasks-due-today = 오늘 마감
tasks-overdue = 연체
tasks-completed = 완료됨
tasks-all = 모든 작업
tasks-priority-high = 높은 우선순위
tasks-priority-medium = 중간 우선순위
tasks-priority-low = 낮은 우선순위
tasks-no-due-date = 마감일 없음
tasks-add-subtask = 하위 할 일 추가
tasks-mark-complete = 완료로 표시
tasks-mark-incomplete = 미완료로 표시
tasks-delete-confirm = 이 할 일을 삭제하시겠습니까?
tasks-count = { $개수 ->
    [zero] No tasks
    [one] { $count } task
   *[other] { $count } tasks
}

# -----------------------------------------------------------------------------
# Calendar
# -----------------------------------------------------------------------------
calendar-title = 달력
calendar-today = 오늘
calendar-new-event = 새로운 이벤트
calendar-all-day = 하루 종일
calendar-repeat = 반복
calendar-reminder = 알림
calendar-view-day = 일
calendar-view-week = 주
calendar-view-month = 월
calendar-view-year = 연도
calendar-no-events = 예정된 이벤트가 없습니다.
calendar-event-title = 이벤트 제목
calendar-event-location = 위치
calendar-event-description = 설명
calendar-event-attendees = 참석자

# -----------------------------------------------------------------------------
# Meet / Video Conferencing
# -----------------------------------------------------------------------------
meet-title = 만나다
meet-join = 회의 참가
meet-start = 회의 시작
meet-mute = 음소거
meet-unmute = 음소거 해제
meet-video-on = 카메라 켜기
meet-video-off = 카메라 끄기
meet-share-screen = 화면 공유
meet-stop-sharing = 공유 중지
meet-end-call = 통화 종료
meet-leave = 회의 나가기
meet-participants = { $개수 ->
    [one] { $count } participant
   *[other] { $count } participants
}
meet-waiting-room = 대기실
meet-admit = 인정하다
meet-remove = 제거
meet-chat = 회의 채팅
meet-raise-hand = 손 들기
meet-lower-hand = 낮은 손
meet-recording = 녹화 중
meet-start-recording = 녹음 시작
meet-stop-recording = 녹음 중지

# -----------------------------------------------------------------------------
# Mail / Email
# -----------------------------------------------------------------------------
mail-title = 메일
mail-compose = 작성
mail-inbox = 받은편지함
mail-sent = 보냄
mail-drafts = 초안
mail-trash = 휴지통
mail-spam = 스팸
mail-starred = 별표 표시됨
mail-archive = 아카이브
mail-to = 받는 사람
mail-cc = CC
mail-bcc = 숨은참조
mail-subject = 주제
mail-body = 메시지
mail-reply = 답장하다
mail-reply-all = 모두 답장
mail-forward = 앞으로
mail-send = 보내기
mail-discard = 폐기
mail-save-draft = 초안 저장
mail-attach = 파일 첨부
mail-unread = { $개수 ->
    [one] { $count } unread
   *[other] { $count } unread
}
mail-empty-inbox = 받은편지함이 비어 있습니다.
mail-no-subject = (제목 없음)

# -----------------------------------------------------------------------------
# Settings
# -----------------------------------------------------------------------------
settings-title = 설정
settings-general = 일반
settings-account = 계정
settings-notifications = 알림
settings-privacy = 개인정보 보호
settings-security = 보안 설정
settings-language = 언어
settings-theme = 테마
settings-theme-light = 빛
settings-theme-dark = 어둠
settings-theme-system = 시스템
settings-save = 변경 사항 저장
settings-saved = 설정이 성공적으로 저장되었습니다.
settings-timezone = 시간대
settings-date-format = 날짜 형식
settings-time-format = 시간 형식

# -----------------------------------------------------------------------------
# Auth / Login
# -----------------------------------------------------------------------------
auth-login = 로그인
auth-logout = 로그아웃
auth-signup = 가입
auth-forgot-password = 비밀번호를 잊으셨나요?
auth-reset-password = 비밀번호 재설정
auth-email = 이메일
auth-password = 비밀번호
auth-confirm-password = 비밀번호 확인
auth-remember-me = 나를 기억해
auth-login-success = 성공적으로 로그인되었습니다
auth-logout-success = 성공적으로 로그아웃되었습니다
auth-invalid-credentials = 잘못된 이메일 또는 비밀번호
auth-session-expired = 세션이 만료되었습니다

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
pagination-previous = 이전
pagination-next = 다음
pagination-first = 첫 번째
pagination-last = 마지막
pagination-page = 페이지 { $current }/{ $total }
pagination-showing = { $total } 중 { $from }~{ $to } 표시 중

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
# Settings - Language & Localization
# -----------------------------------------------------------------------------
settings-language = 언어
settings-language-desc = 선호하는 언어를 선택하세요
settings-display-language = 표시 언어
settings-language-affects = 애플리케이션의 모든 텍스트에 영향을 미칩니다.
settings-date-format = 날짜 형식
settings-date-format-desc = 날짜 표시 방법
settings-time-format = 시간 형식
settings-time-format-desc = 12시간제 또는 24시간제
settings-saved = 설정이 성공적으로 저장되었습니다.
settings-language-changed = 언어가 성공적으로 변경되었습니다
settings-reload-required = 변경 사항을 적용하려면 페이지를 다시 로드해야 합니다.

# Settings - Profile
settings-profile = 프로필 설정
settings-profile-desc = 개인 정보 및 기본 설정 관리
settings-profile-photo = 프로필 사진
settings-profile-photo-desc = 귀하의 프로필 사진이 다른 사용자에게 표시됩니다.
settings-upload-photo = 사진 업로드
settings-remove-photo = 제거
settings-basic-info = 기본정보
settings-display-name = 표시 이름
settings-username = 사용자 이름
settings-email-address = 이메일 주소
settings-bio = 바이오
settings-bio-placeholder = 자신에 대해 말해 보세요...
settings-contact-info = 연락처 정보
settings-phone-number = 전화번호
settings-location = 위치
settings-website = 웹사이트

# Settings - Security
settings-security = 보안 설정
settings-security-desc = 강화된 보안으로 계정을 보호하세요
settings-change-password = 비밀번호 변경
settings-change-password-desc = 보안 강화를 위해 정기적으로 비밀번호를 업데이트하세요.
settings-current-password = 현재 비밀번호
settings-new-password = 새 비밀번호
settings-confirm-password = 새 비밀번호 확인
settings-update-password = 비밀번호 업데이트
settings-2fa = 2단계 인증
settings-2fa-desc = 계정에 추가 보안 계층을 추가하세요
settings-authenticator-app = 인증자 앱
settings-authenticator-desc = 2FA 코드용 인증 앱 사용
settings-enable-2fa = 2FA 활성화
settings-disable-2fa = 2FA 비활성화
settings-active-sessions = 활성 세션
settings-active-sessions-desc = 활성 로그인 세션을 관리하세요
settings-this-device = 이 장치
settings-terminate-session = 종료
settings-terminate-all = 다른 모든 세션 종료

# Settings - Appearance
settings-appearance = 외관
settings-appearance-desc = 애플리케이션의 모양을 사용자 정의하세요.
settings-theme-selection = 테마
settings-theme-selection-desc = 원하는 색상 테마를 선택하세요
settings-theme-dark = 어둠
settings-theme-light = 빛
settings-theme-blue = 블루
settings-theme-purple = 보라색
settings-theme-green = 녹색
settings-theme-orange = 오렌지
settings-layout-preferences = 레이아웃 환경설정
settings-compact-mode = 컴팩트 모드
settings-compact-mode-desc = 더 많은 콘텐츠를 위해 간격을 줄입니다.
settings-show-sidebar = 사이드바 표시
settings-show-sidebar-desc = 항상 탐색 사이드바 표시
settings-animations = 애니메이션
settings-animations-desc = UI 애니메이션 및 전환 활성화

# Settings - Notifications
settings-notifications-title = 알림
settings-notifications-desc = 알림 수신 방법 제어
settings-email-notifications = 이메일 알림
settings-direct-messages = 직접 메시지
settings-direct-messages-desc = 새로운 다이렉트 메시지에 대한 이메일 받기
settings-mentions = 언급
settings-mentions-desc = 누군가가 귀하를 언급하면 이메일을 받습니다.
settings-weekly-digest = 주간 다이제스트
settings-weekly-digest-desc = 주간 활동 요약을 받아보세요.
settings-marketing = 마케팅
settings-marketing-desc = 뉴스 및 제품 업데이트 받기
settings-push-notifications = 푸시 알림
settings-enable-push = 푸시 알림 활성화
settings-enable-push-desc = 브라우저 푸시 알림 수신
settings-notification-sound = 소리
settings-notification-sound-desc = 알림 소리 재생
settings-in-app-notifications = 인앱 알림

# Settings - Storage
settings-storage = 저장
settings-storage-desc = 저장용량 사용량 관리
settings-storage-usage = 스토리지 사용량
settings-storage-used = { $total } 중 { $used } 사용됨
settings-storage-upgrade = 스토리지 업그레이드

# Settings - Privacy
settings-privacy-title = 개인정보 보호
settings-privacy-desc = 개인 정보 보호 설정 관리
settings-data-collection = 데이터 수집
settings-analytics = 분석
settings-analytics-desc = 익명의 사용 데이터를 보내 개선에 도움을 주세요.
settings-crash-reports = 충돌 보고서
settings-crash-reports-desc = 충돌 보고서 자동 보내기
settings-download-data = 데이터 다운로드
settings-download-data-desc = 모든 데이터의 사본을 받으세요
settings-delete-account = 계정 삭제
settings-delete-account-desc = 계정과 모든 데이터를 영구적으로 삭제합니다.
settings-delete-account-warning = 이 작업은 취소할 수 없습니다.

# Settings - Billing
settings-billing = 청구
settings-billing-desc = 구독 및 결제 방법 관리
settings-current-plan = 현재 계획
settings-free-plan = 무료 플랜
settings-pro-plan = 프로 플랜
settings-enterprise-plan = 엔터프라이즈 플랜
settings-upgrade-plan = 업그레이드 계획
settings-payment-methods = 결제 방법
settings-add-payment = 결제 수단 추가
settings-billing-history = 청구 내역

# -----------------------------------------------------------------------------
# Paper (Document Editor)
# -----------------------------------------------------------------------------
paper-title = 종이
paper-new-note = 새 메모
paper-search-notes = 메모 검색...
paper-quick-start = 빠른 시작
paper-template-blank = 공백
paper-template-meeting = 회의
paper-template-todo = 할 일
paper-template-research = 연구
paper-untitled = 제목 없음
paper-placeholder = 쓰기를 시작하거나 / 명령을 입력하세요...
paper-commands = 명령
paper-heading1 = 제목 1
paper-heading1-desc = 큰 섹션 제목
paper-heading2 = 제목 2
paper-heading2-desc = 중간 섹션 제목
paper-heading3 = 제목 3
paper-heading3-desc = 작은 섹션 제목
paper-paragraph = 단락
paper-paragraph-desc = 일반 텍스트
paper-bullet-list = 글머리 기호 목록
paper-bullet-list-desc = 순서가 없는 목록
paper-numbered-list = 번호 매기기 목록
paper-numbered-list-desc = 주문목록
paper-todo-list = 할 일 목록
paper-todo-list-desc = 확인 가능한 작업 목록
paper-quote = 견적
paper-quote-desc = 인용 인용구
paper-divider = 분배기
paper-divider-desc = 수평선
paper-code-block = 코드 블록
paper-code-block-desc = 형식화된 코드
paper-table = 테이블
paper-table-desc = 표 삽입
paper-image = 이미지
paper-image-desc = URL에서 이미지 삽입
paper-callout = 설명선
paper-callout-desc = 강조 표시된 정보 상자
paper-ai-write = AI 쓰기
paper-ai-write-desc = AI로 텍스트 생성
paper-ai-summarize = AI 요약
paper-ai-summarize-desc = 선택한 텍스트 요약
paper-ai-expand = AI 확장
paper-ai-expand-desc = 선택한 텍스트 확장
paper-ai-improve = AI 개선
paper-ai-improve-desc = 글쓰기 품질 향상
paper-ai-translate = AI 번역
paper-ai-translate-desc = 다른 언어로 번역
paper-ai-assistant = AI 어시스턴트
paper-ai-quick-actions = 빠른 작업
paper-ai-rewrite = 재작성
paper-ai-make-shorter = 짧게 만들기
paper-ai-make-longer = 더 길게 만들기
paper-ai-fix-grammar = 문법 수정
paper-ai-tone = 톤
paper-ai-tone-professional = 전문가
paper-ai-tone-casual = 캐주얼
paper-ai-tone-friendly = 우호적
paper-ai-tone-formal = 정장
paper-ai-translate-to = 다음으로 번역하다
paper-ai-custom-prompt = 사용자 정의 프롬프트
paper-ai-custom-placeholder = 당신이 원하는 것을 설명하십시오 ...
paper-ai-generate = 생성
paper-ai-response = AI 응답
paper-ai-apply = 적용
paper-ai-regenerate = 재생성
paper-ai-copy = 복사
paper-word-count = { $count } 단어
paper-char-count = { $count }자
paper-saved = 저장됨
paper-saving = 저장 중...
paper-last-edited = 최종 수정일: { $time }
paper-last-edited-now = 마지막 수정: 방금
paper-export = 문서 내보내기
paper-export-pdf = PDF
paper-export-docx = 워드(.docx)
paper-export-markdown = 마크다운
paper-export-html = HTML
paper-export-txt = 일반 텍스트

# Additional Chat translations
chat-voice = 음성 입력
chat-message-placeholder = 메시지...

# Drive translations
drive-my-drive = 내 드라이브
drive-shared = 나와 공유됨
drive-recent = 최근
drive-starred = 별표 표시됨
drive-trash = 휴지통
drive-loading-storage = 저장공간 로드 중...
drive-storage-used = { $total } 중 { $used } 사용됨
drive-empty-folder = 이 폴더는 비어 있습니다.

# Tasks translations
tasks-active = 활성 의도
tasks-awaiting = 결정을 기다리는 중
tasks-paused = 일시중지됨
tasks-blocked = 차단/문제
tasks-time-saved = 절약된 활동 시간:
tasks-input-placeholder = 무엇을 하고 싶나요? 예: 'CRM 앱 만들기' 또는 '내일 John에게 전화하라고 알려줘'

# Calendar additional translations
calendar-my-calendars = 내 캘린더

# Email additional translations
email-scheduled = 예정됨
email-tracking = 추적

# Email folder translations
email-inbox = 받은편지함
email-starred = 별표 표시됨
email-sent = 보냄
email-drafts = 초안
email-spam = 스팸
email-trash = 휴지통
email-compose = 작성

# -----------------------------------------------------------------------------
# Research
# -----------------------------------------------------------------------------
research-title = 연구
research-search-placeholder = 무엇이든 물어보세요...
research-collections = 컬렉션
research-new-collection = 새로운 컬렉션
research-recent = 최근
research-academic = 학술
research-code = 코드
research-internal = 내부
research-search-all = 모든 것을 검색하세요
research-academic-papers = 학술 논문
research-code-docs = 코드 및 문서
research-internal-kb = 내부 지식 기반
research-sources = 출처
research-trending = 인기 급상승
research-pro-search = 프로 검색
research-include-images = 이미지 포함
research-try-asking = 다음에 대해 물어보세요
research-related = 관련 질문
research-view-all-sources = 모든 소스 보기
research-export-citations = 인용 내보내기
research-save-to-collection = 컬렉션에 저장

# -----------------------------------------------------------------------------
# Admin Panel (additional UI keys)
# -----------------------------------------------------------------------------
admin-panel-title = 관리자 패널
admin-quick-actions = 빠른 작업
admin-create-user = 사용자 생성
admin-create-group = 그룹 만들기
admin-register-dns = DNS 등록
admin-recent-activity = 최근 활동
admin-system-health = 시스템 상태

# -----------------------------------------------------------------------------
# Meet (additional keys)
# -----------------------------------------------------------------------------
meet-new-meeting = 새 회의
meet-join-meeting = 회의 참가
meet-active-rooms = 활성 룸
meet-room-title = 회의실
meet-record = 녹화
meet-camera = 카메라
meet-share = 공유
meet-info = 정보
meet-more = 더보기
meet-share-meeting = 회의 공유
meet-meeting-title = 회의 제목
meet-meeting-code = 회의 코드
meet-meeting-link = 회의 링크
meet-send-invite = 초대장 보내기

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
# Analytics (additional UI keys)
# -----------------------------------------------------------------------------
analytics-dashboard-title = 분석 대시보드
analytics-last-hour = 지난 시간
analytics-last-6h = 지난 6시간
analytics-last-24h = 지난 24시간
analytics-last-7d = 지난 7일
analytics-last-30d = 지난 30일

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
notifications-title = 알림
notifications-clear = 모두 지우기
notifications-empty = 알림 없음

# -----------------------------------------------------------------------------
# All Applications
# -----------------------------------------------------------------------------
nav-all-apps = 모든 애플리케이션

# =============================================================================
# AUTH SCREENS - Complete translations for login, register, forgot/reset password
# =============================================================================

# -----------------------------------------------------------------------------
# Login Screen
# -----------------------------------------------------------------------------
auth-welcome-back = 돌아온 것을 환영합니다, { $name }!
auth-sign-in-to-account = 일반 봇 계정에 로그인하세요
auth-email-address = 이메일 주소
auth-email-placeholder = you@example.com
auth-password-placeholder = ••••••••
auth-sign-in = 로그인
auth-or-continue-with = 아니면 계속해서
auth-dont-have-account = 계정이 없나요?
auth-create-account = 계정 만들기
auth-google = 구글
auth-microsoft = 마이크로소프트
auth-github = GitHub
auth-apple = 사과

# -----------------------------------------------------------------------------
# Two-Factor Authentication
# -----------------------------------------------------------------------------
auth-2fa-title = 2단계 인증
auth-2fa-subtitle = 인증 앱의 6자리 코드를 입력하세요.
auth-2fa-verify = 코드 확인
auth-2fa-didnt-receive = 코드를 받지 못하셨나요?
auth-2fa-resend = 코드 재전송
auth-2fa-back-to-login = 로그인으로 돌아가기
auth-2fa-trust-device = 이 기기를 신뢰하세요
auth-2fa-trust-desc = 30일 동안 이 기기에서 2FA를 요청하지 마세요.

# -----------------------------------------------------------------------------
# Register Screen
# -----------------------------------------------------------------------------
auth-create-your-account = 계정 만들기
auth-join-general-bots = 일반 봇에 가입하고 구축을 시작하세요
auth-first-name = 이름
auth-last-name = 성
auth-create-password = 비밀번호 생성
auth-confirm-your-password = 비밀번호 확인
auth-password-strength = 비밀번호 강도
auth-password-weak = 약함
auth-password-fair = 박람회
auth-password-good = 좋음
auth-password-strong = 강한
auth-password-req-length = 8자 이상
auth-password-req-uppercase = 대문자 1개
auth-password-req-lowercase = 소문자 1개
auth-password-req-number = 하나의 숫자
auth-password-req-special = 특수 문자 1개
auth-passwords-match = 비밀번호가 일치합니다.
auth-passwords-dont-match = 비밀번호가 일치하지 않습니다.
auth-agree-terms = 나는 다음에 동의한다.
auth-terms-of-service = 서비스 약관
auth-and = 그리고
auth-privacy-policy = 개인 정보 보호 정책
auth-sign-up = 가입
auth-already-have-account = 이미 계정이 있나요?
auth-sign-in-link = 로그인
auth-registration-success = 계정이 성공적으로 생성되었습니다!
auth-check-email = 계정을 확인하려면 이메일을 확인하세요.
auth-email-sent-to = 다음 주소로 확인 링크를 보냈습니다.
auth-resend-verification = 확인 이메일 다시 보내기
auth-go-to-login = 로그인으로 이동

# -----------------------------------------------------------------------------
# Forgot Password Screen
# -----------------------------------------------------------------------------
auth-forgot-password-title = 비밀번호를 잊으셨나요?
auth-forgot-password-subtitle = 걱정 마세요! 이메일을 입력하시면 재설정 지침을 보내드립니다.
auth-send-reset-link = 재설정 링크 보내기
auth-back-to-login = 로그인으로 돌아가기
auth-reset-email-sent = 재설정 이메일이 전송되었습니다!
auth-reset-instructions = 비밀번호 재설정 지침을 다음 주소로 보냈습니다.
auth-check-inbox = 받은 편지함을 확인하세요
auth-check-spam = 스팸폴더가 안보이면 확인해 보세요
auth-link-expires = 링크는 1시간 후에 만료됩니다.
auth-resend-email = 이메일 재전송
auth-didnt-receive-email = 이메일을 받지 못하셨나요?

# -----------------------------------------------------------------------------
# Reset Password Screen
# -----------------------------------------------------------------------------
auth-reset-password-title = 비밀번호 재설정
auth-reset-password-subtitle = 귀하의 계정에 대한 새로운 보안 비밀번호를 만드세요
auth-new-password = 새 비밀번호
auth-confirm-new-password = 새 비밀번호 확인
auth-reset-password-btn = 비밀번호 재설정
auth-password-reset-success = 비밀번호 재설정이 성공했습니다!
auth-password-updated = 귀하의 비밀번호가 업데이트되었습니다. 이제 새 비밀번호로 로그인할 수 있습니다.
auth-invalid-token = 유효하지 않거나 만료된 링크
auth-invalid-token-desc = 이 비밀번호 재설정 링크는 유효하지 않거나 만료되었습니다. 새로운 것을 요청하세요.
auth-request-new-link = 새 링크 요청

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

# =============================================================================
# SOURCES SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# Sources Main
# -----------------------------------------------------------------------------
sources-title = 출처
sources-subtitle = 리포지토리, 앱, 프롬프트, 템플릿 및 MCP 서버
sources-search = 소스 검색...

# -----------------------------------------------------------------------------
# Sources Tabs
# -----------------------------------------------------------------------------
sources-repositories = 저장소
sources-apps = 앱
sources-prompts = 프롬프트
sources-templates = 템플릿
sources-servers = MCP 서버
sources-models = AI 모델
sources-news = 뉴스

# -----------------------------------------------------------------------------
# Repository Cards
# -----------------------------------------------------------------------------
sources-repo-connect = 연결하다
sources-repo-disconnect = 연결 끊기
sources-repo-browse = 찾아보기
sources-repo-connected = 연결됨
sources-repo-disconnected = 연결이 끊김
sources-repo-stars = 별
sources-repo-forks = 포크
sources-repo-last-updated = 마지막 업데이트

# -----------------------------------------------------------------------------
# Prompt Cards
# -----------------------------------------------------------------------------
sources-prompt-use = 사용
sources-prompt-copy = 복사
sources-prompt-edit = 편집
sources-prompt-rating = 등급
sources-prompt-uses = 용도

# -----------------------------------------------------------------------------
# Server Cards
# -----------------------------------------------------------------------------
sources-server-active = 활성
sources-server-inactive = 비활성
sources-server-connect = 연결하다
sources-server-configure = 구성

# -----------------------------------------------------------------------------
# Model Cards
# -----------------------------------------------------------------------------
sources-model-active = 활성
sources-model-coming-soon = 출시 예정
sources-model-provider = 공급자
sources-model-context = 맥락
sources-model-tokens = 토큰

# -----------------------------------------------------------------------------
# App Cards
# -----------------------------------------------------------------------------
sources-app-open = 열기
sources-app-edit = 편집
sources-app-installed = 설치됨
sources-app-install = 설치

# -----------------------------------------------------------------------------
# Template Cards
# -----------------------------------------------------------------------------
sources-template-preview = 미리보기
sources-template-use = 템플릿 사용
sources-template-components = 구성요소

# -----------------------------------------------------------------------------
# Categories
# -----------------------------------------------------------------------------
sources-category-all = 모두
sources-category-development = 개발
sources-category-productivity = 생산성
sources-category-communication = 커뮤니케이션
sources-category-analytics = 분석
sources-category-security = 보안
sources-category-other = 기타

# -----------------------------------------------------------------------------
# Empty States
# -----------------------------------------------------------------------------
sources-empty-repos = 연결된 저장소가 없습니다.
sources-empty-apps = 사용 가능한 앱이 없습니다.
sources-empty-prompts = 메시지가 없습니다.
sources-empty-templates = 사용 가능한 템플릿이 없습니다.
sources-empty-servers = 구성된 MCP 서버가 없습니다.
sources-empty-models = 사용 가능한 모델이 없습니다.
sources-empty-results = 검색결과가 없습니다
sources-empty-results-desc = 검색 또는 필터를 조정해 보세요.

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

# =============================================================================
# ATTENDANT / CRM SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# CRM Disabled State
# -----------------------------------------------------------------------------
attendant-crm-disabled = CRM 기능이 활성화되지 않음
attendant-crm-disabled-desc = Attendant Console에서는 이 봇에 대해 CRM 기능을 활성화해야 합니다. 이를 통해 인간 에이전트는 봇에서 전송된 대화를 수신하고 응답할 수 있습니다.
attendant-crm-enable-instruction = CRM 기능을 활성화하려면 봇의
attendant-crm-config-file = 구성.csv
attendant-crm-create-attendant = 그런 다음
attendant-crm-attendant-file = 수행자.csv
attendant-crm-configure-team = 팀 구성을 위한 파일

# -----------------------------------------------------------------------------
# Queue Sidebar
# -----------------------------------------------------------------------------
attendant-title = 교환원 콘솔
attendant-status-online = 온라인
attendant-status-busy = 바쁨
attendant-status-away = 떨어져
attendant-status-offline = 오프라인
attendant-status-ready = 온라인 - 대화 준비 완료
attendant-status-busy-msg = 다른 용무 중 - 대화 처리 중
attendant-status-away-msg = 어웨이 - 곧 돌아올 예정
attendant-status-offline-msg = 오프라인 - 사용할 수 없음

# -----------------------------------------------------------------------------
# Queue Stats
# -----------------------------------------------------------------------------
attendant-waiting = 대기 중
attendant-active = 활성
attendant-resolved = 해결됨
attendant-mine = 광산

# -----------------------------------------------------------------------------
# Queue Filters
# -----------------------------------------------------------------------------
attendant-filter-all = 모두
attendant-filter-waiting = 대기 중
attendant-filter-mine = 광산
attendant-filter-priority = 우선순위

# -----------------------------------------------------------------------------
# Conversation List
# -----------------------------------------------------------------------------
attendant-no-conversations = 대기열에 대화가 없습니다.
attendant-new-conversations-appear = 여기에 새 대화가 표시됩니다.
attendant-unread = 읽지 않음
attendant-typing = 타이핑 중...
attendant-select-conversation = 대화를 선택하세요
attendant-select-conversation-desc = 응답을 시작하려면 대기열에서 대화를 선택하세요.

# -----------------------------------------------------------------------------
# Channel Tags
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = 왓츠앱
attendant-channel-teams = 팀
attendant-channel-instagram = 인스타그램
attendant-channel-web = 웹
attendant-channel-telegram = 텔레그램
attendant-channel-email = 이메일

# -----------------------------------------------------------------------------
# Priority Tags
# -----------------------------------------------------------------------------
attendant-priority-urgent = 긴급
attendant-priority-high = 높음
attendant-priority-normal = 보통

# -----------------------------------------------------------------------------
# Chat Area
# -----------------------------------------------------------------------------
attendant-message-placeholder = 메시지를 입력하세요...
attendant-send = 보내기
attendant-attach-file = 파일첨부
attendant-insert-emoji = 이모티콘 삽입
attendant-quick-responses = 빠른 응답
attendant-transfer = 환승
attendant-resolve = 해결
attendant-more-actions = 추가 작업

# -----------------------------------------------------------------------------
# Quick Responses
# -----------------------------------------------------------------------------
attendant-quick-greeting = 안녕하세요! 오늘은 무엇을 도와드릴까요?
attendant-quick-thanks = 양해해 주셔서 감사합니다.
attendant-quick-checking = 제가 확인해 보겠습니다.
attendant-quick-moment = 잠시만 기다려주세요.

# -----------------------------------------------------------------------------
# Transfer Modal
# -----------------------------------------------------------------------------
attendant-transfer-title = 대화 전송
attendant-transfer-to = 다음으로 환승
attendant-transfer-reason = 이유(선택사항)
attendant-transfer-reason-placeholder = 이 대화를 전달하는 이유는 무엇인가요?
attendant-transfer-cancel = 취소
attendant-transfer-confirm = 환승

# -----------------------------------------------------------------------------
# AI Insights Sidebar
# -----------------------------------------------------------------------------
attendant-ai-insights = AI 인사이트
attendant-ai-summary = 대화 요약
attendant-ai-sentiment = 고객 감정
attendant-sentiment-positive = 긍정적
attendant-sentiment-neutral = 중립
attendant-sentiment-negative = 네거티브
attendant-smart-replies = 스마트 답장
attendant-confidence = 자신감
attendant-source = 소스

# -----------------------------------------------------------------------------
# Customer Details
# -----------------------------------------------------------------------------
attendant-customer-details = 고객 세부정보
attendant-customer-name = 이름
attendant-customer-email = 이메일
attendant-customer-phone = 전화
attendant-customer-location = 위치
attendant-customer-tags = 태그

# -----------------------------------------------------------------------------
# Conversation History
# -----------------------------------------------------------------------------
attendant-history = 역사
attendant-history-resolved = 해결됨
attendant-history-transferred = 전송됨
attendant-history-abandoned = 버려진
attendant-view-history = 전체 기록 보기

# -----------------------------------------------------------------------------
# Toast Messages
# -----------------------------------------------------------------------------
attendant-toast-transferred = 대화가 성공적으로 전송되었습니다.
attendant-toast-resolved = 해결된 것으로 표시된 대화
attendant-toast-assigned = 나에게 할당된 대화
attendant-toast-error = 오류가 발생했습니다
attendant-toast-connection-lost = 연결이 끊겼습니다. 다시 연결하는 중...
attendant-toast-connection-restored = 연결이 복원되었습니다.

# =============================================================================
# CRM
# =============================================================================

# -----------------------------------------------------------------------------
# CRM Navigation & General
# -----------------------------------------------------------------------------
crm-title = CRM
crm-pipeline = 파이프라인
crm-leads = 리드
crm-opportunities = 기회
crm-accounts = 계정
crm-contacts = 연락처
crm-activities = 활동

# -----------------------------------------------------------------------------
# CRM Entities
# -----------------------------------------------------------------------------
crm-lead = 리드
crm-lead-desc = 자격이 없는 잠재고객
crm-opportunity = 기회
crm-opportunity-desc = 적격 판매 기회
crm-account = 계정
crm-account-desc = 회사 또는 조직
crm-contact = 연락처
crm-contact-desc = 계좌에 있는 사람
crm-activity = 활동
crm-activity-desc = 작업, 전화 또는 이메일

# -----------------------------------------------------------------------------
# CRM Actions
# -----------------------------------------------------------------------------
crm-qualify = 자격
crm-convert = 변환
crm-won = 원
crm-lost = 분실
crm-new-lead = 새로운 리드
crm-new-opportunity = 새로운 기회
crm-new-account = 새 계정
crm-new-contact = 새 연락처

# -----------------------------------------------------------------------------
# CRM Fields
# -----------------------------------------------------------------------------
crm-stage = 무대
crm-value = 가치
crm-probability = 확률
crm-close-date = 마감일
crm-company = 회사
crm-phone = 전화
crm-email = 이메일
crm-source = 소스
crm-owner = 소유자

# -----------------------------------------------------------------------------
# CRM Pipeline Stages
# -----------------------------------------------------------------------------
crm-pipeline-new = 새로운
crm-pipeline-contacted = 연락함
crm-pipeline-qualified = 자격을 갖춘
crm-pipeline-proposal = 제안
crm-pipeline-negotiation = 협상
crm-pipeline-closed-won = 마감 수주
crm-pipeline-closed-lost = 마감 분실

# -----------------------------------------------------------------------------
# CRM Stats & Metrics
# -----------------------------------------------------------------------------
crm-subtitle = 리드, 기회, 고객 관리
crm-stage-lead = 리드
crm-stage-qualified = 자격을 갖춘
crm-stage-proposal = 제안
crm-stage-negotiation = 협상
crm-stage-won = 원
crm-stage-lost = 분실
crm-conversion-rate = 전환율
crm-pipeline-value = 파이프라인 가치
crm-avg-deal = 평균 거래 규모
crm-won-month = 이번 달 승리

# -----------------------------------------------------------------------------
# CRM Empty States
# -----------------------------------------------------------------------------
crm-no-leads = 리드를 찾을 수 없습니다
crm-no-opportunities = 추천이 없습니다.
crm-no-accounts = 계정을 찾을 수 없습니다
crm-no-contacts = 연락처를 찾을 수 없습니다.
crm-drag-hint = 카드를 드래그하여 스테이지 변경

# =============================================================================
# Billing
# =============================================================================

# -----------------------------------------------------------------------------
# Billing Navigation & General
# -----------------------------------------------------------------------------
billing-title = 청구
billing-invoices = 송장
billing-payments = 결제
billing-quotes = 인용문
billing-dashboard = 대시보드

# -----------------------------------------------------------------------------
# Billing Entities
# -----------------------------------------------------------------------------
billing-invoice = 송장
billing-invoice-desc = 고객에게 청구
billing-payment = 결제
billing-payment-desc = 결제 완료
billing-quote = 견적
billing-quote-desc = 가격 견적

# -----------------------------------------------------------------------------
# Billing Status
# -----------------------------------------------------------------------------
billing-due-date = 만기일
billing-overdue = 연체
billing-paid = 유료
billing-pending = 보류 중
billing-draft = 초안
billing-sent = 보냄
billing-partial = 부분
billing-cancelled = 취소됨

# -----------------------------------------------------------------------------
# Billing Actions
# -----------------------------------------------------------------------------
billing-new-invoice = 새 송장
billing-new-quote = 새로운 견적
billing-new-payment = 새로운 결제
billing-send-invoice = 송장 보내기
billing-record-payment = 기록 지불
billing-mark-paid = 유료로 표시
billing-void = 공허

# -----------------------------------------------------------------------------
# Billing Fields
# -----------------------------------------------------------------------------
billing-amount = 금액
billing-tax = 세금
billing-subtotal = 소계
billing-total = 합계
billing-discount = 할인
billing-line-items = 광고 항목
billing-add-item = 항목 추가
billing-remove-item = 항목 제거
billing-customer = 고객
billing-issue-date = 발행일
billing-payment-terms = 지불 조건
billing-notes = 메모
billing-invoice-number = 송장 번호
billing-quote-number = 견적 번호

# -----------------------------------------------------------------------------
# Billing Reports
# -----------------------------------------------------------------------------
billing-revenue = 수익
billing-outstanding = 뛰어난
billing-this-month = 이번 달
billing-last-month = 지난달
billing-total-paid = 총 지불액
billing-total-overdue = 총 연체
billing-subtitle = 송장, 결제 및 견적
billing-revenue-month = 이번 달 수익
billing-total-revenue = 총 수익
billing-paid-month = 이번 달에 지급됨

# -----------------------------------------------------------------------------
# Billing Empty States
# -----------------------------------------------------------------------------
billing-no-invoices = 인보이스를 찾을 수 없습니다.
billing-no-payments = 결제를 찾을 수 없습니다.
billing-no-quotes = 견적을 찾을 수 없습니다.

# =============================================================================
# Products
# =============================================================================

# -----------------------------------------------------------------------------
# Products Navigation & General
# -----------------------------------------------------------------------------
products-title = 제품
products-catalog = 카탈로그
products-services = 서비스
products-price-lists = 가격표
products-inventory = 인벤토리

# -----------------------------------------------------------------------------
# Products Entities
# -----------------------------------------------------------------------------
products-product = 제품
products-product-desc = 실제 또는 디지털 제품
products-service = 서비스
products-service-desc = 서비스 제공
products-price-list = 가격표
products-price-list-desc = 가격 책정 계층

# -----------------------------------------------------------------------------
# Products Actions
# -----------------------------------------------------------------------------
products-new-product = 신제품
products-new-service = 새로운 서비스
products-new-price-list = 새로운 가격표
products-new-pricelist = 새로운 가격표
products-edit-product = 제품 편집
products-duplicate = 중복

# -----------------------------------------------------------------------------
# Products Fields
# -----------------------------------------------------------------------------
products-sku = SKU
products-category = 카테고리
products-price = 가격
products-unit = 단위
products-stock = 주식
products-cost = 비용
products-margin = 여백
products-barcode = 바코드

# -----------------------------------------------------------------------------
# Products Status
# -----------------------------------------------------------------------------
products-in-stock = 재고 있음
products-out-of-stock = 품절
products-low-stock = 재고 부족
products-active = 활성
products-inactive = 비활성
products-featured = 추천
products-archived = 보관됨

# -----------------------------------------------------------------------------
# Products Stats & Metrics
# -----------------------------------------------------------------------------
products-subtitle = 제품, 서비스, 가격 관리
products-items = 제품
products-pricelists = 가격표
products-total-products = 총 제품
products-total-services = 종합 서비스

# -----------------------------------------------------------------------------
# Products Empty States
# -----------------------------------------------------------------------------
products-no-products = 제품을 찾을 수 없습니다
products-no-services = 서비스를 찾을 수 없습니다.
products-no-price-lists = 가격표를 찾을 수 없습니다.

# =============================================================================
# Tickets (Support Cases)
# =============================================================================

# -----------------------------------------------------------------------------
# Tickets Navigation & General
# -----------------------------------------------------------------------------
tickets-title = 티켓
tickets-cases = 사례
tickets-open = 열기
tickets-closed = 휴무
tickets-all = 모든 티켓
tickets-my-tickets = 내 티켓

# -----------------------------------------------------------------------------
# Tickets Entities
# -----------------------------------------------------------------------------
tickets-case = 케이스
tickets-case-desc = 지원 티켓
tickets-resolution = 해상도
tickets-resolution-desc = AI가 제안하는 솔루션

# -----------------------------------------------------------------------------
# Tickets Priority
# -----------------------------------------------------------------------------
tickets-priority = 우선순위
tickets-priority-low = 낮음
tickets-priority-medium = 중간
tickets-priority-high = 높음
tickets-priority-urgent = 긴급

# -----------------------------------------------------------------------------
# Tickets Status
# -----------------------------------------------------------------------------
tickets-status = 상태
tickets-status-new = 새로운
tickets-status-open = 열기
tickets-status-pending = 보류 중
tickets-status-resolved = 해결됨
tickets-status-closed = 휴무
tickets-status-on-hold = 보류 중

# -----------------------------------------------------------------------------
# Tickets Actions
# -----------------------------------------------------------------------------
tickets-new-ticket = 신규 티켓
tickets-assign = 할당
tickets-reassign = 재할당
tickets-escalate = 에스컬레이션
tickets-resolve = 해결
tickets-reopen = 재개설
tickets-close = 닫기
tickets-merge = 병합

# -----------------------------------------------------------------------------
# Tickets Fields
# -----------------------------------------------------------------------------
tickets-subject = 주제
tickets-description = 설명
tickets-category = 카테고리
tickets-assigned = 할당 대상
tickets-unassigned = 할당되지 않음
tickets-created = 생성됨
tickets-updated = 업데이트됨
tickets-response-time = 응답 시간
tickets-resolution-time = 해결 시간
tickets-customer = 고객
tickets-internal-notes = 내부 메모
tickets-attachments = 첨부파일

# -----------------------------------------------------------------------------
# Tickets AI Features
# -----------------------------------------------------------------------------
tickets-ai-suggestion = AI 제안
tickets-apply-suggestion = 제안 적용
tickets-ai-summary = AI 요약
tickets-similar-tickets = 유사한 티켓
tickets-suggested-articles = 추천 기사

# -----------------------------------------------------------------------------
# Tickets Empty States
# -----------------------------------------------------------------------------
tickets-no-tickets = 티켓을 찾을 수 없습니다
tickets-no-open = 오픈 티켓 없음
tickets-no-closed = 마감된 티켓 없음

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
security-title = 보안
security-subtitle = 계정 보안 설정 관리
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

# CRM / Mail / Campaigns integration keys
crm-email = 이메일
crm-compose-email = 이메일 작성
crm-send-email = 이메일 보내기
mail-snooze = 스누즈
mail-snooze-later-today = 오늘 늦게(오후 6시)
mail-snooze-tomorrow = 내일 (오전 8시)
mail-snooze-next-week = 다음 주 (월 오전 8시)
mail-crm-log = CRM에 로그인
mail-crm-create-lead = 리드 생성
mail-add-to-list = 목록에 추가
campaign-send-email = 이메일 보내기

# -----------------------------------------------------------------------------
# OAuth Account Linking (Settings)
# -----------------------------------------------------------------------------
oauth-connected-accounts = 연결된 계정
oauth-connect = 연결하다
oauth-unlink = 연결 해제
oauth-not-connected = 연결되지 않음
oauth-linked = 연결됨
oauth-no-accounts = 아직 연결된 계정이 없습니다.
oauth-loading = 연결된 계정 로드 중…

## Payment cards (Stripe SetupIntent)
cards-title = 결제 및 카드
cards-saved = 저장된 카드
cards-hint = 카드는 당사의 결제 제공업체에 의해 안전하게 보관됩니다. 카드 번호는 우리 서버에 절대 도달하지 않습니다.
cards-add = 카드 추가
cards-add-first = 첫 번째 카드 추가
cards-none = 아직 저장된 카드가 없습니다.
cards-empty-hint = 자동 청구 및 더 빠른 결제를 활성화하려면 카드를 추가하세요. 카드 정보를 입력하기 위해 당사의 보안 결제 제공업체로 리디렉션됩니다.
cards-default = 기본값
cards-set-default = 기본값 설정
cards-default-btn = 기본 카드
cards-remove = 제거
cards-remove-confirm = 이 카드를 삭제하시겠습니까?
cards-expires = 만료
cards-load-error = 저장된 카드를 로드할 수 없습니다.
cards-add-error = 카드를 추가할 수 없습니다.
cards-default-error = 기본값을 업데이트할 수 없습니다.
cards-remove-error = 카드를 제거할 수 없습니다.
cards-default-updated = 기본 카드가 업데이트되었습니다.
cards-removed = 카드가 삭제됨

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

## Sources connectors (enterprise-grade release)
sources-connectors = 커넥터
sources-add-connector = 커넥터 추가
sources-connector-name = 이름
sources-connector-description = 설명
sources-connector-schedule = 동기화 일정(크론)
sources-connector-type = 유형
sources-connector-host = 호스트
sources-connector-port = 항구
sources-connector-database = 데이터베이스
sources-connector-username = 사용자 이름
sources-connector-password = 비밀번호
sources-connector-base-url = 기본 URL
sources-connector-api-key = API 키
sources-connector-credentials-hint = 자격 증명은 Vault에 저장되며 저장 후에는 다시 표시되지 않습니다.
sources-create-connector = 커넥터 만들기
sources-test-connector = 테스트
sources-sync-now = 지금 동기화
sources-remove-connector = 제거
sources-connector-health = 건강
sources-connector-last-sync = 마지막 동기화
sources-no-connectors = 구성된 커넥터가 없습니다.

# VDI (remote desktop)
vdi-title = 가상 데스크탑
vdi-new-connection = 새로운 연결
vdi-connection-name = 연결 이름
vdi-host = 호스트
vdi-port = 항구
vdi-protocol = 프로토콜
vdi-rdp-password = RDP 비밀번호
vdi-rdp-domain = RDP 도메인(선택 사항)
vdi-save-connect = 저장 및 연결
vdi-cancel = 취소
vdi-connect = 연결하다
vdi-delete = 삭제
vdi-no-connections = 아직 연결이 없습니다.
vdi-create-first = 시작하려면 새 연결을 만드세요.
vdi-connecting = 연결 중...
vdi-connected = 연결됨
vdi-disconnected = 연결이 끊김
vdi-error = 오류
vdi-clipboard-sent = 클립보드가 전송되었습니다.
vdi-ctrl-alt-del-sent = Ctrl+Alt+Del 보냄
vdi-rdp = RDP
vdi-vnc = VNC
attendant-attach = 파일 첨부
attendant-emoji = 이모지
attendant-uploading = 업로드 중...
attendant-attach-error = 업로드 실패
attendant-emoji-search = 이모지 검색...
meet-record = 녹화
meet-record-title = 회의 녹화
meet-recording = 녹화 중
meet-recordings = 녹화
meet-recordings-empty = 아직 녹화가 없습니다

player-playlists = 재생목록
player-playlist-new = 새 재생목록
player-playlist-name = 재생목록 이름
player-playlist-create = 만들기
player-playlist-rename = 이름 바꾸기
player-playlist-delete = 재생목록 삭제
player-playlist-add = 재생목록에 추가
player-playlist-remove = 제거
player-playlist-play = 플레이
player-playlist-empty = 재생목록이 생성되지 않았습니다.
player-playlist-empty-items = 빈 재생목록
player-playlist-add-current = + 현재 미디어 추가
player-playlist-created = 재생목록이 생성되었습니다.
player-playlist-deleted = 재생목록이 삭제되었습니다.
player-playlist-renamed = 재생목록 이름이 변경됨
player-playlist-item-added = 재생목록에 추가됨
player-playlist-item-removed = 재생목록에서 삭제됨
player-playlist-error = 재생목록을 로드하지 못했습니다.
