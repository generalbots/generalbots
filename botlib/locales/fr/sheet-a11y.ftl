# =============================================================================
# General Bots — fr — Sheets grid accessibility (#2493)
# English is the source of truth: every key must be mirrored in all locales.
# Placeholders ({value}, {range}, ...) are preserved by the FTL loader.
# =============================================================================

# Grid semantics
sheet-a11y-grid-label = Grille de la feuille de calcul
sheet-a11y-rowgroup-label = Lignes de la grille

# Formula bar and name box
sheet-a11y-formula-bar-label = Barre de formule
sheet-a11y-name-box-label = Zone de nom, saisissez une référence de cellule
sheet-a11y-name-box-placeholder = A1
sheet-a11y-invalid-reference = Référence de cellule non valide : {value}

# Cell labels and live announcements
sheet-a11y-cell-label = Cellule {address} : {value}
sheet-a11y-cell-formula-label = Formule {formula}
sheet-a11y-live-selection = {range}, {rows} lignes sur {cols} colonnes, somme {sum}
sheet-a11y-live-edited = Cellule {address} mise à jour vers {value}
sheet-a11y-clear-too-large = Sélection trop grande pour être effacée au clavier

# Accessibility checker
sheet-a11y-checker-title = Contrôle d'accessibilité
sheet-a11y-checker-close = Fermer le contrôle d'accessibilité
sheet-a11y-checker-run = Contrôler l'accessibilité
sheet-a11y-checker-summary = {count} problèmes d'accessibilité trouvés
sheet-a11y-checker-empty = Aucun problème d'accessibilité trouvé.
sheet-a11y-checker-sheet = Feuille
sheet-a11y-checker-sheet-fallback = Feuille {index}
sheet-a11y-severity-warning = Avertissement
sheet-a11y-finding-merged = Les cellules fusionnées en {range} peuvent perturber les lecteurs d'écran ; évitez de fusionner dans les classeurs accessibles.
sheet-a11y-finding-color-only = La mise en forme conditionnelle en {range} utilise uniquement la couleur ; ajoutez une icône, une étiquette ou un motif.
sheet-a11y-finding-no-alt = L'image {name} n'a pas de texte alternatif.
sheet-a11y-unnamed-image = sans nom
sheet-a11y-finding-no-header = La feuille {sheet} contient des données mais n'a pas de ligne d'en-tête.
