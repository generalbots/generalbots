//! Deterministic property fallback for the fuzz targets (#2119).
//!
//! `fuzz/` needs nightly + libfuzzer, so the deep run is a nightly CI job. This
//! file is the part that runs on every PR: the same four highest-risk parsers
//! are driven with a *deterministic* mutation harness (a seeded xorshift PRNG,
//! no new dependency), so a regression that only shows up on malformed input
//! fails here in seconds instead of waiting for the nightly.
//!
//! Properties asserted per surface — all of them are "fail closed" rules, not
//! "the parser accepts this":
//!
//! * JWT: no token that was not signed by the manager validates or decodes,
//!   and attacker-controlled claims JSON never panics.
//! * webhook: any body/signature pair is parsed without panicking, an empty app
//!   secret never validates a delivery, and verification is deterministic.
//! * BASIC: the source transforms are pure and total — garbage in, same text
//!   out, twice.
//! * office import: any byte sequence either imports or returns `Err`.
//!
//! The mutation corpus is fixed, so a failure is reproducible from the seed
//! printed in the assertion message. When a real crash is minimized by the
//! fuzzer, add its bytes to the matching `SEEDS` slice — that is the
//! "corpus/minimized crashes as regressions" half of #2119.

const ROUNDS: usize = 256;

/// xorshift64*: tiny, deterministic, dependency-free.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, limit: usize) -> usize {
        if limit == 0 {
            0
        } else {
            (self.next() % limit as u64) as usize
        }
    }
}

/// Mutates `seed` into a new byte string: flip, insert, delete or splice.
fn mutate(seed: &[u8], rng: &mut Rng) -> Vec<u8> {
    let alphabet = b"{}[]\"',:=/\\&<>%0123456789abcdefAZ_ \n\t";
    let mut out = seed.to_vec();
    for _ in 0..=rng.below(4) {
        if out.is_empty() {
            out.push(alphabet[rng.below(alphabet.len())]);
            continue;
        }
        match rng.below(4) {
            0 => {
                let at = rng.below(out.len());
                out[at] ^= 1 << rng.below(8);
            }
            1 => {
                let at = rng.below(out.len() + 1);
                out.insert(at, alphabet[rng.below(alphabet.len())]);
            }
            2 => {
                let at = rng.below(out.len());
                out.remove(at);
            }
            _ => {
                let at = rng.below(out.len());
                let len = 1 + rng.below(out.len() - at);
                out.truncate(at + len);
            }
        }
    }
    out
}

fn corpus(seeds: &[&[u8]]) -> Vec<Vec<u8>> {
    let mut rng = Rng::new(0x2119_2119_2119_2119);
    let mut cases = Vec::with_capacity(seeds.len() * ROUNDS);
    for seed in seeds {
        for _ in 0..ROUNDS {
            cases.push(mutate(seed, &mut rng));
        }
    }
    cases
}

const JWT_SEEDS: &[&[u8]] = &[
    b"",
    b"a.b.c",
    b"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwiZXhwIjoxfQ.signature",
    b"eyJhbGciOiJub25lIn0.eyJzdWIiOiJhZG1pbiJ9.",
    br#"{"sub":"admin","exp":1}"#,
];

const WEBHOOK_SEEDS: &[&[u8]] = &[
    b"",
    b"{}",
    br#"{"object":"whatsapp_business_account","entry":[]}"#,
    br#"{"object":"whatsapp_business_account","entry":[{"id":"1","changes":[{"field":"messages","value":{"messaging_product":"whatsapp","messages":[{"from":"5511","id":"wamid","type":"text","text":{"body":"hi"}}]}}]}]}"#,
    b"MessageSid=SM1&AccountSid=AC1&From=whatsapp%3A%2B1&To=%2B2&Body=hi&NumMedia=0",
    b"sha256=deadbeef",
];

const BASIC_SEEDS: &[&[u8]] = &[
    b"",
    b"GOTO 10\n",
    b"10 PRINT \"hello\"\n20 GOTO 10\n",
    b"IF x > 1 THEN TALK \"yes\"\n",
    b"SUB greet(name)\nTALK \"hi \" + name\nEND SUB\n",
    b"<LLM> summarize this </LLM>",
    b"' comment only\n",
];

const OFFICE_SEEDS: &[&[u8]] = &[
    b"",
    b"PK\x03\x04",
    b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1",
    b"a,b\n1,2\n",
    b"col1\tcol2\n1\t2\n",
    b"<?xml version=\"1.0\"?><workbook/>",
];

const SECRET: &str = "fuzz-secret-key-that-is-32-chars!!";

fn manager() -> botsecurity_auth::jwt::JwtManager {
    botsecurity_auth::jwt::JwtManager::from_secret(SECRET)
        .expect("the fixed fuzz secret must be usable")
}

#[test]
fn jwt_tokens_never_validate_without_the_key() {
    let manager = manager();
    for case in corpus(JWT_SEEDS) {
        let token = String::from_utf8_lossy(&case);
        assert!(
            manager.validate_access_token(&token).is_err(),
            "unsigned token validated: {token:?}"
        );
        assert!(
            manager.validate_token(&token).is_err(),
            "unsigned token decoded: {token:?}"
        );
        let _ = serde_json::from_str::<botsecurity_auth::jwt::Claims>(&token);
        let _ = serde_json::from_str::<serde_json::Value>(&token);
    }
}

#[test]
fn whatsapp_webhook_bodies_never_panic() {
    for case in corpus(WEBHOOK_SEEDS) {
        let body = String::from_utf8_lossy(&case);
        let _ = serde_json::from_str::<botwhatsapp::models::WhatsAppWebhookPayload>(&body);
        assert!(
            !botwhatsapp::utils::verify_webhook_signature(&case, &body, ""),
            "an empty app secret validated a delivery"
        );
        let first = botwhatsapp::utils::verify_webhook_signature(&case, &body, "app-secret");
        let second = botwhatsapp::utils::verify_webhook_signature(&case, &body, "app-secret");
        assert_eq!(first, second, "signature verification is not deterministic");
    }
}

#[test]
fn basic_transforms_are_deterministic_on_arbitrary_source() {
    for case in corpus(BASIC_SEEDS) {
        let source = String::from_utf8_lossy(&case);
        let goto = botbasic_compiler::goto_transform::transform_goto(&source);
        assert_eq!(goto, botbasic_compiler::goto_transform::transform_goto(&source));
        let if_then = botbasic_compiler::syntax_transforms::convert_if_then_syntax(&source);
        assert_eq!(
            if_then,
            botbasic_compiler::syntax_transforms::convert_if_then_syntax(&source)
        );
        let unquoted = botbasic_compiler::syntax_transforms::preprocess_llm_keyword(&source);
        assert_eq!(
            unquoted,
            botbasic_compiler::syntax_transforms::preprocess_llm_keyword(&source)
        );
        let _ = botbasic_compiler::goto_transform::has_goto_constructs(&source);
    }
}

#[test]
fn spreadsheet_import_never_panics() {
    for case in corpus(OFFICE_SEEDS) {
        for name in ["fuzz.xlsx", "fuzz.xls", "fuzz.ods", "fuzz.csv", "fuzz"] {
            let _ = botsheet::storage::import::import_spreadsheet_bytes(&case, name, "fuzz");
        }
    }
}
