use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpubMetadata {
    pub title: String,
    pub author: String,
    pub publication_date: Option<String>,
    pub language: Option<String>,
    pub description: Option<String>,
    pub chapters: Vec<Chapter>,
}

impl EpubMetadata {
    pub fn new(
        title: String, 
        author: String, 
        publication_date: Option<String>, 
        language: Option<String>, 
        description: Option<String>,
        chapters: Vec<Chapter>
    ) -> Self {
        EpubMetadata {
            title,
            author,
            publication_date,
            language,
            description,
            chapters,
        }
    }
}