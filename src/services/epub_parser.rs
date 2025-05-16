use crate::models::metadata::{Chapter, EpubMetadata};
use epub::doc::EpubDoc;
use scraper::{Html, Selector};
use std::io::Cursor;

pub struct EpubContent {
    pub metadata: EpubMetadata,
    pub chapters: Vec<Chapter>,
}

/// Parse an EPUB file from bytes
///
/// This function takes the raw bytes of an EPUB file and extracts:
/// 1. Metadata (title, author, etc.)
/// 2. Chapter information
/// 3. HTML content for each chapter
///
/// Returns an EpubContent struct with all extracted data
pub fn parse_epub(data: &[u8]) -> Result<EpubContent, String> {
    // Create a cursor to read the EPUB data from memory
    let cursor = Cursor::new(data);

    // Parse the EPUB file
    let mut doc =
        EpubDoc::from_reader(cursor).map_err(|e| format!("Failed to parse EPUB: {}", e))?;

    // Extract metadata from the EPUB document
    let title = doc
        .metadata
        .get("title")
        .and_then(|titles| titles.first())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());

    let author = doc
        .metadata
        .get("creator")
        .and_then(|authors| authors.first())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());

    let publication_date = doc
        .metadata
        .get("date")
        .and_then(|dates| dates.first())
        .map(|s| s.to_string());

    let language = doc
        .metadata
        .get("language")
        .and_then(|langs| langs.first())
        .map(|s| s.to_string());

    let description = doc
        .metadata
        .get("description")
        .and_then(|descs| descs.first())
        .map(|s| s.to_string());

    // Extract chapters and HTML content
    let mut chapters = Vec::new();

    //print chapter length
    println!("Number of chapters: {}", doc.spine.len());

    // Get chapters from the spine (list of content documents in reading order)
    // This approach is inspired by epub-chapter-extractor
    for i in 0..doc.spine.len() {
        // Create a chapter title (either from content or fallback to number)
        let chapter_title = format!("Chapter {}", i + 1);
        let spine_id = doc.spine[i].clone();

        // Set current page to the spine index
        if doc.set_current_page(i) {
            // Get content from current page
            if let Some(content) = doc.get_current_str() {
                // print content
                // Add this chapter to our list
                chapters.push(Chapter {
                    title: chapter_title,
                    path: spine_id,
                    content: extract_text_from_html(&content.0),
                });
            }
        }
    }

    // Return the parsed content
    Ok(EpubContent {
        metadata: EpubMetadata {
            title,
            author,
            publication_date,
            language,
            description,
        },
        chapters,
    })
}

/// Helper function to extract plain text content from HTML
///
/// This function removes HTML tags and returns just the text content.
pub fn extract_text_from_html(html: &str) -> String {
    let document = Html::parse_document(html);

    // Get the body element
    let body_selector =
        Selector::parse("body").unwrap_or_else(|_| Selector::parse("html").unwrap());

    if let Some(body) = document.select(&body_selector).next() {
        body.text().collect::<Vec<_>>().join(" ").trim().to_string()
    } else {
        // Fallback: just get all text nodes from the document
        document
            .root_element()
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_parse_epub() {
        // Path to test EPUB file
        let epub_path = Path::new("moby-dick.epub");

        // Read file contents
        let data = fs::read(epub_path).expect("Failed to read test EPUB file");

        // Parse the EPUB file
        let result = parse_epub(&data);

        // Verify the result is Ok
        assert!(result.is_ok(), "Failed to parse EPUB file");

        let epub_content = result.unwrap();

        // Basic assertions on the parsed content
        assert!(
            !epub_content.metadata.title.is_empty(),
            "Title should not be empty"
        );
        assert!(
            !epub_content.metadata.author.is_empty(),
            "Author should not be empty"
        );

        // Print some info about what we found
        println!("EPUB Title: {}", epub_content.metadata.title);
        println!("EPUB Author: {}", epub_content.metadata.author);
        println!("Number of chapters: {}", epub_content.chapters.len());

        for (i, chapter) in epub_content.chapters.iter().enumerate() {
            println!("Chapter {}: {}", i + 1, chapter.content);
        }

        //println!("First HTML content path: {}", epub_content.chapters[0]);
    }
}
