# =============================================================================
# General Bots — es — Sheets grid accessibility (#2493)
# English is the source of truth: every key must be mirrored in all locales.
# Placeholders ({value}, {range}, ...) are preserved by the FTL loader.
# =============================================================================

# Grid semantics
sheet-a11y-grid-label = Cuadrícula de la hoja de cálculo
sheet-a11y-rowgroup-label = Filas de la cuadrícula

# Formula bar and name box
sheet-a11y-formula-bar-label = Barra de fórmulas
sheet-a11y-name-box-label = Cuadro de nombre, introduzca una referencia de celda
sheet-a11y-name-box-placeholder = A1
sheet-a11y-invalid-reference = Referencia de celda no válida: {value}

# Cell labels and live announcements
sheet-a11y-cell-label = Celda {address}: {value}
sheet-a11y-cell-formula-label = Fórmula {formula}
sheet-a11y-live-selection = {range}, {rows} filas por {cols} columnas, suma {sum}
sheet-a11y-live-edited = Celda {address} actualizada a {value}
sheet-a11y-clear-too-large = La selección es demasiado grande para borrarla con el teclado

# Accessibility checker
sheet-a11y-checker-title = Comprobación de accesibilidad
sheet-a11y-checker-close = Cerrar comprobación de accesibilidad
sheet-a11y-checker-run = Comprobar accesibilidad
sheet-a11y-checker-summary = Se encontraron {count} problemas de accesibilidad
sheet-a11y-checker-empty = No se encontraron problemas de accesibilidad.
sheet-a11y-checker-sheet = Hoja
sheet-a11y-checker-sheet-fallback = Hoja {index}
sheet-a11y-severity-warning = Advertencia
sheet-a11y-finding-merged = Las celdas combinadas en {range} pueden confundir a los lectores de pantalla; evite combinar celdas en hojas accesibles.
sheet-a11y-finding-color-only = El formato condicional en {range} usa solo color; añada un icono, etiqueta o patrón.
sheet-a11y-finding-no-alt = La imagen {name} no tiene texto alternativo.
sheet-a11y-unnamed-image = sin nombre
sheet-a11y-finding-no-header = La hoja {sheet} tiene datos pero no tiene una fila de encabezado.
