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

# Drive → Use in chat (#2568)
drive-use-in-chat = Usar en el chat
drive-use-in-chat-preparing = Preparando %n…
drive-use-in-chat-attached = Archivo adjuntado a la conversación: %n
drive-use-in-chat-failed = No se pudo adjuntar el archivo a la conversación.
drive-use-in-chat-timeout = Abra el chat para adjuntar el archivo.
# Recepción de elementos compartidos (destino de compartir PWA)
drive-share-intake-title = Guardar archivos compartidos en Drive
drive-share-intake-intro = Revisa los elementos compartidos a continuación y guárdalos en tu Drive. Es posible que debas iniciar sesión antes de guardar.
drive-share-intake-heading = Elementos compartidos
drive-share-intake-loading = Cargando elementos compartidos…
drive-share-intake-auth-hint = Si falla el guardado, inicia sesión en General Bots e inténtalo de nuevo. Los archivos compartidos se almacenan temporalmente en este navegador hasta que se guarden o se eliminen.
drive-share-intake-save = Guardar en Drive
drive-share-intake-open-drive = Abrir Drive
drive-share-intake-no-shared = No se encontraron archivos compartidos. Aún puedes abrir Drive y subirlos manualmente.
drive-share-intake-no-files = No se compartieron archivos.
drive-share-intake-text-shown = Texto/enlace compartido mostrado arriba.
drive-share-intake-unavailable = no disponible
drive-share-intake-read-error = No se pudieron leer los elementos compartidos.
drive-share-intake-read-failed = No se pudieron leer los archivos compartidos. Abre Drive para subirlos manualmente.
drive-share-intake-sign-in-required = Debes iniciar sesión para guardar archivos en Drive.
drive-share-intake-uploading = Subiendo
drive-share-intake-saved = Archivos guardados en Drive.
drive-share-intake-saved-partial = Algunos archivos no se pudieron guardar. Puedes volver a intentarlo.

# semantic i18n key (#2542)
drive-my-files = Mis archivos

# semantic i18n key (#2542)
drive-loading-files = Cargando archivos...

# semantic i18n key (#2542)
drive-drop-hint = Suelta archivos o carpetas aquí para subirlos

# semantic i18n key (#2542)
drive-search-all-placeholder = Buscar en todos los archivos…
drive-search-all-aria-label = Buscar en todos los archivos por nombre y contenido

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
drive-starred-empty-title = Ningún archivo destacado
drive-starred-empty-desc = Destaca cualquier archivo, o usa su menú ⋮, para tenerlo a un clic.
drive-shared-empty-title = Ningún archivo compartido
drive-shared-empty-desc = Los archivos compartidos contigo aparecen aquí. Comparte uno tuyo para iniciar una revisión.
drive-list-error-title = No se pudo cargar esta lista
drive-list-error-desc = Drive no respondió a tiempo. Comprueba tu conexión e inténtalo de nuevo.
