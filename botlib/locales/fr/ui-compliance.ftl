# =============================================================================
# General Bots — fr — Compliance
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# TOOLS / COMPLIANCE SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Compliance Main
# -----------------------------------------------------------------------------
compliance-title = Rapport de conformité des API
compliance-subtitle = Analyse de sécurité pour tous les robots - Vérifiez les mots de passe, le code fragile et les erreurs de configuration
compliance-export-report = Exporter le rapport
compliance-run-scan = Exécuter une analyse de conformité
compliance-scanning = Numérisation...

# -----------------------------------------------------------------------------
# Bot Selector
# -----------------------------------------------------------------------------
compliance-all-bots = Tous les robots
compliance-select-bots = Sélectionnez les robots

# -----------------------------------------------------------------------------
# Stats Cards
# -----------------------------------------------------------------------------
compliance-critical = Critique
compliance-critical-desc = Nécessite une action immédiate
compliance-high = Élevé
compliance-high-desc = Risque de sécurité
compliance-medium = Moyen
compliance-medium-desc = Doit être abordé
compliance-low = Faible
compliance-low-desc = Meilleure pratique
compliance-info = Informations
compliance-info-desc = Informatif

# -----------------------------------------------------------------------------
# Filters
# -----------------------------------------------------------------------------
compliance-filter-severity = Gravité
compliance-filter-type = Tapez
compliance-filter-all-severities = Toutes les gravités
compliance-filter-all-types = Tous types
compliance-search-issues = Problèmes de recherche...

# -----------------------------------------------------------------------------
# Issue Types
# -----------------------------------------------------------------------------
compliance-type-password = Mot de passe dans la configuration
compliance-type-hardcoded = Secrets codés en dur
compliance-type-deprecated = Mots clés obsolètes
compliance-type-fragile = Codes fragiles
compliance-type-config = Problèmes de configuration

# -----------------------------------------------------------------------------
# Results Table
# -----------------------------------------------------------------------------
compliance-results = Résultats
compliance-results-count = { $compte ->
    [one] { $count } issue found
   *[other] { $count } issues found
}
compliance-col-severity = Gravité
compliance-col-issue = Problème
compliance-col-location = Emplacement
compliance-col-details = Détails
compliance-col-action = Action
compliance-view-details = Afficher les détails
compliance-fix-issue = Résoudre le problème
compliance-ignore = Ignorer
compliance-no-issues = Aucun problème trouvé
compliance-no-issues-desc = Génial ! Vos robots sont conformes.

# -----------------------------------------------------------------------------
# Scan Progress
# -----------------------------------------------------------------------------
compliance-scan-in-progress = Scan en cours...
compliance-scan-checking = Vérification { $item }...
compliance-scan-complete = Analyse terminée
compliance-scan-failed = Échec de l'analyse

## Compliance frameworks (enterprise-grade release)
compliance-frameworks = Cadres
compliance-new-framework = Nouveau
compliance-framework-name = Nom
compliance-framework-version = Version
compliance-framework-description = Descriptif
compliance-create-framework = Créer un cadre
compliance-controls = Contrôles
compliance-add-control = Ajouter un contrôle
compliance-control-id = ID de contrôle
compliance-control-title = Titre
compliance-control-category = Catégorie
compliance-control-description = Descriptif
compliance-mandatory = Obligatoire
compliance-optional = Facultatif
compliance-evidence = Preuve
compliance-attach-evidence = Joindre des preuves
compliance-evidence-path = Chemin du fichier (artefact de lecteur)
compliance-evidence-type = Tapez
compliance-approve = Approuver
compliance-covered = Couvert
compliance-no-evidence = Aucune preuve
compliance-export-csv = Exporter au format CSV
compliance-archive = Archiver
compliance-total-controls = Contrôles totaux
compliance-coverage = Couverture
compliance-no-frameworks = Aucun framework n'est encore configuré.
