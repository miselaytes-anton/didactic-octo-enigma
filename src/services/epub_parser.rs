use std::io::Cursor;
use epub::doc::EpubDoc;
use crate::models::metadata::{EpubMetadata, Chapter};

pub fn parse_epub(data: &[u8]) -> Result<EpubMetadata, String> {
    // Create a cursor to read the EPUB data from memory
    let cursor = Cursor::new(data);
    
    // Parse the EPUB file
    let mut doc = EpubDoc::from_reader(cursor).map_err(|e| format!("Failed to parse EPUB: {}", e))?;
    
    // Extract metadata from the EPUB document
    let title = doc.metadata.get("title")
        .and_then(|titles| titles.first())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = doc.metadata.get("creator")
        .and_then(|authors| authors.first())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let publication_date = doc.metadata.get("date")
        .and_then(|dates| dates.first())
        .map(|s| s.to_string());
    
    let language = doc.metadata.get("language")
        .and_then(|langs| langs.first())
        .map(|s| s.to_string());
    
    let description = doc.metadata.get("description")
        .and_then(|descs| descs.first())
        .map(|s| s.to_string());
    
    // Extract chapters
    let mut chapters = Vec::new();
    
    // Get chapters from the spine (list of content documents in reading order)
    for i in 0..doc.spine.len() {
        // Get chapter number as a fallback title
        let chapter_title = format!("Chapter {}", i + 1);
        let spine_id = &doc.spine[i];
        
        // Push a basic chapter with the spine ID as the path
        chapters.push(Chapter {
            title: chapter_title,
            path: spine_id.clone(),
        });
    }
    
    Ok(EpubMetadata {
        title,
        author,
        publication_date,
        language,
        description,
        chapters,
    })
}
