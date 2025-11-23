# CREATE SITE Keyword

**Syntax**

```
CREATE SITE "alias", "template-dir", "prompt"
```

**Parameters**

- `"alias"` – Name of the new site (used as a folder name under the configured site path).
- `"template-dir"` – Relative path to a directory containing HTML template files that will be combined.
- `"prompt"` – Text prompt sent to the LLM to generate the final site content.

**Description**

`CREATE SITE` generates a new static website based on existing HTML templates and an LLM‑generated prompt. The keyword performs the following steps:

1. Creates a directory for the new site at `<site_path>/<alias>`.
2. Reads all `.html` files from `<site_path>/<template-dir>` and concatenates their contents, separating each with a clear delimiter.
3. Constructs a prompt that includes the combined template content and the user‑provided `prompt`.
4. Sends the prompt to the configured LLM provider (`utils::call_llm`) and receives generated HTML.
5. Writes the LLM output to `<site_path>/<alias>/index.html`.

The resulting site can be served directly from the `site_path` directory. Errors during directory creation, file reading, or LLM generation are logged and returned as error messages.

**Example**

```basic
CREATE SITE "my_blog", "templates/blog", "Generate a modern blog homepage for a tech writer."
TALK "Site created at /my_blog. Access it via the web server."
```

After execution, a folder `my_blog` is created with an `index.html` containing the LLM‑generated page, ready to be served.
