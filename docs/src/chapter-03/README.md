## gbkb Reference
The knowledge‑base package provides three main commands:

- **ADD_KB** – Create a new vector collection.
- **SET_KB** – Switch the active collection for the current session.
- **ADD_WEBSITE** – Crawl a website and add its pages to the active collection.

**Example:**
```bas
ADD_KB "support_docs"
SET_KB "support_docs"
ADD_WEBSITE "https://docs.generalbots.com"
```
These commands are implemented in the Rust code under `src/kb/` and exposed to BASIC scripts via the engine.
