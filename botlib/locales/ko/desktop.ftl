# General Bots — desktop shell and setup wizard (#2089)

desktop-document-title = General Bots 데스크톱
sidebar-toggle = 사이드바 전환
sidebar-collapsed-chat = 채팅
sidebar-collapsed-terminal = 터미널
sidebar-collapsed-drive = 드라이브
sidebar-collapsed-settings = 설정
sidebar-collapsed-account = 계정
setup-welcome-title = General Bots에 오신 것을 환영합니다
setup-welcome-subtitle = 몇 가지 간단한 단계로 설정을 완료해 봅시다
setup-welcome-quick-subtitle = 두 단계로 시작하기
setup-step-llm = LLM 공급자
setup-step-profile = 사용자 프로필
setup-step-bot = 봇 구성
setup-step-training = 학습
setup-step-ai-model = AI 모델
setup-step-your-name = 이름
setup-choose-provider-title = LLM 공급자 선택
setup-choose-provider-subtitle = 봇에 사용할 AI 모델 공급자를 선택하세요
setup-provider-openai-desc = GPT-4, GPT-3.5 모델
setup-provider-nvidia-desc = NVIDIA NIM 추론
setup-provider-anthropic-desc = Claude 3, Claude 3.5 모델
setup-provider-local-desc = llama.cpp, Ollama 등
setup-profile-title = 어떤 유형이 가장 잘 맞나요?
setup-profile-subtitle = 맞춤 기본값을 위해 프로필 유형을 선택하세요
setup-profile-developer = 개발자
setup-profile-developer-desc = 봇 구축 및 배포, BASIC 스크립트 작성
setup-profile-business = 비즈니스
setup-profile-business-desc = 워크플로 자동화, CRM 관리
setup-profile-support = 지원
setup-profile-support-desc = 고객 서비스, 티켓 관리
setup-bot-title = 봇 구성
setup-bot-subtitle = 봇에 이름, 목적, 선택적으로 템플릿을 지정하세요
setup-bot-name = 봇 이름
setup-bot-name-placeholder = 나의 어시스턴트
setup-bot-purpose = 봇 목적
setup-bot-purpose-placeholder = 봇이 무엇을 해야 하는지 설명하세요...
setup-bot-template = 봇 템플릿
setup-optional = (선택 사항)
setup-template-scratch = — 처음부터 시작 —
setup-name-title = 어떻게 불러 드릴까요?
setup-name-subtitle = 이 이름은 대화에 표시됩니다
setup-name-label = 이름
setup-name-placeholder = 예: Maria
setup-training-title = 봇 학습시키기
setup-training-subtitle = 초기 학습을 위해 .pdf, .xls, .doc 파일을 놓아 두세요. Drive에 접근할 수 있으며 나중에 변경할 수 있습니다.
setup-drop-text = 여기에 파일을 놓거나 클릭하여 선택하세요
setup-drop-hint = .pdf, .xls, .doc — 각 최대 10MB
setup-select-files = 파일 선택
setup-file-status-pending = 대기 중
setup-file-status-uploading = 업로드 중
setup-file-status-done = 완료
setup-file-status-error = 오류
setup-saving = 저장 중...
setup-complete-message = 설정이 완료되었습니다! General Bots에 오신 것을 환영합니다.
setup-failed = 설정 실패: {error}
setup-unknown-error = 알 수 없는 오류
setup-back = 뒤로
setup-next = 다음
setup-finish = 완료
setup-skip = 설정 건너뛰기
agent-all-apps = 모든 앱

# --- Desktop shell panels: versions, accessibility, sidebar, collaboration,
# --- updates, modals and usage (suite/js/*.js) — #2095

versions-component = 구성 요소
versions-version = 버전
versions-status = 상태
versions-action = 작업
versions-up-to-date = 최신
versions-unknown = 알 수 없음
versions-check-update = 업데이트 확인
versions-checking = 확인 중...
versions-checked = 확인됨
versions-error = 오류
versions-update-available = 업데이트: { $version }
versions-could-not-load = 버전 정보를 불러오지 못했습니다.
versions-ensure-backend = 백엔드 서버가 실행 중인지 확인하세요.
versions-retry = 다시 시도

a11y-chat-conversation = 채팅 대화
a11y-chat-messages = 채팅 메시지
a11y-send-message = 메시지 보내기
a11y-type-message = 메시지 입력
a11y-keyboard-shortcuts = 키보드 단축키
a11y-shortcut-focus-chat = 채팅 입력에 포커스
a11y-shortcut-show-help = 이 도움말 표시
a11y-shortcut-escape = 모달 닫기 / 선택 해제
a11y-shortcut-command-palette = 명령 팔레트
a11y-shortcut-privacy = 개인 정보 설정
a11y-shortcut-scroll-history = 채팅 기록 스크롤
a11y-shortcut-navigate = 요소 간 이동
a11y-shortcut-activate = 포커스된 요소 실행
a11y-shortcut-delete-files = 선택한 파일 삭제 (Drive)
a11y-shortcut-select-all = 모든 파일 선택 (Drive)
a11y-shortcut-clipboard = 파일 복사 / 잘라내기 / 붙여넣기
a11y-shortcut-navigate-up = Drive에서 위로 이동
a11y-shortcut-rename = 선택한 파일 이름 바꾸기

sidebar-locked-conversations = 잠긴 대화
sidebar-no-conversations = 아직 대화가 없습니다
sidebar-sign-in-conversations = 로그인하면 대화를 볼 수 있습니다
sidebar-history-unavailable = 기록을 사용할 수 없습니다
sidebar-back-to-conversations = ← 대화로 돌아가기
sidebar-no-locked-conversations = 잠긴 대화가 없습니다
sidebar-locked-conversation = 잠긴 대화
sidebar-sign-in = 로그인
sidebar-add-pin = PIN 추가

collab-collaborators = 공동 작업자 ({ $count })
collab-no-one-else = 다른 사람은 없습니다

update-available-title = 업데이트 사용 가능 — v{ $version }
update-now = 지금 업데이트
update-later = 나중에
update-skip-version = 이 버전 건너뛰기
update-platform = 플랫폼: { $name }
update-asset = 자산: { $name }
update-check-failed = 업데이트 확인 실패: { $error }
update-view-release-notes = 릴리스 노트 보기
update-release-notes-title = 릴리스 노트 — v{ $version }
update-no-release-notes = 릴리스 노트가 없습니다.

modal-alert-title = 경고
modal-input-title = 입력

external-save-to = { $target }에 저장

usage-title = 사용량 및 비용
usage-last-days = 최근 { $days }일
usage-loading = 사용량을 불러오는 중…
usage-per-model = 모델별
usage-per-day = 일별
usage-model = 모델
usage-tokens = 토큰
usage-messages = 메시지
usage-cost = 비용
usage-day = 일
usage-days-7 = 7일
usage-days-30 = 30일
usage-days-90 = 90일

pin-close = 닫기
pin-length-hint = 4~8자리
pin-cancel = 취소
pin-ok = 확인

# --- Collaboration surfaces shared by every suite app: share dialog, activity
# --- trail, conflict banner and version history (suite/js/collab-*.js) — #2095

role-owner = 소유자
role-editor = 편집자
role-commenter = 댓글 작성자
role-viewer = 뷰어

share-request-failed = 요청 실패 ({ $status })
share-warning-title = 공유 링크에 아직 접근할 수 없습니다
share-warning-body = 이 설치 환경은 localhost에서만 접근할 수 있어 여기서 만든 링크는 어디에서도 열리지 않습니다. 먼저 다음 중 하나를 설정하세요: 설정 → 네트워크에서 P2P를 활성화하거나 공유 링크를 자체 도메인으로 지정하세요.
share-open-shared-links = 공유 링크 열기
share-open-network = 네트워크 열기
share-invite-people = 사람 초대
share-anyone-with-link = 링크를 가진 모든 사람
share-create-link = 링크 만들기
share-transfer-ownership = 소유권 이전
share-transfer = 이전
share-new-owner-placeholder = 새 소유자 이메일
share-owner-suffix = · 소유자
share-no-people = 아직 추가된 사람이 없습니다.
share-no-links = 아직 공개 링크가 없습니다.
share-copied = 복사됨
share-revoke = 취소
share-enter-email = 이메일을 입력하세요
share-enter-new-owner = 새 소유자의 이메일을 입력하세요
share-needs-base = 공유 링크를 만들기 전에 P2P 또는 자체 도메인을 설정하세요.

activity-title = 활동
activity-request-failed = 요청 실패 ({ $status })
activity-load-more = 더 불러오기
activity-loading = 불러오는 중…
activity-empty = 아직 활동이 없습니다
activity-action-create = 생성함
activity-action-edit = 편집함
activity-action-comment = 댓글 작성함
activity-action-delete = 삭제함
activity-action-resolve = 다음의 댓글을 해결함
activity-action-reopen = 다음의 댓글을 다시 열음
activity-action-reaction = 반응함
activity-action-share = 공유함
activity-action-restore = 복원함
activity-action-transfer = 이전함

conflict-title = 편집 충돌
conflict-dismiss = 무시
conflict-edited-concurrently = 동시에 편집됨 ({ $op })
conflict-keep-server = 서버 버전 유지
conflict-keep-mine = 내 변경 유지
conflict-not-started = 충돌 감시가 시작되지 않았습니다
conflict-request-failed = 요청 실패 ({ $status })

versions-title = 버전 기록
versions-loading = 불러오는 중…
versions-empty = 아직 버전이 없습니다 — 첫 저장 시 만들어집니다.
versions-version-n = 버전 { $n }
versions-current-tag = (현재)
versions-name = 이름…
versions-restore-failed = 복원 실패: { $error }
versions-rename-failed = 이름 변경 실패: { $error }
versions-restore-dialog-title = 버전 복원
versions-restore-confirm = 이 버전을 복원할까요? 새 현재 버전이 만들어지며 아무것도 잃지 않습니다.
versions-name-dialog-title = 버전 이름 지정
versions-name-prompt = 이 버전의 이름을 지정하세요(예: "v2 — 승인됨"):
versions-request-failed = 요청 실패 ({ $status })

# --- Desktop shell: snap picker, workspace tabs, virtual desktops, context
# --- menu, git panel, browser panel, widgets and data export (suite/js/*.js)

snap-title = "{ $app }" 스냅
snap-layout-top-left = 왼쪽 위
snap-layout-top = 위쪽 절반
snap-layout-top-right = 오른쪽 위
snap-layout-left = 왼쪽 절반
snap-layout-full = 최대화
snap-layout-right = 오른쪽 절반
snap-layout-bottom-left = 왼쪽 아래
snap-layout-bottom = 아래쪽 절반
snap-layout-bottom-right = 오른쪽 아래
snap-layout-third-left = 왼쪽 1/3
snap-layout-two-thirds = 2/3
snap-layout-third-right = 오른쪽 1/3

wstabs-close-tab = 탭 닫기
wstabs-new-tab = 새 워크스페이스 탭
wstabs-default-name = 워크스페이스 { $n }
wstabs-default-main = 기본
wstabs-rename-title = 워크스페이스 이름 변경
wstabs-name-prompt = 워크스페이스 이름:

vdesk-default-name = 데스크톱 { $n }
vdesk-title = 가상 데스크톱
vdesk-new = 새 데스크톱
vdesk-rename-title = 데스크톱 이름 변경
vdesk-name-prompt = 데스크톱 이름:
vdesk-restore-title = 세션 복원
vdesk-restore-body = 이 데스크톱에서 마지막 세션의 창 { $count }개를 다시 열까요?
vdesk-restore-body-short = 마지막 세션을 복원할까요(창 { $count }개)?

ctxmenu-search = DuckDuckGo에서 검색
ctxmenu-search-prompt = DuckDuckGo에서 검색:
ctxmenu-wikipedia = Wikipedia 정의
ctxmenu-wikipedia-prompt = Wikipedia 정의:

git-loading-status = 상태를 불러오는 중...
git-no-changes = 변경 사항 없음
git-new-branch-prompt = 새 브랜치 이름:
git-loading-diff = diff를 불러오는 중...
git-diff-failed = diff를 불러오지 못했습니다
git-commit-message-required = 커밋 메시지를 입력하세요
git-pushing = 푸시 중...
git-committed = 커밋 및 푸시 완료
git-failed = Git 작업 실패

browser-session-active = 브라우저 세션 { $id } 활성.
browser-init-failed = 브라우저를 초기화하지 못했습니다
browser-init-first = 먼저 브라우저를 초기화하세요
browser-capture = 캡처
browser-no-session = 활성 세션이 없습니다
browser-export-failed = 내보내기 실패
browser-no-actions = 기록된 작업이 없습니다

widget-load-failed = 위젯을 불러오지 못했습니다
widget-no-source = 위젯 소스가 구성되지 않았습니다
widget-open-windowed = 창으로 열기
widget-pin-launcher = 런처에 고정

dataexport-title = 데이터 내보내기 및 백업
dataexport-desc = 채팅 기록, 메모리, Drive 메타데이터, 설정을 포함한 모든 데이터를 JSON 백업 파일로 내보냅니다.
dataexport-export-all = 모든 데이터 내보내기
dataexport-import = 백업 가져오기
dataexport-exported = 백업을 내보냈습니다
dataexport-too-large = 백업이 거부되었습니다: 파일이 4MB보다 큽니다
dataexport-invalid = 잘못된 백업 파일: { $error }
dataexport-nothing = 이 백업에는 복원할 수 있는 설정이 없습니다
dataexport-unknown-date = 알 수 없는 날짜
dataexport-restore-confirm = { $date } 백업에서 설정 { $count }개를 복원합니다. 자격 증명과 세션 토큰은 절대 가져오지 않습니다. 계속할까요?
dataexport-restored = 백업을 복원했습니다({ $count }개 항목).
dataexport-rejected = 허용 목록에 의해 { $count }개 항목이 거부되었습니다.
dataexport-reloading = 다시 불러오는 중...

comments-title = 댓글
comments-request-failed = 요청 실패 ({ $status })
comments-loading = 댓글을 불러오는 중…
comments-load-failed = 댓글을 불러오지 못했습니다: { $error }
comments-empty-open = 열린 댓글이 없습니다. @name으로 멘션할 수 있습니다.
comments-empty-resolved = 해결된 댓글이 없습니다. @name으로 멘션할 수 있습니다.
comments-resolved-chip = 해결됨
comments-resolve = 해결
comments-reopen = 다시 열기
comments-resolved-announce = 댓글을 해결했습니다
comments-reopened-announce = 댓글을 다시 열었습니다
comments-resolve-failed = 해결 실패: { $error }
comments-added = 댓글을 추가했습니다
comments-post = 게시
comments-post-failed = 댓글 실패: { $error }
comments-reaction-failed = 반응 실패: { $error }
comments-delete-failed = 삭제 실패: { $error }
comments-reply-failed = 답글 실패: { $error }
comments-delete-title = 댓글 삭제
comments-delete-confirm = 이 댓글을 삭제할까요?
comments-reply-title = 답글
comments-reply-prompt = 댓글에 답글:
comments-filter-open = 열림
comments-filter-all = 전체
comments-filter-resolved = 해결됨
comments-input-placeholder = 댓글 추가… @name으로 멘션
comments-someone = 누군가
comments-mention-announce = { $author } 님이 댓글에서 회원님을 언급했습니다
