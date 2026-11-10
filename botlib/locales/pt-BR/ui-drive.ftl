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
