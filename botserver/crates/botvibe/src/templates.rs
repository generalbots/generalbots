//! Built-in project templates — the Vibe codebase owns the starter content it
//! seeds into a fresh project workspace, so nothing depends on a pre-seeded
//! external tree (the old `/tmp/vibe-workspaces/calculator` fixture).
//!
//! A "calculator" project gets a working arithmetic service: a safe, eval-free
//! expression evaluator (`calc.js`), an HTTP service (`index.js`), a test
//! suite (`test.js`) and a `package.json`. Any other software project gets a
//! minimal README starter. Seeding never clobbers agent output: an already
//! non-empty workspace is left untouched.

const CALCULATOR_CALC_JS: &str = r#"'use strict';
// Safe arithmetic evaluator for the Vibe calculator template.
// Handles + - * / and parentheses; parses without eval().
function tokenize(input) {
  const tokens = [];
  const re = /\s*(\d+(?:\.\d+)?|[-+*/()])/g;
  let m;
  let last = 0;
  while ((m = re.exec(input))) {
    if (m.index !== last) throw new Error('invalid character at position ' + last);
    last = re.lastIndex;
    const tok = m[1];
    tokens.push(/^[-+*/()]$/.test(tok) ? tok : parseFloat(tok));
  }
  if (last !== input.length) throw new Error('invalid character at position ' + last);
  return tokens;
}

function evaluate(expr) {
  const tokens = tokenize(String(expr));
  if (tokens.length === 0) return 0;
  let i = 0;
  const peek = () => tokens[i];
  const next = () => tokens[i++];

  function parseExpr() {
    let value = parseTerm();
    while (peek() === '+' || peek() === '-') {
      const op = next();
      const rhs = parseTerm();
      value = op === '+' ? value + rhs : value - rhs;
    }
    return value;
  }
  function parseTerm() {
    let value = parseFactor();
    while (peek() === '*' || peek() === '/') {
      const op = next();
      const rhs = parseFactor();
      if (op === '/' && rhs === 0) throw new Error('division by zero');
      value = op === '*' ? value * rhs : value / rhs;
    }
    return value;
  }
  function parseFactor() {
    const tok = peek();
    if (tok === '(') {
      next();
      const v = parseExpr();
      if (next() !== ')') throw new Error('unbalanced parentheses');
      return v;
    }
    if (tok === '-') { next(); return -parseFactor(); }
    if (typeof tok === 'number') { next(); return tok; }
    throw new Error('unexpected token: ' + tok);
  }

  const result = parseExpr();
  if (i !== tokens.length) throw new Error('unexpected trailing input');
  return result;
}

module.exports = { evaluate };
"#;

const CALCULATOR_INDEX_JS: &str = r#"'use strict';
const http = require('http');
const { evaluate } = require('./calc');

const port = Number(process.env.PORT || 3000);

function respond(res, code, body) {
  res.writeHead(code, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify(body));
}

const server = http.createServer((req, res) => {
  const url = new URL(req.url, 'http://localhost');
  if (url.pathname === '/health') return respond(res, 200, { status: 'ok' });
  if (url.pathname !== '/' || req.method !== 'GET') return respond(res, 404, { error: 'not found' });
  const expr = url.searchParams.get('expr') || '';
  try {
    const result = evaluate(expr);
    respond(res, 200, { expr, result });
  } catch (err) {
    respond(res, 400, { error: err.message });
  }
});

server.listen(port, () => console.log('calculator listening on http://0.0.0.0:' + port));
"#;

const CALCULATOR_TEST_JS: &str = r#"'use strict';
const assert = require('assert');
const { evaluate } = require('./calc');

const cases = [
  ['2+3', 5],
  ['10-4', 6],
  ['3*4', 12],
  ['20/5', 4],
  ['2+3*4', 14],
  ['(2+3)*4', 20],
  ['-5+2', -3],
  ['1.5*2', 3],
  ['100/0', null], // must throw division by zero
];

let passed = 0;
for (const [expr, want] of cases) {
  if (want === null) {
    assert.throws(() => evaluate(expr), /division by zero/, expr + ' must throw');
  } else {
    assert.strictEqual(evaluate(expr), want, expr + ' => ' + want);
  }
  passed++;
}
console.log('all ' + passed + ' tests passed');
"#;

const CALCULATOR_PACKAGE_JSON: &str = r#"{
  "name": "calculator",
  "version": "1.0.0",
  "description": "Arithmetic calculator service with a safe, eval-free evaluator",
  "main": "index.js",
  "scripts": {
    "start": "node index.js",
    "test": "node test.js"
  },
  "license": "MIT"
}
"#;

const CALCULATOR_README: &str = r#"# Calculator

Arithmetic calculator service owned by Vibe's built-in project templates.

- `calc.js` — safe, eval-free expression evaluator (+, -, *, /, parentheses)
- `index.js` — HTTP service: `GET /?expr=2%2B3*4`
- `test.js` — test suite (`npm test`)
- `/health` — liveness probe

Run locally: `npm start` (PORT env, default 3000).
"#;

const STARTER_README: &str = r#"# Project

This project was scaffolded by Vibe's built-in project template.

Use the file tools to add source code, then `project_test` to verify and
`publish/project` to deploy it to an environment.
"#;

/// True when the project name asks for a calculator-style app.
pub fn is_calculator_project(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("calculator") || n.contains("calc")
}

/// Seed the project workspace at `workspace_root()/{key}` with the built-in
/// starter (or calculator) template. No-op when the workspace already has
/// files (the agent may have started working).
pub fn seed_project_workspace(key: &str, project_name: &str) -> Result<(), String> {
    let key = crate::harness::sanitize_project_id(key)?;
    let dir = crate::harness::workspace_root().join(&key);
    if dir.is_dir() {
        let mut entries =
            std::fs::read_dir(&dir).map_err(|e| format!("read workspace {key}: {e}"))?;
        if entries.next().is_some() {
            return Ok(());
        }
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("create workspace {key}: {e}"))?;

    let calculator = is_calculator_project(project_name);
    if calculator {
        crate::harness::write_rel_file(&key, "calc.js", CALCULATOR_CALC_JS.as_bytes())?;
        crate::harness::write_rel_file(&key, "index.js", CALCULATOR_INDEX_JS.as_bytes())?;
        crate::harness::write_rel_file(&key, "test.js", CALCULATOR_TEST_JS.as_bytes())?;
        crate::harness::write_rel_file(&key, "package.json", CALCULATOR_PACKAGE_JSON.as_bytes())?;
    }
    let readme = if calculator {
        CALCULATOR_README
    } else {
        STARTER_README
    };
    crate::harness::write_rel_file(&key, "README.md", readme.as_bytes())?;

    log::info!(
        "Vibe: seeded workspace '{key}' with {} template",
        if calculator { "calculator" } else { "starter" }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculator_detection() {
        assert!(is_calculator_project("calculator"));
        assert!(is_calculator_project("My Calculator App"));
        assert!(is_calculator_project("calc"));
        assert!(!is_calculator_project("landing-page"));
        assert!(!is_calculator_project(""));
    }

    #[test]
    fn seed_seeds_calculator_and_never_clobbers() {
        let tmp = std::env::temp_dir().join(format!("vibe-templates-test-{}", uuid::Uuid::new_v4()));
        std::env::set_var("VIBE_WORKSPACE_ROOT", &tmp);
        let key = "calculator";

        seed_project_workspace(key, "Calculator").expect("seed");
        let entries = crate::harness::list_rel(key, "", 0).expect("list");
        for want in ["calc.js", "index.js", "test.js", "package.json", "README.md"] {
            assert!(entries.iter().any(|e| e == want), "missing {want}: {entries:?}");
        }
        let calc = crate::harness::read_rel_file(key, "calc.js", 1024 * 1024).expect("read calc.js");
        let calc = String::from_utf8(calc).expect("utf8");
        assert!(calc.contains("evaluate"), "calc.js must export evaluate");

        // Second seed is a no-op (workspace already populated).
        seed_project_workspace(key, "Calculator").expect("re-seed");
        let after = crate::harness::list_rel(key, "", 0).expect("list after");
        assert_eq!(after.len(), entries.len());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn seed_starter_for_non_calculator() {
        let tmp = std::env::temp_dir().join(format!("vibe-templates-test-{}", uuid::Uuid::new_v4()));
        std::env::set_var("VIBE_WORKSPACE_ROOT", &tmp);
        let key = "landing";

        seed_project_workspace(key, "Landing Page").expect("seed");
        let entries = crate::harness::list_rel(key, "", 0).expect("list");
        assert!(entries.iter().any(|e| e == "README.md"));
        assert!(!entries.iter().any(|e| e == "calc.js"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
