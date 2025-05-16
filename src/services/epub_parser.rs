use std::io::Cursor;
use epub::doc::EpubDoc;
use crate::models::metadata::{EpubMetadata, Chapter};

pub struct EpubContent {
    pub metadata: EpubMetadata,
    pub html_content: Vec<(String, String)>, // (path, HTML content)
}

pub fn parse_epub(data: &[u8]) -> Result<EpubContent, String> {
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
    let mut html_content = Vec::new();
    
    // Get chapters from the spine (list of content documents in reading order)
    for i in 0..doc.spine.len() {
        // Get chapter number as a fallback title
        let chapter_title = format!("Chapter {}", i + 1);
        let spine_id = doc.spine[i].clone();
        
        // Try to get the HTML content for this chapter
        // Try different path combinations since EPUB files can have different structures
        let possible_paths = vec![
            spine_id.clone(),                              // Original path
            format!("OEBPS/Text/{}", spine_id),            // Path with OEBPS/Text prefix
            format!("OEBPS/{}", spine_id),                 // Path with OEBPS prefix
            format!("Text/{}", spine_id),                  // Path with Text prefix
        ];
        
        let mut content_found = false;
        
        for path in &possible_paths {
            if content_found {
                break;
            }
            
            // Method 1: Try get_resource_str_by_path
            if let Some(content) = doc.get_resource_str_by_path(path) {
                println!("Found HTML content for {} using path: {}", spine_id, path);
                html_content.push((spine_id.clone(), content));
                content_found = true;
                continue;
            }
            
            // Method 2: Try get_resource_by_path and convert to string
            if let Some(data) = doc.get_resource_by_path(path) {
                match String::from_utf8(data.clone()) {
                    Ok(content) => {
                        println!("Found HTML content for {} using get_resource_by_path with: {}", spine_id, path);
                        html_content.push((spine_id.clone(), content));
                        content_found = true;
                        continue;
                    }
                    Err(_) => {
                        println!("Content for {} with path {} is not valid UTF-8", spine_id, path);
                    }
                }
            }
        }
        
        if !content_found {
            // Log that we couldn't find content despite trying multiple paths
            println!("No HTML content found for {} after trying multiple paths", spine_id);
        }
        
        // Push a basic chapter with the spine ID as the path
        chapters.push(Chapter {
            title: chapter_title,
            path: spine_id,
        });
    }
    
    Ok(EpubContent {
        metadata: EpubMetadata {
            title,
            author,
            publication_date,
            language,
            description,
            chapters,
        },
        html_content,
    })
}
