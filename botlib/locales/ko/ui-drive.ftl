# =============================================================================
# General Bots — ko — Drive and files
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

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

# Drive translations
drive-my-drive = 내 드라이브
drive-shared = 나와 공유됨
drive-recent = 최근
drive-starred = 별표 표시됨
drive-trash = 휴지통
drive-loading-storage = 저장공간 로드 중...
drive-storage-used = { $total } 중 { $used } 사용됨
drive-empty-folder = 이 폴더는 비어 있습니다.

# Drive → Use in chat (#2568)
drive-use-in-chat = 대화에서 사용
drive-use-in-chat-preparing = %n 준비 중…
drive-use-in-chat-attached = 대화에 첨부했습니다: %n
drive-use-in-chat-failed = 파일을 대화에 첨부하지 못했습니다.
drive-use-in-chat-timeout = 파일을 첨부하려면 대화를 여세요.
# 공유 수신(PWA 공유 대상)
drive-share-intake-title = 공유 파일을 Drive에 저장
drive-share-intake-intro = 아래 공유 항목을 확인한 후 Drive에 저장하세요. 저장 전에 로그인해야 할 수 있습니다.
drive-share-intake-heading = 공유 항목
drive-share-intake-loading = 공유 항목을 불러오는 중…
drive-share-intake-auth-hint = 저장에 실패하면 General Bots에 로그인한 뒤 다시 시도하세요. 공유 파일은 저장하거나 삭제할 때까지 이 브라우저에 임시로 저장됩니다.
drive-share-intake-save = Drive에 저장
drive-share-intake-open-drive = Drive 열기
drive-share-intake-no-shared = 공유된 파일을 찾을 수 없습니다. Drive를 열어 수동으로 업로드할 수 있습니다.
drive-share-intake-no-files = 공유된 파일이 없습니다.
drive-share-intake-text-shown = 위에 공유된 텍스트/링크가 표시됩니다.
drive-share-intake-unavailable = 사용할 수 없음
drive-share-intake-read-error = 공유 항목을 읽을 수 없습니다.
drive-share-intake-read-failed = 공유 파일을 읽을 수 없습니다. Drive를 열어 수동으로 업로드하세요.
drive-share-intake-sign-in-required = 파일을 Drive에 저장하려면 로그인이 필요합니다.
drive-share-intake-uploading = 업로드 중
drive-share-intake-saved = 파일을 Drive에 저장했습니다.
drive-share-intake-saved-partial = 일부 파일을 저장하지 못했습니다. 다시 시도할 수 있습니다.

# semantic i18n key (#2542)
drive-my-files = 내 파일

# semantic i18n key (#2542)
drive-loading-files = 파일 불러오는 중...

# semantic i18n key (#2542)
drive-drop-hint = 여기에 파일이나 폴더를 끌어다 놓아 업로드하세요
