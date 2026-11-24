# officestress — synthetic Office load corpus + stress harness (#2531)

Test-only generator of deterministic, synthetic large files for the Office
large-file memory/load-stress lab, plus the measurement harness that drives a
workload over that corpus, records peak RSS and enforces a CI gate. The
generator is stdlib-only and always works offline; the harness degrades to an
explicit `SKIP` when no runtime is available, so it can never fabricate a PASS.

## Why it exists

Issue #2531 needs a reproducible 5–100 MB corpus of DOCX/XLSX/PPTX files with a
controlled feature mix (formulas, images, tables, charts, shared strings,
comments, tracked changes) so peak-RSS/heap/wire measurements are comparable
across runs and machines. A generator that produces different bytes on every
run makes a "10 % regression" gate meaningless — hence fixed seeds, fixed ZIP
metadata and `ZIP_STORED` members, and a manifest that carries the SHA-256.

## Feature matrix

A *feature* is a selectable family of structural elements. Each instance emits
exactly one marker (`<!-- OSF-<FEATURE>-<n> -->` in XML, or an `OSF-<FEATURE>-<n>`
field for CSV). The manifest counts those markers, and `--verify` re-counts
them from the regenerated file, so the counts are a property of the artifact.

`units` is the number of structural elements (paragraphs / rows / slides);
by default it scales with target size (`target_bytes // 16384`, clamped to
8…4096). Feature instance counts are `max(1, int(units * weight))`.

| Kind | Feature | Weight | What is emitted |
|------|---------|--------|-----------------|
| docx | `paragraphs` | 1.00 | `word/document.xml` `<w:p>` body paragraphs |
| docx | `tables` | 0.05 | `<w:tbl>` tables with seeded rows/cells |
| docx | `tracking` | 0.10 | `<w:ins>` tracked-insertion revisions |
| docx | `comments` | 0.04 | `word/comments.xml` `<w:comment>` + anchors |
| docx | `images` | 0.02 | `word/media/imageN.png` + `<w:drawing>` refs |
| docx | `charts` | 0.01 | `word/charts/chartN.xml` + `<w:drawing>` refs |
| xlsx | `cells` | 1.00 | `sheet1.xml` `<c>` numeric value cells |
| xlsx | `formulas` | 0.20 | `<c><f>` formula cells |
| xlsx | `shared_strings` | 0.30 | `xl/sharedStrings.xml` `<si>` + `t="s"` cells |
| xlsx | `tables` | 0.02 | `xl/tables/tableN.xml` `<table>` definitions |
| xlsx | `comments` | 0.04 | `xl/comments1.xml` `<comment>` notes |
| xlsx | `images` | 0.02 | `xl/media/imageN.png` + drawing parts |
| xlsx | `charts` | 0.01 | `xl/charts/chartN.xml` + `<c:chart>` |
| pptx | `slides` | 1.00 | `ppt/slides/slideN.xml` shape trees |
| pptx | `tables` | 0.05 | slide `<p:graphicFrame>` table frames |
| pptx | `comments` | 0.04 | `ppt/commentAuthors.xml` + slide frames |
| pptx | `images` | 0.04 | `ppt/media/imageN.png` + `<p:pic>` refs |
| pptx | `charts` | 0.02 | `ppt/charts/chartN.xml` + graphic frames |
| csv | `rows` | 1.00 | plain UTF-8 data rows with a marker column |
| ods | `cells` | 1.00 | ODF `content.xml` `<table:table-cell>` values |
| ods | `formulas` | 0.20 | ODF `table:formula` attributes |

Sizes are whole MB in the declared 5–100 MB policy. Seeds are non-negative
integers; the same `(kind, size, seed, features, units)` always yields the same
SHA-256. Default seed is `2531`.

## Usage

```bash
# Offline self-check (determinism, exact size, policy, feature counts).
python3 bottest/tools/officestress/officestress.py --self-test

# One file with a chosen feature mix, plus its manifest.
python3 bottest/tools/officestress/officestress.py \
  --kind xlsx --size-mb 25 --seed 2531 \
  --features cells,formulas,shared_strings,tables,charts \
  --out /tmp/opencode/officestress --manifest

# Full acceptance matrix: DOCX/XLSX/PPTX at 5/10/25/50/100 MB.
python3 bottest/tools/officestress/officestress.py --all \
  --out /tmp/opencode/officestress --manifest

# Reproducibility gate: regenerate every manifest case and compare SHA/size/counts.
python3 bottest/tools/officestress/officestress.py --verify \
  --out /tmp/opencode/officestress
```

Output and manifests go under `/tmp/opencode/` by default — never in the repo.
Exit codes: `0` success, `1` self-test/verification failure, `2` bad args/I/O.

## Manifest schema

`manifest.json` is stable (`sort_keys`, sorted cases). Top level records the
generator path, issue, `synthetic_only`, compression, policy and acceptance
sizes. Each case carries:

| Field | Meaning |
|-------|---------|
| `file` | corpus filename |
| `kind` | docx / xlsx / pptx / csv / ods |
| `size_mb`, `size_bytes` | requested and exact written size |
| `seed`, `units` | deterministic inputs |
| `features` | ordered selected feature names |
| `feature_counts` | verified per-feature instance counts |
| `sha256` | SHA-256 of the finished file |

## Determinism notes

* OOXML/ODS members are `ZIP_STORED`; deflate is avoided because its bytes are
  zlib-version dependent. A seeded `corpus/filler.bin` member inflates the
  archive to the exact target size.
* Each `ZipInfo` uses the DOS epoch (1980-01-01), `create_system=0` and fixed
  creator fields, so no host OS, clock, umask or locale leaks into the bytes.
* All synthetic text comes from a fixed vocabulary via one seeded `Random`;
  the filler uses a separately salted `Random`, so changing features does not
  perturb filler bytes. PNG payloads are real 1×1 images built with seeded RGB.
* No document text, tenant name, path or real client data is ever embedded.
  Generated files are structurally OOXML-like and intended for import/memory
  stress, not to open in Word/Excel.

## Files

| File | Role |
|------|------|
| `officestress.py` | CLI, manifest, `--verify`, `--self-test` |
| `os_matrix.py` | policy constants, feature matrix, config resolution |
| `os_builders.py` | content builders + feature-marker verification |
| `os_zip.py` | deterministic ZIP writer, sizing, SHA-256 |

## Measurement harness (`os_measure.py`)

```bash
# Offline logic checks (47 checks, no server/binary/network).
python3 bottest/tools/officestress/os_measure.py --self-test

# No runtime: emits an explicit SKIP report (exit 0, never PASS).
python3 bottest/tools/officestress/os_measure.py --run \
  --report /tmp/opencode/officestress-measure/report.json

# Hardware job: drive a real import/export and measure the server process.
python3 bottest/tools/officestress/os_measure.py --run --server-pid 1234 \
  --workload-cmd 'curl -s -X POST http://localhost:8080/api/import -F file={file}' \
  --profile laptop-4gb --report /tmp/opencode/officestress-measure/report.json

# Bounded leak soak (20 open/edit/save/close cycles by policy).
python3 bottest/tools/officestress/os_measure.py --run --soak --cycles 20 \
  --workload-cmd '<path to your workload script> {file}'

# Re-evaluate a stored report; fails closed (exit 1) on any breach.
python3 bottest/tools/officestress/os_measure.py --gate \
  --report /tmp/opencode/officestress-measure/report.json \
  --baseline /tmp/opencode/officestress-measure/baseline.json
```

`--workload-cmd` is a shell-free command template with `{file}`, `{id}`,
`{kind}`, `{size_mb}`, `{mode}` and `{iteration}` fields (parsed with `shlex`,
never a shell). Peak RSS is read from `/proc/<pid>/status` `VmHWM` when
`--server-pid` is given, otherwise from `/usr/bin/time -v` or `os.wait4`'s
`ru_maxrss`. The report records server RSS, browser heap and wire bytes in
separate fields; browser figures are only merged from a validated
`--observations` JSON (`{"cases": {"docs-large": {"browser_heap_mb": 1.5,
"wire_bytes": 2}}}`). Content/PII keys are rejected. No document text is ever
captured — workload stdout/stderr go to `DEVNULL`.

### Profiles and gate

| Profile | cgroup limit | Docs ceiling | Sheets ceiling | Slides ceiling |
|---------|-------------:|-------------:|---------------:|---------------:|
| `laptop-2gb` | 2048 MB | 450 | 600 | 550 |
| `laptop-4gb` | 4096 MB | 900 | 1200 | 1100 |
| `laptop-8gb` | 8192 MB | 1800 | 2400 | 2200 |

The gate fails on a per-case ceiling breach, a cgroup-limit breach, a workload
error/nonzero exit, document content in events, a short soak (fewer than 20
cycles), RSS not returning to baseline within 10%, session-cache growth over
16 MB, or a peak regression over 10% versus `--baseline`. With no baseline the
regression is reported `UNVERIFIED`, not PASS. A `SKIP` report always yields a
`SKIP` gate verdict; `--require-runtime` turns a SKIP into exit 2.

### What still needs hardware

The harness is infrastructure: the 2/4/8 GB cgroup run, a real server PID,
browser heap/wire observations and the full import/export workloads must be
supplied by a hardware/UAT lane. This repo lane only proves the harness logic,
the SKIP path and the gate's fail-closed behaviour.
