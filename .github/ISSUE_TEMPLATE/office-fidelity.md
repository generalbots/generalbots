---
name: Office fidelity (DOCX/PPTX/XLSX) corpus bug
about: Report an import/edit/export fidelity failure with a minimal corpus file and the failing part
title: "[office-fidelity] <format>: <short description>"
labels: ["enhancement", "area:office"]
---

## Summary

<!-- One sentence: what degraded or broke (e.g. chart flattened to text, pivot lost, borders reset). -->

## Format and feature tags

- **Format:** <!-- DOCX | PPTX | XLSX -->
- **App/crate:** <!-- Docs / Slides / Sheets; botdocs / botslides / botsheet(-core) if known -->
- **Feature tags (mark all that apply):** tables | charts | pivots | masters | themes | footnotes | endnotes | comments | tracked-changes | forms | embedded-objects | fonts | images | styles | numbering | headers-footers | conditional-formats | formulas | other: <!-- specify -->

## Minimal corpus file (required)

- **Attached file:** <!-- attach the smallest de-identified file that reproduces the issue; do not attach customer/confidential data -->
- **Source/origin:** <!-- e.g. synthetic, licensed/de-identified corpus sample, exported from Word/PowerPoint/Excel/LibreOffice/Google -->
- **License/redistribution note:** <!-- confirm the file may be stored in a test corpus/repo, or state restrictions -->

> If the original file cannot be shared, attach a reduced/sanitized reproducer and describe how it was derived.

## Reproduction steps

1. <!-- e.g. Import/open the attached file in Docs/Slides/Sheets -->
2. <!-- e.g. Edit: <exact scripted change, keep minimal> -->
3. <!-- e.g. Save and/or Export as DOCX/PPTX/XLSX -->
4. <!-- e.g. Reopen in General Bots and/or in <reference app/version> -->

## Expected vs actual

- **Expected (faithful behavior):** <!-- what should be preserved/rendered -->
- **Actual (degradation/failure):** <!-- what was lost, flattened, or changed -->

## Failing part / evidence

- **Failing OOXML part(s) / path(s):** <!-- e.g. ppt/slides/slide1.xml, xl/pivotTables/pivotTable1.xml, word/document.xml; include before/after snippets if possible -->
- **Structural/visual diff:** <!-- describe or attach comparison (raster diff, part diff, screenshots) -->
- **Silent degradation?** <!-- yes/no — was the loss surfaced in UI/export messaging? -->
- **Screenshot(s) / export artifact:** <!-- attach if helpful -->

## Environment

- **General Bots version/commit:** <!-- e.g. commit SHA or release tag -->
- **Build/profile:** <!-- debug/release, desktop-local/web, feature flags if relevant -->
- **OS / browser:** <!-- e.g. Ubuntu 24.04 + Chrome 130; Windows 11 + Edge; macOS + Safari -->
- **Reference app used for comparison:** <!-- e.g. Microsoft Word 365 16.x, Excel, PowerPoint, LibreOffice 24.x, Google Docs/Sheets/Slides -->

## Impact

- **Severity:** <!-- blocks workflow / incorrect output / cosmetic -->
- **Data/content loss risk:** <!-- none / recoverable / potential loss -->
- **Notes:** <!-- workarounds, related issues (#2457 corpus/harness, app-specific fidelity issues) -->
