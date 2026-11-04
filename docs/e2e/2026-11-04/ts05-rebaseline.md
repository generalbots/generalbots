# TS-05 rebaseline (published page is stale)

The live page (`gb7features/test-scripts.html`, baseline 2026-11-04) still
declares four gaps. Two of them shipped after that baseline:

| Declared gap | Current state | Replacement check |
|--------------|---------------|-------------------|
| Voice / live mode | Still no user-facing voice mode. The duplex **API** exists (`/v1/audio/duplex`) but there is no button or live UI. | Keep the negative check: no dead button, no help claim. |
| Locked conversations | Shipped (#1682): PIN/device auth, `423` gate, locked history listing. | Lock a conversation, verify the PIN challenge and the `423` API gate, unlock it. |
| SKILL.md packages | Shipped (#1681): import Agent Skills packages as knowledge/tools. | Import a `SKILL.md` package and verify the imported skill is listed and callable. |
| Collaborative documents | Still a gap. | Keep the absence check: no real-time co-editing/comments/presenting. |

## Proposed TS-05 text

**Title:** Declared gaps fail safe — voice and collaborative docs only.

1. Look for voice/live mode. There is no button and no help claim; the duplex
   API is not a user-facing surface.
2. Lock a conversation with a 4–8 digit PIN. Reopen it: the PIN challenge
   appears. Call a locked conversation endpoint without the unlock token and
   confirm `423`. Unlock it and confirm history returns.
3. Import a `SKILL.md` package. Confirm the skill appears in the skills list and
   its tool is callable; remove it and confirm it disappears.
4. Look for real-time co-editing, comments and presenting. They are absent;
   single-user export still works.

**Expected:** voice and collaborative docs are honest absences; locked
conversations and skill import behave as shipped features. Nothing crashes.

The external site is not in this repository, so this file is the source to copy
into the published page; the site owner must publish it.
