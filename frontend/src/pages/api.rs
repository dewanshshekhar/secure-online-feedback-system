use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Receipt {
    pub id: String,
}

#[derive(Clone, Deserialize)]
pub struct FeedbackRecord {
    pub id: String,
    pub category: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Clone, Deserialize)]
pub struct FeedbackPage {
    pub items: Vec<FeedbackRecord>,
    pub has_more: bool,
    pub next_offset: Option<usize>,
}

pub fn category_label(category: &str) -> &'static str {
    match category {
        "teaching" => "Teaching",
        "course-content" => "Course content",
        "facilities" => "Facilities",
        _ => "Something else",
    }
}
