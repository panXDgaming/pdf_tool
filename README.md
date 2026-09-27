# PDF tools

Converters and small PDF tools of [PanPDF](https://github.com/panXDgaming/panpdf.rs):
PDF to Word, Excel, PowerPoint, pictures, HTML, Markdown and text, and back,
plus compress, repair, OCR, sign, redact, compare, protect and unlock.

Each tool is its own small program and runs on your machine; in the browser
the same code runs as WebAssembly on [panpdf.org](https://panpdf.org/tools/).
Nothing is uploaded anywhere.

## Folders

| Folder | What is in it |
| --- | --- |
| `tools/` | one folder per tool, one program each |
| `shared/` | the parts the tools share: reading a page's structure, ZIP, XML, Office files, drawing, layout |
| `bundles/panpdf-tools` | every tool in one program, run by the tool's name (the Android app uses it) |
| `web/ABI.md` | how a web page calls a tool's WebAssembly module |

## Build

Needs Rust (the version is pinned in `rust-toolchain.toml`). The PDF engine is
fetched from panpdf.rs by Cargo.

```sh
git clone https://github.com/panXDgaming/pdf_tool
cd pdf_tool
cargo build --release -p pdf-to-word          # one tool
cargo build --release                         # all of them
```

The tools that lay text out (Word, Excel, PowerPoint and HTML to PDF) need
PanPDF's fonts: clone panpdf.rs beside this folder and run its
`python3 fonts/vendor.py`, or point `PANPDF_FONTS` at a folder of them.

## Use

```sh
target/release/pdf-to-word report.pdf report.docx
target/release/pdf-to-word report.pdf report.docx --pages 3,10-12
target/release/compress-pdf big.pdf small.pdf
target/release/pdf-to-word --help
```

For the browser, build a tool as WebAssembly:

```sh
cargo rustc --release -p pdf-to-word --lib --target wasm32-unknown-unknown --crate-type cdylib
```

## The tools

| Tool | What it does |
| --- | --- |
| `compare-pdf` | shows where two versions of a PDF differ |
| `compress-pdf` | a smaller PDF that looks the same |
| `excel-to-pdf` | .xlsx sheets to PDF pages |
| `html-to-pdf` | a saved web page to PDF |
| `image-to-pdf` | JPG, PNG and GIF pictures to PDF pages |
| `ocr-pdf` | makes a scanned PDF searchable (Tesseract) |
| `pdf-to-excel` | the tables of a PDF to an .xlsx workbook |
| `pdf-to-html` | PDF to one self-contained web page |
| `pdf-to-image` | pages to PNG or JPEG, or the pictures inside the PDF |
| `pdf-to-markdown` | headings, lists and tables as Markdown |
| `pdf-to-pdfa` | PDF to PDF/A for archiving |
| `pdf-to-powerpoint` | every page to a slide with its text in text boxes (.pptx) |
| `pdf-to-text` | every word in reading order, as plain text |
| `pdf-to-word` | PDF to an editable .docx: headings, paragraphs, lists, tables, pictures |
| `powerpoint-to-pdf` | .pptx slides to PDF |
| `protect-pdf` | locks a PDF with a password (AES-256) |
| `redact-pdf` | removes words for good, not just covers them |
| `repair-pdf` | recovers what can still be read from a damaged PDF |
| `scan-to-pdf` | phone photos of paper to straight, clean PDF pages |
| `sign-pdf` | puts a drawn, typed or pictured signature on a page |
| `unlock-pdf` | removes the password from a PDF you may open |
| `word-to-pdf` | .docx to PDF, any script laid out correctly |

## Licence

GNU Affero General Public License 3.0, as PanPDF.
