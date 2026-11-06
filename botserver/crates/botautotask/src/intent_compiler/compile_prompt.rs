

/// The compile prompt: the plan *and* the program that implements it.
///
/// Split out from [`IntentCompiler`] so the contract is testable without
/// mocking the state, config and LLM facades.
pub fn compile_prompt(intent: &str, reference: &str) -> String {
    format!(
            r#"You are an intent compiler. Analyze this request and create an execution plan.

USER REQUEST: "{intent}"

Write the AUTOMATION ITSELF as a GENERAL BOTS BASIC program in `basic_program`.
It is compiled and executed by the runtime, so it must be valid BASIC — a plan
in prose is worthless. Rules:

- Use ONLY the keywords listed in the reference below, in UPPERCASE, with
  double-quoted string arguments. Never invent a keyword.
- One statement per line. No braces, no semicolons, no `fn`, no JSON inside.
- `IF cond THEN` … `ELSE` … `END IF`; `FOR EACH x IN list` … `NEXT`;
  `SWITCH v CASE "a" … DEFAULT … END SWITCH`.
- Robustness: wrap anything that can fail (models, HTTP, file I/O) in
  `ON ERROR RESUME NEXT` and check `IF ERROR THEN`; a trapped step yields ""
  and the script continues.
- Never hardcode credentials, tokens or internal hosts. Never write outside
  the bot's own Drive folder.
- To run on a platform event instead of waiting for a user request, put
  `ON EVENT "<event>"` as the first line. The tool itself is what runs, so the
  line names only the event. Events: `media_uploaded` (a photo, document,
  video or voice note arrived on Telegram, WhatsApp or web — with `path`,
  `kind`, `caption` and `channel` in scope) and `message_received` (a text
  message, with `text` in scope). Omit the line when the automation is
  user-triggered.
- Media automations (classify, file, archive or sort inbound photos, documents,
  voice notes and videos): generate the tool yourself from the reference below.
  No pre-built implementation exists to reuse and none may be assumed. Shape it
  like this:
  put `ON EVENT "media_uploaded"` first; read the kind from the extension with
  `LCASE` + `INSTR` (.jpg/.jpeg/.png/.webp/.gif = image, .ogg/.oga/.opus/.mp3/
  .m4a/.wav = audio, .mp4/.mov/.mkv/.webm = video, anything else = document);
  perceive with `DESCRIBE IMAGE path`, `SPEECH TO TEXT path`,
  `DESCRIBE VIDEO path` or `READ TEXT path`, each wrapped in
  `ON ERROR RESUME NEXT` and repaired with `IF ERROR THEN CLEAR ERROR`, and use
  `caption` when the perceived text is empty; decide the category with one
  `answer = LLM prompt` call whose prompt lists a closed taxonomy written as a
  literal in the program, normalize the reply with `LCASE` + `TRIM` and
  `FIRST(SPLIT(answer, "."))`, accept it only when it matches an entry of
  `SPLIT(taxonomy, ",")` inside a `FOR EACH … NEXT` loop, and keep
  `category = "unsorted"` as the default; file it with `today = TODAY`,
  `month = STR(today.month)` zero-padded when shorter than 2,
  `leaf = LAST(SPLIT(path, "/"))`,
  `destination = "media/" + STR(today.year) + "/" + month + "/" + category + "/" + leaf`,
  then `MOVE path, destination` and `CREATE FILE destination + ".meta.txt" WITH …`
  recording the category, the kind and the caption. A failed `MOVE` means a
  parallel runner already filed the item: `TALK` that and `RETURN`. Never let a
  category name a folder outside the taxonomy and never file outside `media/`.
- Keep it under 40 lines and make it self-contained: the whole program goes
  into `basic_program` as one string with real newlines (\n). Do not put
  comments or blank lines inside `basic_program`: every line is budget.
- The plan is bookkeeping, not the deliverable: at most 3 steps, each with a
  3-5 word name and a one-line description. Spend the budget on the program —
  a verbose plan gets the whole answer cut off.

{reference}

Output rules:
- Answer with the JSON object and nothing else. No analysis, no commentary, no
  markdown fences, no explanation before or after the object.
- Do not restate the request, do not explore alternatives, do not ask questions.

Respond with JSON only:
{{
  "plan_name": "short name",
  "plan_description": "what will be done",
  "steps": [
    {{
      "name": "step name",
      "description": "what this step does",
      "keywords": ["keyword1"],
      "priority": "high|medium|low",
      "risk_level": "high|medium|low",
      "estimated_minutes": 5,
      "requires_approval": false
    }}
  ],
  "alternatives": [],
  "confidence": 0.85,
  "risk_level": "low",
  "estimated_duration_minutes": 10,
  "estimated_cost": 0.01,
  "resource_estimate": {{
    "compute_hours": 0.1,
    "storage_gb": 0.01,
    "api_calls": 5,
    "llm_tokens": 1000,
    "estimated_cost_usd": 0.01
  }},
  "basic_program": "SET answer = 1\nTALK \"done: \" + answer",
  "requires_approval": false,
  "mcp_servers": [],
  "external_apis": [],
  "risks": []
}}"#,
        reference = reference
    )
}
