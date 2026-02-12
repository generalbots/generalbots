mod processing;
mod talk;
mod syntax;
mod types;
mod validators;

pub use processing::{process_hear_input, process_audio_to_text, process_qrcode, process_video_description};
pub use talk::{execute_talk, talk_keyword};
pub use syntax::hear_keyword;
pub use types::{InputType, ValidationResult};
pub use validators::validate_input;
