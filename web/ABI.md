# Browser bundles: what a page calls

Every tool has one WebAssembly module, `web/wasm/<tool_with_underscores>.wasm`
(`pdf-to-word` -> `pdf_to_word.wasm`), built by `scripts/build-wasm.sh`. A
module has **no imports**: instantiate it with an empty import object,
`WebAssembly.instantiate(bytes, {})`, in a Web Worker, one module instance per
worker. Nothing is fetched by the module; the page fetches the input files and
fonts and copies them in.

All exports work over the module's own memory (`exports.memory`):

- An **allocating** export (`input(len)`, `settings(len)`, `font_input(len)`,
  ...) returns a pointer to `len` bytes of room; the page copies its bytes
  there: `new Uint8Array(exports.memory.buffer).set(bytes, ptr)`. Take a new
  `Uint8Array` view of `exports.memory.buffer` **after every call**: memory
  grows and an old view is detached.
- Text (file names, settings, font family names) is UTF-8
  (`new TextEncoder().encode(...)`).
- Settings are `key=value` lines separated by `\n`.
- A call returning `i32` returns `-1` on failure; the reason is then
  `message_ptr()`/`message_len()` (UTF-8). After success the same message
  holds the notes (what was done, what could not be), possibly empty.
- Outputs: `count()` files; file `i` is named by
  `name_ptr(i)`/`name_len(i)` and its bytes are `data_ptr(i)`/`data_len(i)`.
  Copy them out (`.slice(ptr, ptr + len)`) before the next call.
- A Rust panic aborts the module (a `WebAssembly.RuntimeError` trap). Catch
  it in the worker, report it, and throw that instance away. Cancelling is
  terminating the worker.

The examples below use these helpers (the `put` order matters: call the
allocating export first, then take the view):

```js
const { instance } = await WebAssembly.instantiate(await (await fetch(url)).arrayBuffer(), {});
const x = instance.exports;
const mem = () => new Uint8Array(x.memory.buffer);
const enc = (s) => new TextEncoder().encode(s);
const put = (bytes, alloc) => { const ptr = alloc(bytes.length); mem().set(bytes, ptr); };
const text = (ptr, len) => new TextDecoder().decode(mem().slice(ptr, ptr + len));
const message = () => text(x.message_ptr(), x.message_len());
const take = (i) => ({ name: text(x.name_ptr(i), x.name_len(i)),
                       bytes: mem().slice(x.data_ptr(i), x.data_ptr(i) + x.data_len(i)) });
```

There are four frames. Each tool below names its frame.

| frame | tools | how it runs |
|---|---|---|
| **pages** (`shared/wasm`, `export_tool!`) | pdf-to-word, pdf-to-excel, pdf-to-powerpoint, pdf-to-html, pdf-to-markdown, pdf-to-text | one PDF; `begin`, `step` per page, `finish` |
| **bytes** (`shared/wasm`, `export_bytes_tool!`) | word-to-pdf, html-to-pdf, excel-to-pdf, powerpoint-to-pdf | named files in; one `run()` |
| **files** (`shared/files`, `export_files_tool!`) | compress-pdf, image-to-pdf, pdf-to-image, scan-to-pdf, ocr-pdf | named files in; `begin`, `step`, `finish` |
| **pdftool** (`shared/pdftool`, `export_pdf_tool!`) | unlock-pdf, repair-pdf, pdf-to-pdfa, compare-pdf, sign-pdf, redact-pdf, protect-pdf | unnamed files in, by position; one `run()` |

`ocr-pdf` in a browser does not read pages (Tesseract is a program): the
page has them read by a service and hands the words over. Its bundle does
two jobs (`tools/ocr-pdf/src/given.rs`):

- `survey=true` (and `max=N`, `pages`, `password`), one PDF: file 0 is
  `pages.tsv`, a line `pages<TAB>count`, then per chosen page
  `page<TAB>text|scan<TAB>width<TAB>height` (the page as shown, in points).
  More than `max` chosen pages is refused.
- a PDF and a file named `*.tsv` of words, one per line,
  `page<TAB>x0<TAB>y0<TAB>x1<TAB>y1<TAB>text` (points from the bottom-left of
  the page as shown, reading order; a word's following space at its end):
  file 0 is `<stem>-ocr.pdf` with them written as each page's invisible text
  layer. A file strict parsing refuses is written again first as one clean
  revision.

**Random bytes.** A browser gives the module no randomness, and writing
with AES needs it. The files and pdftool frames export `random(len)`: fill
it with 64 bytes from `crypto.getRandomValues` (at least 48) before
**every** run (pdftool: before `run()`; files: before `begin()`). The bytes
are used by that one job and wiped. Without them protect-pdf refuses, and
compress-pdf, sign-pdf and redact-pdf refuse to write an AES-protected file
(RC4 and unprotected files need none); nothing is ever written with
guessable keys. Giving them to every tool of these frames is harmless:

```js
if (x.random) put(crypto.getRandomValues(new Uint8Array(64)), x.random);
```

---

## Frame "pages": PDF to Office/HTML/Markdown/text

Exports: `memory, input, settings, font_name, font_input, font_add, begin,
step, finish, batch_add, batch_finish, count, name_ptr, name_len, data_ptr,
data_len, message_ptr, message_len`.

```js
// once per worker, before the first begin, only if needed (see Fonts):
put(enc("Noto Sans Lao"), x.font_name); put(faceBytes, x.font_input); x.font_add(); // 0 or -1

// per PDF:
put(pdfBytes, x.input);
put(enc("name=report.pdf\npages=1,3-5\npassword=secret\n"), x.settings);
const pages = x.begin();            // pages to read, or -1
for (let done = 0; done < pages; ) {
  done = x.step();                  // pages read so far, or -1; show progress here
  if (done < 0) break;
}
x.finish();                          // 0 or -1
// files: 0 is "main" (name it `<stem>.<ext>` yourself: docx, xlsx, pptx, html, md, txt);
// the others are attachments named "<stem>_files/<name>" (HTML/Markdown pictures).
```

Settings (all optional):

| key | value |
|---|---|
| `name` | the input's file name; its stem names the attachments folder (`report_files/`) and the batch entries |
| `pages` | `1,3-5` counted from 1; absent = every page |
| `password` | the password of a protected PDF (the open password; an owner-only-protected file opens without it) |
| `attachments` | folder name for attachments, instead of `<stem>_files` |

Several files, one download: after each file's `finish() == 0`, call
`batch_add()` (0 or -1) instead of copying the files out; after the last,
`batch_finish()` (0 or -1) and file 0 is `batch.zip` holding `<stem>.<ext>`
and `<stem>_files/...` for each file. Extensions: pdf-to-word `docx`,
pdf-to-excel `xlsx`, pdf-to-powerpoint `pptx`, pdf-to-html `html`,
pdf-to-markdown `md`, pdf-to-text `txt`.

The reference implementation is `web/worker.js`.

**Fonts.** Needed only to read text drawn in legacy Lao fonts (LAOFONT,
Saysettha subsets without meanings). Hand over, once per worker, before
`begin`, from `web/fonts/` (copied there from the engine's font package by
`scripts/build-wasm.sh`):

| `font_name` | file |
|---|---|
| `Noto Sans Lao` | `web/fonts/NotoSansLao-Regular.ttf` |
| `DejaVu Sans` | `web/fonts/DejaVuSans.ttf` |
| `Saysettha OT` | `web/fonts/SaysetthaOT-Regular.ttf` |

`web/worker.js` fetches them only for a PDF whose bytes mention `LAOFONT`,
`Saysettha` or `ObjStm`. Without them those pages read as the file claims.
Faces stay in the instance for every later conversion.

---

## Frame "bytes": Word, HTML, Excel, PowerPoint to PDF

Exports: `memory, input, input_name, input_add, reset, settings, font_name,
font_input, font_add, run, batch_add, batch_finish, count, name_ptr,
name_len, data_ptr, data_len, message_ptr, message_len`.

```js
// fonts first, once per worker (they stay for every later run):
for (const face of faces) {
  put(enc(face.family), x.font_name);   // the family (see below)
  put(face.bytes, x.font_input);
  if (x.font_add() < 0) console.warn(message());
}
// per job:
x.reset();                             // forget inputs of an earlier job
for (const file of files) {            // order: bytes, then name, then add
  put(file.bytes, x.input);
  put(enc(file.name), x.input_name);
  x.input_add();                       // returns how many inputs there are
}
put(enc("paper=A4\norientation=landscape\n"), x.settings);   // may be empty
const status = x.run();                // 0 or -1; one call, show a spinner
// files: every output is named already, e.g. "report.pdf" (one per document input)
```

`run()` takes the inputs (they are gone after it). Several jobs, one ZIP:
after each `run() == 0` call `batch_add()`; at the end `batch_finish()`,
then file 0 is `batch.zip` (a name met twice gets `-2`, `-3`).

Per tool:

| tool | inputs | settings |
|---|---|---|
| word-to-pdf | one or more `.docx` (each becomes `<stem>.pdf`) | none |
| html-to-pdf | one or more `.html`/`.htm`/`.xhtml` pages, plus the pictures (JPEG, PNG, GIF, SVG) and stylesheets they use as more inputs, named by the path the page uses (`img/logo.png`) or just their last part (`logo.png`) | none. Nothing is fetched from the network; a picture not handed over is left out (a note says so). SVG, inline or as a file, is drawn as vector graphics; what it uses and is not drawn (filters, masks, patterns...) is named in a note |
| excel-to-pdf | one or more `.xlsx` | `fit=width\|page\|none` (overrides the file), `paper=A4\|A3\|A5\|Letter\|Legal`, `orientation=portrait\|landscape`, `grid=1` (cell grid lines) |
| powerpoint-to-pdf | one or more `.pptx` | `hidden=1` also prints hidden slides |

**Fonts: required.** These tools lay text out and embed subsets of the faces
they are given; with no face handed over, word-to-pdf and html-to-pdf fail ("no fonts were given") and excel-to-pdf and powerpoint-to-pdf draw no text. A face is
known by the family its own `name` table gives and by the `font_name` it was
handed over as, and a bold or italic request is answered by the face whose own
OS/2 weight and slope are nearest (so hand over each style's file). Pass the
family as `font_name` when you know it (`Liberation Sans`); a file stem
(`LiberationSans-Bold`) also works.

`scripts/build-wasm.sh` copies the engine's whole font package (with its
licence files) into `web/fonts/`; serve the faces from there (nothing new is
downloaded). Which to hand over:

| need | faces (file) | size |
|---|---|---|
| Latin text, stand-ins for Calibri/Arial/Helvetica/Verdana... | `LiberationSans-{Regular,Bold,Italic,BoldItalic}.ttf` | 1.6 MB |
| Times New Roman/Cambria/Georgia... | `LiberationSerif-{Regular,Bold,Italic,BoldItalic}.ttf` | 1.5 MB |
| Courier/Consolas... | `LiberationMono-{Regular,Bold,Italic,BoldItalic}.ttf` | 1.2 MB |
| Lao | `NotoSansLao-{Regular,Bold}.ttf`, `SaysetthaOT-Regular.ttf`, `NotoSerifLao-{Regular,Bold}.ttf` | 0.5 MB |
| Thai | `NotoSansThai-{Regular,Bold}.ttf` | 0.15 MB |
| symbols, other scripts | `DejaVuSans.ttf`, `DejaVuSans-Bold.ttf` | 1.4 MB |
| Chinese/Japanese/Korean (only when the document has CJK) | `NotoSansCJKsc-Regular.otf` | 16 MB |

A minimal set that converts the test kit: Liberation Sans and Serif (8
faces), Noto Sans Lao, Noto Sans Thai, DejaVu Sans. A family the document
names and nobody handed over is set in its stand-in (Calibri -> Liberation
Sans, Cambria -> Liberation Serif, Lao -> Phetsarath OT, Saysettha OT, Noto
Sans Lao in that order), and a character no chosen face draws in any face
that draws it. Bold/italic with no such face is imitated.

Reference runners: `probes/bytes-wasm/run.js` (word, html) and
`probes/office-wasm/run.js` (excel, powerpoint; reads each face's family
from its name table, hands the regular face of a family first).

---

## Frame "files": compress, image to PDF, PDF to image, scan to PDF

Exports: `memory, input, input_name, settings, random, begin, step, finish,
count, name_ptr, name_len, data_ptr, data_len, message_ptr, message_len`.
No font exports.

```js
for (const file of files) {            // order: bytes, then its name
  put(file.bytes, x.input);            // room for one more input
  put(enc(file.name), x.input_name);   // names the input just added
}
put(enc("level=extreme\n"), x.settings);
put(crypto.getRandomValues(new Uint8Array(64)), x.random);   // before every begin
const steps = x.begin();               // steps to do, or -1 (inputs are taken)
for (let done = 0; done < steps; ) {
  done = x.step();                     // steps done so far, or -1; progress here
  if (done < 0) break;
}
x.finish();                            // 0 or -1
// files: named already (see below)
```

A new job starts with `input` again (the inputs of the last one were taken
by `begin`). There is no batch ZIP in this frame: a tool that makes several
files returns them all (zip them in the page if wanted).

| tool | inputs | settings | outputs |
|---|---|---|---|
| compress-pdf | exactly one PDF | `level=low\|recommended\|extreme` (default `recommended`); `password` for a protected file | `<stem>-compressed.pdf` (the file given, unchanged, when nothing got smaller); notes give sizes before/after. A protected file stays protected the same way; an AES-protected file needs the random bytes (see above), else it is refused at `begin` |
| image-to-pdf | JPG, PNG and GIF pictures (a GIF's first frame), in page order | `size=a4\|letter\|fit`; `orientation=auto\|portrait\|landscape`; `margin=none\|small\|big\|<points>`; `merge=true\|false` (default true: one PDF) | `<first stem>.pdf`, or `<stem>.pdf` per picture with `merge=false` |
| pdf-to-image | one or more PDFs | `mode=pages\|extract` (pages drawn, or the pictures inside); `format=jpg\|png` (default jpg for pages, png for extract); `dpi` (default 150); `quality` 1..100 (default 85); `pages=1,3-5`; `password` | one picture per page or per picture, named after the PDF's stem and the page/picture number |
| scan-to-pdf | photographs of paper (JPG/PNG) | `look=colour\|grey\|bw\|original`; `crop=true\|false` (find the page edges; default true); `quality` 1..100 (default 80); `size`, `orientation`, `margin` as image-to-pdf | `<first stem>.pdf` |

Fonts: none can be handed over. pdf-to-image draws text in the fonts the PDF
embeds; text in a font the PDF does not embed may not be drawn in the
browser (the desktop program uses the font package).

Reference runner: `probes/files-wasm/run.mjs`.

---

## Frame "pdftool": unlock, repair, PDF/A, compare, sign, redact, protect

Exports: `memory, input, settings, random, run, reset, count, name_ptr,
name_len, data_ptr, data_len, message_ptr, message_len`. No names for inputs, no font
exports, no batch.

```js
x.reset();                             // forget inputs, settings and outputs
put(pdfBytes, x.input);                // input 0; call input() once per file, in order
put(pictureBytes, x.input);            // input 1 (sign-pdf's picture, named in settings as @1)
put(enc("password=abc\nimage=@1\npages=last\n"), x.settings);
put(crypto.getRandomValues(new Uint8Array(64)), x.random);   // before every run (protect-pdf needs it)
const status = x.run();                // 0 or -1; one call
// file 0 is "main": name it yourself, e.g. `<stem>-unlocked.pdf` (compare-pdf: .html or .txt)
```

A setting whose value is `@N` names input `N` (counted from 0, in the order
`input` was called); that input is then not one of the PDFs.

| tool | inputs | settings | main output |
|---|---|---|---|
| unlock-pdf | one PDF | `password` (the open password; not needed for an owner-only file) | the PDF with no protection |
| repair-pdf | one PDF | `password` | the PDF rebuilt; notes list the repairs |
| pdf-to-pdfa | one PDF | `password` | PDF/A-2b; notes say whether it conforms and why not (fonts not embedded...) |
| compare-pdf | two PDFs: old, then new | `format=html\|txt` (default html, pictures of the pages inside); `no-pictures=1`; `dpi` (20..300, default 72); `password` (old file), `password2` (new file) | the report (`html` or `txt`) |
| sign-pdf | one PDF, and a picture when `image=@1` | the signature, exactly one of: `image=@1` (PNG/JPEG input), `draw=x,y x,y ...; x,y ...` (strokes in a 0..1 box, y up, `;` between strokes), `text=Name` (**does not work in the browser yet**: no fonts in this frame). Where: `pages=1,3-5\|last\|all` (default `last`); `at=X,Y,W,H` (points, page as shown, origin bottom left) or `position=bottom-right\|bottom-left\|bottom-centre\|top-right\|top-left\|centre` (default bottom-right) with `width` (default 160), `height`, `margin` (default 36). Look: `color=R,G,B` (0..1, default dark blue), `pen` (stroke width, default 1.6). `flatten=1`: one revision, the unsigned document is not left inside. `password` | the signed PDF, under the same protection as the input |
| redact-pdf | one PDF | at least one of: `areas=P:X0,Y0,X1,Y1; ...` (page from 1, rectangle in points on the page as shown, origin bottom left); `search=words\|other words` (every place these appear, white space ignored; `case=1` to match case) with `pages=1,3-5` (where to search, default all); `annotations=1` (apply the file's own Redact annotations; the default when nothing else is asked). `color=R,G,B` (box colour, default black). `password` | the redacted PDF (text, pictures and drawings under the boxes removed, one revision); notes say what was removed and which pages could only be redacted as pictures |
| protect-pdf | one PDF | `password` (opens the file; may be empty when `owner-password` is given), `owner-password` (allows everything; default: the same as `password`), `deny` (what a reader may not do, comma-separated: `print`, `print-high`, `copy`, `modify`, `annotate`, `forms`, `assemble`), `input-password` (when the file given is already protected). **Needs `random` before every run.** | the PDF protected with AES-256 (PDF 2.0); name it e.g. `<stem>-protected.pdf`. A note says when the permissions bind no one (no separate owner password) |

Booleans (`no-pictures`, `case`, `annotations`, `flatten`) are true for
`1`, `true`, `yes`, `on` or an empty value.

Fonts: none can be handed over (DEBT-17, DEBT-25). compare-pdf and
redact-pdf read text without the legacy-Lao reference faces (a page drawn in
LAOFONT may read wrong). A page whose text uses a font the PDF does not embed
in a clipping text mode cannot be interpreted without faces: redact-pdf
skips it and says so in a `WARNING:` note (kit page 13), and sign-pdf fails
if such a page is among `pages` (sign other pages, or use the desktop
program). A typed signature (`text=`) always fails in the browser.

Reference runner: `probes/secure/wasm-run.mjs`.

---

## Bundles and their sizes

Built by `scripts/build-wasm.sh` (`--profile wasm`) on 2026-09-25 from
master after the third merge round (protect-pdf and AES compress in the
browser); all twenty-one run the test kit under Node (the runners named
above).

| bundle | frame | raw | gzip -9 |
|---|---|---|---|
| pdf_to_text.wasm | pages | 3,083,495 | 1,312,897 |
| pdf_to_markdown.wasm | pages | 3,246,162 | 1,392,037 |
| pdf_to_html.wasm | pages | 3,246,142 | 1,391,912 |
| pdf_to_excel.wasm | pages | 3,129,788 | 1,332,871 |
| pdf_to_word.wasm | pages | 3,267,717 | 1,399,567 |
| pdf_to_powerpoint.wasm | pages | 3,265,320 | 1,398,980 |
| word_to_pdf.wasm | bytes | 1,464,586 | 756,476 |
| html_to_pdf.wasm | bytes | 1,442,494 | 748,111 |
| excel_to_pdf.wasm | bytes | 1,342,214 | 543,761 |
| powerpoint_to_pdf.wasm | bytes | 1,233,476 | 498,188 |
| compress_pdf.wasm | files | 2,563,530 | 963,029 |
| image_to_pdf.wasm | files | 3,832,036 | 1,601,335 |
| pdf_to_image.wasm | files | 2,560,321 | 965,003 |
| scan_to_pdf.wasm | files | 3,856,140 | 1,612,224 |
| unlock_pdf.wasm | pdftool | 344,004 | 149,271 |
| repair_pdf.wasm | pdftool | 327,263 | 142,612 |
| pdf_to_pdfa.wasm | pdftool | 377,784 | 163,551 |
| compare_pdf.wasm | pdftool | 3,222,802 | 1,384,685 |
| sign_pdf.wasm | pdftool | 3,857,970 | 1,610,136 |
| redact_pdf.wasm | pdftool | 4,037,881 | 1,695,553 |
| protect_pdf.wasm | pdftool | 359,238 | 154,390 |
