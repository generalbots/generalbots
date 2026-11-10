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
