//! Writes the run event JSON Schema to stdout.
//!
//! Usage: `cargo run -p botevents --example dump_schema > schemas/run-event.v1.json`

fn main() {
    let schema = botevents::envelope_schema();
    match serde_json::to_string_pretty(&schema) {
        Ok(text) => println!("{text}"),
        Err(error) => {
            eprintln!("could not serialize schema: {error}");
            std::process::exit(1);
        }
    }
}
