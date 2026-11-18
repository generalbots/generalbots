# =============================================================================
# General Bots — fr — Admin, monitoring and system apps
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Panel (additional UI keys)
# -----------------------------------------------------------------------------
admin-panel-title = Panneau d'administration

# =============================================================================
# MONITORING SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Monitoring Dashboard
# -----------------------------------------------------------------------------
monitoring-title = Tableau de bord de surveillance
monitoring-toggle-view = Basculer la vue
monitoring-last-updated = Dernière mise à jour
monitoring-live-view = Affichage en direct
monitoring-grid-view = Vue Grille

# -----------------------------------------------------------------------------
# Monitoring Panels
# -----------------------------------------------------------------------------
monitoring-sessions = Séances
monitoring-messages = Messages
monitoring-resources = Ressources
monitoring-services = Prestations
monitoring-active-bots = Bots actifs
monitoring-loading = Chargement...

# -----------------------------------------------------------------------------
# Service Status
# -----------------------------------------------------------------------------
monitoring-status-running = Courir
monitoring-status-warning = Avertissement
monitoring-status-stopped = Arrêté
monitoring-status-healthy = Sain
monitoring-status-degraded = Dégradé
monitoring-status-down = Vers le bas

# -----------------------------------------------------------------------------
# Resource Metrics
# -----------------------------------------------------------------------------
monitoring-cpu = Processeur
monitoring-memory = Mémoire
monitoring-disk = Disque
monitoring-network = Réseau
monitoring-requests-per-sec = Requêtes/s
monitoring-active-connections = Connexions actives
monitoring-uptime = Temps de disponibilité

# -----------------------------------------------------------------------------
# Logs
# -----------------------------------------------------------------------------
monitoring-logs-title = Journaux système
monitoring-logs-filter = Filtrer les journaux
monitoring-logs-level = Niveau de journalisation
monitoring-logs-all = Tous les niveaux
monitoring-logs-debug = Débogage
monitoring-logs-info = Informations
monitoring-logs-warning = Avertissement
monitoring-logs-error = Erreur
monitoring-logs-critical = Critique
monitoring-logs-search = Rechercher des journaux...
monitoring-logs-no-results = Aucun journal trouvé

# -----------------------------------------------------------------------------
# Health
# -----------------------------------------------------------------------------
monitoring-health-title = Santé du système
monitoring-health-status = État de santé
monitoring-health-services = Santé des services
monitoring-health-database = Base de données
monitoring-health-cache = Cache
monitoring-health-queue = File d'attente des messages
monitoring-health-storage = Stockage
monitoring-health-external = Services externes

# -----------------------------------------------------------------------------
# Metrics
# -----------------------------------------------------------------------------
monitoring-metrics-title = Mesures de performances
monitoring-metrics-response-time = Temps de réponse
monitoring-metrics-throughput = Débit
monitoring-metrics-error-rate = Taux d'erreur
monitoring-metrics-latency = Latence

# -----------------------------------------------------------------------------
# Alerts
# -----------------------------------------------------------------------------
monitoring-alerts-title = Alertes système
monitoring-alerts-active = Alertes actives
monitoring-alerts-resolved = Résolu
monitoring-alerts-all = Toutes les alertes
monitoring-alert-severity = Gravité
monitoring-alert-critical = Critique
monitoring-alert-high = Élevé
monitoring-alert-medium = Moyen
monitoring-alert-low = Faible
monitoring-alert-info = Informations
monitoring-alert-acknowledge = Reconnaître
monitoring-alert-resolve = Résoudre
monitoring-no-alerts = Aucune alerte active

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
security-tab-compliance = Rapport de conformité des API
security-tab-protection = Protection
security-export-report = Exporter le rapport
security-run-scan = Exécuter une analyse de conformité
security-critical = Critique
security-critical-desc = Action immédiate requise
security-high = Élevé
security-high-desc = Risque de sécurité
security-medium = Moyen
security-medium-desc = Doit être abordé
security-low = Faible
security-low-desc = Meilleure pratique
security-info = Informations
security-info-desc = Informatif
security-filter-severity = Gravité :
security-filter-all-severities = Toutes les gravités
security-filter-type = Tapez :
security-filter-all-types = Tous types
security-type-password = Mot de passe dans la configuration
security-type-hardcoded = Secrets codés en dur
security-type-deprecated = Mots clés obsolètes
security-type-fragile = Codes fragiles
security-type-config = Problèmes de configuration
security-results = Problèmes de conformité
security-col-severity = Gravité
security-col-issue = Type de problème
security-col-location = Emplacement
security-col-details = Descriptif
security-col-action = Action

# -----------------------------------------------------------------------------
# Learn Module
# -----------------------------------------------------------------------------
learn-title = Apprendre
learn-my-progress = Mes progrès
learn-completed = Terminé
learn-in-progress = En cours
learn-certificates = Certificats
learn-time-spent = Temps passé
learn-categories = Catégories
learn-all-courses = Tous les cours
learn-mandatory = Obligatoire
learn-compliance = Conformité
learn-security = Sécurité
learn-skills = Compétences
learn-onboarding = Intégration
learn-difficulty = Difficulté
learn-my-certificates = Mes certificats
learn-view-all = Tout afficher

# -----------------------------------------------------------------------------
# Workspace Module
# -----------------------------------------------------------------------------
workspace-title = Espace de travail
workspace-search-pages = Pages de recherche...
workspace-recent = Récent
workspace-favorites = Favoris
workspace-pages = Pages
workspace-templates = Modèles
workspace-trash = Corbeille
workspace-settings = Paramètres

# -----------------------------------------------------------------------------
# Player Module
# -----------------------------------------------------------------------------
player-title = Lecteur multimédia
player-no-file = Aucun fichier sélectionné
player-search = Rechercher des fichiers...
player-recent = Récent
player-files = Fichiers

# -----------------------------------------------------------------------------
# Goals Module
# -----------------------------------------------------------------------------
goals-title = Objectifs et OKR
goals-dashboard = Tableau de bord
goals-objectives = Objectifs
goals-alignment = Alignement
goals-ai-suggestions = Suggestions d'IA

# semantic i18n key (#2542)
admin-ops-dashboard-title = Tableau de bord des opérations

# semantic i18n key (#2542)
admin-ops-dashboard-subtitle = Métriques système en temps réel, traçage distribué et surveillance des performances

# semantic i18n key (#2542)
admin-ops-auto-refresh-on = Actualisation automatique : ACTIVÉE

# semantic i18n key (#2542)
admin-ops-avg-response = Réponse moyenne

# semantic i18n key (#2542)
admin-ops-requests-min = Requêtes/min

# semantic i18n key (#2542)
admin-ops-disk-io = E/S disque

# semantic i18n key (#2542)
admin-ops-iops-read-write = IOPS (lecture/écriture)

# semantic i18n key (#2542)
admin-ops-in-95-out-50 = Entrée : 95 / Sortie : 50

# semantic i18n key (#2542)
admin-ops-stable = Stable

# semantic i18n key (#2542)
admin-ops-request-metrics = Métriques des requêtes

# semantic i18n key (#2542)
admin-ops-errors-label = Erreurs

# semantic i18n key (#2542)
admin-ops-distributed-tracing = Traçage distribué

# semantic i18n key (#2542)
admin-ops-all-traces = Toutes les traces

# semantic i18n key (#2542)
admin-ops-errors-only = Erreurs uniquement

# semantic i18n key (#2542)
admin-ops-api-requests = Requêtes API

# semantic i18n key (#2542)
admin-ops-post-api-chat-message = POST /api/chat/message

# semantic i18n key (#2542)
admin-ops-trace-a1b2c3d4e5f6 = trace_a1b2c3d4e5f6

# semantic i18n key (#2542)
admin-ops-get-api-kb-search = GET /api/kb/search

# semantic i18n key (#2542)
admin-ops-trace-b2c3d4e5f6g7 = trace_b2c3d4e5f6g7

# semantic i18n key (#2542)
admin-ops-post-api-billing-checkout = POST /api/billing/checkout

# semantic i18n key (#2542)
admin-ops-trace-c3d4e5f6g7h8 = trace_c3d4e5f6g7h8

# semantic i18n key (#2542)
admin-ops-get-api-user-profile = GET /api/user/profile

# semantic i18n key (#2542)
admin-ops-trace-d4e5f6g7h8i9 = trace_d4e5f6g7h8i9

# semantic i18n key (#2542)
admin-ops-api-gateway = Passerelle API

# semantic i18n key (#2542)
admin-ops-primary-2-replicas = Principal + 2 réplicas

# semantic i18n key (#2542)
admin-ops-qdrant-vector-db = Qdrant (base vectorielle)

# semantic i18n key (#2542)
admin-ops-redis-cache = Cache Redis

# semantic i18n key (#2542)
admin-ops-cluster-mode-3-shards = Mode cluster | 3 fragments

# semantic i18n key (#2542)
admin-ops-llm-service = Service LLM

# semantic i18n key (#2542)
admin-ops-openai-gpt-4-high-latency = OpenAI GPT-4 | Latence élevée

# semantic i18n key (#2542)
admin-ops-object-storage = Stockage d'objets

# semantic i18n key (#2542)
admin-ops-s3-compatible = Compatible S3

# semantic i18n key (#2542)
admin-ops-error-tracking = Suivi des erreurs

# semantic i18n key (#2542)
admin-ops-view-all-errors = Voir toutes les erreurs

# semantic i18n key (#2542)
admin-ops-warnings = Avertissements

# semantic i18n key (#2542)
admin-ops-payment-processing-failed-stripe-timeout = Échec du traitement du paiement : délai d'attente de l'API Stripe dépassé

# semantic i18n key (#2542)
admin-ops-billing-checkout-rs-145 = billing/checkout.rs:145

# semantic i18n key (#2542)
admin-ops-rate-limit-approaching-openai = Limite de débit proche pour l'API OpenAI

# semantic i18n key (#2542)
admin-ops-llm-openai-rs-89 = llm/openai.rs:89

# semantic i18n key (#2542)
admin-ops-core-analytics-rs-234 = core/analytics.rs:234

# semantic i18n key (#2542)
admin-ops-cache-miss-rate-above-threshold = Taux d'échec du cache au-dessus du seuil

# semantic i18n key (#2542)
admin-ops-cache-redis-rs-67 = cache/redis.rs:67

# semantic i18n key (#2542)
admin-ops-endpoint-performance = Performance des points de terminaison

# semantic i18n key (#2542)
admin-ops-by-requests = Par requêtes

# semantic i18n key (#2542)
admin-ops-by-latency = Par latence

# semantic i18n key (#2542)
admin-ops-by-error-rate = Par taux d'erreur

# semantic i18n key (#2542)
admin-ops-post = POST

# semantic i18n key (#2542)
admin-ops-api-chat-message = /api/chat/message

# semantic i18n key (#2542)
admin-ops-req-min = req/min

# semantic i18n key (#2542)
admin-ops-errors-metric = erreurs

# semantic i18n key (#2542)
admin-ops-api-kb-search = /api/kb/search
