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
