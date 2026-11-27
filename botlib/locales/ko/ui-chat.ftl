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
chat-consent-affected-count = 영향 받는 레코드
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
chat-tool-result-ready = 도구 결과를 사용할 수 있습니다

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

# -----------------------------------------------------------------------------
# #2342 — chat engineer UX: slash commands, help, inspect/copy-markdown
# -----------------------------------------------------------------------------
chat-cmd-menu-title = Commands
chat-cmd-help = Show keyboard shortcuts
chat-cmd-new = Start a new conversation
chat-cmd-clear = Clear this conversation
chat-cmd-stop = Stop generating
chat-stop = 중지
chat-cmd-failed = Command failed
chat-help = Keyboard shortcuts
chat-inspect = Inspect
chat-inspect-title = Message details
chat-inspect-sender = Sender
chat-inspect-id = Message id
chat-inspect-no-id = none
chat-inspect-length = Characters
chat-copy-markdown = Copy as Markdown
chat-copied-markdown = Copied as Markdown

# Model attribution on each bot answer
chat-model-written-by = 답변을 작성한 모델:

# Assistant parity in chat
assistant-parity-block-allow = 허용
assistant-parity-block-allowed = 허용됨
assistant-parity-block-api-command = API 명령
assistant-parity-block-attachment-read = 첨부 파일 읽음
assistant-parity-block-chars = 자
assistant-parity-block-command = 명령
assistant-parity-block-decision-failed = 결정을 기록하지 못했습니다
assistant-parity-block-denied = 거부됨
assistant-parity-block-deny = 거부
assistant-parity-block-detail = 세부 정보
assistant-parity-block-drive-list = 드라이브 목록
assistant-parity-block-drive-read = 드라이브 파일 읽음
assistant-parity-block-drive-search = 드라이브 검색
assistant-parity-block-elapsed = 경과 시간
assistant-parity-block-error = 오류
assistant-parity-block-file = 파일
assistant-parity-block-kb-search = 지식 검색
assistant-parity-block-notice = 알림
assistant-parity-block-path = 경로
assistant-parity-block-permission = 권한 요청
assistant-parity-block-query = 쿼리
assistant-parity-block-read = 읽기
assistant-parity-block-reason = 이유
assistant-parity-block-result = 결과
assistant-parity-block-retry = 다시 시도
assistant-parity-block-seconds = 초
assistant-parity-block-size = 크기
assistant-parity-block-status-done = 완료
assistant-parity-block-status-failed = 실패
assistant-parity-block-status-needs-you = 입력이 필요합니다
assistant-parity-block-status-running = 실행 중
assistant-parity-block-step = 단계
assistant-parity-block-thinking = 생각 중
assistant-parity-block-tool = 도구
assistant-parity-block-tool-run = 도구 실행
assistant-parity-block-total = 합계
assistant-parity-block-truncated = 잘림
assistant-parity-block-type = 유형
assistant-parity-block-yes = 예
chat-parity-attach-title = 첨부 파일 미리보기
chat-parity-attach-unavailable = 내용을 표시할 수 없습니다. 다운로드하여 여세요.
chat-parity-attach-download-failed = 첨부 파일을 다운로드하지 못했습니다.
chat-parity-model-tiers = 자동 및 단계
chat-parity-model-accounts = 공급자 계정
chat-parity-model-local = 로컬 및 무료
chat-drive-parity-attaching = 첨부 중...
chat-drive-parity-cancel = 취소
chat-drive-parity-delete = 삭제
chat-drive-parity-deleted = 삭제됨
chat-drive-parity-delete-failed = 파일을 삭제하지 못했습니다.
chat-drive-parity-delete-question = 이 파일을 삭제할까요?
chat-drive-parity-delete-title = 파일 삭제
chat-drive-parity-deleting = 삭제 중...
chat-drive-parity-download = 다운로드
chat-drive-parity-download-failed = 파일을 다운로드하지 못했습니다.
chat-drive-parity-downloading = 다운로드 중...
chat-drive-parity-more = 추가 작업
chat-drive-parity-new-name = 새 이름
chat-drive-parity-preview = 미리보기
chat-drive-parity-preview-failed = 파일을 미리 볼 수 없습니다.
chat-drive-parity-previewing = 미리보기를 여는 중...
chat-drive-parity-rename = 이름 변경
chat-drive-parity-renamed = 이름 변경됨
chat-drive-parity-rename-failed = 파일 이름을 변경하지 못했습니다.
chat-drive-parity-rename-title = 파일 이름 변경
chat-drive-parity-renaming = 이름 변경 중...
chat-drive-parity-unavailable = 이 배포에서는 이 작업을 사용할 수 없습니다.
chat-drive-parity-use-in-chat = 대화에서 사용
chat-parity-agent-does = 에이전트가 수행합니다
chat-parity-back = 뒤로
chat-parity-checked-now = 방금 확인함
chat-parity-close = 닫기
chat-parity-coming-soon = 곧 제공
chat-parity-cost-contract = 계약에 따름
chat-parity-cost-free = 무료
chat-parity-cost-free-limit = 무료, 사용 제한 있음
chat-parity-cost-sub = 구독 요금제
chat-parity-cuda-no = 적합한 GPU 없음
chat-parity-cuda-yes = 그래픽 카드 감지됨
chat-parity-depends-agent = 에이전트에 따라 다름
chat-parity-driver = 드라이버
chat-parity-engine = 사용 중인 엔진
chat-parity-feature-docs = 문서
chat-parity-feature-ocr = 이미지 속 텍스트
chat-parity-feature-photos = 사진 및 차트
chat-parity-feature-web = 웹 검색
chat-parity-features = 켜진 기능
chat-parity-gains = 이점
chat-parity-go-agent = 에이전트 제공자로 이동
chat-parity-go-ext = 외부 공급자로 이동
chat-parity-go-net = 조직 네트워크에 머무름
chat-parity-go-server = 설정한 서버로 이동
chat-parity-guide = 어떤 것을 선택해야 할지 모르겠나요?
chat-parity-guide-sub = 두 가지 질문에 답하세요.
chat-parity-in-use = 사용 중
chat-parity-load-failed = 정보를 불러오지 못했습니다.
chat-parity-loading = 불러오는 중...
chat-parity-machine = 이 컴퓨터
chat-parity-machine-ok = 이 컴퓨터: %d, %v GB
chat-parity-machine-slow = 적합한 그래픽 카드가 없어 느립니다
chat-parity-need-acp = 에이전트를 설치하고 로그인하세요
chat-parity-need-free = 없음
chat-parity-need-local = 6GB 이상 그래픽 카드와 4~6GB 다운로드
chat-parity-need-own = 서버 주소와 키
chat-parity-no-read = 읽지 않음
chat-parity-no-work = 작동하지 않음
chat-parity-none-found = 없음
chat-parity-not-yet = 아직
chat-parity-only-net = 네트워크 내 서버에서만
chat-parity-optional = 선택 사항, 켜면 사용
chat-parity-pick = 옵션을 선택하세요
chat-parity-priv-ext = 대화가 외부 호스트(%h)로 전송됩니다.
chat-parity-priv-local = 대화가 이 컴퓨터에 머뭅니다.
chat-parity-priv-net = 대화가 조직 서버(%h)로 전송됩니다.
chat-parity-privacy = 개인정보
chat-parity-q1 = 작성 내용이 이 컴퓨터 밖으로 나가도 되나요?
chat-parity-q1-help = 기밀 또는 개인 정보를 생각해 보세요.
chat-parity-q1-no = 아니요, 여기에 있어야 합니다
chat-parity-q1-no-desc = 로컬 처리만 허용됩니다.
chat-parity-q1-yes = 예, 나가도 됩니다
chat-parity-q1-yes-desc = 외부 서비스에서 처리할 수 있습니다.
chat-parity-q2-no = 아니요, 또는 모르겠습니다
chat-parity-q2-no-desc = 회사 서버나 키가 없습니다.
chat-parity-q2-open = 조직이 AI 서버 또는 키를 제공하나요?
chat-parity-q2-open-help = 모르면 IT에 문의하세요.
chat-parity-q2-secret = 조직이 외부 AI를 허용하나요?
chat-parity-q2-secret-help = 일부 정책은 외부 서비스를 금지합니다.
chat-parity-q2-secret-no-desc = 외부 AI가 허용되지 않습니다.
chat-parity-q2-secret-yes-desc = 외부 AI가 허용됩니다.
chat-parity-q2-yes = 예
chat-parity-q2-yes-desc = 회사 서버나 키가 있습니다.
chat-parity-reads = 읽기
chat-parity-ready = 준비됨
chat-parity-rec-free = 무료 모델로 시작하세요. 설정이 필요 없습니다. 기밀 사항은 이 컴퓨터를 선택하세요.
chat-parity-rec-local = 이 컴퓨터를 사용하세요. 외부로 나가지 않고 GPU도 있습니다.
chat-parity-rec-local-slow = 로컬 모델에 적합한 GPU가 없습니다. 조직 네트워크의 서버를 사용하세요.
chat-parity-rec-local-tight = 로컬 모델은 작동하지만 이 GPU에서는 느릴 수 있습니다.
chat-parity-rec-own = 조직의 서버 또는 키를 사용하세요. 비용과 제한은 계약에 따릅니다.
chat-parity-rec-own-net = 데이터를 보호하려면 조직 네트워크의 서버를 사용하세요.
chat-parity-restart = 다시 시작
chat-parity-retry = 다시 시도
chat-parity-row-cost = 비용
chat-parity-row-docs = 문서
chat-parity-row-needs = 필요한 것
chat-parity-row-ocr = 이미지 속 텍스트
chat-parity-row-offline = 인터넷 없이
chat-parity-row-photos = 사진 및 차트
chat-parity-row-privacy = 내 텍스트
chat-parity-row-web = 웹 검색
chat-parity-rule = 규칙
chat-parity-show-me = 활성화 방법 보기
chat-parity-stay-here = 이 컴퓨터에 머뭅니다
chat-parity-step = 질문 %n / 2
chat-parity-sub-acp = 에이전트 구독
chat-parity-sub-free = 바로 사용 가능
chat-parity-sub-local = 개인적이고 오프라인
chat-parity-sub-own = 조직의 것
chat-parity-suggested = 추천
chat-parity-system-memory = 시스템 메모리
chat-parity-verdict-bad-gpu = 적합한 그래픽 카드 없음
chat-parity-verdict-bad-gpu-long = CUDA 지원 NVIDIA GPU가 없습니다. 로컬 모델은 CPU에서 훨씬 느리게 실행되며 답변에 몇 분이 걸릴 수 있습니다. 데이터를 네트워크 안에 두려면 조직 서버를 사용하세요.
chat-parity-verdict-capacity = 이 GPU는 최대 %v의 VRAM이 필요한 모델을 실행합니다.
chat-parity-verdict-good = 여유 있음
chat-parity-verdict-low-vram = VRAM이 부족한 GPU(%v)
chat-parity-verdict-low-vram-long = 이 GPU의 VRAM은 %v이지만 가장 작은 로컬 모델은 %m이 필요합니다.
chat-parity-verdict-mid = 빡빡하지만 사용 가능
chat-parity-verdict-none = 이 컴퓨터를 검사할 수 없습니다
chat-parity-video-memory = VRAM
chat-parity-where = 엔진 선택
chat-parity-where-acp = 에이전트 (ACP)
chat-parity-where-free = 무료 모델
chat-parity-where-local = 이 컴퓨터에서
chat-parity-where-own = 자체 서버 또는 키
chat-parity-where-sub = 어시스턴트의 네 가지 사용 방법을 비교합니다. 클릭하면 세부 정보와 활성화 방법을 표시합니다.
chat-parity-works = 작동합니다
chat-parity-workshop = 어시스턴트 워크숍
chat-parity-workshop-lead = 어시스턴트가 어디서 어떻게 생각하는지 조정합니다.
chat-parity-workshop-title = 어시스턴트 워크숍
chat-parity-graphics-card = 그래픽 카드

# --- #2921 missing keys added to every locale catalogue ---
chat-confirm-title = 작업 확인
chat-confirm-unavailable = 확인을 사용할 수 없습니다
chat-copy = 복사
chat-delete = 삭제
chat-delete-message-confirm = 이 메시지를 대화에서 삭제하시겠습니까?
chat-delete-title = 메시지 삭제
chat-edit = 편집
chat-edit-unavailable = 입력창을 사용할 수 없습니다
chat-message-actions = 메시지 작업
chat-regenerate = 다시 생성
chat-regenerate-no-source = 다시 생성할 이전 사용자 메시지가 없습니다
chat-regenerate-unavailable = 이 턴은 다시 생성할 수 없습니다
chat-voice-capture-silent = 오디오가 캡처되지 않았습니다
chat-voice-duplex-audio = 오디오
chat-voice-duplex-closed = 음성 세션이 종료되었습니다
chat-voice-duplex-connect = 음성 연결
chat-voice-duplex-connected = 음성이 연결되었습니다
chat-voice-duplex-disabled = 음성이 비활성화되었습니다
chat-voice-duplex-error = 음성 오류
chat-voice-duplex-fallback = 텍스트로 전환합니다
chat-voice-duplex-live = 실시간 음성
chat-voice-duplex-lost = 연결이 끊겼습니다
chat-voice-duplex-timeout = 음성 세션 시간이 초과되었습니다
chat-voice-duplex-unavailable = 음성을 사용할 수 없습니다
chat-voice-mic-failed = 마이크를 사용할 수 없습니다
chat-web-title = 웹 검색
voice-mode-connecting = 연결 중...
voice-mode-fallback-idle = 대기 중
voice-mode-fallback-listening = 듣는 중
voice-mode-fallback-speaking = 말하는 중
voice-mode-fallback-thinking = 생각하는 중
voice-mode-live-listening = 듣는 중
voice-mode-live-ready = 준비됨
voice-mode-live-speaking = 말하는 중

# gbgov assistant parity: busy-send and actionable engine refusals (#2563)
chat-busy-send-head = 답변을 작성하는 중입니다
chat-busy-send-text = 답변이 끝날 때까지 기다리거나, 중지하고 지금 메시지를 보내세요.
chat-busy-send-wait = 계속 기다리기
chat-busy-send-stop = 중지하고 보내기
chat-engine-error-not-linked = %label 계정이 연결되지 않았습니다. 다른 모델을 선택하거나 설정에서 계정을 연결하세요.
chat-engine-error-switch-off = 이 대화에서 %label 에이전트 스위치를 켜야 합니다.
chat-engine-error-unavailable = 구독 에이전트는 이 컴퓨터에서 사용할 수 없습니다. 다른 모델을 선택하세요.
chat-engine-error-model-unavailable = %model 모델은 %label 에서 사용할 수 없습니다. 다른 모델을 선택하세요.
chat-engine-error-workspace-failed = 에이전트 가상 머신을 시작할 수 없습니다. 상태를 확인하고 다시 시도하세요.
chat-engine-error-account-model-off = %model 모델이 %label 에 대해 활성화되지 않았습니다. 다른 모델을 선택하세요.
chat-engine-error-free-unavailable = 무료 모델을 현재 사용할 수 없습니다. 나중에 다시 시도하거나 다른 모델을 선택하세요.
chat-engine-error-limit-reached = %label 요금제의 사용 한도에 도달했습니다. %when 다른 모델을 선택하세요.
chat-engine-error-unsafe-version = %label (%version) 버전은 작업 전에 권한을 요청하지 않아 차단되었습니다. 다른 모델을 선택하세요.
chat-engine-error-choose-model = 다른 모델 선택
chat-engine-error-back-at = %when 에 다시 사용할 수 있습니다.

# ── Completed translations (issue #2970) ──
chat-conversation-clear = 컨텍스트 지우기
chat-conversation-cleared = 대화 컨텍스트가 지워졌습니다
chat-conversation-copy-link = 링크 복사
chat-conversation-delete = 삭제
chat-conversation-delete-confirm = 이 대화를 삭제하시겠습니까?
chat-conversation-delete-failed = 대화를 삭제하지 못했습니다
chat-conversation-delete-title = 대화 삭제
chat-conversation-delete-unavailable = 확인 대화상자를 사용할 수 없습니다
chat-conversation-deleted = 대화가 삭제되었습니다
chat-conversation-export-json = 대화 기록 내보내기 (.json)
chat-conversation-export-md = 대화 기록 내보내기 (.md)
chat-conversation-exported = 대화가 내보내졌습니다
chat-conversation-link-copied = 대화 링크가 복사되었습니다
chat-conversation-link-copy-failed = 링크를 복사하지 못했습니다
chat-conversation-menu = 대화 메뉴
chat-conversation-rename = 이름 바꾸기
chat-conversation-rename-failed = 대화 이름을 변경하지 못했습니다
chat-conversation-renamed = 대화 이름이 변경되었습니다
chat-conversation-title = 대화 제목
chat-conversation-untitled = 제목 없는 대화
