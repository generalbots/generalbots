# UAT — User Acceptance Test Script (chat + suite apps)

This is the acceptance script for the chat experience and the apps it drives. It
answers one question: **does a real person, typing a normal sentence, get the
result they asked for, in an app they can then use?**

- **100 scenarios**, ten per vertical. Every phrase is written the way a user
  types it (Portuguese and English mixed, typos included where they are part of
  the phrase).
- Each step ends in an explicit **OK** condition. A step is `PASS` only when the
  condition is observed; anything else is a failure with evidence.
- Where a scenario is **known to fail today**, it carries a `Reported` line with
  the date and the symptom. That line is the evidence to beat, not a waiver: the
  scenario still has to pass.

## How to run

1. Open the bot's chat (`https://<bot>.<platform>/`) and log in.
2. Send the phrase exactly as written. Do not "help" the bot.
3. Follow the numbered steps. Mark each `PASS` / `FAIL`, screenshot the failure
   and copy the console/network error.
4. A vertical passes when all ten scenarios pass. A release needs all ten
   verticals.
5. File one issue per failed scenario using its ID in the title
   (`UAT DOC-03: …`).

## Conventions

| Token | Meaning |
|-------|---------|
| **App** | The suite app that must open for the scenario |
| **OK** | The observable condition that makes the step pass |
| **Reported** | A defect observed on production before this script existed |
| `pt-BR` / `en` | Language the phrase is typed in |

## Index

| # | Vertical | Apps exercised |
|---|----------|----------------|
| 1 | Documents | Docs, Drive |
| 2 | Spreadsheets | Sheet, Drive |
| 3 | Presentations | Slides, Drive |
| 4 | Canvas and design | Canvas, Designer |
| 5 | Data and database | Database, Sheet |
| 6 | Media | Player, Photos, Video, Jukebox |
| 7 | Personal productivity | Notes, Goals, Calculator, Timer, Clock, Calendar, Tasks |
| 8 | Research and news | Research, Browser |
| 9 | Code and automation | Terminal, Vibe, Browser, Editor |
| 10 | Business | Mail, CRM, Tickets, Billing, Tax, Compliance, HR, KYC, POS, Store |

---

## 1. Documents — 10 scenarios

### DOC-01 · "Create a .docx sample resume very futuristic show to me" · en
**App:** Docs

1. Send the phrase as-is. — **OK** the bot replies and a Docs card opens in the chat with a title.
2. The editor shows **three A4 pages** (210 × 297 mm) with a visible page rail. — **OK**
3. The first page has a name, a headline and a contact block; pages 2–3 continue the content. — **OK**
4. Toolbar → page background → yellow: **every page** turns yellow, not just the first. — **OK**
5. Toolbar → Download → PDF: a three-page PDF downloads and opens with the same yellow pages. — **OK**
6. Close the card, reopen the document from Drive: content and yellow background are still there. — **OK**

**Reported 2026-11-09:** only the first A4 page renders; the rest of the document is invisible.

### DOC-02 · "Gera um currículo de gerente de projetos em português, 2 páginas" · pt-BR
**App:** Docs

1. Send the phrase. — **OK** a Docs card opens with a Portuguese document titled as a resume.
2. Two A4 pages, Portuguese headings ("Experiência", "Formação"). — **OK**
3. Edit a bullet in page 2, press Ctrl+B. — **OK** the bold applies only to the selection.
4. Export PDF: two pages, no clipped text at the page break. — **OK**

### DOC-03 · "Escreva um contrato de prestação de serviços com cláusulas numeradas e baixe em PDF" · pt-BR
**App:** Docs

1. Send the phrase. — **OK** the document opens with numbered clauses 1…N.
2. Page 1 shows the parties and the object of the contract. — **OK**
3. Add a clause in the middle; numbering re-flows without duplicates. — **OK**
4. Download → PDF: the numbering matches the editor exactly. — **OK**

### DOC-04 · "Create a 4-page project proposal with a table of contents and download it as PDF" · en
**App:** Docs

1. Send the phrase. — **OK** a Docs card opens.
2. A table of contents exists and its entries match the section headings. — **OK**
3. Click a TOC entry. — **OK** the editor scrolls to that heading.
4. Export PDF: four pages, TOC links intact. — **OK**

### DOC-05 · "Open my last document about the Q3 report" · en
**App:** Docs / Drive

1. Send the phrase. — **OK** the bot names the document it found (or asks which one if ambiguous).
2. The document opens in a Docs card with its saved content. — **OK**
3. No duplicate cards are created while the document loads. — **OK**

### DOC-06 · "Import this PDF and turn it into an editable document" (attach a PDF) · en
**App:** Docs

1. Attach a 3-page text PDF and send the phrase. — **OK** the bot acknowledges the attachment by name.
2. A Docs card opens with the extracted text in editable paragraphs. — **OK**
3. Text selection and formatting work on the extracted content. — **OK**

### DOC-07 · "Traduza este documento para o inglês mantendo a formatação" · pt-BR
**App:** Docs

1. Send the phrase after DOC-01 exists. — **OK** the bot targets that document.
2. The document comes back in English with headings, bold and lists preserved. — **OK**
3. Undo restores the Portuguese version. — **OK**

### DOC-08 · "Add a header with the company name and page numbers on all pages" · en
**App:** Docs

1. Send the phrase. — **OK** the bot applies a header.
2. Every page shows the header, including page 1. — **OK**
3. Page numbers read 1…N and update after inserting a page. — **OK**

### DOC-09 · "Esse texto está bom? Revise a ortografia e me mostre as mudanças" · pt-BR
**App:** Docs

1. Send the phrase with a document open. — **OK** the bot lists the corrections.
2. Accepting them changes only the flagged words. — **OK**
3. The change is undoable in one step. — **OK**

### DOC-10 · "Share this document with my team as a read-only link" · en
**App:** Docs

1. Send the phrase. — **OK** the bot produces a link.
2. Opening the link in a private window shows the document read-only (no toolbar edit). — **OK**
3. The Toolbar → Share control reflects the same state. — **OK**

---

## 2. Spreadsheets — 10 scenarios

### SHEET-01 · "Gera uma planilha com PIB do Brasil nos últimos anos e gráfico" · pt-BR
**App:** Sheet

1. Send the phrase. — **OK** a Sheet card opens with a header row (`Ano`, `PIB`).
2. At least ten data rows, one per year, with non-empty numbers. — **OK**
3. A chart is rendered next to the data and its bars/lines match the numbers. — **OK**
4. Double-click the chart → the editor opens and changing the type (bar → line) redraws it. — **OK**
5. Sum of the PIB column appears in the toolbar status when the column is selected. — **OK**
6. Download → XLSX: the file opens with data **and** chart. — **OK**

**Reported 2026-11-09:** the sheet opens, typing works, but no cell content is displayed; the chart renders but cannot be edited.

### SHEET-02 · "Create a sheet with brazilian pib data and chart" · en
Same as SHEET-01 in English; **OK** the column headers are English and the chart axis is labelled.

### SHEET-03 · "Create a spreadsheet with 5 cell each one with a different color here" · en
**App:** Sheet

1. Send the phrase. — **OK** a Sheet card opens with five filled cells.
2. The five cells have five distinct background colours, visible immediately. — **OK**
3. Clicking a cell shows its colour in the fill picker. — **OK**
4. Reload the page: colours persist. — **OK**

**Reported 2026-11-09:** the sheet opens and accepts the request but no content is shown.

### SHEET-04 · "Monte uma planilha de despesas do mês com soma e média" · pt-BR
**App:** Sheet

1. Send the phrase. — **OK** columns `Data`, `Descrição`, `Valor`.
2. A `SOMA` row and a `MÉDIA` row evaluate to numbers, not text. — **OK**
3. Type a new value in the range: both totals update. — **OK**

### SHEET-05 · "Import this CSV and give me a pivot by category" (attach CSV) · en
**App:** Sheet

1. Attach a CSV with ≥50 rows and send the phrase. — **OK** the sheet opens with the CSV content.
2. A pivot/second sheet groups by category with totals. — **OK**
3. The totals equal the sum computed by hand for one category. — **OK**

### SHEET-06 · "Make a cash-flow forecast for 12 months with conditional formatting" · en
**App:** Sheet

1. Send the phrase. — **OK** 12 month columns and a balance row.
2. Negative balances are red, positive green. — **OK**
3. Editing a month re-evaluates the balance row. — **OK**

### SHEET-07 · "Coloque uma fórmula que calcule o imposto de 15% sobre a coluna B" · pt-BR
**App:** Sheet

1. Send the phrase. — **OK** column C shows `=B*0.15` results.
2. The formula bar shows the formula (not just the value) when C is selected. — **OK**
3. Changing B updates C. — **OK**

### SHEET-08 · "Crie uma tabela de horários de aula com cores por matéria" · pt-BR
**App:** Sheet

1. Send the phrase. — **OK** a 5×6 grid (days × periods).
2. Each subject has its own colour and a legend. — **OK**

### SHEET-09 · "Merge the title across A1:E1, make it bold and 16pt" · en
**App:** Sheet

1. Send the phrase. — **OK** A1:E1 is one merged cell with bold 16pt text.
2. Unmerge from the toolbar restores five cells without losing the text. — **OK**

### SHEET-10 · "Export this sheet to CSV and to PDF" · en
**App:** Sheet

1. Send the phrase. — **OK** two files are offered.
2. CSV round-trips in a spreadsheet app with the same values. — **OK**
3. PDF shows the same columns and formatting. — **OK**

---

## 3. Presentations — 10 scenarios

### SLIDE-01 · "Crie uma apresentação de 5 slides sobre energia solar com imagens" · pt-BR
**App:** Slides

1. Send the phrase. — **OK** a Slides card opens on slide 1 of 5.
2. Each slide has a title and at least one visual element. — **OK**
3. The slide rail lists 1…5 and clicking a thumbnail navigates. — **OK**
4. Presenter mode opens and advances with the arrow keys. — **OK**
5. Export PPTX: opens with five slides and the same text. — **OK**

### SLIDE-02 · "Insert a new slide after slide 3 with a two-column layout" · en
**App:** Slides

1. Send the phrase with SLIDE-01 open. — **OK** a sixth slide appears between 3 and 4.
2. The two-column placeholder is editable and accepts text in both columns. — **OK**
3. The rail renumbers 1…6. — **OK**

**Reported 2026-11-09:** the deck opens but inserting a new slide does nothing.

### SLIDE-03 · "Troque o tema para escuro e coloque o logo na capa" · pt-BR
**App:** Slides

1. Send the phrase. — **OK** all slides switch to the dark theme.
2. The logo appears on slide 1 only, scaled without distortion. — **OK**
3. Undo returns to the previous theme in one step. — **OK**

### SLIDE-04 · "Make a 10-slide investor pitch: problem, solution, market, traction, team, ask" · en
**App:** Slides

1. Send the phrase. — **OK** ten slides with those section titles.
2. Slide 1 is a title slide with the company name. — **OK**
3. Speaker notes exist for at least three slides. — **OK**

### SLIDE-05 · "Adicione um gráfico de barras no slide 4 com os dados da planilha" · pt-BR
**App:** Slides + Sheet

1. Send the phrase with SHEET-01 available. — **OK** the chart lands on slide 4.
2. The chart values match the sheet. — **OK**
3. Changing the sheet updates the slide after a refresh. — **OK**

### SLIDE-06 · "Export the deck as PDF with one slide per page" · en
**App:** Slides

1. Send the phrase. — **OK** the PDF downloads.
2. Page count equals slide count, one slide per page, no clipping. — **OK**

### SLIDE-07 · "Coloque uma transição de fade entre todos os slides" · pt-BR
**App:** Slides

1. Send the phrase. — **OK** the transition is set for all slides.
2. Presenter mode applies the fade between slides. — **OK**

### SLIDE-08 · "Duplicate slide 2 and change the title to Roadmap" · en
**App:** Slides

1. Send the phrase. — **OK** a new slide 3 is a copy of 2 with the new title.
2. Images and layout are preserved in the copy. — **OK**

### SLIDE-09 · "Start the presentation and show me the presenter view" · en
**App:** Slides

1. Send the phrase. — **OK** presenter view opens with current slide, next slide and notes.
2. The timer runs and the current slide number is visible. — **OK**

### SLIDE-10 · "Traduza a apresentação para inglês mantendo o layout" · pt-BR
**App:** Slides

1. Send the phrase with SLIDE-01 open. — **OK** text becomes English, layout unchanged.
2. Charts and images stay in place. — **OK**

---

## 4. Canvas and design — 10 scenarios

### CANVAS-01 · "Gere um wireframe de site de restaurante em canvas" · pt-BR
**App:** Canvas

1. Send the phrase. — **OK** a Canvas card opens with hero, menu and contact blocks.
2. Blocks are selectable and movable. — **OK**
3. Export PNG produces a readable image at 1× and 2×. — **OK**

### CANVAS-02 · "Design a landing page mockup and give me a .fig file" · en
**App:** Canvas / Designer

1. Send the phrase. — **OK** the frame is drawn in the design canvas.
2. A `.fig` file is offered for download (generated with a real library, not a renamed image). — **OK**
3. Re-importing the `.fig` restores the frames and layers. — **OK**

### CANVAS-03 · "Crie um fluxograma de aprovação de compras" · pt-BR
**App:** Canvas

1. Send the phrase. — **OK** nodes and arrows are drawn left-to-right.
2. Adding a node keeps the arrows connected. — **OK**
3. Export SVG keeps the text selectable. — **OK**

### CANVAS-04 · "Open the canvas and draw a red circle with a blue label below it" · en
**App:** Canvas

1. Send the phrase. — **OK** the circle and label appear.
2. Selecting the circle shows its fill colour in the toolbar. — **OK**
3. Changing the fill updates only the circle. — **OK**

### CANVAS-05 · "Share this canvas so a teammate can edit it with me" · en
**App:** Canvas

1. Send the phrase. — **OK** the toolbar's Share control reports the connection state (not a static hint).
2. In a second browser, moving an element is reflected on the first within ~2 s. — **OK**

**Reported 2026-11-09:** Share only shows "Real-time collaboration needs a canvas session…".

### CANVAS-06 · "Gere um organograma da minha equipe de 12 pessoas" · pt-BR
**App:** Canvas

1. Send the phrase. — **OK** a three-level tree is drawn.
2. Names are legible at default zoom; zoom 200% keeps them sharp. — **OK**

### CANVAS-07 · "Turn this screenshot into an editable wireframe" (attach PNG) · en
**App:** Canvas

1. Attach a screenshot and send the phrase. — **OK** the canvas shows shapes over the layout.
2. Moving a shape does not move the background image. — **OK**

### CANVAS-08 · "Crie um ícone de carrinho de compras e exporte em SVG" · pt-BR
**App:** Canvas

1. Send the phrase. — **OK** the icon is drawn and selected.
2. The SVG downloads and opens with paths (not embedded bitmap). — **OK**

### CANVAS-09 · "Desenhe um mapa mental com 6 ramos sobre marketing digital" · pt-BR
**App:** Canvas

1. Send the phrase. — **OK** a central node with six labelled branches.
2. Adding a seventh branch re-balances the layout without overlapping. — **OK**

### CANVAS-10 · "Undo the last three changes and export the canvas as PDF" · en
**App:** Canvas

1. Send the phrase after several edits. — **OK** three undos apply in order.
2. The PDF matches what is on screen. — **OK**

---

## 5. Data and database — 10 scenarios

### DB-01 · "Show me the tables in the database and the row counts" · en
**App:** Database

1. Send the phrase. — **OK** a table list with counts appears.
2. Clicking a table opens the first rows. — **OK**
3. Sorting a column reorders the rows client-side. — **OK**

### DB-02 · "Importe este CSV e crie a tabela clientes com tipos corretos" (attach CSV) · pt-BR
**App:** Database

1. Attach the CSV and send the phrase. — **OK** the table is created and listed.
2. Column types are inferred (dates as dates, money as numeric). — **OK**
3. Row count matches the CSV. — **OK**

### DB-03 · "Quais foram os 10 produtos mais vendidos no último mês?" · pt-BR
**App:** Database

1. Send the phrase. — **OK** the answer is a table with product and quantity.
2. The SQL/query used is shown on request. — **OK**

### DB-04 · "Create a dashboard with sales by month and top 5 customers" · en
**App:** Database

1. Send the phrase. — **OK** two visual blocks render with real numbers.
2. Changing the date range refreshes both blocks. — **OK**

### DB-05 · "Descreva o esquema das tabelas de pedidos e mostre o diagrama" · pt-BR
**App:** Database

1. Send the phrase. — **OK** an ER diagram shows `pedidos` with its relations.
2. Foreign keys are labelled with the target table. — **OK**

### DB-06 · "Export the customers table to XLSX" · en
**App:** Database

1. Send the phrase. — **OK** the file downloads with a header row.
2. Row count equals the table count. — **OK**

### DB-07 · "Apague os registros de teste criados hoje" · pt-BR
**App:** Database

1. Send the phrase. — **OK** the bot shows what will be deleted and asks for confirmation.
2. Confirming deletes only the matching rows and reports the count. — **OK**

### DB-08 · "Quanto vendemos por estado em 2025?" · pt-BR
**App:** Database

1. Send the phrase. — **OK** a table or chart grouped by state.
2. The total equals the sum of the groups. — **OK**

### DB-09 · "Create an index on customer email and tell me if it helps the query" · en
**App:** Database

1. Send the phrase. — **OK** the index is created and the query plan before/after is shown.
2. A rollback option is offered. — **OK**

### DB-10 · "Gere um relatório em planilha com os KPIs do mês" · pt-BR
**App:** Database + Sheet

1. Send the phrase. — **OK** a Sheet card opens with the KPI table.
2. Each KPI has a value and a comparison against the previous month. — **OK**

## 6. Media — 10 scenarios

### MEDIA-01 · "Me gere um mp3 de uma música calma de piano, 30 segundos" · pt-BR
**App:** Player

1. Send the phrase. — **OK** the bot reports the file and a Player card appears in the chat.
2. Press play: audio starts, the progress bar moves, the timer counts 30 s. — **OK**
3. The card offers **Download**; the downloaded file is a valid `.mp3` (`ffprobe` reads it). — **OK**
4. "Open in player" from the card keeps playing the same track. — **OK**

### MEDIA-02 · "Generate a 20-second mp3 of ocean waves and play it here" · en
**App:** Player

1. Send the phrase. — **OK** a Player card appears with the generated track.
2. It plays without leaving the chat; volume control works. — **OK**
3. Closing the card stops playback (no ghost audio). — **OK**

### MEDIA-03 · "Notícias do Lula" · pt-BR
**App:** Research

1. Send the phrase. — **OK** the answer is a list of recent items, each with a source and a date.
2. Opening the first source shows an article that matches the headline. — **OK**
3. The bot states the retrieval date; nothing is presented as live if it is cached. — **OK**

### MEDIA-04 · "Me dá as capas dos jornais brasil247 e o globo hoje" · pt-BR
**App:** Research / Photos

1. Send the phrase. — **OK** two images are shown with the outlet name and the date.
2. Clicking an image opens it full size with its source URL. — **OK**
3. If today's cover is not published yet, the bot says so instead of showing yesterday's. — **OK**

### MEDIA-05 · "Edite esta foto: fundo desfocado e brilho +20%" (attach JPG) · pt-BR
**App:** Photos

1. Attach the photo and send the phrase. — **OK** an edited preview appears next to the original.
2. The blur respects the subject outline (no halo around the face). — **OK**
3. Download keeps the original resolution. — **OK**

### MEDIA-06 · "Create a 15-second video from these 5 photos with music" (attach 5 JPG) · en
**App:** Video

1. Attach the photos and send the phrase. — **OK** a Video card shows the render progress.
2. The result plays in the card and downloads as `.mp4`. — **OK**
3. Duration is ~15 s and the music is audible. — **OK**

### MEDIA-07 · "Faça um resumo em áudio deste documento" (attach PDF) · pt-BR
**App:** Player

1. Attach the PDF and send the phrase. — **OK** an audio track is produced (not a text reply).
2. The summary covers the document's main points. — **OK**
3. Download and playback both work. — **OK**

### MEDIA-08 · "Crie uma playlist com 5 músicas relaxantes e toque a primeira" · pt-BR
**App:** Jukebox

1. Send the phrase. — **OK** five tracks are listed with play buttons.
2. Play starts track 1; next/previous move through the list. — **OK**

### MEDIA-09 · "Convert this audio to 128 kbps mono and give me the file" (attach MP3) · en
**App:** Player

1. Attach the audio and send the phrase. — **OK** the converted file is offered.
2. `ffprobe` reports 128 kbps, mono. — **OK**

### MEDIA-10 · "Mostre minhas fotos de viagem e crie um álbum com as melhores" · pt-BR
**App:** Photos

1. Send the phrase. — **OK** a grid of photos appears from Drive.
2. An album is created and opening it shows the selected photos only. — **OK**

---

## 7. Personal productivity — 10 scenarios

### NOTE-01 · "Crie uma nota adesiva amarela com 'comprar café'" · pt-BR
**App:** Notes

1. Send the phrase. — **OK** a yellow sticky note appears with the text.
2. Drag it to another position and reload: position persists. — **OK**
3. Edit the text; the change is saved without a separate "save" click. — **OK**

### NOTE-02 · "Add a sticky note: call the accountant on Friday" · en
**App:** Notes

1. Send the phrase. — **OK** the note exists with that text.
2. The note is visible after closing and reopening the app. — **OK**

### GOALS-01 · "Defina uma meta de correr 30 km por mês e acompanhe" · pt-BR
**App:** Goals

1. Send the phrase. — **OK** the goal exists with target 30 and current 0.
2. Register 5 km: the progress bar shows 5/30 (16%). — **OK**

**Reported 2026-11-09:** creating an objective opens the browser `prompt()` and pressing OK does nothing — the HTMX form posts form-encoded to a JSON-only endpoint (415) and never refreshes the list (issue #2032). The period is also frozen at 2025 while the current date is 2026.

### GOALS-02 · "Create a goal to read 12 books this year" · en
**App:** Goals

1. Send the phrase. — **OK** target 12, unit "books".
2. Mark one book done: progress 1/12 and the percentage updates. — **OK**

### CALC-01 · "Quanto é 17,5% de 4.320?" · pt-BR
**App:** Calculator (inline)

1. Send the phrase. — **OK** the answer is 756.
2. The calculation is shown step by step, not only the result. — **OK**

### CALC-02 · "Open the calculator and split 480 by 7 people" · en
**App:** Calculator

1. Send the phrase. — **OK** the Calculator app opens with 480 ÷ 7.
2. The display shows 68.57 and the history shows the operation. — **OK**

### TIMER-01 · "Coloque um timer de 5 minutos e me avise" · pt-BR
**App:** Timer

1. Send the phrase. — **OK** a Timer card counts down from 5:00.
2. At zero the bot posts a message/notification. — **OK**
3. Pause and resume keep the remaining time. — **OK**

### CLOCK-01 · "Que horas são em Tóquio agora?" · pt-BR
**App:** Clock

1. Send the phrase. — **OK** the time is correct for Asia/Tokyo (checked against a reference).
2. The bot states the timezone and the difference to the user's zone. — **OK**

### CALENDAR-01 · "Marque uma reunião com o time amanhã às 15h por 45 minutos" · pt-BR
**App:** Calendar

1. Send the phrase. — **OK** the bot confirms date, time and duration before saving.
2. Confirming creates one event, visible in the calendar view. — **OK**
3. The event has a 45-minute duration, not a default hour. — **OK**

### TASK-01 · "Create a task list for launching the site: content, design, QA, deploy" · en
**App:** Tasks

1. Send the phrase. — **OK** four tasks appear in order.
2. Checking one moves it to done and the counter updates. — **OK**
3. Deleting a task asks for confirmation. — **OK**

---

## 8. Research and news — 10 scenarios

### RES-01 · "Pesquise os preços de passagens Rio–Lisboa em março e compare" · pt-BR
**App:** Research

1. Send the phrase. — **OK** at least three options with price, airline and date.
2. Each price links to its source page. — **OK**
3. The bot states the search date and does not invent prices when a source is missing. — **OK**

### RES-02 · "What changed in the EU AI Act in the last 30 days?" · en
**App:** Research

1. Send the phrase. — **OK** a dated summary with sources from the last 30 days.
2. Each claim cites a link; opening it contains the quoted text. — **OK**

### RES-03 · "Quem é o prefeito de São Paulo e qual o orçamento de 2026?" · pt-BR
**App:** Research

1. Send the phrase. — **OK** the name and the budget figure with a source.
2. The bot flags if the figure is a proposal rather than the approved budget. — **OK**

### RES-04 · "Compare o PIB do Brasil e do México nos últimos 5 anos" · pt-BR
**App:** Research + Sheet

1. Send the phrase. — **OK** a comparison table with five years for both countries.
2. The bot offers to open it as a sheet with a chart. — **OK**
3. Accepting produces a Sheet with the same numbers. — **OK**

### RES-05 · "Find 5 open-source Rust libraries for PDF generation and compare licenses" · en
**App:** Research

1. Send the phrase. — **OK** five libraries with repository link, stars and licence.
2. Each link resolves to the real repository. — **OK**

### RES-06 · "Abra o site do G1 e me mostre as manchetes de economia" · pt-BR
**App:** Browser

1. Send the phrase. — **OK** the browser opens g1.globo.com and lists economy headlines.
2. The headlines correspond to what the page shows at that moment. — **OK**

### RES-07 · "Summarize this page" (with a URL) · en
**App:** Research

1. Send the phrase with a news URL. — **OK** the summary names the page and its publication date.
2. The summary does not add facts that are absent from the page. — **OK**

### RES-08 · "Quais foram os resultados do último Censo do IBGE para Natal?" · pt-BR
**App:** Research

1. Send the phrase. — **OK** population and year with an ibge.gov.br source.
2. The number matches the source table. — **OK**

### RES-09 · "Monitor the price of the iPhone 17 and warn me if it drops 10%" · en
**App:** Automations (research)

1. Send the phrase. — **OK** the bot states the current price, the threshold and when it will check.
2. The automation appears in the list with the next run time. — **OK**

### RES-10 · "Explique as notícias de hoje sobre inteligência artificial em 5 bullets" · pt-BR
**App:** Research

1. Send the phrase. — **OK** five bullets, each with a source and a date.
2. Opening the sources confirms the claims. — **OK**

---

## 9. Code and automation — 10 scenarios

### CODE-01 · "Crie um script em Python que baixa as cotações do dólar e salva em CSV" · pt-BR
**App:** Editor / Terminal

1. Send the phrase. — **OK** a code block appears and an Editor card opens with the script.
2. Running it in the terminal produces `cotacoes.csv` with at least one row. — **OK**
3. The file is offered for download. — **OK**

### CODE-02 · "Build a landing page for my bakery with Node and deploy a preview" · en
**App:** Vibe

1. Send the phrase. — **OK** the bot reports the stack and shows a preview window.
2. The preview renders the page (not a blank frame) and the CSS loads. — **OK**
3. The preview URL stays reachable after closing and reopening the card. — **OK**

### CODE-03 · "Leia este CSV e gere um gráfico com matplotlib" (attach CSV) · pt-BR
**App:** Terminal

1. Attach the CSV and send the phrase. — **OK** the script runs and a PNG is produced.
2. The PNG is displayed in the chat and downloadable. — **OK**

### CODE-04 · "Automate a daily 8am email with the sales total" · en
**App:** Automations

1. Send the phrase. — **OK** the automation is created with the schedule 08:00 and the recipients shown for confirmation.
2. Confirming lists it with the next run time. — **OK**
3. "Run now" produces the email preview. — **OK**

### CODE-05 · "Abra um navegador e faça login no meu e-mail de teste" · pt-BR
**App:** Browser (computer as a service)

1. Send the phrase. — **OK** the browser window opens on the login page.
2. Typing and clicking happen in the live browser, visible to the user. — **OK**
3. Credentials are never echoed in the chat history. — **OK**

### CODE-06 · "Extraia a tabela desta página e me dê um JSON" (URL) · pt-BR
**App:** Browser / Terminal

1. Send the phrase with a URL that contains a table. — **OK** the JSON matches the table rows and headers.
2. Row count equals the page's row count. — **OK**

### CODE-07 · "Write a Node script that resizes every image in this folder" (attach folder) · en
**App:** Terminal

1. Attach the folder and send the phrase. — **OK** the script is shown and executed.
2. Every output image has the requested dimensions. — **OK**

### CODE-08 · "Crie uma planilha a partir deste JSON" (attach JSON) · pt-BR
**App:** Sheet

1. Attach the JSON and send the phrase. — **OK** a Sheet opens with one column per key.
2. Nested objects are flattened with a readable header. — **OK**

### CODE-09 · "Converta estas planilhas em um site HTML estático" (attach XLSX) · pt-BR
**App:** Vibe

1. Attach the file and send the phrase. — **OK** a site preview is produced.
2. The tables on the site match the spreadsheet values. — **OK**

### CODE-10 · "Agende um backup diário do Drive às 2h" · pt-BR
**App:** Automations

1. Send the phrase. — **OK** the automation appears with the cron 02:00 and the target.
2. Running it manually produces a backup artefact listed in the run history. — **OK**

---

## 10. Business — 10 scenarios

### BIZ-01 · "Adicione o cliente João Silva, e-mail joao@exemplo.com e telefone" · pt-BR
**App:** CRM

1. Send the phrase (phone omitted on purpose). — **OK** the bot asks only for the missing field.
2. Confirming creates one contact with the given data. — **OK**
3. Searching "João" finds it. — **OK**

### BIZ-02 · "Open a support ticket for customer 42: login fails after password reset" · en
**App:** Tickets

1. Send the phrase. — **OK** a ticket is created with a title, the customer and the description.
2. The ticket appears in the list with status open and a priority. — **OK**

### BIZ-03 · "Envie e-mail para todos os clientes do plano ouro avisando do novo preço" · pt-BR
**App:** Mail

1. Send the phrase. — **OK** the bot shows the recipient count and the draft before sending.
2. Confirming sends only after explicit approval; the campaign is listed as sent. — **OK**
3. No duplicate email per contact. — **OK**

### BIZ-04 · "Quanto faturamos este mês e qual a variação sobre o mês passado?" · pt-BR
**App:** Billing / Analytics

1. Send the phrase. — **OK** the revenue figure and the percentage variation, with the period stated.
2. The number matches the Billing dashboard. — **OK**

### BIZ-05 · "Gerar uma NF-e de teste para o cliente 42 no valor de R$ 350" · pt-BR
**App:** Tax

1. Send the phrase. — **OK** the bot shows the invoice preview with the tax breakdown.
2. In test mode the document is marked as test and never sent to the tax authority. — **OK**

### BIZ-06 · "What is our compliance score and what is failing?" · en
**App:** Compliance

1. Send the phrase. — **OK** the score plus the failing checks, each with its control id.
2. The list matches the Checks tab of the Compliance app. — **OK**

### BIZ-07 · "Onboard Maria as a developer: contract, laptop request and access" · en
**App:** HR

1. Send the phrase. — **OK** the bot lists the three items and asks for confirmation.
2. Confirming creates the onboarding record with the three steps pending. — **OK**

### BIZ-08 · "Verifique a identidade deste cliente e o score de risco" (attach document) · pt-BR
**App:** KYC

1. Attach an ID image and send the phrase. — **OK** the extracted name and document number are shown for confirmation.
2. The risk score appears with the reasons, and nothing is auto-approved without confirmation. — **OK**

### BIZ-09 · "Abra o caixa, registre R$ 89,90 em 2 itens e feche com cartão" · pt-BR
**App:** POS

1. Send the phrase. — **OK** the POS shows two line items totalling R$ 89,90.
2. Closing with card produces a receipt and a cash-register entry for the day. — **OK**

### BIZ-10 · "Coloque o plano privado em promoção de 20% por 7 dias" · pt-BR
**App:** Store

1. Send the phrase. — **OK** the product shows the promo price and the end date.
2. The public store page reflects the same price while the promo lasts. — **OK**
3. After the end date the original price returns (check the scheduled job). — **OK**

---

## Recording a run

Copy this block per scenario into the issue or a spreadsheet:

```
Scenario: DOC-01
Build:      <botserver commit / date>
Browser:    <name + version>
Result:     PASS | FAIL at step N
Evidence:   <screenshot / console / network>
Notes:      <what the bot actually did>
```

A vertical is green only when its ten rows are PASS. Failures are filed as
`UAT <ID>: <one-line symptom>` and link back to this file.

