# Slides 🟡 PREVIEW — IN TEST

<img src="../../assets/suite/slides-screen.svg" alt="Slides editor screen" style="max-width: 100%; height: auto;">

> **In test.** Slides is a preview application and is **not part of the supported surface**. It is functional and actively being tested, but behaviour and interface may change, and edge cases are not guaranteed. Do not commit a live presentation to it without a rehearsal.

Slides is the suite's presentation editor: a canvas of slides with shape and media tools, a presenter mode with audience engagement, and version history.

## What the shipped editor exposes

Verified from the app's own toolbar and module layout (`botui/ui/suite/slides/`).

### Building slides

| Capability | Notes |
|---|---|
| New Slide | Add a slide to the deck |
| Master Slide | Define the shared layout slides inherit from |
| Add Text / Image | Text boxes and pictures |
| Add Shape | Includes rectangle, circle and triangle insert tools |
| Add Table / Add Chart | Tabular and data visualisation content |
| Rotation | Rotate elements by degrees, with a reset |
| Theme / Background / Transition / Animation | Presentation styling |
| Notes | Speaker notes per slide |

### Presenting

| Capability | Notes |
|---|---|
| Present / stop presenting | Enter and leave presentation mode |
| **Presenter View** | The presenter's own screen, showing what the audience does not see |
| **Laser pointer** | Point at content without changing it |
| Export | Produce the deck in a deliverable format |

### Audience engagement

| Capability | Notes |
|---|---|
| **Questions & answers** | Attendees ask questions; the presenter marks them answered or reopens them |
| Comments | Annotations attached to an element on a slide |
| Collaborators & follow | Who is in the deck, and following their position |
| Activity log / Version history | What changed, and the ability to go back |

## What changed in this page

Earlier revisions listed a precise feature matrix — named themes such as *Professional*, *Creative*, *Academic*, a fixed transition list, and "upload your own template" — presented as finished behaviour. Those specifics could not be verified against the shipped editor, so they have been replaced with the capability list above, which is taken from the actual toolbar and module layout.

## Opening it

Slides is a **preview** application and is additionally gated behind Preview mode as an in-test app. Turn on the **Preview** switch in the left sidebar, then open **Slides** from the app menu.

## A caution for real use

Presenting from an in-test editor carries a specific kind of risk: the failure is public. If the deck matters, rehearse it in the room and have an export as a fallback. [Export](#presenting) exists for exactly this reason.

## See Also

- [Docs](./docs.md) - Documents (in test)
- [Sheets](./sheet.md) - Spreadsheets, the most advanced preview app
- [Meet](./meet.md) - Running the meeting around the presentation
- [Apps overview](./README.md) - Stability classification for the whole suite
