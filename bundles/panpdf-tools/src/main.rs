use std::process::ExitCode;

#[path = "../../../tools/compare-pdf/src/main.rs"]
mod compare_pdf;
#[path = "../../../tools/compress-pdf/src/main.rs"]
mod compress_pdf;
#[path = "../../../tools/excel-to-pdf/src/main.rs"]
mod excel_to_pdf;
#[path = "../../../tools/html-to-pdf/src/main.rs"]
mod html_to_pdf;
#[path = "../../../tools/image-to-pdf/src/main.rs"]
mod image_to_pdf;
#[path = "../../../tools/ocr-pdf/src/main.rs"]
mod ocr_pdf;
#[path = "../../../tools/pdf-to-excel/src/main.rs"]
mod pdf_to_excel;
#[path = "../../../tools/pdf-to-html/src/main.rs"]
mod pdf_to_html;
#[path = "../../../tools/pdf-to-image/src/main.rs"]
mod pdf_to_image;
#[path = "../../../tools/pdf-to-markdown/src/main.rs"]
mod pdf_to_markdown;
#[path = "../../../tools/pdf-to-pdfa/src/main.rs"]
mod pdf_to_pdfa;
#[path = "../../../tools/pdf-to-powerpoint/src/main.rs"]
mod pdf_to_powerpoint;
#[path = "../../../tools/pdf-to-text/src/main.rs"]
mod pdf_to_text;
#[path = "../../../tools/pdf-to-word/src/main.rs"]
mod pdf_to_word;
#[path = "../../../tools/powerpoint-to-pdf/src/main.rs"]
mod powerpoint_to_pdf;
#[path = "../../../tools/protect-pdf/src/main.rs"]
mod protect_pdf;
#[path = "../../../tools/redact-pdf/src/main.rs"]
mod redact_pdf;
#[path = "../../../tools/repair-pdf/src/main.rs"]
mod repair_pdf;
#[path = "../../../tools/scan-to-pdf/src/main.rs"]
mod scan_to_pdf;
#[path = "../../../tools/sign-pdf/src/main.rs"]
mod sign_pdf;
#[path = "../../../tools/unlock-pdf/src/main.rs"]
mod unlock_pdf;
#[path = "../../../tools/word-to-pdf/src/main.rs"]
mod word_to_pdf;

const TOOLS: &[&str] = &[
    "compare-pdf",
    "compress-pdf",
    "excel-to-pdf",
    "html-to-pdf",
    "image-to-pdf",
    "ocr-pdf",
    "pdf-to-excel",
    "pdf-to-html",
    "pdf-to-image",
    "pdf-to-markdown",
    "pdf-to-pdfa",
    "pdf-to-powerpoint",
    "pdf-to-text",
    "pdf-to-word",
    "powerpoint-to-pdf",
    "protect-pdf",
    "redact-pdf",
    "repair-pdf",
    "scan-to-pdf",
    "sign-pdf",
    "unlock-pdf",
    "word-to-pdf",
];

fn main() -> ExitCode {
    let name = std::env::args_os()
        .next()
        .and_then(|arg0| {
            std::path::Path::new(&arg0)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_default();
    if TOOLS.contains(&name.as_str()) {
        return run(&name);
    }
    eprintln!("panpdf-tools: run it as one of: {}", TOOLS.join(", "));
    ExitCode::from(2)
}

fn run(tool: &str) -> ExitCode {
    match tool {
        "compare-pdf" => compare_pdf::main(),
        "compress-pdf" => compress_pdf::main(),
        "excel-to-pdf" => excel_to_pdf::main(),
        "html-to-pdf" => html_to_pdf::main(),
        "image-to-pdf" => image_to_pdf::main(),
        "ocr-pdf" => ocr_pdf::main(),
        "pdf-to-excel" => pdf_to_excel::main(),
        "pdf-to-html" => pdf_to_html::main(),
        "pdf-to-image" => pdf_to_image::main(),
        "pdf-to-markdown" => pdf_to_markdown::main(),
        "pdf-to-pdfa" => pdf_to_pdfa::main(),
        "pdf-to-powerpoint" => pdf_to_powerpoint::main(),
        "pdf-to-text" => pdf_to_text::main(),
        "pdf-to-word" => pdf_to_word::main(),
        "powerpoint-to-pdf" => powerpoint_to_pdf::main(),
        "protect-pdf" => protect_pdf::main(),
        "redact-pdf" => redact_pdf::main(),
        "repair-pdf" => repair_pdf::main(),
        "scan-to-pdf" => scan_to_pdf::main(),
        "sign-pdf" => sign_pdf::main(),
        "unlock-pdf" => unlock_pdf::main(),
        "word-to-pdf" => word_to_pdf::main(),
        _ => ExitCode::from(2),
    }
}
