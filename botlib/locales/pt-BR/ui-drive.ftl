# =============================================================================
# General Bots — pt-BR — Drive and files
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Arquivos / Drive
# -----------------------------------------------------------------------------
drive-title = Arquivos
drive-upload = Enviar
drive-new-folder = Nova Pasta
drive-empty = Nenhum arquivo ainda. Envie algo!
drive-search = Buscar arquivos...
drive-sort-name = Nome
drive-sort-date = Data
drive-sort-size = Tamanho
drive-sort-type = Tipo
drive-view-grid = Visualização em Grade
drive-view-list = Visualização em Lista
drive-selected = { $count ->
    [one] { $count } item selecionado
   *[other] { $count } itens selecionados
}
drive-file-size = { $size ->
    [bytes] { $value } B
    [kb] { $value } KB
    [mb] { $value } MB
    [gb] { $value } GB
   *[other] { $value } bytes
}
drive-drop-files = Solte arquivos aqui para enviar

# Traduções do Drive
drive-my-drive = Meu Drive
drive-shared = Compartilhado comigo
drive-recent = Recentes
drive-starred = Com estrela
drive-trash = Lixeira
drive-loading-storage = Carregando armazenamento...
drive-storage-used = { $used } de { $total } usado
drive-empty-folder = Esta pasta está vazia

# Drive → Use in chat (#2568)
drive-use-in-chat = Usar na conversa
drive-use-in-chat-preparing = Preparando %n…
drive-use-in-chat-attached = Arquivo anexado à conversa: %n
drive-use-in-chat-failed = Não foi possível anexar o arquivo à conversa.
drive-use-in-chat-timeout = Abra a conversa para anexar o arquivo.
# Entrada de compartilhamento (alvo de compartilhamento PWA)
drive-share-intake-title = Salvar arquivos compartilhados no Drive
drive-share-intake-intro = Revise os itens compartilhados abaixo e salve-os no seu Drive. Talvez seja necessário entrar antes de salvar.
drive-share-intake-heading = Itens compartilhados
drive-share-intake-loading = Carregando itens compartilhados…
drive-share-intake-auth-hint = Se falhar ao salvar, entre no General Bots e tente novamente. Os arquivos compartilhados ficam armazenados temporariamente neste navegador até serem salvos ou removidos.
drive-share-intake-save = Salvar no Drive
drive-share-intake-open-drive = Abrir Drive
drive-share-intake-no-shared = Nenhum arquivo compartilhado foi encontrado. Você ainda pode abrir o Drive e enviar manualmente.
drive-share-intake-no-files = Nenhum arquivo foi compartilhado.
drive-share-intake-text-shown = Texto/link compartilhado mostrado acima.
drive-share-intake-unavailable = indisponível
drive-share-intake-read-error = Não foi possível ler os itens compartilhados.
drive-share-intake-read-failed = Não foi possível ler os arquivos compartilhados. Abra o Drive para enviar manualmente.
drive-share-intake-sign-in-required = É necessário entrar para salvar arquivos no Drive.
drive-share-intake-uploading = Enviando
drive-share-intake-saved = Arquivos salvos no Drive.
drive-share-intake-saved-partial = Alguns arquivos não puderam ser salvos. Você pode tentar novamente.

# semantic i18n key (#2542)
drive-my-files = Meus arquivos

# semantic i18n key (#2542)
drive-loading-files = Carregando arquivos...

# semantic i18n key (#2542)
drive-drop-hint = Solte arquivos ou pastas aqui para enviar

# semantic i18n key (#2542)
drive-search-all-placeholder = Pesquisar em todos os arquivos…
drive-search-all-aria-label = Pesquisar todos os arquivos por nome e conteúdo

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
drive-starred-empty-title = Nenhum arquivo favorito
drive-starred-empty-desc = Marque qualquer arquivo, ou use o menu ⋮, para mantê-lo a um clique.
drive-shared-empty-title = Nenhum arquivo compartilhado
drive-shared-empty-desc = Os arquivos compartilhados com você aparecem aqui. Compartilhe um seu para iniciar uma revisão.
drive-list-error-title = Não foi possível carregar esta lista
drive-list-error-desc = O Drive não respondeu a tempo. Verifique sua conexão e tente novamente.
