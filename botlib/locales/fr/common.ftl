# ============================================================================
# General Bots - Common Translations (English)
# ============================================================================
# This file contains shared strings used across all GB components.
# Keep message IDs lowercase with hyphens: category-subcategory-descriptor
# ============================================================================

# -----------------------------------------------------------------------------
# Brand
# -----------------------------------------------------------------------------
app-name = Bots généraux
app-tagline = Votre espace de travail de productivité alimenté par l'IA

# -----------------------------------------------------------------------------
# Common Actions
# -----------------------------------------------------------------------------
action-save = Enregistrer
action-cancel = Annuler
action-delete = Supprimer
action-edit = Modifier
action-close = Fermer
action-confirm = Confirmer
action-retry = Réessayer
action-back = Retour
action-next = Suivant
action-submit = Soumettre
action-search = Rechercher
action-refresh = Actualiser
action-copy = Copier
action-paste = Coller
action-undo = Annuler
action-redo = Refaire
action-select = Sélectionnez
action-select-all = Sélectionner tout
action-clear = Effacer
action-reset = Réinitialiser
action-apply = Postuler
action-create = Créer
action-update = Mise à jour
action-remove = Supprimer
action-add = Ajouter
action-upload = Télécharger
action-download = Télécharger
action-export = Exporter
action-import = Importer
action-share = Partager
action-send = Envoyer
action-reply = Répondre
action-forward = En avant
action-archive = Archiver
action-restore = Restaurer
action-duplicate = Dupliquer
action-rename = Renommer
action-move = Déplacer
action-filter = Filtrer
action-sort = Trier
action-view = Voir
action-hide = Masquer
action-show = Afficher
action-expand = Développer
action-collapse = Réduire
action-enable = Activer
action-disable = Désactiver
action-connect = Se connecter
action-disconnect = Déconnecter
action-sync = Synchroniser
action-start = Commencer
action-stop = Arrêter
action-pause = Pause
action-resume = CV
action-continue = Continuer
action-finish = Terminer
action-complete = Terminé
action-approve = Approuver
action-reject = Rejeter
action-accept = Accepter
action-decline = Refuser
action-login = Connectez-vous
action-logout = Se déconnecter
action-signup = S'inscrire
action-forgot-password = Mot de passe oublié

# -----------------------------------------------------------------------------
# Common Labels
# -----------------------------------------------------------------------------
label-loading = Chargement...
label-saving = Sauvegarde...
label-processing = Traitement...
label-searching = Recherche...
label-uploading = Téléchargement...
label-downloading = Téléchargement...
label-no-results = Aucun résultat trouvé
label-no-data = Aucune donnée disponible
label-empty = Vide
label-none = Aucun
label-all = Tout
label-selected = Sélectionné
label-required = Obligatoire
label-optional = Facultatif
label-default = Par défaut
label-custom = Personnalisé
label-new = Nouveau
label-draft = Brouillon
label-pending = En attente
label-active = Actif
label-inactive = Inactif
label-enabled = Activé
label-disabled = Désactivé
label-public = Publique
label-private = Privé
label-shared = Partagé
label-yes = Oui
label-no = Non
label-on = Sur
label-off = Désactivé
label-true = Vrai
label-false = Faux
label-unknown = Inconnu
label-other = Autre
label-more = Plus
label-less = Moins
label-details = Détails
label-summary = Résumé
label-description = Descriptif
label-name = Nom
label-title = Titre
label-type = Tapez
label-status = Statut
label-priority = Priorité
label-date = Date
label-time = Temps
label-size = Taille
label-count = Compter
label-total = Total
label-average = Moyenne
label-minimum = Minimum
label-maximum = Maximale
label-version = Version
label-id = pièce d'identité
label-created = Créé
label-updated = Mis à jour
label-modified = Modifié
label-deleted = Supprimé
label-by = Par
label-from = De
label-to = À
label-at = À
label-in = Dans
label-of = De

# -----------------------------------------------------------------------------
# Status Messages
# -----------------------------------------------------------------------------
status-success = Succès
status-error = Erreur
status-warning = Avertissement
status-info = Informations
status-loading = Chargement
status-complete = Terminé
status-incomplete = Incomplet
status-failed = Échec
status-cancelled = Annulé
status-pending = En attente
status-in-progress = En cours
status-done = Terminé
status-ready = Prêt
status-not-ready = Pas prêt
status-connected = Connecté
status-disconnected = Déconnecté
status-online = En ligne
status-offline = Hors ligne
status-available = Disponible
status-unavailable = Indisponible
status-busy = Occupé
status-away = Absent

# -----------------------------------------------------------------------------
# Confirmation Dialogs
# -----------------------------------------------------------------------------
confirm-delete = Etes-vous sûr de vouloir supprimer ceci ?
confirm-delete-item = Êtes-vous sûr de vouloir supprimer « { $name } » ?
confirm-delete-items = Êtes-vous sûr de vouloir supprimer { $count ->
    [one] this item
   *[other] these { $count } items
}?
confirm-discard-changes = Vous avez des modifications non enregistrées. Êtes-vous sûr de vouloir les supprimer ?
confirm-logout = Êtes-vous sûr de vouloir vous déconnecter ?
confirm-cancel = Êtes-vous sûr de vouloir annuler ?

# -----------------------------------------------------------------------------
# Time and Dates
# -----------------------------------------------------------------------------
time-now = Tout à l' heure
time-seconds-ago = { $compte ->
    [one] { $count } second ago
   *[other] { $count } seconds ago
}
time-minutes-ago = { $compte ->
    [one] { $count } minute ago
   *[other] { $count } minutes ago
}
time-hours-ago = { $compte ->
    [one] { $count } hour ago
   *[other] { $count } hours ago
}
time-days-ago = { $compte ->
    [one] { $count } day ago
   *[other] { $count } days ago
}
time-weeks-ago = { $compte ->
    [one] { $count } week ago
   *[other] { $count } weeks ago
}
time-months-ago = { $compte ->
    [one] { $count } month ago
   *[other] { $count } months ago
}
time-years-ago = { $compte ->
    [one] { $count } year ago
   *[other] { $count } years ago
}
time-in-seconds = { $compte ->
    [one] in { $count } second
   *[other] in { $count } seconds
}
time-in-minutes = { $compte ->
    [one] in { $count } minute
   *[other] in { $count } minutes
}
time-in-hours = { $compte ->
    [one] in { $count } hour
   *[other] in { $count } hours
}
time-in-days = { $compte ->
    [one] in { $count } day
   *[other] in { $count } days
}
time-today = Aujourd'hui
time-yesterday = Hier
time-tomorrow = Demain
time-this-week = Cette semaine
time-last-week = La semaine dernière
time-next-week = La semaine prochaine
time-this-month = Ce mois-ci
time-last-month = Le mois dernier
time-next-month = Le mois prochain
time-this-year = Cette année
time-last-year = L'année dernière
time-next-year = L'année prochaine

# Days of the week
day-sunday = dimanche
day-monday = lundi
day-tuesday = mardi
day-wednesday = mercredi
day-thursday = jeudi
day-friday = vendredi
day-saturday = samedi
day-sun = Soleil
day-mon = lundi
day-tue = mar.
day-wed = mer
day-thu = jeu.
day-fri = vendredi
day-sat = Samedi

# Months
month-january = janvier
month-february = Février
month-march = Mars
month-april = avril
month-may = mai
month-june = juin
month-july = juillet
month-august = août
month-september = septembre
month-october = octobre
month-november = novembre
month-december = décembre
month-jan = janvier
month-feb = Février
month-mar = mars
month-apr = avril
month-may-short = mai
month-jun = juin
month-jul = juillet
month-aug = Août
month-sep = septembre
month-oct = octobre
month-nov = novembre
month-dec = décembre

# -----------------------------------------------------------------------------
# File Sizes
# -----------------------------------------------------------------------------
size-bytes = { $value }B
size-kilobytes = { $value } Ko
size-megabytes = { $value } Mo
size-gigabytes = { $value } Go
size-terabytes = { $value } To

# -----------------------------------------------------------------------------
# Pagination
# -----------------------------------------------------------------------------
pagination-page = Page { $current } de { $total }
pagination-showing = Affichage de { $from } à { $to } sur { $total }
pagination-items-per-page = Articles par page
pagination-first = D'abord
pagination-previous = Précédent
pagination-next = Suivant
pagination-last = Dernier
pagination-go-to-page = Aller à la page

# -----------------------------------------------------------------------------
# Form Validation
# -----------------------------------------------------------------------------
validation-required = Ce champ est obligatoire
validation-required-field = { $field } est requis
validation-email-invalid = Veuillez saisir une adresse e-mail valide
validation-url-invalid = Veuillez saisir une URL valide
validation-number-invalid = Veuillez entrer un numéro valide
validation-date-invalid = Veuillez entrer une date valide
validation-min-length = Doit contenir au moins { $min } caractères
validation-max-length = Ne doit pas contenir plus de { $max } caractères
validation-min-value = Doit être au moins { $min }
validation-max-value = Ne doit pas dépasser { $max }
validation-pattern-mismatch = Format invalide
validation-passwords-mismatch = Les mots de passe ne correspondent pas
validation-file-too-large = Le fichier est trop volumineux. La taille maximale est de { $max }
validation-file-type-invalid = Type de fichier invalide. Types autorisés : { $types }

# -----------------------------------------------------------------------------
# Accessibility
# -----------------------------------------------------------------------------
a11y-skip-to-content = Passer au contenu principal
a11y-loading = Chargement, veuillez patienter
a11y-menu-open = Ouvrir le menu
a11y-menu-close = Fermer le menu
a11y-expand = Développer
a11y-collapse = Réduire
a11y-selected = Sélectionné
a11y-not-selected = Non sélectionné
a11y-required = Champ obligatoire
a11y-error = Erreur
a11y-success = Succès
a11y-warning = Avertissement
a11y-info = Informations
