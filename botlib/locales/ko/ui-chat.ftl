# =============================================================================
# General Bots — ko — Chat
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

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

# Additional Chat translations
chat-camera = 카메라
chat-camera-title = 사진 찍기
chat-camera-capture = 사진 찍기
chat-camera-cancel = 취소
chat-camera-sent = 사진을 보냈습니다.
chat-camera-denied = 카메라 권한이 거부되었습니다. 사진을 찍으려면 시스템 설정에서 데스크톱 앱의 카메라 액세스를 허용하세요.
chat-camera-missing = 이 기기에서 카메라를 찾을 수 없습니다.
chat-camera-error = 카메라를 시작할 수 없습니다.
chat-camera-unsupported = 이 데스크톱 앱은 카메라 액세스를 제공하지 않습니다.
voice-mode-title = 음성 대화
voice-mode-listening = 듣는 중… 자연스럽게 말하세요
voice-mode-thinking = 생각 중…
voice-mode-speaking = 말하는 중…
voice-mode-idle = 음성 대화
voice-mode-end = 음성 대화 종료
chat-message-placeholder = 메시지...

# Chat message controls and notices
chat-copy-message = 메시지 복사
chat-copied = 복사됨
chat-message-copied = 메시지가 복사되었습니다
chat-copy-failed = 복사 실패
chat-listen-tts = 듣기 (TTS)
chat-listen-message = 이 메시지 듣기
chat-image-unavailable = 이미지를 사용할 수 없음
chat-open = 열기
chat-expand = 펼치기
chat-collapse = 접기
chat-preview = 미리보기
chat-close-preview = 미리보기 닫기
chat-preview-load-failed = 미리보기를 불러올 수 없습니다.
chat-show-more = 더 보기
chat-show-less = 간단히 보기
chat-tool = 도구
chat-param = 매개변수
chat-params = 매개변수
chat-tool-result = 도구 결과
chat-retry = 다시 시도
chat-open-as-document = 문서로 열기
chat-show-more-history = 더 보기…
chat-disconnected-reconnecting = 채팅 연결이 끊어졌습니다. 다시 연결 중...
chat-connect-retrying = 채팅 서버에 연결할 수 없습니다. 다시 시도 중...
chat-connection-lost = 연결이 끊어졌습니다. 다시 연결하는 중...
chat-attachment-sent = 첨부 파일 전송됨: 
chat-attachments-sent = 첨부 파일 전송됨: 
chat-message-not-delivered = 메시지가 전달되지 않았습니다 — 다시 연결 중
chat-attachment-read-failed = 첨부 파일을 읽을 수 없습니다. 다시 첨부하세요.
chat-attachment-queued = 곧 연결됩니다 — 첨부 파일이 대기열에 추가됨
chat-reconnecting-queued = 다시 연결 중 — 연결이 복구되면 메시지가 전송됩니다.
chat-bot-private-signin = 이 봇은 비공개입니다 — 계속하려면 로그인하세요
chat-approved-plan = 승인된 계획: 
chat-retrying-attachment = 첨부 파일 다시 시도 중
chat-expanded-preview = 확장된 미리보기

# #2362 — chat code/artifact control labels
chat-code-show-all = Show all
chat-code-lines = lines
chat-artifact-save-drive = Save to Drive
chat-artifact-download-svg = Download SVG
chat-artifact-download-png = Download PNG
chat-artifact-open-canvas = Open in Canvas
chat-artifact-folder = Folder (optional)
chat-artifact-kind-markdown = Markdown document
chat-artifact-kind-html = HTML page
chat-artifact-kind-code = Code
chat-artifact-words = words

# #2362 — attachment action label
chat-open-in-app = Open in app
# Assistant-look chat: transcript export and answer feedback
chat-export-actions = 대화 작업
chat-export-md = 대화를 Markdown으로 내보내기
chat-export-print = 인쇄 또는 PDF로 저장
chat-export-done = 대화를 내보냈습니다
chat-export-empty = 아직 내보낼 내용이 없습니다
chat-export-untitled = 대화
chat-export-user = 사용자
chat-export-assistant = 어시스턴트
chat-feedback-label = 답변 평가
chat-feedback-good = 좋은 답변
chat-feedback-bad = 나쁜 답변
chat-feedback-comment = 무엇을 개선할까요?
chat-feedback-send = 보내기
chat-staged-files = 첨부 파일
chat-staged-close = 미리보기 닫기
chat-camera-waiting = Waiting for the camera to start…
chat-camera-waiting-frames = Permission granted; waiting for video frames
chat-camera-ready = Camera ready. Take the photo or cancel.
chat-camera-started = Camera started — tap Take photo.
chat-camera-no-device = no video device
chat-camera-device = video device
chat-camera-no-frames = The camera preview did not start. Choose a photo from the system picker instead.
chat-camera-timeout = The camera did not answer. Choose a photo from the system picker instead.

# Chat engineer status strip (#2342)
chat-status-bot = 봇
chat-status-session = 세션
chat-status-model = 모델
chat-status-state = 상태
chat-status-no-session = —
chat-status-copy-session = 세션 ID 복사
chat-status-copied = 복사됨
chat-status-idle = 대기 중
chat-status-streaming = 스트리밍
chat-status-thinking = 생각 중
chat-status-error = 오류
chat-status-stopped = 중지됨

chat-response-ready = 응답 준비 완료
chat-response-started = 어시스턴트가 응답 중입니다

# semantic i18n key (#2542)
chat-no-suggestions-available = 사용 가능한 제안 없음

# semantic i18n key (#2542)
chat-start-conversation = 대화 시작

# semantic i18n key (#2542)
chat-empty-hint = 아래에 메시지를 입력하거나 음성 입력을 사용하세요

# semantic i18n key (#2542)
chat-no-conversations-yet = 아직 대화가 없습니다

# semantic i18n key (#2542)
chat-context-label = 컨텍스트:

# semantic i18n key (#2542)
chat-select-context = 컨텍스트 선택...

# semantic i18n key (#2542)
chat-no-contexts-configured = 구성된 컨텍스트 없음

# semantic i18n key (#2542)
projector-title = 콘텐츠 뷰어

# semantic i18n key (#2542)
projector-loading-content = 콘텐츠 불러오는 중...

# semantic i18n key (#2542)
projector-slide-info = 슬라이드 1 / 1

# semantic i18n key (#2542)
projector-code-info = code.rs

# Projector viewer control (#2542)
chat-projector-rotate = 회전

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
chat-empty-title = 대화를 시작하세요
chat-empty-desc = 아래에 메시지를 입력하거나 봇이 제안하는 항목을 선택하세요.
chat-empty-action = 첫 메시지 작성
