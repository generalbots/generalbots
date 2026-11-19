# =============================================================================
# General Bots — fr — Drive and files
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Drive / Files
# -----------------------------------------------------------------------------
drive-title = Conduire
drive-upload = Télécharger
drive-new-folder = Nouveau dossier
drive-empty = Aucun fichier pour l'instant. Téléchargez quelque chose !
drive-search = Rechercher des fichiers...
drive-sort-name = Nom
drive-sort-date = Date
drive-sort-size = Taille
drive-sort-type = Tapez
drive-view-grid = Vue Grille
drive-view-list = Vue en liste
drive-selected = { $compte ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
drive-file-size = { $taille ->
    [bytes] { $value } B
    [kb] { $value } KB
    [mb] { $value } MB
    [gb] { $value } GB
   *[other] { $value } bytes
}
drive-drop-files = Déposez les fichiers ici pour les télécharger

# Drive translations
drive-my-drive = Mon disque
drive-shared = Partagé avec moi
drive-recent = Récent
drive-starred = Favoris
drive-trash = Corbeille
drive-loading-storage = Chargement du stockage...
drive-storage-used = { $used } sur { $total } utilisés
drive-empty-folder = Ce dossier est vide

# Drive → Use in chat (#2568)
drive-use-in-chat = Utiliser dans la conversation
drive-use-in-chat-preparing = Préparation de %n…
drive-use-in-chat-attached = Fichier joint à la conversation : %n
drive-use-in-chat-failed = Impossible de joindre le fichier à la conversation.
drive-use-in-chat-timeout = Ouvrez la conversation pour joindre le fichier.
# Réception de partage (cible de partage PWA)
drive-share-intake-title = Enregistrer les fichiers partagés dans Drive
drive-share-intake-intro = Vérifiez les éléments partagés ci-dessous, puis enregistrez-les dans votre Drive. Vous devrez peut-être vous connecter avant d’enregistrer.
drive-share-intake-heading = Éléments partagés
drive-share-intake-loading = Chargement des éléments partagés…
drive-share-intake-auth-hint = Si l’enregistrement échoue, connectez-vous à General Bots et réessayez. Les fichiers partagés sont stockés temporairement dans ce navigateur jusqu’à leur enregistrement ou leur suppression.
drive-share-intake-save = Enregistrer dans Drive
drive-share-intake-open-drive = Ouvrir Drive
drive-share-intake-no-shared = Aucun fichier partagé trouvé. Vous pouvez toujours ouvrir Drive et importer manuellement.
drive-share-intake-no-files = Aucun fichier n’a été partagé.
drive-share-intake-text-shown = Texte/lien partagé affiché ci-dessus.
drive-share-intake-unavailable = indisponible
drive-share-intake-read-error = Impossible de lire les éléments partagés.
drive-share-intake-read-failed = Impossible de lire les fichiers partagés. Ouvrez Drive pour importer manuellement.
drive-share-intake-sign-in-required = Connexion requise pour enregistrer des fichiers dans Drive.
drive-share-intake-uploading = Téléversement
drive-share-intake-saved = Fichiers enregistrés dans Drive.
drive-share-intake-saved-partial = Certains fichiers n’ont pas pu être enregistrés. Vous pouvez réessayer.

# semantic i18n key (#2542)
drive-my-files = Mes fichiers

# semantic i18n key (#2542)
drive-loading-files = Chargement des fichiers...

# semantic i18n key (#2542)
drive-drop-hint = Déposez des fichiers ou dossiers ici pour les téléverser

# semantic i18n key (#2542)
drive-search-all-placeholder = Rechercher dans tous les fichiers…
drive-search-all-aria-label = Rechercher tous les fichiers par nom et contenu

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
drive-starred-empty-title = Aucun fichier favori
drive-starred-empty-desc = Mettez un fichier en favori, ou utilisez son menu ⋮, pour l'avoir à portée de clic.
drive-shared-empty-title = Aucun fichier partagé
drive-shared-empty-desc = Les fichiers partagés avec vous apparaissent ici. Partagez le vôtre pour lancer une relecture.
drive-list-error-title = Impossible de charger cette liste
drive-list-error-desc = Drive n'a pas répondu à temps. Vérifiez votre connexion et réessayez.
