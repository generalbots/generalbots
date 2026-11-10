# =============================================================================
# General Bots — es — Drive and files
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Drive / Files
# -----------------------------------------------------------------------------
drive-title = Archivos
drive-upload = Subir
drive-new-folder = Nueva Carpeta
drive-empty = Sin archivos aún. ¡Sube algo!
drive-search = Buscar archivos...
drive-sort-name = Nombre
drive-sort-date = Fecha
drive-sort-size = Tamaño
drive-sort-type = Tipo
drive-view-grid = Vista de Cuadrícula
drive-view-list = Vista de Lista
drive-selected = { $count ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
drive-file-size = { $size ->
    [bytes] { $value } B
    [kb] { $value } KB
    [mb] { $value } MB
    [gb] { $value } GB
   *[other] { $value } bytes
}
drive-drop-files = Arrastra archivos aquí para subir

# Drive translations
drive-my-drive = mi unidad
drive-shared = compartido conmigo
drive-recent = Reciente
drive-starred = Destacado
drive-trash = basura
drive-loading-storage = Cargando almacenamiento...
drive-storage-used = { $used } de { $total } usados
drive-empty-folder = Esta carpeta está vacía
