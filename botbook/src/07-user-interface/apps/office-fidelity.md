# Office Fidelity Contract — DOCX, PPTX, XLSX

Docs, Sheets and Slides all edit real Office files. This page states the single
contract those three formats share, so "open it, change one word, save it" never
costs the parts the editor does not model — themes, master slides and layouts,
speaker notes, charts, pivot tables, images, comments, conditional formats,
headers and footers, styles, numbering.

The contract is pinned by an offline test,
`bottest/tests/office_fidelity.rs`, which round-trips the attached fixtures
(`Rice_and_Beans_Nutrition.pptx`, `Alex_Rivera_Resume_Futuristic.docx`) and a
synthetic workbook with parts the model cannot re-emit.

## The four rules

| # | Rule | What it means in practice |
|---|------|---------------------------|
| 1 | **Originals are durable** | Every import path — upload, Drive open, chat attachment — stores the source package next to the model (Drive object for documents, the workbook snapshot on the sheet, the deck package beside the slide JSON). Fidelity is a property of the stored file, not of a process cache, so a server restart changes nothing. |
| 2 | **Save patches** | A save writes edits *into* the original package and only the affected OOXML parts change. Text goes to the owning slide/paragraph/cell part; every untouched part stays byte-identical. Saving a file you did not edit returns the original bytes. |
| 3 | **Export reuses the save path** | The `.docx` / `.pptx` / `.xlsx` export endpoints call the same preserving path as save, so a download cannot lose something a save would have kept. When the target differs from the file that was opened, the response names the file actually written. |
| 4 | **No silent fallback** | When the source package is missing or cannot be patched, the result is an explicit `regenerated` signal (`x-gb-docx-fidelity` on the DOCX export, the plan label on the sheet export) or an error that leaves the original untouched. The suite never quietly ships a rebuilt file. |

## What the editor cannot express is rejected visibly

Rule 4 also covers edits, not just packages. An edit the model cannot express is
reported with a message naming the limitation instead of being dropped:

| Attempt | Result |
|---|---|
| Add, delete or reorder slides/paragraphs, so a positional text patch cannot land | No silent patch: the save asks for an explicit rebuild, reported as `regenerated`, and the author's theme is carried over |
| Replace an image, add a chart, or edit comments/tracked changes through the text model | Not representable in the model, so the original part is kept exactly as it was — the save never rewrites it with a guess |
| Open a `.xls`, `.xlsb` or `.ods` workbook | Kept as a source snapshot but never patched with xlsx parts; save writes a named `<name>.gbsheet.xlsx` sibling and the response says which file was written |
| Patch a package that turns out to be malformed | Error returned to the caller; the original object is not overwritten |

Chat and agent edits go through the same import → patch → save pipeline, so a
bot editing a document cannot produce a fresh file behind your back either.

## Fidelity harness and QA lab (issue #2457)

Issue #2457 proposes a full fidelity lab: a licensed, de-identified corpus of
100+ files per format tagged by feature, a round-trip harness that scores every
feature structurally and visually, a harness-generated coverage matrix and a
PR/nightly dashboard with a diff gallery, a 24-hour importer fuzz run and a
four-browser smoke matrix. **That lab does not exist yet.** This page states the
contract and points at the small regressions that pin it; it reports no
measured fidelity score, and no PASS is claimed for the items marked
"Not implemented".

| #2457 capability | Status today | Evidence |
|---|---|---|
| Reproducible bug template attaching a minimal corpus file and the failing OOXML part | Present | `.github/ISSUE_TEMPLATE/office-fidelity.md` — asks for format/feature tags, a minimal de-identified file with its origin and licence, repro steps and the failing part; it cites #2457 |
| Round-trip regression: import → edit → save → per-part byte comparison | Present, narrow | `bottest/tests/office_fidelity.rs` with DOCX, PPTX and XLSX submodules. The DOCX/PPTX cases use the attached fixtures; the XLSX case builds a synthetic package carrying parts the model cannot re-emit. Each asserts an untouched save is byte-identical, one edit changes exactly one part, and an unpatchable source reports `regenerated`. Fixtures live in `bottest/tests/fixtures/office-fidelity/` (`Alex_Rivera_Resume_Futuristic.docx`, `Rice_and_Beans_Nutrition.pptx`; the directory also holds `Painel_Financeiro_Completo.xlsx`, which this test does not reference) |
| Fuzz targets that exercise the importers | Present, narrow | Detached `fuzz/` cargo-fuzz package with four targets (`jwt_claims`, `webhook_body`, `basic_compile`, `office_import`) and seeds in `fuzz/corpus/office_import/` (`sheet.csv`, `zip-magic.xlsx`); stable fallback `bottest/tests/fuzz_regressions.rs` runs 256 seeded mutations with six tiny office seeds. The office surface asserts only that a byte sequence imports or returns `Err` — a "never panics" fail-closed rule, not a fidelity score |
| Licensed/de-identified corpus of 100+ files per format, tagged by feature | Not implemented | — |
| Structural plus visual (rasterised page diff) round-trip scoring with a per-feature score | Not implemented | — |
| Feature-coverage matrix generated from the harness ("supported / degraded / dropped") | Not implemented — the four rules above are hand-written, not harness output | — |
| Fidelity dashboard on every PR and nightly with thresholds and a diff gallery | Not implemented | — |
| 24-hour importer fuzz run with zero crashes, OOMs or timeouts and resource caps | Not implemented — the deep run is a nightly job with no 24-hour target | `fuzz/Cargo.toml` |
| Cross-browser smoke matrix (Chrome, Firefox, Edge, Safari) for open/edit/save/export | Not implemented | — |

The fixtures and fuzz seeds above are small regression artifacts, not the
100+ file corpus #2457 asks for; there is currently no number that says how
faithful a save is, and no gate that stops a fidelity regression in CI.

## See Also

- [Docs](./docs.md) — documents (in test)
- [Sheets](./sheet.md) — spreadsheets, and the [Sheet Parity Plan](./sheet-parity-plan.md)
- [Slides](./slides.md) — presentations (in test)
- [Drive](./drive.md) — where the original packages are stored
