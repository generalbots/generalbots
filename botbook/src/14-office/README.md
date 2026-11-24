# Office in the Suite 🟡 PREVIEW — IN TEST

General Bots ships three office applications in the Suite: **Docs** for documents, **Sheet** for spreadsheets and **Slides** for presentations. All three are preview applications: they work, but they are not part of the supported surface, and their behaviour and interface can change. The detailed per-app capability lists live in [Docs](./../07-user-interface/apps/docs.md), [Sheet](./../07-user-interface/apps/sheet.md) and [Slides](./../07-user-interface/apps/slides.md). This chapter describes the path that connects them: asking the assistant for a file, opening the result in an editor, and getting a downloadable file back.

> **In test.** Do not use these apps as the only copy of a document you cannot afford to lose. Some of the controls described in the app chapters are browser-only previews and are labelled as such in the UI.

## The three applications

| App | What it creates | Where the result goes |
|---|---|---|
| Docs | A rich-text document, started blank or from a letter, meeting or report template | A document that the Docs editor can open and later export |
| Sheet | A workbook with cells, formulas, formatting, conditional rules and charts | A spreadsheet stored in the workspace, openable by id or from Drive |
| Slides | A deck of slides with text and media | A presentation that the Slides editor can open and present |

Files belong to the workspace. A document or spreadsheet is not a chat transcript: once created it has its own id, can be reopened later and can be opened from Drive as well as from the chat reply.

## Two ways in

### From chat

The assistant can create and edit office files by calling the office verbs in the command catalog. Ask in ordinary language — "create a spreadsheet with this data", "write a formal letter", "make a slide deck" — and the assistant calls the matching verb. The verb carries a deep link to the app; when the reply is rendered in the web desktop, that link becomes an **open** button. See [Creating office files from chat](./from-chat.md).

### From the Suite

Turn on **Preview** mode and open Docs, Sheet or Slides from the app menu. You can also open an existing file from Drive: Drive passes the object key and bucket to the editor, which loads that file. Both editors accept the same parameters through the desktop deep-link mechanism, so a file opened from chat and a file opened from Drive land in the same place. See [Working in the editors](./editors.md).

## What you can do today

- Create a workbook from rows sent by chat, including formulas, number formats, cell styles, conditional rules and a chart in the same operation.
- Create a document from pasted content or from a formal letter, meeting notes or report template.
- Open the created file in its editor, edit cells or prose directly, and keep working in the app.
- Define data-validation rules on a spreadsheet range and see invalid cells highlighted.
- Reorder a presentation from a slide sorter panel or work from a document outline.
- Export documents to DOCX/ODT/PDF/HTML/Markdown/text and spreadsheets to XLSX/CSV/PDF and other formats, from the toolbar or from chat.

## What is not available yet

Realtime presence — seeing cursors and typing indicators — is separate from true co-editing. There is no supported workflow today for several people to edit the same document or workbook concurrently with conflict-free merging and per-user undo; concurrent cell writes use transport-level last-write-wins. Enterprise controls such as protected ranges, range-level ACLs, classification labels, DLP, retention and scheduled delivery are shown as browser-only policy previews with no server enforcement. These are current limitations of the shipped preview, not settings you can enable. See [Current limitations](./current-limitations.md).

## Chapter contents

- [Creating office files from chat](./from-chat.md) — the verbs, the reply buttons and exports from the conversation.
- [Working in the editors](./editors.md) — opening files, editing cells, validation, outlines and sorting, and the export menus.
- [Current limitations](./current-limitations.md) — what the preview does not do today.

## See Also

- [Docs - AI Writing](./../07-user-interface/apps/docs.md)
- [Sheet - Spreadsheets](./../07-user-interface/apps/sheet.md)
- [Slides - Presentations](./../07-user-interface/apps/slides.md)
- [Office Fidelity Contract](./../07-user-interface/apps/office-fidelity.md)
- [Drive - File Management](./../07-user-interface/apps/drive.md)
