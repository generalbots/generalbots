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
goals-enter-objective-title = Saisissez le titre de l'objectif :
goals-edit-objective-title = Modifier le titre de l'objectif :
goals-delete-objective-confirm = Supprimer cet objectif ? Cette action est irréversible.
goals-enter-key-result-title = Saisissez le titre du résultat clé :
goals-enter-target-value = Saisissez la valeur cible :

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

# semantic i18n key (#2542)
admin-roles-title = Gestion des rôles

# semantic i18n key (#2542)
admin-roles-subtitle = Configurez les rôles et attribuez des autorisations pour contrôler l'accès dans toute la suite

# semantic i18n key (#2542)
admin-roles-sidebar-title = Rôles

# semantic i18n key (#2542)
admin-roles-system-filter = Rôles système

# semantic i18n key (#2542)
admin-roles-custom-filter = Rôles personnalisés

# semantic i18n key (#2542)
admin-roles-select-title = Sélectionnez un rôle

# semantic i18n key (#2542)
admin-roles-select-hint = Choisissez un rôle dans la liste pour afficher et gérer ses autorisations

# semantic i18n key (#2542)
admin-roles-name-label = Nom du rôle

# semantic i18n key (#2542)
admin-roles-name-badge-placeholder = role_name

# semantic i18n key (#2542)
admin-roles-description-placeholder = La description du rôle s'affiche ici

# semantic i18n key (#2542)
admin-roles-permissions-label = Autorisations

# semantic i18n key (#2542)
admin-roles-assigned-users-tab = Utilisateurs attribués

# semantic i18n key (#2542)
admin-roles-assigned-groups-tab = Groupes attribués

# semantic i18n key (#2542)
admin-roles-permission-assignment = Attribution des autorisations

# semantic i18n key (#2542)
admin-roles-permission-assignment-help = Utilisez la double liste pour attribuer des autorisations à ce rôle

# semantic i18n key (#2542)
admin-roles-available-permissions = Autorisations disponibles

# semantic i18n key (#2542)
admin-roles-category-documents = Documents

# semantic i18n key (#2542)
admin-roles-category-spreadsheets = Feuilles de calcul

# semantic i18n key (#2542)
admin-roles-category-presentations = Présentations

# semantic i18n key (#2542)
admin-roles-category-meetings = Réunions

# semantic i18n key (#2542)
admin-roles-category-ai-bots = IA et bots

# semantic i18n key (#2542)
admin-roles-category-automation = Automatisation

# semantic i18n key (#2542)
admin-roles-assigned-permissions = Autorisations attribuées

# semantic i18n key (#2542)
admin-roles-no-permissions-assigned = Aucune autorisation attribuée

# semantic i18n key (#2542)
admin-roles-reset-to-default = Réinitialiser par défaut

# semantic i18n key (#2542)
admin-roles-users-with-role = Utilisateurs avec ce rôle

# semantic i18n key (#2542)
admin-roles-groups-with-role = Groupes avec ce rôle

# semantic i18n key (#2542)
admin-roles-change-history = Historique des modifications de rôle

# semantic i18n key (#2542)
admin-roles-create-new-role = Créer un nouveau rôle

# semantic i18n key (#2542)
admin-roles-name-help = Minuscules, sans espaces (utilisez des traits de soulignement)

# semantic i18n key (#2542)
admin-roles-copy-permissions-from = Copier les autorisations depuis

# semantic i18n key (#2542)
admin-roles-start-with-no-permissions = Commencer sans autorisations

# semantic i18n key (#2542)
admin-roles-standard-user = Utilisateur standard

# semantic i18n key (#2542)
admin-roles-power-user = Utilisateur avancé

# semantic i18n key (#2542)
admin-roles-viewer = Lecteur

# semantic i18n key (#2542)
admin-roles-guest-user = Utilisateur invité

# semantic i18n key (#2542)
admin-roles-create-role-btn = Créer un rôle

# semantic i18n key (#2542)
admin-roles-delete-role-title = Supprimer le rôle

# semantic i18n key (#2542)
admin-roles-delete-warning = Cela supprimera le rôle de tous les utilisateurs et groupes. Cette action est irréversible.

# semantic i18n key (#2542)
admin-roles-assign-to-users = Attribuer le rôle aux utilisateurs

# semantic i18n key (#2542)
admin-roles-search-users-label = Rechercher des utilisateurs

# semantic i18n key (#2542)
admin-roles-type-to-search-users = Tapez pour rechercher des utilisateurs

# semantic i18n key (#2542)
admin-roles-expiration-optional = Expiration (facultatif)

# semantic i18n key (#2542)
admin-roles-expiration-help = Laissez vide pour une attribution permanente

# semantic i18n key (#2542)
admin-roles-assign-to-groups = Attribuer le rôle aux groupes

# semantic i18n key (#2542)
admin-roles-search-groups-label = Rechercher des groupes

# semantic i18n key (#2542)
admin-roles-type-to-search-groups = Tapez pour rechercher des groupes

# semantic i18n key (#2542)
admin-roles-assign-role-btn = Attribuer le rôle

# semantic i18n key (#2542)
admin-onboarding-step-welcome = Bienvenue

# semantic i18n key (#2542)
admin-onboarding-step-welcome-desc = Parlez-nous de vous

# semantic i18n key (#2542)
admin-onboarding-step-org-desc = Créez votre espace de travail

# semantic i18n key (#2542)
admin-onboarding-step-plan = Choisir un forfait

# semantic i18n key (#2542)
admin-onboarding-step-plan-desc = Sélectionnez votre abonnement

# semantic i18n key (#2542)
admin-onboarding-step-payment-desc = Ajouter un moyen de paiement

# semantic i18n key (#2542)
admin-onboarding-step-start = Commencer

# semantic i18n key (#2542)
admin-onboarding-step-start-desc = Créez votre premier bot

# semantic i18n key (#2542)
admin-onboarding-view-guide = Voir le guide

# semantic i18n key (#2542)
admin-onboarding-welcome-title = Bienvenue sur General Bots !

# semantic i18n key (#2542)
admin-onboarding-welcome-subtitle = Configurons tout en quelques minutes. D'abord, parlez-nous un peu de vous.

# semantic i18n key (#2542)
admin-onboarding-full-name = Nom complet

# semantic i18n key (#2542)
admin-onboarding-your-role = Votre rôle

# semantic i18n key (#2542)
admin-onboarding-role-placeholder = Sélectionnez votre rôle...

# semantic i18n key (#2542)
admin-onboarding-role-developer = Développeur / Ingénieur

# semantic i18n key (#2542)
admin-onboarding-role-product-manager = Chef de produit

# semantic i18n key (#2542)
admin-onboarding-role-customer-support = Support client

# semantic i18n key (#2542)
admin-onboarding-role-executive = Dirigeant / Fondateur

# semantic i18n key (#2542)
admin-onboarding-use-case-label = Que souhaitez-vous créer ?

# semantic i18n key (#2542)
admin-onboarding-use-case-placeholder = Sélectionnez le cas d'utilisation principal...

# semantic i18n key (#2542)
admin-onboarding-usecase-support-bot = Bot de support client

# semantic i18n key (#2542)
admin-onboarding-usecase-sales-assistant = Assistant commercial

# semantic i18n key (#2542)
admin-onboarding-usecase-internal-kb = Base de connaissances interne

# semantic i18n key (#2542)
admin-onboarding-usecase-lead-generation = Génération de leads

# semantic i18n key (#2542)
admin-onboarding-usecase-employee-onboarding = Intégration des employés

# semantic i18n key (#2542)
admin-onboarding-usecase-ecommerce = Assistant e-commerce

# semantic i18n key (#2542)
admin-onboarding-usecase-other = Autre chose

# semantic i18n key (#2542)
admin-onboarding-hear-about-label = Comment avez-vous entendu parler de nous ?

# semantic i18n key (#2542)
admin-onboarding-source-search = Moteur de recherche

# semantic i18n key (#2542)
admin-onboarding-source-social = Réseaux sociaux

# semantic i18n key (#2542)
admin-onboarding-source-friend = Ami / Collègue

# semantic i18n key (#2542)
admin-onboarding-create-org-title = Créez votre organisation

# semantic i18n key (#2542)
admin-onboarding-create-org-subtitle = Votre organisation est l'endroit où vous gérerez les bots, les membres de l'équipe et les bases de connaissances.

# semantic i18n key (#2542)
admin-onboarding-org-name = Nom de l'organisation

# semantic i18n key (#2542)
admin-onboarding-org-name-hint = Cela sera visible par les membres de votre équipe

# semantic i18n key (#2542)
admin-onboarding-org-url = URL de l'organisation

# semantic i18n key (#2542)
admin-onboarding-org-url-prefix = app.generalbots.com/

# semantic i18n key (#2542)
admin-onboarding-org-url-hint = Uniquement des lettres minuscules, des chiffres et des traits d'union

# semantic i18n key (#2542)
admin-onboarding-team-size = Taille de l'équipe

# semantic i18n key (#2542)
admin-onboarding-team-size-placeholder = Combien de personnes utiliseront ceci ?

# semantic i18n key (#2542)
admin-onboarding-team-just-me = Juste moi

# semantic i18n key (#2542)
admin-onboarding-industry-placeholder = Sélectionnez votre secteur (facultatif)

# semantic i18n key (#2542)
admin-onboarding-industry-technology = Technologie

# semantic i18n key (#2542)
admin-onboarding-industry-healthcare = Santé

# semantic i18n key (#2542)
admin-onboarding-industry-finance = Finance / Banque

# semantic i18n key (#2542)
admin-onboarding-industry-retail = Commerce de détail / E-commerce

# semantic i18n key (#2542)
admin-onboarding-industry-education = Éducation

# semantic i18n key (#2542)
admin-onboarding-industry-manufacturing = Fabrication

# semantic i18n key (#2542)
admin-onboarding-industry-consulting = Conseil / Services

# semantic i18n key (#2542)
admin-onboarding-industry-government = Gouvernement

# semantic i18n key (#2542)
admin-onboarding-industry-nonprofit = Association à but non lucratif

# semantic i18n key (#2542)
admin-onboarding-choose-plan-title = Choisissez votre forfait

# semantic i18n key (#2542)
admin-onboarding-choose-plan-subtitle = Sélectionnez le forfait qui correspond le mieux à vos besoins. Vous pouvez changer de forfait à tout moment.

# semantic i18n key (#2542)
admin-onboarding-plan-free = Gratuit

# semantic i18n key (#2542)
admin-onboarding-plan-free-desc = Pour les particuliers et les petits projets

# semantic i18n key (#2542)
admin-onboarding-per-month = /mois

# semantic i18n key (#2542)
admin-onboarding-select-free = Sélectionner Gratuit

# semantic i18n key (#2542)
admin-onboarding-most-popular = Le plus populaire

# semantic i18n key (#2542)
admin-onboarding-plan-pro-desc = Pour les équipes en croissance

# semantic i18n key (#2542)
admin-onboarding-select-pro = Sélectionner Pro

# semantic i18n key (#2542)
admin-dashboard-organization-title = Tableau de bord de l'organisation

# semantic i18n key (#2542)
admin-dashboard-organization-subtitle = Vue d'ensemble de l'activité, des membres et de l'état du système de votre organisation

# semantic i18n key (#2542)
admin-dashboard-team-members = Membres de l'équipe

# semantic i18n key (#2542)
admin-dashboard-system-uptime = Disponibilité du système

# semantic i18n key (#2542)
admin-dashboard-api-server = Serveur API

# semantic i18n key (#2542)
admin-dashboard-operational = Opérationnel

# semantic i18n key (#2542)
admin-dashboard-latency-45ms = Latence : 45 ms

# semantic i18n key (#2542)
admin-dashboard-requests-1-2k-min = Requêtes : 1,2 K/min

# semantic i18n key (#2542)
admin-dashboard-connections-45-100 = Connexions : 45/100

# semantic i18n key (#2542)
admin-dashboard-query-time-12ms = Temps de requête : 12 ms

# semantic i18n key (#2542)
admin-dashboard-vector-database = Base de données vectorielle

# semantic i18n key (#2542)
admin-dashboard-vectors-2-4m = Vecteurs : 2,4 M

# semantic i18n key (#2542)
admin-dashboard-search-8ms = Recherche : 8 ms

# semantic i18n key (#2542)
admin-dashboard-tokens-125k-min = Jetons : 125 K/min

# semantic i18n key (#2542)
admin-dashboard-latency-850ms = Latence : 850 ms

# semantic i18n key (#2542)
admin-dashboard-high-usage = Utilisation élevée

# semantic i18n key (#2542)
admin-dashboard-used-34-5gb = Utilisé : 34,5 Go

# semantic i18n key (#2542)
admin-dashboard-iops-2-1k = IOPS : 2,1 K

# semantic i18n key (#2542)
admin-dashboard-members-filter = Membres

# semantic i18n key (#2542)
admin-dashboard-customer-support-bot = Bot de support client

# semantic i18n key (#2542)
admin-dashboard-security-alert = Alerte de sécurité :

# semantic i18n key (#2542)
admin-dashboard-customer-feedback-survey = Enquête de satisfaction client

# semantic i18n key (#2542)
admin-dashboard-sales-assistant = Assistant commercial

# semantic i18n key (#2542)
admin-dashboard-load-more-activity = Charger plus d'activité

# semantic i18n key (#2542)
admin-dashboard-john-smith = John Smith

# semantic i18n key (#2542)
admin-dashboard-role-distribution = Répartition des rôles

# semantic i18n key (#2542)
admin-dashboard-manage-roles = Gérer les rôles

# semantic i18n key (#2542)
admin-dashboard-bot-status = État des bots

# semantic i18n key (#2542)
admin-dashboard-manage-bots = Gérer les bots

# semantic i18n key (#2542)
admin-dashboard-handles-customer-inquiries = Gère les demandes des clients

# semantic i18n key (#2542)
admin-dashboard-product-recommendations = Recommandations de produits

# semantic i18n key (#2542)
admin-dashboard-hr-assistant = Assistant RH

# semantic i18n key (#2542)
admin-dashboard-employee-questions = Questions des employés

# semantic i18n key (#2542)
admin-dashboard-it-helpdesk = Support informatique

# semantic i18n key (#2542)
admin-dashboard-technical-support = Support technique

# semantic i18n key (#2542)
admin-dashboard-pending-invitations = Invitations en attente

# semantic i18n key (#2542)
admin-dashboard-loading-invitations = Chargement des invitations…

# semantic i18n key (#2542)
admin-dashboard-invite-member = Inviter un membre

# semantic i18n key (#2542)
admin-dashboard-bulk-invite = Invitation en masse

# semantic i18n key (#2542)
admin-dashboard-emails-label = E-mails (un par ligne, séparés par une virgule ou un point-virgule)

# semantic i18n key (#2542)
admin-dashboard-manage-users = Gérer les utilisateurs

# semantic i18n key (#2542)
admin-dashboard-roles-permissions = Rôles et autorisations

# semantic i18n key (#2542)
workspace-placeholder-untitled = Sans titre

# semantic i18n key (#2542)
admin-dashboard-placeholder-invite-email = collegue@exemple.com
admin-dashboard-placeholder-bulk-emails = a@exemple.com
b@exemple.com

# semantic i18n key (#2542)
admin-roles-placeholder-search = Rechercher des rôles...
admin-roles-placeholder-filter-available = Filtrer les disponibles...
admin-roles-placeholder-filter-assigned = Filtrer les attribués...
admin-roles-placeholder-name-example = p. ex., content_editor
admin-roles-placeholder-display-name-example = p. ex., Éditeur de contenu
admin-roles-placeholder-description = Décrivez à quoi sert ce rôle...
admin-roles-placeholder-search-users = Tapez pour rechercher des utilisateurs...
admin-roles-placeholder-search-groups = Tapez pour rechercher des groupes...

# semantic i18n key (#2542)
admin-onboarding-placeholder-full-name = Jean Dupont
admin-onboarding-placeholder-org-name = Entreprise Exemple
admin-onboarding-placeholder-org-slug = exemple
admin-onboarding-team-2-5 = 2 à 5 personnes
admin-onboarding-team-6-20 = 6 à 20 personnes
admin-onboarding-team-21-50 = 21 à 50 personnes
admin-onboarding-team-51-plus = Plus de 50 personnes
# semantic i18n key (#2542)
admin-ops-configure-alerts = Configurer les alertes
admin-ops-trace-search-placeholder = Rechercher l’ID de trace...
admin-ops-traces-slow = Lent (>500 ms)

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
workspace-state-empty-recent-title = Rien d'ouvert pour le moment
workspace-state-empty-recent-desc = Les pages ouvertes apparaissent ici, pour reprendre où vous en étiez.
workspace-state-empty-favorites-title = Aucun favori pour le moment
workspace-state-empty-favorites-desc = Marquez une page comme favorite pour la garder en haut de la barre latérale.
workspace-state-empty-pages-title = Aucune page pour le moment
workspace-state-empty-pages-desc = Les pages sont les notes et documents de cet espace. Créez la première.

# -----------------------------------------------------------------------------
# #2604 — Protection panel (status comes from /api/security/protection)
# -----------------------------------------------------------------------------
protection-title = Protection
protection-subtitle = Host security tools reported by the server — never a hardcoded "enabled" state.
protection-scanner-status = Scanner status
protection-loading = Loading scanner status…
protection-unknown = unknown
protection-days-old = {days} days old
protection-unknown-tool = unknown tool
protection-active = Active
protection-not-scanning = NOT SCANNING — no scanner is active
protection-version = Version
protection-signature-age = Signature age
protection-last-scan = Last scan
protection-never = never
protection-run-scan = Run scan
protection-opt-out = Uploads are NOT scanned: unscanned imports are allowed.
protection-no-scanner = No scanner is registered.
protection-quarantine = Quarantine
protection-tool = Tool
protection-list = List quarantined files
protection-quarantine-empty = No quarantined files.
protection-restore = Restore
protection-action-failed = The action failed.
protection-load-error = Scanner status is unavailable.
protection-clamav = ClamAV
protection-lmd = LMD
protection-quarantined-count = {count} en quarantaine
