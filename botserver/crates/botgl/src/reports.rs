use crate::types::{BalanceSheetRow, IncomeStatementRow};

pub fn format_currency(value: &rust_decimal::Decimal) -> String {
    format!("{:.2}", value)
}

pub fn generate_chart_of_accounts() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("1", "Ativo", "asset"),
        ("1.1", "Ativo Circulante", "asset"),
        ("1.1.1", "Caixa e Equivalentes", "asset"),
        ("1.1.2", "Contas a Receber", "asset"),
        ("1.1.3", "Estoques", "asset"),
        ("1.2", "Ativo Nao Circulante", "asset"),
        ("1.2.1", "Imobilizado", "asset"),
        ("1.2.2", "Intangivel", "asset"),
        ("2", "Passivo", "liability"),
        ("2.1", "Passivo Circulante", "liability"),
        ("2.1.1", "Contas a Pagar", "liability"),
        ("2.1.2", "Obrigacoes Tributarias", "liability"),
        ("2.2", "Passivo Nao Circulante", "liability"),
        ("2.2.1", "Financiamentos", "liability"),
        ("3", "Patrimonio Liquido", "equity"),
        ("3.1", "Capital Social", "equity"),
        ("3.2", "Reservas", "equity"),
        ("4", "Receitas", "revenue"),
        ("4.1", "Receita Operacional", "revenue"),
        ("4.2", "Receita Nao Operacional", "revenue"),
        ("5", "Despesas", "expense"),
        ("5.1", "Despesas Operacionais", "expense"),
        ("5.2", "Despesas Administrativas", "expense"),
        ("5.3", "Despesas Tributarias", "expense"),
    ]
}
