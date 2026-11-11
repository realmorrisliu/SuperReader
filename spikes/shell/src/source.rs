use std::collections::HashMap;

use spike_epub::PackedBook;
use spike_pdf::{Document, OutlineEntry};
use spike_report::{pdfium_lib, PdfEngine};

pub struct OpenedBook {
    pub packed: PackedBook,
    pub toc: Vec<OutlineEntry>,
    pub key: String,
}

pub fn open_pdf(path: &str) -> Result<OpenedBook, String> {
    let bytes = std::fs::read(path).map_err(|err| format!("read {path}: {err}"))?;
    if bytes.is_empty() {
        return Err(format!("empty pdf {path}"));
    }
    let engine = match std::env::var("SPIKE_ENGINE").as_deref() {
        Ok("hayro") => PdfEngine::Hayro,
        _ => PdfEngine::Pdfium,
    };
    let (doc, _, _) = Document::open(engine, bytes, &pdfium_lib())?;
    let count = doc.page_count();
    if count == 0 {
        return Err(format!("no pages in {path}"));
    }
    let mut title = doc.title();
    if title.is_empty() {
        title = std::path::Path::new(path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("PDF")
            .to_string();
    }
    let toc = doc.outline();
    let mut files = HashMap::new();
    files.insert("/fixed.html".to_string(), fixed_html(count).into_bytes());
    for index in 0..count {
        let png = doc.page_png(index, 1400)?;
        files.insert(format!("/pages/{index}.png"), png);
    }
    let packed = PackedBook {
        title,
        href: "/fixed.html".to_string(),
        language: "en".to_string(),
        declared_page: None,
        files,
        spine: vec!["/fixed.html".to_string()],
    };
    Ok(OpenedBook {
        packed,
        toc,
        key: path.to_string(),
    })
}

pub fn fixed_html(count: usize) -> String {
    let mut html = String::from(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>\
section.spike-sheet{display:flex;align-items:center;justify-content:center;break-after:column;break-inside:avoid;margin:0;padding:0;}\
section.spike-sheet img{max-width:100%;max-height:100%;object-fit:contain;}\
</style></head><body class=\"spike-fixed\"><!-- spike-fixed -->",
    );
    for index in 0..count {
        html.push_str(&format!(
            "<section class=\"spike-sheet\"><img alt=\"\" src=\"epub://book/pages/{index}.png\"></section>"
        ));
    }
    html.push_str("</body></html>");
    html
}

#[cfg(test)]
mod tests {
    use super::fixed_html;

    #[test]
    fn each_pdf_page_is_one_sheet() {
        let html = fixed_html(3);
        assert_eq!(html.matches("<section class=\"spike-sheet\">").count(), 3);
        assert!(html.contains("object-fit:contain"));
        assert!(html.contains("epub://book/pages/0.png"));
        assert!(html.contains("epub://book/pages/2.png"));
        assert!(html.contains("spike-fixed"));
        assert!(!html.to_ascii_lowercase().contains("spread"));
    }
}
