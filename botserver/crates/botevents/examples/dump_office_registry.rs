//! Writes the Office event schema registry (one example per event) to stdout.
//!
//! Usage: `cargo run -p botevents --example dump_office_registry`

fn main() {
    let registry = botevents::registry_json();
    match serde_json::to_string_pretty(&registry) {
        Ok(text) => println!("{text}"),
        Err(error) => {
            eprintln!("could not serialize registry: {error}");
            std::process::exit(1);
        }
    }
}
