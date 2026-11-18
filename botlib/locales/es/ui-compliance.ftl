# =============================================================================
# General Bots — es — Compliance
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
compliance-title = Informe de cumplimiento de API
compliance-subtitle = Escaneo de seguridad para todos los bots: verifique contraseñas, códigos frágiles y configuraciones incorrectas
compliance-export-report = Informe de exportación
compliance-run-scan = Ejecutar análisis de cumplimiento
compliance-scanning = Escaneando...

# -----------------------------------------------------------------------------
# Bot Selector
# -----------------------------------------------------------------------------
compliance-all-bots = Todos los robots
compliance-select-bots = Seleccionar robots

# -----------------------------------------------------------------------------
# Stats Cards
# -----------------------------------------------------------------------------
compliance-critical = Crítico
compliance-critical-desc = Requiere acción inmediata
compliance-high = Alto
compliance-high-desc = Riesgo de seguridad
compliance-medium = Medio
compliance-medium-desc = debe abordarse
compliance-low = Bajo
compliance-low-desc = Mejores prácticas
compliance-info = Información
compliance-info-desc = Informativo

# -----------------------------------------------------------------------------
# Filters
# -----------------------------------------------------------------------------
compliance-filter-severity = Gravedad
compliance-filter-type = Tipo
compliance-filter-all-severities = Todas las gravedades
compliance-filter-all-types = Todos los tipos
compliance-search-issues = Problemas de búsqueda...

# -----------------------------------------------------------------------------
# Issue Types
# -----------------------------------------------------------------------------
compliance-type-password = Contraseña en configuración
compliance-type-hardcoded = Secretos codificados
compliance-type-deprecated = Palabras clave obsoletas
compliance-type-fragile = Código frágil
compliance-type-config = Problemas de configuración

# -----------------------------------------------------------------------------
# Results Table
# -----------------------------------------------------------------------------
compliance-results = Resultados
compliance-results-count = { $cuenta ->
    [one] { $count } issue found
   *[other] { $count } issues found
}
compliance-col-severity = Gravedad
compliance-col-issue = Problema
compliance-col-location = Ubicación
compliance-col-details = Detalles
compliance-col-action = acción
compliance-view-details = Ver detalles
compliance-fix-issue = Solucionar problema
compliance-ignore = ignorar
compliance-no-issues = No se encontraron problemas
compliance-no-issues-desc = ¡Genial! Tus bots cumplen.

# -----------------------------------------------------------------------------
# Scan Progress
# -----------------------------------------------------------------------------
compliance-scan-in-progress = Escaneo en progreso...
compliance-scan-checking = Comprobando { $item }...
compliance-scan-complete = Escaneo completo
compliance-scan-failed = Error de escaneo

## Compliance frameworks (enterprise-grade release)
compliance-frameworks = Marcos
compliance-new-framework = Nuevo
compliance-framework-name = Nombre
compliance-framework-version = Versión
compliance-framework-description = Descripción
compliance-create-framework = Crear Marco
compliance-controls = Controles
compliance-add-control = Añadir Control
compliance-control-id = ID del Control
compliance-control-title = Título
compliance-control-category = Categoría
compliance-control-description = Descripción
compliance-mandatory = Obligatorio
compliance-optional = Opcional
compliance-evidence = Evidencias
compliance-attach-evidence = Adjuntar Evidencia
compliance-evidence-path = Ruta del archivo (artefacto en drive)
compliance-evidence-type = Tipo
compliance-approve = Aprobar
compliance-covered = Cubierto
compliance-no-evidence = Sin evidencia
compliance-export-csv = Exportar CSV
compliance-archive = Archivar
compliance-total-controls = Total de Controles
compliance-coverage = Cobertura
compliance-no-frameworks = Aún no hay marcos configurados.

# semantic i18n key (#2542)
admin-compliance-dashboard-title = Panel de cumplimiento

# semantic i18n key (#2542)
admin-compliance-subtitle = Estado de cumplimiento SOC 2 Tipo II, monitoreo de controles y evidencia de auditoría

# semantic i18n key (#2542)
admin-compliance-compliant = Conforme

# semantic i18n key (#2542)
admin-compliance-overall-score = Puntuación general de cumplimiento

# semantic i18n key (#2542)
admin-compliance-controls-passing = Controles aprobados

# semantic i18n key (#2542)
admin-compliance-evidence-items-due = Elementos de evidencia pendientes

# semantic i18n key (#2542)
admin-compliance-until-next-audit = Hasta la próxima auditoría

# semantic i18n key (#2542)
admin-compliance-next-audit-date = Programado: 7 de marzo de 2025

# semantic i18n key (#2542)
admin-compliance-trust-service-criteria = Criterios de servicios de confianza

# semantic i18n key (#2542)
admin-compliance-availability = Disponibilidad

# semantic i18n key (#2542)
admin-compliance-processing-integrity = Integridad del procesamiento

# semantic i18n key (#2542)
admin-compliance-confidentiality = Confidencialidad

# semantic i18n key (#2542)
admin-compliance-controls-requiring-attention = Controles que requieren atención

# semantic i18n key (#2542)
admin-compliance-control-mfa-privileged = Autenticación multifactor para cuentas privilegiadas

# semantic i18n key (#2542)
admin-compliance-due-jan-25-2025 = Vence: 25 ene 2025

# semantic i18n key (#2542)
admin-compliance-remediate = Corregir

# semantic i18n key (#2542)
admin-compliance-control-incident-response-testing = Prueba del plan de respuesta a incidentes de seguridad

# semantic i18n key (#2542)
admin-compliance-issue-annual-test-overdue = Prueba anual atrasada por 15 días

# semantic i18n key (#2542)
admin-compliance-control-dpa-documentation = Documentación del acuerdo de procesamiento de datos

# semantic i18n key (#2542)
admin-compliance-due-feb-1-2025 = Vence: 1 feb 2025

# semantic i18n key (#2542)
admin-compliance-control-data-classification-review = Revisión de la política de clasificación de datos

# semantic i18n key (#2542)
admin-compliance-issue-policy-review-pending = Revisión de la política pendiente de aprobación

# semantic i18n key (#2542)
admin-compliance-due-feb-15-2025 = Vence: 15 feb 2025

# semantic i18n key (#2542)
admin-compliance-control-privacy-notice-updates = Actualizaciones del aviso de privacidad

# semantic i18n key (#2542)
admin-compliance-issue-privacy-policy-update = La política de privacidad necesita actualizarse para la nueva recopilación de datos

# semantic i18n key (#2542)
admin-compliance-due-feb-28-2025 = Vence: 28 feb 2025

# semantic i18n key (#2542)
admin-compliance-evidence-collection = Recopilación de evidencia

# semantic i18n key (#2542)
admin-compliance-current = Actual

# semantic i18n key (#2542)
admin-compliance-due-soon = Vence pronto

# semantic i18n key (#2542)
admin-compliance-evidence-pentest-q4 = Informe de prueba de penetración T4 2024

# semantic i18n key (#2542)
admin-compliance-evidence-cc4-vuln = CC4.1.1 - Evaluación de vulnerabilidades

# semantic i18n key (#2542)
admin-compliance-evidence-access-review-jan = Informe de revisión de acceso - Enero de 2025

# semantic i18n key (#2542)
admin-compliance-evidence-cc6-access = CC6.2.1 - Control de acceso

# semantic i18n key (#2542)
admin-compliance-due-in-3-days = Vence en 3 días

# semantic i18n key (#2542)
admin-compliance-evidence-backup-restore = Resultados de la prueba de restauración de copias de seguridad

# semantic i18n key (#2542)
admin-compliance-evidence-a1-backup = A1.2.3 - Copia de seguridad y recuperación

# semantic i18n key (#2542)
admin-compliance-due-in-5-days = Vence en 5 días

# semantic i18n key (#2542)
admin-compliance-evidence-security-training = Registros de capacitación en concienciación de seguridad

# semantic i18n key (#2542)
admin-compliance-evidence-cc1-training = CC1.4.1 - Capacitación en seguridad

# semantic i18n key (#2542)
admin-compliance-view-all-evidence = Ver toda la evidencia

# semantic i18n key (#2542)
admin-compliance-access-events = Eventos de acceso

# semantic i18n key (#2542)
admin-compliance-configuration-changes = Cambios de configuración

# semantic i18n key (#2542)
admin-compliance-security-events = Eventos de seguridad

# semantic i18n key (#2542)
admin-compliance-compliance-events = Eventos de cumplimiento

# semantic i18n key (#2542)
admin-compliance-failed-login-attempt = Intento de inicio de sesión fallido

# semantic i18n key (#2542)
admin-compliance-failed-login-details = Usuario: admin@company.com | IP: 192.168.1.105 | Motivo: Contraseña no válida

# semantic i18n key (#2542)
admin-compliance-user-logged-in = Usuario inició sesión

# semantic i18n key (#2542)
admin-compliance-user-logged-in-details = Usuario: sarah.johnson@company.com | IP: 10.0.0.45
