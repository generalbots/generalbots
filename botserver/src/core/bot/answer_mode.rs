use std::fmt;
use std::str::FromStr;

pub use super::answer_mode_ops::*;
pub use super::answer_mode_config::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerMode {
    Default,
    Data,
    Chart,
}

impl FromStr for AnswerMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "data" => Ok(AnswerMode::Data),
            "chart" => Ok(AnswerMode::Chart),
            _ => Ok(AnswerMode::Default),
        }
    }
}

impl fmt::Display for AnswerMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AnswerMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnswerMode::Default => "default",
            AnswerMode::Data => "data",
            AnswerMode::Chart => "chart",
        }
    }

    pub fn short_name(&self) -> &'static str {
        self.as_str()
    }
}
