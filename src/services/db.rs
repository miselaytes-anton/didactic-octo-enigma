use rusqlite::{Connection, Result, params};
use serde_json::Value;
use std::path::Path;
use crate::models::metadata::EpubMetadata;

pub struct Document {
    pub id: i64,
    pub metadata: String, // JSON string
    pub chapters_html: String, // JSON string containing HTML chapters
}

pub fn init_db() -> Result<Connection> {
    let db_path = "epub_documents.db";
    let is_new = !Path::new(db_path).exists();
    
    let conn = Connection::open(db_path)?;
    
    if is_new {
        conn.execute(
            "CREATE TABLE documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                metadata TEXT NOT NULL,
                chapters_html TEXT NOT NULL
            )",
            [],
        )?;
    }
    
    Ok(conn)
}

pub fn save_document(metadata: &EpubMetadata, chapters_html: &Value) -> Result<i64> {
    let conn = init_db()?;
    
    let metadata_json = serde_json::to_value(metadata)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    
    conn.execute(
        "INSERT INTO documents (metadata, chapters_html) VALUES (?1, ?2)",
        params![
            metadata_json.to_string(),
            chapters_html.to_string(),
        ],
    )?;
    
    Ok(conn.last_insert_rowid())
}

pub fn get_document(id: i64) -> Result<Document> {
    let conn = init_db()?;
    
    let mut stmt = conn.prepare("SELECT id, metadata, chapters_html FROM documents WHERE id = ?1")?;
    let document = stmt.query_row(params![id], |row| {
        Ok(Document {
            id: row.get(0)?,
            metadata: row.get(1)?,
            chapters_html: row.get(2)?,
        })
    })?;
    
    Ok(document)
}

pub fn get_chapter_html(id: i64, chapter_path: &str) -> Result<String> {
    let document = get_document(id)?;
    
    // Parse the chapter HTML JSON
    let chapters_html: serde_json::Value = serde_json::from_str(&document.chapters_html)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid JSON: {}", e)))?;
    
    // Get the HTML for the specific chapter
    match chapters_html.get(chapter_path) {
        Some(html) => {
            if let Some(html_str) = html.as_str() {
                Ok(html_str.to_string())
            } else {
                Err(rusqlite::Error::InvalidParameterName(
                    "Chapter HTML is not a string".to_string()
                ))
            }
        },
        None => Err(rusqlite::Error::QueryReturnedNoRows),
    }
}

pub fn get_chapter_html_by_index(id: i64, index: usize) -> Result<String> {
    let document = get_document(id)?;
    
    // Parse metadata to get chapter path at the given index
    let metadata: serde_json::Value = serde_json::from_str(&document.metadata)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid metadata JSON: {}", e)))?;
    
    let chapters = serde_json::Value = serde_json::from_str(&document.chapters_html)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid chapters JSON: {}", e)))?;
    
    // Check if index is valid
    if index >= chapters.len() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    
    // Get the path of the chapter at the given index
    let chapter_path = chapters[index]["path"].as_str()
        .ok_or_else(|| rusqlite::Error::InvalidParameterName("Invalid chapter path".into()))?;
    
    // Now use the path to get the HTML content
    get_chapter_html(id, chapter_path)
}
