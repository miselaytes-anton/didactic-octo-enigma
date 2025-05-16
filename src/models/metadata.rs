use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpubMetadata {
    pub title: String,
    pub author: String,
    pub publication_date: Option<String>,
    pub language: Option<String>,
    pub description: Option<String>,
}

impl EpubMetadata {
    pub fn new(
        title: String,
        author: String,
        publication_date: Option<String>,
        language: Option<String>,
        description: Option<String>,
    ) -> Self {
        EpubMetadata {
            title,
            author,
            publication_date,
            language,
            description,
        }
    }
}
