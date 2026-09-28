//! BASIC reference for code generation: the closed keyword catalog *with
//! signatures*.
//!
//! Names alone were not enough. Asked to write BASIC from a keyword list, the
//! model produced `SEND MAIL TO x SUBJECT y BODY z` and `GET USER MEMORY` —
//! neither exists (the real form is `SEND MAIL to, subject, body, attachments`
//! with four positional arguments, and the memory keyword is `GET BOT MEMORY`),
//! so the generated `.bas` could not compile. This module renders the catalog
//! plus the exact calling convention of the keywords a generated automation
//! realistically uses; anything not listed must not be emitted.

use super::get_all_keywords;

/// `(keyword, signature)` for the keywords worth generating. Kept short on
/// purpose: a long, wrong list is worse than a short, right one, and the
/// instruction below forbids anything outside this list.
const SIGNATURES: &[(&str, &str)] = &[
    ("TALK", r#"TALK "text""#),
    ("PRINT", r#"PRINT value"#),
    ("SET", r#"SET name = expression"#),
    ("GET", r#"GET FROM table WHERE condition"#),
    ("SAVE", r#"SAVE record TO table"#),
    ("FIND", r#"FIND value IN table"#),
    ("FIRST", r#"FIRST(list)"#),
    ("LAST", r#"LAST(list)"#),
    ("COUNT", r#"COUNT(list)"#),
    ("FILTER", r#"FILTER list, "field", "value""#),
    ("GROUP BY", r#"GROUP BY list, "field""#),
    ("AGGREGATE", r#"AGGREGATE list, "field", "sum""#),
    ("FORMAT", r#"FORMAT "template {0} {1}", a, b"#),
    ("SPLIT", r#"SPLIT("text", ",")"#),
    ("TRIM", r#"TRIM("text")"#),
    ("UPPER", r#"UPPER("text")"#),
    ("LEN", r#"LEN(text)"#),
    ("INSTR", r#"INSTR(text, "needle")"#),
    ("REPLACE", r#"REPLACE(text, "from", "to")"#),
    ("LEFT", r#"LEFT(text, 10)"#),
    ("STR", r#"STR(value)"#),
    ("TODAY", "TODAY (a map: TODAY.year, TODAY.month)"),
    ("NOW", "NOW"),
    ("WAIT", "WAIT seconds"),
    ("CREATE FILE", r#"CREATE FILE "path" WITH content"#),
    ("WRITE FILE", r#"WRITE FILE "path" WITH content"#),
    ("READ FILE", r#"content = READ FILE "path""#),
    ("DELETE FILE", r#"DELETE FILE "path""#),
    ("MOVE", r#"MOVE "old/path", "new/path""#),
    ("COPY", r#"COPY "from", "to""#),
    ("LIST FILES", r#"LIST FILES "folder""#),
    ("GET FILE", r#"GET FILE "path""#),
    ("UPLOAD", r#"UPLOAD data TO "path""#),
    ("GET HTTP", r#"GET HTTP "url""#),
    ("POST HTTP", r#"POST HTTP "url" WITH data"#),
    ("PUT HTTP", r#"PUT HTTP "url" WITH data"#),
    ("DELETE HTTP", r#"DELETE HTTP "url""#),
    ("WEBHOOK", r#"WEBHOOK "url" WITH data"#),
    ("SEND MAIL", r#"SEND MAIL "to@x.com", "subject", "body", "attachments""#),
    ("SEND SMS", r#"SEND SMS TO "+5511999999999" MESSAGE "text""#),
    ("SEND TO", r#"SEND TO "target" MESSAGE "text""#),
    ("SET BOT MEMORY", r#"SET BOT MEMORY "key" = value"#),
    ("GET BOT MEMORY", r#"value = GET BOT MEMORY "key""#),
    ("SET CONTEXT", r#"SET CONTEXT "key" = value"#),
    ("REMEMBER", r#"REMEMBER "key" = value"#),
    ("RECALL", r#"value = RECALL "key""#),
    ("USE TOOL", r#"USE TOOL "tool_name""#),
    ("CLEAR TOOLS", "CLEAR TOOLS"),
    ("USE KB", r#"USE KB "knowledge_base""#),
    ("CLEAR KB", "CLEAR KB"),
    ("USE WEBSITE", r#"USE WEBSITE "url""#),
    ("ADD SUGGESTION", r#"ADD SUGGESTION "reply the user can tap""#),
    ("CLEAR SUGGESTIONS", "CLEAR SUGGESTIONS"),
    ("DESCRIBE IMAGE", r#"text = DESCRIBE IMAGE "inbox/x.jpg""#),
    ("DESCRIBE VIDEO", r#"text = DESCRIBE VIDEO "inbox/x.mp4""#),
    ("SPEECH TO TEXT", r#"text = SPEECH TO TEXT "inbox/x.ogg""#),
    ("LLM", r#"answer = LLM "prompt""#),
    ("ON EVENT", r#"ON EVENT "media_uploaded" CALL "tool_name""#),
    ("CREATE TASK", r#"CREATE TASK "title", "assignee", "due", project"#),
    ("TRANSFER TO HUMAN", "TRANSFER TO HUMAN"),
    ("SET SCHEDULE", r#"SET SCHEDULE "0 9 * * 1"  ' cron, set by the platform for scheduled intents"#),
];

/// Signature for a keyword, when one is known.
pub fn signature_of(keyword: &str) -> Option<&'static str> {
    SIGNATURES
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(keyword))
        .map(|(_, signature)| *signature)
}

/// The generator reference: the closed catalog with the calling convention of
/// every keyword that has one. Names without a signature are still listed — the
/// model may use them only in the form documented by the runtime help.
pub fn basic_keyword_reference() -> String {
    let mut out = String::from("## BASIC reference\n\n### Keywords with signatures (use exactly this form)\n");
    for (_, signature) in SIGNATURES {
        out.push_str(&format!("- {signature}\n"));
    }
    out.push_str(
        "\nAnything not written above is unavailable: do not invent a keyword, an \
         argument name or an argument order. There is no `EXIT`, no `GET USER MEMORY`, \
         no `SEND MAIL TO … SUBJECT …` form and no `ON CHANGE`.\n",
    );
    out.push_str("\n### Full keyword catalog (closed set)\n");
    let keywords = get_all_keywords();
    for chunk in keywords.chunks(6) {
        out.push_str("- ");
        out.push_str(&chunk.join(", "));
        out.push('\n');
    }
    out.push_str(
        "\n### Control flow\n- `IF condition THEN` … `ELSE` … `END IF`\n\
         - `FOR EACH item IN list` … `NEXT`\n\
         - `WHILE condition` … `WEND`\n\
         - `SWITCH value` / `CASE \"a\"` / `DEFAULT` / `END SWITCH`\n\
         - `ON ERROR RESUME NEXT` … `IF ERROR THEN` … `CLEAR ERROR` … `ON ERROR GOTO 0`\n\
         - `CALL \"procedure\"` to call another script of the bot\n",
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_carries_signatures_and_the_closed_set() {
        let reference = basic_keyword_reference();
        assert!(reference.contains("SEND MAIL \"to@x.com\""));
        assert!(reference.contains("CREATE FILE \"path\" WITH content"));
        assert!(reference.contains("ON EVENT \"media_uploaded\" CALL"));
        // The catalog itself is still exposed…
        assert!(reference.contains("CLASSIFY") || reference.contains("DESCRIBE"));
        // …and the anti-invention rule is stated.
        assert!(reference.contains("do not invent a keyword"));
    }

    #[test]
    fn signature_lookup_is_case_insensitive() {
        assert!(signature_of("send mail").is_some());
        assert!(signature_of("Send Mail").is_some());
        assert!(signature_of("NOT A KEYWORD").is_none());
    }

    #[test]
    fn every_signature_names_a_real_catalog_keyword() {
        // A signature for a keyword the runtime does not register would teach
        // the model to write code that cannot compile.
        let catalog: Vec<String> = get_all_keywords().iter().map(|k| k.to_lowercase()).collect();
        for (name, _) in SIGNATURES {
            let lowered = name.to_lowercase();
            let known = catalog.iter().any(|k| k == &lowered)
                || catalog.iter().any(|k| k.starts_with(&lowered));
            assert!(known, "signature for unknown keyword '{name}'");
        }
    }
}
