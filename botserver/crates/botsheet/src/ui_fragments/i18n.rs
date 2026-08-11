//! Server-side fragment i18n: compact EN/PT catalogs keyed by string id,
//! selected from the request `Accept-Language` header (#792, gap 29).
//!
//! Values may carry `[name]` placeholders substituted by [`tf`]. Missing PT
//! entries fall back to EN, and missing EN entries fall back to the key so a
//! fragment never renders an empty label.

use axum::http::header::ACCEPT_LANGUAGE;
use axum::http::HeaderMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Pt,
}

impl Lang {
    /// First `Accept-Language` locale; anything starting with `pt` selects
    /// Portuguese, anything else defaults to English.
    pub fn from_headers(headers: &HeaderMap) -> Lang {
        let Some(value) = headers.get(ACCEPT_LANGUAGE) else {
            return Lang::En;
        };
        let Ok(value) = value.to_str() else {
            return Lang::En;
        };
        let first = value
            .split(',')
            .next()
            .unwrap_or("en")
            .trim()
            .to_ascii_lowercase();
        if first.starts_with("pt") {
            Lang::Pt
        } else {
            Lang::En
        }
    }
}

const EN: &[(&str, &str)] = &[
    ("common.error_label", "Error:"),
    ("common.spreadsheet", "Spreadsheet"),
    ("common.sheets", "Sheets"),
    ("common.filled_cells", "Filled Cells"),
    ("common.named_ranges", "Named Ranges"),
    ("common.owner", "Owner"),
    ("metadata.sheets_count", "[n] sheet(s)"),
    ("metadata.updated", "Updated [date]"),
    ("sidebar.empty", "No spreadsheets yet. Create one to start."),
    ("sidebar.title", "My Sheets"),
    ("sidebar.new_title", "New Sheet"),
    ("sidebar.new", "+ New"),
    ("search.empty", "No results for this search."),
    ("search.title", "Search Results"),
    ("search.count", "[n] result(s) found"),
    ("recent.empty", "No recent spreadsheets."),
    ("recent.title", "Recent"),
    ("tabs.add_title", "New worksheet"),
    ("tabs.delete_title", "Delete worksheet"),
    ("tabs.confirm_delete", "Delete [name]?"),
    ("preview.cells", "[n] cells"),
    ("panel.ranges.title", "Named Ranges"),
    ("panel.ranges.add", "Add Range"),
    ("panel.ranges.new", "+ New"),
    (
        "panel.ranges.empty",
        "No named ranges defined. Create named ranges to simplify formula references.",
    ),
    ("panel.sheet_n", "Sheet [n]"),
    ("panel.global", "Global"),
    ("panel.charts.title", "Charts"),
    (
        "panel.charts.empty",
        "No charts created. Use the Insert Chart button to add one.",
    ),
    ("panel.charts.series", "Type: [t] • [n] series"),
    ("panel.validations.title", "Data Validations"),
    ("panel.validations.empty", "No data validations configured."),
    ("panel.type", "Type: [t]"),
    ("panel.cf.title", "Conditional Formatting"),
    ("panel.cf.empty", "No conditional formatting rules applied."),
    ("panel.links.title", "External Links"),
    ("panel.links.empty", "No external links configured."),
    ("panel.links.info", "Type: [t] • Status: [s]"),
    ("panel.comments.title", "Comments"),
    ("panel.comments.empty", "No comments added."),
    ("panel.comments.resolved", "Resolved"),
    ("panel.protection.title", "Sheet Protection"),
    ("panel.protection.protected", "🔒 Protected"),
    ("panel.protection.unprotected", "🔓 Unprotected"),
    ("panel.protection.locked", "Locked cells: [n] • [p]"),
    ("panel.protection.fmt_allowed", "Formatting allowed"),
    ("panel.protection.fmt_blocked", "Formatting blocked"),
    ("panel.protection.empty", "No protection configured."),
    ("panel.arrays.title", "Array Formulas"),
    ("panel.arrays.empty", "No array formulas defined."),
    ("panel.arrays.dynamic", "Dynamic"),
    ("panel.arrays.range", "Range: [a]:[b] → [c]:[d]"),
    ("form.share.email", "Collaborator email"),
    ("form.share.permission", "Permission"),
    ("form.share.view", "View"),
    ("form.share.comment", "Comment"),
    ("form.share.edit", "Edit"),
    ("form.share.submit", "Share"),
    ("form.share.removed", "Range \"[name]\" removed."),
    ("toast.invalid_value", "Invalid value"),
    ("form.find.label", "Find"),
    ("form.find.placeholder", "Text to find"),
    ("form.replace.label", "Replace with"),
    ("form.replace.placeholder", "Replacement text"),
    ("form.apply", "Apply"),
    ("form.cf.range", "Range (e.g. A1:D10)"),
    ("form.cf.rule", "Rule"),
    ("form.cf.gt", "Greater than"),
    ("form.cf.lt", "Less than"),
    ("form.cf.eq", "Equal to"),
    ("form.cf.between", "Between"),
    ("form.cf.contains", "Contains text"),
    ("form.cf.duplicates", "Duplicate values"),
    ("form.value", "Value"),
    ("form.cf.bg", "Background color"),
    ("form.cf.submit", "Apply Rule"),
    ("form.dv.cell", "Cell or range"),
    ("form.dv.kind", "Validation type"),
    ("form.dv.list", "List"),
    ("form.dv.number", "Number"),
    ("form.dv.date", "Date"),
    ("form.dv.text_length", "Text length"),
    ("form.dv.custom", "Custom formula"),
    ("form.dv.values", "Values (comma-separated for list)"),
    ("form.dv.error_msg", "Error message"),
    ("form.dv.submit", "Apply Validation"),
    ("form.range", "Range"),
    ("form.fmt.code", "Custom format (spreadsheet pattern)"),
    ("form.fmt.examples", "Examples:"),
    ("form.fmt.submit", "Apply Format"),
    ("form.img.file", "Image file"),
    ("form.img.anchor", "Anchor to cell"),
    ("form.img.submit", "Insert Image"),
    ("form.print.title", "Print Preview"),
    (
        "form.print.description",
        "The sheet will be rendered in landscape A4 with 2cm margins.",
    ),
    ("form.print.orientation", "Orientation"),
    ("form.print.portrait", "Portrait"),
    ("form.print.landscape", "Landscape"),
    ("form.print.scale", "Scale (%)"),
    ("form.print.submit", "Generate PDF"),
    ("form.chart.type", "Chart type"),
    ("form.chart.bar", "Bar"),
    ("form.chart.line", "Line"),
    ("form.chart.pie", "Pie"),
    ("form.chart.scatter", "Scatter"),
    ("form.chart.area", "Area"),
    ("form.chart.column", "Column"),
    ("form.chart.data_range", "Data range"),
    ("form.chart.title", "Title"),
    ("form.chart.anchor", "Anchor at"),
    ("form.chart.submit", "Create Chart"),
    ("modal.share_title", "Share \"[name]\""),
    ("modal.chart", "Insert Chart"),
    ("modal.find_replace", "Find & Replace"),
    ("modal.conditional_format", "Conditional Formatting"),
    ("modal.data_validation", "Data Validation"),
    ("modal.custom_format", "Custom Number Format"),
    ("modal.insert_image", "Insert Image"),
    ("modal.print_preview", "Print Preview"),
    ("modal.ai_assistant", "AI Assistant"),
    (
        "modal.ai_placeholder",
        "Ask: 'Create a formula to sum column B if A is greater than 100'",
    ),
    ("modal.ai_submit", "Ask AI"),
    ("panel.advanced.title", "Advanced Ranges"),
    ("panel.advanced.cf_desc", "Visual rules to highlight data"),
    ("panel.advanced.dv_desc", "Restrict data entry"),
    ("panel.advanced.charts_desc", "Visualize data"),
    ("panel.advanced.print_desc", "Export for printing"),
    ("panel.advanced.ai_desc", "Generate formulas and insights"),
    ("panel.advanced.find_desc", "Search text in the sheet"),
];

const PT: &[(&str, &str)] = &[
    ("common.error_label", "Erro:"),
    ("common.spreadsheet", "Planilha"),
    ("common.sheets", "Planilhas"),
    ("common.filled_cells", "Células Preenchidas"),
    ("common.named_ranges", "Ranges Nomeados"),
    ("common.owner", "Proprietário"),
    ("metadata.sheets_count", "[n] planilha(s)"),
    ("metadata.updated", "Atualizado [date]"),
    ("sidebar.empty", "Nenhuma planilha. Crie uma nova para começar."),
    ("sidebar.title", "Minhas Planilhas"),
    ("sidebar.new_title", "Nova Planilha"),
    ("sidebar.new", "+ Nova"),
    ("search.empty", "Nenhum resultado para esta busca."),
    ("search.title", "Resultados da Busca"),
    ("search.count", "[n] resultado(s) encontrado(s)"),
    ("recent.empty", "Nenhuma planilha recente."),
    ("recent.title", "Recentes"),
    ("tabs.add_title", "Nova Planilha"),
    ("tabs.delete_title", "Excluir planilha"),
    ("tabs.confirm_delete", "Excluir [name]?"),
    ("preview.cells", "[n] células"),
    ("panel.ranges.title", "Ranges Nomeados"),
    ("panel.ranges.add", "Adicionar Range"),
    ("panel.ranges.new", "+ Novo"),
    (
        "panel.ranges.empty",
        "Nenhum range nomeado definido. Crie ranges nomeados para facilitar referências em fórmulas.",
    ),
    ("panel.sheet_n", "Planilha [n]"),
    ("panel.global", "Global"),
    ("panel.charts.title", "Gráficos"),
    (
        "panel.charts.empty",
        "Nenhum gráfico criado. Use o botão Inserir Gráfico para adicionar.",
    ),
    ("panel.charts.series", "Tipo: [t] • [n] série(s)"),
    ("panel.validations.title", "Validações de Dados"),
    ("panel.validations.empty", "Nenhuma validação de dados configurada."),
    ("panel.type", "Tipo: [t]"),
    ("panel.cf.title", "Formatação Condicional"),
    ("panel.cf.empty", "Nenhuma regra de formatação condicional aplicada."),
    ("panel.links.title", "Links Externos"),
    ("panel.links.empty", "Nenhum link externo configurado."),
    ("panel.links.info", "Tipo: [t] • Status: [s]"),
    ("panel.comments.title", "Comentários"),
    ("panel.comments.empty", "Nenhum comentário adicionado."),
    ("panel.comments.resolved", "Resolvido"),
    ("panel.protection.title", "Proteção da Planilha"),
    ("panel.protection.protected", "🔒 Protegida"),
    ("panel.protection.unprotected", "🔓 Desprotegida"),
    ("panel.protection.locked", "Células bloqueadas: [n] • [p]"),
    ("panel.protection.fmt_allowed", "Formatação permitida"),
    ("panel.protection.fmt_blocked", "Formatação bloqueada"),
    ("panel.protection.empty", "Nenhuma proteção configurada."),
    ("panel.arrays.title", "Fórmulas de Matriz"),
    ("panel.arrays.empty", "Nenhuma fórmula de matriz (array) definida."),
    ("panel.arrays.dynamic", "Dinâmica"),
    ("panel.arrays.range", "Range: [a]:[b] → [c]:[d]"),
    ("form.share.email", "Email do colaborador"),
    ("form.share.permission", "Permissão"),
    ("form.share.view", "Visualizar"),
    ("form.share.comment", "Comentar"),
    ("form.share.edit", "Editar"),
    ("form.share.submit", "Compartilhar"),
    ("form.share.removed", "Range \"[name]\" removido."),
    ("toast.invalid_value", "Valor inválido"),
    ("form.find.label", "Localizar"),
    ("form.find.placeholder", "Texto a buscar"),
    ("form.replace.label", "Substituir por"),
    ("form.replace.placeholder", "Texto substituto"),
    ("form.apply", "Aplicar"),
    ("form.cf.range", "Range (ex: A1:D10)"),
    ("form.cf.rule", "Regra"),
    ("form.cf.gt", "Maior que"),
    ("form.cf.lt", "Menor que"),
    ("form.cf.eq", "Igual a"),
    ("form.cf.between", "Entre"),
    ("form.cf.contains", "Contém texto"),
    ("form.cf.duplicates", "Valores duplicados"),
    ("form.value", "Valor"),
    ("form.cf.bg", "Cor de fundo"),
    ("form.cf.submit", "Aplicar Regra"),
    ("form.dv.cell", "Célula ou range"),
    ("form.dv.kind", "Tipo de validação"),
    ("form.dv.list", "Lista"),
    ("form.dv.number", "Número"),
    ("form.dv.date", "Data"),
    ("form.dv.text_length", "Comprimento do texto"),
    ("form.dv.custom", "Fórmula personalizada"),
    ("form.dv.values", "Valores (separados por vírgula para lista)"),
    ("form.dv.error_msg", "Mensagem de erro"),
    ("form.dv.submit", "Aplicar Validação"),
    ("form.range", "Range"),
    ("form.fmt.code", "Formato personalizado (padrão de planilha)"),
    ("form.fmt.examples", "Exemplos:"),
    ("form.fmt.submit", "Aplicar Formato"),
    ("form.img.file", "Arquivo de imagem"),
    ("form.img.anchor", "Ancorar na célula"),
    ("form.img.submit", "Inserir Imagem"),
    ("form.print.title", "Pré-visualização de Impressão"),
    (
        "form.print.description",
        "A planilha será renderizada no formato paisagem A4 com margens de 2cm.",
    ),
    ("form.print.orientation", "Orientação"),
    ("form.print.portrait", "Retrato"),
    ("form.print.landscape", "Paisagem"),
    ("form.print.scale", "Escala (%)"),
    ("form.print.submit", "Gerar PDF"),
    ("form.chart.type", "Tipo de gráfico"),
    ("form.chart.bar", "Barras"),
    ("form.chart.line", "Linhas"),
    ("form.chart.pie", "Pizza"),
    ("form.chart.scatter", "Dispersão"),
    ("form.chart.area", "Área"),
    ("form.chart.column", "Colunas"),
    ("form.chart.data_range", "Range de dados"),
    ("form.chart.title", "Título"),
    ("form.chart.anchor", "Ancorar em"),
    ("form.chart.submit", "Criar Gráfico"),
    ("modal.share_title", "Compartilhar \"[name]\""),
    ("modal.chart", "Inserir Gráfico"),
    ("modal.find_replace", "Localizar e Substituir"),
    ("modal.conditional_format", "Formatação Condicional"),
    ("modal.data_validation", "Validação de Dados"),
    ("modal.custom_format", "Formato Numérico Personalizado"),
    ("modal.insert_image", "Inserir Imagem"),
    ("modal.print_preview", "Pré-visualização de Impressão"),
    ("modal.ai_assistant", "Assistente IA"),
    (
        "modal.ai_placeholder",
        "Pergunte: 'Crie uma fórmula para somar a coluna B se A for maior que 100'",
    ),
    ("modal.ai_submit", "Perguntar à IA"),
    ("panel.advanced.title", "Ranges Avançados"),
    ("panel.advanced.cf_desc", "Regras visuais para destacar dados"),
    ("panel.advanced.dv_desc", "Restringir entrada de dados"),
    ("panel.advanced.charts_desc", "Visualizar dados"),
    ("panel.advanced.print_desc", "Exportar para impressão"),
    ("panel.advanced.ai_desc", "Gere fórmulas e insights"),
    ("panel.advanced.find_desc", "Buscar texto na planilha"),
];

fn lookup<'a>(table: &'a [(&str, &str)], key: &str) -> Option<&'a str> {
    table.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// Resolves a catalog key for the given language, falling back to EN and
/// then to the key itself so a fragment never renders an empty label.
pub fn t(lang: Lang, key: &str) -> String {
    match lang {
        Lang::En => lookup(EN, key),
        Lang::Pt => lookup(PT, key).or_else(|| lookup(EN, key)),
    }
    .map(str::to_string)
    .unwrap_or_else(|| key.to_string())
}

/// Resolves a key and substitutes `[name]` placeholders with the given pairs.
pub fn tf(lang: Lang, key: &str, pairs: &[(&str, &str)]) -> String {
    let mut out = t(lang, key);
    for (name, value) in pairs {
        out = out.replace(&format!("[{name}]"), value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_default_without_header() {
        assert_eq!(Lang::from_headers(&HeaderMap::new()), Lang::En);
    }

    #[test]
    fn portuguese_accept_language() {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_LANGUAGE, "pt-BR,pt;q=0.9,en;q=0.8".parse().unwrap());
        assert_eq!(Lang::from_headers(&headers), Lang::Pt);
    }

    #[test]
    fn english_accept_language() {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
        assert_eq!(Lang::from_headers(&headers), Lang::En);
    }

    #[test]
    fn unknown_locale_falls_back_to_english() {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_LANGUAGE, "de-DE,de;q=0.9".parse().unwrap());
        assert_eq!(Lang::from_headers(&headers), Lang::En);
    }

    #[test]
    fn lookup_translates() {
        assert_eq!(t(Lang::En, "common.sheets"), "Sheets");
        assert_eq!(t(Lang::Pt, "common.sheets"), "Planilhas");
        assert_eq!(t(Lang::Pt, "common.sheets"), "Planilhas");
    }

    #[test]
    fn missing_key_returns_key() {
        assert_eq!(t(Lang::En, "no.such.key"), "no.such.key");
    }

    #[test]
    fn placeholder_substitution() {
        assert_eq!(tf(Lang::En, "metadata.sheets_count", &[("n", "3")]), "3 sheet(s)");
        assert_eq!(tf(Lang::Pt, "tabs.confirm_delete", &[("name", "Sheet1")]), "Excluir Sheet1?");
    }

    #[test]
    fn pt_falls_back_to_en_for_missing_entry() {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_LANGUAGE, "pt".parse().unwrap());
        let lang = Lang::from_headers(&headers);
        assert_eq!(t(lang, "common.error_label"), "Erro:");
        assert_eq!(t(Lang::Pt, "no.such.key"), "no.such.key");
    }
}