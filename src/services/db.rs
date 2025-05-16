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



pub fn get_chapter_html_by_index(id: i64, index: usize) -> Result<String> {
    let document = get_document(id)?;
    
    // Parse chapters_html as an array
    let binding = serde_json::from_str::<serde_json::Value>(&document.chapters_html)
         .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Invalid chapters_html JSON: {}", e)))?;
    let chaptersObject = binding
        .as_object()
        .ok_or_else(|| rusqlite::Error::InvalidParameterName("chapters_html is not an object".into()))?;

    let chapters = chaptersObject
        .get("chapters")
        .and_then(|v| v.as_array())
        .ok_or_else(|| rusqlite::Error::InvalidParameterName("chapters is not an array".into()))?;
    
    // Check if index is valid
    if index >= chapters.len() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    
    // Get the HTML text of the chapter at the given index
    let chapter_html = chapters[index]["content"].as_str()
        .ok_or_else(|| rusqlite::Error::InvalidParameterName("Invalid chapter content".into()))?;

    Ok(chapter_html.to_string())
}
