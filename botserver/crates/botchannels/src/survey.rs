use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsatSurvey {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub bot_id: Uuid,
    pub score: u32,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsatStats {
    pub total_responses: u64,
    pub average_score: f64,
    pub distribution: CsatDistribution,
    pub response_rate: f64,
    pub bot_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsatDistribution {
    pub score_1: u64,
    pub score_2: u64,
    pub score_3: u64,
    pub score_4: u64,
    pub score_5: u64,
}

impl CsatDistribution {
    pub fn new() -> Self {
        Self {
            score_1: 0,
            score_2: 0,
            score_3: 0,
            score_4: 0,
            score_5: 0,
        }
    }

    pub fn total(&self) -> u64 {
        self.score_1 + self.score_2 + self.score_3 + self.score_4 + self.score_5
    }
}

impl Default for CsatDistribution {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum SurveyError {
    InvalidScore(u32),
    DuplicateResponse(Uuid),
    StorageError(String),
}

impl std::fmt::Display for SurveyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidScore(score) => write!(f, "CSAT score {score} is invalid (must be 1-5)"),
            Self::DuplicateResponse(cid) => {
                write!(f, "Survey already exists for conversation {cid}")
            }
            Self::StorageError(msg) => write!(f, "Survey storage error: {msg}"),
        }
    }
}

impl std::error::Error for SurveyError {}

pub struct SurveyService {
    surveys: Vec<CsatSurvey>,
}

impl SurveyService {
    pub fn new() -> Self {
        Self {
            surveys: Vec::new(),
        }
    }

    pub fn send_after_conversation(
        &mut self,
        conversation_id: Uuid,
        bot_id: Uuid,
        score: u32,
        comment: Option<String>,
    ) -> Result<CsatSurvey, SurveyError> {
        if !(1..=5).contains(&score) {
            return Err(SurveyError::InvalidScore(score));
        }

        if self.surveys.iter().any(|s| s.conversation_id == conversation_id) {
            return Err(SurveyError::DuplicateResponse(conversation_id));
        }

        let survey = CsatSurvey {
            id: Uuid::new_v4(),
            conversation_id,
            bot_id,
            score,
            comment,
            created_at: Utc::now(),
        };

        self.surveys.push(survey.clone());
        Ok(survey)
    }

    pub fn get_csat_stats(&self, bot_id: Uuid) -> CsatStats {
        let bot_surveys: Vec<_> = self.surveys.iter().filter(|s| s.bot_id == bot_id).collect();
        let total = bot_surveys.len() as u64;

        if total == 0 {
            return CsatStats {
                total_responses: 0,
                average_score: 0.0,
                distribution: CsatDistribution::new(),
                response_rate: 0.0,
                bot_id,
            };
        }

        let mut dist = CsatDistribution::new();
        let mut sum: u64 = 0;

        for survey in &bot_surveys {
            sum += survey.score as u64;
            match survey.score {
                1 => dist.score_1 += 1,
                2 => dist.score_2 += 1,
                3 => dist.score_3 += 1,
                4 => dist.score_4 += 1,
                5 => dist.score_5 += 1,
                _ => {}
            }
        }

        CsatStats {
            total_responses: total,
            average_score: sum as f64 / total as f64,
            distribution: dist,
            response_rate: 0.0,
            bot_id,
        }
    }

    pub fn list_surveys(&self) -> &[CsatSurvey] {
        &self.surveys
    }

    pub fn get_by_conversation(&self, conversation_id: Uuid) -> Option<&CsatSurvey> {
        self.surveys
            .iter()
            .find(|s| s.conversation_id == conversation_id)
    }
}

impl Default for SurveyService {
    fn default() -> Self {
        Self::new()
    }
}
