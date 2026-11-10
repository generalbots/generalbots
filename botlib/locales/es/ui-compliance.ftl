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
