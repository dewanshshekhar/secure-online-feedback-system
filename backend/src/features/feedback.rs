use serde::{Deserialize, Serialize};

pub const MAX_MESSAGE_CHARS: usize = 2_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeedbackCategory {
    Teaching,
    CourseContent,
    Facilities,
    Other,
}

impl FeedbackCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Teaching => "teaching",
            Self::CourseContent => "course-content",
            Self::Facilities => "facilities",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeedbackSubmission {
    pub category: FeedbackCategory,
    pub message: String,
}

#[derive(Debug, PartialEq)]
pub struct ValidatedFeedback {
    pub category: FeedbackCategory,
    pub message: String,
}

impl FeedbackSubmission {
    pub fn validate(self) -> Result<ValidatedFeedback, ValidationError> {
        let message = self.message.trim().to_owned();
        if message.is_empty() {
            return Err(ValidationError::EmptyMessage);
        }
        if message.chars().count() > MAX_MESSAGE_CHARS {
            return Err(ValidationError::MessageTooLong);
        }
        Ok(ValidatedFeedback {
            category: self.category,
            message,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    EmptyMessage,
    MessageTooLong,
}

impl ValidationError {
    pub const fn message(&self) -> &'static str {
        match self {
            Self::EmptyMessage => "message must not be empty",
            Self::MessageTooLong => "message must be 2,000 characters or fewer",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_a_valid_message() {
        let feedback = FeedbackSubmission {
            category: FeedbackCategory::Teaching,
            message: "  Helpful examples  ".into(),
        };
        assert_eq!(
            feedback.validate().unwrap(),
            ValidatedFeedback {
                category: FeedbackCategory::Teaching,
                message: "Helpful examples".into(),
            }
        );
    }

    #[test]
    fn rejects_empty_or_oversized_messages() {
        let empty = FeedbackSubmission {
            category: FeedbackCategory::Other,
            message: " \n ".into(),
        };
        assert_eq!(empty.validate(), Err(ValidationError::EmptyMessage));

        let oversized = FeedbackSubmission {
            category: FeedbackCategory::Other,
            message: "a".repeat(MAX_MESSAGE_CHARS + 1),
        };
        assert_eq!(oversized.validate(), Err(ValidationError::MessageTooLong));
    }
}
