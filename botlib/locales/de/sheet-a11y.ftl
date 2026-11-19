# =============================================================================
# General Bots — de — Sheets grid accessibility (#2493)
# English is the source of truth: every key must be mirrored in all locales.
# Placeholders ({value}, {range}, ...) are preserved by the FTL loader.
# =============================================================================

# Grid semantics
sheet-a11y-grid-label = Tabellenraster
sheet-a11y-rowgroup-label = Rasterzeilen

# Formula bar and name box
sheet-a11y-formula-bar-label = Formelleiste
sheet-a11y-name-box-label = Namensfeld, geben Sie einen Zellbezug ein
sheet-a11y-name-box-placeholder = A1
sheet-a11y-invalid-reference = Ungültiger Zellbezug: {value}

# Cell labels and live announcements
sheet-a11y-cell-label = Zelle {address}: {value}
sheet-a11y-cell-formula-label = Formel {formula}
sheet-a11y-live-selection = {range}, {rows} Zeilen mal {cols} Spalten, Summe {sum}
sheet-a11y-live-edited = Zelle {address} aktualisiert auf {value}
sheet-a11y-clear-too-large = Auswahl zu groß, um mit der Tastatur gelöscht zu werden

# Accessibility checker
sheet-a11y-checker-title = Barrierefreiheitsprüfung
sheet-a11y-checker-close = Barrierefreiheitsprüfung schließen
sheet-a11y-checker-run = Barrierefreiheit prüfen
sheet-a11y-checker-summary = {count} Probleme mit der Barrierefreiheit gefunden
sheet-a11y-checker-empty = Keine Probleme mit der Barrierefreiheit gefunden.
sheet-a11y-checker-sheet = Blatt
sheet-a11y-checker-sheet-fallback = Blatt {index}
sheet-a11y-severity-warning = Warnung
sheet-a11y-finding-merged = Zusammengeführte Zellen in {range} können Screenreader verwirren; vermeiden Sie das Zusammenführen in barrierefreien Arbeitsmappen.
sheet-a11y-finding-color-only = Die bedingte Formatierung in {range} verwendet nur Farbe; fügen Sie ein Symbol, eine Beschriftung oder ein Muster hinzu.
sheet-a11y-finding-no-alt = Das Bild {name} hat keinen Alternativtext.
sheet-a11y-unnamed-image = unbenannt
sheet-a11y-finding-no-header = Das Blatt {sheet} enthält Daten, aber keine Kopfzeile.
