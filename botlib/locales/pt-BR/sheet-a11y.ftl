# =============================================================================
# General Bots — pt-BR — Sheets grid accessibility (#2493)
# English is the source of truth: every key must be mirrored in all locales.
# Placeholders ({value}, {range}, ...) are preserved by the FTL loader.
# =============================================================================

# Grid semantics
sheet-a11y-grid-label = Grade da planilha
sheet-a11y-rowgroup-label = Linhas da grade

# Formula bar and name box
sheet-a11y-formula-bar-label = Barra de fórmulas
sheet-a11y-name-box-label = Caixa de nome, informe uma referência de célula
sheet-a11y-name-box-placeholder = A1
sheet-a11y-invalid-reference = Referência de célula inválida: {value}

# Cell labels and live announcements
sheet-a11y-cell-label = Célula {address}: {value}
sheet-a11y-cell-formula-label = Fórmula {formula}
sheet-a11y-live-selection = {range}, {rows} linhas por {cols} colunas, soma {sum}
sheet-a11y-live-edited = Célula {address} atualizada para {value}
sheet-a11y-clear-too-large = Seleção muito grande para limpar com o teclado

# Accessibility checker
sheet-a11y-checker-title = Verificação de acessibilidade
sheet-a11y-checker-close = Fechar verificação de acessibilidade
sheet-a11y-checker-run = Verificar acessibilidade
sheet-a11y-checker-summary = {count} problemas de acessibilidade encontrados
sheet-a11y-checker-empty = Nenhum problema de acessibilidade encontrado.
sheet-a11y-checker-sheet = Planilha
sheet-a11y-checker-sheet-fallback = Planilha {index}
sheet-a11y-severity-warning = Aviso
sheet-a11y-finding-merged = Células mescladas em {range} podem confundir leitores de tela; evite mesclar em planilhas acessíveis.
sheet-a11y-finding-color-only = A formatação condicional em {range} usa apenas cor; adicione um ícone, rótulo ou padrão.
sheet-a11y-finding-no-alt = A imagem {name} não possui texto alternativo.
sheet-a11y-unnamed-image = sem nome
sheet-a11y-finding-no-header = A planilha {sheet} tem dados, mas não possui uma linha de cabeçalho.
