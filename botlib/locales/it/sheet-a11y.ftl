# =============================================================================
# General Bots — en — Sheets grid accessibility (#2493)
# English is the source of truth: every key must be mirrored in all locales.
# Placeholders ({value}, {range}, ...) are preserved by the FTL loader.
# =============================================================================

# Grid semantics
sheet-a11y-grid-label = Griglia di fogli di calcolo
sheet-a11y-rowgroup-label = Righe di griglia

# Formula bar and name box
sheet-a11y-formula-bar-label = Barra di formula
sheet-a11y-name-box-label = Casella di nome, inserire un riferimento della cella
sheet-a11y-name-box-placeholder = A1
sheet-a11y-invalid-reference = Riferimento della cella non valido: {value}

# Cell labels and live announcements
sheet-a11y-cell-label = Cella {address}: {value}
sheet-a11y-cell-formula-label = Formula {formula}
sheet-a11y-live-selection = {range},{rows}righe per{cols}colonne, somma{sum}
sheet-a11y-live-edited = Cella {address} aggiornata a {value}
sheet-a11y-clear-too-large = Selezione troppo grande per essere chiarita con la tastiera

# Accessibility checker
sheet-a11y-checker-title = Controllo accessibilità
sheet-a11y-checker-close = Chiudi controllo accessibilità
sheet-a11y-checker-run = Verifica accessibilità
sheet-a11y-checker-summary = Problemi di accessibilità {count} trovati
sheet-a11y-checker-empty = Non sono stati riscontrati problemi di accessibilità.
sheet-a11y-checker-sheet = Foglio
sheet-a11y-checker-sheet-fallback = Scheda {index}
sheet-a11y-severity-warning = Attenzione
sheet-a11y-finding-merged = Le celle unite a {range} possono confondere i lettori dello schermo; evitare la fusione in libri di lavoro accessibili.
sheet-a11y-finding-color-only = Il formato condizionale in {range} utilizza solo il colore; aggiungi un'icona, un'etichetta o un motivo.
sheet-a11y-finding-no-alt = Immagine {name} non ha testo alternativo.
sheet-a11y-unnamed-image = senza nome
sheet-a11y-finding-no-header = Scheda di lavoro {sheet} ha dati ma nessuna riga di intestazione.
