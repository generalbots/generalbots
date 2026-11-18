# =============================================================================
# General Bots — ko — Productivity apps (tasks, calendar, meet, mail, research, paper)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

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

# ── #2307 Tasks goal/monitor detail panel ──────────────────────────
tasks-detail-untitled-goal = 제목 없는 목표
tasks-detail-goal-progress = 목표 진행률
tasks-detail-current-value = 현재 값
tasks-detail-target-value = 목표 값
tasks-detail-percent = 진행률
tasks-detail-no-actions = 아직 작업이 없습니다
tasks-detail-pending-info = 누락된 정보
tasks-detail-scheduler-info = 스케줄러 정보
tasks-detail-monitor-info = 모니터 정보
tasks-detail-unavailable = 사용할 수 없음
tasks-detail-load-failed = 목표 세부 정보를 불러올 수 없습니다


# ── #2307 Database insert/confirm dialogs ──────────────────────────
db-insert-row = 행 삽입
db-insert-into = 삽입 대상
db-provide-value = 값을 하나 이상 입력하세요
db-insert-failed = 삽입 실패
db-confirm-title = 확인
db-confirm-accept = 확인
db-confirm-delete = 이 행을 삭제하시겠습니까?
db-confirm-drop = 테이블 삭제
db-delete-failed = 삭제 실패
db-drop-failed = 테이블 삭제 실패

# Tasks detail view (#2307)
tasks-detail-type-goal = Goal
tasks-detail-status-pending = Pending
tasks-detail-pending-title = Pending items
tasks-detail-required-reason = A reason is required.
tasks-detail-type-scheduler = Scheduler
tasks-detail-type-monitor = Monitor
tasks-detail-last-action = Last action
tasks-detail-next-run = Next run
tasks-detail-file = File
tasks-detail-interval = Interval
tasks-detail-last-check = Last check
tasks-detail-last-value = Last value
tasks-detail-progress = Progress
tasks-detail-app = App
tasks-detail-field = Field
tasks-detail-reason = Reason
tasks-detail-cron = Cron

# Database dialog copy (#2307)
db-error = Error
db-database = Database
db-empty-query = Execute a query to see results
db-insert = Insert
db-select-table = Select a table
tasks-load-error = 작업을 불러오지 못했습니다.
tasks-sign-in-to-load = 작업을 불러오려면 로그인하세요.
calendar-collapse-sidebar = 사이드바 접기
calendar-prev-month = 이전 달
calendar-next-month = 다음 달
calendar-prev-period = 이전 기간
calendar-next-period = 다음 기간
calendar-add-calendar = 캘린더 추가
paper-toggle-sidebar = 사이드바 전환
paper-insert-ai-response = AI 응답 삽입
paper-replace-selection = 선택 항목 바꾸기
paper-highlight = 강조
paper-align-left = 왼쪽 정렬
paper-align-center = 가운데 정렬
paper-align-right = 오른쪽 정렬
paper-insert-link = 링크 삽입
paper-insert-image = 이미지 삽입
paper-insert-table = 표 삽입
paper-bold = 굵게
paper-italic = 기울임꼴
paper-underline = 밑줄
paper-strikethrough = 취소선
products-view-grid = 격자 보기
products-view-list = 목록 보기
sheet-freeze-panes = 틀 고정

# semantic i18n key (#2542)
db-schema-title = 🗄️ 데이터베이스 스키마

# semantic i18n key (#2542)
db-loading-tables = 테이블 불러오는 중...

# semantic i18n key (#2542)
db-query-builder = 쿼리 빌더

# semantic i18n key (#2542)
db-run-query = 쿼리 실행

# semantic i18n key (#2542)
db-select-table-hint = 데이터를 보려면 테이블을 선택하세요

# semantic i18n key (#2542)
db-prev-page = ◀ 이전

# semantic i18n key (#2542)
db-page-info-default = 1 / 1 페이지

# semantic i18n key (#2542)
db-next-page = 다음 ▶

# semantic i18n key (#2542)
research-searching-sources = 소스 검색 중...

# semantic i18n key (#2542)
research-suggestion-latest-ai = 최신 AI 개발 동향

# semantic i18n key (#2542)
research-suggestion-quantum = 양자 컴퓨팅 설명

# semantic i18n key (#2542)
research-suggestion-software-architecture = 소프트웨어 아키텍처

# semantic i18n key (#2542)
research-suggestion-ai-productivity = AI 생산성 도구

# semantic i18n key (#2542)
research-related-heading = 관련

# semantic i18n key (#2542)
research-loading = 리서치 불러오는 중…

# semantic i18n key (#2542)
research-service-unavailable = 리서치 서비스를 사용할 수 없음

# semantic i18n key (#2542)
calendar-mini-month-placeholder = 2025년 1월

# semantic i18n key (#2542)
calendar-upcoming = 예정된 일정

# semantic i18n key (#2542)
calendar-import-export = 가져오기 / 내보내기

# semantic i18n key (#2542)
calendar-popup-event-title = 이벤트 제목

# semantic i18n key (#2542)
calendar-popup-location-placeholder = 회의실 A

# semantic i18n key (#2542)
calendar-popup-event-description = 이벤트 설명

# semantic i18n key (#2542)
calendar-personal = 개인

# semantic i18n key (#2542)
calendar-import-ical-title = iCal 파일 가져오기

# semantic i18n key (#2542)
calendar-select-ics-file = .ics 파일 선택

# semantic i18n key (#2542)
calendar-import-to-calendar-label = 캘린더로 가져오기

# semantic i18n key (#2542)
calendar-default-calendar = 기본 캘린더

# semantic i18n key (#2542)
calendar-loading = 캘린더 불러오는 중…

# semantic i18n key (#2542)
calendar-unavailable = 캘린더를 사용할 수 없음

# semantic i18n key (#2542)
tasks-progress-status = 상태

# semantic i18n key (#2542)
tasks-progress-sample-title = 사용자 인증 시스템 구현

# semantic i18n key (#2542)
tasks-progress-runtime = 실행 시간:

# semantic i18n key (#2542)
tasks-progress-sample-action = 토큰 만료 전략 선택

# semantic i18n key (#2542)
tasks-progress-estimated = 예상:

# semantic i18n key (#2542)
tasks-progress-sample-decision = 보안 구성에 대한 승인이 필요합니다

# semantic i18n key (#2542)
tasks-progress-log = 진행 로그

# semantic i18n key (#2542)
tasks-progress-terminal-live = 터미널(실시간 에이전트 활동)

# semantic i18n key (#2542)
tasks-progress-processed = 처리됨:

# semantic i18n key (#2542)
tasks-progress-data-points = 데이터 포인트

# semantic i18n key (#2542)
tasks-progress-sample-speed = 분당 ~8개 소스

# semantic i18n key (#2542)
tasks-progress-sample-insight = 경쟁 인사이트를 종합하는 중...

# semantic i18n key (#2542)
tasks-progress-sample-pricing = 가격 데이터를 교차 참조하는 중...

# semantic i18n key (#2542)
tasks-progress-view-details = 세부 정보 보기 ▸
