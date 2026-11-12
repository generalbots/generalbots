//! Structural guarantees of the PDF exporter (#2250, #2251).

use super::*;
use crate::storage_docx::PAGE_BREAK_MARKER;

/// Counts page objects: `/Type /Page /Parent` is emitted only for pages,
/// never for the `/Type /Pages` tree node.
fn page_count(pdf: &[u8]) -> usize {
    String::from_utf8_lossy(pdf)
        .matches("/Type /Page /Parent")
        .count()
}

fn stream_text(pdf: &[u8]) -> String {
    String::from_utf8_lossy(pdf).into_owned()
}

#[test]
fn two_page_document_exports_two_pdf_pages() {
    let html = format!(
        "<h1>Currículo</h1>\
         <h2>Experiência</h2><p>Gerente de projetos</p>\
         {PAGE_BREAK_MARKER}\
         <h2>Formação</h2><p>MBA em gestão</p>"
    );

    let pages = layout_pages(&parse_blocks(&html));
    assert_eq!(pages.len(), 2, "the page-break marker must start a new page");
    assert!(pages[0].text().contains("Experi"), "page 1 keeps its section");
    assert!(pages[1].text().contains("Forma"), "page 2 starts the new section");

    let pdf = html_to_pdf(&html);
    assert_eq!(page_count(&pdf), 2, "the PDF must expose two pages");
    assert!(stream_text(&pdf).contains("(Gerente de projetos) Tj"));
}

#[test]
fn editor_page_break_is_honoured_too() {
    // Ctrl+Enter in the editor emits `<hr class="gb-page-break">`.
    let pdf = html_to_pdf("<p>primeira</p><hr class=\"gb-page-break\"><p>segunda</p>");
    assert_eq!(page_count(&pdf), 2);
}

#[test]
fn numbered_clauses_keep_their_numbering_and_order() {
    let html = "<h1>Contrato de Prestação de Serviços</h1>\
                <p>Partes: Contratante e Contratada.</p>\
                <ol>\
                  <li>Objeto do contrato</li>\
                  <li>Prazo de vigência</li>\
                  <li>Valor e forma de pagamento</li>\
                </ol>";

    let pages = layout_pages(&parse_blocks(html));
    assert_eq!(pages.len(), 1);
    let text = pages[0].text();
    assert!(text.contains("1. Objeto do contrato"), "clause 1: {text}");
    assert!(text.contains("2. Prazo de vigência"), "clause 2: {text}");
    assert!(
        text.contains("3. Valor e forma de pagamento"),
        "clause 3: {text}"
    );

    let pdf = stream_text(&html_to_pdf(html));
    assert!(pdf.contains("(1. Objeto do contrato) Tj"));
    // ê = 0xEA is written as the WinAnsi octal escape \352.
    assert!(
        pdf.contains("(2. Prazo de vig\\352ncia) Tj"),
        "clause 2 keeps its number in the PDF stream"
    );
    assert!(pdf.contains("(3. Valor e forma de pagamento) Tj"));
    // The heading is a heading in the PDF stream, not body text.
    assert!(pdf.contains("/F2 18.0 Tf"), "h1 renders bold at 18pt");
}

#[test]
fn list_numbering_continues_from_the_start_attribute() {
    let pages = layout_pages(&parse_blocks(
        "<ol start=\"4\"><li>Quarta</li><li>Quinta</li></ol>",
    ));
    let text = pages[0].text();
    assert!(text.contains("4. Quarta"), "{text}");
    assert!(text.contains("5. Quinta"), "{text}");
}

#[test]
fn bold_runs_and_portuguese_accents_survive_export() {
    let pdf = stream_text(&html_to_pdf(
        "<p><strong>Prestação</strong> de serviços</p>",
    ));
    // ç = 0xE7 and ã = 0xE3 in WinAnsi, written as octal escapes.
    assert!(pdf.contains("Presta\\347\\343o"), "accents kept: {pdf}");
    assert!(pdf.contains("/F2 11.0 Tf"), "the bold run keeps its font");
    assert!(pdf.contains("/F1 11.0 Tf"), "the plain run keeps its font");
}

#[test]
fn no_line_is_laid_out_outside_the_page_margins() {
    let mut html = String::from("<h1>Relatório longo</h1>");
    for index in 0..120 {
        html.push_str(&format!(
            "<p>Parágrafo {index} com texto suficiente para ocupar uma linha inteira da página.</p>"
        ));
    }

    let pages = layout_pages(&parse_blocks(&html));
    assert!(pages.len() > 1, "a long document paginates");
    for page in &pages {
        for line in &page.lines {
            assert!(
                line.y >= layout::MARGIN && line.y <= layout::PAGE_HEIGHT - layout::MARGIN + 0.01,
                "line baseline {} is outside the printable area",
                line.y
            );
        }
    }
}
