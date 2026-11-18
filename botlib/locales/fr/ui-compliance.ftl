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

# semantic i18n key (#2542)
admin-compliance-dashboard-title = Tableau de bord de conformité

# semantic i18n key (#2542)
admin-compliance-subtitle = État de conformité SOC 2 Type II, surveillance des contrôles et preuves d'audit

# semantic i18n key (#2542)
admin-compliance-compliant = Conforme

# semantic i18n key (#2542)
admin-compliance-overall-score = Score global de conformité

# semantic i18n key (#2542)
admin-compliance-controls-passing = Contrôles réussis

# semantic i18n key (#2542)
admin-compliance-evidence-items-due = Éléments de preuve dus

# semantic i18n key (#2542)
admin-compliance-until-next-audit = Jusqu'au prochain audit

# semantic i18n key (#2542)
admin-compliance-next-audit-date = Prévu : 7 mars 2025

# semantic i18n key (#2542)
admin-compliance-trust-service-criteria = Critères de services de confiance

# semantic i18n key (#2542)
admin-compliance-availability = Disponibilité

# semantic i18n key (#2542)
admin-compliance-processing-integrity = Intégrité du traitement

# semantic i18n key (#2542)
admin-compliance-confidentiality = Confidentialité

# semantic i18n key (#2542)
admin-compliance-controls-requiring-attention = Contrôles nécessitant une attention

# semantic i18n key (#2542)
admin-compliance-control-mfa-privileged = Authentification multifacteur pour les comptes privilégiés

# semantic i18n key (#2542)
admin-compliance-due-jan-25-2025 = Échéance : 25 janv. 2025

# semantic i18n key (#2542)
admin-compliance-remediate = Corriger

# semantic i18n key (#2542)
admin-compliance-control-incident-response-testing = Test du plan de réponse aux incidents de sécurité

# semantic i18n key (#2542)
admin-compliance-issue-annual-test-overdue = Test annuel en retard de 15 jours

# semantic i18n key (#2542)
admin-compliance-control-dpa-documentation = Documentation de l'accord de traitement des données

# semantic i18n key (#2542)
admin-compliance-due-feb-1-2025 = Échéance : 1 févr. 2025

# semantic i18n key (#2542)
admin-compliance-control-data-classification-review = Révision de la politique de classification des données

# semantic i18n key (#2542)
admin-compliance-issue-policy-review-pending = Révision de la politique en attente d'approbation

# semantic i18n key (#2542)
admin-compliance-due-feb-15-2025 = Échéance : 15 févr. 2025

# semantic i18n key (#2542)
admin-compliance-control-privacy-notice-updates = Mises à jour de l'avis de confidentialité

# semantic i18n key (#2542)
admin-compliance-issue-privacy-policy-update = La politique de confidentialité doit être mise à jour pour la nouvelle collecte de données

# semantic i18n key (#2542)
admin-compliance-due-feb-28-2025 = Échéance : 28 févr. 2025

# semantic i18n key (#2542)
admin-compliance-evidence-collection = Collecte de preuves

# semantic i18n key (#2542)
admin-compliance-current = À jour

# semantic i18n key (#2542)
admin-compliance-due-soon = Bientôt dû

# semantic i18n key (#2542)
admin-compliance-evidence-pentest-q4 = Rapport de test d'intrusion T4 2024

# semantic i18n key (#2542)
admin-compliance-evidence-cc4-vuln = CC4.1.1 - Évaluation des vulnérabilités

# semantic i18n key (#2542)
admin-compliance-evidence-access-review-jan = Rapport de revue des accès - Janvier 2025

# semantic i18n key (#2542)
admin-compliance-evidence-cc6-access = CC6.2.1 - Contrôle d'accès

# semantic i18n key (#2542)
admin-compliance-due-in-3-days = Échéance dans 3 jours

# semantic i18n key (#2542)
admin-compliance-evidence-backup-restore = Résultats du test de restauration des sauvegardes

# semantic i18n key (#2542)
admin-compliance-evidence-a1-backup = A1.2.3 - Sauvegarde et récupération

# semantic i18n key (#2542)
admin-compliance-due-in-5-days = Échéance dans 5 jours

# semantic i18n key (#2542)
admin-compliance-evidence-security-training = Dossiers de formation à la sensibilisation à la sécurité

# semantic i18n key (#2542)
admin-compliance-evidence-cc1-training = CC1.4.1 - Formation à la sécurité

# semantic i18n key (#2542)
admin-compliance-view-all-evidence = Voir toutes les preuves

# semantic i18n key (#2542)
admin-compliance-access-events = Événements d'accès

# semantic i18n key (#2542)
admin-compliance-configuration-changes = Modifications de configuration

# semantic i18n key (#2542)
admin-compliance-security-events = Événements de sécurité

# semantic i18n key (#2542)
admin-compliance-compliance-events = Événements de conformité

# semantic i18n key (#2542)
admin-compliance-failed-login-attempt = Tentative de connexion échouée

# semantic i18n key (#2542)
admin-compliance-failed-login-details = Utilisateur : admin@company.com | IP : 192.168.1.105 | Raison : Mot de passe invalide

# semantic i18n key (#2542)
admin-compliance-user-logged-in = Utilisateur connecté

# semantic i18n key (#2542)
admin-compliance-user-logged-in-details = Utilisateur : sarah.johnson@company.com | IP : 10.0.0.45
