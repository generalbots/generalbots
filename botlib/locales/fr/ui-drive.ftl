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
