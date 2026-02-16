use thiserror::Error;

#[derive(Error, Debug)]
pub enum PollError {
    #[error("Poll question is required")]
    EmptyQuestion,
    #[error("Poll requires at least 2 options")]
    TooFewOptions,
    #[error("Poll supports at most {0} options")]
    TooManyOptions(usize),
    #[error("maxSelections must be at least 1")]
    MinSelections,
    #[error("maxSelections cannot exceed option count")]
    MaxSelectionsExceeded,
    #[error("durationSeconds must be at least 1")]
    MinDurationSeconds,
    #[error("durationHours must be at least 1")]
    MinDurationHours,
    #[error("durationSeconds and durationHours are mutually exclusive")]
    MutuallyExclusiveDuration,
}

#[derive(Debug, Clone)]
pub struct PollInput {
    pub question: String,
    pub options: Vec<String>,
    pub max_selections: Option<u32>,
    pub duration_seconds: Option<u32>,
    pub duration_hours: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedPollInput {
    pub question: String,
    pub options: Vec<String>,
    pub max_selections: u32,
    pub duration_seconds: Option<u32>,
    pub duration_hours: Option<u32>,
}

pub struct NormalizePollOptions {
    pub max_options: Option<usize>,
}

impl Default for NormalizePollOptions {
    fn default() -> Self {
        Self { max_options: None }
    }
}

pub fn normalize_poll_input(
    input: &PollInput,
    options: &NormalizePollOptions,
) -> Result<NormalizedPollInput, PollError> {
    let question = input.question.trim().to_string();
    if question.is_empty() {
        return Err(PollError::EmptyQuestion);
    }

    let cleaned: Vec<String> = input.options.iter()
        .map(|o| o.trim().to_string())
        .filter(|o| !o.is_empty())
        .collect();

    if cleaned.len() < 2 {
        return Err(PollError::TooFewOptions);
    }

    if let Some(max) = options.max_options {
        if cleaned.len() > max {
            return Err(PollError::TooManyOptions(max));
        }
    }

    let max_selections = input.max_selections.unwrap_or(1);
    if max_selections < 1 {
        return Err(PollError::MinSelections);
    }
    if max_selections as usize > cleaned.len() {
        return Err(PollError::MaxSelectionsExceeded);
    }

    let duration_seconds = input.duration_seconds;
    if let Some(ds) = duration_seconds {
        if ds < 1 {
            return Err(PollError::MinDurationSeconds);
        }
    }

    let duration_hours = input.duration_hours;
    if let Some(dh) = duration_hours {
        if dh < 1 {
            return Err(PollError::MinDurationHours);
        }
    }

    if duration_seconds.is_some() && duration_hours.is_some() {
        return Err(PollError::MutuallyExclusiveDuration);
    }

    Ok(NormalizedPollInput {
        question,
        options: cleaned,
        max_selections,
        duration_seconds,
        duration_hours,
    })
}

pub fn normalize_poll_duration_hours(value: Option<u32>, default_hours: u32, max_hours: u32) -> u32 {
    let base = value.unwrap_or(default_hours);
    base.max(1).min(max_hours)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_question_options_and_validates_max_selections() {
        let result = normalize_poll_input(
            &PollInput {
                question: "  Lunch? ".to_string(),
                options: vec![" Pizza ".to_string(), " ".to_string(), "Sushi".to_string()],
                max_selections: Some(2),
                duration_seconds: None,
                duration_hours: None,
            },
            &NormalizePollOptions::default(),
        ).unwrap();

        assert_eq!(result, NormalizedPollInput {
            question: "Lunch?".to_string(),
            options: vec!["Pizza".to_string(), "Sushi".to_string()],
            max_selections: 2,
            duration_seconds: None,
            duration_hours: None,
        });
    }

    #[test]
    fn enforces_max_option_count() {
        let result = normalize_poll_input(
            &PollInput {
                question: "Q".to_string(),
                options: vec!["A".to_string(), "B".to_string(), "C".to_string()],
                max_selections: None,
                duration_seconds: None,
                duration_hours: None,
            },
            &NormalizePollOptions { max_options: Some(2) },
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at most 2"));
    }

    #[test]
    fn clamps_poll_duration_with_defaults() {
        assert_eq!(normalize_poll_duration_hours(None, 24, 48), 24);
        assert_eq!(normalize_poll_duration_hours(Some(999), 24, 48), 48);
        assert_eq!(normalize_poll_duration_hours(Some(1), 24, 48), 1);
    }

    #[test]
    fn rejects_both_duration_types() {
        let result = normalize_poll_input(
            &PollInput {
                question: "Q".to_string(),
                options: vec!["A".to_string(), "B".to_string()],
                max_selections: None,
                duration_seconds: Some(60),
                duration_hours: Some(1),
            },
            &NormalizePollOptions::default(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mutually exclusive"));
    }
}
